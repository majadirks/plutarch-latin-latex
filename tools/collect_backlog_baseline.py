import hashlib
import json
import re
from pathlib import Path

import fitz


ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "tmp/pdfs/backlog_artifacts/baseline"
UNITS = {unit["id"]: unit for unit in json.loads((ROOT / "facsimile_units.json").read_text(encoding="utf-8"))}


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


manifest = []
for volume in range(1, 9):
    tex_path = BASELINE / f"tomus_{volume}_inner.tex"
    pdf_path = BASELINE / f"tomus_{volume}_inner.pdf"
    tex = tex_path.read_text(encoding="utf-8")
    doc = fitz.open(pdf_path)
    blocks = tex.split("\\unitrecto")
    ordered = []
    for block in blocks:
        images = re.findall(r"\{facsimile_([a-z0-9_]+)_([0-9]+)\.png\}", block)
        if not images:
            continue
        unit_id = images[0][0]
        assert unit_id in UNITS, unit_id
        assert all(name == unit_id and int(number) == i + 1 for i, (name, number) in enumerate(images))
        ordered.append((unit_id, len(images)))
    assert ordered, volume
    first_id = ordered[0][0]
    title = UNITS[first_id]["latin_title"]
    openings = [i + 1 for i, page in enumerate(doc) if title in " ".join(page.get_text().splitlines()) and page.get_images()]
    assert openings, (volume, first_id, title)
    next_open = openings[0]
    rows = []
    for unit_id, image_count in ordered:
        page = doc[next_open - 1]
        assert UNITS[unit_id]["latin_title"] in " ".join(page.get_text().splitlines()), (volume, unit_id, next_open)
        for offset in range(image_count):
            body_page = doc[next_open + offset - 1]
            assert len(body_page.get_images()) == 1, (volume, unit_id, next_open + offset)
        rows.append({"unit_id": unit_id, "body_image_count": image_count, "opening_physical_page": next_open})
        next_open += image_count
        if next_open % 2 == 0:
            next_open += 1
    last_body = rows[-1]["opening_physical_page"] + rows[-1]["body_image_count"] - 1
    trailing = len(doc) - last_body - 4
    assert 0 <= trailing < 32, (volume, trailing)
    manifest.append({
        "tomus": volume,
        "units": rows,
        "inner_pdf_pages": len(doc),
        "back_flyleaf_pages": 4,
        "trailing_signature_blank_count": trailing,
        "tex_path": str(tex_path.relative_to(ROOT)).replace("\\", "/"),
        "tex_sha256": sha256(tex_path),
        "pdf_path": str(pdf_path.relative_to(ROOT)).replace("\\", "/"),
        "pdf_sha256": sha256(pdf_path),
    })

out = BASELINE / "baseline_volume_manifest.json"
out.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
print(out)
for volume in manifest:
    print(volume["tomus"], len(volume["units"]), volume["inner_pdf_pages"], volume["trailing_signature_blank_count"])
