// Test support for frozen audit identities and original-source preservation.

fn audit_repair_ledger() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "fixtures/inner_tomi_audit_repair_source_trace.json"
    ))
    .unwrap()
}

fn portable_audit_evidence() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "fixtures/inner_tomi_portable_audit_evidence.json"
    ))
    .unwrap()
}

fn evidence_string<'a>(value: &'a serde_json::Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("missing or invalid evidence {key}: {value}"))
}

fn evidence_array<'a>(
    value: &'a serde_json::Value,
    key: &str,
) -> Result<&'a Vec<serde_json::Value>> {
    value[key]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("missing evidence array {key}"))
}

fn evidence_digest(reference: &serde_json::Value, key: &str) -> Result<String> {
    let digest = evidence_string(reference, key)?;
    anyhow::ensure!(
        digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()),
        "malformed evidence digest {key}: {digest}"
    );
    Ok(digest.to_ascii_lowercase())
}

fn normalized_evidence_path(path: &str) -> Result<String> {
    let path = path.replace('\\', "/");
    anyhow::ensure!(
        !path.is_empty() && !path.starts_with('/') && !path.contains(':'),
        "escaping evidence path: {path}"
    );
    let parts: Vec<_> = path.split('/').collect();
    anyhow::ensure!(
        parts
            .iter()
            .all(|p| !p.is_empty() && *p != ".." && *p != "."),
        "escaping evidence path: {path}"
    );
    Ok(path)
}

fn resolve_evidence_path(root: &std::path::Path, logical_path: &str) -> Result<std::path::PathBuf> {
    let logical = normalized_evidence_path(logical_path)?;
    let canonical_root = root
        .canonicalize()
        .map_err(|e| anyhow::anyhow!("missing evidence root {}: {e}", root.display()))?;
    let path = canonical_root.join(&logical);
    let resolved = path.canonicalize().map_err(|e| {
        anyhow::anyhow!(
            "missing historical evidence: logical {logical}, resolved {}: {e}",
            path.display()
        )
    })?;
    anyhow::ensure!(
        resolved.starts_with(&canonical_root),
        "escaping evidence path: logical {logical}, resolved {} outside {}",
        resolved.display(),
        canonical_root.display()
    );
    Ok(resolved)
}

fn read_verified_evidence_bytes(
    root: &std::path::Path,
    reference: &serde_json::Value,
) -> Result<Vec<u8>> {
    Ok(authenticate_evidence_file(root, reference, true)?.0)
}

fn authenticate_evidence_file(
    root: &std::path::Path,
    reference: &serde_json::Value,
    retain: bool,
) -> Result<(Vec<u8>, u64)> {
    use sha2::Digest;
    use std::io::Read;
    let logical = evidence_string(reference, "path")?;
    let expected = evidence_digest(reference, "sha256")?;
    let resolved = resolve_evidence_path(root, logical)?;
    let mut reader = std::io::BufReader::new(std::fs::File::open(&resolved)?);
    let mut hash = sha2::Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut bytes = Vec::new();
    let mut size = 0;
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
        size += count as u64;
        if retain {
            bytes.extend_from_slice(&buffer[..count]);
        }
    }
    let actual = format!("{:x}", hash.finalize());
    anyhow::ensure!(actual == expected,
        "evidence hash mismatch: logical {logical}, resolved {}, expected {expected}, actual {actual}", resolved.display());
    Ok((bytes, size))
}

struct AuditEvidenceRoot {
    root: std::path::PathBuf,
}

impl AuditEvidenceRoot {
    fn from_environment() -> Self {
        Self {
            root: std::env::var_os("PLUTARCH_AUDIT_EVIDENCE_ROOT")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").into()),
        }
    }

    fn resolve(&self, logical_path: &str) -> Result<std::path::PathBuf> {
        resolve_evidence_path(&self.root, logical_path)
    }

    fn read_verified(&self, reference: &serde_json::Value) -> Result<Vec<u8>> {
        read_verified_evidence_bytes(&self.root, reference)
    }

    fn read_verified_json(&self, reference: &serde_json::Value) -> Result<serde_json::Value> {
        Ok(serde_json::from_slice(&self.read_verified(reference)?)?)
    }

    fn verify_streamed(&self, reference: &serde_json::Value) -> Result<u64> {
        Ok(authenticate_evidence_file(&self.root, reference, false)?.1)
    }
}

fn authenticated_audit_documents(
    root: &AuditEvidenceRoot,
    ledger: &serde_json::Value,
    kinds: &[AuditDocumentKind],
) -> Result<serde_json::Value> {
    let mut documents = serde_json::Map::new();
    for (path, (reference, kind)) in required_audit_references(ledger)? {
        if kinds.contains(&kind) {
            documents.insert(
                path,
                serde_json::json!({"source_sha256":reference["sha256"],
                "value": root.read_verified_json(&reference)?}),
            );
        }
    }
    Ok(serde_json::json!({"schema_version":1,"documents":documents}))
}

fn archive_manifest_reference(ledger: &serde_json::Value, key: &str) -> Result<serde_json::Value> {
    let archive = &ledger[key];
    Ok(
        serde_json::json!({"path":format!("{}/manifest.json", evidence_string(archive,"path")?),
        "sha256":evidence_digest(archive,"manifest_sha256")?}),
    )
}

fn authenticate_evidence_manifest(
    root: &AuditEvidenceRoot,
    reference: &serde_json::Value,
    pinned: &serde_json::Value,
    seen: &mut std::collections::BTreeMap<String, String>,
) -> Result<(u64, u64)> {
    let manifest = root.read_verified_json(reference)?;
    let logical = normalized_evidence_path(evidence_string(reference, "path")?)?;
    let archive_directory = logical.rsplit_once('/').map(|(dir, _)| dir).unwrap_or("");
    let is_stage = manifest.is_array();
    let entries = if is_stage {
        manifest.as_array().unwrap()
    } else {
        evidence_array(&manifest, "files")?
    };
    let mut total = 0;
    for entry in entries {
        let relative = normalized_evidence_path(evidence_string(entry, "path")?)?;
        let path = if is_stage || archive_directory.is_empty() {
            relative
        } else {
            format!("{archive_directory}/{relative}")
        };
        let hash = evidence_digest(entry, "sha256")?;
        anyhow::ensure!(
            !seen.contains_key(&path),
            "duplicate or conflicting manifest path: {path}"
        );
        seen.insert(path.clone(), hash.clone());
        let bytes = root.verify_streamed(&serde_json::json!({"path":path,"sha256":hash}))?;
        for key in ["bytes", "size"] {
            if let Some(size) = entry.get(key) {
                anyhow::ensure!(
                    size.as_u64() == Some(bytes),
                    "declared manifest size differs: {path}"
                );
            }
        }
        total += bytes;
    }
    let count = entries.len() as u64;
    for metadata in [&manifest, pinned] {
        for key in ["file_count", "verified_files"] {
            if let Some(expected) = metadata.get(key) {
                anyhow::ensure!(
                    expected.as_u64() == Some(count),
                    "manifest {key} differs: {logical}"
                );
            }
        }
        if let Some(expected) = metadata.get("total_bytes") {
            anyhow::ensure!(
                expected.as_u64() == Some(total),
                "manifest total_bytes differs: {logical}"
            );
        }
    }
    println!("authenticated manifest {logical}: {count} files, {total} bytes");
    Ok((count, total))
}

#[test]
#[ignore = "requires frozen inner-tomi audit evidence archives"]
fn audit_evidence_historical_inventory_authenticated() -> Result<()> {
    let ledger = audit_repair_ledger();
    let root = AuditEvidenceRoot::from_environment();
    let documents = authenticated_audit_documents(
        &root,
        &ledger,
        &[AuditDocumentKind::Candidates, AuditDocumentKind::Reviews],
    )?;
    root.read_verified(&archive_manifest_reference(&ledger, "archive")?)?;
    validate_omission_inventory_data(&ledger, &documents)?;
    println!(
        "historical candidate/review and manifest bytes authenticated; Tier A 39/69, Tier B 20/44"
    );
    Ok(())
}

#[test]
#[ignore = "requires frozen inner-tomi audit evidence archives"]
fn audit_evidence_baseline_diagnosis_authenticated() -> Result<()> {
    let ledger = audit_repair_ledger();
    let root = AuditEvidenceRoot::from_environment();
    root.read_verified(&archive_manifest_reference(&ledger, "diagnostic_archive")?)?;
    let documents = authenticated_audit_documents(&root, &ledger, &[AuditDocumentKind::Baseline])?;
    let mut overlays = std::collections::BTreeMap::new();
    for event in evidence_array(&ledger, "events")?
        .iter()
        .filter(|e| e["initial_tier"] == "A")
    {
        let reference = &event["diagnosis"]["reviewed_source_overlay"];
        let path = normalized_evidence_path(evidence_string(reference, "path")?)?;
        let hash = evidence_digest(reference, "sha256")?;
        if let Some(previous) = overlays.get(&path) {
            anyhow::ensure!(*previous == hash, "conflicting overlay identity: {path}");
        } else {
            root.read_verified(reference)?;
            overlays.insert(path, hash);
        }
    }
    validate_omission_diagnosis_data(&ledger, &documents)
}

#[test]
#[ignore = "requires frozen inner-tomi audit evidence archives"]
fn audit_evidence_post_dedup_traces_authenticated() -> Result<()> {
    let ledger = audit_repair_ledger();
    let root = AuditEvidenceRoot::from_environment();
    root.read_verified(&ledger["diagnosis_archives"]["post-dedup"])?;
    let documents = authenticated_audit_documents(&root, &ledger, &[AuditDocumentKind::PostDedup])?;
    validate_post_dedup_data(&ledger, &documents)
}

#[test]
#[ignore = "requires frozen inner-tomi audit evidence archives"]
fn audit_evidence_post_mask_signatures_authenticated() -> Result<()> {
    let ledger = audit_repair_ledger();
    let root = AuditEvidenceRoot::from_environment();
    root.read_verified(&ledger["diagnosis_archives"]["post-masks"])?;
    let documents = authenticated_audit_documents(&root, &ledger, &[AuditDocumentKind::PostMasks])?;
    validate_source_signature_data(&ledger, &documents)
}

#[test]
#[ignore = "requires frozen inner-tomi audit evidence archives"]
fn audit_evidence_restoration_reports_authenticated() -> Result<()> {
    let ledger = audit_repair_ledger();
    let root = AuditEvidenceRoot::from_environment();
    let documents = authenticated_audit_documents(
        &root,
        &ledger,
        &[
            AuditDocumentKind::Candidates,
            AuditDocumentKind::Reviews,
            AuditDocumentKind::Restoration,
        ],
    )?;
    validate_tier_a_completion_data(&ledger, &documents)?;
    println!("historical passing reports authenticated for 69 initial Tier-A lines and the separate Cato label");
    Ok(())
}

#[test]
#[ignore = "requires frozen inner-tomi audit evidence archives"]
fn audit_evidence_portable_fixture_matches_archives() -> Result<()> {
    let ledger = audit_repair_ledger();
    let root = AuditEvidenceRoot::from_environment();
    let documents = load_fixture_source_documents(&ledger, &root.root)?;
    let projected = project_audit_evidence(&ledger, &documents)?;
    anyhow::ensure!(
        projected == portable_audit_evidence(),
        "tracked projection differs from authenticated archives"
    );
    Ok(())
}

#[test]
#[ignore = "requires frozen inner-tomi audit evidence archives"]
fn audit_evidence_archive_manifests_authenticated() -> Result<()> {
    let ledger = audit_repair_ledger();
    let root = AuditEvidenceRoot::from_environment();
    let mut seen = std::collections::BTreeMap::new();
    for key in ["archive", "diagnostic_archive"] {
        authenticate_evidence_manifest(
            &root,
            &archive_manifest_reference(&ledger, key)?,
            &ledger[key],
            &mut seen,
        )?;
    }
    for key in ["post-dedup", "post-masks"] {
        authenticate_evidence_manifest(
            &root,
            &ledger["diagnosis_archives"][key],
            &serde_json::Value::Null,
            &mut seen,
        )?;
    }
    Ok(())
}

fn tiny_evidence_file(
    root: &AuditEvidenceRoot,
    path: &str,
    bytes: &[u8],
) -> Result<serde_json::Value> {
    use sha2::Digest;
    let file = root.root.join(path);
    std::fs::create_dir_all(file.parent().unwrap())?;
    std::fs::write(file, bytes)?;
    Ok(serde_json::json!({"path":path,"sha256":format!("{:x}",sha2::Sha256::digest(bytes))}))
}

#[test]
fn evidence_reader_rejects_missing_file() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let root = AuditEvidenceRoot {
        root: dir.path().to_path_buf(),
    };
    let reference = serde_json::json!({"path":"archive/missing.json","sha256":"a".repeat(64)});
    let error = root.read_verified(&reference).unwrap_err().to_string();
    assert!(
        error.contains("missing historical evidence") && error.contains("archive/missing.json")
    );
    assert!(root.resolve("archive/missing.json").is_err());
    Ok(())
}

#[test]
fn evidence_reader_rejects_hash_mismatch() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let root = AuditEvidenceRoot {
        root: dir.path().to_path_buf(),
    };
    let mut reference = tiny_evidence_file(&root, "archive/tiny.json", b"{}")?;
    assert_eq!(root.read_verified_json(&reference)?, serde_json::json!({}));
    assert_eq!(root.read_verified(&reference)?, b"{}");
    reference["sha256"] = "a".repeat(64).into();
    let error = root.read_verified(&reference).unwrap_err().to_string();
    assert!(
        error.contains("hash mismatch")
            && error.contains("expected")
            && error.contains("actual")
            && error.contains("resolved")
            && error.contains("archive/tiny.json")
    );
    assert!(root
        .verify_streamed(&reference)
        .unwrap_err()
        .to_string()
        .contains("hash mismatch"));
    reference["sha256"] = "bad".into();
    assert!(root
        .read_verified(&reference)
        .unwrap_err()
        .to_string()
        .contains("malformed"));
    Ok(())
}

#[test]
fn evidence_reader_rejects_escaping_path() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let root = AuditEvidenceRoot {
        root: dir.path().to_path_buf(),
    };
    for path in [
        "../outside",
        "archive/../../outside",
        "/absolute",
        "C:/absolute",
        "C:relative",
        "\\\\server\\share\\file",
        "archive\\..\\outside",
    ] {
        let reference = serde_json::json!({"path":path,"sha256":"a".repeat(64)});
        assert!(
            root.read_verified(&reference)
                .unwrap_err()
                .to_string()
                .contains("escaping evidence path"),
            "{path}"
        );
    }
    let reference = tiny_evidence_file(&root, "archive/nested/file", b"tiny")?;
    let mut windows_reference = reference.clone();
    windows_reference["path"] = "archive\\nested\\file".into();
    assert_eq!(root.read_verified(&windows_reference)?, b"tiny");
    assert!(root
        .resolve("archive/nested/file")?
        .starts_with(dir.path().canonicalize()?));
    let outside = tempfile::tempdir()?;
    std::fs::write(outside.path().join("file"), b"tiny")?;
    let link = root.root.join("escape");
    #[cfg(windows)]
    let linked = std::os::windows::fs::symlink_dir(outside.path(), &link);
    #[cfg(unix)]
    let linked = std::os::unix::fs::symlink(outside.path(), &link);
    match linked {
        Ok(()) => assert!(root
            .resolve("escape/file")
            .unwrap_err()
            .to_string()
            .contains("escaping evidence path")),
        #[cfg(windows)]
        Err(error) if error.raw_os_error() == Some(1314) => eprintln!(
            "symlink test unavailable: Windows privilege 1314; lexical escape checks passed"
        ),
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

#[test]
fn evidence_manifest_supports_archive_and_stage_formats() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let root = AuditEvidenceRoot {
        root: dir.path().to_path_buf(),
    };
    for size_key in ["bytes", "size"] {
        let file = tiny_evidence_file(&root, &format!("archive/{size_key}/tiny"), b"abc")?;
        let mut entry = serde_json::json!({"path":"tiny","sha256":file["sha256"]});
        entry[size_key] = 3.into();
        let manifest = serde_json::json!({"source":"C:/descriptive/only", "destination":"Z:/not/read",
            "files":[entry],"file_count":1,"total_bytes":3});
        let reference = tiny_evidence_file(
            &root,
            &format!("archive/{size_key}/manifest.json"),
            &serde_json::to_vec(&manifest)?,
        )?;
        assert_eq!(
            authenticate_evidence_manifest(
                &root,
                &reference,
                &serde_json::json!({"verified_files":1}),
                &mut std::collections::BTreeMap::new()
            )?,
            (1, 3)
        );
    }
    let one = tiny_evidence_file(&root, "archive/post-dedup/tiny", b"same bytes")?;
    let two = tiny_evidence_file(&root, "archive/post-masks/tiny", b"same bytes")?;
    assert_eq!(one["sha256"], two["sha256"]);
    let stage = tiny_evidence_file(
        &root,
        "archive/stage/manifest.json",
        &serde_json::to_vec(&serde_json::json!([one, two]))?,
    )?;
    assert_eq!(
        authenticate_evidence_manifest(
            &root,
            &stage,
            &serde_json::Value::Null,
            &mut std::collections::BTreeMap::new()
        )?,
        (2, 20)
    );
    Ok(())
}

#[test]
fn evidence_manifest_rejects_duplicate_or_mismatched_entries() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let root = AuditEvidenceRoot {
        root: dir.path().to_path_buf(),
    };
    let entry = tiny_evidence_file(&root, "archive/unit/tiny", b"abc")?;
    for (kind, expected) in [
        ("duplicate", "duplicate"),
        ("hash", "hash mismatch"),
        ("size", "size differs"),
        ("escape", "escaping"),
        ("count", "file_count differs"),
        ("total", "total_bytes differs"),
    ] {
        let mut entries = vec![entry.clone()];
        let mut pinned = serde_json::Value::Null;
        match kind {
            "duplicate" => {
                let mut same = entry.clone();
                same["path"] = "archive\\unit\\tiny".into();
                same["sha256"] = "b".repeat(64).into();
                entries.push(same);
            }
            "hash" => entries[0]["sha256"] = "a".repeat(64).into(),
            "size" => entries[0]["size"] = 4.into(),
            "escape" => entries[0]["path"] = "../outside".into(),
            "count" => pinned = serde_json::json!({"file_count":2}),
            "total" => pinned = serde_json::json!({"total_bytes":4}),
            _ => unreachable!(),
        }
        let reference = tiny_evidence_file(
            &root,
            "archive/unit/manifest.json",
            &serde_json::to_vec(&entries)?,
        )?;
        let error = authenticate_evidence_manifest(
            &root,
            &reference,
            &pinned,
            &mut std::collections::BTreeMap::new(),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains(expected), "{kind}: {error}");
    }
    Ok(())
}

#[test]
fn audit_completion_rejects_wrong_unit_and_missing_report_line() -> Result<()> {
    let ledger = audit_repair_ledger();
    let evidence = portable_audit_evidence();
    validate_tier_a_completion_data(&ledger, &evidence)?;
    let event = evidence_array(&ledger, "events")?
        .iter()
        .find(|e| e["initial_tier"] == "A")
        .unwrap();
    let path = evidence_string(&event["passing_evidence"], "path")?;
    for kind in ["unit", "line", "event"] {
        let mut changed = evidence.clone();
        let report = &mut changed["documents"][path]["value"];
        let expected = match kind {
            "unit" => {
                report["unit_id"] = "wrong_unit".into();
                "unit differs"
            }
            "line" => {
                let line: ReviewedSourceLine =
                    serde_json::from_value(event["diagnosis"]["reviewed_lines"][0].clone())?;
                let line = serde_json::to_value(line)?;
                report["reviewed_lines"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|r| *r != line);
                "reviewed line absent"
            }
            "event" => {
                report["events"]
                    .as_array_mut()
                    .unwrap()
                    .retain(|id| *id != event["event_id"]);
                "event absent"
            }
            _ => unreachable!(),
        };
        let error = validate_tier_a_completion_data(&ledger, &changed)
            .unwrap_err()
            .to_string();
        assert!(error.contains(expected), "{kind}: {error}");
    }
    let extra = &ledger["additional_source_losses"][0];
    let mut changed = evidence.clone();
    let report =
        &mut changed["documents"][evidence_string(&extra["passing_evidence"], "path")?]["value"];
    report["reviewed_lines"]
        .as_array_mut()
        .unwrap()
        .retain(|r| *r != extra["reviewed_lines"][0]);
    assert!(validate_tier_a_completion_data(&ledger, &changed)
        .unwrap_err()
        .to_string()
        .contains("additional source glyph region absent"));
    Ok(())
}

fn historical_audit_json_reference(
    ledger: &serde_json::Value,
    name: &str,
) -> Result<serde_json::Value> {
    let key = match name {
        "candidates.json" => "candidates_sha256",
        "reviews.json" => "reviews_sha256",
        _ => anyhow::bail!("unknown historical audit JSON: {name}"),
    };
    Ok(
        serde_json::json!({"path": format!("{}/runs/step102-final-all-5/{name}",
        evidence_string(&ledger["archive"], "path")?), "sha256": evidence_digest(ledger, key)?}),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AuditDocumentKind {
    Candidates,
    Reviews,
    Baseline,
    PostDedup,
    PostMasks,
    Restoration,
    Disposition,
    DispositionTrace,
    FullWorkBaseline,
}

type AuditReferences = std::collections::BTreeMap<String, (serde_json::Value, AuditDocumentKind)>;

fn required_audit_references(ledger: &serde_json::Value) -> Result<AuditReferences> {
    let mut references = AuditReferences::new();
    let mut add = |reference: serde_json::Value, kind| -> Result<()> {
        let path = normalized_evidence_path(evidence_string(&reference, "path")?)?;
        let digest = evidence_digest(&reference, "sha256")?;
        if let Some((previous, previous_kind)) = references.get(&path) {
            anyhow::ensure!(
                evidence_digest(previous, "sha256")? == digest && *previous_kind == kind,
                "conflicting evidence identity: {path}"
            );
        } else {
            references.insert(path, (reference, kind));
        }
        Ok(())
    };
    add(
        historical_audit_json_reference(ledger, "candidates.json")?,
        AuditDocumentKind::Candidates,
    )?;
    add(
        historical_audit_json_reference(ledger, "reviews.json")?,
        AuditDocumentKind::Reviews,
    )?;
    for event in evidence_array(ledger, "events")? {
        if event["initial_tier"] == "A" {
            add(
                event["diagnosis"]["baseline_trace"].clone(),
                AuditDocumentKind::Baseline,
            )?;
            add(
                event["diagnosis"]["post_dedup"]["trace"].clone(),
                AuditDocumentKind::PostDedup,
            )?;
            add(
                event["diagnosis"]["post_masks"]["trace"].clone(),
                AuditDocumentKind::PostMasks,
            )?;
        }
        if event["initial_tier"] == "B" && event["promoted_tier"] == "A" {
            add(
                event["diagnosis"]["baseline_trace"].clone(),
                AuditDocumentKind::Baseline,
            )?;
            if event["status"] == "unresolved" {
                add(
                    event["disposition_evidence"].clone(),
                    AuditDocumentKind::Disposition,
                )?;
                add(
                    event["disposition_trace"].clone(),
                    AuditDocumentKind::DispositionTrace,
                )?;
            }
            for historical in event["historical_dispositions"]
                .as_array()
                .into_iter()
                .flatten()
            {
                add(
                    historical["disposition_evidence"].clone(),
                    AuditDocumentKind::Disposition,
                )?;
                add(
                    historical["disposition_trace"].clone(),
                    AuditDocumentKind::DispositionTrace,
                )?;
            }
        }
        if event["initial_tier"] == "A"
            || (event["promoted_tier"] == "A"
                && (event["status"] == "installed" || event["status"] == "restored"))
        {
            add(
                event["passing_evidence"].clone(),
                AuditDocumentKind::Restoration,
            )?;
        }
    }
    if ledger["events"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["promoted_tier"] == "A")
    {
        add(
            ledger["tier_b_baseline"].clone(),
            AuditDocumentKind::FullWorkBaseline,
        )?;
    }
    for item in evidence_array(ledger, "additional_source_losses")? {
        add(
            item["passing_evidence"].clone(),
            AuditDocumentKind::Restoration,
        )?;
    }
    Ok(references)
}

fn load_fixture_source_documents(
    ledger: &serde_json::Value,
    root: &std::path::Path,
) -> Result<serde_json::Value> {
    authenticated_audit_documents(
        &AuditEvidenceRoot {
            root: root.to_path_buf(),
        },
        ledger,
        &[
            AuditDocumentKind::Candidates,
            AuditDocumentKind::Reviews,
            AuditDocumentKind::Baseline,
            AuditDocumentKind::PostDedup,
            AuditDocumentKind::PostMasks,
            AuditDocumentKind::Restoration,
            AuditDocumentKind::Disposition,
            AuditDocumentKind::DispositionTrace,
            AuditDocumentKind::FullWorkBaseline,
        ],
    )
}

fn portable_evidence_document<'a>(
    evidence: &'a serde_json::Value,
    reference: &serde_json::Value,
) -> Result<&'a serde_json::Value> {
    let path = normalized_evidence_path(evidence_string(reference, "path")?)?;
    let record = evidence["documents"]
        .get(&path)
        .ok_or_else(|| anyhow::anyhow!("missing portable evidence document: {path}"))?;
    anyhow::ensure!(
        evidence_digest(record, "source_sha256")? == evidence_digest(reference, "sha256")?,
        "portable evidence source digest differs: {path}"
    );
    let value = record
        .get("value")
        .filter(|v| v.is_object())
        .ok_or_else(|| anyhow::anyhow!("missing portable evidence value: {path}"))?;
    Ok(value)
}

