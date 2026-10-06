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
    settings = json.loads(Path(args.config).read_text(encoding="utf-8")) if args.config else {}
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
    safe_write_path(args.run_dir, work, repo)
    progress(f"Authorized {args.scope} scope: {len(selected)} units, {args.detectors} detectors")
    raise AuditError("Detector stages are not implemented yet; no reader OCR, alignment or detector scores were produced. Annotation and metadata preparation are available.")


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
    run = commands.add_parser("run", help="Check protected detector scope; extraction/detection stages are not implemented yet")
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
            request_run(args)
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
