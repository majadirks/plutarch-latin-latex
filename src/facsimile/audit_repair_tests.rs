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
    use sha2::Digest;
    let logical = evidence_string(reference, "path")?;
    let expected = evidence_digest(reference, "sha256")?;
    let resolved = resolve_evidence_path(root, logical)?;
    let bytes = std::fs::read(&resolved)?;
    let actual = format!("{:x}", sha2::Sha256::digest(&bytes));
    anyhow::ensure!(actual == expected,
        "evidence hash mismatch: logical {logical}, resolved {}, expected {expected}, actual {actual}",
        resolved.display());
    Ok(bytes)
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
        if event["initial_tier"] == "A" || event["promoted_tier"] == "A" {
            add(
                event["passing_evidence"].clone(),
                AuditDocumentKind::Restoration,
            )?;
        }
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
    let mut documents = serde_json::Map::new();
    for (path, (reference, _)) in required_audit_references(ledger)? {
        let bytes = read_verified_evidence_bytes(root, &reference)?;
        documents.insert(
            path,
            serde_json::json!({"source_sha256": reference["sha256"],
            "value": serde_json::from_slice::<serde_json::Value>(&bytes)?}),
        );
    }
    Ok(serde_json::json!({"schema_version": 1, "documents": documents}))
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
                project_evidence_fields(value, &["unit_id", "events", "reviewed_lines"])?
            }
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
        .unwrap_or(std::path::Path::new("."))
        .canonicalize()?
        .join(out.file_name().unwrap());
    anyhow::ensure!(
        absolute != tracked.canonicalize()?,
        "export to a temporary review file, not the tracked fixture"
    );
    let root = std::env::var_os("PLUTARCH_AUDIT_EVIDENCE_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| env!("CARGO_MANIFEST_DIR").into());
    let ledger = audit_repair_ledger();
    let documents = load_fixture_source_documents(&ledger, &root)?;
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

fn archived_audit_documents() -> Result<serde_json::Value> {
    load_fixture_source_documents(
        &audit_repair_ledger(),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    )
}

fn validate_audit_omission_inventory(fixture: &serde_json::Value) -> Result<()> {
    let documents = archived_audit_documents()?;
    let archive = &fixture["archive"];
    read_verified_evidence_bytes(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
        &serde_json::json!({"path":format!("{}/manifest.json",archive["path"].as_str().unwrap()), "sha256":archive["manifest_sha256"]}),
    )?;
    validate_omission_inventory_data(fixture, &documents)
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
    validate_audit_omission_inventory(&audit_repair_ledger()).unwrap();
}

#[test]
fn audit_omission_diagnosis_covers_all_source_lines() {
    let ledger = audit_repair_ledger();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let archive = &ledger["diagnostic_archive"];
    read_verified_evidence_bytes(root, &serde_json::json!({"path":format!("{}/manifest.json", archive["path"].as_str().unwrap()), "sha256":archive["manifest_sha256"]})).unwrap();
    for event in ledger["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["initial_tier"] == "A")
    {
        read_verified_evidence_bytes(root, &event["diagnosis"]["reviewed_source_overlay"]).unwrap();
    }
    validate_omission_diagnosis_data(&audit_repair_ledger(), &archived_audit_documents().unwrap())
        .unwrap();
}

#[test]
fn audit_omission_post_dedup_covers_all_source_lines() {
    validate_post_dedup_data(&audit_repair_ledger(), &archived_audit_documents().unwrap()).unwrap();
}

#[test]
fn audit_omission_source_signatures_match_proposed_repairs() {
    validate_source_signature_data(&audit_repair_ledger(), &archived_audit_documents().unwrap())
        .unwrap();
}

fn validate_tier_a_completion(fixture: &serde_json::Value) -> Result<()> {
    validate_tier_a_completion_data(fixture, &archived_audit_documents()?)
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
    validate_tier_a_completion_data(&ledger, &evidence)
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
        .filter(|e| e["initial_tier"] == "A" || e["promoted_tier"] == "A")
    {
        anyhow::ensure!(
            event["status"] == "restored",
            "unresolved Tier-A event: {}",
            event["event_id"]
        );
        let reference = &event["passing_evidence"];
        let report = portable_evidence_document(documents, reference)?;
        anyhow::ensure!(
            report["unit_id"] == event["unit_id"],
            "passing report unit differs"
        );
        anyhow::ensure!(
            report["events"]
                .as_array()
                .unwrap()
                .contains(&event["event_id"]),
            "event absent from passing evidence"
        );
        let lines: Vec<ReviewedSourceLine> =
            serde_json::from_value(event["diagnosis"]["reviewed_lines"].clone())?;
        anyhow::ensure!(
            lines.len() as u64 == event["missing_line_count"].as_u64().unwrap(),
            "source line count differs"
        );
        for line in &lines {
            anyhow::ensure!(
                report["reviewed_lines"]
                    .as_array()
                    .unwrap()
                    .contains(&serde_json::to_value(line)?),
                "reviewed line absent from passing evidence"
            );
        }
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
    validate_tier_a_completion(&fixture).unwrap();
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
            validate_tier_a_completion(&changed).is_err(),
            "completion rejects {kind}"
        );
    }
}

#[test]
fn audit_omission_inventory_rejects_false_positive_and_duplicate_events() {
    let fixture = audit_repair_ledger();
    validate_audit_omission_inventory(&fixture).unwrap();
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
            validate_audit_omission_inventory(&invalid).is_err(),
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
    for record in SOURCE_BODY_BANDS {
        let mut specifications: Vec<_> = (1000..=2000)
            .step_by(100)
            .map(|top| (top, top + 40, 10, 700))
            .collect();
        specifications.push((record.observed.top, record.observed.bottom, 10, 700));
        let mut col = column(record.dimensions.1, record.dimensions.0, &specifications);
        if record.protected.top < record.observed.top {
            col.put_pixel(15, record.protected.top, Luma([170]));
        }
        if record.protected.bottom > record.observed.bottom {
            col.put_pixel(15, record.protected.bottom - 1, Luma([170]));
        }
        let raw = line_bands(&col, LINE_MIN_INK);
        let kept = drop_furniture(&raw, col.height());
        assert!(
            !kept.contains(&record.observed),
            "furniture filtering drops the reviewed body interval"
        );
        let rescued = retain_known_source_body_bands(
            None,
            record.source,
            record.page,
            record.dimensions,
            &raw,
            &kept,
        );
        assert_eq!(
            rescued.iter().filter(|b| **b == record.protected).count(),
            1
        );
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
        let mut wrong = raw.clone();
        wrong.retain(|b| *b != record.observed);
        assert_eq!(
            retain_known_source_body_bands(
                None,
                record.source,
                record.page,
                record.dimensions,
                &wrong,
                &kept
            ),
            kept
        );
        let pages = vec![(col, rescued)];
        let metrics = line_metrics(&pages);
        let geom = TrimGeom {
            dpi: 300,
            tb_w: record.dimensions.0,
            tb_h: 700,
            pad_x: 0,
            pad_y: 0,
        };
        let canvases = assert_scaled_stream(&pages, &metrics, 1.0, &geom, 700, 700);
        assert!(!canvases.is_empty());
        assert!(strips_in_reading_order(&pages, &metrics)
            .iter()
            .any(
                |s| s.band.top <= record.protected.top && s.band.bottom >= record.protected.bottom
            ));
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

fn assert_tier_a_unit_lines_restored(id: &str) {
    let fixture = audit_repair_ledger();
    let events: Vec<_> = fixture["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| {
            e["unit_id"] == id
                && ["installed", "restored"]
                    .iter()
                    .any(|status| e["status"] == *status)
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
    let originals = lines
        .iter()
        .map(|l| (l.source, l.source_page))
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
    let composition = assert_reviewed_source_lines_survive(&lines, &context, false);
    let dir = std::path::Path::new("archive/inner_tomi_audit_repairs_2026_10_08/restoration")
        .join(fixture["restoration_stage"].as_str().unwrap());
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
    let evidence = serde_json::json!({"unit_id":id,"scale":context.scale,"events":events.iter().map(|e|e["event_id"].clone()).collect::<Vec<_>>(),
        "reviewed_lines":lines,"line_coverage":composition.line_coverage,"canvas_ranges":composition.canvas_ranges,"canvas_hashes":canvas_hashes,"source_pages":traces,"verification":"original pixels, unique retained bands and strips, source order, and returned canvases passed"});
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
    assert_tier_a_unit_lines_restored("romulus");
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
    assert_tier_a_unit_lines_restored("marcius_coriolanus");
}

#[test]
#[ignore = "requires original source PDFs and production-scale composition"]
fn phocion_c3_line_restored() {
    assert_tier_a_unit_lines_restored("phocion");
}

#[test]
#[ignore = "requires original source PDFs and production-scale composition"]
fn brutus_c5_line_restored() {
    assert_tier_a_unit_lines_restored("brutus");
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
    let dir = std::path::Path::new("archive/inner_tomi_audit_repairs_2026_10_08/s2");
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
        assert_tier_a_unit_lines_restored(id);
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
        if whole_line {
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
            let crop = preserve_known_body_crop_edge(
                range.unit.source,
                p,
                &raw,
                detect_latin_column(&raw).unwrap(),
            );
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
    validate_audit_omission_inventory(&fixture).unwrap();
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
