"""Offline inspection of facsimile readers; never repairs source material."""

import argparse
import hashlib
import html
import importlib
import importlib.metadata
import json
import math
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from contextlib import contextmanager


PACKAGES = {
    "numpy": "numpy", "Pillow": "PIL", "opencv-python-headless": "cv2",
    "scikit-learn": "sklearn", "RapidFuzz": "rapidfuzz", "PyMuPDF": "pymupdf",
}

DEVELOPMENT_UNITS = {
    "agesilaus", "alexander", "phocion", "cato_minor", "agis", "romulus",
    "marcius_coriolanus", "aemilius_paulus", "timoleon", "philopoemen",
}
VALIDATION_UNITS = {
    "cicero", "antonius", "aratus", "otho", "brutus",
    "comparatio_demosthenis_et_ciceronis",
}

DETECTOR_TYPES = {
    "core": {"missing_occurrence", "duplicate", "order", "watermark", "head"},
    "prose": {"missing_occurrence", "duplicate", "order", "terminal_loss"},
    "boundaries": {"terminal_loss"},
    "watermark": {"watermark"},
    "head": {"head"},
    "scraps": {"foreign_scraps"},
    "furniture": {"watermark", "head", "foreign_scraps"},
    "layout": {"copied_gap", "rule", "lateral_clipping", "slicing", "blank_body"},
}
DETECTOR_TYPES["all"] = set.union(*DETECTOR_TYPES.values())
RELATIONS = {
    "missing_occurrence": "missing_expected_position", "duplicate": "repeated_occurrence",
    "order": "reversal", "watermark": "source_watermark", "head": "source_head",
    "foreign_scraps": "foreign_column", "copied_gap": "copied_margin",
    "rule": "vertical_rule", "lateral_clipping": "lateral_loss", "slicing": "horizontal_slice",
    "blank_body": "blank_body", "terminal_loss": "terminal_loss",
}


class AuditError(Exception):
    """An actionable configuration or tool failure."""


class CalibrationError(AuditError):
    """An explicitly requested calibration gate failed."""


def json_hash(data):
    return hashlib.sha256(json.dumps(data, ensure_ascii=False, sort_keys=True,
                                     separators=(",", ":"), allow_nan=False).encode()).hexdigest()


def load_benchmark(root, manifest, work, *, include_labels=True):
    units_path, _ = snapshot_input(root, manifest, role="units")
    units = json.loads(units_path.read_text(encoding="utf-8"))
    data, hashes = {}, {}
    for name in ("splits", "labels", "templates"):
        path = Path(work) / "spec" / f"{name}.json"
        hashes[name] = hash_file(path)
        if name == "labels" and not include_labels:
            continue
        data[name] = validate_benchmark(json.loads(path.read_text(encoding="utf-8")),
                                        manifest, units, splits=data.get("splits"))
    return units, data, hashes


def check_scope(scope, units, splits, *, freeze=None, metadata_only=False, annotation_only=False):
    """Return a bounded unit selection; detector requests cannot cross split boundaries."""
    if scope not in ("development", "validation", "all"):
        raise AuditError("Unknown processing scope")
    allowed = set(splits["development_units"] if scope == "development" else
                  splits["validation_units"] if scope == "validation" else
                  splits["development_units"] + splits["validation_units"])
    requested = set(units) if units is not None else allowed
    if not requested or requested - allowed:
        raise AuditError(f"Unit request expands {scope} scope: {sorted(requested - allowed)}")
    if not (metadata_only or annotation_only) and requested & set(splits["validation_units"]) and not freeze:
        raise AuditError("Validation reader OCR/alignment/detection requires a complete frozen development configuration. Metadata inventory and annotation-only gold curation remain available.")
    return sorted(requested)


def check_freeze(path, settings, manifest, hashes, work):
    freeze_path = Path(path).resolve()
    freeze = json.loads(freeze_path.read_text(encoding="utf-8"))
    if (freeze.get("schema_version") != 1 or freeze.get("baseline_id") != manifest["baseline_id"]
            or freeze.get("complete_final_development") is not True
            or freeze.get("settings_sha256") != json_hash(settings)
            or freeze.get("splits_sha256") != hashes["splits"]
            or freeze.get("templates_sha256") != hashes["templates"]
            or freeze.get("labels_sha256") != hashes["labels"]
            or freeze.get("script_sha256") != hash_file(Path(__file__))
            or freeze.get("input_hashes") != {r["path"]: r["sha256"] for r in manifest["inputs"]}):
        raise AuditError("Freeze is incomplete or differs from the baseline, settings or benchmark identities")
    origin = Path(freeze["development_run_dir"]).resolve()
    if not contained(origin, (Path(work).resolve() / "runs")):
        raise AuditError("Freeze development run must be beneath this work directory's runs/")
    for name, key in (("run-manifest.json", "run_manifest_sha256"),
                      ("candidates.json", "candidates_sha256"),
                      ("evaluation-development.json", "evaluation_sha256")):
        if hash_file(origin / name) != freeze[key]:
            raise AuditError(f"Freeze development evidence changed: {name}")
    return freeze


def overlap(first, second):
    a, b = first["box"], second["box"]
    intersection = max(0, min(a[2], b[2]) - max(a[0], b[0])) * max(0, min(a[3], b[3]) - max(a[1], b[1]))
    area_a, area_b = (a[2] - a[0]) * (a[3] - a[1]), (b[2] - b[0]) * (b[3] - b[1])
    return {"iou": intersection / (area_a + area_b - intersection),
            "gold_coverage": intersection / area_b, "candidate_coverage": intersection / area_a}


def same_page(first, second):
    return all(first.get(key) == second.get(key) for key in
               ("role", "input_sha256", "page", "tomus", "source_index"))


def region_matches(candidate, gold, mode="tiny", rules=None):
    if not same_page(candidate, gold) or candidate.get("box") is None or gold.get("box") is None:
        return False
    scores = overlap(candidate, gold)
    c, g = candidate["box"], gold["box"]
    if mode == "source_correspondence":
        return scores["candidate_coverage"] >= 0.5 and scores["gold_coverage"] > 0
    if mode in ("missing_source", "seam"):
        pitch = gold.get("line_height")
        if not pitch or pitch <= 0:
            return False
        extra_height = max(0, g[1] - c[1]) + max(0, c[3] - g[3])
        return (scores["gold_coverage"] >= (0.8 if mode == "missing_source" else 0.5)
                and extra_height <= pitch + 1e-9
                and c[0] >= g[0] - 0.02 and c[2] <= g[2] + 0.02)
    rules = rules or {"tiny_event": "iou", "minimum_iou": 0.1}
    if rules["tiny_event"] == "iou":
        return scores["iou"] >= rules["minimum_iou"]
    if rules["tiny_event"] == "center_distance":
        pitch = gold.get("line_height")
        if not pitch:
            return False
        distance = math.hypot((c[0] + c[2] - g[0] - g[2]) / 2,
                              (c[1] + c[3] - g[1] - g[3]) / 2)
        return distance <= pitch and (c[2] - c[0]) * (c[3] - c[1]) <= 10 * (g[2] - g[0]) * (g[3] - g[1])
    raise AuditError("Tiny-event localization rule must be chosen before scoring")


def localized_match(candidate, case, rules):
    """Match types, expected source positions and output regions, never Latin anchors."""
    if candidate["type"] != case["defect_type"] or candidate["unit_id"] != case["unit_id"]:
        return False
    relation = "complete_endpoint" if case["label"] == "complete_endpoint" else RELATIONS[case["defect_type"]]
    if candidate["relation"] != relation:
        return False
    output = candidate["output_regions"]
    sources = candidate["source_regions"]
    if case["label"] == "negative":
        return any(region_matches(c, g, rules=rules) for c in output for g in case["output_regions"])
    if case["defect_type"] == "missing_occurrence":
        return (all(any(region_matches(c, g, "missing_source") for c in sources) for g in case["source_regions"])
                and all(any(region_matches(c, g, "seam") for c in output) for g in case["output_regions"])
                and candidate.get("unique_loss") is True)
    if case["defect_type"] == "duplicate":
        contributions = candidate.get("contributions", [])
        if len(contributions) != 2 or len(case.get("contributions", [])) != 2:
            return False
        if candidate.get("incomplete_second_opening", False) != case.get("incomplete_second_opening", False):
            return False
        ordered = sorted(contributions, key=lambda c: (c["start_page"], c["start_box"][1]))
        if (ordered[0]["end_page"], ordered[0]["end_box"][3]) > (ordered[1]["start_page"], ordered[1]["start_box"][1]):
            return False
        for c, g in zip(ordered, case["contributions"]):
            for edge in ("start", "end"):
                if c[f"{edge}_page"] != g[f"{edge}_page"]:
                    return False
                cb, gb = c[f"{edge}_box"], g[f"{edge}_box"]
                if max(abs(cb[1] - gb[1]), abs(cb[3] - gb[3])) > g.get("line_height", 0):
                    return False
                if max(abs(cb[0] - gb[0]), abs(cb[2] - gb[2])) > 0.02:
                    return False
        return all(any(region_matches(c, g, "source_correspondence") for c in sources) for g in case["source_regions"])
    return (all(any(region_matches(c, g, rules=rules) for c in output) for g in case["output_regions"])
            and all(any(region_matches(c, g, "source_correspondence") for c in sources) for g in case["source_regions"]))


def validate_candidates(data, manifest, units, types, selected, body_pages, work):
    if data.get("schema_version") != 1 or not isinstance(data.get("candidates"), list):
        raise AuditError("Expected schema_version: 1 and a candidates list")
    identifiers = set()
    unit_bounds = {}
    for index, unit in enumerate(units):
        following = units[index + 1] if index + 1 < len(units) else None
        end = following["start_page"] if following and following["source"] == unit["source"] else (637 if unit["source"] == 0 else 708)
        unit_bounds[unit["id"]] = (unit["source"], unit["start_page"], end)
    for candidate in data["candidates"]:
        identifier = candidate["event_id"]
        if not isinstance(identifier, str) or not re.fullmatch(r"[A-Za-z0-9_:-]+", identifier) or identifier in identifiers:
            raise AuditError("Candidate event IDs must be unique and stable")
        identifiers.add(identifier)
        if candidate["unit_id"] not in selected or candidate["type"] not in types:
            raise AuditError("Candidate expands requested unit/detector scope")
        if candidate["confidence"] not in ("high", "low", "unknown") or candidate["status"] not in ("candidate", "inconclusive"):
            raise AuditError("Invalid candidate confidence/status")
        relations = {RELATIONS[candidate["type"]]}
        if candidate["type"] == "terminal_loss":
            relations.add("complete_endpoint")
        if candidate["relation"] not in relations:
            raise AuditError("Invalid candidate relation")
        if not candidate.get("confidence_reasons") or not candidate.get("evidence_provenance"):
            raise AuditError("Candidates need evidence provenance and confidence reasons")
        for region in candidate["source_regions"] + candidate["output_regions"]:
            validate_box(region["box"])
            role = region["role"]
            if role not in ("source_pdf", "reader_pdf"):
                raise AuditError("Candidate regions must use captured PDFs")
            _, record = snapshot_input(Path("."), manifest, role=role, tomus=region.get("tomus"), source_index=region.get("source_index"))
            if region["input_sha256"] != record["sha256"] or region["page"] < 1:
                raise AuditError("Candidate input identity/page mismatch")
            limit = manifest["expected_counts"]["physical_by_tomus"][region["tomus"] - 1] if role == "reader_pdf" else (637 if region["source_index"] == 0 else 710)
            if region["page"] > limit:
                raise AuditError("Candidate page outside captured input")
            if role == "reader_pdf":
                owners = {r["unit_id"] for r in body_pages if r["tomus"] == region["tomus"] and r["page"] == region["page"]}
                if owners != {candidate["unit_id"]}:
                    raise AuditError("Candidate output page is not owned by its selected unit; frontmatter/separators cannot be scored as body")
            else:
                source_index, start, end = unit_bounds[candidate["unit_id"]]
                if region["source_index"] != source_index or not start <= region["page"] <= end:
                    raise AuditError("Candidate source region is outside its unit's inclusive source partition")
        if not candidate["output_regions"]:
            raise AuditError("Candidates require localized output evidence")
        for crop in candidate.get("crops", []):
            path = (Path(work).resolve() / relative_path(crop["path"])).resolve()
            if not any(contained(path, Path(work).resolve() / name) for name in ("cache", "runs")) or hash_file(path) != crop["sha256"]:
                raise AuditError("Candidate crop identity/path mismatch")
        if not candidate.get("crops"):
            raise AuditError("Candidates require existing identity-checked evidence crops")
        for contribution in candidate.get("contributions", []):
            validate_box(contribution["start_box"])
            validate_box(contribution["end_box"])
    return data["candidates"]


def evaluate_cases(candidates, labels, selected_units, types, requested_pages):
    results = []
    for case in labels["cases"]:
        pages = {f"{r['tomus']}:{r['page']}" for r in case["output_regions"]}
        if case["unit_id"] not in selected_units or case["defect_type"] not in types or not pages <= requested_pages:
            status, matches = "not_requested", []
        elif case["annotation_status"] != "reviewed":
            status, matches = "pending_annotation", []
        else:
            matches = [c["event_id"] for c in candidates if c["status"] == "candidate"
                       and localized_match(c, case, labels["localization_rule"])]
            if case["label"] == "negative":
                high = {c["event_id"] for c in candidates if c["confidence"] == "high"}
                status = "negative_violation" if high & set(matches) else "negative_no_high_candidate"
            else:
                status = "hit" if matches else "miss"
        results.append({"case_id": case["case_id"], "unit_id": case["unit_id"],
                        "type": case["defect_type"], "label": case["label"], "status": status,
                        "event_ids": matches})
    counts = {status: sum(r["status"] == status for r in results) for status in
              ("hit", "miss", "not_requested", "pending_annotation", "negative_violation", "negative_no_high_candidate")}
    matched = {event for r in results for event in r["event_ids"]}
    return {"schema_version": 1, "baseline_id": labels["baseline_id"], "counts": counts,
            "cases": results, "unreviewed_event_ids": sorted(c["event_id"] for c in candidates if c["event_id"] not in matched),
            "limitations": ["Unreviewed candidates are not negatives.",
                            "Pending annotations cannot establish hits or clean negatives.",
                            "Manual candidate verification is not detector performance."]}


def freeze_development(run, record, evaluation, candidates, benchmark, hashes, manifest, work, repo):
    dev = set(benchmark["splits"]["development_units"])
    pending = [c["case_id"] for c in benchmark["labels"]["cases"]
               if c["unit_id"] in dev and c["annotation_status"] != "reviewed"]
    if (record["scope"] != "development" or set(record["units"]) != dev or record["detectors"] != "all"
            or record.get("final_development") is not True or record.get("complete") is not True
            or record.get("producer") != "detectors" or record.get("errors")
            or pending or evaluation["counts"]["pending_annotation"] or evaluation["counts"]["negative_violation"]):
        raise CalibrationError("Freeze requires a complete final detector run over all development units/types, reviewed annotations and negatives, and no processing errors or negative regressions")
    requested = set(record.get("requested_body_pages", []))
    completed = set(record.get("completed_body_pages", []))
    if not requested or requested != completed:
        raise CalibrationError("Development coverage is incomplete")
    body_pages, _ = cached_inventory(Path(work) / "snapshots" / manifest["baseline_id"], manifest, work, repo)
    actual = {f"{r['tomus']}:{r['page']}" for r in body_pages if r["unit_id"] in dev}
    if requested != actual:
        raise CalibrationError("Final development coverage must include every actual development body page")
    for kind in ("missing_occurrence", "duplicate", "watermark", "head"):
        high = {c["event_id"] for c in candidates if c["type"] == kind and c["confidence"] == "high"}
        if not any(r["status"] == "hit" and r["type"] == kind and high & set(r["event_ids"]) for r in evaluation["cases"]):
            raise CalibrationError(f"Freeze requires a strong localized development positive for {kind}")
    if not record["settings"].get("thresholds") or not record.get("source_index_identity") or not record.get("versions"):
        raise CalibrationError("Freeze requires recorded thresholds, source index/profile identities and tool/model versions")
    freeze = {"schema_version": 1, "baseline_id": manifest["baseline_id"], "complete_final_development": True,
              "development_run_dir": str(run), "settings": record["settings"], "settings_sha256": json_hash(record["settings"]),
              "splits_sha256": hashes["splits"], "templates_sha256": hashes["templates"], "labels_sha256": hashes["labels"],
              "source_index_identity": record["source_index_identity"], "versions": record["versions"],
              "script_sha256": hash_file(Path(__file__)),
              "input_hashes": {r["path"]: r["sha256"] for r in manifest["inputs"]},
              "run_manifest_sha256": hash_file(run / "run-manifest.json"),
              "candidates_sha256": hash_file(run / "candidates.json"),
              "evaluation_sha256": hash_file(run / "evaluation-development.json"),
              "known_misses": [r["case_id"] for r in evaluation["cases"] if r["status"] == "miss"]}
    if (run / "freeze.json").exists():
        raise CalibrationError("Freeze already exists; preserve it and use a new run for revisions")
    write_json(run / "freeze.json", freeze, work, repo)


def evaluate_run(args):
    repo, work = Path(args.repo_root).resolve(), Path(args.work_dir).resolve()
    run = safe_write_path(args.run_dir, work, repo)
    record_path, candidates_path = run / "run-manifest.json", run / "candidates.json"
    record_hash, candidates_hash = hash_file(record_path), hash_file(candidates_path)
    record = json.loads(record_path.read_text(encoding="utf-8"))
    root, manifest = resolve_baseline(work, record["baseline_id"])
    units, benchmark, hashes = load_benchmark(root, manifest, work)
    if (record.get("schema_version") != 1 or record.get("spec_hashes") != hashes
            or record.get("settings_sha256") != json_hash(record["settings"])
            or record.get("candidates_sha256") != candidates_hash
            or record.get("input_hashes") != {r["path"]: r["sha256"] for r in manifest["inputs"]}):
        raise AuditError("Run manifest differs from its settings, inputs, specs or candidate file; create a new run for changed configuration")
    freeze = check_freeze(record["freeze_path"], record["settings"], manifest, hashes, work) if record.get("freeze_path") else None
    selected = check_scope(record["scope"], record["units"], benchmark["splits"], freeze=freeze)
    if freeze and (record.get("versions") != freeze["versions"] or record.get("source_index_identity") != freeze["source_index_identity"]):
        raise AuditError("Validation tool/model or source-index identity differs from the frozen development run")
    if args.split != "all" and args.split != record["scope"]:
        raise AuditError("Evaluation split must match the requested run scope")
    if args.split == "all" and record["scope"] != "all":
        raise AuditError("An all-case evaluation requires an all-unit frozen run")
    types = DETECTOR_TYPES[record["detectors"]]
    if record["settings"].get("localization_rule") != benchmark["labels"]["localization_rule"]:
        raise AuditError("Run localization settings must match the predeclared benchmark rule")
    body_pages, _ = cached_inventory(root, manifest, work, repo)
    allowed_pages = {f"{r['tomus']}:{r['page']}" for r in body_pages if r["unit_id"] in selected}
    requested, completed = record.get("requested_body_pages", []), record.get("completed_body_pages", [])
    if (not requested or len(set(requested)) != len(requested) or len(set(completed)) != len(completed)
            or not set(completed) <= set(requested) <= allowed_pages):
        raise AuditError("Coverage must name distinct actual body pages in the requested unit scope")
    candidates = validate_candidates(json.loads(candidates_path.read_text(encoding="utf-8")), manifest, units, types, selected, body_pages, work)
    if any(f"{r['tomus']}:{r['page']}" not in completed for c in candidates for r in c["output_regions"]):
        raise AuditError("Candidate output regions must belong to completed coverage")
    evaluation = evaluate_cases(candidates, benchmark["labels"], selected, types, set(requested))
    evaluation.update({"scope": record["scope"], "units": selected, "detectors": record["detectors"],
                       "producer": record.get("producer", "unknown"), "spec_hashes": hashes,
                       "run_manifest_sha256": record_hash, "candidates_sha256": candidates_hash,
                       "processing_complete": record.get("complete") is True and not record.get("errors") and set(requested) == set(completed),
                       "requested_coverage": record.get("requested_body_pages", []),
                       "completed_coverage": record.get("completed_body_pages", [])})
    evaluation["candidate_burden"] = {
        "grouped_core_events": sum(c["type"] in DETECTOR_TYPES["core"] for c in candidates),
        "processed_body_pages": len(completed),
        "events_per_100_pages": 100 * sum(c["type"] in DETECTOR_TYPES["core"] for c in candidates) / len(completed) if completed else None,
        "sample_kind": record.get("producer", "unknown"),
    }
    evaluation["reviewed_precision"] = {"value": None, "reason": "No separate candidate adjudication sample supplied; benchmark hits are not a representative precision sample."}
    evaluation["panel_false_positive_rate"] = {"value": None, "fully_reviewed_pages": 0,
                                               "reason": "Panel page review is pending; no unreviewed page is counted as negative."}
    with run_directory(run, work, repo):
        destination = run / f"evaluation-{args.split}.json"
        if (run / "freeze.json").exists():
            if not destination.exists() or json.loads(destination.read_text(encoding="utf-8")) != evaluation:
                raise AuditError("Frozen evaluation cannot be changed; use a new run")
        else:
            write_json(destination, evaluation, work, repo)
        resolve_baseline(work, record["baseline_id"])
        if hash_file(record_path) != record_hash or hash_file(candidates_path) != candidates_hash:
            raise AuditError("Run inputs changed during evaluation")
        if any(hash_file(work / "spec" / f"{name}.json") != digest for name, digest in hashes.items()):
            raise AuditError("Benchmark specs changed during evaluation")
        if args.freeze:
            freeze_development(run, record, evaluation, candidates, benchmark, hashes, manifest, work, repo)
    progress(json.dumps(evaluation["counts"], indent=2))
    progress(f"Evaluation: {destination}")
    if args.enforce_gates and (evaluation["counts"]["miss"] or evaluation["counts"]["pending_annotation"] or evaluation["counts"]["negative_violation"] or not evaluation["processing_complete"] or set(requested) != allowed_pages or record.get("producer") != "detectors"):
        raise CalibrationError("Requested evaluation gates failed; inspect misses, pending annotations, negative regressions and coverage")
    return 0 if evaluation["processing_complete"] else 3


