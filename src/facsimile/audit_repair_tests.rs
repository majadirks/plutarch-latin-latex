// Test support for frozen audit identities and original-source preservation.

fn audit_repair_ledger() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "fixtures/inner_tomi_audit_repair_source_trace.json"
    ))
    .unwrap()
}

fn audit_file_bytes(fixture: &serde_json::Value, name: &str) -> Result<Vec<u8>> {
    let original = std::path::Path::new(fixture["original_run"].as_str().unwrap()).join(name);
    let archived = std::path::Path::new(fixture["archive"]["path"].as_str().unwrap())
        .join("runs/step102-final-all-5")
        .join(name);
    std::fs::read(&original)
        .or_else(|_| std::fs::read(&archived))
        .map_err(|e| {
            anyhow::anyhow!(
                "missing historical evidence: neither {} nor {} is readable: {e}",
                original.display(),
                archived.display()
            )
        })
}

fn validate_audit_omission_inventory(fixture: &serde_json::Value) -> Result<()> {
    use sha2::Digest;
    let candidates_bytes = audit_file_bytes(fixture, "candidates.json")?;
    let reviews_bytes = audit_file_bytes(fixture, "reviews.json")?;
    for (bytes, key) in [
        (&candidates_bytes, "candidates_sha256"),
        (&reviews_bytes, "reviews_sha256"),
    ] {
        anyhow::ensure!(
            format!("{:x}", sha2::Sha256::digest(bytes)) == fixture[key].as_str().unwrap(),
            "frozen audit hash mismatch: {key}"
        );
    }
    let candidates: serde_json::Value = serde_json::from_slice(&candidates_bytes)?;
    let reviews: serde_json::Value = serde_json::from_slice(&reviews_bytes)?;
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
    let archive = std::path::Path::new(fixture["archive"]["path"].as_str().unwrap());
    let manifest = std::fs::read(archive.join("manifest.json"))?;
    anyhow::ensure!(
        format!("{:x}", sha2::Sha256::digest(&manifest)) == fixture["archive"]["manifest_sha256"],
        "archive manifest hash differs"
    );
    Ok(())
}

#[test]
fn audit_omission_inventory_matches_frozen_review() {
    validate_audit_omission_inventory(&audit_repair_ledger()).unwrap();
}

#[test]
fn audit_omission_diagnosis_covers_all_source_lines() {
    use sha2::Digest;
    let fixture = audit_repair_ledger();
    let archive = &fixture["diagnostic_archive"];
    let manifest = std::fs::read(
        std::path::Path::new(archive["path"].as_str().unwrap()).join("manifest.json"),
    )
    .unwrap();
    assert_eq!(
        format!("{:x}", sha2::Sha256::digest(&manifest)),
        archive["manifest_sha256"]
    );
    let mut counts = [0, 0];
    for event in fixture["events"].as_array().unwrap() {
        if event["initial_tier"] == "B" {
            assert_eq!(event["status"], "deferred");
            assert!(!event["repair_read_status"].as_str().unwrap().is_empty());
            continue;
        }
        let diagnosis = &event["diagnosis"];
        for key in ["baseline_trace", "reviewed_source_overlay"] {
            let evidence = &diagnosis[key];
            let bytes = std::fs::read(evidence["path"].as_str().unwrap()).unwrap();
            assert_eq!(
                format!("{:x}", sha2::Sha256::digest(&bytes)),
                evidence["sha256"],
                "frozen {key} for {}",
                event["event_id"]
            );
        }
        let lines = diagnosis["reviewed_lines"].as_array().unwrap();
        assert_eq!(
            lines.len() as u64,
            event["missing_line_count"].as_u64().unwrap()
        );
        let mut previous = None;
        for line in lines {
            let r = &line["region"];
            assert!(r[0].as_u64().unwrap() < r[2].as_u64().unwrap());
            assert!(r[1].as_u64().unwrap() < r[3].as_u64().unwrap());
            assert!(previous.is_none_or(|top| top < r[1].as_u64().unwrap()));
            previous = r[1].as_u64();
            assert!(!line["baseline"]["first_loss_stage"]
                .as_str()
                .unwrap()
                .is_empty());
            assert!(!line["landmark"].as_str().unwrap().is_empty());
        }
        assert!(!diagnosis["regression_name"].as_str().unwrap().is_empty());
        counts[0] += 1;
        counts[1] += lines.len();
    }
    assert_eq!(counts, [39, 69]);
}

#[test]
fn audit_omission_post_dedup_covers_all_source_lines() {
    use sha2::Digest;
    let fixture = audit_repair_ledger();
    let mut counts = [0, 0];
    for event in fixture["events"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["initial_tier"] == "A")
    {
        let measured = &event["diagnosis"]["post_dedup"];
        let lines = measured["lines"]
            .as_array()
            .expect("post-dedup source-line evidence");
        assert_eq!(
            lines.len() as u64,
            event["missing_line_count"].as_u64().unwrap()
        );
        let bytes = std::fs::read(measured["trace"]["path"].as_str().unwrap()).unwrap();
        assert_eq!(
            format!("{:x}", sha2::Sha256::digest(bytes)),
            measured["trace"]["sha256"]
        );
        for (line, reviewed) in lines
            .iter()
            .zip(event["diagnosis"]["reviewed_lines"].as_array().unwrap())
        {
            assert_eq!(line["region"], reviewed["region"]);
            assert!(!line["measured"]["first_loss_stage"]
                .as_str()
                .unwrap()
                .is_empty());
        }
        assert!(measured["scale"].as_f64().unwrap() > 0.0);
        if !measured["complete"].as_bool().unwrap() {
            assert!(!measured["proposed_body_bands"]
                .as_array()
                .unwrap()
                .is_empty());
        }
        counts[0] += 1;
        counts[1] += lines.len();
    }
    assert_eq!(counts, [39, 69]);
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
        .filter(|r| r["responsible_step"] == "3.1")
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

struct ReviewedLineContext<'a> {
    traces: &'a [RetainedSourcePageTrace],
    originals: &'a std::collections::BTreeMap<(usize, u32), GrayImage>,
    pages: &'a [PageBands],
    metrics: &'a LineMetrics,
    scale: f32,
    geom: TrimGeom,
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
) {
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
    let (strips, canvas_ranges, _) =
        ranges_for_composed_canvases(pages, metrics, *scale, geom, canvases);
    let mut previous = None;
    let mut seen = std::collections::BTreeSet::new();
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
            "whole reviewed line covered once by retained bands"
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
            "reviewed right glyphs fit output text width"
        );
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
            let crop = preserve_known_body_crop_edge(range.unit.source, p, &raw, detect_latin_column(&raw).unwrap());
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
        let crop = preserve_known_body_crop_edge(trace.source, trace.page, &masked, detect_latin_column(&masked).unwrap());
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