fn project_evidence_fields(
    value: &serde_json::Value,
    fields: &[&str],
) -> Result<serde_json::Value> {
    let mut object = serde_json::Map::new();
    for field in fields {
        let v = value
            .get(*field)
            .filter(|v| !v.is_null())
            .ok_or_else(|| anyhow::anyhow!("missing projection field {field}"))?;
        object.insert((*field).to_owned(), v.clone());
    }
    Ok(object.into())
}

fn project_audit_evidence(
    ledger: &serde_json::Value,
    documents: &serde_json::Value,
) -> Result<serde_json::Value> {
    let candidate_reference = historical_audit_json_reference(ledger, "candidates.json")?;
    let candidates = portable_evidence_document(documents, &candidate_reference)?;
    let selected: Vec<_> = evidence_array(candidates, "candidates")?
        .iter()
        .filter(|c| c["type"] == "missing_occurrence" && c["unique_loss"] == true)
        .collect();
    let candidate_ids: std::collections::BTreeSet<_> = selected
        .iter()
        .map(|c| evidence_string(c, "event_id"))
        .collect::<Result<_>>()?;
    let mut projected = serde_json::Map::new();
    for (path, (reference, kind)) in required_audit_references(ledger)? {
        let value = portable_evidence_document(documents, &reference)?;
        let projection = match kind {
            AuditDocumentKind::Candidates => serde_json::json!({"candidates": selected.iter()
                .map(|c| project_evidence_fields(c, &["event_id", "unit_id", "type", "unique_loss",
                    "source_regions", "missing_line_count", "confidence", "crops"]))
                .collect::<Result<Vec<_>>>()?}),
            AuditDocumentKind::Reviews => {
                serde_json::json!({"events": evidence_array(value, "events")?.iter()
                .filter(|r| r["event_id"].as_str().is_some_and(|id| candidate_ids.contains(id)))
                .map(|r| project_evidence_fields(r, &["event_id", "verdict", "evidence"]))
                .collect::<Result<Vec<_>>>()?})
            }
            AuditDocumentKind::Baseline => {
                project_evidence_fields(value, &["unit_id", "metrics", "scale"])?
            }
            AuditDocumentKind::PostDedup => {
                let mut v = project_evidence_fields(value, &["unit_id", "metrics", "scale"])?;
                v["source_pages"] = evidence_array(value, "source_pages")?
                    .iter()
                    .map(|p| {
                        // A null retained index identifies a deliberately excluded source page.
                        anyhow::ensure!(
                            p.get("retained_index").is_some(),
                            "missing retained_index"
                        );
                        let mut row = project_evidence_fields(p, &["page"])?;
                        row["retained_index"] = p["retained_index"].clone();
                        Ok(row)
                    })
                    .collect::<Result<Vec<_>>>()?
                    .into();
                v
            }
            AuditDocumentKind::PostMasks => {
                let mut v = project_evidence_fields(value, &["unit_id"])?;
                let required_pages: std::collections::BTreeSet<_> =
                    evidence_array(ledger, "planned_body_band_repairs")?
                        .iter()
                        .filter(|r| r["unit_id"] == value["unit_id"])
                        .map(|r| {
                            (
                                r["source"].clone().to_string(),
                                r["source_page"].clone().to_string(),
                            )
                        })
                        .collect();
                let pages: Vec<_> = evidence_array(value, "pages")?
                    .iter()
                    .filter(|p| {
                        required_pages
                            .contains(&(p["source"].to_string(), p["source_page"].to_string()))
                    })
                    .map(|p| {
                        let mut page = project_evidence_fields(
                            p,
                            &["source", "source_page", "raster", "crop_edges", "raw_bands"],
                        )?;
                        for key in ["candidate_crops", "candidate_results"] {
                            let variants = match p.get(key) {
                                None => Vec::new(),
                                Some(a) => a
                                    .as_array()
                                    .ok_or_else(|| anyhow::anyhow!("malformed {key}"))?
                                    .iter()
                                    .map(|v| {
                                        project_evidence_fields(v, &["crop_edges", "raw_bands"])
                                    })
                                    .collect::<Result<Vec<_>>>()?,
                            };
                            page[key] = variants.into();
                        }
                        Ok(page)
                    })
                    .collect::<Result<_>>()?;
                let found: std::collections::BTreeSet<_> = pages
                    .iter()
                    .map(|p| (p["source"].to_string(), p["source_page"].to_string()))
                    .collect();
                anyhow::ensure!(
                    found == required_pages && pages.len() == found.len(),
                    "missing or duplicate signature page"
                );
                v["pages"] = pages.into();
                v
            }
            AuditDocumentKind::Restoration => {
                let projected = project_restoration_report(value)?;
                if evidence_array(ledger, "events")?.iter().any(|e| {
                    e["initial_tier"] == "B"
                        && e["promoted_tier"] == "A"
                        && e["passing_evidence"]["path"] == reference["path"]
                }) {
                    anyhow::ensure!(
                        projected["schema_version"] == 2
                            && projected["verification_policy"] == "promoted_tier_b",
                        "promoted restoration cannot downgrade to legacy projection"
                    );
                }
                projected
            }
            AuditDocumentKind::Disposition => {
                let mut projected = project_evidence_fields(
                    value,
                    &[
                        "schema_version",
                        "event_id",
                        "unit_id",
                        "status",
                        "reason",
                        "tested_configuration",
                        "failure_stage",
                        "trace_evidence",
                    ],
                )?;
                if value["failure_stage"] == "not_established" {
                    projected["remaining_uncertainty"] = value["remaining_uncertainty"].clone();
                }
                projected
            }
            AuditDocumentKind::DispositionTrace => project_evidence_fields(
                value,
                &[
                    "schema_version",
                    "event_id",
                    "unit_id",
                    "tested_configuration",
                    "stage_records",
                ],
            )?,
            AuditDocumentKind::FullWorkBaseline => project_evidence_fields(
                value,
                &[
                    "schema_version",
                    "scale",
                    "configuration",
                    "units",
                    "protection_record_count",
                ],
            )?,
        };
        projected.insert(
            path,
            serde_json::json!({"source_sha256": reference["sha256"], "value": projection}),
        );
    }
    Ok(serde_json::json!({"schema_version": 1, "documents": projected}))
}

#[test]
#[ignore = "requires frozen archives and explicit PLUTARCH_PORTABLE_FIXTURE_OUT"]
fn export_portable_fixture() -> Result<()> {
    let out = std::env::var_os("PLUTARCH_PORTABLE_FIXTURE_OUT").ok_or_else(|| {
        anyhow::anyhow!("PLUTARCH_PORTABLE_FIXTURE_OUT must name an explicit review file")
    })?;
    let out = std::path::PathBuf::from(out);
    let tracked = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/facsimile/fixtures/inner_tomi_portable_audit_evidence.json");
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let absolute = out
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(std::path::Path::new("."))
        .canonicalize()?
        .join(out.file_name().unwrap());
    anyhow::ensure!(
        absolute != tracked.canonicalize()?
            && out.canonicalize().ok().as_ref() != Some(&tracked.canonicalize()?),
        "export to a temporary review file, not the tracked fixture"
    );
    let root = AuditEvidenceRoot::from_environment();
    let ledger = audit_repair_ledger();
    let documents = load_fixture_source_documents(&ledger, &root.root)?;
    let projection = project_audit_evidence(&ledger, &documents)?;
    let mut bytes = serde_json::to_vec_pretty(&projection)?;
    bytes.push(b'\n');
    std::fs::write(&out, &bytes)?;
    println!(
        "authenticated portable projection: {} documents, {} bytes, {}",
        projection["documents"].as_object().unwrap().len(),
        bytes.len(),
        out.display()
    );
    Ok(())
}

fn validate_omission_inventory_data(
    fixture: &serde_json::Value,
    documents: &serde_json::Value,
) -> Result<()> {
    let candidates = portable_evidence_document(
        documents,
        &historical_audit_json_reference(fixture, "candidates.json")?,
    )?;
    let reviews = portable_evidence_document(
        documents,
        &historical_audit_json_reference(fixture, "reviews.json")?,
    )?;
    let reviewed: std::collections::BTreeMap<_, _> = reviews["events"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r["event_id"].as_str().unwrap(), r))
        .collect();
    let eligible: std::collections::BTreeMap<_, _> = candidates["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| {
            c["type"] == "missing_occurrence"
                && c["unique_loss"] == true
                && reviewed
                    .get(c["event_id"].as_str().unwrap())
                    .is_some_and(|r| r["verdict"] == "confirmed")
        })
        .map(|c| (c["event_id"].as_str().unwrap(), c))
        .collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut counts = [[0u64; 2]; 2];
    for event in fixture["events"].as_array().unwrap() {
        let id = event["event_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("event ID missing"))?;
        anyhow::ensure!(
            id != "miss_tiberius_gracchus_403_690",
            "false positive entered repair inventory"
        );
        anyhow::ensure!(seen.insert(id), "duplicate omission event: {id}");
        let candidate = eligible
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("unconfirmed nomination: {id}"))?;
        let review = reviewed[id];
        for key in [
            "unit_id",
            "source_regions",
            "missing_line_count",
            "confidence",
        ] {
            anyhow::ensure!(event[key] == candidate[key], "frozen {key} differs: {id}");
        }
        anyhow::ensure!(
            event["verdict"] == review["verdict"],
            "verdict differs: {id}"
        );
        anyhow::ensure!(
            event["candidate_crops"] == candidate["crops"],
            "candidate crops differ: {id}"
        );
        anyhow::ensure!(
            event["review_evidence"] == review["evidence"],
            "review crops differ: {id}"
        );
        let tier_a = candidate["confidence"] == "high"
            || ["miss_romulus_38_2595", "miss_marcius_coriolanus_292_10538"].contains(&id);
        anyhow::ensure!(
            !tier_a || event["status"] != "deferred",
            "Tier-A omission cannot be deferred: {id}"
        );
        anyhow::ensure!(
            event["initial_tier"] == if tier_a { "A" } else { "B" },
            "tier differs: {id}"
        );
        let count = &mut counts[usize::from(!tier_a)];
        count[0] += 1;
        count[1] += event["missing_line_count"].as_u64().unwrap();
    }
    anyhow::ensure!(
        seen == eligible.keys().copied().collect(),
        "confirmed omission inventory incomplete"
    );
    anyhow::ensure!(
        counts == [[39, 69], [20, 44]],
        "initial tier partition differs: {counts:?}"
    );
    Ok(())
}

#[test]
fn audit_omission_inventory_matches_frozen_review() {
    validate_omission_inventory_data(&audit_repair_ledger(), &portable_audit_evidence()).unwrap();
}

#[test]
fn audit_omission_diagnosis_covers_all_source_lines() {
    validate_omission_diagnosis_data(&audit_repair_ledger(), &portable_audit_evidence()).unwrap();
}

#[test]
fn audit_omission_post_dedup_covers_all_source_lines() {
    validate_post_dedup_data(&audit_repair_ledger(), &portable_audit_evidence()).unwrap();
}

#[test]
fn audit_omission_source_signatures_match_proposed_repairs() {
    validate_source_signature_data(&audit_repair_ledger(), &portable_audit_evidence()).unwrap();
}

fn validate_trace_metadata(trace: &serde_json::Value, unit: &serde_json::Value) -> Result<()> {
    anyhow::ensure!(trace["unit_id"] == *unit, "trace unit differs");
    let metrics = &trace["metrics"];
    anyhow::ensure!(
        metrics["height_med"].as_u64().is_some_and(|v| v > 0)
            && metrics["width_med"].as_u64().is_some_and(|v| v > 0)
            && metrics["height_mad"].as_u64().is_some(),
        "invalid trace metrics"
    );
    anyhow::ensure!(
        trace["scale"]
            .as_f64()
            .is_some_and(|v| v.is_finite() && v > 0.0),
        "invalid trace scale"
    );
    Ok(())
}

fn validate_omission_diagnosis_data(
    fixture: &serde_json::Value,
    documents: &serde_json::Value,
) -> Result<()> {
    let mut counts = [0, 0];
    for event in evidence_array(fixture, "events")? {
        if event["initial_tier"] == "B" {
            if event["promoted_tier"] == "A" {
                validate_tier_b_promoted_diagnosis(event, documents)?;
                continue;
            }
            anyhow::ensure!(event["status"] == "deferred", "Tier-B disposition differs");
            evidence_string(event, "repair_read_status")?;
            continue;
        }
        let diagnosis = &event["diagnosis"];
        let trace = portable_evidence_document(documents, &diagnosis["baseline_trace"])?;
        validate_trace_metadata(trace, &event["unit_id"])?;
        let overlay = &diagnosis["reviewed_source_overlay"];
        normalized_evidence_path(evidence_string(overlay, "path")?)?;
        evidence_digest(overlay, "sha256")?;
        let origin = evidence_array(overlay, "source_origin")?;
        anyhow::ensure!(
            origin.len() == 2 && origin.iter().all(|v| v.as_u64().is_some()),
            "invalid recorded source origin"
        );
        let lines = evidence_array(diagnosis, "reviewed_lines")?;
        anyhow::ensure!(
            Some(lines.len() as u64) == event["missing_line_count"].as_u64(),
            "reviewed source line count differs"
        );
        let mut previous = None;
        for line in lines {
            let region: [u32; 4] = serde_json::from_value(line["region"].clone())?;
            anyhow::ensure!(
                region[0] < region[2] && region[1] < region[3],
                "empty reviewed source rectangle"
            );
            anyhow::ensure!(
                previous.is_none_or(|top| top < region[1]),
                "reviewed source lines out of order"
            );
            previous = Some(region[1]);
            evidence_string(&line["baseline"], "first_loss_stage")?;
            evidence_string(line, "landmark")?;
        }
        evidence_string(diagnosis, "regression_name")?;
        counts[0] += 1;
        counts[1] += lines.len();
    }
    anyhow::ensure!(
        counts == [39, 69],
        "diagnosis inventory differs: {counts:?}"
    );
    Ok(())
}

fn validate_post_dedup_data(
    fixture: &serde_json::Value,
    documents: &serde_json::Value,
) -> Result<()> {
    let mut counts = [0, 0];
    for event in evidence_array(fixture, "events")?
        .iter()
        .filter(|e| e["initial_tier"] == "A")
    {
        let measured = &event["diagnosis"]["post_dedup"];
        let trace = portable_evidence_document(documents, &measured["trace"])?;
        validate_trace_metadata(trace, &event["unit_id"])?;
        anyhow::ensure!(
            trace["metrics"] == measured["metrics"] && trace["scale"] == measured["scale"],
            "post-dedup metadata differs"
        );
        let retained: Vec<_> = evidence_array(trace, "source_pages")?
            .iter()
            .filter(|p| !p["retained_index"].is_null())
            .map(|p| p["page"].clone())
            .collect();
        anyhow::ensure!(
            serde_json::Value::from(retained) == measured["source_pages"],
            "post-dedup retained source sequence differs"
        );
        let lines = evidence_array(measured, "lines")?;
        let reviewed = evidence_array(&event["diagnosis"], "reviewed_lines")?;
        anyhow::ensure!(
            Some(lines.len() as u64) == event["missing_line_count"].as_u64()
                && lines.len() == reviewed.len(),
            "post-dedup line count differs"
        );
        for (line, reviewed) in lines.iter().zip(reviewed) {
            anyhow::ensure!(
                line["region"] == reviewed["region"],
                "post-dedup source region differs"
            );
            evidence_string(&line["measured"], "first_loss_stage")?;
        }
        if !measured["complete"]
            .as_bool()
            .ok_or_else(|| anyhow::anyhow!("missing completion status"))?
        {
            anyhow::ensure!(
                !evidence_array(measured, "proposed_body_bands")?.is_empty(),
                "missing proposed body bands"
            );
        }
        counts[0] += 1;
        counts[1] += lines.len();
    }
    anyhow::ensure!(
        counts == [39, 69],
        "post-dedup inventory differs: {counts:?}"
    );
    Ok(())
}