def request_run(args):
    repo, work = Path(args.repo_root).resolve(), Path(args.work_dir).resolve()
    root, manifest = resolve_baseline(work, args.baseline)
    _, benchmark, hashes = load_benchmark(root, manifest, work, include_labels=False)
    settings = json.loads(Path(args.config).read_text(encoding="utf-8")) if args.config else detector_settings()
    script_hash = hash_file(Path(__file__))
    freeze = check_freeze(args.freeze, settings, manifest, hashes, work) if args.freeze else None
    selected = check_scope(args.scope, args.units, benchmark["splits"], freeze=freeze)
    if args.tomus:
        tomus_units = set()
        for tomus in args.tomus:
            tex, _ = snapshot_input(root, manifest, role="reader_tex", tomus=tomus)
            for asset in parse_tex_refs(tex):
                match = re.fullmatch(r"facsimile_(.+)_\d+\.png", Path(asset).name)
                if match:
                    tomus_units.add(match[1])
        selected = [u for u in selected if u in tomus_units]
        if not selected:
            raise AuditError("Tomus subset contains no units in the requested scope")
    previous_path = safe_write_path(Path(args.run_dir) / "run-manifest.json", work, repo)
    if previous_path.exists():
        previous = json.loads(previous_path.read_text(encoding="utf-8"))
        if not args.resume:
            raise AuditError("Run already exists; choose a new --run-dir or explicitly --resume")
        identity = {"baseline_id": args.baseline, "scope": args.scope, "detectors": args.detectors,
                    "units": selected, "settings_sha256": json_hash(settings), "spec_hashes": hashes,
                    "script_sha256": script_hash}
        if any(previous.get(key) != value for key, value in identity.items()):
            raise AuditError("Resume differs from the saved run identity; choose a new --run-dir")
    progress(f"Authorized {args.scope} scope: {len(selected)} units, {args.detectors} detectors")
    reader_map = cached_reader_map(root, manifest, work, repo)
    source_regions = cached_source_regions(root, manifest, work, repo)
    index = cached_source_index(root, manifest, work, repo)
    selected_set = set(selected)
    selected_body = [r for r in reader_map["body_pages"] if r["unit_id"] in selected_set]
    selected_source = [u for u in source_regions["units"] if u["unit_id"] in selected_set]
    boundary_only = args.detectors == "boundaries"
    if boundary_only:
        edge_rows = []
        for unit_id in selected:
            rows = sorted((r for r in selected_body if r["unit_id"] == unit_id), key=lambda r: r["page"])
            edge_rows.extend(r for i, r in enumerate(rows) if i < 2 or i >= len(rows) - 2)
        selected_body = edge_rows

    tesseract = args.tesseract or shutil.which("tesseract")
    if not tesseract:
        raise AuditError("Tesseract must be available for OCR; supply --tesseract PATH.")
    reader_ocr = {"requested_pages": len(selected_body), "completed_pages": 0, "unreadable": []}
    reader_streams, reader_tomus = {}, {}
    for row in sorted(selected_body, key=lambda r: (r["tomus"], r["page"])):
        pdf, record = snapshot_input(root, manifest, role="reader_pdf", tomus=row["tomus"])
        observation = ocr_page_region(pdf, record, row["page"], OUTPUT_OCR_DPI, OUTPUT_PSM, args.lang,
                                      work, repo, box=row["body_box"], tesseract=tesseract, pdftoppm=args.pdftoppm)
        reader_ocr["completed_pages"] += 1
        if not observation["readable"]:
            reader_ocr["unreadable"].append({"tomus": row["tomus"], "page": row["page"], "unit_id": row["unit_id"]})
        reader_tomus[row["unit_id"]] = row["tomus"]
        stream = reader_streams.setdefault(row["unit_id"], [])
        by_line = {}
        for word in observation["words"]:
            by_line.setdefault(tuple(word["line_id"]), []).append(word)
        for line_words in sorted(by_line.values(), key=lambda ws: min(w["box"][1] for w in ws)):
            for word in sorted(line_words, key=lambda w: w["box"][0]):
                stream.append({"raw": word["text"], "strict": strict_view(word["text"]),
                               "tolerant": tolerant_view(word["text"]), "page": row["page"],
                               "tomus": row["tomus"], "box": word["box"], "conf": word["conf"]})
    source_ocr = {"requested_pages": 0, "completed_pages": 0, "unreadable": []}
    for unit in selected_source:
        if boundary_only or args.detectors in ("watermark", "head", "scraps"):
            continue  # Boundary evidence uses regional raster observations below.
        pdf, record = snapshot_input(root, manifest, role="source_pdf", source_index=unit["source"])
        for page in unit["pages"]:
            if page.get("skipped"):
                continue
            source_ocr["requested_pages"] += 1
            observation = ocr_page_region(pdf, record, page["page"], SOURCE_OCR_DPI, SOURCE_PSM, args.lang,
                                          work, repo, box=page["region"], tesseract=tesseract, pdftoppm=args.pdftoppm)
            source_ocr["completed_pages"] += 1
            if not observation["readable"]:
                source_ocr["unreadable"].append({"source_index": unit["source"], "page": page["page"], "unit_id": unit["unit_id"]})

    loaded = load_source_index(index)
    before = {view: view_identity(*vectorizer_vocab_idf(loaded[view])) for view in ("strict", "tolerant")}
    reader_docs = []
    for unit_id, stream in reader_streams.items():
        strict = [token["strict"] for token in stream]
        for start, end in passage_windows(len(strict), LOCAL_WINDOW, LOCAL_STRIDE):
            reader_docs.append(" ".join(strict[start:end]))
    if reader_docs:
        loaded["strict"].transform(reader_docs)  # Transform-only; must never refit the index.
    after = {view: view_identity(*vectorizer_vocab_idf(loaded[view])) for view in ("strict", "tolerant")}
    stable = before == after and all(before[view] == index["identity"][view] for view in ("strict", "tolerant"))
    if not stable:
        raise AuditError("Source index changed during reader-window transformation; the index must not refit.")
    progress(f"Source index stable across {len(reader_docs)} reader windows (transform-only)")

    requested_types = DETECTOR_TYPES[args.detectors]
    pending_types = sorted(requested_types - IMPLEMENTED_DETECTORS)
    active = requested_types & IMPLEMENTED_DETECTORS
    templates = make_templates(benchmark["templates"], source_regions, root, manifest, work, repo,
                               settings, args.pdftoppm) if "watermark" in active else []
    vectorizer = loaded["strict"]
    index_by_unit = {unit["unit_id"]: unit for unit in index["units"]}
    region_by_unit = {unit["unit_id"]: unit for unit in source_regions["units"]}
    requested_pages = sorted({f"{r['tomus']}:{r['page']}" for r in selected_body})
    candidates, alignment_debug, errors, boundaries = [], {}, [], {}
    with run_directory(args.run_dir, work, repo) as run:
        exemptions = [{"tomus": tomus["tomus"], "page": page["page"],
                       "role": "frontmatter_notice" if asset.startswith("front_qr_") else "frontmatter_image",
                       "asset": asset, "reason": "mapped frontmatter asset; body image detectors do not search it"}
                      for tomus in reader_map["tomi"] for page in tomus["pages"] if page["kind"] == "frontmatter"
                      for asset in page["image_assets"]]
        write_json(run / "image-exemptions.json", {"schema_version": 1, "baseline_id": args.baseline,
                   "search_role": "mapped body image occurrences", "exemptions": exemptions}, work, repo)
        for unit_id in selected:
            index_unit = index_by_unit[unit_id]
            source_unit = dict(index_unit, boxes=source_token_boxes(index_unit, region_by_unit[unit_id]))
            source_tokens = [dict(token, box=box) for token, box in zip(source_unit["tokens"], source_unit["boxes"])]
            blocks = (find_source_duplicates(source_unit, vectorizer, settings)
                      if active & {"duplicate", "order", "missing_occurrence"} else [])
            reader = reader_streams.get(unit_id, [])
            path = []
            coverage = {"runs": [], "duplicates": [], "orders": []}
            omissions = []
            if reader and active and not boundary_only:
                anchors = find_reader_occurrences(reader, source_unit, vectorizer, settings)
                path = chain_matches(anchors, blocks, settings)
                coverage = assign_occurrence_coverage(path, blocks, settings)
                if "missing_occurrence" in active:
                    omissions = find_missing_occurrences(reader, source_unit, anchors, path, settings)
            alignment_debug[unit_id] = {
                "source_duplicate_blocks": blocks,
                "reader_runs": coverage["runs"],
                "reader_token_count": len(reader),
                "repeated_visits": len(coverage["duplicates"]),
                "reversals": len(coverage["orders"]),
                "missing_occurrences": [{k: om[k] for k in
                                         ("gap", "missing_line_count", "unique_loss", "confidence",
                                          "ordered", "left_informative", "right_informative",
                                          "displaced")} for om in omissions]}
            source_pdf, source_record = snapshot_input(root, manifest, role="source_pdf", source_index=index_unit["source_index"])
            if "terminal_loss" in active:
                boundaries[unit_id] = compare_boundaries(
                    reader, source_unit, region_by_unit[unit_id], root, manifest, run, work, repo,
                    settings, args.lang, tesseract, args.pdftoppm, reader_tomus.get(unit_id))
                candidates.extend(boundaries[unit_id].pop("candidates"))
            if "watermark" in active:
                rows = [row for row in selected_body if row["unit_id"] == unit_id]
                candidates.extend(find_watermarks(rows, reader, source_unit, path, templates,
                    root, manifest, run, work, repo, settings, args.pdftoppm, args.workers))
            if "head" in active:
                rows = [row for row in selected_body if row["unit_id"] == unit_id]
                candidates.extend(find_head_candidates(rows, reader, source_unit, path, root, manifest,
                    run, work, repo, settings, args.lang, tesseract, args.pdftoppm))
            if "foreign_scraps" in active:
                rows = [row for row in selected_body if row["unit_id"] == unit_id]
                candidates.extend(find_foreign_scraps(rows, reader, source_unit, path, root, manifest,
                    run, work, repo, settings, args.pdftoppm))
            if reader:
                reader_record = snapshot_input(root, manifest, role="reader_pdf", tomus=reader_tomus[unit_id])[1]
                heights = sorted(token["box"][3] - token["box"][1] for token in reader)
                line_height = heights[len(heights) // 2]
                if "duplicate" in active:
                    for pair in coverage["duplicates"]:
                        candidates.append(build_duplicate_candidate(
                            unit_id, pair, reader, source_tokens, blocks, reader_record, source_record,
                            index_unit["source_index"], reader_tomus[unit_id], line_height, root,
                            manifest, run, work, repo, settings))
                if "order" in active:
                    for order in coverage["orders"]:
                        candidates.append(build_order_candidate(
                            unit_id, order, reader, source_tokens, index_unit["source_index"], reader_record,
                            source_record, reader_tomus[unit_id], root, manifest, run, work, repo, settings))
                if "missing_occurrence" in active:
                    for omission in omissions:
                        candidates.append(build_missing_candidate(
                            unit_id, omission, reader, source_tokens, reader_record, source_record,
                            index_unit["source_index"], reader_tomus[unit_id], line_height, root,
                            manifest, run, work, repo, settings))
            progress(f"Detectors on {unit_id}: {len(blocks)} source-duplicate blocks, "
                     f"{len(coverage['duplicates'])} reader repeats, {len(coverage['orders'])} reversals, "
                     f"{len(omissions)} omissions")
        completed_pages = requested_pages if not reader_ocr["unreadable"] else [
            page for page in requested_pages if page not in
            {f"{u['tomus']}:{u['page']}" for u in reader_ocr["unreadable"]}]
        complete = not pending_types and not errors and set(completed_pages) == set(requested_pages)
        versions = {name: importlib.metadata.version(name) for name in PACKAGES}
        versions.update({"python": sys.version.split()[0], "tesseract": tesseract_version(tesseract),
                         "pdftoppm": run_external([args.pdftoppm or shutil.which("pdftoppm"), "-v"]).splitlines()[0]})
        resolve_baseline(work, args.baseline)
        candidates_payload = {"schema_version": 1, "candidates": candidates}
        write_json(run / "candidates.json", candidates_payload, work, repo)
        write_json(run / "alignment_debug.json", {"schema_version": 1, "baseline_id": args.baseline,
                   "units": alignment_debug}, work, repo)
        write_json(run / "boundaries.json", {"schema_version": 1, "baseline_id": args.baseline,
                   "scope": args.scope, "units": boundaries,
                   "interior_completeness": "not established by boundary correspondence"}, work, repo)
        if boundaries:
            write_boundary_review(boundaries, run, work, repo)
        record = {
            "schema_version": 1, "baseline_id": args.baseline, "scope": args.scope,
            "detectors": args.detectors, "units": selected, "tomus": args.tomus,
            "settings": settings, "settings_sha256": json_hash(settings), "spec_hashes": hashes,
            "script_sha256": script_hash,
            "input_hashes": {r["path"]: r["sha256"] for r in manifest["inputs"]},
            "candidates_sha256": hash_file(run / "candidates.json"),
            "freeze_path": str(Path(args.freeze).resolve()) if args.freeze else None,
            "requested_body_pages": requested_pages, "completed_body_pages": completed_pages,
            "implemented_detectors": sorted(active), "pending_detectors": pending_types,
            "producer": "detectors", "final_development": args.scope == "development" and args.detectors == "all",
            "boundary_only": boundary_only, "boundary_units_completed": sorted(boundaries),
            "image_exemptions": exemptions, "workers": args.workers,
            "complete": complete, "errors": errors, "versions": versions,
            "source_index_identity": index["identity"],
            "ocr": {"lang": args.lang, "reader": reader_ocr, "source": source_ocr,
                    "tesseract_version": versions["tesseract"]},
            "reader_inventory": {"corpus_body_pages": len(reader_map["body_pages"]),
                                 "corpus_units": len(reader_map["units"]),
                                 "selected_body_pages": len(selected_body)},
        }
        write_json(run / "run-manifest.json", record, work, repo)
    progress(f"Wrote {len(candidates)} candidates into {run}")
    if pending_types:
        progress(f"Detectors not yet implemented for requested scope: {', '.join(pending_types)}")
    return 0 if complete else 3


def cached_inventory(root, manifest, work, repo):
    metadata = {"inputs": {r["path"]: r["sha256"] for r in manifest["inputs"] if r["role"] in ("reader_pdf", "reader_tex")},
                "adapter": "displayed-mediabox-occurrences-v1"}
    path = Path(work) / "cache" / "inventory" / f"{json_hash(metadata)}.json"
    cached = reusable_entry(path, metadata, work, repo)
    if cached:
        return cached["body_pages"], cached["geometry"]
    with run_directory(path.parent, work, repo):
        rows, geometry = benchmark_inventory(root, manifest)
        write_json(path, {"schema_version": 1, "complete": True, "metadata": metadata,
                          "data": {"body_pages": rows, "geometry": geometry}}, work, repo)
    return rows, geometry


def cached_reader_map(root, manifest, work, repo):
    """Build or reuse the full reader page/folio/unit map across all eight tomi."""
    metadata = {"inputs": {r["path"]: r["sha256"] for r in manifest["inputs"]
                           if r["role"] in ("reader_pdf", "reader_tex")},
                "adapter": "reader-map-folios-v1"}
    path = Path(work) / "cache" / "reader_map" / f"{json_hash(metadata)}.json"
    cached = reusable_entry(path, metadata, work, repo)
    if cached:
        return cached
    with run_directory(path.parent, work, repo):
        data = map_reader_pages(root, manifest)
        write_json(path, {"schema_version": 1, "complete": True, "metadata": metadata,
                          "data": data}, work, repo)
    return data


def cached_source_regions(root, manifest, work, repo):
    """Build or reuse expected-Latin source regions for every unit across the corpus."""
    metadata = {"inputs": {r["path"]: r["sha256"] for r in manifest["inputs"]
                           if r["role"] in ("source_pdf", "units", "duplicate_policy")},
                "adapter": "source-latin-regions-v1",
                "latin_column": [SOURCE_LATIN_X0, SOURCE_LATIN_X1]}
    path = Path(work) / "cache" / "source_regions" / f"{json_hash(metadata)}.json"
    cached = reusable_entry(path, metadata, work, repo)
    if cached:
        return cached
    units_path, _ = snapshot_input(root, manifest, role="units")
    units = json.loads(units_path.read_text(encoding="utf-8"))
    exclusions = read_approved_exclusions(root, manifest)
    partitions = partition_source_units(units, exclusions)
    with run_directory(path.parent, work, repo):
        data = extract_source_regions(root, manifest, partitions)
        write_json(path, {"schema_version": 1, "complete": True, "metadata": metadata,
                          "data": data}, work, repo)
    return data


def cached_source_index(root, manifest, work, repo):
    """Build or reuse the source-only TF-IDF index and per-unit anchor information.

    The index is fit over every unit's expected-Latin windows across the whole corpus,
    which is permitted early; reader output never contributes to the vocabulary or IDF.
    Persisted vocabulary/IDF/settings and source hashes let later stages transform reader
    windows without refitting."""
    source = cached_source_regions(root, manifest, work, repo)
    metadata = {"adapter": "source-tfidf-index-v1", "source_regions_adapter": source["adapter"],
                "inputs": {r["path"]: r["sha256"] for r in manifest["inputs"]
                           if r["role"] in ("source_pdf", "units", "duplicate_policy")},
                "windows": {"long_size": LONG_WINDOW, "long_stride": LONG_STRIDE},
                "views": VIEW_ADAPTER, "vectorizer": VECTORIZER_SETTINGS}
    path = Path(work) / "cache" / "source_index" / f"{json_hash(metadata)}.json"
    cached = reusable_entry(path, metadata, work, repo)
    if cached:
        return cached
    units_payload, strict_docs, tolerant_docs, windows = [], [], [], []
    for unit in source["units"]:
        tokens = unit_source_tokens(unit)
        views = comparison_views([token["raw"] for token in tokens])
        strict = [view["strict"] for view in views]
        tolerant = [view["tolerant"] for view in views]
        for start, end in passage_windows(len(tokens), LONG_WINDOW, LONG_STRIDE):
            windows.append({"unit_id": unit["unit_id"], "source_index": unit["source"],
                            "start": start, "end": end})
            strict_docs.append(" ".join(strict[start:end]))
            tolerant_docs.append(" ".join(tolerant[start:end]))
        units_payload.append({"unit_id": unit["unit_id"], "source_index": unit["source"],
                              "token_count": len(tokens), "tokens": tokens,
                              "strict": strict, "tolerant": tolerant,
                              "anchors": anchor_information(strict)})
    with run_directory(path.parent, work, repo):
        fit = fit_source_index(strict_docs, tolerant_docs)
        strict_terms, strict_idf = vectorizer_vocab_idf(fit["strict"][0])
        tolerant_terms, tolerant_idf = vectorizer_vocab_idf(fit["tolerant"][0])
        identity = {"adapter": metadata["adapter"], "settings": VECTORIZER_SETTINGS,
                    "views": VIEW_ADAPTER, "windows": metadata["windows"],
                    "unit_count": len(units_payload), "window_count": len(windows),
                    "strict": view_identity(strict_terms, strict_idf),
                    "tolerant": view_identity(tolerant_terms, tolerant_idf),
                    "source_tokens_sha256": json_hash([unit["tokens"] for unit in units_payload])}
        data = {"adapter": metadata["adapter"], "settings": VECTORIZER_SETTINGS, "views": VIEW_ADAPTER,
                "windows_spec": metadata["windows"], "identity": identity,
                "strict_vocabulary": strict_terms, "strict_idf": strict_idf,
                "tolerant_vocabulary": tolerant_terms, "tolerant_idf": tolerant_idf,
                "windows": windows, "units": units_payload}
        write_json(path, {"schema_version": 1, "complete": True, "metadata": metadata,
                          "data": data}, work, repo)
    return data


def validate_box(box):
    if (not isinstance(box, list) or len(box) != 4
            or any(isinstance(n, bool) or not isinstance(n, (int, float))
                   or not math.isfinite(n) for n in box)
            or not 0 <= box[0] < box[2] <= 1 or not 0 <= box[1] < box[3] <= 1):
        raise AuditError(f"Invalid normalized box: {box}")
    return box


def snapshot_input(root, manifest, *, role, tomus=None, source_index=None):
    root = Path(root).resolve()
    matches = [r for r in manifest["inputs"] if r["role"] == role
               and (tomus is None or r.get("tomus") == tomus)
               and (source_index is None or r.get("source_index") == source_index)]
    if len(matches) != 1:
        raise AuditError(f"Expected one captured {role} input for {tomus}/{source_index}")
    record = matches[0]
    path = root / relative_path(record["snapshot_path"])
    if not contained(path.resolve(), root):
        raise AuditError("Captured input redirects outside snapshot")
    return path, record


def page_geometry(page):
    return {"mediabox": list(page.mediabox), "cropbox": list(page.cropbox),
            "rotation": page.rotation, "render_box_policy": "mediabox"}


def normalized_pdf_box(page, box):
    """Map unrotated PyMuPDF points to displayed MediaBox coordinates."""
    import pymupdf
    mx0, my0, mx1, my1 = page.mediabox
    inverse = ~page.transformation_matrix
    corners = []
    for x, y in ((box[0], box[1]), (box[2], box[1]), (box[2], box[3]), (box[0], box[3])):
        point = pymupdf.Point(x, y) * inverse
        u, v = (point.x - mx0) / (mx1 - mx0), (my1 - point.y) / (my1 - my0)
        rotated = {0: (u, v), 90: (1 - v, u), 180: (1 - u, 1 - v), 270: (v, 1 - u)}
        if page.rotation not in rotated:
            raise AuditError("Unsupported page rotation")
        corners.append(rotated[page.rotation])
    return [min(p[0] for p in corners), min(p[1] for p in corners),
            max(p[0] for p in corners), max(p[1] for p in corners)]


def pixel_bounds(box, size):
    validate_box(box)
    width, height = size
    return (math.floor(box[0] * width), math.floor(box[1] * height),
            math.ceil(box[2] * width), math.ceil(box[3] * height))


def parse_tex_refs(path):
    text = re.sub(r"(?<!\\)%[^\n]*", "", Path(path).read_text(encoding="utf-8"))
    text = text.split(r"\begin{document}", 1)[1].split(r"\end{document}", 1)[0]
    return re.findall(r"\\includegraphics(?:\s*\[[^\]]*\])?\s*\{([^{}]+\.png)\}", text)


def enumerate_image_occurrences(page):
    """Enumerate displayed image occurrences, including repeated uses of one object."""
    return [{"xref": image["xref"], "box": normalized_pdf_box(page, image["bbox"])}
            for image in page.get_image_info(xrefs=True)]


def benchmark_inventory(root, manifest):
    """Collect metadata for fixed panel selection without OCR or detector scores."""
    import pymupdf
    rows, geometry = [], []
    for tomus in range(1, 9):
        tex, _ = snapshot_input(root, manifest, role="reader_tex", tomus=tomus)
        pdf, record = snapshot_input(root, manifest, role="reader_pdf", tomus=tomus)
        refs = parse_tex_refs(tex)
        with pymupdf.open(pdf) as document:
            occurrences = []
            for number, page in enumerate(document, 1):
                geometry.append(dict(page_geometry(page), tomus=tomus, page=number))
                occurrences.extend((number, occurrence) for occurrence in enumerate_image_occurrences(page))
            if len(refs) != len(occurrences):
                raise AuditError(f"Ambiguous tomus {tomus} image inventory: {len(refs)} TeX refs / {len(occurrences)} occurrences")
            body = []
            for asset, (number, occurrence) in zip(refs, occurrences):
                match = re.fullmatch(r"facsimile_(.+)_(\d+)\.png", Path(asset).name)
                if match:
                    validate_box(occurrence["box"])
                    body.append({"tomus": tomus, "page": number, "unit_id": match[1],
                                 "ordinal": int(match[2]), "asset": asset,
                                 "input_sha256": record["sha256"], "body_box": occurrence["box"]})
            if len({r["page"] for r in body}) != len(body):
                raise AuditError("Multiple body occurrences on one page need manual mapping")
            expected = manifest["expected_counts"]
            if len(document) != expected["physical_by_tomus"][tomus - 1] or len(body) != expected["body_by_tomus"][tomus - 1]:
                raise AuditError(f"Pinned page count mismatch in tomus {tomus}")
            rows.extend(body)
            progress(f"Metadata inventory: tomus {tomus}, {len(document)} physical / {len(body)} body pages")
    return rows, geometry


BODY_ASSET = re.compile(r"facsimile_(.+)_(\d+)\.png")


def read_footer_folio(page):
    """Return every generated Arabic folio printed in the bottom margin of a page.

    The reader's `folio` page style prints `\\thepage` (body folio = physical page -
    body-first page + 1) in the bottom margin; unit openings suppress it with an empty
    page style, so a body opening carries no footer word and its folio is inferred."""
    found = []
    for x0, y0, x1, y1, text, *_ in page.get_text("words"):
        label = text.strip()
        if label.isdigit():
            box = normalized_pdf_box(page, [x0, y0, x1, y1])
            if box[1] >= 0.9:
                found.append({"folio": int(label), "box": box})
    return found


def map_reader_pages(root, manifest):
    """Map ordered TeX image references to displayed reader pages, folios and units.

    Only actual ordered `\\includegraphics` occurrences are mapped, not unique image
    objects; front-matter scans, blank recto separators and signature padding are kept
    distinct from numbered body images. Printed folios come from the generated footer;
    suppressed unit openings are inferred from the tomus's single page/folio offset."""
    import pymupdf
    expected = manifest["expected_counts"]
    tomi, body_rows, unit_summaries = [], [], []
    for tomus in range(1, 9):
        tex, _ = snapshot_input(root, manifest, role="reader_tex", tomus=tomus)
        pdf, record = snapshot_input(root, manifest, role="reader_pdf", tomus=tomus)
        refs = parse_tex_refs(tex)
        with pymupdf.open(pdf) as document:
            scanned, occurrences = [], []
            for number, page in enumerate(document, 1):
                occ = enumerate_image_occurrences(page)
                scanned.append({"page": number, "occurrences": occ,
                                "folios": read_footer_folio(page),
                                "word_count": len(page.get_text("words"))})
                occurrences.extend((number, o) for o in occ)
            if len(refs) != len(occurrences):
                raise AuditError(f"Ambiguous tomus {tomus} image inventory: {len(refs)} TeX refs / {len(occurrences)} occurrences")
            placements = {number: [] for number in range(1, len(document) + 1)}
            for asset, (number, occ) in zip(refs, occurrences):
                match = BODY_ASSET.fullmatch(Path(asset).name)
                placements[number].append({"asset": asset, "box": occ["box"],
                                           "unit_id": match[1] if match else None,
                                           "ordinal": int(match[2]) if match else None})
            body_numbers = [n for n, items in placements.items() if any(i["unit_id"] for i in items)]
            if any(sum(1 for i in placements[n] if i["unit_id"]) > 1 for n in body_numbers):
                raise AuditError(f"Multiple body occurrences on one tomus {tomus} page need manual mapping")
            first_body, last_body = (min(body_numbers), max(body_numbers)) if body_numbers else (None, None)
            offsets = {scan["page"] - scan["folios"][0]["folio"]
                       for scan, number in zip(scanned, placements)
                       if len(scan["folios"]) == 1 and any(i["unit_id"] for i in placements[scan["page"]])}
            offset = next(iter(offsets)) if len(offsets) == 1 else None
            pages, anomalies = [], []
            for scan in scanned:
                number = scan["page"]
                items = placements[number]
                body = next((i for i in items if i["unit_id"]), None)
                front_assets = [Path(i["asset"]).name for i in items if not i["unit_id"]]
                if body:
                    kind = "body"
                elif number < (first_body or 0):
                    kind = "frontmatter"
                elif last_body is not None and number > last_body:
                    kind = "padding"
                else:
                    kind = "separator"
                entry = {"page": number, "kind": kind, "has_image": bool(items),
                         "image_assets": front_assets, "word_count": scan["word_count"]}
                if body:
                    validate_box(body["box"])
                    is_opening = body["ordinal"] == 1
                    printed = scan["folios"]
                    if len(printed) == 1:
                        folio, provenance = printed[0]["folio"], "printed"
                        if offset is not None and number - offset != folio:
                            anomalies.append({"page": number, "printed": folio, "offset": offset})
                    elif offset is not None:
                        folio, provenance = number - offset, "inferred"
                    else:
                        folio, provenance = None, "unknown"
                    entry.update({"unit_id": body["unit_id"], "ordinal": body["ordinal"],
                                  "asset": body["asset"], "body_box": body["box"],
                                  "is_opening": is_opening, "folio": folio,
                                  "folio_provenance": provenance})
                    body_rows.append({"tomus": tomus, "page": number, "unit_id": body["unit_id"],
                                      "ordinal": body["ordinal"], "asset": body["asset"],
                                      "input_sha256": record["sha256"], "body_box": body["box"],
                                      "folio": folio, "folio_provenance": provenance,
                                      "is_opening": is_opening})
                pages.append(entry)
            tomus_body = [p for p in pages if p["kind"] == "body"]
            if len(document) != expected["physical_by_tomus"][tomus - 1] or len(tomus_body) != expected["body_by_tomus"][tomus - 1]:
                raise AuditError(f"Pinned page count mismatch in tomus {tomus}")
            order, grouped = [], {}
            for row in tomus_body:
                grouped.setdefault(row["unit_id"], []).append(row)
                if row["unit_id"] not in order:
                    order.append(row["unit_id"])
            units = []
            for unit_id in order:
                rows = grouped[unit_id]
                ordinals = [r["ordinal"] for r in rows]
                if sorted(ordinals) != list(range(1, len(rows) + 1)):
                    raise AuditError(f"Non-contiguous body ordinals for {unit_id} in tomus {tomus}")
                opening, closing = rows[0], rows[-1]
                units.append({"unit_id": unit_id, "tomus": tomus, "page_count": len(rows),
                              "opening_page": opening["page"], "opening_folio": opening["folio"],
                              "opening_folio_provenance": opening["folio_provenance"],
                              "closing_page": closing["page"], "closing_folio": closing["folio"]})
            unit_summaries.extend(units)
            tomi.append({"tomus": tomus, "input_sha256": record["sha256"],
                         "physical_pages": len(document), "body_pages": len(tomus_body),
                         "first_body_page": first_body, "last_body_page": last_body,
                         "folio_offset": offset, "offset_candidates": sorted(offsets),
                         "folio_anomalies": anomalies, "units": units, "pages": pages})
            progress(f"Reader map: tomus {tomus}, {len(document)} physical / {len(tomus_body)} body / "
                     f"{len(units)} units, folio offset {offset}")
    if len(body_rows) != expected["body_pages"] or len(unit_summaries) != expected["units"]:
        raise AuditError(f"Reader map totals {len(body_rows)} body / {len(unit_summaries)} units differ from pinned corpus counts")
    return {"schema_version": 1, "adapter": "reader-map-folios-v1", "tomi": tomi,
            "body_pages": body_rows, "units": unit_summaries}


SOURCE_LATIN_X0 = 0.44  # Broad right-column/gutter start before source-word consensus.
SOURCE_LATIN_X1 = 1.0
SOURCE_FINAL_PAGE = {0: 637, 1: 708}


def read_approved_exclusions(root, manifest):
    """Read the complete unit-scoped duplicate skip_pages from approved ledger records.

    Only `duplicate_page` dispositions authorize dropping expected Latin, and the whole
    `skip_pages` list is honored, not just the representative `source_page`. Masks and
    row exclusions never remove expected prose, so they are ignored here."""
    policy_path, _ = snapshot_input(root, manifest, role="duplicate_policy")
    policy = json.loads(policy_path.read_text(encoding="utf-8"))
    exclusions = {}
    for record in policy.get("repairs", []):
        if record.get("disposition") != "duplicate_page":
            continue
        pages = record.get("skip_pages")
        if (not isinstance(pages, list) or not pages
                or any(isinstance(p, bool) or not isinstance(p, int) for p in pages)):
            raise AuditError(f"Duplicate-page record needs an explicit skip_pages list: {record.get('finding_ids')}")
        exclusions.setdefault((record["unit_id"], record["source"]), set()).update(pages)
    return {key: sorted(value) for key, value in exclusions.items()}


def partition_source_units(units, exclusions=None):
    """Derive each unit's inclusive source range, mirroring `units.rs::ranges`.

    The next unit in the same source shares its opening page as an inclusive tail cut
    above its printed title; a source's final unit runs to its recorded `last_page`
    (or the pinned 637/708 endpoint). Approved duplicate skips are attached per unit."""
    exclusions = exclusions or {}
    partitions = []
    for index, unit in enumerate(units):
        following = units[index + 1] if index + 1 < len(units) else None
        if following and following["source"] == unit["source"]:
            end_page, shared_tail = following["start_page"], True
            tail_cut = following.get("title_top_frac")
        else:
            end_page = unit.get("last_page", SOURCE_FINAL_PAGE[unit["source"]])
            shared_tail, tail_cut = False, None
        skip_pages = exclusions.get((unit["id"], unit["source"]), [])
        if any(not unit["start_page"] <= page <= end_page for page in skip_pages):
            raise AuditError(f"Approved skip page outside {unit['id']} partition")
        partitions.append({"unit_id": unit["id"], "source": unit["source"],
                           "start_page": unit["start_page"], "end_page": end_page,
                           "shared_tail": shared_tail, "title_top_frac": unit.get("title_top_frac"),
                           "open_body_frac": unit.get("body_top_frac"), "tail_cut_frac": tail_cut,
                           "skip_pages": skip_pages})
    return partitions


def group_lines(words):
    """Group word observations into reading lines by vertical overlap."""
    lines = []
    for word in sorted(words, key=lambda w: (w["box"][1], w["box"][0])):
        top, bottom = word["box"][1], word["box"][3]
        height = bottom - top
        placed = False
        for line in lines:
            overlap = min(bottom, line["box"][3]) - max(top, line["box"][1])
            if overlap > 0.5 * min(height, line["box"][3] - line["box"][1]):
                line["words"].append(word)
                line["box"] = [min(line["box"][0], word["box"][0]), min(line["box"][1], top),
                               max(line["box"][2], word["box"][2]), max(line["box"][3], bottom)]
                placed = True
                break
        if not placed:
            lines.append({"box": list(word["box"]), "words": [word]})
    result = []
    for line in sorted(lines, key=lambda l: l["box"][1]):
        ordered = sorted(line["words"], key=lambda w: w["box"][0])
        result.append({"box": line["box"], "text": " ".join(w["text"] for w in ordered)})
    return result


def extract_source_regions(root, manifest, partitions):
    """Extract generous expected-Latin regions per unit from embedded source words.

    A broad right-column region is refined to actual word extent plus a character-width
    margin, never trimmed to the production bands. The opening title block and the shared
    tail (next unit's territory) are cut by their fractions; head/footer furniture is
    recorded separately so running heads and folios do not masquerade as body prose."""
    import pymupdf
    documents = {}
    try:
        results = []
        for partition in partitions:
            source = partition["source"]
            if source not in documents:
                pdf, _ = snapshot_input(root, manifest, role="source_pdf", source_index=source)
                documents[source] = pymupdf.open(pdf)
            document = documents[source]
            pages, limitations = [], []
            line_total = word_total = head_total = 0
            for number in range(partition["start_page"], partition["end_page"] + 1):
                if number in partition["skip_pages"]:
                    pages.append({"page": number, "skipped": True,
                                  "reason": "approved duplicate scan contributes no expected Latin"})
                    continue
                if not 1 <= number <= len(document):
                    raise AuditError(f"Source page {number} outside {partition['unit_id']} document")
                page = document[number - 1]
                words = []
                for x0, y0, x1, y1, text, *_ in page.get_text("words"):
                    if text.strip():
                        words.append({"box": normalized_pdf_box(page, [x0, y0, x1, y1]), "text": text})
                is_opening = number == partition["start_page"]
                top_cut = partition["open_body_frac"] if is_opening and partition["open_body_frac"] else 0.0
                bottom_cut = (partition["tail_cut_frac"] if number == partition["end_page"]
                              and partition["shared_tail"] and partition["tail_cut_frac"] else 1.0)
                latin, head, footer = [], [], []
                for word in words:
                    center_x = (word["box"][0] + word["box"][2]) / 2
                    center_y = (word["box"][1] + word["box"][3]) / 2
                    if center_x < SOURCE_LATIN_X0:
                        continue
                    if center_y < top_cut or center_y >= bottom_cut:
                        head.append(word) if center_y < top_cut else footer.append(word)
                        continue
                    if center_y < 0.06:
                        head.append(word)
                    elif center_y > 0.94:
                        footer.append(word)
                    latin.append(word)
                lines = group_lines(latin)
                if latin:
                    widths = [(w["box"][2] - w["box"][0]) / max(len(w["text"]), 1) for w in latin]
                    margin = sorted(widths)[len(widths) // 2]
                    left = max(0.0, min(SOURCE_LATIN_X0, min(w["box"][0] for w in latin) - margin))
                    region = [left, top_cut, SOURCE_LATIN_X1, bottom_cut]
                else:
                    region = [SOURCE_LATIN_X0, top_cut, SOURCE_LATIN_X1, bottom_cut]
                    limitations.append(f"page {number}: no embedded Latin-column words; raster OCR consensus deferred to Phase 3")
                validate_box(region)
                line_total += len(lines)
                word_total += len(latin)
                head_total += len(head)
                pages.append({"page": number, "skipped": False, "region": region,
                              "lines": lines, "head_observations": head, "footer_observations": footer,
                              "provenance": {"embedded_words": True, "open_body_cut": is_opening and top_cut > 0,
                                             "shared_tail_cut": bottom_cut < 1.0}})
            results.append({"unit_id": partition["unit_id"], "source": source,
                            "start_page": partition["start_page"], "end_page": partition["end_page"],
                            "shared_tail": partition["shared_tail"], "skip_pages": partition["skip_pages"],
                            "line_count": line_total, "word_count": word_total, "head_count": head_total,
                            "pages": pages, "limitations": limitations})
            progress(f"Source extraction: {partition['unit_id']} (v{source + 1} "
                     f"{partition['start_page']}-{partition['end_page']}), {line_total} lines, "
                     f"{len(partition['skip_pages'])} skipped")
    finally:
        for document in documents.values():
            document.close()
    return {"schema_version": 1, "adapter": "source-latin-regions-v1",
            "latin_column": [SOURCE_LATIN_X0, SOURCE_LATIN_X1], "units": results}


LONG_WINDOW, LONG_STRIDE = 64, 16
LOCAL_WINDOW, LOCAL_STRIDE = 16, 4
VIEW_ADAPTER = "strict-casefold-tolerant-ligature-longs-v1"
VECTORIZER_SETTINGS = {"analyzer": "char", "ngram_range": [3, 5], "lowercase": False,
                       "min_df": 1, "max_df": 1.0, "sublinear_tf": True,
                       "smooth_idf": True, "norm": "l2"}
LIGATURES = {"æ": "ae", "Æ": "ae", "œ": "oe", "Œ": "oe",
             "ﬀ": "ff", "ﬁ": "fi", "ﬂ": "fl", "ﬃ": "ffi",
             "ﬄ": "ffl", "ſ": "s"}


def strict_view(word):
    """Fold case and surrounding whitespace only; keep u/v/i/j and short words intact."""
    return word.strip().casefold()


def tolerant_view(word):
    """Additionally expand ligatures and long-s; output Latin is never normalized by this."""
    text = word.strip()
    for glyph, expansion in LIGATURES.items():
        text = text.replace(glyph, expansion)
    return text.casefold()


def comparison_views(words):
    """Return raw/strict/tolerant parallel token views keyed to original word indices."""
    return [{"index": index, "raw": raw, "strict": strict_view(raw), "tolerant": tolerant_view(raw)}
            for index, raw in enumerate(words)]


def passage_windows(count, size, stride):
    """Yield (start, end) token intervals, always including the trailing tokens."""
    if count <= 0:
        return []
    starts = list(range(0, max(count - size, 0) + 1, stride))
    if starts[-1] + size < count:
        starts.append(count - size)
    return [(start, min(start + size, count)) for start in sorted(set(starts))]


def anchor_information(strict_tokens):
    """Mark tokens informative when they belong to a rare 2-4 token shingle within a unit.

    A shingle is rare when it occurs at no more than max(2, 10% of the unit's tokens)
    distinct positions. Established duplicate aliases are collapsed by callers before this;
    natural repeated wording stays separate and simply lowers informativeness there."""
    count = len(strict_tokens)
    threshold = max(2, math.floor(0.10 * count))
    informative = [False] * count
    for size in (2, 3, 4):
        if count < size:
            continue
        positions = {}
        for start in range(count - size + 1):
            positions.setdefault(tuple(strict_tokens[start:start + size]), []).append(start)
        for locations in positions.values():
            if len(locations) <= threshold:
                for start in locations:
                    for index in range(start, start + size):
                        informative[index] = True
    return {"token_count": count, "threshold": threshold,
            "informative": informative, "informative_count": sum(informative)}


def anchor_uniqueness(position_scores, cutoff=0.80):
    """1 / distinct context-correct positions scoring >= cutoff; None when none qualify.

    Callers collapse overlapping windows of one interval to a single position first."""
    qualifying = sum(1 for score in position_scores if score >= cutoff)
    return 1.0 / qualifying if qualifying else None


def placement_margin(position_scores):
    """Best minus second-best distinct-position score; 1.0 when no competitor exists."""
    ordered = sorted(position_scores, reverse=True)
    return 1.0 if len(ordered) <= 1 else ordered[0] - ordered[1]


def fit_source_index(strict_docs, tolerant_docs):
    """Fit strict/tolerant char TF-IDF vectorizers on source-reference windows only."""
    from sklearn.feature_extraction.text import TfidfVectorizer

    def build(docs):
        vectorizer = TfidfVectorizer(analyzer="char", ngram_range=(3, 5), lowercase=False,
                                     min_df=1, max_df=1.0, sublinear_tf=True,
                                     smooth_idf=True, norm="l2")
        matrix = vectorizer.fit_transform(docs)
        return vectorizer, matrix

    return {"settings": VECTORIZER_SETTINGS,
            "strict": build(strict_docs), "tolerant": build(tolerant_docs)}


def vectorizer_vocab_idf(vectorizer):
    terms = [term for term, _ in sorted(vectorizer.vocabulary_.items(), key=lambda kv: kv[1])]
    return terms, [float(value) for value in vectorizer.idf_]


def view_identity(terms, idf):
    return {"n_features": len(terms), "vocab_sha256": json_hash(terms),
            "idf_sha256": json_hash([round(value, 10) for value in idf])}


def rebuild_vectorizer(vocabulary, idf):
    """Rebuild a transform-only vectorizer from persisted vocabulary/IDF; never refit."""
    from sklearn.feature_extraction.text import TfidfVectorizer
    import numpy
    vectorizer = TfidfVectorizer(analyzer="char", ngram_range=(3, 5), lowercase=False,
                                 sublinear_tf=True, smooth_idf=True, norm="l2",
                                 vocabulary={term: index for index, term in enumerate(vocabulary)})
    vectorizer._validate_vocabulary()
    vectorizer.idf_ = numpy.asarray(idf, dtype=numpy.float64)
    return vectorizer


def load_source_index(index):
    """Reconstruct transform-only vectorizers; reader windows never alter vocabulary/IDF."""
    return {"strict": rebuild_vectorizer(index["strict_vocabulary"], index["strict_idf"]),
            "tolerant": rebuild_vectorizer(index["tolerant_vocabulary"], index["tolerant_idf"])}


def unit_source_tokens(unit):
    """Flatten a unit's expected-Latin line tokens with page/line provenance."""
    tokens = []
    for page in unit["pages"]:
        if page.get("skipped"):
            continue
        for line_index, line in enumerate(page["lines"]):
            for word in line["text"].split():
                tokens.append({"raw": word, "page": page["page"], "line": line_index})
    return tokens


DEFAULT_LOCALIZATION_RULE = {
    "tiny_event": "iou", "minimum_iou": 0.1, "omission_source_coverage": 0.8,
    "maximum_extra_source_lines": 1, "duplicate_endpoint_tolerance_line_heights": 1,
}
SOURCE_DUPLICATE = {"cosine_min": 0.55, "ordered_min": 0.75, "informative_min": 40,
                    "short_unit_fraction": 0.60, "top_k": 8}
RETRIEVAL = {"cosine_min": 0.55, "top_k": 8, "anchor_min_tokens": 8}
CHAIN = {"forward_source": 0.04, "forward_output": 0.02, "gap_cap": 400,
         "restart_base": 2.0, "restart_backward": 0.5, "backward_cap": 64, "beam": 8}
DUPLICATE_HIGH = {"cosine_min": 0.80, "ordered_min": 0.90, "informative_min": 40}
ORDER_HIGH = {"backward_tokens_min": 64}
INCOMPLETE_OPENING_TOKENS = 8  # A later copy that starts this many tokens in has lost its opening.
MATRIX_CAP = 400  # Bounded local alignment is split or marked inconclusive beyond this, never truncated.
OMISSION = {"min_missing_tokens": 5, "min_missing_lines": 1, "context_min_informative": 8,
            "ordered_min": 0.80, "context_window": 16, "line_fraction": 0.8,
            "latin_line_fraction": 0.5, "seam_line_span": 0, "fuzzy_cover_min": 0.75}
HYPHENS = "-‐‑­‒–—"
BOUNDARY = {"source_window": 48, "output_window": 96, "coverage_min": 0.80,
            "ordered_min": 0.75, "minimum_tokens": 8, "edge_tokens": 6}
WATERMARK = {"dpi": 90, "nomination_min": 0.60, "high_min": 0.80,
             "scale_min": 0.75, "scale_max": 1.30, "scale_step": 0.05,
             "fallback_min": 0.30, "fallback_max": 1.50,
             "rotation_min": -3, "rotation_max": 3, "template_std_min": 2.0,
             "ink_fraction_min": 0.005,
             "stable_distance_line_heights": 1.0, "glyph_support_min": 2,
             "faint_min_gray": 110, "faint_ink_fraction_min": 0.45,
             "edge_strip_line_heights": 4, "event_edge_line_heights": 3,
             "partial_bands": [[0.0, 0.25], [0.25, 0.50], [0.50, 0.75], [0.75, 1.0]]}
HEAD = {"nomination_min": 0.80, "high_min": 0.80, "faint_min_gray": 0,
        "faint_ink_fraction_min": 0.40, "suffix_min": 0.80, "source_head_bottom": 0.12,
        "padding_line_heights": 2, "minimum_cues": 3, "image_capital_height_min": 0.70}
REGISTRATION = {"dpi": 150, "features": 4000, "ratio": 0.70, "minimum_inliers": 24,
                "reprojection_px": 2.5, "minimum_inlier_fraction": 0.30}
SCRAPS = {"minimum_components": 3, "minimum_lines": 2, "correlation_min": 0.80,
          "margin_body_fraction": 0.15, "greek_characters_min": 2}
IMPLEMENTED_DETECTORS = {"duplicate", "order", "missing_occurrence", "terminal_loss", "watermark", "head", "foreign_scraps"}


def detector_settings():
    """Return the calibratable detector configuration embedded in every run manifest.

    Thresholds and the pre-declared localization rule are frozen with the configuration
    before validation; a changed value requires a new run folder and fresh calibration."""
    return {"localization_rule": DEFAULT_LOCALIZATION_RULE,
            "windows": {"long": [LONG_WINDOW, LONG_STRIDE], "local": [LOCAL_WINDOW, LOCAL_STRIDE]},
            "views": VIEW_ADAPTER,
            "thresholds": {"source_duplicate": SOURCE_DUPLICATE, "retrieval": RETRIEVAL,
                           "chain": CHAIN, "duplicate_high": DUPLICATE_HIGH, "order_high": ORDER_HIGH,
                           "omission": OMISSION, "boundary": BOUNDARY,
                           "watermark": WATERMARK,
                           "head": HEAD,
                           "registration": REGISTRATION, "scraps": SCRAPS,
                           "matrix_cap": MATRIX_CAP, "repeat_min_tokens": 16}}


def align_tokens(source, output, mode="exact", settings=None):
    """Shared token aligner.

    ``exact`` mode takes two strict-string lists and returns maximal common diagonals; each
    run is a maximal (source_offset, output_offset, length) stretch of equal strict tokens and
    ordered similarity comes from a monotonic non-overlapping chain. ``local`` mode takes two
    view-token lists ({strict, tolerant, optional box}) and runs a bounded token edit-distance
    program with discretionary join/split transitions, returning preserved operations, covered
    source tokens and an ordered-similarity score for omission and boundary nomination."""
    if mode == "local":
        return bounded_align(source, output, settings)
    if mode != "exact":
        raise AuditError("align_tokens mode must be 'exact' or 'local'")
    m, n = len(source), len(output)
    runs = []
    for i in range(m):
        for j in range(n):
            if source[i] == output[j] and (i == 0 or j == 0 or source[i - 1] != output[j - 1]):
                length = 0
                while i + length < m and j + length < n and source[i + length] == output[j + length]:
                    length += 1
                runs.append((i, j, length))
    chain = chain_runs(runs)
    matched = sum(length for _, _, length in chain)
    return {"runs": runs, "chain": chain, "matched": matched,
            "ordered_similarity": matched / max(m, n, 1)}


def char_similarity(first, second):
    """Normalized character Levenshtein similarity in [0, 1]; empty vs empty is identical."""
    from rapidfuzz.distance import Levenshtein
    if not first and not second:
        return 1.0
    return float(Levenshtein.normalized_similarity(first, second))


def strip_trailing_hyphen(text):
    """Drop a single discretionary line-break hyphen so split halves can rejoin."""
    return text[:-1] if text and text[-1] in HYPHENS else text


def joined_views(first, second):
    """Concatenate two view tokens, dropping the first's trailing line-hyphen if present."""
    return {"strict": strip_trailing_hyphen(first["strict"]) + second["strict"],
            "tolerant": strip_trailing_hyphen(first["tolerant"]) + second["tolerant"]}


def token_substitution(source_token, output_token, fuzzy_min):
    """Return (cost, kind, covered) for aligning one source view token to one output token.

    Strict equality is free, tolerant (ligature/long-s) equality costs 0.10, otherwise the
    cost is ``min(1.8, 2*(1-sim))`` for the better of strict/tolerant character similarity.
    A substitution only counts as coverage when it is strict/tolerant equal or sim>=fuzzy_min;
    a cheaper low-similarity substitution is still permitted but stays unmatched."""
    if source_token["strict"] == output_token["strict"]:
        return 0.0, "strict", True
    if source_token["tolerant"] == output_token["tolerant"]:
        return 0.10, "tolerant", True
    sim = max(char_similarity(source_token["strict"], output_token["strict"]),
              char_similarity(source_token["tolerant"], output_token["tolerant"]))
    return min(1.8, 2 * (1 - sim)), "fuzzy", sim >= fuzzy_min


def bounded_align(source, output, settings):
    """Bounded token edit-distance alignment with discretionary join/split transitions.

    Deletion/insertion cost 1.0; the diagonal uses :func:`token_substitution`; join (two
    source tokens to one output) and split (one source token to two output) each add 0.15 and
    are permitted only when the joined comparison itself reaches coverage, so arbitrary adjacent
    words cannot be glued. Ties prefer strict, tolerant, join/split, fuzzy diagonals, then source
    deletion and output insertion. Matrices above the configured cap are marked inconclusive and
    never silently truncated. Operations and the covered source-token set are preserved."""
    if settings is None:
        raise AuditError("Bounded local alignment requires detector settings")
    fuzzy_min = settings["thresholds"]["omission"]["fuzzy_cover_min"]
    cap = settings["thresholds"].get("matrix_cap", MATRIX_CAP)
    m, n = len(source), len(output)
    if m > cap or n > cap:
        return {"inconclusive": True, "reason": "matrix_exceeds_cap", "cost": None,
                "ordered_similarity": 0.0, "matched": 0, "operations": [], "covered_source": []}
    infinity = float("inf")
    distance = [[infinity] * (n + 1) for _ in range(m + 1)]
    back = [[None] * (n + 1) for _ in range(m + 1)]
    distance[0][0] = 0.0
    for i in range(1, m + 1):
        distance[i][0] = float(i)
        back[i][0] = (i - 1, 0, {"op": "delete", "source": [i - 1], "output": [], "covered": False, "cost": 1.0})
    for j in range(1, n + 1):
        distance[0][j] = float(j)
        back[0][j] = (0, j - 1, {"op": "insert", "source": [], "output": [j - 1], "covered": False, "cost": 1.0})
    diagonal_priority = {"strict": 1, "tolerant": 2, "fuzzy": 4}
    for i in range(1, m + 1):
        for j in range(1, n + 1):
            moves = []
            cost, kind, covered = token_substitution(source[i - 1], output[j - 1], fuzzy_min)
            moves.append((distance[i - 1][j - 1] + cost, diagonal_priority[kind],
                          (i - 1, j - 1, {"op": kind, "source": [i - 1], "output": [j - 1],
                                          "covered": covered, "cost": cost})))
            if i >= 2:
                jc, _, jcov = token_substitution(joined_views(source[i - 2], source[i - 1]), output[j - 1], fuzzy_min)
                if jcov:
                    moves.append((distance[i - 2][j - 1] + 0.15 + jc, 3,
                                  (i - 2, j - 1, {"op": "join", "source": [i - 2, i - 1], "output": [j - 1],
                                                  "covered": True, "cost": 0.15 + jc})))
            if j >= 2:
                sc, _, scov = token_substitution(source[i - 1], joined_views(output[j - 2], output[j - 1]), fuzzy_min)
                if scov:
                    moves.append((distance[i - 1][j - 2] + 0.15 + sc, 3,
                                  (i - 1, j - 2, {"op": "split", "source": [i - 1], "output": [j - 2, j - 1],
                                                  "covered": True, "cost": 0.15 + sc})))
            moves.append((distance[i - 1][j] + 1.0, 5,
                          (i - 1, j, {"op": "delete", "source": [i - 1], "output": [], "covered": False, "cost": 1.0})))
            moves.append((distance[i][j - 1] + 1.0, 6,
                          (i, j - 1, {"op": "insert", "source": [], "output": [j - 1], "covered": False, "cost": 1.0})))
            value, _, step = min(moves, key=lambda move: (round(move[0], 9), move[1]))
            distance[i][j] = value
            back[i][j] = step
    operations, i, j = [], m, n
    while i > 0 or j > 0:
        pi, pj, step = back[i][j]
        operations.append(step)
        i, j = pi, pj
    operations.reverse()
    covered_source = sorted({index for step in operations if step["covered"] for index in step["source"]})
    cost = distance[m][n]
    return {"inconclusive": False, "cost": cost, "operations": operations,
            "matched": len(covered_source), "covered_source": covered_source,
            "ordered_similarity": max(0.0, 1 - cost / max(m, n, 1))}


def chain_runs(runs):
    """Pick a strictly increasing, non-overlapping run subset maximizing matched tokens."""
    ordered = sorted(runs, key=lambda run: (run[0], run[1]))
    if not ordered:
        return []
    best = [length for _, _, length in ordered]
    previous = [-1] * len(ordered)
    for index, (si, oi, length) in enumerate(ordered):
        for earlier in range(index):
            sk, ok, lk = ordered[earlier]
            if sk + lk <= si and ok + lk <= oi and best[earlier] + length > best[index]:
                best[index] = best[earlier] + length
                previous[index] = earlier
    end = max(range(len(ordered)), key=lambda index: best[index])
    chain = []
    while end != -1:
        chain.append(ordered[end])
        end = previous[end]
    return list(reversed(chain))


def token_pages(unit, start, end):
    """The distinct source pages spanned by an inclusive-exclusive token interval."""
    return sorted({unit["tokens"][i]["page"] for i in range(start, min(end, len(unit["tokens"])))})


def find_source_duplicates(unit, vectorizer, settings):
    """Nominate scan-duplicate passages inside one source unit's expected Latin.

    Distinct, non-overlapping long windows are ranked by char TF-IDF cosine; the top
    partners are refined by exact-token alignment counting informative matched tokens.
    Consecutive matching windows merge into one block carrying both source occurrences."""
    th = settings["thresholds"]["source_duplicate"]
    long_size, long_stride = settings["windows"]["long"]
    strict, informative = unit["strict"], unit["anchors"]["informative"]
    windows = passage_windows(len(strict), long_size, long_stride)
    if len(windows) < 2:
        return []
    matrix = vectorizer.transform([" ".join(strict[s:e]) for s, e in windows])
    similarity = (matrix @ matrix.T).toarray()
    minimum = min(th["informative_min"], math.ceil(th["short_unit_fraction"] * long_size))
    pairs = []
    for wi, (s1, e1) in enumerate(windows):
        partners = sorted((wj for wj in range(len(windows)) if windows[wj][0] >= e1),
                          key=lambda wj: similarity[wi][wj], reverse=True)
        for wj in partners[:th["top_k"]]:
            cosine = float(similarity[wi][wj])
            if cosine < th["cosine_min"]:
                break
            s2, e2 = windows[wj]
            alignment = align_tokens(strict[s1:e1], strict[s2:e2])
            if alignment["ordered_similarity"] < th["ordered_min"] or not alignment["chain"]:
                continue
            matched_informative = sum(1 for so, _, length in alignment["chain"]
                                      for k in range(length) if informative[s1 + so + k])
            if matched_informative < minimum:
                continue
            chain = alignment["chain"]
            first = (s1 + chain[0][0], s1 + chain[-1][0] + chain[-1][2])
            second = (s2 + chain[0][1], s2 + chain[-1][1] + chain[-1][2])
            pairs.append({"first": first, "second": second, "cosine": cosine,
                          "ordered_similarity": alignment["ordered_similarity"],
                          "matched_informative": matched_informative,
                          "offset": second[0] - first[0]})
    return merge_duplicate_blocks(pairs, unit, long_size)


def merge_duplicate_blocks(pairs, unit, long_size):
    """Group window pairs sharing a diagonal and overlapping ranges into single blocks."""
    if not pairs:
        return []
    pairs = sorted(pairs, key=lambda p: (p["first"][0], p["second"][0]))
    parent = list(range(len(pairs)))

    def find(node):
        while parent[node] != node:
            parent[node] = parent[parent[node]]
            node = parent[node]
        return node

    def near(first, second):  # Overlapping, or separated by at most one window.
        return max(first[0], second[0]) - min(first[1], second[1]) <= long_size

    for a in range(len(pairs)):
        for b in range(a + 1, len(pairs)):
            if (abs(pairs[a]["offset"] - pairs[b]["offset"]) <= long_size
                    and near(pairs[a]["first"], pairs[b]["first"])
                    and near(pairs[a]["second"], pairs[b]["second"])):
                parent[find(a)] = find(b)
    groups = {}
    for index in range(len(pairs)):
        groups.setdefault(find(index), []).append(pairs[index])
    blocks = []
    for members in groups.values():
        first = (min(p["first"][0] for p in members), max(p["first"][1] for p in members))
        second = (min(p["second"][0] for p in members), max(p["second"][1] for p in members))
        overlap = min(first[1], second[1]) - max(first[0], second[0])
        shorter = min(first[1] - first[0], second[1] - second[0])
        if shorter <= 0 or overlap > 0.5 * shorter:  # Occurrences must be distinct scans.
            continue
        blocks.append({"first_tokens": first, "second_tokens": second,
                       "offset": round(sum(p["offset"] for p in members) / len(members)),
                       "cosine": max(p["cosine"] for p in members),
                       "ordered_similarity": max(p["ordered_similarity"] for p in members),
                       "matched_informative": max(p["matched_informative"] for p in members),
                       "first_pages": token_pages(unit, *first),
                       "second_pages": token_pages(unit, *second)})
    return sorted(blocks, key=lambda block: block["first_tokens"])


def intervals_overlap(a0, a1, b0, b1):
    return a0 < b1 and b0 < a1


def anchor_reward(anchor):
    return anchor["length"] * (0.5 * anchor["cosine"] + 0.5 * anchor["ordered_similarity"])


def find_reader_occurrences(reader, source_unit, vectorizer, settings):
    """Retrieve and refine reader token windows against the unit's source windows.

    Each surviving anchor is a maximal exact-match run of at least the configured length
    with source/output token intervals and the retrieval cosine/ordered scores. Overlapping
    hits for one position collapse to the best-rewarded anchor; distinct occurrences survive."""
    th = settings["thresholds"]["retrieval"]
    local_size, local_stride = settings["windows"]["local"]
    reader_strict = [token["strict"] for token in reader]
    source_strict = source_unit["strict"]
    reader_windows = passage_windows(len(reader_strict), local_size, local_stride)
    source_windows = passage_windows(len(source_strict), local_size, local_stride)
    if not reader_windows or not source_windows:
        return []
    reader_matrix = vectorizer.transform([" ".join(reader_strict[s:e]) for s, e in reader_windows])
    source_matrix = vectorizer.transform([" ".join(source_strict[s:e]) for s, e in source_windows])
    similarity = reader_matrix @ source_matrix.T
    anchors = []
    for ri, (rs, re_) in enumerate(reader_windows):
        row = similarity.getrow(ri).toarray().ravel()
        for si in sorted(range(len(source_windows)), key=lambda si: row[si], reverse=True)[:th["top_k"]]:
            cosine = float(row[si])
            if cosine < th["cosine_min"]:
                break
            ss, se = source_windows[si]
            alignment = align_tokens(source_strict[ss:se], reader_strict[rs:re_])
            for s_off, r_off, length in alignment["chain"]:
                if length < th["anchor_min_tokens"]:
                    continue
                anchors.append({"source_start": ss + s_off, "source_end": ss + s_off + length,
                                "output_start": rs + r_off, "output_end": rs + r_off + length,
                                "length": length, "cosine": cosine,
                                "ordered_similarity": alignment["ordered_similarity"]})
    kept = []
    for anchor in sorted(anchors, key=lambda a: (-anchor_reward(a), -a["length"], a["output_start"], a["source_start"])):
        if not any(intervals_overlap(anchor["output_start"], anchor["output_end"], other["output_start"], other["output_end"])
                   and intervals_overlap(anchor["source_start"], anchor["source_end"], other["source_start"], other["source_end"])
                   for other in kept):
            kept.append(anchor)
    return sorted(kept, key=lambda a: (a["output_start"], a["source_start"]))


def canonical_position(position, aliases):
    """Fold a later scan-duplicate position onto its canonical first-occurrence position."""
    for alias in aliases:
        first0, _ = alias["first_tokens"]
        second0, second1 = alias["second_tokens"]
        if second0 <= position < second1:
            return first0 + (position - second0)
    return position


def canonical_interval(anchor, aliases):
    start = canonical_position(anchor["source_start"], aliases)
    end = canonical_position(anchor["source_end"], aliases)
    return (min(start, end), max(start, end))


def chain_matches(anchors, aliases, settings):
    """Chain reader anchors by increasing output position, allowing penalized restarts.

    Forward logical-source steps are cheap; a backward or repeated logical visit pays a
    restart cost. The maximum reward-minus-cost path is returned with its ordered anchors."""
    ch = settings["thresholds"]["chain"]
    nodes = sorted(anchors, key=lambda a: (a["output_start"], a["source_start"]))
    if not nodes:
        return []
    canon = [canonical_interval(node, aliases) for node in nodes]
    best = [anchor_reward(node) for node in nodes]
    previous = [-1] * len(nodes)
    for i in range(len(nodes)):
        for j in range(i):
            if nodes[j]["output_end"] > nodes[i]["output_start"]:
                continue
            output_gap = nodes[i]["output_start"] - nodes[j]["output_end"]
            source_gap = canon[i][0] - canon[j][1]
            if source_gap >= 0:
                cost = ch["forward_source"] * min(source_gap, ch["gap_cap"]) + ch["forward_output"] * min(output_gap, ch["gap_cap"])
            else:
                backward = canon[j][1] - canon[i][0]
                cost = (ch["restart_base"] + ch["restart_backward"] * min(backward / ch["backward_cap"], 4)
                        + ch["forward_output"] * min(output_gap, ch["gap_cap"]))
            value = best[j] + anchor_reward(nodes[i]) - cost
            if value > best[i]:
                best[i] = value
                previous[i] = j
    end = max(range(len(nodes)), key=lambda index: best[index])
    path = []
    while end != -1:
        path.append(nodes[end])
        end = previous[end]
    return list(reversed(path))


def summarize_anchors(anchors):
    """Collapse a set of anchors into one output/source/canonical contribution summary."""
    return {"output_start": min(a["output_start"] for a in anchors),
            "output_end": max(a["output_end"] for a in anchors),
            "source_start": min(a["source_start"] for a in anchors),
            "source_end": max(a["source_end"] for a in anchors),
            "canon_start": min(a["canon_start"] for a in anchors),
            "canon_end": max(a["canon_end"] for a in anchors),
            "cosine": max(a["cosine"] for a in anchors),
            "ordered_similarity": max(a["ordered_similarity"] for a in anchors),
            "matched": sum(a["length"] for a in anchors)}


def assign_occurrence_coverage(path, aliases, settings):
    """Segment a chained path into forward reading runs and flag repeated/reversed visits.

    A run breaks where the canonical source position jumps backward. Two runs overlapping the
    same logical source occurrence are a repeated visit; each contribution is restricted to the
    shared canonical span so it localizes the actual copy rather than the whole run. A later run
    starting earlier in the source is a reversal. Normal forward reading yields one run."""
    if not path:
        return {"runs": [], "duplicates": [], "orders": []}
    enriched = []
    for anchor in path:
        start, end = canonical_interval(anchor, aliases)
        enriched.append(dict(anchor, canon_start=start, canon_end=end))
    groups, current = [], [0]
    for k in range(1, len(enriched)):
        if enriched[k]["canon_start"] < enriched[k - 1]["canon_start"]:
            groups.append(current)
            current = [k]
        else:
            current.append(k)
    groups.append(current)
    runs = [{"anchors": [enriched[k] for k in group], **summarize_anchors([enriched[k] for k in group])}
            for group in groups]
    floor = settings["thresholds"]["repeat_min_tokens"]
    duplicates = []
    for i in range(len(runs)):
        for j in range(i + 1, len(runs)):
            low = max(runs[i]["canon_start"], runs[j]["canon_start"])
            high = min(runs[i]["canon_end"], runs[j]["canon_end"])
            shorter = min(runs[i]["canon_end"] - runs[i]["canon_start"],
                          runs[j]["canon_end"] - runs[j]["canon_start"])
            if shorter <= 0 or high - low < 0.5 * shorter:
                continue
            first = [a for a in runs[i]["anchors"] if a["canon_start"] < high and low < a["canon_end"]]
            second = [a for a in runs[j]["anchors"] if a["canon_start"] < high and low < a["canon_end"]]
            if not first or not second:
                continue
            ca, cb = summarize_anchors(first), summarize_anchors(second)
            if min(ca["matched"], cb["matched"]) >= floor:
                duplicates.append((ca, cb))
    summaries = [{key: run[key] for key in run if key != "anchors"} for run in runs]
    orders = [{"from": summaries[r - 1], "to": summaries[r]} for r in range(1, len(summaries))
              if summaries[r]["canon_start"] < summaries[r - 1]["canon_start"]
              and summaries[r]["matched"] >= floor]
    return {"runs": summaries, "duplicates": duplicates, "orders": orders}


def is_latin_token(strict):
    """A token carries Latin content when its casefolded form holds an ASCII letter.

    Greek marginal scraps, interlinear fragments and bare punctuation fail this, so they are
    never counted as missing Latin lines even though the Latin-only reader always drops them."""
    return any("a" <= character <= "z" for character in strict)


def deletion_runs(operations):
    """Group an alignment's operations into contiguous source-only deletion runs.

    Each run is (local_source_start, local_source_end, output_cursor) where the cursor is the
    output index at which the deleted source material would have been read. Covered diagonals,
    joins, splits and insertions reset the run and advance the cursor; a true omission appears
    as one deletion run because its tokens are absent from the output entirely."""
    runs, current, cursor = [], None, 0
    for step in operations:
        if step["op"] == "delete":
            index = step["source"][0]
            current = (current[0], index + 1, current[2]) if current and index == current[1] else (index, index + 1, cursor)
        else:
            if current:
                runs.append(current)
                current = None
            if step["output"]:
                cursor = step["output"][-1] + 1
    if current:
        runs.append(current)
    return runs


def find_missing_occurrences(reader, source_unit, anchors, path, settings):
    """Nominate expected source spans absent at their context-correct output seam.

    Consecutive chained anchors bound a gap where the reader drifts from the source through OCR
    noise. A bounded local alignment of that whole gap reveals a true omission as a contiguous
    run of deleted source tokens flanked by covered matches, isolating it from scattered noise.
    A copy of those tokens matched elsewhere by any anchor is a displacement (order evidence),
    so only a uniquely-lost span counts as a missing occurrence; weak or one-sided context, or a
    gap too large to align within the matrix cap, yields low/unknown confidence."""
    om = settings["thresholds"]["omission"]
    cap = settings["thresholds"].get("matrix_cap", MATRIX_CAP)
    window = om["context_window"]
    informative = source_unit["anchors"]["informative"]
    source_views = [{"strict": s, "tolerant": t}
                    for s, t in zip(source_unit["strict"], source_unit["tolerant"])]
    reader_views = [{"strict": token["strict"], "tolerant": token["tolerant"]} for token in reader]
    tokens = source_unit["tokens"]
    line_sizes, line_latin = {}, {}
    for index, token in enumerate(tokens):
        key = (token["page"], token["line"])
        line_sizes[key] = line_sizes.get(key, 0) + 1
        line_latin[key] = line_latin.get(key, 0) + (1 if is_latin_token(source_unit["strict"][index]) else 0)
    latin_lines = {key for key, size in line_sizes.items()
                   if line_latin[key] >= om["latin_line_fraction"] * size}
    omissions = []
    for left, right in zip(path, path[1:]):
        s0, s1 = left["source_end"], right["source_start"]
        o0, o1 = left["output_end"], right["output_start"]
        if s1 - s0 < om["min_missing_tokens"] or o1 < o0:
            continue
        if s1 - s0 > cap or o1 - o0 > cap:
            continue  # Oversized divergence: alignment is inconclusive rather than truncated.
        local = align_tokens(source_views[s0:s1], reader_views[o0:o1], mode="local", settings=settings)
        if local.get("inconclusive"):
            continue
        covered = {s0 + index for index in local["covered_source"]}
        covered |= set(range(left["source_start"], left["source_end"]))
        covered |= set(range(right["source_start"], right["source_end"]))
        for local_start, local_end, cursor in deletion_runs(local["operations"]):
            if local_end - local_start < om["min_missing_tokens"]:
                continue
            abs_start, abs_end = s0 + local_start, s0 + local_end
            deleted_per_line = {}
            for i in range(abs_start, abs_end):
                key = (tokens[i]["page"], tokens[i]["line"])
                deleted_per_line[key] = deleted_per_line.get(key, 0) + 1
            full_lines = {key for key, count in deleted_per_line.items()
                          if key in latin_lines and count >= om["line_fraction"] * line_sizes[key]}
            if len(full_lines) < om["min_missing_lines"]:
                continue  # Only whole absent Latin lines are omissions; dropped words/Greek are not.
            missing_tokens = sorted(i for i in range(abs_start, abs_end)
                                    if (tokens[i]["page"], tokens[i]["line"]) in full_lines
                                    and is_latin_token(source_unit["strict"][i]))
            if len(missing_tokens) < om["min_missing_tokens"]:
                continue
            left_info = sum(1 for i in range(max(0, abs_start - window), abs_start)
                            if i in covered and informative[i])
            right_info = sum(1 for i in range(abs_end, min(len(informative), abs_end + window))
                             if i in covered and informative[i])
            left_cover = sum(1 for i in range(max(0, abs_start - window), abs_start) if i in covered)
            right_cover = sum(1 for i in range(abs_end, min(len(informative), abs_end + window)) if i in covered)
            context_ratio = (left_cover + right_cover) / (2 * window)
            seam = min(max(o0 + cursor, o0), o1)
            displaced = any(intervals_overlap(an["source_start"], an["source_end"], abs_start, abs_end)
                            for an in anchors)
            unique_loss = not displaced
            strong = (left_info >= om["context_min_informative"]
                      and right_info >= om["context_min_informative"] and unique_loss)
            confidence = "high" if strong else ("low" if unique_loss else "unknown")
            omissions.append({"gap": missing_tokens, "seam_output": (seam, seam),
                              "missing_line_count": len(full_lines), "unique_loss": unique_loss,
                              "confidence": confidence, "ordered": round(context_ratio, 4),
                              "left_informative": left_info, "right_informative": right_info,
                              "displaced": displaced})
    return omissions


def image_views(gray):
    """Keep faint grayscale ink and a local-contrast view as separately scored observations."""
    import cv2
    return {"grayscale": gray, "contrast": cv2.createCLAHE(clipLimit=2.0, tileGridSize=(8, 8)).apply(gray)}


def ink_line_height(gray):
    """Estimate character height from the same dark-ink components in source and reader.

    The upper quartile excludes small punctuation and interlinear scraps; long rules
    and page-spanning components do not supply the typography scale.
    """
    import cv2
    import numpy as np
    _, _, stats, _ = cv2.connectedComponentsWithStats((gray < 120).astype(np.uint8), 8)
    heights = sorted(int(s[3]) for s in stats[1:] if
                     max(4, 0.002 * gray.shape[0]) <= s[3] <= 0.06 * gray.shape[0]
                     and 2 <= s[2] <= 3 * s[3] and s[4] >= 5)
    return float(heights[3 * len(heights) // 4]) if len(heights) >= 8 else None


def make_templates(spec, source_regions, root, manifest, work, repo, settings, pdftoppm):
    """Derive full, word-group and partial-height templates from reviewed source logos.

    Every fragment retains its original source crop and pixel offset. No reader image,
    label, candidate score or expected output location contributes to template pixels.
    """
    import cv2
    import numpy as np
    from PIL import Image
    th = settings["thresholds"]["watermark"]
    templates = []
    for entry in spec["templates"]:
        if entry["annotation_status"] != "reviewed":
            raise AuditError("Watermark templates require reviewed original-source regions")
        for region in entry["source_regions"]:
            pdf, record = snapshot_input(root, manifest, role="source_pdf", source_index=region["source_index"])
            if region["input_sha256"] != record["sha256"]:
                raise AuditError("Watermark template source hash differs from captured PDF")
            raster, meta = render_page(pdf, record, region["page"], 300, work, repo, pdftoppm=pdftoppm)
            with Image.open(raster) as image:
                full = np.asarray(image.convert("L"))
            bounds = pixel_bounds(region["box"], meta["raster_dimensions"])
            gray = full[bounds[1]:bounds[3], bounds[0]:bounds[2]].copy()
            ink = gray < 248
            if float(gray.std()) < th["template_std_min"] or float(ink.mean()) < th["ink_fraction_min"]:
                raise AuditError(f"Constant or near-empty watermark template: {entry['template_id']}")
            ys, xs = np.where(ink)
            tight = [max(0, int(xs.min()) - 2), max(0, int(ys.min()) - 2),
                     min(gray.shape[1], int(xs.max()) + 3), min(gray.shape[0], int(ys.max()) + 3)]
            mask = (ink * 255).astype(np.uint8)
            _, _, components, _ = cv2.connectedComponentsWithStats(mask, 8)
            maximum_height = max((int(c[3]) for c in components[1:]), default=0)
            letters = [c for c in components[1:] if c[3] >= 0.45 * maximum_height and c[4] >= 8]
            letters = sorted(letters, key=lambda c: int(c[0]))
            fragments = [("full_logo", tight, max(len(letters), 2))]
            if len(letters) >= 3:
                word = [min(int(c[0]) for c in letters), min(int(c[1]) for c in letters),
                        max(int(c[0] + c[2]) for c in letters), max(int(c[1] + c[3]) for c in letters)]
                fragments.append(("logo_word", word, len(letters)))
                for name, group in (("left_glyph_group", letters[:2]), ("right_glyph_group", letters[-3:])):
                    box = [min(int(c[0]) for c in group), min(int(c[1]) for c in group),
                           max(int(c[0] + c[2]) for c in group), max(int(c[1] + c[3]) for c in group)]
                    fragments.append((name, box, len(group)))
                midpoint = (word[1] + word[3]) // 2
                fragments.extend([("upper_logo", [word[0], word[1], word[2], midpoint], len(letters)),
                                  ("lower_logo", [word[0], midpoint, word[2], word[3]], len(letters))])
                for index, (top, bottom) in enumerate(th["partial_bands"]):
                    y0 = word[1] + round(top * (word[3] - word[1]))
                    y1 = word[1] + round(bottom * (word[3] - word[1]))
                    support = sum(int(c[1]) < y1 and y0 < int(c[1] + c[3]) for c in letters)
                    fragments.append((f"logo_band_{index}", [word[0], y0, word[2], y1], support))
            source_heights = [line["box"][3] - line["box"][1]
                              for unit in source_regions["units"] if unit["source"] == region["source_index"]
                              for page in unit["pages"] if page["page"] == region["page"]
                              for line in page.get("lines", []) if len(line["text"].split()) >= 4]
            source_height = (sorted(source_heights)[len(source_heights) // 2] * meta["raster_dimensions"][1]
                             if source_heights else None)
            column = pixel_bounds([0.52, 0.10, 0.97, 0.90], meta["raster_dimensions"])
            source_height = ink_line_height(full[column[1]:column[3], column[0]:column[2]]) or source_height
            for name, box, support in fragments:
                pixels = gray[box[1]:box[3], box[0]:box[2]].copy()
                if min(pixels.shape) < 4 or float(pixels.std()) < th["template_std_min"]:
                    continue
                source_box = [(bounds[0] + box[0]) / full.shape[1], (bounds[1] + box[1]) / full.shape[0],
                              (bounds[0] + box[2]) / full.shape[1], (bounds[1] + box[3]) / full.shape[0]]
                templates.append({"id": f"{entry['template_id']}_{name}", "family": entry["template_id"],
                                  "fragment_type": name, "pixels": pixels, "views": image_views(pixels),
                                  "glyph_support": support, "source_line_height": source_height,
                                  "offset": box[:2], "full_dimensions": [gray.shape[1], gray.shape[0]],
                                  "glyph_boxes": [[max(int(c[0]), box[0]) - box[0],
                                                   max(int(c[1]), box[1]) - box[1],
                                                   min(int(c[0] + c[2]), box[2]) - box[0],
                                                   min(int(c[1] + c[3]), box[3]) - box[1]]
                                                  for c in letters if int(c[0]) < box[2] and box[0] < int(c[0] + c[2])
                                                  and int(c[1]) < box[3] and box[1] < int(c[1] + c[3])],
                                  "source_region": dict(region, box=source_box),
                                  "pixels_sha256": hashlib.sha256(pixels.tobytes()).hexdigest()})
    progress(f"Prepared {len(templates)} original-source watermark fragments")
    return templates


def group_peaks(peaks, distance):
    """Group overlapping image peaks without dropping lower-confidence nominations."""
    groups = []
    for peak in sorted(peaks, key=lambda p: (-p["score"], p["template_id"], p["box"])):
        x, y = peak["origin"]
        group = next((g for g in groups if any(
            math.hypot(x - p["origin"][0], y - p["origin"][1]) <= distance for p in g)), None)
        if group is None:
            groups.append([peak])
        else:
            group.append(peak)
    return groups


def match_templates(gray, templates, line_height, settings, *, scales=None, rotations=None, kind="watermark"):
    """Search nonconstant source templates with normalized correlation in two views.

    Returned values are similarity scores, never probabilities. Each scale/rotation/view
    search contributes nonoverlapping local maxima; peak grouping preserves components.
    """
    import cv2
    import numpy as np
    th = settings["thresholds"]["watermark"]
    if kind == "head":
        th = dict(th, **settings["thresholds"]["head"])
    elif kind != "watermark":
        raise AuditError("Image matching kind must be watermark or head")
    if not gray.size or float(gray.std()) < th["template_std_min"]:
        return []
    views = image_views(gray)
    faint_mask = ((gray >= th["faint_min_gray"]) & (gray < 250)).astype(np.float32)
    peaks = []
    results = {}
    for template in templates:
        ratio = line_height / template["source_line_height"] if line_height and template["source_line_height"] else None
        bank = scales if scales is not None else (
            [ratio * float(s) for s in np.arange(th["scale_min"], th["scale_max"] + 0.001, th["scale_step"])]
            if ratio else [float(s) for s in np.arange(th["fallback_min"], th["fallback_max"] + 0.001, th["scale_step"])])
        angles = rotations if rotations is not None else range(th["rotation_min"], th["rotation_max"] + 1)
        scheduled = set()
        for scale in bank:
            width = round(template["pixels"].shape[1] * scale)
            height = round(template["pixels"].shape[0] * scale)
            if min(width, height) < 4 or width > gray.shape[1] or height > gray.shape[0]:
                continue
            for angle in angles:
                if (width, height, angle) in scheduled:
                    continue
                scheduled.add((width, height, angle))
                key = (template["pixels_sha256"], width, height, angle)
                if key not in results:
                    found = []
                    transform = cv2.getRotationMatrix2D((width / 2, height / 2), angle, 1.0)
                    raw_needle = cv2.resize(template["pixels"], (width, height), interpolation=cv2.INTER_AREA)
                    raw_needle = cv2.warpAffine(raw_needle, transform, (width, height), borderValue=255)
                    expected_ink = raw_needle < 245
                    support = cv2.matchTemplate(faint_mask, expected_ink.astype(np.float32), cv2.TM_CCORR)
                    support /= max(int(expected_ink.sum()), 1)
                    for view, search in views.items():
                        needle = cv2.resize(template["views"][view], (width, height), interpolation=cv2.INTER_AREA)
                        needle = cv2.warpAffine(needle, transform, (width, height), borderValue=255)
                        if float(needle.std()) < th["template_std_min"]:
                            continue
                        scores = cv2.matchTemplate(search, needle, cv2.TM_CCOEFF_NORMED)
                        scores[~np.isfinite(scores)] = -1
                        scores[support < th["faint_ink_fraction_min"]] = -1
                        while True:
                            _, score, _, location = cv2.minMaxLoc(scores)
                            if score < th["nomination_min"]:
                                break
                            x, y = location
                            glyph_support = 0
                            for glyph in template["glyph_boxes"]:
                                gx0, gx1 = round(glyph[0] * width / template["pixels"].shape[1]), round(glyph[2] * width / template["pixels"].shape[1])
                                gy0, gy1 = round(glyph[1] * height / template["pixels"].shape[0]), round(glyph[3] * height / template["pixels"].shape[0])
                                ink = expected_ink[gy0:gy1, gx0:gx1]
                                values = gray[y + gy0:y + gy1, x + gx0:x + gx1][ink]
                                if values.size >= 3 and float(np.mean((values >= th["faint_min_gray"]) & (values < 250))) >= th["faint_ink_fraction_min"]:
                                    glyph_support += 1
                            found.append({"view": view, "score": float(score), "rotation": angle,
                                          "box": [x, y, x + width, y + height],
                                          "observed_glyph_support": glyph_support,
                                          "faint_ink_fraction": float(support[y, x])})
                            x0, x1 = max(0, x - width // 2), min(scores.shape[1], x + width // 2 + 1)
                            y0, y1 = max(0, y - height // 2), min(scores.shape[0], y + height // 2 + 1)
                            scores[y0:y1, x0:x1] = -1
                    results[key] = found
                for peak in results[key]:
                    peaks.append(dict(peak, template_id=template["id"], family=template["family"],
                                      fragment_type=template["fragment_type"], scale=float(scale),
                                      glyph_support=template["glyph_support"],
                                      origin=[peak["box"][0] - scale * template["offset"][0],
                                              peak["box"][1] - scale * template["offset"][1]]))
    return group_peaks(peaks, max(line_height or 8, 4))


def watermark_pages(rows, reader, templates, root, manifest, work, repo, settings, pdftoppm, workers):
    """Prepare PDFs on the main thread and schedule each pure image search once.

    Worker threads receive NumPy arrays and comparison data, never PDF objects.
    Saved matches bind pixels, template identities, settings and helper code.
    """
    from concurrent.futures import ThreadPoolExecutor
    import cv2
    import inspect
    import numpy as np
    from PIL import Image
    th = settings["thresholds"]["watermark"]
    code_hash = hashlib.sha256("".join(inspect.getsource(f) for f in
        (match_templates, group_peaks, image_views, ink_line_height)).encode()).hexdigest()
    template_identity = [{k: v for k, v in t.items() if k not in ("pixels", "views")} for t in templates]
    cv2.setNumThreads(1)
    with ThreadPoolExecutor(max_workers=workers) as executor:
        scheduled = []
        for row in rows:
            pdf, record = snapshot_input(root, manifest, role="reader_pdf", tomus=row["tomus"])
            raster, meta = render_page(pdf, record, row["page"], th["dpi"], work, repo, pdftoppm=pdftoppm)
            bounds = pixel_bounds(row["body_box"], meta["raster_dimensions"])
            with Image.open(raster) as image:
                gray = np.asarray(image.convert("L"))[bounds[1]:bounds[3], bounds[0]:bounds[2]].copy()
            heights = sorted(t["box"][3] - t["box"][1] for t in reader if t["page"] == row["page"])
            pitch = ink_line_height(gray) or (heights[len(heights) // 2] * meta["raster_dimensions"][1] if heights else None)
            metadata = {"raster_sha256": meta["raster_sha256"], "body_box": row["body_box"],
                        "settings": th, "templates": template_identity, "line_height": pitch,
                        "opencv_version": cv2.__version__, "helper_sha256": code_hash}
            cache = Path(work) / "cache" / "watermark_matches" / f"{json_hash(metadata)}.json"
            reused = reusable_entry(cache, metadata, work, repo)
            future = None if reused is not None else executor.submit(match_templates, gray, templates, pitch, settings)
            scheduled.append((row, meta, bounds, pitch, cache, metadata, reused, future))
        for row, meta, bounds, pitch, cache, metadata, reused, future in scheduled:
            groups = reused["groups"] if reused is not None else future.result()
            if reused is None:
                write_json(cache, {"schema_version": 1, "complete": True, "metadata": metadata,
                                   "data": {"groups": groups}}, work, repo)
            yield row, meta, bounds, pitch, groups


def source_context_at_region(reader, path, source_unit, page, box):
    """Locate the nearest occurrence-assigned prose on either side of an image region."""
    before = max((i for i, token in enumerate(reader) if token["page"] < page or
                  (token["page"] == page and token["box"][3] <= box[1])), default=-1)
    after = min((i for i, token in enumerate(reader) if token["page"] > page or
                 (token["page"] == page and token["box"][1] >= box[3])), default=len(reader))
    left, right = None, None
    for anchor in path:
        preceding = min(before, anchor["output_end"] - 1)
        following = max(after, anchor["output_start"])
        if preceding >= anchor["output_start"]:
            index = preceding
            if left is None or index > left[0]:
                left = (index, anchor["source_start"] + index - anchor["output_start"])
        if following < anchor["output_end"]:
            index = following
            if right is None or index < right[0]:
                right = (index, anchor["source_start"] + index - anchor["output_start"])
    return {side: {"output_token": pair[0], "source_token": pair[1],
                   "source_page": source_unit["tokens"][pair[1]]["page"]} if pair else None
            for side, pair in (("before", left), ("after", right))}


def source_tail_correspondence(context, source_unit, templates, root, manifest, work, repo, settings, pdftoppm):
    """Corroborate a source footer at an occurrence-aligned transition between prose pages."""
    import numpy as np
    from PIL import Image
    before, after = context["before"], context["after"]
    if not before or not after or before["source_page"] >= after["source_page"]:
        return None
    pdf, record = snapshot_input(root, manifest, role="source_pdf", source_index=source_unit["source_index"])
    raster, meta = render_page(pdf, record, before["source_page"], 300, work, repo, pdftoppm=pdftoppm)
    bounds = pixel_bounds([SOURCE_LATIN_X0, 0.93, 1.0, 1.0], meta["raster_dimensions"])
    with Image.open(raster) as image:
        gray = np.asarray(image.convert("L"))[bounds[1]:bounds[3], bounds[0]:bounds[2]].copy()
    matches = match_templates(gray, templates, None, settings, scales=[1.0], rotations=[0])
    peaks = [p for group in matches for p in group if p["fragment_type"] in ("full_logo", "logo_word")
             and p["score"] >= settings["thresholds"]["watermark"]["high_min"]]
    if not peaks:
        return None
    peak = max(peaks, key=lambda p: (p["score"], p["fragment_type"] == "full_logo"))
    region = {"role": "source_pdf", "source_index": source_unit["source_index"],
              "page": before["source_page"], "input_sha256": record["sha256"],
              "box": [(bounds[0] + peak["box"][0]) / meta["raster_dimensions"][0],
                      (bounds[1] + peak["box"][1]) / meta["raster_dimensions"][1],
                      (bounds[0] + peak["box"][2]) / meta["raster_dimensions"][0],
                      (bounds[1] + peak["box"][3]) / meta["raster_dimensions"][1]]}
    return {"region": region, "correlation": peak["score"], "context": context}


def find_watermarks(rows, reader, source_unit, path, templates, root, manifest, run, work, repo,
                    settings, pdftoppm, workers=1):
    """Search actual body image occurrences, refining nominated regions at 300 DPI."""
    import numpy as np
    from PIL import Image
    th = settings["thresholds"]["watermark"]
    template_by_id = {t["id"]: t for t in templates}
    candidates = []
    write_json(run / "watermark-templates.json", {"schema_version": 1,
        "templates": [{k: v for k, v in t.items() if k not in ("pixels", "views")} for t in templates]}, work, repo)
    for row, meta, bounds, pitch, groups in watermark_pages(
            rows, reader, templates, root, manifest, work, repo, settings, pdftoppm, workers):
        pdf, record = snapshot_input(root, manifest, role="reader_pdf", tomus=row["tomus"])
        for number, group in enumerate(groups):
            coarse = max(group, key=lambda p: p["score"])
            box = union_box([[(bounds[0] + p["box"][0]) / meta["raster_dimensions"][0],
                              (bounds[1] + p["box"][1]) / meta["raster_dimensions"][1],
                              (bounds[0] + p["box"][2]) / meta["raster_dimensions"][0],
                              (bounds[1] + p["box"][3]) / meta["raster_dimensions"][1]] for p in group])
            padding = max((pitch or 8) / meta["raster_dimensions"][1], 0.01)
            retry_box = [max(row["body_box"][0], box[0] - padding), max(row["body_box"][1], box[1] - padding),
                         min(row["body_box"][2], box[2] + padding), min(row["body_box"][3], box[3] + padding)]
            full_raster, full_meta = render_page(pdf, record, row["page"], 300, work, repo, pdftoppm=pdftoppm)
            retry_bounds = pixel_bounds(retry_box, full_meta["raster_dimensions"])
            with Image.open(full_raster) as image:
                retry_gray = np.asarray(image.convert("L"))[retry_bounds[1]:retry_bounds[3], retry_bounds[0]:retry_bounds[2]].copy()
            involved = [t for t in templates if t["family"] in {p["family"] for p in group}]
            scale = coarse["scale"] * 300 / th["dpi"]
            refined = match_templates(retry_gray, involved, (pitch * 300 / th["dpi"]) if pitch else None,
                                      settings, scales=[scale * s for s in (0.95, 1.0, 1.05)],
                                      rotations=range(max(-3, coarse["rotation"] - 1), min(3, coarse["rotation"] + 1) + 1))
            refined_peaks = [p for g in refined for p in g]
            strong = [p for p in refined_peaks if p["score"] >= th["high_min"]]
            stable = any(p["template_id"] == q["template_id"] and p["view"] != q["view"]
                         and math.hypot(p["box"][0] - q["box"][0], p["box"][1] - q["box"][1])
                         <= (pitch or 8) * 300 / th["dpi"] for p in strong for q in strong)
            distinct = {p["fragment_type"] for p in strong}
            distinctive = {"left_glyph_group", "right_glyph_group"} <= distinct
            supported = any(p["observed_glyph_support"] >= th["glyph_support_min"] for p in strong)
            context = source_context_at_region(reader, path, source_unit, row["page"], box)
            tail = source_tail_correspondence(context, source_unit, templates, root, manifest,
                                              work, repo, settings, pdftoppm)
            source_tail = tail is not None
            high = stable and supported and (distinctive or source_tail)
            event = f"watermark_{row['unit_id']}_{row['page']}_{number}"
            output_region = {"role": "reader_pdf", "tomus": row["tomus"], "page": row["page"],
                             "box": box, "input_sha256": record["sha256"]}
            template_region = template_by_id[coarse["template_id"]]["source_region"]
            source_regions = [tail["region"]] if tail else []
            evidence = []
            for i, region in enumerate(source_regions + [template_region, output_region]):
                evidence.append(save_region_crop(root, manifest, region, run, work, repo, 300, f"{event}_{i}.png"))
            candidates.append({"event_id": event, "unit_id": row["unit_id"], "type": "watermark",
                "relation": "source_watermark", "source_regions": source_regions, "output_regions": [output_region],
                "primary_score": max((p["score"] for p in refined_peaks), default=coarse["score"]),
                "component_scores": {"coarse_correlation": coarse["score"], "stable_views": stable,
                                     "distinctive_fragments": distinctive, "source_tail_match": source_tail,
                                     "two_glyph_support": supported,
                                     "reader_line_height": pitch / meta["raster_dimensions"][1] if pitch else None},
                "confidence": "high" if high else "low", "status": "candidate" if refined_peaks else "inconclusive",
                "confidence_reasons": [f"{name}: {'satisfied' if value else 'failed'}" for name, value in
                                       (("stable grayscale/contrast location", stable), ("two-glyph support", supported),
                                        ("second distinct fragment or matched source tail", distinctive or source_tail))],
                "evidence_provenance": {"method": "original-source normalized-correlation template matching",
                                        "coarse_peaks": group, "refined_peaks": refined_peaks,
                                        "source_context": context, "source_tail": tail,
                                        "template_exemplar_region": template_region,
                                        "template_pixels_sha256": template_by_id[coarse["template_id"]]["pixels_sha256"]},
                "linked_events": [], "crops": evidence})
        progress(f"Watermark search {row['unit_id']} page {row['page']}: {len(groups)} grouped nominations")
    return merge_logo_events(candidates, rows, root, manifest, run, work, repo, templates, settings, pdftoppm)


def join_body_edges(first, second, root, manifest, run, work, repo, settings, pdftoppm, name):
    """Virtually join actual same-unit bottom/top image strips, preserving both coordinates."""
    import numpy as np
    from PIL import Image
    if (first["unit_id"] != second["unit_id"] or first["tomus"] != second["tomus"]
            or second["page"] != first["page"] + 1 or second["ordinal"] != first["ordinal"] + 1):
        raise AuditError("Diagnostic edge joins require adjacent body images of the same unit")
    strips, mappings, heights = [], [], []
    for row, edge in ((first, "bottom"), (second, "top")):
        pdf, record = snapshot_input(root, manifest, role="reader_pdf", tomus=row["tomus"])
        raster, meta = render_page(pdf, record, row["page"], 300, work, repo, pdftoppm=pdftoppm)
        bounds = pixel_bounds(row["body_box"], meta["raster_dimensions"])
        with Image.open(raster) as image:
            gray = np.asarray(image.convert("L"))
        body = gray[bounds[1]:bounds[3], bounds[0]:bounds[2]]
        height = ink_line_height(body) or max(8, 0.01 * gray.shape[0])
        count = min(body.shape[0], math.ceil(settings["thresholds"]["watermark"]["edge_strip_line_heights"] * height))
        y0 = bounds[3] - count if edge == "bottom" else bounds[1]
        strip = gray[y0:y0 + count, bounds[0]:bounds[2]].copy()
        strips.append(strip)
        heights.append(height)
        mappings.append({"tomus": row["tomus"], "page": row["page"], "edge": edge,
                         "input_sha256": record["sha256"], "raster_dimensions": meta["raster_dimensions"],
                         "pixel_bounds": [bounds[0], y0, bounds[2], y0 + count],
                         "virtual_top": 0 if edge == "bottom" else strips[0].shape[0]})
    canvas = np.full((sum(s.shape[0] for s in strips), max(s.shape[1] for s in strips)), 255, dtype=np.uint8)
    for strip, mapping in zip(strips, mappings):
        top = mapping["virtual_top"]
        canvas[top:top + strip.shape[0], :strip.shape[1]] = strip
    target = safe_write_path(run / "crops" / name, work, repo)
    target.parent.mkdir(parents=True, exist_ok=True)
    Image.fromarray(canvas).save(target)
    return canvas, mappings, sum(heights) / len(heights), {
        "path": target.relative_to(Path(work).resolve()).as_posix(), "sha256": hash_file(target)}


def merge_logo_events(candidates, rows, root, manifest, run, work, repo, templates, settings, pdftoppm):
    """Group overlapping components and corroborated edge fragments into logical logo events.

    All component records and their failed confidence predicates remain in the evidence.
    A merged event inherits confidence from its strongest supported component; a virtual
    stitch adds image evidence without repairing either reader page.
    """
    if not candidates:
        return []
    th = settings["thresholds"]["watermark"]
    row_by_page = {(r["tomus"], r["page"]): r for r in rows}
    parents = list(range(len(candidates)))

    def root_index(index):
        while parents[index] != index:
            parents[index] = parents[parents[index]]
            index = parents[index]
        return index

    for i, first in enumerate(candidates):
        a = first["output_regions"][0]
        a_sources = {(r["source_index"], r["page"]) for r in first["source_regions"]}
        for j in range(i + 1, len(candidates)):
            second = candidates[j]
            b = second["output_regions"][0]
            if first["unit_id"] != second["unit_id"] or a["tomus"] != b["tomus"] or abs(a["page"] - b["page"]) > 1:
                continue
            b_sources = {(r["source_index"], r["page"]) for r in second["source_regions"]}
            if a_sources and b_sources and not a_sources & b_sources:
                continue
            same_page = a["page"] == b["page"]
            overlap = (min(a["box"][2], b["box"][2]) > max(a["box"][0], b["box"][0])
                       and min(a["box"][3], b["box"][3]) > max(a["box"][1], b["box"][1]))
            edges = False
            if not same_page and a_sources & b_sources:
                low, high = sorted((a, b), key=lambda r: r["page"])
                low_row = row_by_page[(low["tomus"], low["page"])]
                high_row = row_by_page[(high["tomus"], high["page"])]
                line_heights = [c["component_scores"].get("reader_line_height") for c in (first, second)]
                tolerance = th["event_edge_line_heights"] * max((h for h in line_heights if h), default=0.0)
                edges = (high_row["ordinal"] == low_row["ordinal"] + 1
                         and low["box"][3] >= low_row["body_box"][3] - tolerance
                         and high["box"][1] <= high_row["body_box"][1] + tolerance
                         and min(low["box"][2], high["box"][2]) > max(low["box"][0], high["box"][0]))
            if (same_page and overlap) or edges:
                parents[root_index(j)] = root_index(i)
    groups = {}
    for index, candidate in enumerate(candidates):
        groups.setdefault(root_index(index), []).append(candidate)
    merged = []
    ranking = {"unknown": 0, "low": 1, "high": 2}
    for group in groups.values():
        if len(group) == 1:
            merged.append(group[0])
            continue
        winner = max(group, key=lambda c: (ranking[c["confidence"]], c["status"] == "candidate", c["primary_score"], c["event_id"]))
        event = dict(winner)
        component_ids = sorted(c["event_id"] for c in group)
        suffix = hashlib.sha256("|".join(component_ids).encode()).hexdigest()[:12]
        event["event_id"] = f"watermark_{winner['unit_id']}_{suffix}"
        event["linked_events"] = component_ids
        event["status"] = "candidate" if any(c["status"] == "candidate" for c in group) else "inconclusive"
        event["primary_score"] = max(c["primary_score"] for c in group)
        for field in ("source_regions", "output_regions"):
            regions = {}
            for component in group:
                for region in component[field]:
                    key = (region["role"], region.get("tomus", region.get("source_index")), region["page"])
                    regions.setdefault(key, []).append(region)
            event[field] = [dict(items[0], box=union_box([r["box"] for r in items])) for items in regions.values()]
        event["crops"] = list({crop["path"]: crop for c in group for crop in c["crops"]}.values())
        event["component_scores"] = dict(winner["component_scores"], component_event_count=len(group))
        event["evidence_provenance"] = {"method": "overlapping and source-corresponding same-unit edge logo components",
                                       "components": group, "virtual_stitches": []}
        output = sorted(event["output_regions"], key=lambda r: r["page"])
        for first, second in zip(output, output[1:]):
            if second["page"] != first["page"] + 1:
                continue
            a, b = row_by_page[(first["tomus"], first["page"])], row_by_page[(second["tomus"], second["page"])]
            canvas, mappings, height, crop = join_body_edges(a, b, root, manifest, run, work, repo,
                settings, pdftoppm, f"{event['event_id']}_{first['page']}_{second['page']}_virtual.png")
            stitched = match_templates(canvas, templates, height, settings)
            peaks = [p for g in stitched for p in g]
            event["crops"].append(crop)
            event["evidence_provenance"]["virtual_stitches"].append({"mappings": mappings,
                "peaks": peaks, "crosses_join": any(p["box"][1] < mappings[1]["virtual_top"] < p["box"][3] for p in peaks),
                "diagnostic_only": True})
        merged.append(event)
    return sorted(merged, key=lambda c: c["event_id"])


def pdf_word_observations(page):
    """Preserve original embedded words in the common displayed MediaBox coordinate frame."""
    return [{"raw": raw, "box": normalized_pdf_box(page, [x0, y0, x1, y1]),
             "provider": "embedded_source_words", "confidence": None}
            for x0, y0, x1, y1, raw, *_ in page.get_text("words") if raw.strip()]


def match_source_image(row, source_index, source_page, root, manifest, work, repo, settings, pdftoppm):
    """Register retained prose pixels to an original source page without using crop policy."""
    import cv2
    import numpy as np
    from PIL import Image
    th = settings["thresholds"]["registration"]
    reader_pdf, reader_record = snapshot_input(root, manifest, role="reader_pdf", tomus=row["tomus"])
    source_pdf, source_record = snapshot_input(root, manifest, role="source_pdf", source_index=source_index)
    reader_raster, reader_meta = render_page(reader_pdf, reader_record, row["page"], th["dpi"], work, repo, pdftoppm=pdftoppm)
    source_raster, source_meta = render_page(source_pdf, source_record, source_page, th["dpi"], work, repo, pdftoppm=pdftoppm)
    metadata = {"reader_render": reader_meta, "source_render": source_meta,
                "body_box": row["body_box"], "settings": th, "adapter": "source-image-affine-v1", "opencv": cv2.__version__}
    cache = Path(work) / "cache" / "image_registration" / f"{json_hash(metadata)}.json"
    reused = reusable_entry(cache, metadata, work, repo)
    if reused is not None:
        return reused
    with Image.open(reader_raster) as image:
        target = np.asarray(image.convert("L"))
    with Image.open(source_raster) as image:
        source = np.asarray(image.convert("L"))
    rb = pixel_bounds(row["body_box"], reader_meta["raster_dimensions"])
    sb = pixel_bounds([SOURCE_LATIN_X0, 0.10, 1.0, 0.94], source_meta["raster_dimensions"])
    sift = cv2.SIFT_create(nfeatures=th["features"])
    sk, sd = sift.detectAndCompute(source[sb[1]:sb[3], sb[0]:sb[2]], None)
    rk, rd = sift.detectAndCompute(target[rb[1]:rb[3], rb[0]:rb[2]], None)
    result = {"matched": False, "reason": "insufficient source/output image correspondences", "matrix": None,
              "source_dimensions": source_meta["raster_dimensions"], "reader_dimensions": reader_meta["raster_dimensions"],
              "source_page": source_page, "source_index": source_index, "reader_page": row["page"], "tomus": row["tomus"]}
    if sd is not None and rd is not None and len(sk) >= 8 and len(rk) >= 8:
        pairs = cv2.BFMatcher(cv2.NORM_L2).knnMatch(sd, rd, k=2)
        good = [p[0] for p in pairs if len(p) == 2 and p[0].distance < th["ratio"] * p[1].distance]
        if len(good) >= th["minimum_inliers"]:
            first = np.float32([[sk[p.queryIdx].pt[0] + sb[0], sk[p.queryIdx].pt[1] + sb[1]] for p in good])
            second = np.float32([[rk[p.trainIdx].pt[0] + rb[0], rk[p.trainIdx].pt[1] + rb[1]] for p in good])
            cv2.setRNGSeed(0)
            matrix, mask = cv2.estimateAffinePartial2D(first, second, method=cv2.RANSAC,
                ransacReprojThreshold=th["reprojection_px"], maxIters=2000)
            if matrix is not None and mask is not None and np.isfinite(matrix).all():
                selected = mask.ravel().astype(bool)
                count = int(selected.sum())
                fraction = count / len(good)
                scale = math.hypot(float(matrix[0, 0]), float(matrix[1, 0]))
                valid = count >= th["minimum_inliers"] and fraction >= th["minimum_inlier_fraction"] and 0.2 <= scale <= 5
                result.update({"matched": valid, "reason": "registered prose pixels" if valid else "weak image registration",
                    "matrix": matrix.tolist(), "inliers": count, "inlier_fraction": fraction,
                    "source_inlier_box": [float(first[selected, 0].min()), float(first[selected, 1].min()),
                                          float(first[selected, 0].max()), float(first[selected, 1].max())] if count else None})
    write_json(cache, {"schema_version": 1, "complete": True, "metadata": metadata, "data": result}, work, repo)
    return result


def project_source_box(box, registration):
    """Project all source-box corners through the measured image transform."""
    import numpy as np
    sw, sh = registration["source_dimensions"]
    rw, rh = registration["reader_dimensions"]
    matrix = np.asarray(registration["matrix"])
    corners = np.asarray([[box[0] * sw, box[1] * sh, 1], [box[2] * sw, box[1] * sh, 1],
                          [box[0] * sw, box[3] * sh, 1], [box[2] * sw, box[3] * sh, 1]])
    points = corners @ matrix.T
    return [float(points[:, 0].min() / rw), float(points[:, 1].min() / rh),
            float(points[:, 0].max() / rw), float(points[:, 1].max() / rh)]


def compare_registered_region(row, source_index, source_page, box, registration, root, manifest,
                              work, repo, pdftoppm):
    """Compare a full-resolution output region with its independently registered source ink."""
    import cv2
    import numpy as np
    from PIL import Image
    rp, rr = snapshot_input(root, manifest, role="reader_pdf", tomus=row["tomus"])
    sp, sr = snapshot_input(root, manifest, role="source_pdf", source_index=source_index)
    ri, rm = render_page(rp, rr, row["page"], 300, work, repo, pdftoppm=pdftoppm)
    si, sm = render_page(sp, sr, source_page, 300, work, repo, pdftoppm=pdftoppm)
    with Image.open(ri) as image:
        target = np.asarray(image.convert("L"))
    with Image.open(si) as image:
        source = np.asarray(image.convert("L"))
    matrix = np.asarray(registration["matrix"], dtype=np.float64)
    sx = sm["raster_dimensions"][0] / registration["source_dimensions"][0]
    sy = sm["raster_dimensions"][1] / registration["source_dimensions"][1]
    rx = rm["raster_dimensions"][0] / registration["reader_dimensions"][0]
    ry = rm["raster_dimensions"][1] / registration["reader_dimensions"][1]
    full_matrix = np.diag([rx, ry]) @ matrix @ np.diag([1 / sx, 1 / sy, 1])
    bounds = pixel_bounds(box, rm["raster_dimensions"])
    local_matrix = full_matrix.copy()
    local_matrix[:, 2] -= [bounds[0], bounds[1]]
    expected = cv2.warpAffine(source, local_matrix, (bounds[2] - bounds[0], bounds[3] - bounds[1]), borderValue=255)
    observed = target[bounds[1]:bounds[3], bounds[0]:bounds[2]]
    score = float(cv2.matchTemplate(observed, expected, cv2.TM_CCOEFF_NORMED)[0, 0]) if min(observed.std(), expected.std()) > 2 else 0.0
    inverse = cv2.invertAffineTransform(full_matrix)
    points = np.asarray([[bounds[0], bounds[1], 1], [bounds[2], bounds[1], 1],
                         [bounds[0], bounds[3], 1], [bounds[2], bounds[3], 1]]) @ inverse.T
    source_box = [float(points[:, 0].min() / source.shape[1]), float(points[:, 1].min() / source.shape[0]),
                  float(points[:, 0].max() / source.shape[1]), float(points[:, 1].max() / source.shape[0])]
    clipped = any(v < 0 or v > 1 for v in source_box)
    source_box = union_box([source_box])
    return {"correlation": score if math.isfinite(score) else 0.0, "source_box": source_box,
            "source_box_clipped": clipped, "dpi": 300}


def find_foreign_scraps(rows, reader, source_unit, path, root, manifest, run, work, repo, settings, pdftoppm):
    """Nominate marginal ink corroborated against actual neighboring Greek-column words."""
    import cv2
    import numpy as np
    import pymupdf
    import unicodedata
    from PIL import Image
    th = settings["thresholds"]["scraps"]
    source_pdf, source_record = snapshot_input(root, manifest, role="source_pdf", source_index=source_unit["source_index"])
    candidates, diagnostics = [], []
    with pymupdf.open(source_pdf) as document:
        source_words = {p: pdf_word_observations(document[p - 1]) for p in sorted({t["page"] for t in source_unit["tokens"]})}
    for row in rows:
        page_anchors = [a for a in path if any(t["page"] == row["page"] for t in reader[a["output_start"]:a["output_end"]])]
        pages = sorted({p for a in page_anchors for p in token_pages(source_unit, a["source_start"], a["source_end"])})
        pdf, record = snapshot_input(root, manifest, role="reader_pdf", tomus=row["tomus"])
        raster, meta = render_page(pdf, record, row["page"], 150, work, repo, pdftoppm=pdftoppm)
        with Image.open(raster) as image:
            gray = np.asarray(image.convert("L"))
        bounds = pixel_bounds(row["body_box"], meta["raster_dimensions"])
        height = ink_line_height(gray[bounds[1]:bounds[3], bounds[0]:bounds[2]]) or 12
        observed_regions, source_regions, component_boxes, scores, registrations = [], [], [], [], []
        for page in pages:
            registration = match_source_image(row, source_unit["source_index"], page, root, manifest, work, repo, settings, pdftoppm)
            registrations.append(registration)
            if not registration["matched"]:
                continue
            words = source_words[page]
            latin = [w["box"][0] for w in words if 0.12 < w["box"][1] < 0.94
                     and (w["box"][0] + w["box"][2]) / 2 >= 0.50 and len(w["raw"]) >= 3
                     and sum("a" <= c <= "z" for c in w["raw"].casefold()) >= 3]
            latin_left = sorted(latin)[max(0, len(latin) // 10)] if latin else 0.52
            greek = [w for w in words if sum("GREEK" in unicodedata.name(c, "") for c in w["raw"]) >= th["greek_characters_min"]
                     and (w["box"][0] + w["box"][2]) / 2 < latin_left and 0.12 < w["box"][1] < 0.94]
            for word in greek:
                projected = project_source_box(word["box"], registration)
                margin_end = row["body_box"][0] + th["margin_body_fraction"] * (row["body_box"][2] - row["body_box"][0])
                box = [max(row["body_box"][0], projected[0]), max(row["body_box"][1], projected[1]),
                       min(margin_end, projected[2]), min(row["body_box"][3], projected[3])]
                if box[2] <= box[0] or box[3] <= box[1]:
                    continue
                pixels = pixel_bounds(box, meta["raster_dimensions"])
                patch = gray[pixels[1]:pixels[3], pixels[0]:pixels[2]]
                _, _, components, _ = cv2.connectedComponentsWithStats((patch < 150).astype(np.uint8), 8)
                eligible = [s for s in components[1:] if 0.3 * height <= s[3] <= 2.0 * height and s[4] >= 3 and s[2] <= 3 * height]
                if not eligible:
                    continue
                correspondence = compare_registered_region(row, source_unit["source_index"], page, box, registration,
                    root, manifest, work, repo, pdftoppm)
                if correspondence["correlation"] < th["correlation_min"]:
                    continue
                scores.append(correspondence["correlation"])
                observed_regions.append({"role": "reader_pdf", "tomus": row["tomus"], "page": row["page"],
                                         "input_sha256": record["sha256"], "box": box})
                source_regions.append({"role": "source_pdf", "source_index": source_unit["source_index"], "page": page,
                                       "input_sha256": source_record["sha256"], "box": correspondence["source_box"]})
                component_boxes.extend([[(pixels[0] + int(s[0])) / gray.shape[1], (pixels[1] + int(s[1])) / gray.shape[0],
                                         (pixels[0] + int(s[0] + s[2])) / gray.shape[1], (pixels[1] + int(s[1] + s[3])) / gray.shape[0]] for s in eligible])
        unique = {tuple(round(v, 5) for v in b): b for b in component_boxes}
        line_groups = group_lines([{"box": b, "text": "ink"} for b in unique.values()])
        diagnostics.append({"tomus": row["tomus"], "page": row["page"], "registrations": registrations,
                            "corroborated_components": len(unique), "lines": len(line_groups)})
        if len(unique) >= th["minimum_components"] and len(line_groups) >= th["minimum_lines"]:
            output = dict(observed_regions[0], box=union_box([r["box"] for r in observed_regions]))
            event = f"scraps_{row['unit_id']}_{row['page']}"
            crops = [save_region_crop(root, manifest, r, run, work, repo, 300, f"{event}_{i}.png")
                     for i, r in enumerate(source_regions + [output])]
            candidates.append({"event_id": event, "unit_id": row["unit_id"], "type": "foreign_scraps",
                "relation": "foreign_column", "source_regions": source_regions, "output_regions": [output],
                "primary_score": min(scores), "component_scores": {"components": len(unique), "lines": len(line_groups),
                                                                       "source_correlations": scores},
                "confidence": "low", "status": "candidate", "confidence_reasons": [
                    "multiple marginal components on multiple lines match original Greek-column words",
                    "Greek raster OCR model unavailable; glyph readings remain uncertain"],
                "evidence_provenance": {"method": "registered neighboring-column ink", "registrations": registrations,
                                        "components": list(unique.values())}, "linked_events": [], "crops": crops})
        progress(f"Scrap search {row['unit_id']} page {row['page']}: {len(unique)} components on {len(line_groups)} lines")
    write_json(run / f"scrap-debug-{source_unit['unit_id']}.json", {"schema_version": 1,
               "unit_id": source_unit["unit_id"], "pages": diagnostics,
               "limitation": "Greek raster OCR is unavailable; image evidence does not establish exact glyph text."}, work, repo)
    return candidates


def furniture_pattern(raw):
    """Classify observed capitals and separate folios without expanding their text."""
    letters = [(i, c) for i, c in enumerate(raw) if c.isalpha()]
    capitals = bool(letters) and len(letters) <= 20 and all(c.isupper() for _, c in letters)
    digits = raw.strip().rstrip(".")
    return {"capital": capitals, "folio": digits.isdigit() and 2 <= len(digits) <= 4,
            "roman": bool(re.fullmatch(r"[IVXLCDM]+\.?", raw.strip())),
            "clean": "".join(c for _, c in letters), "raw_offsets": [i for i, _ in letters]}


def source_head_references(source_unit, root, manifest, work, repo, settings):
    """Inventory full-width original source heads/folios with the shared PDF adapter."""
    import pymupdf
    pdf, record = snapshot_input(root, manifest, role="source_pdf", source_index=source_unit["source_index"])
    pages = sorted({t["page"] for t in source_unit["tokens"]})
    metadata = {"source_sha256": record["sha256"], "pages": pages,
                "head_bottom": settings["thresholds"]["head"]["source_head_bottom"],
                "adapter": "full-width-source-furniture-v1", "render_box_policy": "mediabox"}
    cache = Path(work) / "cache" / "source_furniture" / f"{json_hash(metadata)}.json"
    reused = reusable_entry(cache, metadata, work, repo)
    if reused is not None:
        return reused["references"]
    references = {}
    with pymupdf.open(pdf) as document:
        for number in pages:
            page = document[number - 1]
            entries = []
            for observation in pdf_word_observations(page):
                raw, box = observation["raw"], observation["box"]
                pattern = furniture_pattern(raw)
                if 0.005 <= (box[1] + box[3]) / 2 <= metadata["head_bottom"] and (pattern["capital"] or pattern["folio"]):
                    entries.append({"raw": raw, "pattern": pattern, "confidence": None,
                                    "provider": "embedded_source_words",
                                    "region": {"role": "source_pdf", "source_index": source_unit["source_index"],
                                               "page": number, "box": box, "input_sha256": record["sha256"]}})
            references[str(number)] = entries
    write_json(cache, {"schema_version": 1, "complete": True, "metadata": metadata,
                       "data": {"references": references}}, work, repo)
    return references


def head_templates(references, root, manifest, work, repo, pdftoppm):
    """Build original head suffix/folio image hypotheses with unchanged source exemplars."""
    import cv2
    import numpy as np
    from PIL import Image
    templates = []
    for number, entry in enumerate(references):
        region = entry["region"]
        pdf, record = snapshot_input(root, manifest, role="source_pdf", source_index=region["source_index"])
        raster, meta = render_page(pdf, record, region["page"], 300, work, repo, pdftoppm=pdftoppm)
        with Image.open(raster) as image:
            full = np.asarray(image.convert("L"))
        expanded = [max(0, region["box"][0] - 0.003), max(0, region["box"][1] - 0.003),
                    min(1, region["box"][2] + 0.003), min(1, region["box"][3] + 0.003)]
        bounds = pixel_bounds(expanded, meta["raster_dimensions"])
        gray = full[bounds[1]:bounds[3], bounds[0]:bounds[2]].copy()
        _, _, stats, _ = cv2.connectedComponentsWithStats((gray < 160).astype(np.uint8), 8)
        tallest = max((int(s[3]) for s in stats[1:]), default=0)
        glyphs = sorted((s for s in stats[1:] if s[3] >= 0.45 * tallest and s[4] >= 5), key=lambda s: int(s[0]))
        if not glyphs:
            continue
        column = pixel_bounds([0.52, 0.12, 0.97, 0.90], meta["raster_dimensions"])
        source_height = ink_line_height(full[column[1]:column[3], column[0]:column[2]])
        counts = [len(glyphs)] if entry["pattern"]["folio"] else range(1, min(len(glyphs), 20) + 1)
        for count in counts:
            group = glyphs[-count:]
            x0 = max(0, min(int(s[0]) for s in group) - 2)
            y0 = max(0, min(int(s[1]) for s in group) - 2)
            x1 = min(gray.shape[1], max(int(s[0] + s[2]) for s in group) + 2)
            y1 = min(gray.shape[0], max(int(s[1] + s[3]) for s in group) + 2)
            pixels = gray[y0:y1, x0:x1].copy()
            if min(pixels.shape) < 4 or float(pixels.std()) < 2:
                continue
            templates.append({"id": f"head_{region['source_index']}_{region['page']}_{number}_{count}",
                "family": f"head_{region['source_index']}_{region['page']}_{number}",
                "fragment_type": "folio" if entry["pattern"]["folio"] else "head_suffix",
                "pixels": pixels, "views": image_views(pixels), "source_line_height": source_height,
                "glyph_support": count, "glyph_boxes": [[int(s[0]) - x0, int(s[1]) - y0,
                                                           int(s[0] + s[2]) - x0, int(s[1] + s[3]) - y0] for s in group],
                "offset": [x0, y0], "source_region": region, "source_reference": entry,
                "pixels_sha256": hashlib.sha256(pixels.tobytes()).hexdigest()})
    return templates


def source_head_suffix_score(observed, reference):
    """Score a comparison-only suffix; Greek/Latin lookalikes retain explicit ambiguity."""
    first = tolerant_view(furniture_pattern(observed)["clean"])
    second = tolerant_view(reference["pattern"]["clean"])
    if not first or not second:
        return 0.0
    return char_similarity(first, second[-len(first):])


def find_head_candidates(rows, reader, source_unit, path, root, manifest, run, work, repo,
                         settings, lang, tesseract, pdftoppm):
    """Combine observed patterns, isolation, source heads/folios and image correspondence."""
    import cv2
    import numpy as np
    from PIL import Image
    th = settings["thresholds"]["head"]
    references = source_head_references(source_unit, root, manifest, work, repo, settings)
    candidates = []
    template_cache = {}
    for row in rows:
        words = [t for t in reader if t["page"] == row["page"]]
        pdf, record = snapshot_input(root, manifest, role="reader_pdf", tomus=row["tomus"])
        raster, meta = render_page(pdf, record, row["page"], 180, work, repo, pdftoppm=pdftoppm)
        bounds = pixel_bounds(row["body_box"], meta["raster_dimensions"])
        with Image.open(raster) as image:
            gray = np.asarray(image.convert("L"))[bounds[1]:bounds[3], bounds[0]:bounds[2]].copy()
        height = ink_line_height(gray) or 12.0
        norm_height = height / meta["raster_dimensions"][1]
        lines = group_lines([{"box": w["box"], "text": w["raw"]} for w in words])
        prose = [line for line in lines if len(line["text"].split()) >= 3
                 and sum(sum(c.islower() for c in word) >= 2 for word in line["text"].split()) >= 2]

        def isolated(box):
            return not any(min(box[3], w["box"][3]) - max(box[1], w["box"][1])
                           >= 0.4 * min(box[3] - box[1], w["box"][3] - w["box"][1]) for w in prose)

        observations = []
        for word in words:
            pattern = furniture_pattern(word["raw"])
            if (pattern["capital"] or pattern["folio"]) and not pattern["roman"] and isolated(word["box"]):
                observations.append({"box": word["box"], "raw": word["raw"], "pattern": pattern,
                                     "provider": "reader_raster_ocr", "isolation": True})
        _, _, components, _ = cv2.connectedComponentsWithStats((gray < 160).astype(np.uint8), 8)
        for component in components[1:]:
            x, y, width, char_height, area = (int(v) for v in component)
            if not 0.6 * height <= char_height <= 1.8 * height or area < 5 or width > 3 * height:
                continue
            box = [(bounds[0] + x) / meta["raster_dimensions"][0], (bounds[1] + y) / meta["raster_dimensions"][1],
                   (bounds[0] + x + width) / meta["raster_dimensions"][0], (bounds[1] + y + char_height) / meta["raster_dimensions"][1]]
            if isolated(box) and not any(overlap({"box": box}, {"box": o["box"]})["iou"] > 0 for o in observations):
                observations.append({"box": box, "raw": "", "pattern": furniture_pattern(""),
                                     "provider": "isolated_image_component", "isolation": True,
                                     "capital_height": char_height >= th["image_capital_height_min"] * height})
        page_events = []
        for observation in observations:
            context = source_context_at_region(reader, path, source_unit, row["page"], observation["box"])
            before, after = context["before"], context["after"]
            seam = bool(before and after and before["source_page"] < after["source_page"])
            source_pages = {p["source_page"] for p in (before, after) if p}
            if seam:
                source_pages = {after["source_page"]}
            refs = [r for number in source_pages for r in references.get(str(number), [])]
            if not refs:
                continue
            suffix = max((source_head_suffix_score(observation["raw"], ref) for ref in refs), default=0.0)
            folio = any(observation["pattern"]["folio"] and ref["pattern"]["folio"]
                        and observation["raw"].rstrip(".") == ref["raw"].rstrip(".") for ref in refs)
            if not seam and not folio:
                continue
            key = tuple(sorted(source_pages))
            if key not in template_cache:
                template_cache[key] = head_templates(refs, root, manifest, work, repo, pdftoppm)
            templates = template_cache[key]
            pad = th["padding_line_heights"] * norm_height
            box = observation["box"]
            retry_box = [max(row["body_box"][0], box[0] - pad), max(row["body_box"][1], box[1] - pad),
                         min(row["body_box"][2], box[2] + pad), min(row["body_box"][3], box[3] + pad)]
            full_raster, full_meta = render_page(pdf, record, row["page"], 300, work, repo, pdftoppm=pdftoppm)
            retry_bounds = pixel_bounds(retry_box, full_meta["raster_dimensions"])
            with Image.open(full_raster) as image:
                retry_gray = np.asarray(image.convert("L"))[retry_bounds[1]:retry_bounds[3], retry_bounds[0]:retry_bounds[2]].copy()
            matched = match_templates(retry_gray, templates, height * 300 / 180, settings, kind="head")
            peaks = [p for group in matched for p in group]
            located = []
            for peak in peaks:
                actual = [(retry_bounds[0] + peak["box"][0]) / full_meta["raster_dimensions"][0],
                          (retry_bounds[1] + peak["box"][1]) / full_meta["raster_dimensions"][1],
                          (retry_bounds[0] + peak["box"][2]) / full_meta["raster_dimensions"][0],
                          (retry_bounds[1] + peak["box"][3]) / full_meta["raster_dimensions"][1]]
                scores = overlap({"box": actual}, {"box": box})
                if scores["gold_coverage"] >= 0.3:
                    located.append(dict(peak, normalized_box=actual))
            peaks = located
            image_score = max((p["score"] for p in peaks), default=0.0)
            if image_score < th["nomination_min"] and suffix < th["suffix_min"] and not folio:
                continue
            best = max(peaks, key=lambda p: p["score"]) if peaks else None
            best_template = next((t for t in templates if best and t["id"] == best["template_id"]), None)
            if best:
                box = union_box([box, best["normalized_box"]])
            source_regions = [best_template["source_region"]] if best_template else [max(refs, key=lambda r: (
                observation["pattern"]["folio"] and r["pattern"]["folio"] and observation["raw"].rstrip(".") == r["raw"].rstrip("."),
                source_head_suffix_score(observation["raw"], r)))["region"]]
            pattern_cue = observation["pattern"]["capital"] or observation["pattern"]["folio"] or observation.get("capital_height", False)
            cues = {"capital_number_pattern": pattern_cue, "isolation": observation["isolation"],
                    "source_suffix_match": suffix >= th["suffix_min"], "source_folio_agreement": folio,
                    "source_head_image": image_score >= th["high_min"]}
            strong = seam and sum(cues.values()) >= th["minimum_cues"]
            single = not observation["pattern"]["folio"] and len(observation["pattern"]["clean"]) <= 1
            if single and image_score < th["high_min"]:
                strong = False
            retry = ocr_region_retry(pdf, record, row["page"], retry_box, work, repo,
                                     tesseract=tesseract, pdftoppm=pdftoppm, psm=11, lang=lang)
            output = {"role": "reader_pdf", "tomus": row["tomus"], "page": row["page"],
                      "box": box, "input_sha256": record["sha256"]}
            event = f"head_{row['unit_id']}_{row['page']}_{len(page_events)}"
            crops = [save_region_crop(root, manifest, region, run, work, repo, 300, f"{event}_{i}.png")
                     for i, region in enumerate(source_regions + [output])]
            page_events.append({"event_id": event, "unit_id": row["unit_id"], "type": "head", "relation": "source_head",
                "source_regions": source_regions, "output_regions": [output], "primary_score": max(image_score, suffix, float(folio)),
                "component_scores": dict(cues, image_correlation=image_score, suffix_similarity=suffix, source_boundary=seam),
                "confidence": "high" if strong else "low", "status": "candidate",
                "raw_ocr_excerpts": {"output": observation["raw"], "source": [r["raw"] for r in refs]},
                "confidence_reasons": [f"{name}: {'satisfied' if value else 'failed'}" for name, value in cues.items()]
                    + ["Greek/Latin glyph identity remains uncertain without a Greek OCR model"],
                "evidence_provenance": {"method": "isolated head/folio source correspondence", "context": context,
                                        "observation": observation, "source_references": refs, "image_peaks": peaks,
                                        "reader_retry": retry},
                "linked_events": [], "crops": crops})
        groups = []
        for event in page_events:
            region = event["output_regions"][0]
            source_pages = {(r["source_index"], r["page"]) for r in event["source_regions"]}
            group = next((g for g in groups if any(
                source_pages & {(r["source_index"], r["page"]) for r in c["source_regions"]}
                and abs((region["box"][1] + region["box"][3]) / 2
                        - (c["output_regions"][0]["box"][1] + c["output_regions"][0]["box"][3]) / 2) <= 3 * norm_height
                for c in g)), None)
            if group is None:
                groups.append([event])
            else:
                group.append(event)
        for group in groups:
            if len(group) == 1:
                candidates.append(group[0])
                continue
            winner = max(group, key=lambda c: (c["confidence"] == "high", c["primary_score"], c["event_id"]))
            combined = dict(winner, event_id=f"head_{row['unit_id']}_{row['page']}_{len(candidates)}")
            combined["output_regions"] = [dict(group[0]["output_regions"][0], box=union_box(
                [c["output_regions"][0]["box"] for c in group]))]
            source = {}
            for c in group:
                for r in c["source_regions"]:
                    source.setdefault((r["source_index"], r["page"]), []).append(r)
            combined["source_regions"] = [dict(rs[0], box=union_box([r["box"] for r in rs])) for rs in source.values()]
            combined["linked_events"] = [c["event_id"] for c in group]
            combined["crops"] = [crop for c in group for crop in c["crops"]]
            combined["evidence_provenance"] = {"method": "same-source isolated head and folio components",
                                                "components": group}
            candidates.append(combined)
        progress(f"Head search {row['unit_id']} page {row['page']}: {len(page_events)} candidates")
    return candidates


def boundary_alignment(source, output, side, settings):
    """Align an expected edge to its reader edge, retaining token operations and IDs.

    Exact anchors locate the local comparison inside the edge window; lexical matches
    elsewhere in the unit cannot supply boundary coverage. The bounded occurrence aligner
    handles OCR substitutions and split words, including continuations on another page.
    """
    th = settings["thresholds"]["boundary"]
    if not source or not output:
        return {"status": "inconclusive", "reason": "no readable edge tokens",
                "coverage": 0.0, "edge_coverage": 0.0, "operations": []}
    expected = source[:th["source_window"]] if side == "start" else source[-th["source_window"]:]
    observed = output[:th["output_window"]] if side == "start" else output[-th["output_window"]:]
    exact = align_tokens([t["strict"] for t in expected], [t["strict"] for t in observed])
    anchors = [a for a in exact["runs"] if a[2] >= 3]
    if not anchors:
        return {"status": "inconclusive", "reason": "no context anchor at reader edge",
                "coverage": 0.0, "edge_coverage": 0.0, "operations": [],
                "source_tokens": expected, "output_tokens": observed}
    anchor = max(anchors, key=lambda a: (a[2], -abs(a[0] - a[1]), -a[1]))
    # A small allowance retains OCR insertions beside the anchored source interval.
    offset = anchor[1] - anchor[0]
    if side == "terminal":
        observed = observed[max(0, offset - 8):]
    else:
        observed = observed[:min(len(observed), max(0, offset) + len(expected) + 8)]
    alignment = align_tokens(expected, observed, mode="local", settings=settings)
    if alignment.get("inconclusive"):
        return dict(alignment, status="inconclusive", source_tokens=expected, output_tokens=observed)
    operations = alignment["operations"]
    first = next((i for i, op in enumerate(operations) if op["source"]), len(operations))
    last = next((i for i in range(len(operations) - 1, -1, -1) if operations[i]["source"]), -1)
    edge_operations = operations[first:last + 1]
    edge_cost = sum(op["cost"] for op in edge_operations)
    output_count = len({i for op in edge_operations for i in op["output"]})
    alignment["full_window_ordered_similarity"] = alignment["ordered_similarity"]
    alignment["ordered_similarity"] = max(0.0, 1 - edge_cost / max(len(expected), output_count, 1))
    covered = set(alignment["covered_source"])
    edge = list(range(min(th["edge_tokens"], len(expected)))) if side == "start" else list(
        range(max(0, len(expected) - th["edge_tokens"]), len(expected)))
    coverage = len(covered) / len(expected)
    edge_coverage = sum(i in covered for i in edge) / len(edge)
    matched = (len(expected) >= th["minimum_tokens"] and coverage >= th["coverage_min"]
               and edge_coverage >= th["coverage_min"]
               and (edge[0] if side == "start" else edge[-1]) in covered
               and alignment["ordered_similarity"] >= th["ordered_min"])
    return dict(alignment, status="matched" if matched else "inconclusive",
                reason="edge correspondence" if matched else "edge OCR or correspondence unresolved",
                coverage=coverage, edge_coverage=edge_coverage,
                source_tokens=expected, output_tokens=observed,
                source_occurrence_ids=[t["occurrence_id"] for t in expected])


def write_boundary_review(boundaries, run, work, repo):
    """Write a local comparison page with raw observations, scores and linked crops."""
    sections = []
    for unit, record in boundaries.items():
        sections.append(f"<h2>{html.escape(unit)}</h2><p>Partition end: {record['partition_end_page']}; "
                        f"observed terminal prose page: {record['terminal_source_page']}</p>")
        for side in ("start", "terminal"):
            edge = record[side]
            sections.append(f"<h3>{side}: {html.escape(edge['status'])}</h3>"
                            f"<p>{html.escape(edge['reason'])}</p>")
            scores = {k: edge.get(k) for k in ("coverage", "edge_coverage", "ordered_similarity",
                                              "source_raster_agreement")}
            sections.append(f"<pre>{html.escape(json.dumps(scores, indent=2))}</pre>")
            for role in ("source", "output"):
                excerpt = " ".join(t["raw"] for t in edge.get(f"{role}_tokens", []))
                sections.append(f"<p>{role} raw OCR: {html.escape(excerpt)}</p>")
            for crop in edge["crops"]:
                target = Path(work).resolve() / relative_path(crop["path"])
                path = html.escape(target.relative_to(run).as_posix(), quote=True)
                sections.append(f"<a href='{path}'><img src='{path}' alt='{path}'></a>")
    safe_write_path(run / "boundaries.html", work, repo).write_text(
        "<!doctype html><meta charset='utf-8'><title>Boundary correspondence</title>"
        "<style>body{font:16px system-ui;margin:2rem;max-width:1200px}img{max-width:100%;display:block;margin:1rem 0}</style>"
        "<h1>Starts and terminal prose</h1><p>Matched endpoints do not establish interior completeness. "
        "Inconclusive OCR is not a terminal-loss finding.</p>" + "".join(sections), encoding="utf-8")


def ocr_view_tokens(observation):
    """Preserve raw raster words, page coordinates and observed reading order."""
    tokens = []
    for line in observation["lines"]:
        words = [w for w in observation["words"] if w["line_id"] == line["line_id"]]
        for word in sorted(words, key=lambda w: w["box"][0]):
            tokens.append({"raw": word["text"], "strict": strict_view(word["text"]),
                           "tolerant": tolerant_view(word["text"]), "box": word["box"],
                           "page": observation["page"], "conf": word["conf"]})
    return tokens


def compare_boundaries(reader, source_unit, region_unit, root, manifest, run, work, repo,
                       settings, lang, tesseract, pdftoppm, tomus):
    """Compare actual first/last source prose independently of interior completeness.

    Latin-majority lines inside the partition supply expected occurrences; following
    title pages and punctuation do not define an endpoint. Raster observations corroborate
    those lines. Failed correspondence remains inconclusive unless a whole terminal line
    has source support and independent reader retries preserve its absence.
    """
    th = settings["thresholds"]["boundary"]
    pdf, source_record = snapshot_input(root, manifest, role="source_pdf",
                                         source_index=source_unit["source_index"])
    reader_record = snapshot_input(root, manifest, role="reader_pdf", tomus=tomus)[1] if tomus else None
    reader_pdf = snapshot_input(root, manifest, role="reader_pdf", tomus=tomus)[0] if tomus else None
    eligible = set()
    for page in region_unit["pages"]:
        if page.get("skipped"):
            continue
        for index, line in enumerate(page["lines"]):
            words = line["text"].split()
            latin = sum(is_latin_token(strict_view(w)) for w in words)
            letters = [c for c in line["text"] if c.isalpha()]
            heading = len(words) <= 4 and letters and all(c.isupper() for c in letters)
            if (words and latin >= settings["thresholds"]["omission"]["latin_line_fraction"] * len(words)
                    and not heading
                    and 0.06 <= (line["box"][1] + line["box"][3]) / 2 <= 0.94):
                eligible.add((page["page"], index))
    source = [dict(token, strict=source_unit["strict"][i], tolerant=source_unit["tolerant"][i],
                   box=source_unit["boxes"][i], occurrence_id=f"{region_unit['unit_id']}:{i}")
              for i, token in enumerate(source_unit["tokens"])
              if (token["page"], token["line"]) in eligible and is_latin_token(source_unit["strict"][i])]
    result = {"partition_end_page": region_unit["end_page"],
              "terminal_source_page": source[-1]["page"] if source else None,
              "interior_completeness": "not assessed", "candidates": []}
    for side in ("start", "terminal"):
        comparison = boundary_alignment(source, reader, side, settings)
        expected = comparison.get("source_tokens", [])
        observed = comparison.get("output_tokens", [])
        source_regions = page_union_regions(expected, "source_pdf", source_record,
                                             source_index=source_unit["source_index"])
        output_regions = page_union_regions(observed, "reader_pdf", reader_record, tomus=tomus) if reader_record else []
        source_retry = []
        for region in source_regions:
            observation = ocr_page_region(pdf, source_record, region["page"], 300, 6, lang, work, repo,
                                          box=region["box"], tesseract=tesseract, pdftoppm=pdftoppm)
            source_retry.extend(ocr_view_tokens(observation))
        agreement = align_tokens(expected, source_retry, mode="local", settings=settings) if expected else None
        agreement_score = (len(agreement["covered_source"]) / len(expected)
                           if agreement and not agreement.get("inconclusive") else 0.0)
        comparison["source_raster_agreement"] = agreement_score
        retry_results = []
        if comparison["status"] != "matched" and output_regions:
            for psm in (6, 11):
                retry = []
                for region in output_regions:
                    observation = ocr_page_region(reader_pdf, reader_record, region["page"], 300, psm,
                                                  lang, work, repo, box=region["box"],
                                                  tesseract=tesseract, pdftoppm=pdftoppm)
                    retry.extend(ocr_view_tokens(observation))
                retry_results.append(boundary_alignment(source, retry, side, settings))
            if any(r["status"] == "matched" for r in retry_results):
                comparison["status"] = "inconclusive"
                comparison["reason"] = "reader observations conflict; retry finds correspondence"
        if agreement_score < th["coverage_min"]:
            comparison["status"] = "inconclusive"
            comparison["reason"] = "source embedded/raster observations disagree"
        covered = set(comparison.get("covered_source", []))
        last_line = ((expected[-1]["page"], expected[-1]["line"]) if expected else None)
        terminal_line = [i for i, t in enumerate(expected) if (t["page"], t["line"]) == last_line]
        line_absent = bool(terminal_line) and all(i not in covered for i in terminal_line)
        supported = set(agreement.get("covered_source", [])) if agreement else set()
        terminal_support = (sum(i in supported for i in terminal_line) / len(terminal_line)
                            if terminal_line else 0.0)
        if side == "terminal" and terminal_support < th["coverage_min"]:
            comparison["status"] = "inconclusive"
            comparison["reason"] = "terminal source line lacks raster corroboration"
        retries_absent = len(retry_results) == 2 and all(
            r.get("operations") and all(i not in r.get("covered_source", []) for i in terminal_line)
            for r in retry_results)
        if (side == "terminal" and line_absent and retries_absent
                and comparison.get("coverage", 0) >= 0.5 and agreement_score >= th["coverage_min"]
                and terminal_support >= th["coverage_min"]):
            event = f"terminal_{region_unit['unit_id']}_{expected[-1]['page']}"
            result["candidates"].append({
                "event_id": event, "unit_id": region_unit["unit_id"], "type": "terminal_loss",
                "relation": "terminal_loss", "source_regions": page_union_regions(
                    [expected[i] for i in terminal_line], "source_pdf", source_record,
                    source_index=source_unit["source_index"]),
                "output_regions": output_regions, "confidence": "low", "status": "candidate",
                "primary_score": agreement_score,
                "component_scores": {"source_raster_agreement": agreement_score,
                                     "edge_coverage": comparison["edge_coverage"]},
                "confidence_reasons": ["source-supported terminal line unmatched after two reader retries",
                                       "one-sided evidence requires visual adjudication"],
                "evidence_provenance": {"method": "source partition edge occurrence alignment with raster corroboration and two reader retries"},
                "linked_events": [], "crops": []})
        comparison["source_regions"], comparison["output_regions"] = source_regions, output_regions
        comparison["reader_retries"] = retry_results
        comparison["crops"] = []
        for i, region in enumerate(source_regions + output_regions):
            comparison["crops"].append(save_region_crop(root, manifest, region, run, work, repo, 300,
                f"boundary_{region_unit['unit_id']}_{side}_{i}.png"))
        if result["candidates"] and side == "terminal":
            result["candidates"][-1]["crops"] = comparison["crops"]
        if side == "terminal" and comparison["status"] == "matched":
            result["candidates"].append({
                "event_id": f"endpoint_{region_unit['unit_id']}_{expected[-1]['page']}",
                "unit_id": region_unit["unit_id"], "type": "terminal_loss",
                "relation": "complete_endpoint", "source_regions": source_regions,
                "output_regions": output_regions, "confidence": "high", "status": "candidate",
                "primary_score": comparison["ordered_similarity"],
                "component_scores": {"source_raster_agreement": agreement_score,
                                     "edge_coverage": comparison["edge_coverage"]},
                "confidence_reasons": ["terminal occurrence matches at the reader edge",
                                       "source raster corroborates expected prose",
                                       "interior completeness is not established"],
                "evidence_provenance": {"method": "source partition edge occurrence alignment",
                                        "source_occurrence_ids": comparison["source_occurrence_ids"]},
                "raw_ocr_excerpts": {"source": " ".join(t["raw"] for t in expected),
                                     "output": " ".join(t["raw"] for t in observed)},
                "linked_events": [], "crops": comparison["crops"]})
        result[side] = comparison
    return result


def source_token_boxes(index_unit, region_unit):
    """Attach each index token a coarse box drawn from its expected-Latin source line."""
    lookup = {}
    for page in region_unit["pages"]:
        if page.get("skipped"):
            continue
        for line_index, line in enumerate(page["lines"]):
            lookup[(page["page"], line_index)] = line["box"]
    default = [SOURCE_LATIN_X0, 0.0, SOURCE_LATIN_X1, 1.0]
    return [lookup.get((token["page"], token["line"]), default) for token in index_unit["tokens"]]


def union_box(boxes):
    """A clamped bounding box with guaranteed positive area over normalized boxes."""
    x0 = max(0.0, min(min(b[0] for b in boxes), 0.999))
    y0 = max(0.0, min(min(b[1] for b in boxes), 0.999))
    x1 = min(1.0, max(max(b[2] for b in boxes), x0 + 1e-3))
    y1 = min(1.0, max(max(b[3] for b in boxes), y0 + 1e-3))
    return [x0, y0, x1, y1]


def page_union_regions(tokens, role, record, *, tomus=None, source_index=None, occurrence_id=None):
    """Group tokens by physical page into one clamped diagnostic region per page."""
    by_page = {}
    for token in tokens:
        by_page.setdefault(token["page"], []).append(token["box"])
    regions = []
    for page in sorted(by_page):
        region = {"role": role, "page": page, "input_sha256": record["sha256"],
                  "box": union_box(by_page[page])}
        if role == "reader_pdf":
            region["tomus"] = tomus
        else:
            region["source_index"] = source_index
        if occurrence_id:
            region["occurrence_id"] = occurrence_id
        regions.append(region)
    return regions


def save_region_crop(root, manifest, region, run, work, repo, dpi, name):
    """Render (cache-hit) the region's page and save a cropped evidence PNG under the run."""
    from PIL import Image
    pdf, record = snapshot_input(root, manifest, role=region["role"],
                                 tomus=region.get("tomus"), source_index=region.get("source_index"))
    raster, _ = render_page(pdf, record, region["page"], dpi, work, repo)
    destination = safe_write_path(run / "crops" / name, work, repo)
    destination.parent.mkdir(parents=True, exist_ok=True)
    with Image.open(raster) as image:
        image.crop(pixel_bounds(region["box"], image.size)).save(destination)
    return {"path": destination.relative_to(Path(work).resolve()).as_posix(), "sha256": hash_file(destination)}


def build_duplicate_candidate(unit_id, pair, reader, source_tokens, blocks, reader_record,
                              source_record, source_index, tomus, line_height, root, manifest,
                              run, work, repo, settings):
    """Assemble a reader-duplicate candidate with two output contributions and source pair."""
    first_run, second_run = sorted(pair, key=lambda r: r["output_start"])
    block = next((b for b in blocks
                  if intervals_overlap(b["first_tokens"][0], b["first_tokens"][1], first_run["canon_start"], first_run["canon_end"])), None)
    if block:
        first_source = [source_tokens[i] for i in range(*block["first_tokens"])]
        second_source = [source_tokens[i] for i in range(*block["second_tokens"])]
    else:
        first_source = [source_tokens[i] for i in range(first_run["source_start"], first_run["source_end"])]
        second_source = [source_tokens[i] for i in range(second_run["source_start"], second_run["source_end"])]
    source_regions = (page_union_regions(first_source, "source_pdf", source_record, source_index=source_index, occurrence_id="occurrence_1")
                      + page_union_regions(second_source, "source_pdf", source_record, source_index=source_index, occurrence_id="occurrence_2"))
    contributions, output_regions = [], []
    for ordinal, run_span in enumerate((first_run, second_run), 1):
        tokens = reader[run_span["output_start"]:run_span["output_end"]]
        contributions.append({"start_page": tokens[0]["page"], "end_page": tokens[-1]["page"],
                              "start_box": validate_box(list(tokens[0]["box"])),
                              "end_box": validate_box(list(tokens[-1]["box"])),
                              "line_height": line_height})
        output_regions.extend(page_union_regions(tokens, "reader_pdf", reader_record, tomus=tomus,
                                                 occurrence_id=f"output_{ordinal}"))
    cosine = max(first_run["cosine"], second_run["cosine"])
    ordered = max(first_run["ordered_similarity"], second_run["ordered_similarity"])
    matched = min(first_run["matched"], second_run["matched"])
    high = settings["thresholds"]["duplicate_high"]
    corroborated = block is not None
    incomplete = (second_run["canon_start"] - first_run["canon_start"]) > INCOMPLETE_OPENING_TOKENS
    predicates = {"cosine>=0.80": cosine >= high["cosine_min"],
                  "ordered>=0.90": ordered >= high["ordered_min"],
                  "informative_tokens>=40": matched >= high["informative_min"],
                  "two_nonoverlapping_contributions": True,
                  "source_scan_or_folio_corroboration": corroborated}
    confidence = "high" if all(predicates.values()) else "low"
    candidate = {
        "event_id": f"dup_{unit_id}_{source_regions[0]['page']}_{source_regions[-1]['page']}",
        "unit_id": unit_id, "type": "duplicate", "relation": "repeated_occurrence",
        "source_regions": source_regions, "output_regions": output_regions,
        "contributions": contributions, "incomplete_second_opening": incomplete,
        "confidence": confidence, "status": "candidate",
        "primary_score": round((cosine + ordered) / 2, 4),
        "component_scores": {"reader_source_cosine": round(cosine, 4),
                             "ordered_similarity": round(ordered, 4), "matched_tokens": matched,
                             "source_duplicate_cosine": round(block["cosine"], 4) if block else None},
        "confidence_reasons": [f"{name}: {'satisfied' if value else 'failed'}" for name, value in predicates.items()],
        "linked_events": [],
        "evidence_provenance": {"method": "Reader token windows retrieved against source-only TF-IDF windows, refined by exact-token alignment, chained across restarts; a repeated logical source occurrence yields two output contributions.",
                                "source_duplicate_block": block and {"first_pages": block["first_pages"], "second_pages": block["second_pages"]}},
        "crops": []}
    for index, region in enumerate(source_regions + output_regions):
        dpi = SOURCE_OCR_DPI if region["role"] == "source_pdf" else OUTPUT_OCR_DPI
        candidate["crops"].append(save_region_crop(root, manifest, region, run, work, repo, dpi,
                                                   f"{candidate['event_id']}_{index}.png"))
    return candidate


def build_order_candidate(unit_id, order, reader, source_tokens, source_index, reader_record,
                          source_record, tomus, root, manifest, run, work, repo, settings):
    """Assemble a reversal candidate for a later run that re-enters earlier source material."""
    target = order["to"]
    tokens = reader[target["output_start"]:target["output_end"]]
    source_slice = [source_tokens[i] for i in range(target["source_start"], target["source_end"])]
    output_regions = page_union_regions(tokens, "reader_pdf", reader_record, tomus=tomus)
    source_regions = page_union_regions(source_slice, "source_pdf", source_record, source_index=source_index)
    backward = order["from"]["canon_start"] - target["canon_start"]
    confidence = "low" if backward >= settings["thresholds"]["order_high"]["backward_tokens_min"] else "unknown"
    candidate = {
        "event_id": f"order_{unit_id}_{tokens[0]['page']}_{target['output_start']}",
        "unit_id": unit_id, "type": "order", "relation": "reversal",
        "source_regions": source_regions, "output_regions": output_regions,
        "confidence": confidence, "status": "candidate",
        "primary_score": round(target["ordered_similarity"], 4),
        "component_scores": {"backward_tokens": backward, "ordered_similarity": round(target["ordered_similarity"], 4)},
        "confidence_reasons": [f"backward_tokens>=64: {'satisfied' if backward >= settings['thresholds']['order_high']['backward_tokens_min'] else 'failed'}"],
        "linked_events": [],
        "evidence_provenance": {"method": "Chained reader anchors re-enter earlier canonical source material after a penalized restart, indicating a reversed reading order."},
        "crops": []}
    for index, region in enumerate(source_regions + output_regions):
        dpi = SOURCE_OCR_DPI if region["role"] == "source_pdf" else OUTPUT_OCR_DPI
        candidate["crops"].append(save_region_crop(root, manifest, region, run, work, repo, dpi,
                                                   f"{candidate['event_id']}_{index}.png"))
    return candidate


def cluster_page_lines(tokens):
    """Group one page's OCR tokens into text lines by vertical proximity of word centers."""
    rows = sorted(tokens, key=lambda token: (token["box"][1] + token["box"][3]) / 2)
    lines, current = [], []
    for token in rows:
        center = (token["box"][1] + token["box"][3]) / 2
        height = token["box"][3] - token["box"][1]
        if current and center - (current[-1]["box"][1] + current[-1]["box"][3]) / 2 > 0.6 * height:
            lines.append(current)
            current = [token]
        else:
            current.append(token)
    if current:
        lines.append(current)
    return lines


def measure_source_regions(gap_tokens, source_tokens, source_record, source_index):
    """Tightly bound absent source lines, taking the left edge from Latin-led lines only.

    The expected-Latin line box spans any Greek marginal scrap that opens a line, so the left
    edge is read from lines whose first token is Latin; the vertical extent and right edge still
    cover every missing line so the measured box matches hand-drawn Latin-column gold."""
    line_keys = {(token["page"], token["line"]) for token in gap_tokens}
    leading_latin = {}
    for token in source_tokens:
        key = (token["page"], token["line"])
        if key in line_keys and key not in leading_latin:
            leading_latin[key] = is_latin_token(strict_view(token["raw"]))
    line_box = {}
    for token in gap_tokens:
        line_box.setdefault((token["page"], token["line"]), token["box"])
    by_page = {}
    for key, box in line_box.items():
        by_page.setdefault(key[0], []).append((key, box))
    regions = []
    for page in sorted(by_page):
        entries = by_page[page]
        all_boxes = [box for _, box in entries]
        latin_boxes = [box for key, box in entries if leading_latin.get(key)] or all_boxes
        box = union_box([[min(b[0] for b in latin_boxes), min(b[1] for b in all_boxes),
                          max(b[2] for b in all_boxes), max(b[3] for b in all_boxes)]])
        regions.append({"role": "source_pdf", "page": page, "input_sha256": source_record["sha256"],
                        "box": box, "source_index": source_index, "occurrence_id": "expected"})
    return regions


def measure_seam_regions(reader, seam_index, tomus, reader_record, line_height, span):
    """Bound the output seam as the reader lines flanking the join, spanning the visual gap.

    The absent lines leave a blank band between the last read line and the one that resumes, so
    the seam box unions the clustered line before the join and the line at the join (plus any
    empty rows between them); ``span`` adds optional extra context lines on each side."""
    seam_index = max(0, min(seam_index, len(reader) - 1))
    page = reader[seam_index]["page"]
    page_tokens = [token for token in reader if token["page"] == page]
    lines = cluster_page_lines(page_tokens)

    def line_of(center):
        return min(range(len(lines)), key=lambda i: abs(
            center - sum((t["box"][1] + t["box"][3]) / 2 for t in lines[i]) / len(lines[i])))

    seam_line = line_of((reader[seam_index]["box"][1] + reader[seam_index]["box"][3]) / 2)
    pre = reader[seam_index - 1] if seam_index > 0 and reader[seam_index - 1]["page"] == page else reader[seam_index]
    pre_line = line_of((pre["box"][1] + pre["box"][3]) / 2)
    low, high = min(pre_line, seam_line) - span, max(pre_line, seam_line) + span
    chosen = [token for i in range(max(0, low), min(len(lines), high + 1)) for token in lines[i]]
    return [{"role": "reader_pdf", "page": page, "input_sha256": reader_record["sha256"],
             "tomus": tomus, "box": union_box([token["box"] for token in chosen]),
             "region_kind": "seam", "line_height": line_height}]


def build_missing_candidate(unit_id, omission, reader, source_tokens, reader_record, source_record,
                            source_index, tomus, line_height, root, manifest, run, work, repo, settings):
    """Assemble a missing-occurrence candidate: absent source lines at a located output seam."""
    gap_tokens = [source_tokens[i] for i in omission["gap"]]
    source_regions = measure_source_regions(gap_tokens, source_tokens, source_record, source_index)
    span = settings["thresholds"]["omission"]["seam_line_span"]
    output_regions = measure_seam_regions(reader, omission["seam_output"][0], tomus, reader_record,
                                          line_height, span)
    confidence = omission["confidence"]
    om = settings["thresholds"]["omission"]
    predicates = {"left_context_informative>=8": omission["left_informative"] >= om["context_min_informative"],
                  "right_context_informative>=8": omission["right_informative"] >= om["context_min_informative"],
                  "context_covered>=0.80": omission["ordered"] >= om["ordered_min"],
                  "unique_loss": omission["unique_loss"]}
    candidate = {
        "event_id": f"miss_{unit_id}_{source_regions[0]['page']}_{omission['gap'][0]}",
        "unit_id": unit_id, "type": "missing_occurrence", "relation": "missing_expected_position",
        "source_regions": source_regions, "output_regions": output_regions,
        "missing_line_count": omission["missing_line_count"], "unique_loss": omission["unique_loss"],
        "confidence": confidence, "status": "candidate",
        "primary_score": round(omission["ordered"], 4),
        "component_scores": {"context_covered": omission["ordered"],
                             "left_informative": omission["left_informative"],
                             "right_informative": omission["right_informative"],
                             "missing_tokens": len(omission["gap"])},
        "confidence_reasons": [f"{name}: {'satisfied' if value else 'failed'}" for name, value in predicates.items()]
                             + (["displaced copy elsewhere: order evidence, not unique loss"] if omission["displaced"] else []),
        "linked_events": [],
        "evidence_provenance": {"method": "Expected Latin lines between two context-correct reader anchors are deleted at the contiguous output seam under bounded local alignment; a copy at another output position would be order evidence, not coverage.",
                                "context_covered": omission["ordered"],
                                "displaced_copy": omission["displaced"]},
        "crops": []}
    for index, region in enumerate(source_regions + output_regions):
        dpi = SOURCE_OCR_DPI if region["role"] == "source_pdf" else OUTPUT_OCR_DPI
        candidate["crops"].append(save_region_crop(root, manifest, region, run, work, repo, dpi,
                                                   f"{candidate['event_id']}_{index}.png"))
    return candidate


def make_splits(units, body_pages, manifest):
    ids = {u["id"] for u in units}
    if len(ids) != 68 or not (DEVELOPMENT_UNITS | VALIDATION_UNITS) <= ids:
        raise AuditError("Unit inventory does not match required split units")
    assignments = {}
    for unit in sorted(ids):
        byte = hashlib.sha256(f"audit-split-v1|{manifest['baseline_id']}|{unit}".encode()).digest()[0]
        assignments[unit] = ("development" if unit in DEVELOPMENT_UNITS else
                             "validation" if unit in VALIDATION_UNITS else
                             "development" if byte % 2 == 0 else "validation")
    panel = []
    for tomus in range(1, 9):
        pages = [r for r in body_pages if r["tomus"] == tomus]
        pages.sort(key=lambda r: hashlib.sha256(
            f"audit-panel-v1|{manifest['baseline_id']}|{tomus}|{r['page']}".encode()).digest())
        panel.extend(dict(r, split=assignments[r["unit_id"]], review_status="pending") for r in pages[:25])
    return {"schema_version": 1, "baseline_id": manifest["baseline_id"],
            "report_sha256": manifest["report_sha256"],
            "development_units": sorted(u for u in ids if assignments[u] == "development"),
            "validation_units": sorted(u for u in ids if assignments[u] == "validation"),
            "selection_rule": "SHA-256 first byte parity with mandatory unit overrides; panel takes first 25 hash-sorted body pages per tomus",
            "unit_seed": "audit-split-v1", "panel_seed": "audit-panel-v1", "review_panel": panel,
            "head_folds": [{"held_out_unit": "philopoemen", "training_unit": "agis"},
                           {"held_out_unit": "agis", "training_unit": "philopoemen"}],
            "head_fold_limitation": "Both units participate in other development work; these are sensitivity runs, not independent validation."}


def validate_benchmark(data, manifest, units, *, splits=None):
    """Validate reviewed and pending benchmark records in one shared entry point."""
    if (data.get("schema_version") != 1 or data.get("baseline_id") != manifest["baseline_id"]
            or data.get("report_sha256") != manifest["report_sha256"]):
        raise AuditError("Benchmark baseline/report identity mismatch")
    unit_ids = {u["id"] for u in units}
    if "development_units" in data:
        dev, val = set(data["development_units"]), set(data["validation_units"])
        if dev & val or dev | val != unit_ids or not DEVELOPMENT_UNITS <= dev or not VALIDATION_UNITS <= val:
            raise AuditError("Split lists must be disjoint and cover the 68 pinned units")
        panel = data["review_panel"]
        if len({(r["tomus"], r["page"]) for r in panel}) != 200:
            raise AuditError("Review panel must contain 200 distinct pages")
        for tomus in range(1, 9):
            if sum(r["tomus"] == tomus for r in panel) != 25:
                raise AuditError("Panel must contain 25 pages per tomus")
        for row in panel:
            if row["unit_id"] not in unit_ids or row["split"] != ("development" if row["unit_id"] in dev else "validation"):
                raise AuditError("Panel has an invalid inherited unit split")
            if (row["tomus"] not in range(1, 9) or not 1 <= row["page"] <= manifest["expected_counts"]["physical_by_tomus"][row["tomus"] - 1]):
                raise AuditError("Review-panel page outside reader bounds")
            validate_box(row["body_box"])
            _, record = snapshot_input(Path("."), manifest, role="reader_pdf", tomus=row["tomus"])
            if row["input_sha256"] != record["sha256"]:
                raise AuditError("Review-panel reader hash mismatch")
        return data
    seen = set()
    for case in data.get("cases", data.get("templates", [])):
        identifier = case.get("case_id", case.get("template_id"))
        if (not isinstance(identifier, str) or not re.fullmatch(r"[A-Za-z0-9_]+", identifier)
                or identifier in seen or case["unit_id"] not in unit_ids):
            raise AuditError("Missing/duplicate annotation ID or invalid unit")
        seen.add(identifier)
        if case["annotation_status"] not in ("reviewed", "pending") or not case.get("annotation_provenance"):
            raise AuditError("Annotations need explicit status and provenance")
        if "label" in case and case["label"] not in ("positive", "negative", "complete_endpoint"):
            raise AuditError("Invalid annotation label")
        if not case.get("expected_relation") and "label" in case:
            raise AuditError("Missing expected relation")
        for region in case.get("source_regions", []) + case.get("output_regions", []):
            role = region["role"]
            if role not in ("source_pdf", "reader_pdf"):
                raise AuditError("Gold regions must refer to captured source/reader PDFs")
            _, record = snapshot_input(Path("."), manifest, role=role,
                                       tomus=region.get("tomus"), source_index=region.get("source_index"))
            if region.get("input_sha256") != record["sha256"]:
                raise AuditError("Gold region input hash mismatch")
            limit = manifest["expected_counts"]["physical_by_tomus"][region["tomus"] - 1] if role == "reader_pdf" else (637 if region["source_index"] == 0 else 710)
            if not isinstance(region["page"], int) or not 1 <= region["page"] <= limit:
                raise AuditError("Gold page outside pinned bounds")
            if region.get("box") is not None:
                validate_box(region["box"])
            elif case["annotation_status"] == "reviewed":
                raise AuditError("Reviewed gold regions need manually measured boxes")
        if case["annotation_status"] == "reviewed" and not case.get("output_regions") and "label" in case:
            raise AuditError("Reviewed cases require output localization")
        if splits and "templates" in data and case["unit_id"] not in splits["development_units"]:
            raise AuditError("Templates must belong to development source units")
        if "templates" in data:
            byte = hashlib.sha256(f"audit-split-v1|{manifest['baseline_id']}|{case['unit_id']}".encode()).digest()[0]
            if case["unit_id"] in VALIDATION_UNITS or (case["unit_id"] not in DEVELOPMENT_UNITS and byte % 2):
                raise AuditError("Validation-unit sources cannot become development templates")
        if "templates" in data and (case.get("output_regions") or len(case.get("source_regions", [])) != 1):
            raise AuditError("A template requires exactly one source region and no output region")
        if case.get("defect_type") == "duplicate" and case.get("label") == "positive":
            if len({r.get("occurrence_id") for r in case["source_regions"]}) != 2 or len(case.get("contributions", [])) != 2:
                raise AuditError("Duplicate gold needs both source occurrences and two output contributions")
            if case["annotation_status"] == "reviewed":
                for contribution in case["contributions"]:
                    validate_box(contribution["start_box"])
                    validate_box(contribution["end_box"])
                    if not isinstance(contribution.get("line_height"), (int, float)) or contribution["line_height"] <= 0:
                        raise AuditError("Reviewed duplicate endpoints need an independently measured reader line height")
        if case.get("case_id") in ("C1", "C2", "C3", "C5"):
            if case.get("missing_line_count") != {"C1": 4, "C2": 2, "C3": 1, "C5": 1}[case["case_id"]]:
                raise AuditError("Distinct omission counts must retain the final report's 4/2/1/1 lines")
    return data


def render_page(pdf, record, page_number, dpi, work, repo, *, pdftoppm=None):
    """Render the full MediaBox with Poppler; cache only verified completed rasters."""
    import pymupdf
    from PIL import Image
    executable = pdftoppm or shutil.which("pdftoppm")
    if not executable or not isinstance(dpi, int) or not 45 <= dpi <= 600:
        raise AuditError("Poppler must be available and DPI must be between 45 and 600")
    version = run_external([executable, "-v"]).splitlines()[0]
    with pymupdf.open(pdf) as document:
        if not 1 <= page_number <= len(document):
            raise AuditError("Render page outside input PDF bounds")
        geometry = page_geometry(document[page_number - 1])
    metadata = {"input_sha256": record["sha256"], "page": page_number, "dpi": dpi,
                "poppler_version": version, **geometry}
    key = hashlib.sha256(json.dumps(metadata, sort_keys=True).encode()).hexdigest()
    folder = safe_write_path(Path(work) / "cache" / "renders" / key, work, repo)
    raster, sidecar = folder / "page.png", folder / "metadata.json"
    cached = reusable_entry(sidecar, metadata, work, repo)
    if cached and raster.exists() and hash_file(raster) == cached.get("raster_sha256"):
        return raster, cached
    folder.mkdir(parents=True, exist_ok=True)
    lock = folder / "render.lock"
    try:
        descriptor = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
    except FileExistsError as exc:
        raise AuditError(f"Render is already locked; inspect any interrupted invocation before removing {lock}") from exc
    os.close(descriptor)
    try:
        with tempfile.TemporaryDirectory(dir=folder) as temporary:
            prefix = Path(temporary) / "page"
            with progress_heartbeat(f"Rendering {record['path']} page {page_number}"):
                run_external([executable, "-png", "-f", str(page_number), "-l", str(page_number),
                              "-r", str(dpi), "-singlefile", str(pdf), str(prefix)], timeout=180)
            rendered = prefix.with_suffix(".png")
            with Image.open(rendered) as image:
                image.load()
                dimensions = list(image.size)
            os.replace(rendered, safe_write_path(raster, work, repo))
        data = {**metadata, "raster_dimensions": dimensions, "raster_sha256": hash_file(raster),
                "unusual_coordinate_status": "requires independently inspected overlay" if
                geometry["rotation"] or geometry["cropbox"] != geometry["mediabox"] else "standard corpus geometry"}
        write_json(sidecar, {"schema_version": 1, "complete": True, "metadata": metadata, "data": data}, work, repo)
        progress(f"Rendered {record['path']} page {page_number} at {dpi} DPI")
        return raster, data
    finally:
        lock.unlink()


SOURCE_OCR_DPI = 300   # Source columns read at full resolution.
OUTPUT_OCR_DPI = 180   # Reader body pages read coarsely first.
RETRY_OCR_DPI = 300    # Implicated regions retried at full resolution.
SOURCE_PSM = 6         # Assume a single uniform block for a source column.
OUTPUT_PSM = 3         # Full automatic segmentation for a whole reader page.
RETRY_PSM = 11         # Sparse-text alternate for regional retries.
OCR_ADAPTER = "tesseract-tsv-v1"


def tesseract_version(executable):
    return run_external([executable, "--version"]).splitlines()[0]


def parse_tesseract_tsv(tsv):
    """Parse Tesseract TSV output into word rows with pixel boxes and raw confidence."""
    records = tsv.splitlines()
    if not records:
        return []
    column = {name: index for index, name in enumerate(records[0].split("\t"))}
    rows = []
    for line in records[1:]:
        fields = line.split("\t")
        if len(fields) <= column["text"] or int(fields[column["level"]]) != 5:
            continue
        text = fields[column["text"]]
        if not text.strip():
            continue
        left, top = int(fields[column["left"]]), int(fields[column["top"]])
        width, height = int(fields[column["width"]]), int(fields[column["height"]])
        rows.append({"line_id": [int(fields[column["block_num"]]), int(fields[column["par_num"]]),
                                 int(fields[column["line_num"]])],
                     "pixel_box": [left, top, left + width, top + height],
                     "conf": float(fields[column["conf"]]), "text": text})
    return rows


def run_tesseract_tsv(image_path, *, psm, lang, tesseract, timeout=240):
    """Run Tesseract once in TSV mode, preserving per-word boxes and confidence."""
    with tempfile.TemporaryDirectory() as temporary:
        base = Path(temporary) / "ocr"
        with progress_heartbeat(f"OCR {Path(image_path).name} psm {psm}"):
            run_external([tesseract, str(image_path), str(base), "--psm", str(psm),
                          "-l", lang, "tsv"], timeout=timeout)
        return parse_tesseract_tsv(base.with_suffix(".tsv").read_text(encoding="utf-8"))


def normalize_tsv_box(pixel_box, offset, dimensions):
    """Convert a cropped-raster TSV box back to the displayed full-raster [0,1] frame."""
    width, height = dimensions
    ox, oy = offset
    box = [(pixel_box[0] + ox) / width, (pixel_box[1] + oy) / height,
           (pixel_box[2] + ox) / width, (pixel_box[3] + oy) / height]
    return [min(max(value, 0.0), 1.0) for value in box]


def group_ocr_lines(words):
    """Group OCR words by Tesseract's own line identity, ordered left to right."""
    grouped = {}
    for word in words:
        grouped.setdefault(tuple(word["line_id"]), []).append(word)
    lines = []
    for line_id, items in grouped.items():
        ordered = sorted(items, key=lambda w: w["box"][0])
        lines.append({"line_id": list(line_id),
                      "box": [min(w["box"][0] for w in ordered), min(w["box"][1] for w in ordered),
                              max(w["box"][2] for w in ordered), max(w["box"][3] for w in ordered)],
                      "text": " ".join(w["text"] for w in ordered),
                      "mean_conf": sum(w["conf"] for w in ordered) / len(ordered)})
    return sorted(lines, key=lambda l: (l["box"][1], l["box"][0]))


def ocr_page_region(pdf, record, page_number, dpi, psm, lang, work, repo, *,
                    box=None, tesseract=None, pdftoppm=None, timeout=240):
    """Raster-OCR a full MediaBox page or a cropped region; boxes map to the full raster.

    Reuse is keyed on the rendered pixels plus OCR settings, never a bare filename. Blank
    output on inked body is recorded as unreadable/inconclusive, never as clean prose. The
    embedded source words gathered in the source-region cache remain a separate observation."""
    from PIL import Image
    executable = tesseract or shutil.which("tesseract")
    if not executable:
        raise AuditError("Tesseract must be available; supply --tesseract PATH.")
    raster, render_meta = render_page(pdf, record, page_number, dpi, work, repo, pdftoppm=pdftoppm)
    dimensions = render_meta["raster_dimensions"]
    version = tesseract_version(executable)
    metadata = {"adapter": OCR_ADAPTER, "raster_sha256": render_meta["raster_sha256"],
                "dpi": dpi, "psm": psm, "lang": lang, "tesseract_version": version,
                "crop": [round(value, 6) for value in box] if box else None}
    path = Path(work) / "cache" / "ocr" / f"{json_hash(metadata)}.json"
    cached = reusable_entry(path, metadata, work, repo)
    if cached:
        return cached
    clipped = False
    if box:
        crop_box = pixel_bounds(box, dimensions)
        offset = (crop_box[0], crop_box[1])
        clipped = crop_box[0] <= 0 or crop_box[1] <= 0 or crop_box[2] >= dimensions[0] or crop_box[3] >= dimensions[1]
        with Image.open(raster) as image:
            crop = image.crop(crop_box)
            with tempfile.TemporaryDirectory() as temporary:
                region_path = Path(temporary) / "region.png"
                crop.save(region_path)
                rows = run_tesseract_tsv(region_path, psm=psm, lang=lang, tesseract=executable, timeout=timeout)
    else:
        offset = (0, 0)
        rows = run_tesseract_tsv(raster, psm=psm, lang=lang, tesseract=executable, timeout=timeout)
    words = []
    for row in rows:
        normalized = normalize_tsv_box(row["pixel_box"], offset, dimensions)
        if normalized[0] >= normalized[2] or normalized[1] >= normalized[3]:
            continue
        words.append({"box": normalized, "text": row["text"], "conf": row["conf"], "line_id": row["line_id"]})
    data = {"adapter": OCR_ADAPTER, "input_sha256": record["sha256"], "path": record["path"],
            "role": record["role"], "page": page_number, "tomus": record.get("tomus"),
            "source_index": record.get("source_index"), "dpi": dpi, "psm": psm, "lang": lang,
            "provider": "tesseract_raster", "tesseract_version": version,
            "raster_dimensions": dimensions, "box": box, "clipped": clipped,
            "word_count": len(words), "readable": bool(words),
            "words": words, "lines": group_ocr_lines(words)}
    write_json(path, {"schema_version": 1, "complete": True, "metadata": metadata, "data": data}, work, repo)
    progress(f"OCR {record['path']} page {page_number} ({'region' if box else 'full'}) "
             f"dpi {dpi} psm {psm}: {len(words)} words{' [unreadable]' if not words else ''}")
    return data


def ocr_region_retry(pdf, record, page_number, box, work, repo, *,
                     tesseract=None, pdftoppm=None, psm=RETRY_PSM, lang="eng"):
    """Retry an implicated region at full resolution with an alternate segmentation mode."""
    return ocr_page_region(pdf, record, page_number, RETRY_OCR_DPI, psm, lang, work, repo,
                           box=box, tesseract=tesseract, pdftoppm=pdftoppm)


@contextmanager
def run_directory(path, work, repo):
    run = safe_write_path(path, work, repo)
    run.mkdir(parents=True, exist_ok=True)
    lock = safe_write_path(run / "run.lock", work, repo)
    try:
        descriptor = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
    except FileExistsError as exc:
        raise AuditError(f"Another invocation owns {lock}; inspect before removing a stale lock") from exc
    os.close(descriptor)
    try:
        yield run
    finally:
        lock.unlink()


def annotate_regions(args):
    """Render supplied gold or pending overview regions; never score or edit specs."""
    from PIL import Image, ImageDraw
    repo, work = Path(args.repo_root).resolve(), Path(args.work_dir).resolve()
    root, manifest = resolve_baseline(work, args.baseline)
    units_path, _ = snapshot_input(root, manifest, role="units")
    units = json.loads(units_path.read_text(encoding="utf-8"))
    labels_path = Path(args.labels).resolve()
    labels_hash = hash_file(labels_path)
    labels = validate_benchmark(json.loads(labels_path.read_text(encoding="utf-8")), manifest, units)
    records = labels.get("cases", labels.get("templates", []))
    records = [dict(c, case_id=c.get("case_id", c.get("template_id"))) for c in records]
    requested = set(args.cases or [c["case_id"] for c in records if c["annotation_status"] == "reviewed"])
    if not requested or requested - {c["case_id"] for c in records}:
        raise AuditError("Select existing --cases, or provide reviewed regions")
    run_path = args.run_dir or work / "runs" / f"annotations_{args.baseline}"
    with run_directory(run_path, work, repo) as run:
        output_manifest = run / "annotation-manifest.json"
        if output_manifest.exists():
            raise AuditError("Annotation run already exists; select a new --run-dir")
        drafts, sections = [], []
        for case in records:
            if case["case_id"] not in requested:
                continue
            thumbs, evidence = [], []
            for index, region in enumerate(case["source_regions"] + case["output_regions"]):
                pdf, record = snapshot_input(root, manifest, role=region["role"], tomus=region.get("tomus"), source_index=region.get("source_index"))
                raster, metadata = render_page(pdf, record, region["page"], args.dpi, work, repo, pdftoppm=args.pdftoppm)
                box = region.get("box") or [0, 0, 1, 1]
                target = safe_write_path(run / "crops" / f"{case['case_id']}_{index}.png", work, repo)
                target.parent.mkdir(parents=True, exist_ok=True)
                with Image.open(raster) as image:
                    crop = image.crop(pixel_bounds(box, image.size))
                    crop.save(target)
                    crop.thumbnail((520, 700))
                    thumb = Image.new("RGB", (540, 745), "white")
                    thumb.paste(crop.convert("RGB"), (10, 40))
                    ImageDraw.Draw(thumb).text((10, 10), f"{case['case_id']} {region['role']} p{region['page']} ({'gold' if region.get('box') else 'pending overview'})", fill="black")
                    thumbs.append(thumb)
                evidence.append({"region": region, "crop_path": target.relative_to(run).as_posix(), "render": metadata,
                                 "localization": "gold" if region.get("box") else "overview_not_gold"})
            sheet = Image.new("RGB", (540 * min(3, len(thumbs)), 745 * math.ceil(len(thumbs) / 3)), "white")
            for index, thumb in enumerate(thumbs):
                sheet.paste(thumb, ((index % 3) * 540, (index // 3) * 745))
            sheet_path = safe_write_path(run / f"{case['case_id']}_contact.png", work, repo)
            sheet.save(sheet_path)
            drafts.append(dict(case, evidence=evidence))
            sections.append(f"<h2>{html.escape(case['case_id'])}: {html.escape(case.get('defect_type', case.get('fragment_type', 'annotation')))}</h2><p>{html.escape(case['annotation_status'])}</p><img style='max-width:100%' src='{html.escape(sheet_path.name, quote=True)}'>")
            progress(f"Annotation evidence: {case['case_id']} ({len(evidence)} regions)")
        write_json(run / "annotation-draft.json", {"schema_version": 1, "baseline_id": args.baseline,
                   "report_sha256": manifest["report_sha256"], "labels_sha256": labels_hash, "cases": drafts}, work, repo)
        page = safe_write_path(run / "index.html", work, repo)
        page.write_text("<!doctype html><meta charset='utf-8'><title>Gold annotation review</title><h1>Annotation only: no detector scores</h1>" + "".join(sections), encoding="utf-8")
        resolve_baseline(work, args.baseline)
        if hash_file(labels_path) != labels_hash:
            raise AuditError("Labels changed during annotation")
        write_json(output_manifest, {"schema_version": 1, "baseline_id": args.baseline, "labels_sha256": labels_hash,
                   "cases": sorted(requested), "dpi": args.dpi, "annotation_only": True, "complete": True}, work, repo)
        progress(f"Annotation review: {page}")


def hash_file(path):
    digest = hashlib.sha256()
    last = time.monotonic()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
            if time.monotonic() - last >= 30:
                progress(f"Hashing {path}")
                last = time.monotonic()
    return digest.hexdigest()


def relative_path(value):
    path = Path(value)
    if path.is_absolute() or path.drive or ".." in path.parts or not path.parts:
        raise AuditError(f"Expected a relative path without traversal: {value}")
    return path


def contained(path, root):
    return path == root or root in path.parents


def safe_write_path(destination, work_dir, repo_root, *, capture_root=None):
    """Authorize actual resolved destinations, including junction/symlink targets."""
    destination = Path(destination).resolve()
    work = Path(work_dir).resolve()
    repo = Path(repo_root).resolve()
    protected = [repo / name for name in (
        "src", "build_facsimile", "cache", "transcription_cache", "transcriptions",
    )]
    protected.extend([work / "spec"])
    protected.extend(path.resolve() for path in repo.glob("*.pdf"))
    protected.append((repo / "facsimile_units.json").resolve())
    if any(contained(destination, path.resolve()) for path in protected):
        raise AuditError(f"Protected input/spec destination: {destination}")
    if capture_root is None:
        roots = [(work / "cache").resolve(), (work / "runs").resolve()]
        snapshot = (work / "snapshots").resolve()
        if contained(destination, snapshot):
            raise AuditError(f"Snapshot destination is read-only: {destination}")
    else:
        root = Path(capture_root).resolve()
        snapshots = work / "snapshots"
        if root.parent != snapshots or not contained(root, work):
            raise AuditError("Capture root must be a direct, non-redirected snapshot child.")
        if (root / "manifest.json").exists():
            raise AuditError(f"Completed snapshot cannot be overwritten: {root}")
        roots = [root]
    if not any(contained(destination, root) and contained(root, work) for root in roots):
        raise AuditError(f"Destination is outside the authorized output directories: {destination}")
    if destination.suffix.lower() == ".pdf" and capture_root is None:
        raise AuditError("Ordinary processing cannot write PDF inputs.")
    return destination


def write_json(path, data, work, repo, *, capture_root=None):
    path = safe_write_path(path, work, repo, capture_root=capture_root)
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=path.parent,
                                     suffix=".tmp", delete=False) as stream:
        temporary = Path(stream.name)
        json.dump(data, stream, ensure_ascii=False, indent=2)
        stream.write("\n")
    try:
        safe_write_path(path, work, repo, capture_root=capture_root)
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def validate_data(data):
    """Validate the small baseline and snapshot formats without a schema framework."""
    if data.get("schema_version") != 1:
        raise AuditError("Expected schema_version: 1")
    baseline_id = data.get("baseline_id", "")
    if not isinstance(baseline_id, str) or not re.fullmatch(r"[A-Za-z0-9_]+", baseline_id):
        raise AuditError("Invalid baseline ID; use letters, digits and underscores.")
    if data.get("render_box_policy") != "mediabox":
        raise AuditError("Expected render_box_policy: mediabox")
    records = data.get("inputs")
    if not isinstance(records, list) or not records:
        raise AuditError("Expected a nonempty inputs list")
    paths = set()
    for record in records:
        path = str(relative_path(record["path"]))
        if path in paths or not record.get("role"):
            raise AuditError(f"Duplicate input or missing role: {path}")
        paths.add(path)
        if not re.fullmatch(r"[0-9a-f]{64}", record.get("sha256", "")):
            raise AuditError(f"Invalid SHA-256: {path}")
        if "source_index" in record and record["source_index"] not in (0, 1):
            raise AuditError(f"Invalid source index: {path}")
        if "tomus" in record and record["tomus"] not in range(1, 9):
            raise AuditError(f"Invalid tomus: {path}")
        if "snapshot_path" in record:
            captured = relative_path(record["snapshot_path"])
            if captured != Path("inputs") / relative_path(record["path"]):
                raise AuditError(f"Snapshot must preserve the original relative tree: {path}")
    for key in ("report_path", "report_sha256", "expected_counts"):
        if key not in data:
            raise AuditError(f"Missing {key}")
    if not any(r["path"] == data["report_path"] and r["sha256"] == data["report_sha256"]
               and r["role"] == "report" for r in records):
        raise AuditError("Report identity must match its input record")
    return data


def resolve_baseline(work_dir, baseline_id):
    if not re.fullmatch(r"[A-Za-z0-9_]+", baseline_id):
        raise AuditError("Invalid baseline ID")
    root = Path(work_dir).resolve() / "snapshots" / baseline_id
    if root.resolve() != root:
        raise AuditError("Snapshot path redirects outside its original location")
    if (root / "manifest.json").resolve() != root / "manifest.json":
        raise AuditError("Snapshot manifest redirects outside its original location")
    manifest = validate_data(json.loads((root / "manifest.json").read_text(encoding="utf-8")))
    if manifest["baseline_id"] != baseline_id or manifest.get("capture_complete") is not True:
        raise AuditError("Snapshot is incomplete or has a different baseline ID")
    for record in manifest["inputs"]:
        path = root / relative_path(record["snapshot_path"])
        if not contained(path.resolve(), root) or hash_file(path) != record["sha256"]:
            raise AuditError(f"Snapshot identity mismatch: {path}")
        progress(f"Verified captured input: {record['path']}")
    return root, manifest


def capture_inputs(args):
    repo, work = Path(args.repo_root).resolve(), Path(args.work_dir).resolve()
    spec_path = Path(args.baseline_spec).resolve()
    spec_hash = hash_file(spec_path)
    spec = validate_data(json.loads(spec_path.read_text(encoding="utf-8")))
    root = work / "snapshots" / spec["baseline_id"]
    safe_write_path(root / "manifest.json", work, repo, capture_root=root)
    if root.exists():
        raise AuditError(f"Snapshot already exists; preserve it and use a new baseline ID: {root}")
    originals = []
    for record in spec["inputs"]:
        original = (repo / relative_path(record["path"])).resolve()
        if not contained(original, repo) or hash_file(original) != record["sha256"]:
            raise AuditError(f"Pinned input mismatch: {original}; locate a byte-identical preserved copy.")
        originals.append(original)
        progress(f"Verified original: {record['path']}")
    root.parent.mkdir(parents=True, exist_ok=True)
    root.mkdir()  # Exclusive creation prevents concurrent capture of this baseline.
    captured = []
    for record, original in zip(spec["inputs"], originals):
        relative = Path("inputs") / relative_path(record["path"])
        destination = safe_write_path(root / relative, work, repo, capture_root=root)
        destination.parent.mkdir(parents=True, exist_ok=True)
        with original.open("rb") as source, destination.open("xb") as output:
            last = time.monotonic()
            while block := source.read(1024 * 1024):
                output.write(block)
                if time.monotonic() - last >= 30:
                    progress(f"Copying {record['path']}")
                    last = time.monotonic()
        if hash_file(destination) != record["sha256"]:
            raise AuditError(f"Captured copy hash mismatch: {destination}")
        captured.append(dict(record, snapshot_path=relative.as_posix()))
        progress(f"Copied and verified: {record['path']}")
    for record, original in zip(spec["inputs"], originals):
        if hash_file(original) != record["sha256"]:
            raise AuditError(f"Original changed during capture: {original}")
    if hash_file(spec_path) != spec_hash:
        raise AuditError("Baseline spec changed during capture")
    manifest = dict(spec, spec_sha256=spec_hash, inputs=captured, capture_complete=True)
    write_json(root / "manifest.json", manifest, work, repo, capture_root=root)
    progress(f"Completed read-only snapshot: {root}")


def progress(message):
    print(message, flush=True)


@contextmanager
def progress_heartbeat(message):
    stopped = threading.Event()

    def report():
        while not stopped.wait(30):
            progress(message)

    worker = threading.Thread(target=report, daemon=True)
    worker.start()
    try:
        yield
    finally:
        stopped.set()
        worker.join()


def reusable_entry(path, metadata, work_dir, repo_root):
    """Return completed JSON diagnostic data only when all identity/settings match."""
    path = safe_write_path(path, work_dir, repo_root)
    try:
        entry = json.loads(path.read_text(encoding="utf-8"))
        if (entry.get("schema_version") == 1 and entry.get("complete") is True
                and entry.get("metadata") == metadata and "data" in entry):
            return entry["data"]
    except (OSError, ValueError, AttributeError):
        pass
    return None


def run_external(arguments, timeout=30):
    """Execute an argument list without a shell or a visible Windows window."""
    try:
        result = subprocess.run(
            arguments, shell=False, capture_output=True, text=True,
            encoding="utf-8", errors="replace", timeout=timeout,
            creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise AuditError(f"Cannot run {arguments[0]}: {exc}") from exc
    if result.returncode:
        raise AuditError(f"{arguments[0]} exited {result.returncode}: {result.stderr.strip()}")
    return (result.stdout + result.stderr).strip()


def check_prerequisites(args):
    progress(f"Existing interpreter: {sys.executable} ({sys.version.split()[0]})")
    errors = []
    versions = {}
    if sys.version_info < (3, 10):
        errors.append("Use an existing Python 3.10 or newer interpreter.")
    for distribution, module in PACKAGES.items():
        try:
            with progress_heartbeat(f"Checking import: {distribution}"):
                importlib.import_module(module)
            versions[distribution] = importlib.metadata.version(distribution)
            progress(f"{distribution}: {versions[distribution]}")
        except Exception as exc:
            errors.append(f"{distribution}: {exc}")
    tools = {}
    for name, option in (("pdftoppm", args.pdftoppm), ("tesseract", args.tesseract)):
        executable = option or shutil.which(name)
        if not executable:
            errors.append(f"Cannot resolve {name}; supply --{name} PATH.")
            continue
        try:
            output = run_external([executable, "-v" if name == "pdftoppm" else "--version"])
            tools[name] = {"path": str(Path(executable).resolve()), "version": output.splitlines()[0]}
            progress(f"{name}: {tools[name]}")
            if name == "tesseract":
                models = run_external([executable, "--list-langs"])
                tools[name]["models"] = models.splitlines()[1:]
                progress(models)
                if "eng" not in tools[name]["models"]:
                    errors.append("Tesseract English model (eng) is required; select an installed model directory.")
        except AuditError as exc:
            errors.append(str(exc))
    if errors:
        raise AuditError("Prerequisite check failed:\n" + "\n".join(errors) +
                         f'\nInstall dependencies separately using "{sys.executable}" -m pip install -r tools/requirements-inner-audit.txt')
    progress(json.dumps({"python": sys.version, "packages": versions, "tools": tools}, indent=2))
    return {"python": sys.version, "packages": versions, "tools": tools}


def make_parser():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    check = commands.add_parser("check", help="Check the existing interpreter, imports, external tools and OCR models")
    check.add_argument("--pdftoppm", help="Explicit Poppler executable path")
    check.add_argument("--tesseract", help="Explicit Tesseract executable path")
    capture = commands.add_parser("capture", help="Capture and verify pinned inputs into a new snapshot")
    capture.add_argument("--repo-root", default=".")
    capture.add_argument("--work-dir", default="tmp/pdfs/inner_tomi_audit")
    capture.add_argument("--baseline-spec", required=True)
    annotate = commands.add_parser("annotate", help="Render gold regions/contact sheets without running detectors or changing specs")
    annotate.add_argument("--repo-root", default=".")
    annotate.add_argument("--work-dir", default="tmp/pdfs/inner_tomi_audit")
    annotate.add_argument("--baseline", required=True)
    annotate.add_argument("--labels", required=True)
    annotate.add_argument("--run-dir")
    annotate.add_argument("--cases", nargs="+")
    annotate.add_argument("--dpi", type=int, default=300)
    annotate.add_argument("--pdftoppm")
    evaluate = commands.add_parser("evaluate", help="Evaluate localized candidate evidence against reviewed benchmark regions")
    evaluate.add_argument("--repo-root", default=".")
    evaluate.add_argument("--work-dir", default="tmp/pdfs/inner_tomi_audit")
    evaluate.add_argument("--run-dir", required=True)
    evaluate.add_argument("--split", choices=("development", "validation", "all"), required=True)
    evaluate.add_argument("--freeze", action="store_true", help="Freeze a complete final development detector run")
    evaluate.add_argument("--enforce-gates", action="store_true")
    run = commands.add_parser("run", help="OCR selected readers/sources and nominate duplicate-scan, reader-order and missing-occurrence candidates; remaining detectors are reported as pending")
    run.add_argument("--repo-root", default=".")
    run.add_argument("--work-dir", default="tmp/pdfs/inner_tomi_audit")
    run.add_argument("--baseline", required=True)
    run.add_argument("--scope", choices=("development", "validation", "all"), default="development")
    run.add_argument("--units", nargs="+")
    run.add_argument("--tomus", nargs="+", type=int, choices=range(1, 9))
    run.add_argument("--detectors", choices=tuple(DETECTOR_TYPES), default="core")
    run.add_argument("--run-dir", required=True)
    run.add_argument("--freeze", help="Explicit development freeze JSON")
    run.add_argument("--config", help="Semantic settings JSON; must match the freeze")
    run.add_argument("--resume", action="store_true")
    run.add_argument("--workers", type=int, choices=range(1, 5), default=1)
    run.add_argument("--lang", default="eng", help="Tesseract OCR model language(s)")
    run.add_argument("--pdftoppm", help="Explicit Poppler executable path")
    run.add_argument("--tesseract", help="Explicit Tesseract executable path")
    return parser


def main():
    args = make_parser().parse_args()
    try:
        if args.command == "check":
            check_prerequisites(args)
        elif args.command == "capture":
            capture_inputs(args)
        elif args.command == "annotate":
            annotate_regions(args)
        elif args.command == "evaluate":
            return evaluate_run(args)
        elif args.command == "run":
            return request_run(args)
        return 0
    except CalibrationError as exc:
        print(f"Calibration gate failed: {exc}", file=sys.stderr, flush=True)
        return 4
    except (AuditError, OSError, ValueError, KeyError, TypeError) as exc:
        print(f"Configuration/tool error: {exc}", file=sys.stderr, flush=True)
        return 2
    except KeyboardInterrupt:
        print("Processing interrupted; requested work is incomplete.", file=sys.stderr)
        return 3


if __name__ == "__main__":
    sys.exit(main())
