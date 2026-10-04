"""Spike: auto-detect Life/Comparatio openings for the facsimile unit table.

Two deterministic signals, validated here, replace the hand-authored
`facsimile_units.json`:

1. The LARGE display-title band. A Life/Comparatio opens with its name set in big
   type inside the Latin column; that line-band is far taller than a body line, so
   a band whose height exceeds a multiple of the page's median body-line height,
   sitting in the upper part of the column, marks an opening page.

2. The embedded OCR text layer. `pdftotext` on the source PDF already carries the
   Latin title (display title and the per-page running head), so the title string
   can be read deterministically — no vision call. Tesseract on the title band is
   the fallback when the OCR layer is absent.

Run: python spikes/facsimile_unit_detect_spike.py
(needs pdftoppm + pdftotext on PATH, numpy, Pillow)
"""

import subprocess
import tempfile
from pathlib import Path

import numpy as np
from PIL import Image

PDF = "Ploutarchou_Bioi_v1.pdf"
DPI = 300
# Known openings (page, expected title) for validation.
KNOWN = [
    (14, "THESEUS"),
    (34, "ROMULUS"),
    (56, "COMPARATIO"),
    (60, "LYCURGUS"),
    (84, "NUMA"),
    (102, "COMPARATIO"),
    (108, "SOLON"),
]
# Some interior body pages that must NOT be flagged as openings.
NEGATIVES = [20, 40, 70, 90, 120]


def ink(a, thr=170):
    return (a < thr).astype(np.int32)


def smooth(v, k=9):
    return np.convolve(v, np.ones(k) / k, mode="same")


def detect_latin_column(a):
    H, W = a.shape
    col = smooth(ink(a).sum(axis=0))
    cmax = col.max()
    lo, hi = int(0.40 * W), int(0.62 * W)
    gutter = lo + int(np.argmin(col[lo:hi]))
    x = gutter
    while x < W - 1 and col[x] < 0.35 * cmax:
        x += 1
    while x > gutter and col[x] > 0.20 * cmax:
        x -= 1
    left = x + 1
    x = W - 1
    while x > left and col[x] < 0.18 * cmax:
        x -= 1
    return left, min(W, x + 8)


def line_bands(col_ink, min_ink=3):
    row = col_ink.sum(axis=1)
    texty = row > min_ink
    bands, y, H = [], 0, len(texty)
    while y < H:
        if texty[y]:
            s = y
            while y < H and texty[y]:
                y += 1
            bands.append((s, y))
        else:
            y += 1
    return bands


def render(page, out):
    prefix = out / f"p{page}"
    subprocess.run(
        ["pdftoppm", "-png", "-f", str(page), "-l", str(page), "-r", str(DPI),
         "-singlefile", PDF, str(prefix)],
        check=True, capture_output=True,
    )
    return np.asarray(Image.open(f"{prefix}.png").convert("L"))


def ocr_title(page):
    """First substantial ALL-CAPS token(s) from the page's OCR text layer."""
    txt = subprocess.run(
        ["pdftotext", "-f", str(page), "-l", str(page), PDF, "-"],
        check=True, capture_output=True, text=True,
    ).stdout
    for raw in txt.splitlines():
        line = "".join(c for c in raw if c.isalpha() or c in " Ææ").strip()
        caps = [w for w in line.split() if len(w) >= 3 and w == w.upper()]
        if caps:
            return " ".join(caps)
    return ""


def header_band_ratio(a):
    """Tallest band in the top third of the column / median band height."""
    L, R = detect_latin_column(a)
    col_ink = ink(a[:, L:R])
    bands = line_bands(col_ink)
    H = a.shape[0]
    body = [b for b in bands if 0.065 * H <= (b[0] + b[1]) / 2 <= 0.95 * H]
    if len(body) < 5:
        return 0.0
    heights = np.array([b[1] - b[0] for b in body])
    med = np.median(heights)
    top = [b for b in body if (b[0] + b[1]) / 2 <= 0.33 * H]
    if not top:
        return 0.0
    return max(b[1] - b[0] for b in top) / med


def main():
    with tempfile.TemporaryDirectory() as td:
        out = Path(td)
        print("OPENINGS (expect a tall header band + a title):")
        for page, want in KNOWN:
            a = render(page, out)
            ratio = header_band_ratio(a)
            title = ocr_title(page)
            ok = ratio >= 1.6 and want in title.upper()
            print(f"  p{page:>3}  header/body={ratio:4.1f}  title={title!r:35} "
                  f"{'OK' if ok else 'CHECK'}")
        print("INTERIOR pages (expect NO tall header band):")
        for page in NEGATIVES:
            a = render(page, out)
            ratio = header_band_ratio(a)
            print(f"  p{page:>3}  header/body={ratio:4.1f}  "
                  f"{'OK' if ratio < 1.6 else 'FALSE-POSITIVE'}")


if __name__ == "__main__":
    main()