fn measured_signature_matches(
    page: &serde_json::Value,
    repair: &serde_json::Value,
) -> Result<bool> {
    let mut variants = vec![page];
    for key in ["candidate_crops", "candidate_results"] {
        if let Some(array) = page.get(key) {
            variants.extend(
                array
                    .as_array()
                    .ok_or_else(|| anyhow::anyhow!("invalid {key}"))?,
            );
        }
    }
    for variant in variants {
        let edges: [u32; 2] = serde_json::from_value(variant["crop_edges"].clone())?;
        anyhow::ensure!(edges[0] < edges[1], "invalid measured crop edges");
        if repair["column_dimensions"][0].as_u64() == Some((edges[1] - edges[0]) as u64)
            && repair["column_dimensions"][1] == page["raster"][1]
            && evidence_array(variant, "raw_bands")?.contains(&repair["observed_band"])
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn validate_source_signature_data(
    fixture: &serde_json::Value,
    documents: &serde_json::Value,
) -> Result<()> {
    let mut traces = std::collections::BTreeMap::new();
    for event in evidence_array(fixture, "events")?
        .iter()
        .filter(|e| e["initial_tier"] == "A")
    {
        let trace =
            portable_evidence_document(documents, &event["diagnosis"]["post_masks"]["trace"])?;
        anyhow::ensure!(
            trace["unit_id"] == event["unit_id"],
            "post-mask trace unit differs"
        );
        traces.insert(evidence_string(event, "unit_id")?, trace);
    }
    let repairs = evidence_array(fixture, "planned_body_band_repairs")?;
    anyhow::ensure!(
        repairs.len() == 47,
        "finite repair signature inventory differs"
    );
    for repair in repairs {
        let trace = traces
            .get(evidence_string(repair, "unit_id")?)
            .ok_or_else(|| anyhow::anyhow!("missing repair unit trace"))?;
        let page = evidence_array(trace, "pages")?
            .iter()
            .find(|p| p["source_page"] == repair["source_page"] && p["source"] == repair["source"])
            .ok_or_else(|| anyhow::anyhow!("missing repair signature page: {repair}"))?;
        anyhow::ensure!(
            measured_signature_matches(page, repair)?,
            "refreshed source signature differs: {repair}"
        );
    }
    Ok(())
}

#[test]
fn portable_audit_evidence_schema_and_size_are_valid() {
    let evidence = portable_audit_evidence();
    assert_eq!(evidence["schema_version"], 1);
    assert!(include_bytes!("fixtures/inner_tomi_portable_audit_evidence.json").len() < 1024 * 1024);
    assert!(evidence["documents"].is_object());
}

#[test]
fn portable_audit_evidence_has_every_required_document() -> Result<()> {
    let ledger = audit_repair_ledger();
    let evidence = portable_audit_evidence();
    let references = required_audit_references(&ledger)?;
    assert_eq!(
        references.len(),
        evidence["documents"].as_object().unwrap().len()
    );
    for (reference, _) in references.values() {
        portable_evidence_document(&evidence, reference)?;
    }
    validate_omission_inventory_data(&ledger, &evidence)?;
    validate_omission_diagnosis_data(&ledger, &evidence)?;
    validate_post_dedup_data(&ledger, &evidence)?;
    validate_source_signature_data(&ledger, &evidence)?;
    validate_tier_a_completion_data(&ledger, &evidence)?;
    if ledger.get("tier_b_delivery").is_some() {
        validate_tier_b_repair_state_data(&ledger, &evidence)?;
    }
    Ok(())
}

#[test]
fn portable_evidence_document_rejects_missing_or_mismatched_reference() {
    let evidence = portable_audit_evidence();
    let reference =
        historical_audit_json_reference(&audit_repair_ledger(), "candidates.json").unwrap();
    for key in ["path", "sha256"] {
        let mut changed = reference.clone();
        changed[key] = if key == "path" {
            "archive/missing.json".into()
        } else {
            "0".repeat(64).into()
        };
        assert!(portable_evidence_document(&evidence, &changed).is_err());
    }
    let mut invalid = evidence.clone();
    invalid["documents"][reference["path"].as_str().unwrap()]["source_sha256"] = "invalid".into();
    assert!(portable_evidence_document(&invalid, &reference).is_err());
    invalid["documents"][reference["path"].as_str().unwrap()]
        .as_object_mut()
        .unwrap()
        .remove("source_sha256");
    assert!(portable_evidence_document(&invalid, &reference).is_err());
}

#[test]
fn portable_audit_projection_is_deterministic() -> Result<()> {
    let digest = "a".repeat(64);
    let ledger = serde_json::json!({"archive":{"path":"archive/test"},
        "candidates_sha256":digest, "reviews_sha256":digest,
        "events":[], "additional_source_losses":[], "planned_body_band_repairs":[]});
    let refs = required_audit_references(&ledger)?;
    let mut forward = serde_json::Map::new();
    let mut reverse = serde_json::Map::new();
    for (path, (reference, kind)) in &refs {
        let value = if *kind == AuditDocumentKind::Candidates {
            serde_json::json!({"candidates":[]})
        } else {
            serde_json::json!({"events":[]})
        };
        forward.insert(
            path.clone(),
            serde_json::json!({"source_sha256":reference["sha256"], "value":value}),
        );
    }
    for (path, value) in forward.iter().rev() {
        reverse.insert(path.clone(), value.clone());
    }
    let one = project_audit_evidence(&ledger, &serde_json::json!({"documents":forward}))?;
    let two = project_audit_evidence(&ledger, &serde_json::json!({"documents":reverse}))?;
    assert_eq!(one, two);
    assert_eq!(
        serde_json::to_vec_pretty(&one)?,
        serde_json::to_vec_pretty(&two)?
    );
    let mut conflict = ledger.clone();
    conflict["events"] = serde_json::json!([{"initial_tier":"B","promoted_tier":"A",
        "passing_evidence":{"path":refs.keys().next().unwrap(),"sha256":"b".repeat(64)}}]);
    assert!(required_audit_references(&conflict).is_err());
    Ok(())
}

fn validate_tier_a_completion_data(
    fixture: &serde_json::Value,
    documents: &serde_json::Value,
) -> Result<()> {
    validate_omission_inventory_data(fixture, documents)?;
    let mut counts = [0, 0];
    for event in fixture["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["initial_tier"] == "A")
    {
        anyhow::ensure!(
            event["status"] == "restored",
            "unresolved Tier-A event: {}",
            event["event_id"]
        );
        validate_restored_event_evidence(
            event,
            documents,
            ReviewedEvidencePolicy::HistoricalTierA,
            true,
        )?;
        let lines: Vec<ReviewedSourceLine> =
            serde_json::from_value(event["diagnosis"]["reviewed_lines"].clone())?;
        if event["initial_tier"] == "A" {
            counts[0] += 1;
            counts[1] += lines.len();
        }
    }
    anyhow::ensure!(counts == [39, 69], "initial Tier-A completion differs");
    for extra in fixture["additional_source_losses"].as_array().unwrap() {
        anyhow::ensure!(
            extra["status"] == "restored",
            "additional reviewed source item unresolved"
        );
        let reference = &extra["passing_evidence"];
        let report = portable_evidence_document(documents, reference)?;
        anyhow::ensure!(
            report["unit_id"] == extra["unit_id"],
            "additional report unit differs"
        );
        for line in extra["reviewed_lines"].as_array().unwrap() {
            anyhow::ensure!(
                report["reviewed_lines"].as_array().unwrap().contains(line),
                "additional source glyph region absent"
            );
        }
    }
    Ok(())
}

#[test]
fn audit_tier_a_completion_requires_all_lines() {
    let fixture = audit_repair_ledger();
    validate_tier_a_completion_data(&fixture, &portable_audit_evidence()).unwrap();
    for kind in ["unresolved", "missing_line", "missing_evidence"] {
        let mut changed = fixture.clone();
        let event = changed["events"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["initial_tier"] == "A")
            .unwrap();
        match kind {
            "unresolved" => event["status"] = "unresolved".into(),
            "missing_line" => {
                event["diagnosis"]["reviewed_lines"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            "missing_evidence" => event["passing_evidence"]["path"] = serde_json::Value::Null,
            _ => unreachable!(),
        }
        assert!(
            validate_tier_a_completion_data(&changed, &portable_audit_evidence()).is_err(),
            "completion rejects {kind}"
        );
    }
}

#[test]
fn audit_omission_inventory_rejects_false_positive_and_duplicate_events() {
    let fixture = audit_repair_ledger();
    validate_omission_inventory_data(&fixture, &portable_audit_evidence()).unwrap();
    for kind in [
        "duplicate",
        "false_positive",
        "missing",
        "unreviewed",
        "changed_region",
    ] {
        let mut invalid = fixture.clone();
        let events = invalid["events"].as_array_mut().unwrap();
        match kind {
            "duplicate" => events.push(events[0].clone()),
            "false_positive" => events[0]["event_id"] = "miss_tiberius_gracchus_403_690".into(),
            "missing" => {
                events.pop();
            }
            "unreviewed" => events[0]["verdict"] = "pending".into(),
            "changed_region" => events[0]["source_regions"][0]["box"][0] = 0.0.into(),
            _ => unreachable!(),
        }
        assert!(
            validate_omission_inventory_data(&invalid, &portable_audit_evidence()).is_err(),
            "reject {kind}"
        );
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
struct ReviewedSourceLine {
    source: usize,
    source_page: u32,
    region: [u32; 4],
    role: String,
}

fn audit_fingerprint_image(record: &serde_json::Value) -> GrayImage {
    use base64::Engine;
    let region: [u32; 4] =
        std::array::from_fn(|i| record["glyph_region"][i].as_u64().unwrap() as u32);
    if let Some(encoded) = record["png_base64"].as_str() {
        image::load_from_memory(
            &base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .unwrap(),
        )
        .unwrap()
        .to_luma8()
    } else {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(record["gray_base64"].as_str().unwrap())
            .unwrap();
        GrayImage::from_raw(region[2] - region[0], region[3] - region[1], bytes).unwrap()
    }
}

#[test]
fn audit_source_fingerprints_match_reviewed_regions() {
    use sha2::Digest;
    let fixture = audit_repair_ledger();
    for record in fixture["source_fingerprints"].as_object().unwrap().values() {
        let image = audit_fingerprint_image(record);
        let region: [u32; 4] =
            std::array::from_fn(|i| record["glyph_region"][i].as_u64().unwrap() as u32);
        assert_eq!(
            image.dimensions(),
            (region[2] - region[0], region[3] - region[1])
        );
        assert_eq!(
            format!("{:x}", sha2::Sha256::digest(image.as_raw())),
            record["sha256"].as_str().unwrap()
        );
        assert!(region[2] <= record["column_dimensions"][0].as_u64().unwrap() as u32);
        assert!(region[3] <= record["column_dimensions"][1].as_u64().unwrap() as u32);
    }
}

#[test]
fn cato_major_folio_cleanup_preserves_rightmost_body_pixels() {
    let mut raster = GrayImage::from_pixel(1948, 2980, Luma([255]));
    raster.put_pixel(1710, 350, Luma([17]));
    let corrected = preserve_known_body_crop_edge(
        0,
        432,
        &raster,
        Column {
            left: 907,
            right: 1710,
        },
    );
    assert_eq!([corrected.left, corrected.right], [907, 1711]);
    assert_eq!(
        crop_column(&raster, &corrected).get_pixel(803, 350),
        raster.get_pixel(1710, 350)
    );
    for (source, page, left, right) in [
        (1, 432, 907, 1710),
        (0, 431, 907, 1710),
        (0, 432, 908, 1710),
        (0, 432, 907, 1712),
    ] {
        let result = preserve_known_body_crop_edge(source, page, &raster, Column { left, right });
        assert_eq!([result.left, result.right], [left, right]);
    }
    let other = GrayImage::from_pixel(1948, 2981, Luma([255]));
    let unchanged = preserve_known_body_crop_edge(
        0,
        432,
        &other,
        Column {
            left: 907,
            right: 1710,
        },
    );
    assert_eq!([unchanged.left, unchanged.right], [907, 1710]);
}

#[test]
fn known_body_fragments_preserve_faint_latin_pixels() {
    let fixture = audit_repair_ledger();
    for record in fixture["planned_fragment_repairs"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| {
            ["3.1", "4.2"]
                .iter()
                .any(|step| r["responsible_step"] == *step)
        })
    {
        let source = record["source"].as_u64().unwrap() as usize;
        let page = record["source_page"].as_u64().unwrap() as u32;
        let dimensions = (
            record["column_dimensions"][0].as_u64().unwrap() as u32,
            record["column_dimensions"][1].as_u64().unwrap() as u32,
        );
        let pair: Vec<_> = record["observed_pairs"]
            .as_array()
            .unwrap()
            .iter()
            .map(ledger_band)
            .collect();
        let protected = ledger_band(&record["retained_band"]);
        let mut column = GrayImage::from_pixel(dimensions.0, dimensions.1, Luma([255]));
        for band in &pair {
            let width = if band.bottom - band.top < 8 {
                LINE_MIN_INK + 1
            } else {
                30
            };
            for y in band.top..band.bottom {
                for x in 10..10 + width {
                    column.put_pixel(x, y, Luma([170]));
                }
            }
        }
        if protected.top < pair[0].top {
            column.put_pixel(15, protected.top, Luma([170]));
        }
        assert_eq!(line_bands(&column, LINE_MIN_INK), pair);
        let joined = coalesce_known_body_fragments(source, page, dimensions, &pair);
        assert_eq!(joined, vec![protected]);
        let metrics = LineMetrics {
            height_med: 38,
            height_mad: 1,
            width_med: 700,
        };
        let pages = vec![(column.clone(), joined)];
        assert_eq!(strips_in_reading_order(&pages, &metrics).len(), 1);
        assert_eq!(
            strips_in_reading_order(&[(column.clone(), pair.clone())], &metrics).len(),
            1,
            "small fragment would be filtered"
        );
        let geom = TrimGeom {
            dpi: 300,
            tb_w: dimensions.0,
            tb_h: 100,
            pad_x: 0,
            pad_y: 0,
        };
        let canvases = assert_scaled_stream(&pages, &metrics, 1.0, &geom, 100, 100);
        assert_eq!(
            canvases,
            vec![image::imageops::crop_imm(
                &column,
                0,
                protected.top,
                dimensions.0,
                protected.bottom - protected.top
            )
            .to_image()]
        );
        for (other_source, other_page, other_dimensions) in [
            (2, page, dimensions),
            (source, page + 1, dimensions),
            (source, page, (dimensions.0 + 1, dimensions.1)),
        ] {
            assert_eq!(
                coalesce_known_body_fragments(other_source, other_page, other_dimensions, &pair),
                pair
            );
        }
        let mut other = pair.clone();
        other[1].bottom += 1;
        assert_eq!(
            coalesce_known_body_fragments(source, page, dimensions, &other),
            other
        );
    }
}

#[test]
fn known_body_crop_corrections_preserve_clipped_glyphs() {
    let fixture = audit_repair_ledger();
    for record in fixture["candidate_crop_cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| {
            ["4.1", "4.2"]
                .iter()
                .any(|step| r["responsible_step"] == *step)
        })
    {
        let source = record["source"].as_u64().unwrap() as usize;
        let page = record["source_page"].as_u64().unwrap() as u32;
        let dimensions = (
            record["raster"][0].as_u64().unwrap() as u32,
            record["raster"][1].as_u64().unwrap() as u32,
        );
        let before = ledger_crop_edges(&serde_json::json!({"crop_edges":record["detected_crop"]}));
        let after = ledger_crop_edges(&serde_json::json!({"crop_edges":record["proposed_crop"]}));
        let mut raster = GrayImage::from_pixel(dimensions.0, dimensions.1, Luma([255]));
        for x in [after[0], after[1] - 1] {
            raster.put_pixel(x, 500, Luma([11]));
        }
        let corrected = preserve_known_body_crop_edge(
            source,
            page,
            &raster,
            Column {
                left: before[0],
                right: before[1],
            },
        );
        assert_eq!([corrected.left, corrected.right], after);
        let crop = crop_column(&raster, &corrected);
        assert_eq!(crop.get_pixel(0, 500), &Luma([11]));
        assert_eq!(crop.get_pixel(crop.width() - 1, 500), &Luma([11]));
        for (s, p, left, right) in [
            (2, page, before[0], before[1]),
            (source, page + 1, before[0], before[1]),
            (source, page, before[0] + 1, before[1]),
        ] {
            let kept = preserve_known_body_crop_edge(s, p, &raster, Column { left, right });
            assert_eq!([kept.left, kept.right], [left, right]);
        }
        let other = GrayImage::from_pixel(dimensions.0 + 1, dimensions.1, Luma([255]));
        let kept = preserve_known_body_crop_edge(
            source,
            page,
            &other,
            Column {
                left: before[0],
                right: before[1],
            },
        );
        assert_eq!([kept.left, kept.right], before);
    }
}

#[test]
fn known_source_body_bands_survive_furniture_filters() {
    let mut groups = std::collections::BTreeMap::new();
    for r in SOURCE_BODY_BANDS {
        groups
            .entry((
                r.source,
                r.page,
                r.unit_id,
                r.dimensions,
                r.observed.top,
                r.observed.bottom,
            ))
            .or_insert_with(Vec::new)
            .push(r);
    }
    for records in groups.values() {
        let record = records[0];
        let specifications: Vec<_> = (1000..=2000)
            .step_by(100)
            .filter(|top| *top + 40 < record.observed.top || *top > record.observed.bottom)
            .map(|top| (top, top + 40, 10, 700))
            .chain(std::iter::once((
                record.observed.top,
                record.observed.bottom,
                10,
                700,
            )))
            .collect();
        let col = column(record.dimensions.1, record.dimensions.0, &specifications);
        let raw = line_bands(&col, LINE_MIN_INK);
        assert!(raw.contains(&record.observed));
        let furniture = drop_furniture(&raw, col.height());
        if record.observed.bottom - record.observed.top < 100 {
            assert!(
                !furniture.contains(&record.observed),
                "ordinary reviewed furniture example"
            );
        }
        let kept: Vec<_> = raw
            .iter()
            .copied()
            .filter(|b| *b != record.observed)
            .collect();
        let rescued = retain_known_source_body_bands(
            None,
            record.source,
            record.page,
            record.dimensions,
            &raw,
            &kept,
        );
        for r in records {
            assert_eq!(rescued.iter().filter(|b| **b == r.protected).count(), 1);
        }
        assert_eq!(
            retain_known_source_body_bands(
                Some(record.unit_id),
                record.source,
                record.page,
                record.dimensions,
                &rescued,
                &rescued
            ),
            rescued
        );
        assert_eq!(
            retain_known_source_body_bands(
                Some("adjacent_comparison"),
                record.source,
                record.page,
                record.dimensions,
                &raw,
                &kept
            ),
            kept
        );
        for (source, page, dimensions) in [
            (2, record.page, record.dimensions),
            (record.source, record.page + 1, record.dimensions),
            (
                record.source,
                record.page,
                (record.dimensions.0 + 1, record.dimensions.1),
            ),
        ] {
            assert_eq!(
                retain_known_source_body_bands(None, source, page, dimensions, &raw, &kept),
                kept
            );
        }
        assert_eq!(
            retain_known_source_body_bands(
                None,
                record.source,
                record.page,
                record.dimensions,
                &kept,
                &kept
            ),
            kept
        );
        let parent = retain_known_source_body_bands(
            None,
            record.source,
            record.page,
            record.dimensions,
            &raw,
            &raw,
        );
        assert_eq!(
            retain_known_source_body_bands(
                None,
                record.source,
                record.page,
                record.dimensions,
                &raw,
                &parent
            ),
            parent
        );
        let pages = vec![(col.clone(), rescued)];
        let metrics = line_metrics(&pages);
        let geom = TrimGeom {
            dpi: 300,
            tb_w: record.dimensions.0,
            tb_h: record.dimensions.1,
            pad_x: 0,
            pad_y: 0,
        };
        assert_scaled_stream(&pages, &metrics, 1.0, &geom, geom.tb_h, geom.tb_h);
        for r in records {
            let before = image::imageops::crop_imm(
                &col,
                0,
                r.protected.top,
                col.width(),
                r.protected.bottom - r.protected.top,
            )
            .to_image();
            let after = image::imageops::crop_imm(
                &pages[0].0,
                0,
                r.protected.top,
                col.width(),
                r.protected.bottom - r.protected.top,
            )
            .to_image();
            assert_eq!(before, after);
            assert_eq!(strips_in_reading_order(&pages,&metrics).iter().filter(|s|s.band.top<=r.protected.top && s.band.bottom>=r.protected.bottom).count(),1);
        }
    }
    let record = SOURCE_BODY_BANDS
        .iter()
        .find(|r| r.source == 0 && r.page == 292)
        .unwrap();
    let col = column(
        record.dimensions.1,
        record.dimensions.0,
        &[
            (280, 312, 10, 510),
            (335, 368, 10, 710),
            (385, 417, 10, 710),
            (434, 466, 10, 710),
        ],
    );
    let raw = line_bands(&col, LINE_MIN_INK);
    let metrics = LineMetrics {
        height_med: 33,
        height_mad: 1,
        width_med: 700,
    };
    let trimmed = drop_interior_running_head(&col, &raw, &metrics);
    assert!(!trimmed.contains(&record.observed));
    let protected = retain_known_source_body_bands(
        Some(record.unit_id),
        0,
        292,
        record.dimensions,
        &raw,
        &trimmed,
    );
    assert!(protected.contains(&record.protected));
    let shared = drop_shared_tail(&col, &raw, Some(320.0 / col.height() as f32), &metrics);
    assert_eq!(
        retain_known_source_body_bands(
            Some("adjacent_comparison"),
            0,
            292,
            record.dimensions,
            &raw,
            &shared
        ),
        shared
    );
}

fn assert_unit_lines_restored(id: &str) {
    assert_unit_lines_restored_with(id, &[], None, false);
}

fn assert_unit_lines_restored_with(
    id: &str,
    selected: &[String],
    stage: Option<&str>,
    candidate: bool,
) {
    let fixture = audit_repair_ledger();
    let events: Vec<_> = fixture["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| {
            e["unit_id"] == id
                && ((candidate && selected.iter().any(|s| e["event_id"] == *s))
                    || ["installed", "restored"]
                        .iter()
                        .any(|status| e["status"] == *status))
        })
        .collect();
    assert!(!events.is_empty());
    let mut lines: Vec<ReviewedSourceLine> = events
        .iter()
        .flat_map(|e| {
            serde_json::from_value::<Vec<ReviewedSourceLine>>(
                e["diagnosis"]["reviewed_lines"].clone(),
            )
            .unwrap()
        })
        .collect();
    for event in &events {
        if event["diagnosis"]["reviewed_neighbor_lines"].is_array() {
            lines.extend(
                serde_json::from_value::<Vec<ReviewedSourceLine>>(
                    event["diagnosis"]["reviewed_neighbor_lines"].clone(),
                )
                .unwrap(),
            );
        }
    }
    if id == "romulus" {
        lines.extend(
            serde_json::from_value::<Vec<ReviewedSourceLine>>(fixture["c6_reviewed_lines"].clone())
                .unwrap(),
        );
    }
    for extra in fixture["additional_source_losses"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| {
            r["unit_id"] == id
                && ["installed", "restored"]
                    .iter()
                    .any(|status| r["status"] == *status)
        })
    {
        lines.extend(
            serde_json::from_value::<Vec<ReviewedSourceLine>>(extra["reviewed_lines"].clone())
                .unwrap(),
        );
    }
    lines.sort_by_key(|l| (l.source_page, l.region[1]));
    lines.dedup_by_key(|l| (l.source_page, l.region));
    let policies: std::collections::BTreeMap<_, _> = events
        .iter()
        .filter(|e| e["initial_tier"] == "B")
        .flat_map(|e| {
            serde_json::from_value::<Vec<ReviewedSourceLine>>(
                e["diagnosis"]["reviewed_lines"].clone(),
            )
            .unwrap()
        })
        .map(|l| {
            (
                (l.source, l.source_page, l.region),
                ReviewedEvidencePolicy::PromotedTierB,
            )
        })
        .collect();
    let (pages, metrics, traces) = audit_traced_unit(id);
    let sequence: Vec<_> = traces
        .iter()
        .filter(|t| t.retained_index.is_some())
        .map(|t| t.page)
        .collect();
    let seam: &[u32] = match id {
        "phocion" => &[296, 299],
        "marcius_coriolanus" => &[291, 292],
        "brutus" => &[623, 624],
        _ => &[],
    };
    if !seam.is_empty() {
        assert!(sequence.windows(seam.len()).any(|w| w == seam));
    }
    let ledger: Vec<ReviewedBands> =
        serde_json::from_str(include_str!("fixtures/closing_body_bands.json")).unwrap();
    let originals = lines
        .iter()
        .map(|l| (l.source, l.source_page))
        .chain(
            ledger
                .iter()
                .filter(|r| {
                    traces.iter().any(|t| {
                        t.source == r.source && t.page == r.page && t.retained_index.is_some()
                    })
                })
                .map(|r| (r.source, r.page)),
        )
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|(s, p)| ((s, p), real_source_image(s, p)))
        .collect();
    let context = ReviewedLineContext {
        traces: &traces,
        originals: &originals,
        pages: &pages,
        metrics: &metrics,
        scale: production_body_scale(),
        geom: TrimGeom::default(),
    };
    assert_reviewed_band_ledger(id, &ledger, &pages, &metrics, &traces, &originals);
    let composition = assert_reviewed_source_lines_survive_with_policies(
        &lines,
        &context,
        &policies,
        ReviewedEvidencePolicy::HistoricalTierA,
    );
    let dir = std::path::Path::new("tmp/pdfs/inner_tomi_audit_repairs/restoration-current")
        .join(stage.unwrap_or_else(|| fixture["restoration_stage"].as_str().unwrap()));
    std::fs::create_dir_all(&dir).unwrap();
    use sha2::Digest;
    let canvas_hashes: Vec<_> = composition.canvases.iter().enumerate().map(|(i,c)|serde_json::json!({"canvas":i+1,"width":c.width(),"height":c.height(),"sha256":format!("{:x}",sha2::Sha256::digest(c.as_raw()))})).collect();
    let touched: std::collections::BTreeSet<_> = composition
        .line_coverage
        .iter()
        .flat_map(|l| l["canvas_owners"].as_array().unwrap())
        .map(|v| v.as_u64().unwrap() as usize)
        .collect();
    for i in touched {
        composition.canvases[i - 1]
            .save(dir.join(format!("{id}-canvas-{i:03}.png")))
            .unwrap();
    }
    let mut evidence = serde_json::json!({"unit_id":id,"scale":context.scale,"events":events.iter().map(|e|e["event_id"].clone()).collect::<Vec<_>>(),
        "reviewed_lines":lines,"line_coverage":composition.line_coverage,"canvas_ranges":composition.canvas_ranges,"canvas_hashes":canvas_hashes,"source_pages":traces,"verification":"original pixels, unique retained bands and strips, source order, and returned canvases passed"});
    if !selected.is_empty() {
        evidence["schema_version"] = serde_json::json!(2);
        evidence["verification_policy"] = serde_json::json!("promoted_tier_b");
        evidence["selected_events"] = serde_json::json!(selected);
        evidence["selected_line_count"] = serde_json::json!(events
            .iter()
            .filter(|e| selected.iter().any(|s| e["event_id"] == *s))
            .map(|e| e["measured_line_count"]
                .as_u64()
                .or_else(|| e["missing_line_count"].as_u64())
                .unwrap())
            .sum::<u64>());
    }
    std::fs::write(
        dir.join(format!("{id}.json")),
        serde_json::to_vec_pretty(&evidence).unwrap(),
    )
    .unwrap();
    println!(
        "{id}: {} events / {} reviewed regions preserved",
        events.len(),
        lines.len()
    );
}

#[test]
#[ignore = "requires original source PDFs and production-scale composition"]
fn romulus_c1_c6_lines_restored() {
    assert_unit_lines_restored("romulus");
}

#[test]
fn known_source_verse_label_survives_ink_floor() {
    let fixture = audit_repair_ledger();
    let glyphs = audit_fingerprint_image(&fixture["source_fingerprints"]["cato_minor_verse_label"]);
    let mut col = column(2838, 788, &[(1964, 1994, 100, 160), (2080, 2110, 100, 160)]);
    image::imageops::replace(&mut col, &glyphs, 3, 2026);
    let bands = vec![
        Band {
            top: 1964,
            bottom: 1994,
        },
        Band {
            top: 2026,
            bottom: 2048,
        },
        Band {
            top: 2080,
            bottom: 2110,
        },
    ];
    let metrics = LineMetrics {
        height_med: 38,
        height_mad: 1,
        width_med: 700,
    };
    let pages = vec![(col.clone(), bands.clone())];
    let strips = strips_in_reading_order(&pages, &metrics);
    assert_eq!(strips.len(), 3);
    assert!(is_known_source_verse_label(&col, &strips[1].band));
    let geom = TrimGeom {
        dpi: 300,
        tb_w: 788,
        tb_h: 100,
        pad_x: 0,
        pad_y: 0,
    };
    let canvases = assert_scaled_stream(&pages, &metrics, 1.0, &geom, 100, 100);
    let flattened: Vec<_> = canvases
        .iter()
        .flat_map(|c| c.as_raw().iter().copied())
        .collect();
    let expected: Vec<_> = strips
        .iter()
        .flat_map(|s| {
            image::imageops::crop_imm(&col, 0, s.band.top, 788, s.band.bottom - s.band.top)
                .to_image()
                .into_raw()
        })
        .collect();
    assert_eq!(flattened, expected);
    let mut changed = col.clone();
    changed.put_pixel(3, 2026, Luma([col.get_pixel(3, 2026)[0] ^ 1]));
    assert!(!is_known_source_verse_label(&changed, &strips[1].band));
    assert_eq!(
        strips_in_reading_order(&[(changed, bands)], &metrics).len(),
        2
    );
    assert!(!is_known_source_verse_label(
        &col,
        &Band {
            top: 2011,
            bottom: 2064
        }
    ));
    for (w, h) in [(789, 2838), (788, 2839)] {
        let mut other = GrayImage::from_pixel(w, h, Luma([255]));
        image::imageops::replace(&mut other, &glyphs, 3, 2026);
        assert!(!is_known_source_verse_label(&other, &strips[1].band));
    }
}

#[test]
#[ignore = "requires original source PDFs and production-scale composition"]
fn coriolanus_c2_lines_restored() {
    assert_unit_lines_restored("marcius_coriolanus");
}

#[test]
#[ignore = "requires original source PDFs and production-scale composition"]
fn phocion_c3_line_restored() {
    assert_unit_lines_restored("phocion");
}

#[test]
#[ignore = "requires original source PDFs and production-scale composition"]
fn brutus_c5_line_restored() {
    assert_unit_lines_restored("brutus");
}

fn comparison_cluster_fixture() -> (PageBands, LineMetrics, TrimGeom) {
    let fixture = audit_repair_ledger();
    let sample = audit_fingerprint_image(&fixture["source_fingerprints"]["comparison_whole_strip"]);
    let mut col = column(2836, 817, &[(1468, 1498, 10, 800), (1649, 1679, 10, 800)]);
    image::imageops::replace(&mut col, &sample, 0, 1507);
    (
        (
            col,
            vec![
                Band {
                    top: 1468,
                    bottom: 1498,
                },
                Band {
                    top: 1516,
                    bottom: 1647,
                },
                Band {
                    top: 1649,
                    bottom: 1679,
                },
            ],
        ),
        LineMetrics {
            height_med: 30,
            height_mad: 1,
            width_med: 790,
        },
        TrimGeom {
            dpi: 300,
            tb_w: 817,
            tb_h: 200,
            pad_x: 0,
            pad_y: 0,
        },
    )
}

#[test]
fn merged_strip_wraps_at_complete_line_boundary() {
    let (page, metrics, geom) = comparison_cluster_fixture();
    let strips = strips_in_reading_order(std::slice::from_ref(&page), &metrics);
    assert!(matches!(strips[1].class, BandClass::Tall));
    assert!(is_known_whole_source_strip(&page.0, &strips[1].band));
    assert!(!is_known_whole_source_strip(
        &page.0,
        &Band {
            top: 1508,
            bottom: 1648
        }
    ));
    let sample = image::imageops::crop_imm(&page.0, 0, 1507, 817, 141).to_image();
    for (w, h) in [(818, 2836), (817, 2837)] {
        let mut other = GrayImage::from_pixel(w, h, Luma([255]));
        image::imageops::replace(&mut other, &sample, 0, 1507);
        assert!(!is_known_whole_source_strip(&other, &strips[1].band));
    }
    let canvases =
        assert_scaled_stream(std::slice::from_ref(&page), &metrics, 1.0, &geom, 100, 200);
    assert_eq!(
        canvases.iter().map(GrayImage::height).collect::<Vec<_>>(),
        vec![39, 172]
    );
    assert_eq!(
        image::imageops::crop_imm(&canvases[1], 0, 0, 817, 141).to_image(),
        image::imageops::crop_imm(&page.0, 0, 1507, 817, 141).to_image()
    );
    let oversized = assert_scaled_stream(&[page], &metrics, 1.0, &geom, 100, 100);
    assert!(oversized.len() >= 3);
    assert!(oversized.iter().all(|c| c.height() <= 100));
}

#[test]
fn unaffected_ordinary_line_packing_is_unchanged() {
    let (mut page, metrics, geom) = comparison_cluster_fixture();
    let band = Band {
        top: 1507,
        bottom: 1648,
    };
    page.0
        .put_pixel(0, 1507, Luma([page.0.get_pixel(0, 1507)[0] ^ 1]));
    assert!(!is_known_whole_source_strip(&page.0, &band));
    assert!(!is_known_whole_source_strip(
        &page.0,
        &Band {
            top: 1508,
            bottom: 1648
        }
    ));
    let changed = assert_scaled_stream(&[page], &metrics, 1.0, &geom, 100, 200);
    assert!(
        changed[0].height() > 39,
        "unrecognized Tall strip retains fill-splitting"
    );
    let (page, metrics) = page_and_metrics(
        160,
        100,
        &[
            (10, 30, 5, 95),
            (40, 60, 5, 95),
            (70, 90, 5, 95),
            (100, 120, 5, 95),
        ],
    );
    let geom = TrimGeom {
        dpi: 300,
        tb_w: 100,
        tb_h: 60,
        pad_x: 0,
        pad_y: 0,
    };
    let ordinary = assert_scaled_stream(&[page], &metrics, 1.0, &geom, 60, 60);
    assert_eq!(
        ordinary.iter().map(GrayImage::height).collect::<Vec<_>>(),
        vec![55, 55]
    );
}

/// A synthetic Antonius p562 column: the measured 22-line block occupies its
/// cut-point strip 249..1364, flanked by one ordinary line above and below, so
/// `strips_in_reading_order` yields the block as a single Tall strip carrying the
/// stored source pixels of its fingerprint region.
fn antonius_block_fixture() -> (PageBands, LineMetrics, TrimGeom) {
    let fixture = audit_repair_ledger();
    let sample = audit_fingerprint_image(&fixture["source_fingerprints"]["antonius_whole_strip"]);
    let mut col = column(2835, 815, &[(198, 243, 10, 800), (1372, 1417, 10, 800)]);
    image::imageops::replace(&mut col, &sample, 0, 249);
    (
        (
            col,
            vec![
                Band {
                    top: 198,
                    bottom: 243,
                },
                Band {
                    top: 255,
                    bottom: 1356,
                },
                Band {
                    top: 1372,
                    bottom: 1417,
                },
            ],
        ),
        LineMetrics {
            height_med: 48,
            height_mad: 2,
            width_med: 760,
        },
        TrimGeom {
            dpi: 300,
            tb_w: 815,
            tb_h: 1300,
            pad_x: 0,
            pad_y: 0,
        },
    )
}

#[test]
fn antonius_whole_block_wraps_without_bisecting_glyphs() {
    let (page, metrics, geom) = antonius_block_fixture();
    let strips = strips_in_reading_order(std::slice::from_ref(&page), &metrics);
    assert_eq!(strips.len(), 3);
    assert!(matches!(strips[1].class, BandClass::Tall));
    let block = Band {
        top: 249,
        bottom: 1364,
    };
    assert_eq!(strips[1].band, block);
    assert!(is_known_whole_source_strip(&page.0, &block));
    // Off-by-one strip intervals are not the measured block.
    assert!(!is_known_whole_source_strip(
        &page.0,
        &Band {
            top: 250,
            bottom: 1364
        }
    ));
    assert!(!is_known_whole_source_strip(
        &page.0,
        &Band {
            top: 249,
            bottom: 1363
        }
    ));
    // Altered column dimensions no longer match the production column.
    let sample = image::imageops::crop_imm(&page.0, 0, 249, 815, 1115).to_image();
    for (w, h) in [(816, 2835), (815, 2836)] {
        let mut other = GrayImage::from_pixel(w, h, Luma([255]));
        image::imageops::replace(&mut other, &sample, 0, 249);
        assert!(!is_known_whole_source_strip(&other, &block));
    }
    // A single flipped source pixel breaks the fingerprint.
    let mut tampered = page.0.clone();
    let shade = tampered.get_pixel(0, 249)[0];
    tampered.put_pixel(0, 249, Luma([shade ^ 1]));
    assert!(!is_known_whole_source_strip(&tampered, &block));

    // A partially filled first canvas: the recognized block wraps whole onto a fresh
    // canvas rather than being fill-split, and that canvas reproduces the source strip.
    let canvases =
        assert_scaled_stream(std::slice::from_ref(&page), &metrics, 1.0, &geom, 100, 1265);
    assert_eq!(canvases.len(), 2, "block wraps whole to a fresh canvas");
    assert_eq!(
        image::imageops::crop_imm(&canvases[1], 0, 0, 815, 1115).to_image(),
        sample,
        "the wrapped block is the whole source strip, not bisected"
    );

    // An undersized budget invokes splitting and cannot promise whole glyphs.
    let oversized = assert_scaled_stream(&[page], &metrics, 1.0, &geom, 400, 400);
    assert!(oversized.len() >= 3);
    assert!(oversized.iter().all(|c| c.height() <= 400));
}

#[test]
#[ignore = "requires original source PDFs and production-scale composition"]
fn comparison_s2_line_is_whole() {
    let fixture = audit_repair_ledger();
    let case = fixture["furniture_cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["finding_ids"] == serde_json::json!(["S2"]))
        .unwrap();
    let (pages, metrics, traces) = audit_traced_unit("comparatio_demosthenis_et_ciceronis");
    let lines: Vec<ReviewedSourceLine> =
        serde_json::from_value(case["proposed_correction"]["reviewed_lines"].clone()).unwrap();
    let originals = lines
        .iter()
        .map(|l| {
            (
                (l.source, l.source_page),
                real_source_image(l.source, l.source_page),
            )
        })
        .collect();
    let context = ReviewedLineContext {
        traces: &traces,
        originals: &originals,
        pages: &pages,
        metrics: &metrics,
        scale: production_body_scale(),
        geom: TrimGeom::default(),
    };
    let composition = assert_reviewed_source_lines_survive(&lines, &context, true);
    let dir = std::path::Path::new("tmp/pdfs/inner_tomi_audit_repairs/s2-current");
    std::fs::create_dir_all(dir).unwrap();
    use sha2::Digest;
    let hashes: Vec<_> = composition.canvases.iter().enumerate().map(|(i,c)| {
        c.save(dir.join(format!("comparison-canvas-{:03}.png",i+1))).unwrap();
        serde_json::json!({"canvas":i+1,"sha256":format!("{:x}",sha2::Sha256::digest(c.as_raw()))})
    }).collect();
    std::fs::write(dir.join("verification.json"),serde_json::to_vec_pretty(&serde_json::json!({"line_coverage":composition.line_coverage,"canvas_hashes":hashes,"scale":context.scale})).unwrap()).unwrap();
}

#[test]
#[ignore = "requires original source PDFs and production-scale composition"]
fn audit_tier_a_source_lines_restored() {
    let fixture = audit_repair_ledger();
    let mut units = std::collections::BTreeSet::new();
    for event in fixture["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["initial_tier"] == "A")
    {
        if ["installed", "restored"]
            .iter()
            .any(|status| event["status"] == *status)
        {
            units.insert(event["unit_id"].as_str().unwrap());
        } else {
            println!(
                "pending {}: {} source lines",
                event["event_id"], event["missing_line_count"]
            );
        }
    }
    assert!(!units.is_empty());
    for id in units {
        assert_unit_lines_restored(id);
    }
}

struct ReviewedLineContext<'a> {
    traces: &'a [RetainedSourcePageTrace],
    originals: &'a std::collections::BTreeMap<(usize, u32), GrayImage>,
    pages: &'a [PageBands],
    metrics: &'a LineMetrics,
    scale: f32,
    geom: TrimGeom,
}

struct ReviewedComposition {
    canvases: Vec<GrayImage>,
    canvas_ranges: Vec<CanvasRange>,
    line_coverage: Vec<serde_json::Value>,
}

fn assert_reviewed_band_ledger(
    id: &str,
    ledger: &[ReviewedBands],
    pages: &[PageBands],
    metrics: &LineMetrics,
    traces: &[RetainedSourcePageTrace],
    originals: &std::collections::BTreeMap<(usize, u32), GrayImage>,
) {
    for record in ledger.iter().filter(|r| {
        traces
            .iter()
            .any(|t| t.source == r.source && t.page == r.page && t.retained_index.is_some())
            && !(r.page == 434 && id != "cato_major")
    }) {
        let owned_body: Vec<_> = record
            .body
            .iter()
            .filter(|rows| {
                let signatures: Vec<_> = SOURCE_BODY_BANDS
                    .iter()
                    .filter(|r| {
                        r.source == record.source
                            && r.page == record.page
                            && [r.protected.top, r.protected.bottom] == **rows
                    })
                    .collect();
                signatures.is_empty() || signatures.iter().any(|r| r.unit_id == id)
            })
            .collect();
        if owned_body.is_empty() && !record.body.is_empty() {
            continue;
        }
        let trace = retained_page(traces, record.source, record.page).unwrap();
        let index = trace.retained_index.unwrap();
        let retained = &pages[index];
        assert_eq!(record.dpi, BODY_DPI);
        assert_eq!(
            record.column, trace.crop,
            "review ledger uses production crop"
        );
        assert!(
            record.body.windows(2).all(|w| w[0][1] <= w[1][0]),
            "sorted non-overlapping body entries"
        );
        for &&[top, bottom] in &owned_body {
            assert!(top < bottom);
            let mapped_top = trace.map_row(top).unwrap();
            let mapped_bottom = trace.map_row(bottom - 1).unwrap() + 1;
            assert_eq!(mapped_bottom - mapped_top, bottom - top);
            assert_eq!(
                retained
                    .1
                    .iter()
                    .filter(|b| b.top == mapped_top && b.bottom == mapped_bottom)
                    .count(),
                1,
                "exact retained-band multiplicity: {id}, source {} page {}, expected {mapped_top}..{mapped_bottom}, retained {:?}",record.source,record.page,retained.1
            );
            assert_source_region_pixels_survive(
                &originals[&(record.source, record.page)],
                &retained.0,
                trace,
                [trace.crop[0], top, trace.crop[1], bottom],
            );
        }
        for &[top, bottom] in &record.excluded {
            if (id == "cato_major" && record.body.first().is_some_and(|b| top < b[0]))
                || (id == "comparatio_aristidis_et_catonis"
                    && record.body.last().is_some_and(|b| bottom > b[1]))
            {
                continue;
            }
            assert!(
                !strips_in_reading_order(std::slice::from_ref(retained), metrics)
                    .iter()
                    .any(|s| s.band.top < bottom && s.band.bottom > top),
                "strip overlaps excluded furniture"
            );
        }
        assert!(record.noise.iter().all(|b| b[0] < b[1] && b[1] - b[0] < 12));
    }
}

fn assert_source_region_pixels_survive(
    original: &GrayImage,
    retained: &GrayImage,
    trace: &RetainedSourcePageTrace,
    region: [u32; 4],
) {
    let [left, top, right, bottom] = region;
    for y in top..bottom {
        for x in left..right {
            assert_eq!(
                original.get_pixel(x, y),
                retained.get_pixel(x - trace.crop[0], trace.map_row(y).unwrap()),
                "original Latin pixels at {}:{}, {x},{y}",
                trace.source,
                trace.page
            );
        }
    }
}

/// Verify reviewed glyph regions once in the unscaled strips and the composed stream.
fn assert_reviewed_source_lines_survive(
    lines: &[ReviewedSourceLine],
    context: &ReviewedLineContext<'_>,
    whole_line: bool,
) -> ReviewedComposition {
    assert_reviewed_source_lines_survive_with_policies(
        lines,
        context,
        &std::collections::BTreeMap::new(),
        if whole_line {
            ReviewedEvidencePolicy::PromotedTierB
        } else {
            ReviewedEvidencePolicy::HistoricalTierA
        },
    )
}

type ReviewedRegionKey = (usize, u32, [u32; 4]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReviewedEvidencePolicy {
    HistoricalTierA,
    PromotedTierB,
}

fn assert_reviewed_source_lines_survive_with_policies(
    lines: &[ReviewedSourceLine],
    context: &ReviewedLineContext<'_>,
    policies: &std::collections::BTreeMap<ReviewedRegionKey, ReviewedEvidencePolicy>,
    default_policy: ReviewedEvidencePolicy,
) -> ReviewedComposition {
    assert!(!lines.is_empty(), "reviewed source regions are required");
    let ReviewedLineContext {
        traces,
        originals,
        pages,
        metrics,
        scale,
        geom,
    } = context;
    let canvases = assert_scaled_stream(
        pages,
        metrics,
        *scale,
        geom,
        geom.first_body_budget(),
        geom.body_budget(),
    );
    let (strips, canvas_ranges, canvases) =
        ranges_for_composed_canvases(pages, metrics, *scale, geom, canvases);
    let mut previous = None;
    let mut seen = std::collections::BTreeSet::new();
    let mut line_coverage = Vec::new();
    for line in lines {
        assert!(
            seen.insert((line.source, line.source_page, line.region)),
            "repeated reviewed region"
        );
        assert_eq!(
            traces
                .iter()
                .filter(|t| t.source == line.source && t.page == line.source_page)
                .count(),
            1,
            "source page identity occurs once in trace"
        );
        let trace =
            retained_page(traces, line.source, line.source_page).expect("source identity traced");
        let index = trace.retained_index.expect("reviewed page retained");
        let [left, top, right, bottom] = line.region;
        assert!(left < right && top < bottom);
        assert!(
            left >= trace.crop[0] && right <= trace.crop[1],
            "reviewed glyphs inside column"
        );
        let mapped_top = trace.map_row(top).expect("reviewed top survives");
        let mapped_bottom = trace.map_row(bottom - 1).expect("reviewed bottom survives") + 1;
        assert_eq!(
            mapped_bottom - mapped_top,
            bottom - top,
            "no deleted Latin rows"
        );
        assert_eq!(
            pages[index]
                .1
                .iter()
                .filter(|b| b.top <= mapped_top && b.bottom >= mapped_bottom)
                .count(),
            1,
            "whole reviewed line covered once: source {} page {} region {:?}, retained {:?}",
            line.source,
            line.source_page,
            line.region,
            pages[index].1
        );
        let owners: Vec<_> = strips
            .iter()
            .filter(|(s, _, _)| {
                s.page_img_index == index
                    && s.band.top <= mapped_top
                    && s.band.bottom >= mapped_bottom
            })
            .collect();
        assert_eq!(owners.len(), 1, "whole reviewed line extracted once");
        let position = (index, top);
        assert!(
            previous.is_none_or(|p| p < position),
            "reviewed source order"
        );
        previous = Some(position);
        let original = &originals[&(line.source, line.source_page)];
        assert!(
            (top..bottom)
                .any(|y| (left..right).any(|x| original.get_pixel(x, y)[0] <= INK_THRESHOLD)),
            "reviewed region contains glyph ink"
        );
        assert_source_region_pixels_survive(original, &pages[index].0, trace, line.region);
        let (strip, start, end) = owners[0];
        let ratio = (*end - *start) as f64 / (strip.band.bottom - strip.band.top) as f64;
        let lo = start + ((mapped_top - strip.band.top) as f64 * ratio).floor() as u32;
        let hi = start + ((mapped_bottom - strip.band.top) as f64 * ratio).ceil() as u32;
        assert!(
            (lo..hi).all(|row| canvas_ranges
                .iter()
                .filter(|(_, a, b)| *a <= row && row < *b)
                .count()
                == 1),
            "all reviewed glyph rows returned once on canvases"
        );
        assert!(
            ((right - trace.crop[0]) as f32 * scale).ceil() as u32 + geom.pad_x <= geom.tb_w,
            "reviewed right glyphs fit output text width: source {} page {} region {:?}",
            line.source,
            line.source_page,
            line.region
        );
        line_coverage.push(serde_json::json!({"source":line.source,"source_page":line.source_page,"region":line.region,
            "scaled_row_stream":[lo,hi],"canvas_owners":canvas_ranges.iter().filter(|(_,a,b)|*a < hi && *b > lo).map(|(i,_,_)|*i).collect::<Vec<_>>() }));
        if policies
            .get(&(line.source, line.source_page, line.region))
            .copied()
            .unwrap_or(default_policy)
            == ReviewedEvidencePolicy::PromotedTierB
        {
            assert_eq!(
                canvas_ranges
                    .iter()
                    .filter(|(_, a, b)| *a < hi && *b > lo)
                    .count(),
                1,
                "reviewed line stays whole on one canvas"
            );
        }
    }
    ReviewedComposition {
        canvases,
        canvas_ranges,
        line_coverage,
    }
}

#[test]
fn reviewed_source_line_assertion_rejects_missing_and_duplicate_lines() {
    let (page, metrics) = page_and_metrics(
        100,
        100,
        &[(10, 20, 5, 95), (30, 40, 5, 95), (50, 60, 5, 95)],
    );
    let originals = std::collections::BTreeMap::from([((0, 1), page.0.clone())]);
    let traces = [RetainedSourcePageTrace {
        source: 0,
        page: 1,
        retained_index: Some(0),
        crop: [0, 100],
        bands: vec![[10, 20], [30, 40], [50, 60]],
        deleted_rows: vec![],
    }];
    let pages = [page];
    let context = ReviewedLineContext {
        traces: &traces,
        originals: &originals,
        pages: &pages,
        metrics: &metrics,
        scale: 1.2,
        geom: TrimGeom {
            dpi: 300,
            tb_w: 140,
            tb_h: 500,
            pad_x: 3,
            pad_y: 2,
        },
    };
    let line = ReviewedSourceLine {
        source: 0,
        source_page: 1,
        region: [5, 10, 95, 20],
        role: "retained_body".into(),
    };
    assert_reviewed_source_lines_survive(std::slice::from_ref(&line), &context, false);
    let mut absent = line.clone();
    absent.region = [5, 70, 95, 80];
    assert!(
        std::panic::catch_unwind(|| assert_reviewed_source_lines_survive(
            &[absent],
            &context,
            false
        ))
        .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| assert_reviewed_source_lines_survive(
            &[line.clone(), line],
            &context,
            false
        ))
        .is_err()
    );
}

/// Run production extraction once and bind retained images to physical source pages.
fn audit_traced_unit(id: &str) -> (Vec<PageBands>, LineMetrics, Vec<RetainedSourcePageTrace>) {
    let range = operational_range(id);
    let source_pages = std::cell::RefCell::new(Vec::new());
    let (pages, metrics) = unit_body_strips(&range, |s, p| {
        source_pages.borrow_mut().push(p);
        Ok(real_source_image(s, p))
    })
    .unwrap();
    let source_pages = source_pages.into_inner();
    assert_eq!(
        source_pages.len(),
        pages.len(),
        "every requested page rendered"
    );
    let expected: Vec<_> = (range.unit.start_page..=range.end_page)
        .filter(|p| !skip_known_duplicate_source_page(id, range.unit.source, *p))
        .collect();
    assert_eq!(source_pages, expected);
    let traces = (range.unit.start_page..=range.end_page)
        .map(|p| {
            let index = source_pages.iter().position(|&page| page == p);
            let mut raw = real_source_image(range.unit.source, p);
            mask_known_source_furniture(range.unit.source, p, &mut raw);
            let crop = production_source_column(range.unit.source, p, &raw).unwrap();
            RetainedSourcePageTrace {
                source: range.unit.source,
                page: p,
                retained_index: index,
                crop: [crop.left, crop.right],
                bands: index
                    .map(|i| pages[i].1.iter().map(|b| [b.top, b.bottom]).collect())
                    .unwrap_or_default(),
                deleted_rows: vec![],
            }
        })
        .collect();
    (pages, metrics, traces)
}

/// Save diagnostic stage measurements; normalized detector boxes remain locators.
fn audit_save_unit_trace(id: &str, selected: &std::collections::BTreeSet<u32>, group: &str) {
    let (pages, metrics, traces) = audit_traced_unit(id);
    let dir = std::path::Path::new("tmp/pdfs/inner_tomi_audit_repairs")
        .join(group)
        .join(id);
    std::fs::create_dir_all(&dir).unwrap();
    let geom = TrimGeom::default();
    let scale = production_body_scale();
    let packed = compose_body_canvases(
        &pages,
        scale,
        &geom,
        geom.first_body_budget(),
        geom.body_budget(),
        &metrics,
    );
    let packed_details: Vec<_> = packed
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let ink: u64 = ink_profile_rows(c, INK_THRESHOLD)
                .into_iter()
                .map(u64::from)
                .sum();
            c.save(dir.join(format!("canvas-before-filter-{}.png", i + 1)))
                .unwrap();
            serde_json::json!({"canvas":i+1,"height":c.height(),"ink":ink})
        })
        .collect();
    let mut canvases = packed;
    retain_body_canvases(&mut canvases, scale, &metrics);
    let (strips, canvas_ranges, canvases) =
        ranges_for_composed_canvases(&pages, &metrics, scale, &geom, canvases);
    for (i, canvas) in canvases.iter().enumerate() {
        canvas
            .save(dir.join(format!("canvas-{}.png", i + 1)))
            .unwrap();
    }
    let mut records = Vec::new();
    let fixture = audit_repair_ledger();
    let cases: Vec<_> = fixture["candidate_mask_cases"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|c| c["unit_id"] == id)
        .collect();
    let candidate_unit = if cases.is_empty() {
        None
    } else {
        let range = operational_range(id);
        let requested = std::cell::RefCell::new(Vec::new());
        let (candidate_pages, candidate_metrics) = unit_body_strips(&range, |source, page| {
            requested.borrow_mut().push(page);
            let mut raw = real_source_image(source, page);
            for case in cases.iter().filter(|c| {
                c["source"].as_u64() == Some(source as u64)
                    && c["source_page"].as_u64() == Some(page as u64)
            }) {
                for r in case["mask_rectangles"].as_array().unwrap() {
                    apply_source_furniture_mask(
                        &SourceFurnitureMask {
                            source,
                            page,
                            left: r[0].as_f64().unwrap(),
                            top: r[1].as_f64().unwrap(),
                            right: r[2].as_f64().unwrap(),
                            bottom: r[3].as_f64().unwrap(),
                        },
                        &mut raw,
                    );
                }
            }
            Ok(raw)
        })
        .unwrap();
        assert_eq!(
            candidate_pages.len(),
            pages.len(),
            "candidate masks preserve requested pages"
        );
        Some((requested.into_inner(), candidate_pages, candidate_metrics))
    };
    for trace in traces.iter().filter(|t| selected.contains(&t.page)) {
        let raw = real_source_image(trace.source, trace.page);
        let raw_crop = detect_latin_column(&raw).unwrap();
        let mut masked = raw.clone();
        mask_known_source_furniture(trace.source, trace.page, &mut masked);
        let crop = preserve_known_body_crop_edge(
            trace.source,
            trace.page,
            &masked,
            detect_latin_column(&masked).unwrap(),
        );
        let column = crop_column(&masked, &crop);
        let bands = line_bands(&column, LINE_MIN_INK);
        let zone = drop_furniture(&bands, column.height());
        let (_, pass_a) =
            render_and_band(trace.source, trace.page, &|_, _| Ok(raw.clone())).unwrap();
        let as_rows = |bands: &[Band]| bands.iter().map(|b| [b.top, b.bottom]).collect::<Vec<_>>();
        let retained_strips: Vec<_> = strips.iter().filter(|(s,_,_)| Some(s.page_img_index) == trace.retained_index)
            .map(|(s,a,b)| serde_json::json!({"rows":[s.band.top,s.band.bottom],"class":format!("{:?}",s.class),"scaled_rows":[a,b]})).collect();
        let band_details: Vec<_> = bands.iter().map(|b| serde_json::json!({"rows":[b.top,b.bottom],
            "ink_bounds":band_ink_bounds(&column,b), "span":band_ink_span(&column,b),
            "coverage":band_ink_coverage(&column,b), "class":format!("{:?}",classify_band(&column,b,&metrics))})).collect();
        let candidate_crops: Vec<_> = fixture["candidate_crop_cases"].as_array().into_iter().flatten()
            .filter(|c| c["source"].as_u64() == Some(trace.source as u64)
                && c["source_page"].as_u64() == Some(trace.page as u64))
            .map(|c| {
                let edges = crate::facsimile::columns::Column {
                    left: c["proposed_crop"][0].as_u64().unwrap() as u32,
                    right: c["proposed_crop"][1].as_u64().unwrap() as u32,
                };
                let mut candidate = masked.clone();
                for case in cases.iter().filter(|c| c["source_page"].as_u64() == Some(trace.page as u64)) {
                    for r in case["mask_rectangles"].as_array().unwrap() {
                        apply_source_furniture_mask(&SourceFurnitureMask {
                            source: trace.source, page: trace.page, left: r[0].as_f64().unwrap(),
                            top: r[1].as_f64().unwrap(), right: r[2].as_f64().unwrap(), bottom: r[3].as_f64().unwrap(),
                        }, &mut candidate);
                    }
                }
                let col = crop_column(&candidate, &edges);
                col.save(dir.join(format!("s{}-p{}-candidate-crop.png",trace.source,trace.page))).unwrap();
                let raw_bands = line_bands(&col, LINE_MIN_INK);
                let details: Vec<_> = raw_bands.iter().map(|b| serde_json::json!({"rows":[b.top,b.bottom],
                    "ink_bounds":band_ink_bounds(&col,b),"class":format!("{:?}",classify_band(&col,b,&metrics))})).collect();
                serde_json::json!({"crop_edges":[edges.left,edges.right],"raw_bands":as_rows(&raw_bands),
                    "zone_and_gap_filtered_bands":as_rows(&drop_furniture(&raw_bands,col.height())),"band_details":details})
            }).collect();
        let mut candidate_results = Vec::new();
        for case in fixture["candidate_mask_cases"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|case| {
                case["source"].as_u64() == Some(trace.source as u64)
                    && case["source_page"].as_u64() == Some(trace.page as u64)
            })
        {
            let mut candidate = raw.clone();
            for rectangle in case["mask_rectangles"].as_array().unwrap() {
                apply_source_furniture_mask(
                    &SourceFurnitureMask {
                        source: trace.source,
                        page: trace.page,
                        left: rectangle[0].as_f64().unwrap(),
                        top: rectangle[1].as_f64().unwrap(),
                        right: rectangle[2].as_f64().unwrap(),
                        bottom: rectangle[3].as_f64().unwrap(),
                    },
                    &mut candidate,
                );
            }
            let boundary: [u32; 4] = std::array::from_fn(|i| {
                case["preserved_latin_boundary"][i].as_u64().unwrap() as u32
            });
            for y in boundary[1]..boundary[3] {
                for x in boundary[0]..boundary[2] {
                    assert_eq!(
                        candidate.get_pixel(x, y),
                        raw.get_pixel(x, y),
                        "candidate preserves reviewed Latin boundary"
                    );
                }
            }
            mask_known_source_furniture(trace.source, trace.page, &mut candidate);
            let edges = detect_latin_column(&candidate).unwrap();
            let candidate_col = crop_column(&candidate, &edges);
            let (_, candidate_bands) =
                render_and_band(trace.source, trace.page, &|_, _| Ok(candidate.clone())).unwrap();
            let candidate_details: Vec<_> = candidate_bands
                .iter()
                .map(|b| {
                    serde_json::json!({
                "rows":[b.top,b.bottom], "ink_bounds":band_ink_bounds(&candidate_col,b)})
                })
                .collect();
            candidate_col
                .save(dir.join(format!(
                    "s{}-p{}-candidate-column.png",
                    trace.source, trace.page
                )))
                .unwrap();
            candidate_results.push(serde_json::json!({"finding_ids":case["finding_ids"],
                "crop_edges":[edges.left,edges.right],"raw_bands":as_rows(&line_bands(&candidate_col,LINE_MIN_INK)),
                "pass_a_bands":as_rows(&candidate_bands),"band_details":candidate_details,
                "full_unit_pass_b_bands":candidate_unit.as_ref().map(|(ids,cp,_)|as_rows(&cp[ids.iter().position(|p|*p==trace.page).unwrap()].1)),
                "full_unit_strip_intervals":candidate_unit.as_ref().map(|(ids,cp,cm)|strips_in_reading_order(cp,cm).iter()
                    .filter(|s|s.page_img_index==ids.iter().position(|p|*p==trace.page).unwrap()).map(|s|[s.band.top,s.band.bottom]).collect::<Vec<_>>())}));
        }
        raw.save(dir.join(format!("s{}-p{}-raw.png", trace.source, trace.page)))
            .unwrap();
        column
            .save(dir.join(format!("s{}-p{}-column.png", trace.source, trace.page)))
            .unwrap();
        records.push(serde_json::json!({"source":trace.source,"source_page":trace.page,
            "raster":[raw.width(),raw.height()],"raw_crop_edges":[raw_crop.left,raw_crop.right],
            "crop_edges":[crop.left,crop.right],"raw_bands":as_rows(&bands),"band_details":band_details,
            "zone_and_gap_filtered_bands":as_rows(&zone),"pass_a_bands":as_rows(&pass_a),
            "retained_trace":trace,"strips":retained_strips,"candidate_results":candidate_results,
            "candidate_crops":candidate_crops}));
    }
    assert_eq!(
        records.len(),
        selected.len(),
        "all requested physical pages traced"
    );
    let canvas_details: Vec<_> = canvases
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let ink: u64 = ink_profile_rows(c, INK_THRESHOLD)
                .into_iter()
                .map(u64::from)
                .sum();
            serde_json::json!({"canvas":i+1,"height":c.height(),"ink":ink})
        })
        .collect();
    let result = serde_json::json!({"unit_id":id,"scale":scale,"metrics":{"height_med":metrics.height_med,
        "height_mad":metrics.height_mad,"width_med":metrics.width_med},"strip_minimum_ink":minimum_ink(&metrics),
        "canvas_minimum_ink":minimum_ink(&metrics).map(|v| (v as f64 * (scale as f64).powi(2)).ceil() as u64),
        "source_pages":traces,"pages":records,"canvas_ranges":canvas_ranges,
        "canvases_before_filter":packed_details,"canvases":canvas_details});
    std::fs::write(
        dir.join("trace.json"),
        serde_json::to_vec_pretty(&result).unwrap(),
    )
    .unwrap();
    println!(
        "{group}/{id}: {} source pages, {} strips, {} canvases; saved {}",
        pages.len(),
        strips.len(),
        canvases.len(),
        dir.display()
    );
    assert!(!strips.is_empty() && !canvases.is_empty());
}

#[test]
#[ignore = "requires original PDFs and production-DPI raster evidence"]
fn audit_duplicate_blocks_source_trace() {
    let fixture = audit_repair_ledger();
    let mut units: std::collections::BTreeMap<&str, std::collections::BTreeSet<u32>> =
        std::collections::BTreeMap::new();
    for block in fixture["duplicate_blocks"]
        .as_array()
        .unwrap()
        .iter()
        .chain(fixture["existing_duplicate_evidence"].as_array().unwrap())
    {
        let pages = units.entry(block["unit_id"].as_str().unwrap()).or_default();
        for key in ["keep_pages", "skip_pages", "retain_sequence"] {
            if let Some(values) = block[key].as_array() {
                pages.extend(values.iter().map(|p| p.as_u64().unwrap() as u32));
            }
        }
    }
    for (id, pages) in units {
        audit_save_unit_trace(id, &pages, "duplicates");
    }
}

#[test]
fn audit_duplicate_inventory_covers_reviewed_blocks() {
    let fixture = audit_repair_ledger();
    let blocks = fixture["duplicate_blocks"].as_array().unwrap();
    assert_eq!(blocks.len(), 9);
    assert_eq!(
        blocks
            .iter()
            .map(|b| b["skip_pages"].as_array().unwrap().len())
            .sum::<usize>(),
        20
    );
    let mut identities = std::collections::BTreeSet::new();
    for block in blocks
        .iter()
        .chain(fixture["existing_duplicate_evidence"].as_array().unwrap())
    {
        let id = block["unit_id"].as_str().unwrap();
        let lines: Vec<ReviewedSourceLine> =
            serde_json::from_value(block["reviewed_lines"].clone()).unwrap();
        assert!(!lines.is_empty(), "{id}: reviewed Latin regions required");
        let mut seen = std::collections::BTreeSet::new();
        for line in lines {
            assert!(
                seen.insert((line.source_page, line.region)),
                "unique reviewed region"
            );
            assert_eq!(line.source, 1);
            let measurement = block["baseline_measurements"]
                .as_array()
                .unwrap()
                .iter()
                .find(|m| m["source_page"].as_u64() == Some(line.source_page as u64))
                .unwrap();
            let [left, top, right, bottom] = line.region;
            assert!(left < right && top < bottom);
            assert!(right <= measurement["raster"][0].as_u64().unwrap() as u32);
            assert!(bottom <= measurement["raster"][1].as_u64().unwrap() as u32);
            assert_eq!(
                measurement["strips"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|s| s[0].as_u64().unwrap() <= top as u64
                        && s[1].as_u64().unwrap() >= bottom as u64)
                    .count(),
                1,
                "{id}: reviewed baseline region extracted once"
            );
        }
        if let Some(finding) = block["finding_id"].as_str() {
            assert!(identities.insert(finding));
            assert!(block["source_pair_evidence"].as_array().unwrap().len() >= 2);
        }
    }
    assert_eq!(identities.len(), 9);
    let pending = fixture["duplicate_blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["finding_id"] == "D7")
        .unwrap();
    assert_eq!(
        pending["pending_line_regions"][0]["event_id"],
        "extra_cato_minor_360_et_label"
    );
}

#[test]
#[ignore = "requires original PDFs and production-DPI raster evidence"]
fn audit_watermark_and_head_source_trace() {
    let fixture = audit_repair_ledger();
    let mut units: std::collections::BTreeMap<&str, std::collections::BTreeSet<u32>> =
        std::collections::BTreeMap::new();
    for case in fixture["furniture_cases"].as_array().unwrap() {
        let p = case["source_page"].as_u64().unwrap() as u32;
        let range = operational_range(case["unit_id"].as_str().unwrap());
        units
            .entry(case["unit_id"].as_str().unwrap())
            .or_default()
            .extend(p.saturating_sub(1).max(range.unit.start_page)..=(p + 1).min(range.end_page));
    }
    for (id, pages) in units {
        audit_save_unit_trace(id, &pages, "furniture");
    }
}

#[test]
#[ignore = "requires original PDFs and production-DPI raster evidence"]
fn audit_omission_source_trace() {
    let fixture = audit_repair_ledger();
    validate_omission_inventory_data(&fixture, &portable_audit_evidence()).unwrap();
    let mut units: std::collections::BTreeMap<&str, std::collections::BTreeSet<u32>> =
        std::collections::BTreeMap::new();
    let mut events = 0;
    for event in fixture["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["initial_tier"] == "A")
    {
        events += 1;
        let id = event["unit_id"].as_str().unwrap();
        let range = operational_range(id);
        for region in event["source_regions"].as_array().unwrap() {
            let p = region["page"].as_u64().unwrap() as u32;
            units.entry(id).or_default().extend(
                p.saturating_sub(1).max(range.unit.start_page)..=(p + 1).min(range.end_page),
            );
        }
    }
    assert_eq!(events, 39, "all initial Tier-A events scheduled");
    for (id, pages) in units {
        audit_save_unit_trace(
            id,
            &pages,
            &format!("omissions-{}", fixture["diagnosis_stage"].as_str().unwrap()),
        );
    }
}

#[test]
#[ignore = "requires original PDFs and production-DPI raster evidence"]
fn audit_candidate_source_trace() {
    let fixture = audit_repair_ledger();
    let mut units: std::collections::BTreeMap<&str, std::collections::BTreeSet<u32>> =
        std::collections::BTreeMap::new();
    for key in ["candidate_mask_cases", "candidate_crop_cases"] {
        for case in fixture[key].as_array().unwrap() {
            units
                .entry(case["unit_id"].as_str().unwrap())
                .or_default()
                .insert(case["source_page"].as_u64().unwrap() as u32);
        }
    }
    assert!(!units.is_empty());
    for (id, pages) in units {
        audit_save_unit_trace(
            id,
            &pages,
            &format!(
                "candidates-{}",
                fixture["diagnosis_stage"].as_str().unwrap()
            ),
        );
    }
}

type TierBTarget = (&'static str, usize, u32, (u32, u32), (u32, u32), (u32, u32));
const TIER_B_TARGETS: &[TierBTarget] = &[
    (
        "aemilius_paulus",
        0,
        326,
        (790, 2982),
        (273, 304),
        (273, 304),
    ),
    ("agesilaus", 1, 110, (798, 2834), (2519, 2558), (2519, 2558)),
    (
        "alcibiades",
        0,
        242,
        (787, 2981),
        (2598, 2623),
        (2598, 2623),
    ),
    ("antonius", 1, 562, (815, 2835), (249, 1356), (249, 1356)),
    (
        "artoxerxes",
        1,
        633,
        (786, 2826),
        (2522, 2554),
        (2522, 2554),
    ),
    ("brutus", 1, 619, (805, 2824), (120, 225), (193, 225)),
    ("caius_marius", 0, 519, (818, 2973), (246, 292), (246, 292)),
    ("cato_minor", 1, 329, (839, 2831), (40, 210), (180, 210)),
    (
        "cato_minor",
        1,
        352,
        (804, 2842),
        (2526, 2564),
        (2526, 2564),
    ),
    ("crassus", 1, 44, (789, 2822), (191, 224), (191, 224)),
    ("demosthenes", 1, 445, (827, 2842), (44, 226), (197, 226)),
    ("dio", 1, 566, (805, 2838), (2540, 2577), (2540, 2577)),
    ("dio", 1, 580, (812, 2835), (2550, 2597), (2550, 2597)),
    ("lucullus", 0, 631, (795, 2978), (252, 287), (252, 287)),
    ("lysander", 0, 535, (796, 2969), (245, 278), (245, 278)),
    ("lysander", 0, 535, (796, 2969), (306, 339), (306, 339)),
    ("marcellus", 0, 377, (808, 2957), (249, 282), (249, 282)),
    ("nicias", 1, 25, (812, 2832), (204, 247), (204, 247)),
    ("pericles", 0, 214, (802, 2988), (2599, 2628), (2599, 2628)),
    ("pericles", 0, 215, (796, 2974), (249, 272), (249, 272)),
    ("phocion", 1, 291, (873, 2836), (173, 239), (206, 239)),
    ("timoleon", 0, 318, (791, 2988), (263, 295), (263, 295)),
];

fn tier_b_sha(bytes: &[u8]) -> String {
    use sha2::Digest;
    format!("{:x}", sha2::Sha256::digest(bytes))
}

fn tier_b_file_reference(path: &std::path::Path) -> serde_json::Value {
    serde_json::json!({"path":path.to_string_lossy().replace('\\',"/"),"sha256":tier_b_sha(&std::fs::read(path).unwrap())})
}

fn tier_b_configuration() -> serde_json::Value {
    let sources: Vec<_> = (1..=2)
        .map(|s| tier_b_file_reference(std::path::Path::new(&format!("Ploutarchou_Bioi_v{s}.pdf"))))
        .collect();
    let production_files: Vec<_> = [
        "src/facsimile/reflow.rs",
        "src/facsimile/columns.rs",
        "src/facsimile/lines.rs",
        "src/facsimile/metrics.rs",
        "src/facsimile/mod.rs",
        "src/facsimile/units.rs",
    ]
    .iter()
    .map(|p| tier_b_file_reference(std::path::Path::new(p)))
    .collect();
    serde_json::json!({"sources":sources,"unit_table":tier_b_file_reference(std::path::Path::new("facsimile_units.json")),"production_files":production_files})
}

fn tier_b_canvas_records(canvases: &[GrayImage]) -> Vec<serde_json::Value> {
    canvases.iter().enumerate().map(|(i,c)|serde_json::json!({"canvas":i+1,"width":c.width(),"height":c.height(),"sha256":tier_b_sha(c.as_raw())})).collect()
}

fn tier_b_unit_snapshot(id: &str, scale: f32) -> serde_json::Value {
    let range = operational_range(id);
    let (pages, metrics, traces) = audit_traced_unit(id);
    let geom = TrimGeom::default();
    let canvases = compose_pages(
        &pages,
        scale,
        &geom,
        geom.first_body_budget(),
        geom.body_budget(),
        &metrics,
    );
    let canvas_hashes = tier_b_canvas_records(&canvases);
    let columns: Vec<_> = traces.iter().filter_map(|t|t.retained_index.map(|i|serde_json::json!({"source":t.source,"page":t.page,"width":pages[i].0.width(),"height":pages[i].0.height(),"sha256":tier_b_sha(pages[i].0.as_raw())}))).collect();
    serde_json::json!({"schema_version":1,"unit_id":id,"source":range.unit.source,"scale":scale,
        "requested_source_pages":traces.iter().filter(|t|t.retained_index.is_some()).map(|t|t.page).collect::<Vec<_>>(),
        "retained_source_pages":traces.iter().filter(|t|t.retained_index.is_some()).map(|t|t.page).collect::<Vec<_>>(),
        "source_pages":traces,"retained_columns":columns,"metrics":{"height_med":metrics.height_med,"height_mad":metrics.height_mad,"width_med":metrics.width_med},
        "canvas_count":canvas_hashes.len(),"canvas_sequence_sha256":tier_b_sha(&serde_json::to_vec(&canvas_hashes).unwrap()),"canvas_hashes":canvas_hashes})
}

fn validate_tier_b_baseline_manifest(
    manifest: &serde_json::Value,
    ids: &[String],
    configuration: &serde_json::Value,
    scale: f32,
) -> Result<()> {
    anyhow::ensure!(manifest["schema_version"] == 1, "baseline schema");
    anyhow::ensure!(
        manifest["configuration"] == *configuration,
        "baseline configuration mismatch"
    );
    anyhow::ensure!(
        manifest["scale"]
            .as_f64()
            .is_some_and(|v| v.is_finite() && v > 0.0 && (v as f32).to_bits() == scale.to_bits()),
        "baseline scale mismatch"
    );
    let units = manifest["units"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("baseline units missing"))?;
    anyhow::ensure!(units.len() == ids.len(), "baseline missing or extra units");
    for (unit, id) in units.iter().zip(ids) {
        anyhow::ensure!(
            unit["unit_id"] == *id,
            "baseline unit ordering or duplication"
        );
        anyhow::ensure!(
            unit["source"].as_u64().is_some_and(|s| s < 2),
            "baseline source identity"
        );
        anyhow::ensure!(
            unit["trace"]["path"]
                .as_str()
                .is_some_and(|p| !p.is_empty()),
            "baseline missing trace reference"
        );
        anyhow::ensure!(
            unit["trace"]["sha256"]
                .as_str()
                .is_some_and(|s| s.len() == 64),
            "baseline trace digest"
        );
        anyhow::ensure!(
            unit["canvas_sequence_sha256"]
                .as_str()
                .is_some_and(|s| s.len() == 64),
            "baseline canvas digest"
        );
    }
    Ok(())
}

#[test]
#[ignore = "requires every original source raster and full-work composition"]
fn capture_tier_b_full_work_baseline() {
    assert_eq!(
        SOURCE_BODY_BANDS.len(),
        47,
        "reject an already-repaired configuration"
    );
    let units =
        crate::facsimile::units::load_units(std::path::Path::new("facsimile_units.json")).unwrap();
    let ids: Vec<_> = units.iter().map(|u| u.id.clone()).collect();
    let configuration = tier_b_configuration();
    let scale = production_body_scale();
    let dir = std::path::Path::new("tmp/pdfs/inner_tomi_audit_repairs/tier-b-baseline");
    let archive =
        std::path::Path::new("archive/tier_b_omission_repairs_2026_10_09/baseline/full-work");
    assert!(
        !archive.join("manifest.json").exists(),
        "immutable baseline already archived"
    );
    std::fs::create_dir_all(dir).unwrap();
    std::fs::create_dir_all(archive).unwrap();
    let mut entries = Vec::new();
    for id in &ids {
        let trace = tier_b_unit_snapshot(id, scale);
        let bytes = serde_json::to_vec_pretty(&trace).unwrap();
        let filename = format!("{id}.json");
        std::fs::write(dir.join(&filename), &bytes).unwrap();
        let path = archive.join(&filename);
        assert!(!path.exists(), "baseline trace must not be overwritten");
        std::fs::write(&path, &bytes).unwrap();
        entries.push(serde_json::json!({"unit_id":id,"source":trace["source"],"trace":tier_b_file_reference(&path),"metrics":trace["metrics"],"canvas_count":trace["canvas_count"],"canvas_sequence_sha256":trace["canvas_sequence_sha256"]}));
        println!("baseline {id}: {} canvases", trace["canvas_count"]);
    }
    let manifest = serde_json::json!({"schema_version":1,"scale":scale,"configuration":configuration,"units":entries,"protection_record_count":47});
    validate_tier_b_baseline_manifest(&manifest, &ids, &configuration, scale).unwrap();
    let bytes = serde_json::to_vec_pretty(&manifest).unwrap();
    std::fs::write(dir.join("manifest.json"), &bytes).unwrap();
    std::fs::write(archive.join("manifest.json"), &bytes).unwrap();
    for unit in manifest["units"].as_array().unwrap() {
        let bytes = std::fs::read(unit["trace"]["path"].as_str().unwrap()).unwrap();
        assert_eq!(tier_b_sha(&bytes), unit["trace"]["sha256"]);
        let trace: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            tier_b_sha(&serde_json::to_vec(&trace["canvas_hashes"]).unwrap()),
            unit["canvas_sequence_sha256"]
        );
    }
    println!(
        "archived {} ordered operational units at scale {scale}",
        ids.len()
    );
}

#[test]
fn tier_b_baseline_manifest_rejects_missing_units_and_configuration_mismatch() {
    let ids = vec!["one".to_string(), "two".to_string()];
    let configuration =
        serde_json::json!({"sources":["a","b"],"unit_table":"units","production_files":["code"]});
    let entry = |id| serde_json::json!({"unit_id":id,"source":0,"trace":{"path":"trace.json","sha256":"a".repeat(64)},"canvas_sequence_sha256":"b".repeat(64)});
    let good = serde_json::json!({"schema_version":1,"scale":1.0,"configuration":configuration,"units":[entry("one"),entry("two")]});
    validate_tier_b_baseline_manifest(&good, &ids, &configuration, 1.0).unwrap();
    for mode in [
        "missing",
        "duplicate",
        "reordered",
        "configuration",
        "trace",
        "scale",
    ] {
        let mut bad = good.clone();
        match mode {
            "missing" => {
                bad["units"].as_array_mut().unwrap().pop();
            }
            "duplicate" => bad["units"][1] = entry("one"),
            "reordered" => bad["units"].as_array_mut().unwrap().reverse(),
            "configuration" => bad["configuration"]["unit_table"] = serde_json::json!("different"),
            "trace" => bad["units"][0]["trace"] = serde_json::Value::Null,
            _ => bad["scale"] = serde_json::json!(0.5),
        }
        assert!(
            validate_tier_b_baseline_manifest(&bad, &ids, &configuration, 1.0).is_err(),
            "reject {mode}"
        );
    }
}

fn measured_glyph_region(column: &GrayImage, crop_left: u32, top: u32, bottom: u32) -> [u32; 4] {
    let mut bounds = [column.width(), bottom, 0, top];
    for y in top..bottom {
        for x in 0..column.width() {
            if column.get_pixel(x, y)[0] <= 200 {
                bounds[0] = bounds[0].min(x);
                bounds[1] = bounds[1].min(y);
                bounds[2] = bounds[2].max(x + 1);
                bounds[3] = bounds[3].max(y + 1);
            }
        }
    }
    assert!(
        bounds[0] < bounds[2] && bounds[1] < bounds[3],
        "measured ink is required"
    );
    bounds[0] += crop_left;
    bounds[2] += crop_left;
    bounds
}

fn measure_tier_b_source_bands() {
    let ledger = audit_repair_ledger();
    let dir = std::path::Path::new("tmp/pdfs/inner_tomi_audit_repairs/band_extract");
    std::fs::create_dir_all(dir).unwrap();
    let mut records = Vec::new();
    let pages: std::collections::BTreeSet<_> = TIER_B_TARGETS
        .iter()
        .map(|(id, s, p, _, _, _)| (*id, *s, *p))
        .collect();
    let mut units = std::collections::BTreeMap::new();
    for &(id, source, page) in &pages {
        let (_, _, traces) = units.entry(id).or_insert_with(|| audit_traced_unit(id));
        let trace = retained_page(traces, source, page).unwrap();
        let raw = real_source_image(source, page);
        let raw_column = detect_latin_column(&raw).unwrap();
        let mut masked = raw.clone();
        mask_known_source_furniture(source, page, &mut masked);
        let masked_column = detect_latin_column(&masked).unwrap();
        let production = production_source_column(source, page, &masked).unwrap();
        let column = crop_column(&masked, &production);
        let (raw_bands, furniture) = band_source_column(source, page, &column);
        let (_, pass_a) = render_and_band(source, page, &|_, _| Ok(raw.clone())).unwrap();
        let as_rows = |bands: &[Band]| bands.iter().map(|b| [b.top, b.bottom]).collect::<Vec<_>>();
        let targets: Vec<_> = TIER_B_TARGETS
            .iter()
            .filter(|(i, s, p, _, _, _)| *i == id && *s == source && *p == page)
            .collect();
        let event = ledger["events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| {
                e["initial_tier"] == "B"
                    && e["unit_id"] == id
                    && e["source_regions"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|r| r["page"] == page)
            })
            .unwrap();
        let mut regions = Vec::new();
        let mut neighbors = Vec::new();
        for &&(_, _, _, dimensions, (top, bottom), (ptop, pbottom)) in &targets {
            assert_eq!(
                crop_column(&masked, &masked_column).dimensions(),
                dimensions
            );
            assert!(
                raw_bands.contains(&Band { top, bottom }),
                "exact observed detector evidence"
            );
            if id != "antonius" {
                let mut region = measured_glyph_region(&column, production.left, ptop, pbottom);
                if id == "dio" && page == 580 {
                    region[0] = 889;
                    assert!((ptop..pbottom).any(|y| raw.get_pixel(889, y)[0] <= INK_THRESHOLD));
                }
                regions.push(ReviewedSourceLine {
                    source,
                    source_page: page,
                    region,
                    role: "promoted_missing_line".into(),
                });
            } else {
                let measured: &[[u32; 4]] = &[
                    [46, 0, 815, 53],
                    [43, 46, 815, 103],
                    [42, 98, 815, 155],
                    [41, 146, 815, 204],
                    [39, 202, 815, 255],
                    [2, 249, 813, 302],
                    [36, 296, 811, 358],
                    [34, 347, 810, 405],
                    [32, 401, 808, 457],
                    [31, 452, 807, 512],
                    [28, 505, 805, 561],
                    [28, 557, 802, 616],
                    [28, 610, 799, 661],
                    [22, 657, 797, 714],
                    [0, 707, 796, 767],
                    [18, 760, 791, 821],
                    [16, 814, 795, 872],
                    [15, 866, 793, 924],
                    [14, 921, 791, 975],
                    [10, 971, 789, 1029],
                    [8, 1023, 787, 1079],
                    [6, 1075, 244, 1107],
                ];
                regions.extend(measured.iter().map(|r| ReviewedSourceLine {
                    source,
                    source_page: page,
                    region: [
                        production.left + r[0],
                        ptop + r[1],
                        production.left + r[2],
                        ptop + r[3],
                    ],
                    role: "promoted_missing_line".into(),
                }));
            }
            let candidates: Vec<_> = traces
                .iter()
                .filter(|t| t.retained_index.is_some())
                .flat_map(|t| t.bands.iter().map(move |b| (t, b)))
                .collect();
            for (neighbor, b) in [
                candidates
                    .iter()
                    .rev()
                    .find(|(t, b)| t.page < page || (t.page == page && b[1] < top)),
                candidates
                    .iter()
                    .find(|(t, b)| t.page > page || (t.page == page && b[0] > bottom)),
            ]
            .into_iter()
            .flatten()
            {
                let mut neighbor_raw = real_source_image(neighbor.source, neighbor.page);
                mask_known_source_furniture(neighbor.source, neighbor.page, &mut neighbor_raw);
                let neighbor_column = crop_column(
                    &neighbor_raw,
                    &Column {
                        left: neighbor.crop[0],
                        right: neighbor.crop[1],
                    },
                );
                neighbors.push(ReviewedSourceLine {
                    source: neighbor.source,
                    source_page: neighbor.page,
                    region: measured_glyph_region(&neighbor_column, neighbor.crop[0], b[0], b[1]),
                    role: "neighbor".into(),
                });
            }
            image::imageops::crop_imm(&column, 0, ptop, column.width(), pbottom - ptop)
                .to_image()
                .save(dir.join(format!("{id}_p{page}_b{ptop:05}.png")))
                .unwrap();
        }
        neighbors.sort_by_key(|l| (l.source_page, l.region));
        neighbors.dedup_by_key(|l| (l.source_page, l.region));
        let raw_path = dir.join(format!("{id}_p{page}_source.png"));
        raw.save(&raw_path).unwrap();
        let mut overlay = raw.clone();
        for col in [&raw_column, &masked_column, &production] {
            for y in 0..overlay.height() {
                overlay.put_pixel(col.left, y, Luma([120]));
                overlay.put_pixel(col.right - 1, y, Luma([120]));
            }
        }
        for l in regions
            .iter()
            .chain(neighbors.iter().filter(|l| l.source_page == page))
        {
            let [left, top, right, bottom] = l.region;
            for x in left..right {
                overlay.put_pixel(x, top, Luma([120]));
                overlay.put_pixel(x, bottom - 1, Luma([120]));
            }
        }
        let overlay_path = dir.join(format!("{id}_p{page}_overlay.png"));
        overlay.save(&overlay_path).unwrap();
        let profile = ink_profile_rows(&column, INK_THRESHOLD);
        records.push(serde_json::json!({"event_id":event["event_id"],"unit_id":id,"source":source,"page":page,"dpi":BODY_DPI,
            "raster_dimensions":[raw.width(),raw.height()],"raster_sha256":tier_b_sha(raw.as_raw()),"source_image":tier_b_file_reference(&raw_path),"overlay":tier_b_file_reference(&overlay_path),"overlay_origin":[0,0],
            "raw_detected_column":[raw_column.left,raw_column.right],"masked_detected_column":[masked_column.left,masked_column.right],"production_column":[production.left,production.right],"column_dimensions":[column.width(),column.height()],
            "raw_bands":as_rows(&raw_bands),"furniture_filtered_bands":as_rows(&furniture),"pass_a_bands":as_rows(&pass_a),"pass_b_bands":trace.bands,
            "reviewed_lines":regions,"reviewed_neighbor_lines":neighbors,"row_ink_profile":profile,
            "antonius_measurement_status":if id=="antonius" {"22 measured printed lines; historical count 23 retained by approved inventory correction"} else {"measured"}}));
        println!(
            "measured {id} source {source} page {page}: production {}..{}",
            production.left, production.right
        );
    }
    assert_eq!(records.len(), 21);
    assert_eq!(
        records
            .iter()
            .map(|r| r["event_id"].as_str().unwrap())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        20
    );
    std::fs::write(
        "tmp/pdfs/inner_tomi_audit_repairs/tier_b_measurements.json",
        serde_json::to_vec_pretty(&serde_json::json!({"schema_version":1,"pages":records}))
            .unwrap(),
    )
    .unwrap();
}

fn select_tier_b_restoration_events(
    ledger: &serde_json::Value,
    stage: &str,
    candidate: bool,
) -> Result<Vec<String>> {
    let expected: &[(&str, u32)] = match stage {
        "head" => &[
            ("aemilius_paulus", 326),
            ("caius_marius", 519),
            ("crassus", 44),
            ("lucullus", 631),
            ("lysander", 535),
            ("marcellus", 377),
            ("nicias", 25),
            ("timoleon", 318),
        ],
        "foot" => &[
            ("agesilaus", 110),
            ("artoxerxes", 633),
            ("cato_minor", 352),
            ("dio", 566),
            ("dio", 580),
        ],
        "verse" => &[("alcibiades", 242), ("pericles", 214)],
        "merged" => &[
            ("brutus", 619),
            ("cato_minor", 329),
            ("demosthenes", 445),
            ("phocion", 291),
        ],
        "antonius" => &[("antonius", 562)],
        "non-antonius" | "delivery" => &[],
        _ => anyhow::bail!("unknown restoration selection"),
    };
    let complete = stage == "delivery" && ledger["tier_b_delivery"]["state"] == "complete";
    if stage == "delivery" {
        anyhow::ensure!(
            complete || ledger["tier_b_delivery"]["state"] == "partial",
            "delivery is in progress"
        );
    }
    let selected: Vec<_> = ledger["events"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("events missing"))?
        .iter()
        .filter(|e| {
            e["initial_tier"] == "B"
                && if expected.is_empty() {
                    complete || e["unit_id"] != "antonius"
                } else {
                    expected.iter().any(|(id, page)| {
                        e["unit_id"] == *id
                            && e["source_regions"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .any(|r| r["page"] == *page)
                    })
                }
        })
        .collect();
    let counts = match stage {
        "head" => (8, 9),
        "foot" => (5, 5),
        "verse" => (2, 3),
        "merged" => (4, 4),
        "antonius" => (1, 22),
        _ if complete => (20, 43),
        _ => (19, 21),
    };
    anyhow::ensure!(
        selected.len() == counts.0,
        "missing event or wrong event count"
    );
    let mut ids = std::collections::BTreeSet::new();
    let mut count = 0;
    for e in selected {
        let id = e["event_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("missing event identity"))?;
        anyhow::ensure!(ids.insert(id.to_string()), "duplicate event");
        let status = e["status"].as_str().unwrap_or("");
        anyhow::ensure!(
            if stage == "delivery" {
                status == "restored"
            } else {
                status == "installed"
                    || status == "restored"
                    || (candidate && status == "pending_repair")
            },
            "unverified selected status"
        );
        let lines = e["diagnosis"]["reviewed_lines"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("missing reviewed rectangles"))?;
        let n = e["measured_line_count"]
            .as_u64()
            .or_else(|| e["missing_line_count"].as_u64())
            .unwrap() as usize;
        anyhow::ensure!(lines.len() == n && n > 0, "wrong line count");
        let mut regions = std::collections::BTreeSet::new();
        for l in lines {
            let l: ReviewedSourceLine = serde_json::from_value(l.clone())?;
            anyhow::ensure!(
                l.region[0] < l.region[2]
                    && l.region[1] < l.region[3]
                    && regions.insert((l.source, l.source_page, l.region)),
                "invalid or duplicate region"
            );
        }
        count += n;
    }
    anyhow::ensure!(count == counts.1, "wrong stage line count");
    Ok(ids.into_iter().collect())
}

fn run_tier_b_restoration_selection(stage: &str) {
    let ledger = audit_repair_ledger();
    let ids = select_tier_b_restoration_events(&ledger, stage, stage != "delivery").unwrap();
    let mut groups = std::collections::BTreeMap::<String, Vec<String>>::new();
    for id in &ids {
        let event = ledger["events"]
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e["event_id"] == *id)
            .unwrap();
        groups
            .entry(event["unit_id"].as_str().unwrap().to_string())
            .or_default()
            .push(id.clone());
    }
    for (unit, events) in groups {
        assert_unit_lines_restored_with(
            &unit,
            &events,
            Some(&format!("tier-b-{stage}")),
            stage != "delivery",
        );
    }
    let line_count: u64 = ledger["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| ids.iter().any(|id| e["event_id"] == *id))
        .map(|e| {
            e["measured_line_count"]
                .as_u64()
                .or_else(|| e["missing_line_count"].as_u64())
                .unwrap()
        })
        .sum();
    println!(
        "Tier-B {stage}: {} selected events / {line_count} measured lines verified",
        ids.len()
    );
}

macro_rules! tier_b_restoration_tests {
    ($($name:ident => $stage:literal),* $(,)?) => {$(
        #[test]
        #[ignore = "requires original source PDFs and reviewed candidate production-scale composition"]
        fn $name() {run_tier_b_restoration_selection($stage);}
    )*};
}
tier_b_restoration_tests! {
    tier_b_head_source_lines_restored => "head",
    tier_b_foot_source_lines_restored => "foot",
    tier_b_verse_source_lines_restored => "verse",
    tier_b_merged_source_lines_restored => "merged",
    tier_b_non_antonius_source_lines_restored => "non-antonius",
    antonius_reviewed_lines_survive_composition_whole => "antonius",
    tier_b_source_lines_restored => "delivery",
}

const ANTONIUS_AUDIT_BAND: &[SourceBodyBand] = &[SourceBodyBand {
    source: 1,
    page: 562,
    unit_id: "antonius",
    dimensions: (815, 2835),
    observed: Band {
        top: 249,
        bottom: 1356,
    },
    protected: Band {
        top: 249,
        bottom: 1356,
    },
}];

const ANTONIUS_ANNOTATION_AUDIT_BAND: &[SourceBodyBand] = &[SourceBodyBand {
    source: 1,
    page: 562,
    unit_id: "antonius",
    dimensions: (826, 2835),
    observed: Band {
        top: 249,
        bottom: 1356,
    },
    protected: Band {
        top: 249,
        bottom: 1356,
    },
}];

struct AntoniusAuditConfigurationGuard {
    extra: &'static [SourceBodyBand],
    disabled: bool,
    crop: Option<Column>,
}

impl Drop for AntoniusAuditConfigurationGuard {
    fn drop(&mut self) {
        AUDIT_EXTRA_BODY_BANDS.with(|slot| slot.set(self.extra));
        AUDIT_DISABLE_ANTONIUS_WHOLE_STRIP.with(|slot| slot.set(self.disabled));
        AUDIT_ANTONIUS_CROP_OVERRIDE.with(|slot| slot.set(self.crop));
    }
}

fn with_antonius_audit_configuration<R>(
    protected: bool,
    whole: bool,
    run: impl FnOnce() -> R,
) -> R {
    let guard = AntoniusAuditConfigurationGuard {
        extra: AUDIT_EXTRA_BODY_BANDS.with(|slot| {
            slot.replace(if whole {
                ANTONIUS_ANNOTATION_AUDIT_BAND
            } else if protected {
                ANTONIUS_AUDIT_BAND
            } else {
                &[]
            })
        }),
        disabled: AUDIT_DISABLE_ANTONIUS_WHOLE_STRIP.with(|slot| slot.replace(!whole)),
        crop: AUDIT_ANTONIUS_CROP_OVERRIDE.with(|slot| {
            slot.replace(if whole {
                Some(Column {
                    left: 895,
                    right: 1721,
                })
            } else {
                None
            })
        }),
    };
    let result = run();
    drop(guard);
    result
}

/// Trace three explicit configurations through production extraction and packing.
/// Plain protection demonstrates the split boundary; whole-block recognition must
/// preserve every measured missing line on one canvas and both historical anchors.
#[test]
#[ignore = "requires source PDFs and full-work production-scale composition"]
fn antonius_block_composition_trace() {
    assert!(
        !SOURCE_BODY_BANDS
            .iter()
            .any(|r| r.source == 1 && r.page == 562),
        "diagnosis requires the pre-installation Antonius table"
    );
    let ledger = audit_repair_ledger();
    let event = ledger["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["event_id"] == "miss_antonius_562_20869")
        .unwrap();
    let reviewed: Vec<ReviewedSourceLine> =
        serde_json::from_value(event["diagnosis"]["reviewed_lines"].clone()).unwrap();
    assert_eq!(reviewed.len(), 22, "approved measured inventory");
    let mut lines = reviewed.clone();
    lines.extend(
        serde_json::from_value::<Vec<ReviewedSourceLine>>(
            event["diagnosis"]["reviewed_neighbor_lines"].clone(),
        )
        .unwrap(),
    );
    lines.sort_by_key(|l| (l.source_page, l.region[1]));
    lines.dedup_by_key(|l| (l.source, l.source_page, l.region));
    let policies = reviewed
        .iter()
        .map(|l| {
            (
                (l.source, l.source_page, l.region),
                ReviewedEvidencePolicy::PromotedTierB,
            )
        })
        .collect();
    let originals = lines
        .iter()
        .map(|l| (l.source, l.source_page))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|(s, p)| ((s, p), real_source_image(s, p)))
        .collect();
    let scale = production_body_scale();
    let geom = TrimGeom::default();
    let root = std::path::Path::new("tmp/pdfs/inner_tomi_audit_repairs/antonius-composition");
    let annotations = [
        ReviewedSourceLine {
            source: 1,
            source_page: 562,
            region: [908, 507, 931, 539],
            role: "approved_annotation".into(),
        },
        ReviewedSourceLine {
            source: 1,
            source_page: 562,
            region: [895, 947, 913, 967],
            role: "approved_annotation".into(),
        },
    ];
    let mut summaries = serde_json::Map::new();
    let mut selected_column = None;
    for (name, protected, whole) in [
        ("baseline", false, false),
        ("plain", true, false),
        ("candidate", true, true),
    ] {
        with_antonius_audit_configuration(protected, whole, || {
            audit_save_unit_trace(
                "antonius",
                &std::collections::BTreeSet::from([561, 562]),
                &format!("antonius-composition/{name}"),
            );
            let (pages, metrics, traces) = audit_traced_unit("antonius");
            let trace = retained_page(&traces, 1, 562).unwrap();
            let index = trace.retained_index.unwrap();
            let column = &pages[index].0;
            let (raw, furniture) = band_source_column(1, 562, column);
            let (_, pass_a) = render_and_band(1, 562, &|s, p| Ok(real_source_image(s, p))).unwrap();
            let rows = |bands: &[Band]| bands.iter().map(|b| [b.top, b.bottom]).collect::<Vec<_>>();
            assert!(raw.contains(&ANTONIUS_AUDIT_BAND[0].observed));
            assert!(
                !furniture.contains(&ANTONIUS_AUDIT_BAND[0].observed),
                "first loss is the production furniture filter"
            );
            assert_eq!(
                pass_a.contains(&ANTONIUS_AUDIT_BAND[0].protected),
                protected
            );
            assert_eq!(
                pages[index].1.contains(&ANTONIUS_AUDIT_BAND[0].protected),
                protected
            );
            let mut stage_lines = lines.clone();
            if whole {
                stage_lines.extend(annotations.iter().cloned());
                stage_lines.sort_by_key(|l| (l.source_page, l.region[1]));
            }
            let context = ReviewedLineContext {
                traces: &traces,
                originals: &originals,
                pages: &pages,
                metrics: &metrics,
                scale,
                geom,
            };
            let packed = compose_body_canvases(
                &pages,
                scale,
                &geom,
                geom.first_body_budget(),
                geom.body_budget(),
                &metrics,
            );
            let packed_hashes = tier_b_canvas_records(&packed);
            let mut returned = packed;
            retain_body_canvases(&mut returned, scale, &metrics);
            let (strips, ranges, returned) =
                ranges_for_composed_canvases(&pages, &metrics, scale, &geom, returned);
            let mut coverage = Vec::new();
            let mut first_failed = serde_json::Value::Null;
            let mut strip_record = serde_json::Value::Null;
            if protected {
                let checked = if whole {
                    assert_reviewed_source_lines_survive_with_policies(
                        &stage_lines,
                        &context,
                        &policies,
                        ReviewedEvidencePolicy::HistoricalTierA,
                    )
                } else {
                    assert_reviewed_source_lines_survive(&stage_lines, &context, false)
                };
                assert_eq!(
                    tier_b_canvas_records(&checked.canvases),
                    tier_b_canvas_records(&returned)
                );
                coverage = checked.line_coverage;
                for line in &reviewed {
                    let row = coverage
                        .iter()
                        .find(|row| {
                            row["source"] == line.source
                                && row["source_page"] == line.source_page
                                && row["region"] == serde_json::json!(line.region)
                        })
                        .unwrap();
                    if row["canvas_owners"].as_array().unwrap().len() != 1 && first_failed.is_null()
                    {
                        first_failed = row.clone();
                    }
                }
                if whole {
                    assert!(first_failed.is_null());
                    for annotation in &annotations {
                        let row = coverage
                            .iter()
                            .find(|r| {
                                r["region"] == serde_json::json!(annotation.region)
                                    && r["source_page"] == 562
                            })
                            .unwrap();
                        assert_eq!(
                            row["canvas_owners"].as_array().unwrap().len(),
                            1,
                            "approved annotation stays whole"
                        );
                    }
                    for neighbor in lines
                        .iter()
                        .filter(|l| l.source == 1 && l.source_page == 562 && l.role == "neighbor")
                    {
                        let row = coverage
                            .iter()
                            .find(|row| {
                                row["source"] == neighbor.source
                                    && row["source_page"] == neighbor.source_page
                                    && row["region"] == serde_json::json!(neighbor.region)
                            })
                            .unwrap();
                        assert_eq!(
                            row["canvas_owners"].as_array().unwrap().len(),
                            1,
                            "following chapter cluster stays whole"
                        );
                    }
                } else {
                    assert!(
                        !first_failed.is_null(),
                        "plain protection must reproduce a bisected reviewed line"
                    );
                }
                let (strip, _, _) = strips
                    .iter()
                    .find(|(s, _, _)| {
                        s.page_img_index == index && s.band.top <= 249 && s.band.bottom >= 1356
                    })
                    .unwrap();
                assert_eq!(
                    strip.band,
                    Band {
                        top: 249,
                        bottom: 1364
                    }
                );
                let scaled = scale_strip(column, &strip.band, scale).unwrap();
                assert!(scaled.height() <= geom.body_budget());
                let fingerprint =
                    image::imageops::crop_imm(column, 0, 249, column.width(), 1115).to_image();
                assert_eq!(
                    tier_b_sha(fingerprint.as_raw()),
                    if whole {
                        "5bd51a4290f3d012bcedf037f3cb8b0d67a4479bd381b3449a8165171b832632"
                    } else {
                        "ab6e88cedbcb8117631ab2ee3d942eb10392f2da3616000cbd18f6548f5dae95"
                    }
                );
                assert_eq!(is_known_whole_source_strip(column, &strip.band), whole);
                let ink: u64 = ink_profile_rows(&fingerprint, INK_THRESHOLD)
                    .into_iter()
                    .map(u64::from)
                    .sum();
                strip_record = serde_json::json!({"interval":[249,1364],"class":format!("{:?}",strip.class),"ink":ink,"scaled_dimensions":[scaled.width(),scaled.height()],"recognition_enabled":whole,"fingerprint":tier_b_sha(fingerprint.as_raw())});
                for canvas in coverage
                    .iter()
                    .flat_map(|r| r["canvas_owners"].as_array().unwrap())
                    .filter_map(|v| v.as_u64())
                    .collect::<std::collections::BTreeSet<_>>()
                {
                    returned[canvas as usize - 1]
                        .save(
                            root.join(name)
                                .join(format!("antonius-canvas-{canvas:03}.png")),
                        )
                        .unwrap();
                }
            }
            let mut overlay = originals[&(1, 562)].clone();
            for line in &reviewed {
                let [left, top, right, bottom] = line.region;
                for x in left..right {
                    overlay.put_pixel(x, top, Luma([120]));
                    overlay.put_pixel(x, bottom - 1, Luma([120]));
                }
            }
            let overlay_path = root.join(name).join("reviewed-glyphs.png");
            overlay.save(&overlay_path).unwrap();
            let mut configuration = tier_b_configuration();
            configuration["audit_overrides"] = serde_json::json!({"temporary_source_body_bands":if protected {serde_json::json!([{"source":1,"page":562,"unit_id":"antonius","dimensions":[column.width(),column.height()],"observed":[249,1356],"protected":[249,1356]}])} else {serde_json::json!([])},"antonius_whole_strip_recognition":whole,"production_crop":trace.crop});
            let result = serde_json::json!({"schema_version":1,"event_id":event["event_id"],"unit_id":"antonius","configuration":name,"tested_configuration":configuration,
                "measured_line_count":reviewed.len(),"historical_line_count":23,"scale":scale,"production_column":trace.crop,"column_dimensions":[column.width(),column.height()],
                "raw_bands":rows(&raw),"furniture_filtered_bands":rows(&furniture),"pass_a_bands":rows(&pass_a),"pass_b_bands":trace.bands,
                "metrics":{"height_med":metrics.height_med,"height_mad":metrics.height_mad,"width_med":metrics.width_med},"strip":strip_record,
                "first_body_budget":geom.first_body_budget(),"body_budget":geom.body_budget(),"first_loss_stage":"pass_a_furniture_filter","first_failed_boundary":first_failed,
                "line_coverage":coverage,"canvas_ranges":ranges,"packed_canvas_hashes":packed_hashes,"canvas_hashes":tier_b_canvas_records(&returned),
                "source_pages":traces,"reviewed_lines":reviewed,"reviewed_annotations":annotations,"reviewed_neighbor_lines":event["diagnosis"]["reviewed_neighbor_lines"],"source_overlay":tier_b_file_reference(&overlay_path),
                "full_trace":tier_b_file_reference(&root.join(name).join("antonius/trace.json"))});
            std::fs::write(
                root.join(name).join("antonius.json"),
                serde_json::to_vec_pretty(&result).unwrap(),
            )
            .unwrap();
            println!(
                "{name}: {} canvases, first failed boundary {}",
                returned.len(),
                result["first_failed_boundary"]
            );
            summaries.insert(name.to_string(), result);
            if whole {
                selected_column = Some(column.clone());
            }
        });
    }
    let column = selected_column.unwrap();
    let ink = ink_profile_rows(&column, INK_THRESHOLD);
    let gaps: Vec<_>=reviewed.windows(2).map(|w| {
        let bottom=w[0].region[3];let top=w[1].region[1];
        let lo=bottom.min(top).saturating_sub(6);let hi=(bottom.max(top)+6).min(column.height());
        serde_json::json!({"vertical_gap":i64::from(top)-i64::from(bottom),"rows":[lo,hi],"minimum_ink":ink[lo as usize..hi as usize].iter().min().unwrap()})
    }).collect();
    assert!(
        gaps.iter().any(|g| g["vertical_gap"].as_i64().unwrap() < 0),
        "overlap rules out disjoint horizontal splitting"
    );
    let decision = serde_json::json!({"schema_version":1,"event_id":event["event_id"],"unit_id":"antonius","outcome":"candidate_verified","selected_method":"whole_block",
        "measured_line_count":22,"historical_line_count":23,"inter_line_seams":gaps,"stages":summaries.keys().map(|name|(name,tier_b_file_reference(&root.join(name).join("antonius.json")))).collect::<std::collections::BTreeMap<_,_>>()});
    std::fs::write(
        root.join("method-decision.json"),
        serde_json::to_vec_pretty(&decision).unwrap(),
    )
    .unwrap();
    assert!(AUDIT_EXTRA_BODY_BANDS.with(|slot| slot.get().is_empty()));
    assert!(!AUDIT_DISABLE_ANTONIUS_WHOLE_STRIP.with(|slot| slot.get()));
}

#[test]
fn antonius_trace_configuration_restores_on_failure() {
    let result = std::panic::catch_unwind(|| {
        with_antonius_audit_configuration(true, false, || panic!("intentional diagnostic failure"))
    });
    assert!(result.is_err());
    assert!(AUDIT_EXTRA_BODY_BANDS.with(|slot| slot.get().is_empty()));
    assert!(!AUDIT_DISABLE_ANTONIUS_WHOLE_STRIP.with(|slot| slot.get()));
}

#[test]
#[ignore = "requires the original Antonius source raster"]
fn antonius_annotation_crop_measurement() {
    let raw = real_source_image(1, 562);
    let mut masked = raw.clone();
    mask_known_source_furniture(1, 562, &mut masked);
    let col = crop_column(
        &masked,
        &Column {
            left: 895,
            right: 1721,
        },
    );
    let (bands, kept) = band_source_column(1, 562, &col);
    println!(
        "raster {:?}, annotation-preserving column {:?}, raw {:?}, kept {:?}",
        raw.dimensions(),
        col.dimensions(),
        bands,
        kept
    );
    assert!(
        bands.contains(&Band {
            top: 249,
            bottom: 1356
        }),
        "annotation extension retains the original block detector evidence"
    );
    let image = image::imageops::crop_imm(&col, 0, 249, col.width(), 1115).to_image();
    println!("block fingerprint {}", tier_b_sha(image.as_raw()));
    image
        .save("tmp/pdfs/inner_tomi_audit_repairs/antonius-with-complete-markers.png")
        .unwrap();
}

#[test]
fn tier_b_restoration_selection_rejects_missing_events_and_wrong_counts() {
    let mut ledger = audit_repair_ledger();
    for e in ledger["events"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .filter(|e| e["initial_tier"] == "B")
    {
        e["status"] = serde_json::json!("pending_repair");
        let lines: Vec<_>=(0..e["missing_line_count"].as_u64().unwrap()).map(|i|serde_json::json!({"source":0,"source_page":1,"region":[0,i*10,10,i*10+5],"role":"synthetic"})).collect();
        e["diagnosis"] = serde_json::json!({"reviewed_lines":lines});
    }
    assert_eq!(
        select_tier_b_restoration_events(&ledger, "head", true)
            .unwrap()
            .len(),
        8
    );
    for kind in [
        "missing",
        "wrong",
        "rectangle",
        "duplicate",
        "empty",
        "status",
    ] {
        let mut bad = ledger.clone();
        let events = bad["events"].as_array_mut().unwrap();
        let i = events
            .iter()
            .position(|e| e["initial_tier"] == "B" && e["unit_id"] == "lysander")
            .unwrap();
        match kind {
            "missing" => {
                events.remove(i);
            }
            "wrong" => events[i]["missing_line_count"] = serde_json::json!(1),
            "rectangle" => {
                events[i]["diagnosis"]["reviewed_lines"][0]["region"] =
                    serde_json::json!([1, 1, 0, 0])
            }
            "duplicate" => {
                events[i]["diagnosis"]["reviewed_lines"][1] =
                    events[i]["diagnosis"]["reviewed_lines"][0].clone()
            }
            "empty" => events.clear(),
            _ => events[i]["status"] = serde_json::json!("deferred"),
        }
        assert!(
            select_tier_b_restoration_events(&bad, "head", true).is_err(),
            "reject {kind}"
        );
    }
}

#[test]
fn reviewed_band_ledger_uses_production_crop_bounds() {
    let original = column(100, 110, &[(10, 20, 15, 95)]);
    let retained = image::imageops::crop_imm(&original, 10, 0, 90, 100).to_image();
    let pages = vec![(
        retained,
        vec![Band {
            top: 10,
            bottom: 20,
        }],
    )];
    let metrics = line_metrics(&pages);
    let traces = vec![RetainedSourcePageTrace {
        source: 0,
        page: 1,
        retained_index: Some(0),
        crop: [10, 100],
        bands: vec![[10, 20]],
        deleted_rows: vec![],
    }];
    let originals = std::collections::BTreeMap::from([((0, 1), original)]);
    let mut ledger = vec![ReviewedBands {
        source: 0,
        page: 1,
        dpi: BODY_DPI,
        column: [10, 100],
        body: vec![[10, 20]],
        excluded: vec![],
        noise: vec![],
    }];
    assert_reviewed_band_ledger("unit", &ledger, &pages, &metrics, &traces, &originals);
    ledger[0].column = [11, 100];
    assert!(std::panic::catch_unwind(|| assert_reviewed_band_ledger(
        "unit", &ledger, &pages, &metrics, &traces, &originals
    ))
    .is_err());
}

#[test]
fn reviewed_band_ledger_respects_shared_page_ownership() {
    let record = SOURCE_BODY_BANDS
        .iter()
        .find(|r| r.unit_id == "timoleon" && r.page == 318)
        .unwrap();
    let col = column(
        record.dimensions.1,
        record.dimensions.0,
        &[(1953, 1986, 10, 700)],
    );
    let pages = vec![(
        col.clone(),
        vec![Band {
            top: 1953,
            bottom: 1986,
        }],
    )];
    let metrics = line_metrics(&pages);
    let crop = [0, col.width()];
    let traces = vec![RetainedSourcePageTrace {
        source: record.source,
        page: record.page,
        retained_index: Some(0),
        crop,
        bands: vec![[1953, 1986]],
        deleted_rows: vec![],
    }];
    let originals = std::collections::BTreeMap::from([((record.source, record.page), col)]);
    let ledger = vec![ReviewedBands {
        source: record.source,
        page: record.page,
        dpi: BODY_DPI,
        column: crop,
        body: vec![[record.protected.top, record.protected.bottom]],
        excluded: vec![],
        noise: vec![],
    }];
    assert_reviewed_band_ledger(
        "aemilius_paulus",
        &ledger,
        &pages,
        &metrics,
        &traces,
        &originals,
    );
    assert!(std::panic::catch_unwind(|| assert_reviewed_band_ledger(
        "timoleon", &ledger, &pages, &metrics, &traces, &originals
    ))
    .is_err());
}

#[test]
fn tier_b_crop_edges_preserve_reviewed_head_glyphs() {
    for (source, page, dimensions, raw_bounds, expected, glyph) in [
        (0, 326, (1966, 2982), [922, 1712], [921, 1712], [921, 280]),
        (
            0,
            535,
            (1978, 2969),
            [1049, 1845],
            [1045, 1845],
            [1045, 255],
        ),
        (
            0,
            377,
            (1952, 2957),
            [1048, 1856],
            [1048, 1857],
            [1856, 265],
        ),
        (0, 318, (1937, 2988), [900, 1691], [899, 1691], [899, 270]),
    ] {
        assert_reviewed_crop_edge_signature(source, page, dimensions, raw_bounds, expected, glyph);
    }
}

fn assert_reviewed_crop_edge_signature(
    source: usize,
    page: u32,
    dimensions: (u32, u32),
    raw_bounds: [u32; 2],
    expected: [u32; 2],
    glyph: [u32; 2],
) {
    let mut raster = GrayImage::from_pixel(dimensions.0, dimensions.1, Luma([255]));
    raster.put_pixel(glyph[0], glyph[1], Luma([100]));
    let original = Column {
        left: raw_bounds[0],
        right: raw_bounds[1],
    };
    let preserved = preserve_known_body_crop_edge(source, page, &raster, original);
    assert_eq!([preserved.left, preserved.right], expected);
    let col = crop_column(&raster, &preserved);
    assert_eq!(col.get_pixel(glyph[0] - preserved.left, glyph[1])[0], 100);
    for (s, p, col) in [
        (1 - source, page, original),
        (source, page + 1, original),
        (
            source,
            page,
            Column {
                left: original.left + 1,
                right: original.right,
            },
        ),
    ] {
        let result = preserve_known_body_crop_edge(s, p, &raster, col);
        assert_eq!([result.left, result.right], [col.left, col.right]);
    }
    let other = GrayImage::from_pixel(dimensions.0 + 1, dimensions.1, Luma([255]));
    let result = preserve_known_body_crop_edge(source, page, &other, original);
    assert_eq!([result.left, result.right], raw_bounds);
}

#[test]
fn tier_b_crop_edges_preserve_reviewed_foot_glyphs() {
    for (source, page, dimensions, raw_bounds, expected, glyph) in [
        (1, 352, (1830, 2842), [879, 1683], [876, 1683], [876, 2535]),
        (1, 566, (1813, 2838), [892, 1697], [888, 1697], [888, 2550]),
        (1, 580, (1816, 2835), [892, 1704], [889, 1704], [889, 2560]),
    ] {
        assert_reviewed_crop_edge_signature(source, page, dimensions, raw_bounds, expected, glyph);
    }
}

#[test]
fn known_source_body_band_groups_preserve_subranges_and_parent_semantics() {
    let record = SOURCE_BODY_BANDS[0].observed;
    let first = Band {
        top: record.top,
        bottom: record.top + 5,
    };
    let second = Band {
        top: record.top + 10,
        bottom: record.bottom,
    };
    assert_eq!(
        merge_source_body_bands(vec![first, second]),
        vec![first, second]
    );
    assert_eq!(
        merge_source_body_bands(vec![
            first,
            Band {
                top: first.bottom,
                bottom: record.bottom
            }
        ]),
        vec![record]
    );
    assert_eq!(
        merge_source_body_bands(vec![record, first, second]),
        vec![record]
    );
}

#[test]
fn tier_b_validation_preserves_historical_tier_a_evidence() {
    validate_tier_a_completion_data(&audit_repair_ledger(), &portable_audit_evidence()).unwrap();
    let (page, _) = page_and_metrics(120, 100, &[(10, 70, 5, 95), (90, 100, 5, 95)]);
    let metrics = LineMetrics {
        height_med: 10,
        height_mad: 0,
        width_med: 90,
    };
    let originals = std::collections::BTreeMap::from([((0, 1), page.0.clone())]);
    let traces = [RetainedSourcePageTrace {
        source: 0,
        page: 1,
        retained_index: Some(0),
        crop: [0, 100],
        bands: vec![[10, 70], [90, 100]],
        deleted_rows: vec![],
    }];
    let pages = [page];
    let context = ReviewedLineContext {
        traces: &traces,
        originals: &originals,
        pages: &pages,
        metrics: &metrics,
        scale: 1.0,
        geom: TrimGeom {
            dpi: 1,
            tb_w: 100,
            tb_h: 40,
            pad_x: 0,
            pad_y: 0,
        },
    };
    let lines = [
        ReviewedSourceLine {
            source: 0,
            source_page: 1,
            region: [5, 10, 95, 70],
            role: "historical".into(),
        },
        ReviewedSourceLine {
            source: 0,
            source_page: 1,
            region: [5, 90, 95, 100],
            role: "new".into(),
        },
    ];
    let policies = std::collections::BTreeMap::from([(
        (0, 1, lines[1].region),
        ReviewedEvidencePolicy::PromotedTierB,
    )]);
    let result = assert_reviewed_source_lines_survive_with_policies(
        &lines,
        &context,
        &policies,
        ReviewedEvidencePolicy::HistoricalTierA,
    );
    assert!(
        result.line_coverage[0]["canvas_owners"]
            .as_array()
            .unwrap()
            .len()
            > 1
    );
    assert_eq!(
        result.line_coverage[1]["canvas_owners"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert!(
        std::panic::catch_unwind(|| assert_reviewed_source_lines_survive(&lines, &context, true))
            .is_err()
    );
}

fn measured_event_line_count(event: &serde_json::Value) -> Result<usize> {
    let historical = event["missing_line_count"]
        .as_u64()
        .ok_or_else(|| anyhow::anyhow!("missing historical line count"))?;
    if let Some(measured) = event["measured_line_count"].as_u64() {
        anyhow::ensure!(
            event["event_id"] == "miss_antonius_562_20869" && historical == 23 && measured == 22,
            "unapproved measured count correction"
        );
        evidence_string(&event["line_count_decision"], "path")?;
        evidence_digest(&event["line_count_decision"], "sha256")?;
        Ok(measured as usize)
    } else {
        Ok(historical as usize)
    }
}

fn reviewed_region_key(value: &serde_json::Value) -> Result<ReviewedRegionKey> {
    let source = value["source"]
        .as_u64()
        .filter(|s| *s < 2)
        .ok_or_else(|| anyhow::anyhow!("invalid reviewed source"))? as usize;
    let page = value["source_page"]
        .as_u64()
        .and_then(|p| u32::try_from(p).ok())
        .filter(|p| *p > 0)
        .ok_or_else(|| anyhow::anyhow!("invalid reviewed page"))?;
    let region: [u32; 4] = serde_json::from_value(value["region"].clone())?;
    anyhow::ensure!(
        region[0] < region[2] && region[1] < region[3],
        "empty reviewed region"
    );
    Ok((source, page, region))
}

fn project_restoration_report(value: &serde_json::Value) -> Result<serde_json::Value> {
    if value.get("schema_version").is_none() && value.get("verification_policy").is_none() {
        return project_evidence_fields(value, &["unit_id", "events", "reviewed_lines"]);
    }
    anyhow::ensure!(
        value["schema_version"] == 2 && value["verification_policy"] == "promoted_tier_b",
        "invalid modern restoration policy"
    );
    let mut projected = project_evidence_fields(
        value,
        &[
            "schema_version",
            "verification_policy",
            "unit_id",
            "events",
            "reviewed_lines",
            "scale",
            "line_coverage",
            "canvas_ranges",
            "canvas_hashes",
        ],
    )?;
    let pairs: std::collections::BTreeSet<_> = evidence_array(value, "reviewed_lines")?
        .iter()
        .map(|l| reviewed_region_key(l).map(|(s, p, _)| (s, p)))
        .collect::<Result<_>>()?;
    let pages: Vec<_> = evidence_array(value, "source_pages")?
        .iter()
        .filter(|p| {
            p["source"]
                .as_u64()
                .zip(p["page"].as_u64())
                .is_some_and(|(s, p)| pairs.contains(&(s as usize, p as u32)))
        })
        .map(|p| {
            anyhow::ensure!(
                p["retained_index"].as_u64().is_some(),
                "reviewed source page not retained"
            );
            project_evidence_fields(
                p,
                &[
                    "source",
                    "page",
                    "retained_index",
                    "crop",
                    "bands",
                    "deleted_rows",
                ],
            )
        })
        .collect::<Result<_>>()?;
    let found: std::collections::BTreeSet<_> = pages
        .iter()
        .map(|p| {
            (
                p["source"].as_u64().unwrap() as usize,
                p["page"].as_u64().unwrap() as u32,
            )
        })
        .collect();
    anyhow::ensure!(
        found == pairs && pages.len() == pairs.len(),
        "missing or duplicate reviewed source record"
    );
    projected["source_pages"] = pages.into();
    Ok(projected)
}

fn validate_promoted_line_coverage(
    report: &serde_json::Value,
    required: &[ReviewedSourceLine],
) -> Result<()> {
    anyhow::ensure!(
        report["schema_version"] == 2 && report["verification_policy"] == "promoted_tier_b",
        "promoted report requires versioned coverage policy"
    );
    anyhow::ensure!(
        report["scale"]
            .as_f64()
            .is_some_and(|v| v.is_finite() && v > 0.0),
        "invalid report scale"
    );
    let ranges: Vec<(usize, u32, u32)> = serde_json::from_value(report["canvas_ranges"].clone())?;
    anyhow::ensure!(!ranges.is_empty(), "missing canvas ranges");
    let hashes = evidence_array(report, "canvas_hashes")?;
    let mut hash_map = std::collections::BTreeMap::new();
    for hash in hashes {
        let id = hash["canvas"]
            .as_u64()
            .filter(|v| *v > 0)
            .ok_or_else(|| anyhow::anyhow!("invalid canvas id"))? as usize;
        let width = hash["width"]
            .as_u64()
            .filter(|v| *v > 0)
            .ok_or_else(|| anyhow::anyhow!("invalid canvas width"))?;
        let height = hash["height"]
            .as_u64()
            .filter(|v| *v > 0)
            .ok_or_else(|| anyhow::anyhow!("invalid canvas height"))?;
        evidence_digest(hash, "sha256")?;
        anyhow::ensure!(
            hash_map.insert(id, (width, height)).is_none(),
            "duplicate canvas hash record"
        );
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut previous = None;
    for &(id, start, end) in &ranges {
        anyhow::ensure!(
            id > 0 && start < end && seen.insert(id),
            "invalid or duplicate canvas range"
        );
        if let Some((previous_id, previous_end)) = previous {
            anyhow::ensure!(
                id > previous_id && start >= previous_end,
                "unordered or overlapping stream ranges"
            );
        }
        let (_, height) = hash_map
            .get(&id)
            .ok_or_else(|| anyhow::anyhow!("range has no canvas hash"))?;
        anyhow::ensure!(
            *height >= u64::from(end - start),
            "canvas dimensions differ from stream extent"
        );
        previous = Some((id, end));
    }
    anyhow::ensure!(
        seen == hash_map.keys().copied().collect(),
        "canvas/hash identity mismatch"
    );
    let reviewed = evidence_array(report, "reviewed_lines")?;
    let coverage = evidence_array(report, "line_coverage")?;
    let pages = evidence_array(report, "source_pages")?;
    let mut source_identities = std::collections::BTreeSet::new();
    let mut retained_indices = std::collections::BTreeSet::new();
    let mut previous_page = None;
    let mut previous_index = None;
    for page in pages {
        let source = page["source"]
            .as_u64()
            .filter(|s| *s < 2)
            .ok_or_else(|| anyhow::anyhow!("invalid retained source"))?;
        let physical = page["page"]
            .as_u64()
            .filter(|p| *p > 0)
            .ok_or_else(|| anyhow::anyhow!("invalid physical page"))?;
        anyhow::ensure!(
            source_identities.insert((source, physical)),
            "duplicate source identity"
        );
        anyhow::ensure!(
            previous_page.is_none_or(|(s, p)| s == source && p < physical),
            "retained source records out of order"
        );
        previous_page = Some((source, physical));
        anyhow::ensure!(
            page.get("retained_index").is_some(),
            "missing retained index marker"
        );
        if page["retained_index"].is_null() {
            anyhow::ensure!(
                skip_known_duplicate_source_page(
                    evidence_string(report, "unit_id")?,
                    source as usize,
                    physical as u32
                ),
                "unreviewed excluded source page"
            );
            anyhow::ensure!(
                evidence_array(page, "bands")?.is_empty(),
                "excluded source page has retained bands"
            );
            continue;
        }
        let index = page["retained_index"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("invalid retained index"))?;
        anyhow::ensure!(
            retained_indices.insert(index) && previous_index.is_none_or(|i| i < index),
            "duplicate or out-of-order retained index"
        );
        previous_index = Some(index);
    }
    let mut required_keys = std::collections::BTreeSet::new();
    for line in required {
        let key = (line.source, line.source_page, line.region);
        anyhow::ensure!(required_keys.insert(key), "duplicate required line");
        let matching = |v: &&serde_json::Value| reviewed_region_key(v).is_ok_and(|k| k == key);
        anyhow::ensure!(
            reviewed.iter().filter(matching).count() == 1,
            "missing or duplicate required reviewed line"
        );
        let matches: Vec<_> = coverage.iter().filter(matching).collect();
        anyhow::ensure!(
            matches.len() == 1,
            "missing, wrong-region, or duplicate required coverage"
        );
        let entry = matches[0];
        let interval: [u32; 2] = serde_json::from_value(entry["scaled_row_stream"].clone())?;
        anyhow::ensure!(interval[0] < interval[1], "empty scaled interval");
        let owners: Vec<usize> = serde_json::from_value(entry["canvas_owners"].clone())?;
        anyhow::ensure!(
            owners.len() == 1,
            "promoted line must have exactly one owner"
        );
        let owner = ranges
            .iter()
            .find(|(id, _, _)| *id == owners[0])
            .ok_or_else(|| anyhow::anyhow!("out-of-range canvas owner"))?;
        anyhow::ensure!(
            owner.1 <= interval[0] && owner.2 >= interval[1],
            "line spans canvases or a stream gap"
        );
        anyhow::ensure!(
            ranges
                .iter()
                .filter(|(_, start, end)| *start < interval[1] && *end > interval[0])
                .count()
                == 1,
            "competing canvas ownership"
        );
        let matches: Vec<_> = pages
            .iter()
            .filter(|p| p["source"] == line.source && p["page"] == line.source_page)
            .collect();
        anyhow::ensure!(
            matches.len() == 1,
            "missing or duplicate retained source page"
        );
        let page = matches[0];
        anyhow::ensure!(
            page["retained_index"].as_u64().is_some(),
            "required source page excluded"
        );
        let crop: [u32; 2] = serde_json::from_value(page["crop"].clone())?;
        anyhow::ensure!(
            crop[0] <= line.region[0] && crop[1] >= line.region[2],
            "required glyphs outside production crop"
        );
        let bands: Vec<[u32; 2]> = serde_json::from_value(page["bands"].clone())?;
        anyhow::ensure!(
            bands.windows(2).all(|w| w[0][1] <= w[1][0]),
            "overlapping source bands"
        );
        anyhow::ensure!(
            bands
                .iter()
                .filter(|b| b[0] <= line.region[1] && b[1] >= line.region[3])
                .count()
                == 1,
            "source band does not cover required glyphs once"
        );
        let deleted: Vec<[u32; 2]> = serde_json::from_value(page["deleted_rows"].clone())?;
        anyhow::ensure!(
            deleted
                .iter()
                .all(|b| b[0] >= line.region[3] || b[1] <= line.region[1]),
            "required glyph rows deleted"
        );
    }
    Ok(())
}

fn validate_restored_event_evidence(
    event: &serde_json::Value,
    documents: &serde_json::Value,
    policy: ReviewedEvidencePolicy,
    require_human: bool,
) -> Result<()> {
    let reference = &event["passing_evidence"];
    let report = portable_evidence_document(documents, reference)?;
    anyhow::ensure!(
        report["unit_id"] == event["unit_id"],
        "passing report unit differs"
    );
    anyhow::ensure!(
        evidence_array(report, "events")?.contains(&event["event_id"]),
        "event absent from passing evidence"
    );
    let lines: Vec<ReviewedSourceLine> =
        serde_json::from_value(event["diagnosis"]["reviewed_lines"].clone())?;
    anyhow::ensure!(
        lines.len() == measured_event_line_count(event)?,
        "source line count differs"
    );
    for line in &lines {
        anyhow::ensure!(
            evidence_array(event, "source_regions")?
                .iter()
                .any(|r| r["source_index"] == line.source && r["page"] == line.source_page),
            "reviewed line outside event source identity"
        );
        anyhow::ensure!(
            evidence_array(report, "reviewed_lines")?.contains(&serde_json::to_value(line)?),
            "reviewed line absent from passing evidence"
        );
    }
    if policy == ReviewedEvidencePolicy::PromotedTierB {
        validate_promoted_line_coverage(report, &lines)?;
        if require_human {
            let review = &event["human_review"];
            normalized_evidence_path(evidence_string(review, "path")?)?;
            evidence_digest(review, "sha256")?;
            evidence_string(review, "event_section")?;
            anyhow::ensure!(
                review["event_id"] == event["event_id"]
                    && review["passing_report_sha256"] == reference["sha256"],
                "human review differs from accepted event/report"
            );
        }
    }
    Ok(())
}

fn validate_tier_b_promoted_diagnosis(
    event: &serde_json::Value,
    documents: &serde_json::Value,
) -> Result<()> {
    let promotion = &event["promotion"];
    normalized_evidence_path(evidence_string(promotion, "worksheet_path")?)?;
    evidence_digest(promotion, "worksheet_sha256")?;
    evidence_string(promotion, "event_section")?;
    evidence_string(promotion, "sign_off")?;
    evidence_string(event, "repair_read_status")?;
    let diagnosis = &event["diagnosis"];
    let trace = portable_evidence_document(documents, &diagnosis["baseline_trace"])?;
    validate_trace_metadata(trace, &event["unit_id"])?;
    let overlay = &diagnosis["reviewed_source_overlay"];
    normalized_evidence_path(evidence_string(overlay, "path")?)?;
    evidence_digest(overlay, "sha256")?;
    let origin: [u32; 2] = serde_json::from_value(overlay["source_origin"].clone())?;
    anyhow::ensure!(origin == [0, 0], "full-source overlay origin differs");
    let lines = evidence_array(diagnosis, "reviewed_lines")?;
    anyhow::ensure!(
        lines.len() == measured_event_line_count(event)?,
        "promoted diagnosis line count differs"
    );
    let mut seen = std::collections::BTreeSet::new();
    let mut previous = None;
    for line in lines {
        let key = reviewed_region_key(line)?;
        anyhow::ensure!(seen.insert(key), "duplicate promoted source line");
        anyhow::ensure!(
            previous.is_none_or(|p| p < (key.1, key.2[1])),
            "promoted lines out of source order"
        );
        previous = Some((key.1, key.2[1]));
        anyhow::ensure!(
            evidence_array(event, "source_regions")?
                .iter()
                .any(|r| r["source_index"] == key.0 && r["page"] == key.1),
            "promoted source identity differs"
        );
        evidence_string(&line["baseline"], "first_loss_stage")?;
        evidence_string(line, "landmark")?;
    }
    anyhow::ensure!(
        !evidence_array(diagnosis, "reviewed_neighbor_lines")?.is_empty(),
        "missing neighboring anchors"
    );
    evidence_string(diagnosis, "regression_name")?;
    Ok(())
}

fn validate_tier_b_disposition(
    event: &serde_json::Value,
    disposition: &serde_json::Value,
    documents: &serde_json::Value,
) -> Result<()> {
    let report = portable_evidence_document(documents, &disposition["disposition_evidence"])?;
    let trace = portable_evidence_document(documents, &disposition["disposition_trace"])?;
    anyhow::ensure!(
        report["schema_version"] == 1
            && trace["schema_version"] == 1
            && report["status"] == "unresolved",
        "invalid disposition schema/status"
    );
    for value in [report, trace] {
        anyhow::ensure!(
            value["event_id"] == event["event_id"] && value["unit_id"] == event["unit_id"],
            "disposition event/unit differs"
        );
        anyhow::ensure!(
            value["tested_configuration"] == disposition["tested_configuration"],
            "disposition configuration differs"
        );
        let configuration = &value["tested_configuration"];
        anyhow::ensure!(
            evidence_array(configuration, "sources")?.len() == 2
                && !evidence_array(configuration, "production_files")?.is_empty(),
            "missing disposition configuration identities"
        );
        evidence_digest(&configuration["unit_table"], "sha256")?;
        for reference in evidence_array(configuration, "sources")?
            .iter()
            .chain(evidence_array(configuration, "production_files")?)
        {
            evidence_digest(reference, "sha256")?;
            normalized_evidence_path(evidence_string(reference, "path")?)?;
        }
    }
    anyhow::ensure!(
        report["trace_evidence"] == disposition["disposition_trace"],
        "disposition trace reference differs"
    );
    anyhow::ensure!(
        report["reason"] == disposition["reason"],
        "disposition reason differs"
    );
    evidence_string(report, "reason")?;
    let stage = evidence_string(report, "failure_stage")?;
    if stage == "not_established" {
        evidence_string(report, "remaining_uncertainty")?;
    }
    anyhow::ensure!(
        !evidence_array(trace, "stage_records")?.is_empty(),
        "missing measured disposition stages"
    );
    Ok(())
}

fn validate_tier_b_repair_state_data(
    ledger: &serde_json::Value,
    documents: &serde_json::Value,
) -> Result<()> {
    validate_omission_inventory_data(ledger, documents)?;
    let delivery = &ledger["tier_b_delivery"];
    let state = evidence_string(delivery, "state")?;
    anyhow::ensure!(
        ["in_progress", "partial", "complete"].contains(&state),
        "invalid delivery state"
    );
    let promoted: Vec<_> = evidence_array(ledger, "events")?
        .iter()
        .filter(|e| e["initial_tier"] == "B")
        .collect();
    anyhow::ensure!(
        promoted.len() == 20
            && promoted
                .iter()
                .map(|e| e["missing_line_count"].as_u64().unwrap_or(0))
                .sum::<u64>()
                == 44,
        "initial Tier-B inventory differs"
    );
    let manifest = portable_evidence_document(documents, &ledger["tier_b_baseline"])?;
    let units: Vec<crate::facsimile::units::Unit> =
        serde_json::from_str(include_str!("../../facsimile_units.json"))?;
    let ids: Vec<_> = units.iter().map(|u| u.id.clone()).collect();
    let scale = manifest["scale"]
        .as_f64()
        .ok_or_else(|| anyhow::anyhow!("baseline scale missing"))? as f32;
    validate_tier_b_baseline_manifest(manifest, &ids, &manifest["configuration"], scale)?;
    anyhow::ensure!(
        manifest["protection_record_count"] == 47,
        "unsafe pre-installation baseline"
    );
    anyhow::ensure!(
        manifest["configuration"]["unit_table"]["sha256"]
            == tier_b_sha(include_bytes!("../../facsimile_units.json")),
        "baseline unit-table configuration differs"
    );
    for (entry, unit) in evidence_array(manifest, "units")?.iter().zip(&units) {
        anyhow::ensure!(
            entry["source"] == unit.source,
            "baseline source/unit mismatch"
        );
    }
    let mut restored = [0, 0];
    let mut unresolved = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for event in promoted {
        let id = evidence_string(event, "event_id")?;
        anyhow::ensure!(seen.insert(id), "duplicate promoted event");
        anyhow::ensure!(event["promoted_tier"] == "A", "missing promotion");
        validate_tier_b_promoted_diagnosis(event, documents)?;
        let status = evidence_string(event, "status")?;
        match status {
            "pending_repair" => {
                anyhow::ensure!(
                    event.get("passing_evidence").is_none(),
                    "pending event claims passing evidence"
                );
            }
            "installed" | "restored" => {
                validate_restored_event_evidence(
                    event,
                    documents,
                    ReviewedEvidencePolicy::PromotedTierB,
                    status == "restored",
                )?;
                if status == "restored" {
                    restored[0] += 1;
                    restored[1] += measured_event_line_count(event)?;
                }
            }
            "unresolved" => {
                validate_tier_b_disposition(event, event, documents)?;
                unresolved.push(id.to_string());
                anyhow::ensure!(
                    event.get("passing_evidence").is_none(),
                    "unresolved event claims passing evidence"
                );
            }
            _ => anyhow::bail!("invalid promoted event status {status}"),
        }
        for historical in event["historical_dispositions"]
            .as_array()
            .into_iter()
            .flatten()
        {
            validate_tier_b_disposition(event, historical, documents)?;
        }
    }
    unresolved.sort();
    let mut declared: Vec<String> =
        serde_json::from_value(delivery["unresolved_event_ids"].clone())?;
    declared.sort();
    anyhow::ensure!(
        declared == unresolved,
        "unresolved delivery inventory differs"
    );
    anyhow::ensure!(
        delivery["restored_event_count"] == restored[0]
            && delivery["restored_line_count"] == restored[1],
        "false restored delivery counts"
    );
    match state {
        "partial" => anyhow::ensure!(
            restored == [19, 21] && unresolved == ["miss_antonius_562_20869"],
            "partial delivery must restore 19/21 and authenticate unresolved Antonius"
        ),
        "complete" => anyhow::ensure!(
            restored == [20, 43] && unresolved.is_empty(),
            "complete delivery must restore 20/43 measured lines without unresolved events"
        ),
        _ => {}
    }
    Ok(())
}

#[test]
#[ignore = "requires authenticated promoted evidence and every baseline unit trace"]
fn audit_evidence_tier_b_state_and_baseline_authenticated() -> Result<()> {
    let ledger = audit_repair_ledger();
    let root = AuditEvidenceRoot::from_environment();
    let documents = load_fixture_source_documents(&ledger, &root.root)?;
    validate_tier_b_repair_state_data(&ledger, &documents)?;
    let manifest = root.read_verified_json(&ledger["tier_b_baseline"])?;
    for entry in evidence_array(&manifest, "units")? {
        root.verify_streamed(&entry["trace"])?;
        let trace = root.read_verified_json(&entry["trace"])?;
        anyhow::ensure!(
            trace["unit_id"] == entry["unit_id"]
                && trace["source"] == entry["source"]
                && trace["scale"] == manifest["scale"],
            "baseline trace identity differs"
        );
        anyhow::ensure!(
            tier_b_sha(&serde_json::to_vec(&trace["canvas_hashes"])?)
                == entry["canvas_sequence_sha256"],
            "baseline ordered canvas digest differs"
        );
        anyhow::ensure!(
            trace["metrics"] == entry["metrics"] && trace["canvas_count"] == entry["canvas_count"],
            "baseline trace summary differs"
        );
    }
    for event in evidence_array(&ledger, "events")?
        .iter()
        .filter(|e| e["initial_tier"] == "B")
    {
        root.verify_streamed(&event["diagnosis"]["reviewed_source_overlay"])?;
        root.verify_streamed(&event["promotion"]["worksheet_reference"])?;
        if event["status"] == "restored" {
            root.verify_streamed(&event["human_review"])?;
        }
        if event.get("line_count_decision").is_some() {
            root.verify_streamed(&event["line_count_decision"])?;
        }
    }
    println!("authenticated all baseline unit traces and staged Tier-B evidence");
    Ok(())
}

fn synthetic_promoted_report(event: &serde_json::Value) -> serde_json::Value {
    let lines: Vec<ReviewedSourceLine> =
        serde_json::from_value(event["diagnosis"]["reviewed_lines"].clone()).unwrap();
    let mut grouped = std::collections::BTreeMap::<(usize, u32), Vec<[u32; 4]>>::new();
    for l in &lines {
        grouped
            .entry((l.source, l.source_page))
            .or_default()
            .push(l.region);
    }
    let pages: Vec<_>=grouped.into_iter().enumerate().map(|(i,((source,page),regions))| {
        let left=regions.iter().map(|r|r[0]).min().unwrap();
        let right=regions.iter().map(|r|r[2]).max().unwrap();
        let bands=merge_source_body_bands(regions.iter().map(|r|Band {top:r[1],bottom:r[3]}).collect());
        serde_json::json!({"source":source,"page":page,"retained_index":i,"crop":[left,right],"bands":bands.iter().map(|b|[b.top,b.bottom]).collect::<Vec<_>>(),"deleted_rows":[]})
    }).collect();
    let coverage: Vec<_>=lines.iter().enumerate().map(|(i,l)|serde_json::json!({"source":l.source,"source_page":l.source_page,"region":l.region,"scaled_row_stream":[i*100+10,i*100+90],"canvas_owners":[1]})).collect();
    let extent = lines.len() * 100 + 100;
    serde_json::json!({"schema_version":2,"verification_policy":"promoted_tier_b","unit_id":event["unit_id"],"events":[event["event_id"]],"reviewed_lines":lines,
        "scale":1.0,"line_coverage":coverage,"canvas_ranges":[[1,0,extent]],"canvas_hashes":[{"canvas":1,"width":1100,"height":extent,"sha256":"a".repeat(64)}],"source_pages":pages})
}

fn synthetic_tier_b_delivery(state: &str) -> (serde_json::Value, serde_json::Value) {
    let mut ledger = audit_repair_ledger();
    let mut documents = portable_audit_evidence();
    let mut restored = [0, 0];
    for e in ledger["events"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .filter(|e| e["initial_tier"] == "B")
    {
        for key in [
            "passing_evidence",
            "human_review",
            "disposition_evidence",
            "disposition_trace",
            "tested_configuration",
        ] {
            e.as_object_mut().unwrap().remove(key);
        }
        let id = e["event_id"].as_str().unwrap().to_string();
        if state == "partial" && e["unit_id"] == "antonius" {
            e["status"] = serde_json::json!("unresolved");
            e["reason"] = serde_json::json!("candidate glyph-safe composition not established");
            let configuration = serde_json::json!({"sources":[{"path":"source1.pdf","sha256":"b".repeat(64)},{"path":"source2.pdf","sha256":"c".repeat(64)}],"unit_table":{"path":"units.json","sha256":"d".repeat(64)},"production_files":[{"path":"code.rs","sha256":"e".repeat(64)}]});
            e["tested_configuration"] = configuration.clone();
            let trace_ref = serde_json::json!({"path":"archive/synthetic/antonius-trace.json","sha256":"f".repeat(64)});
            let report_ref = serde_json::json!({"path":"archive/synthetic/antonius-disposition.json","sha256":"0".repeat(64)});
            let trace = serde_json::json!({"schema_version":1,"event_id":id,"unit_id":"antonius","tested_configuration":configuration,"stage_records":[{"production_column":[906,1721],"first_failed_boundary":"not_established"}]});
            let report = serde_json::json!({"schema_version":1,"event_id":id,"unit_id":"antonius","status":"unresolved","reason":e["reason"],"tested_configuration":configuration,"failure_stage":"not_established","remaining_uncertainty":"candidate reader check remains incomplete","trace_evidence":trace_ref});
            documents["documents"][trace_ref["path"].as_str().unwrap()] =
                serde_json::json!({"source_sha256":trace_ref["sha256"],"value":trace});
            documents["documents"][report_ref["path"].as_str().unwrap()] =
                serde_json::json!({"source_sha256":report_ref["sha256"],"value":report});
            e["disposition_evidence"] = report_ref;
            e["disposition_trace"] = trace_ref;
        } else {
            let report = synthetic_promoted_report(e);
            let reference = serde_json::json!({"path":format!("archive/synthetic/{id}.json"),"sha256":tier_b_sha(&serde_json::to_vec(&report).unwrap())});
            documents["documents"][reference["path"].as_str().unwrap()] =
                serde_json::json!({"source_sha256":reference["sha256"],"value":report});
            e["passing_evidence"] = reference.clone();
            e["status"] = serde_json::json!("restored");
            e["human_review"] = serde_json::json!({"path":"llm_artifacts/synthetic_review.md","sha256":"a".repeat(64),"event_id":id,"event_section":id,"passing_report_sha256":reference["sha256"]});
            restored[0] += 1;
            restored[1] += measured_event_line_count(e).unwrap();
        }
    }
    ledger["tier_b_delivery"] = serde_json::json!({"state":state,"restored_event_count":restored[0],"restored_line_count":restored[1],"unresolved_event_ids":if state=="partial" {vec!["miss_antonius_562_20869"]} else {vec![]}});
    (ledger, documents)
}

#[test]
fn tier_b_promotions_preserve_initial_inventory() {
    let ledger = audit_repair_ledger();
    validate_omission_inventory_data(&ledger, &portable_audit_evidence()).unwrap();
    let events: Vec<_> = ledger["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["initial_tier"] == "B")
        .collect();
    assert_eq!(events.len(), 20);
    assert_eq!(
        events
            .iter()
            .map(|e| e["missing_line_count"].as_u64().unwrap())
            .sum::<u64>(),
        44
    );
    assert_eq!(
        events
            .iter()
            .map(|e| measured_event_line_count(e).unwrap())
            .sum::<usize>(),
        43
    );
    assert!(events.iter().all(|e| e["promoted_tier"] == "A"));
}

#[test]
fn tier_b_delivery_fixture_is_consistent() {
    validate_tier_b_repair_state_data(&audit_repair_ledger(), &portable_audit_evidence()).unwrap();
}

#[test]
fn tier_b_restoration_projection_requires_coverage_fields() {
    let event = audit_repair_ledger()["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["initial_tier"] == "B")
        .unwrap()
        .clone();
    let good = synthetic_promoted_report(&event);
    project_restoration_report(&good).unwrap();
    for key in [
        "schema_version",
        "verification_policy",
        "scale",
        "line_coverage",
        "canvas_ranges",
        "canvas_hashes",
        "source_pages",
    ] {
        let mut bad = good.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(
            project_restoration_report(&bad).is_err(),
            "projection rejects missing {key}"
        );
    }
    let mut bad = good.clone();
    let duplicate = bad["source_pages"][0].clone();
    bad["source_pages"].as_array_mut().unwrap().push(duplicate);
    assert!(project_restoration_report(&bad).is_err());
    let legacy = serde_json::json!({"unit_id":"legacy","events":[],"reviewed_lines":[]});
    assert_eq!(project_restoration_report(&legacy).unwrap(), legacy);
}

#[test]
fn tier_b_coverage_rejects_missing_wrong_region_and_invalid_owners() {
    let event = audit_repair_ledger()["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["initial_tier"] == "B" && e["unit_id"] == "lysander")
        .unwrap()
        .clone();
    let lines: Vec<ReviewedSourceLine> =
        serde_json::from_value(event["diagnosis"]["reviewed_lines"].clone()).unwrap();
    let good = synthetic_promoted_report(&event);
    validate_promoted_line_coverage(&good, &lines).unwrap();
    for kind in [
        "missing",
        "region",
        "empty_owner",
        "extra_owner",
        "invalid_owner",
        "gap",
        "hash",
        "crop",
        "source",
        "deleted",
        "downgrade",
        "dimensions",
    ] {
        let mut bad = good.clone();
        match kind {
            "missing" => {
                bad["line_coverage"].as_array_mut().unwrap().remove(0);
            }
            "region" => bad["line_coverage"][0]["region"] = serde_json::json!([0, 0, 1, 1]),
            "empty_owner" => bad["line_coverage"][0]["canvas_owners"] = serde_json::json!([]),
            "extra_owner" => bad["line_coverage"][0]["canvas_owners"] = serde_json::json!([1, 2]),
            "invalid_owner" => bad["line_coverage"][0]["canvas_owners"] = serde_json::json!([99]),
            "gap" => bad["canvas_ranges"][0][1] = serde_json::json!(50),
            "hash" => bad["canvas_hashes"][0]["sha256"] = serde_json::json!("invalid"),
            "crop" => bad["source_pages"][0]["crop"] = serde_json::json!([0, 1]),
            "source" => bad["source_pages"][0]["page"] = serde_json::json!(9999),
            "deleted" => {
                bad["source_pages"][0]["deleted_rows"] =
                    serde_json::json!([[lines[0].region[1], lines[0].region[3]]])
            }
            "downgrade" => {
                bad.as_object_mut().unwrap().remove("schema_version");
            }
            _ => bad["canvas_hashes"][0]["height"] = serde_json::json!(1),
        }
        assert!(
            validate_promoted_line_coverage(&bad, &lines).is_err(),
            "coverage rejects {kind}"
        );
    }
}

#[test]
fn tier_b_coverage_rejects_duplicate_and_split_lines() {
    let event = audit_repair_ledger()["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["initial_tier"] == "B")
        .unwrap()
        .clone();
    let lines: Vec<ReviewedSourceLine> =
        serde_json::from_value(event["diagnosis"]["reviewed_lines"].clone()).unwrap();
    let good = synthetic_promoted_report(&event);
    for kind in [
        "duplicate_reviewed",
        "duplicate_coverage",
        "duplicate_required",
        "split",
        "overlap",
    ] {
        let mut bad = good.clone();
        let mut required = lines.clone();
        match kind {
            "duplicate_reviewed" => {
                let line = bad["reviewed_lines"][0].clone();
                bad["reviewed_lines"].as_array_mut().unwrap().push(line);
            }
            "duplicate_coverage" => {
                let line = bad["line_coverage"][0].clone();
                bad["line_coverage"].as_array_mut().unwrap().push(line);
            }
            "duplicate_required" => required.push(lines[0].clone()),
            "split" => {
                bad["canvas_ranges"] = serde_json::json!([[1, 0, 50], [2, 50, 200]]);
                let mut second = bad["canvas_hashes"][0].clone();
                second["canvas"] = serde_json::json!(2);
                bad["canvas_hashes"].as_array_mut().unwrap().push(second);
                bad["line_coverage"][0]["canvas_owners"] = serde_json::json!([1, 2]);
            }
            _ => {
                bad["canvas_ranges"] = serde_json::json!([[1, 0, 200], [2, 40, 100]]);
            }
        }
        assert!(
            validate_promoted_line_coverage(&bad, &required).is_err(),
            "reject {kind}"
        );
    }
}

#[test]
fn tier_b_coverage_preserves_reviewed_duplicate_page_exclusions() {
    let event = audit_repair_ledger()["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| {
            e["initial_tier"] == "B"
                && e["unit_id"] == "cato_minor"
                && e["source_regions"][0]["page"] == 352
        })
        .unwrap()
        .clone();
    let lines: Vec<ReviewedSourceLine> =
        serde_json::from_value(event["diagnosis"]["reviewed_lines"].clone()).unwrap();
    let mut report = synthetic_promoted_report(&event);
    report["source_pages"].as_array_mut().unwrap().insert(0,serde_json::json!({"source":1,"page":339,"retained_index":null,"crop":[0,100],"bands":[],"deleted_rows":[]}));
    validate_promoted_line_coverage(&report, &lines).unwrap();
    let projected = project_restoration_report(&report).unwrap();
    assert_eq!(projected["source_pages"].as_array().unwrap().len(), 1);
    validate_promoted_line_coverage(&projected, &lines).unwrap();
    for kind in [
        "unknown_exclusion",
        "retained_bands",
        "missing_marker",
        "required_excluded",
    ] {
        let mut bad = report.clone();
        match kind {
            "unknown_exclusion" => bad["source_pages"][0]["page"] = serde_json::json!(338),
            "retained_bands" => bad["source_pages"][0]["bands"] = serde_json::json!([[1, 2]]),
            "missing_marker" => {
                bad["source_pages"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("retained_index");
            }
            _ => bad["source_pages"][1]["retained_index"] = serde_json::Value::Null,
        }
        assert!(
            validate_promoted_line_coverage(&bad, &lines).is_err(),
            "reject {kind}"
        );
    }
}

#[test]
fn tier_b_delivery_accepts_partial_and_complete() {
    for state in ["partial", "complete"] {
        let (ledger, documents) = synthetic_tier_b_delivery(state);
        validate_tier_b_repair_state_data(&ledger, &documents).unwrap();
        let one = project_audit_evidence(&ledger, &documents).unwrap();
        let two = project_audit_evidence(&ledger, &documents).unwrap();
        assert_eq!(one, two);
        validate_tier_b_repair_state_data(&ledger, &one).unwrap();
    }
    let (mut ledger, documents) = synthetic_tier_b_delivery("partial");
    ledger["tier_b_delivery"]["state"] = serde_json::json!("in_progress");
    let event = ledger["events"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["initial_tier"] == "B" && e["unit_id"] == "lysander")
        .unwrap();
    event["status"] = serde_json::json!("installed");
    event.as_object_mut().unwrap().remove("human_review");
    ledger["tier_b_delivery"]["restored_event_count"] = serde_json::json!(18);
    ledger["tier_b_delivery"]["restored_line_count"] = serde_json::json!(19);
    validate_tier_b_repair_state_data(&ledger, &documents).unwrap();
    let event = ledger["events"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|e| e["initial_tier"] == "B" && e["unit_id"] == "aemilius_paulus")
        .unwrap();
    event["status"] = serde_json::json!("pending_repair");
    event.as_object_mut().unwrap().remove("passing_evidence");
    event.as_object_mut().unwrap().remove("human_review");
    ledger["tier_b_delivery"]["restored_event_count"] = serde_json::json!(17);
    ledger["tier_b_delivery"]["restored_line_count"] = serde_json::json!(18);
    validate_tier_b_repair_state_data(&ledger, &documents).unwrap();
    ledger["tier_b_delivery"]["restored_line_count"] = serde_json::json!(20);
    assert!(validate_tier_b_repair_state_data(&ledger, &documents).is_err());
}

#[test]
fn tier_b_delivery_rejects_missing_lines_and_evidence() {
    let (good, documents) = synthetic_tier_b_delivery("complete");
    for kind in ["line", "evidence", "human", "status", "event", "count"] {
        let mut bad = good.clone();
        let event = bad["events"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["initial_tier"] == "B")
            .unwrap();
        match kind {
            "line" => {
                event["diagnosis"]["reviewed_lines"]
                    .as_array_mut()
                    .unwrap()
                    .pop();
            }
            "evidence" => {
                event.as_object_mut().unwrap().remove("passing_evidence");
            }
            "human" => {
                event.as_object_mut().unwrap().remove("human_review");
            }
            "status" => event["status"] = serde_json::json!("pending_repair"),
            "event" => event["event_id"] = serde_json::json!("invented"),
            _ => bad["tier_b_delivery"]["restored_event_count"] = serde_json::json!(99),
        }
        assert!(
            validate_tier_b_repair_state_data(&bad, &documents).is_err(),
            "delivery rejects {kind}"
        );
    }
}

#[test]
fn tier_b_delivery_rejects_unresolved_complete_claim() {
    let (mut ledger, documents) = synthetic_tier_b_delivery("partial");
    ledger["tier_b_delivery"]["state"] = serde_json::json!("complete");
    assert!(validate_tier_b_repair_state_data(&ledger, &documents).is_err());
}

#[test]
fn tier_b_delivery_rejects_missing_unresolved_evidence() {
    let (good, documents) = synthetic_tier_b_delivery("partial");
    for key in [
        "disposition_evidence",
        "disposition_trace",
        "reason",
        "tested_configuration",
    ] {
        let mut bad = good.clone();
        let event = bad["events"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|e| e["initial_tier"] == "B" && e["unit_id"] == "antonius")
            .unwrap();
        event.as_object_mut().unwrap().remove(key);
        assert!(
            validate_tier_b_repair_state_data(&bad, &documents).is_err(),
            "unresolved rejects missing {key}"
        );
    }
}

#[test]
fn tier_b_disposition_rejects_wrong_event_and_digest_mismatch() {
    let (ledger, good) = synthetic_tier_b_delivery("partial");
    let event = ledger["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["status"] == "unresolved")
        .unwrap();
    validate_tier_b_disposition(event, event, &good).unwrap();
    for kind in ["event", "digest", "trace", "configuration", "reason"] {
        let mut bad = good.clone();
        let path = event["disposition_evidence"]["path"].as_str().unwrap();
        match kind {
            "event" => bad["documents"][path]["value"]["event_id"] = serde_json::json!("wrong"),
            "digest" => bad["documents"][path]["source_sha256"] = serde_json::json!("b".repeat(64)),
            "trace" => {
                bad["documents"][path]["value"]["trace_evidence"]["sha256"] =
                    serde_json::json!("b".repeat(64))
            }
            "configuration" => {
                bad["documents"][path]["value"]["tested_configuration"]["unit_table"]["sha256"] =
                    serde_json::json!("b".repeat(64))
            }
            _ => bad["documents"][path]["value"]["reason"] = serde_json::json!("different"),
        }
        assert!(
            validate_tier_b_disposition(event, event, &bad).is_err(),
            "disposition rejects {kind}"
        );
    }
}

#[test]
fn antonius_following_chapter_wraps_without_bisecting_glyphs() {
    let ledger = audit_repair_ledger();
    let mut column = GrayImage::from_pixel(826, 2835, Luma([255]));
    for (key, top) in [
        ("antonius_annotated_whole_strip", 249),
        ("antonius_annotated_following_chapter", 1364),
    ] {
        image::imageops::replace(
            &mut column,
            &audit_fingerprint_image(&ledger["source_fingerprints"][key]),
            0,
            top,
        );
    }
    let next = Band {
        top: 1364,
        bottom: 1691,
    };
    assert!(is_known_whole_source_strip(&column, &next));
    assert!(!is_known_whole_source_strip(
        &column,
        &Band {
            top: 1365,
            bottom: 1691
        }
    ));
    let mut changed = column.clone();
    changed.put_pixel(0, 1364, Luma([0]));
    assert!(!is_known_whole_source_strip(&changed, &next));
    let metrics = LineMetrics {
        height_med: 33,
        height_mad: 2,
        width_med: 777,
    };
    let starter = page_and_metrics(500, 826, &[(100, 400, 20, 790)]).0;
    let pages = [
        starter,
        (
            column,
            vec![
                Band {
                    top: 249,
                    bottom: 1356,
                },
                Band {
                    top: 1373,
                    bottom: 1691,
                },
            ],
        ),
    ];
    let geom = TrimGeom {
        dpi: 300,
        tb_w: 1004,
        tb_h: 1394,
        pad_x: 0,
        pad_y: 0,
    };
    let scale = 1.2023952;
    let expected = scale_strip(&pages[1].0, &next, scale).unwrap();
    let canvases = assert_scaled_stream(&pages, &metrics, scale, &geom, 1252, 1394);
    assert_eq!(
        canvases.len(),
        3,
        "the body block and following chapter wrap separately"
    );
    let retained =
        image::imageops::crop_imm(&canvases[2], 0, 0, expected.width(), expected.height())
            .to_image();
    assert_eq!(
        retained, expected,
        "every following-chapter glyph stays intact"
    );
}
