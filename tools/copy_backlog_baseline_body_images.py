import hashlib
import json
import shutil
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "tmp/pdfs/backlog_artifacts/baseline"
manifest = json.loads((BASELINE / "baseline_volume_manifest.json").read_text(encoding="utf-8"))
pages = json.loads((BASELINE / "baseline_reader_pages.json").read_text(encoding="utf-8"))
prior_manifest = BASELINE / "baseline_body_images.json"
expected_hashes = (
    {record["path"]: record["sha256"] for record in json.loads(prior_manifest.read_text(encoding="utf-8"))}
    if prior_manifest.exists() else {}
)
records = []
for page in pages:
    unit = next((item for item in manifest[page["tomus"] - 1]["units"]
                 if item["opening_physical_page"] <= page["physical_page"]
                 < item["opening_physical_page"] + item["body_image_count"]), None)
    if unit is None:
        continue
    image_index = page["physical_page"] - unit["opening_physical_page"] + 1
    image_name = f"facsimile_{unit['unit_id']}_{image_index}.png"
    source = ROOT / "build_facsimile" / image_name
    destination = BASELINE / image_name
    if not destination.exists():
        assert source.exists(), f"missing baseline body image: {source}"
        source_hash = hashlib.sha256(source.read_bytes()).hexdigest()
        assert not expected_hashes or source_hash == expected_hashes[str(destination.relative_to(ROOT)).replace("\\", "/")], source
        shutil.copy2(source, destination)
    destination_hash = hashlib.sha256(destination.read_bytes()).hexdigest()
    assert not expected_hashes or destination_hash == expected_hashes[str(destination.relative_to(ROOT)).replace("\\", "/")], destination
    records.append({
        "tomus": page["tomus"],
        "physical_page": page["physical_page"],
        "unit_id": unit["unit_id"],
        "canvas_index": image_index,
        "path": str(destination.relative_to(ROOT)).replace("\\", "/"),
        "sha256": destination_hash,
    })

out = BASELINE / "baseline_body_images.json"
out.write_text(json.dumps(records, indent=2) + "\n", encoding="utf-8")
print(out, len(records))
