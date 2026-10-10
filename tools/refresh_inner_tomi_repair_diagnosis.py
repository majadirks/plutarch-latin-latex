"""Bind finite omission diagnoses to measured production source-to-canvas traces."""

import argparse
import hashlib
import json
import math
from pathlib import Path
import shutil


FIXTURE = Path("src/facsimile/fixtures/inner_tomi_audit_repair_source_trace.json")


def evidence(path):
    return {"path": path.as_posix(), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def covering(bands, top, bottom):
    return [b for b in bands if b[0] <= top and b[1] >= bottom]


def line_coverage(line, trace):
    page = next(p for p in trace["pages"] if p["source_page"] == line["source_page"])
    left, top, right, bottom = line["region"]
    crop = page["crop_edges"]
    bands = page["retained_trace"]["bands"]
    strips = [s for s in page["strips"] if covering([s["rows"]], top, bottom)]
    result = {"crop_edges": crop, "raw_covering_bands": covering(page["raw_bands"], top, bottom),
              "zone_covering_bands": covering(page["zone_and_gap_filtered_bands"], top, bottom),
              "pass_a_covering_bands": covering(page["pass_a_bands"], top, bottom),
              "pass_b_covering_bands": covering(bands, top, bottom), "extracted_strips": strips}
    stage = "survives"
    if left < crop[0] or right > crop[1]:
        stage = "column_detection"
    else:
        for key, name in [("pass_a_covering_bands", "pass_a"),
                          ("pass_b_covering_bands", "pass_b"), ("extracted_strips", "strip_filter")]:
            if len(result[key]) != 1:
                stage = "line_bands" if name == "pass_a" and not result["raw_covering_bands"] else name
                break
    result["scaled_row_stream"] = None
    result["returned_canvas_membership"] = []
    if len(strips) == 1:
        strip = strips[0]
        a, b = strip["rows"]
        lo, hi = strip["scaled_rows"]
        ratio = (hi - lo) / (b - a)
        interval = [lo + math.floor((top - a) * ratio), lo + math.ceil((bottom - a) * ratio)]
        result["scaled_row_stream"] = interval
        members = [c for c in trace["canvas_ranges"] if c[1] < interval[1] and c[2] > interval[0]]
        result["returned_canvas_membership"] = members
        if stage == "survives" and any(sum(a <= row < b for _, a, b in members) != 1
                                        for row in range(*interval)):
            stage = "canvas_filter"
    result["scaled_right_boundary"] = math.ceil((right - crop[0]) * trace["scale"])
    if stage == "survives" and result["scaled_right_boundary"] > 1004:
        stage = "canvas_width"
    result["first_loss_stage"] = stage
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--stage", required=True)
    args = parser.parse_args()
    fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))
    assert args.stage == fixture["diagnosis_stage"]
    events = [e for e in fixture["events"] if e["initial_tier"] == "A" or e.get("promoted_tier") == "A"]
    assert len(events) >= 39 and sum(e["missing_line_count"] for e in events) >= 69
    archive = Path("archive/inner_tomi_audit_repairs_2026_10_08") / args.stage
    source = Path("tmp/pdfs/inner_tomi_audit_repairs") / ("omissions-" + args.stage)
    assert source.is_dir()
    traces = {}
    manifest = []
    for unit in sorted({e["unit_id"] for e in events}):
        target = archive / unit / "trace.json"
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source / unit / "trace.json", target)
        traces[unit] = (json.loads(target.read_text(encoding="utf-8")), evidence(target))
        manifest.append(evidence(target))
    manifest_path = archive / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    fixture.setdefault("diagnosis_archives", {})[args.stage] = evidence(manifest_path)
    complete = 0
    for event in events:
        trace, reference = traces[event["unit_id"]]
        diagnosis = event["diagnosis"]
        lines = diagnosis["reviewed_lines"]
        assert len(lines) == event["missing_line_count"]
        coverage = [dict(source_page=l["source_page"], region=l["region"],
                         baseline_loss=l["baseline"]["first_loss_stage"], measured=line_coverage(l, trace)) for l in lines]
        survives = all(l["measured"]["first_loss_stage"] == "survives" for l in coverage)
        complete += survives
        corrections = [r for r in fixture["planned_body_band_repairs"] if event["event_id"] in r["event_ids"]]
        baseline = json.loads(Path(diagnosis["baseline_trace"]["path"]).read_text(encoding="utf-8"))
        diagnosis[args.stage.replace("-", "_")] = dict(trace=reference, metrics=trace["metrics"],
            baseline_metrics=baseline["metrics"], metrics_changed=baseline["metrics"] != trace["metrics"],
            scale=trace["scale"], source_pages=[t["page"] for t in trace["source_pages"] if t["retained_index"] is not None],
            lines=coverage, complete=survives, proposed_body_bands=corrections,
            proposed_crop_cases=[c for c in fixture["candidate_crop_cases"] if c["source"] == event["source_regions"][0]["source_index"]
                                 and c["source_page"] == event["source_regions"][0]["page"]])
        diagnosis["status"] = "measured_complete" if survives else "safe_correction_specified"
        print(event["event_id"], len(lines), "complete" if survives else
              ",".join(sorted({l["measured"]["first_loss_stage"] for l in coverage})))
    FIXTURE.write_text(json.dumps(fixture, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"{args.stage}: {len(events)} events / {sum(e['missing_line_count'] for e in events)} lines; {complete} complete; {len(events)-complete} require restoration")


if __name__ == "__main__":
    main()
