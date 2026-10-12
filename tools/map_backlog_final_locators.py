import hashlib
import json
from pathlib import Path
import fitz

root = Path(__file__).resolve().parents[1]
fixture = json.loads((root / "src/facsimile/fixtures/backlog_artifact_source_trace.json").read_text(encoding="utf-8"))
manifest = json.loads((root / "tmp/pdfs/backlog_artifacts/final/final_volume_manifest.json").read_text(encoding="utf-8"))
units = {row["unit_id"]: (volume["tomus"], row) for volume in manifest for row in volume["units"]}
trace_root = root / "tmp/pdfs/inner_tomi_audit_repairs"
final_dir = root / "tmp/pdfs/backlog_artifacts/final"
final_dir.mkdir(parents=True, exist_ok=True)
docs = {v["tomus"]: fitz.open(root / v["pdf_path"]) for v in manifest}
for record in fixture["repairs"]:
    ident = record["backlog_id"]
    unit = record["unit_id"]
    tomus, unit_manifest = units[unit]
    if ident in {"V1", "VII1"}:
        indices = [record["unmerged_final_canvas"]["canvas_index"] - 1]
    else:
        group = "backlog_artifacts_after_masks" if record.get("disposition") == "mask" else "backlog_artifacts_final"
        trace = json.loads((trace_root / group / unit / "trace.json").read_text(encoding="utf-8"))
        indices = []
        for line in record["reviewed_lines"]:
            page = next(row for row in trace["pages"] if row["source"] == line["source"] and row["source_page"] == line["source_page"])
            top, bottom = line["region"][1], line["region"][3]
            strips = [s for s in page["strips"] if s["rows"][0] <= top and s["rows"][1] >= bottom]
            assert len(strips) == 1, (ident, line, len(strips))
            start, end = strips[0]["scaled_rows"]
            owners = [entry[0] for entry in trace["canvas_ranges"] if entry[1] < end and entry[2] > start]
            assert len(owners) == 1, (ident, line, owners)
            indices.extend(owners)
        indices = sorted(set(indices))
    assert max(indices) <= unit_manifest["body_image_count"], (ident, indices, unit_manifest)
    physical = [unit_manifest["opening_physical_page"] + index - 1 for index in indices]
    folios = []
    page_hashes = []
    page_paths = []
    png_hashes = []
    for number in physical:
        page = docs[tomus][number - 1]
        nums = [line.strip() for line in page.get_text().splitlines() if line.strip().isdigit()]
        assert len(nums) <= 1, (ident, tomus, number, nums)
        folios.append(int(nums[0]) if nums else None)
        image = page.get_pixmap(matrix=fitz.Matrix(2, 2), alpha=False)
        digest = hashlib.sha256(image.samples).hexdigest()
        page_hashes.append(digest)
        out = final_dir / f"{ident}-tomus{tomus}-page{number}.png"
        image.save(out)
        page_paths.append(str(out.relative_to(root)).replace("\\", "/"))
        png_hashes.append(hashlib.sha256(out.read_bytes()).hexdigest())
    record["final_reader"] = {"tomus": tomus, "physical_pages": physical, "folios": folios, "body_canvas_indices": indices, "rendered_page_sha256": page_hashes, "page_png_paths": page_paths, "page_png_sha256": png_hashes, "render_scale": 2}
    print(ident, tomus, physical, folios, indices)
(root / "src/facsimile/fixtures/backlog_artifact_source_trace.json").write_text(json.dumps(fixture, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
