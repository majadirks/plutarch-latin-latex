import hashlib
import json
from pathlib import Path

import fitz


ROOT = Path(__file__).resolve().parents[1]
BASELINE = ROOT / "tmp/pdfs/backlog_artifacts/baseline"
PAGES = {
    1: [22, 23, 24, 113, 212, 213, 340],
    2: [20, 21, 81, 139, 177, 178, 209, 210, 223, 248, 249, 290],
    3: [273, 274, 275],
    5: [196, 197, 198, 199],
    7: [44, 45, 46, 47, 287, 291, 344],
}

records = []
for volume, pages in PAGES.items():
    source = BASELINE / f"tomus_{volume}_inner.pdf"
    with fitz.open(source) as pdf:
        for page_number in pages:
            pix = pdf[page_number - 1].get_pixmap(matrix=fitz.Matrix(300 / 72, 300 / 72), colorspace=fitz.csGRAY, alpha=False)
            name = f"tomus_{volume}_physical_{page_number:03}.png"
            path = BASELINE / name
            pix.save(path)
            records.append({
                "tomus": volume,
                "physical_page": page_number,
                "path": str(path.relative_to(ROOT)).replace("\\", "/"),
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "raster": [pix.width, pix.height],
                "command": f"python tmp/render_backlog_baseline_pages.py",
            })

out = BASELINE / "baseline_reader_pages.json"
out.write_text(json.dumps(records, indent=2) + "\n", encoding="utf-8")
print(out, len(records))
