import numpy as np
from PIL import Image, ImageDraw

D = r"C:/Users/majad/AppData/Local/Temp/claude/pbspike"
DPI = 300

def load(name):
    img = Image.open(f"{D}/{name}").convert("L")
    return img, np.asarray(img)

def ink_of(a, thr=170):
    return (a < thr).astype(np.int32)

def smooth(v, k=9):
    return np.convolve(v, np.ones(k)/k, mode="same")

def detect_latin_column(a):
    """Return (L, R): left/right px of the dense Latin text block, excluding the
    gutter marginal letters by cutting at the dense-text edge, not the first ink."""
    H, W = a.shape
    ink = ink_of(a)
    col = smooth(ink.sum(axis=0))
    cmax = col.max()
    # Gutter = lowest-ink column in the central band.
    lo, hi = int(0.40*W), int(0.62*W)
    gutter = lo + int(np.argmin(col[lo:hi]))
    # Find a column firmly inside the dense Latin body, then back off left to the
    # text block's own left-margin whitespace. The sparse marginal letters live in
    # the gutter across a gap from the margin, so this excludes them cleanly.
    x = gutter
    while x < W-1 and col[x] < 0.35*cmax:
        x += 1
    x_dense = x
    while x > gutter and col[x] > 0.20*cmax:
        x -= 1
    latin_left = x + 1
    x = W-1
    while x > latin_left and col[x] < 0.18*cmax:
        x -= 1
    latin_right = x
    return latin_left, min(W, latin_right+8), gutter

def line_bands(colimg_ink, min_ink=3):
    """Rows of the cropped column -> list of (top,bottom) text bands."""
    row = colimg_ink.sum(axis=1)
    texty = row > min_ink
    bands = []
    y = 0; H = len(texty)
    while y < H:
        if texty[y]:
            s = y
            while y < H and texty[y]:
                y += 1
            bands.append((s, y))
        else:
            y += 1
    return bands

def drop_furniture(bands):
    """Drop a running-head band at top and a watermark band at bottom when they are
    separated from the body by an abnormally large gap."""
    if len(bands) < 3:
        return bands
    gaps = [bands[i+1][0]-bands[i][1] for i in range(len(bands)-1)]
    med = np.median(gaps)
    b = list(bands)
    # leading furniture: first gap much larger than median
    if gaps[0] > 2.2*med:
        b = b[1:]; gaps = gaps[1:]
    if gaps and gaps[-1] > 2.2*med:
        b = b[:-1]
    return b

def main():
    img, a = load("v1_p210_300.png")
    H, W = a.shape
    L, R, gutter = detect_latin_column(a)
    colw = R - L
    print(f"page {W}x{H}px  gutter={gutter}  Latin col x=[{L},{R}] width={colw}px "
          f"= {colw/DPI:.2f}in")

    col_gray = a[:, L:R]
    col_ink = ink_of(col_gray)
    # Furniture zones: running head/page number live in the top band, the Google
    # watermark in the bottom band. Keep only line-bands whose center is in the body.
    top_zone, bot_zone = int(0.065*H), int(0.95*H)
    bands = [b for b in line_bands(col_ink) if top_zone <= (b[0]+b[1])//2 <= bot_zone]
    bands = drop_furniture(bands)
    print(f"detected {len(bands)} body line-bands after dropping furniture")

    # Diagnostic: column crop with cut lines (midpoints of inter-line gaps).
    diag = Image.fromarray(col_gray).convert("RGB")
    dd = ImageDraw.Draw(diag)
    cuts = [bands[0][0]]
    for i in range(len(bands)-1):
        cuts.append((bands[i][1]+bands[i+1][0])//2)
    cuts.append(bands[-1][1])
    for c in cuts:
        dd.line([(0, c), (col_gray.shape[1], c)], fill=(220, 30, 30), width=2)
    diag.save(f"{D}/out_col_cuts.png")

    # Trim page geometry (10.5 x 16 cm) with margins.
    cm = DPI/2.54
    page_w, page_h = round(10.5*cm), round(16.0*cm)
    mx, mtop, mbot = round(1.0*cm), round(1.2*cm), round(1.2*cm)
    tb_w = page_w - 2*mx
    tb_h = page_h - mtop - mbot
    scale = tb_w / colw
    print(f"trim page {page_w}x{page_h}px  text block {tb_w}x{tb_h}px  scale x{scale:.3f} "
          f"(>1 = enlarged)")

    # Fill one trim page with whole line-strips, cutting only at gaps.
    page = Image.new("L", (page_w, page_h), 255)
    y = mtop
    placed = 0
    # Build strips between successive cut points (each strip = one line + half-gaps).
    for i in range(len(cuts)-1):
        top, bot = cuts[i], cuts[i+1]
        strip = col_gray[top:bot, :]
        sh = round(strip.shape[0]*scale)
        sw = round(strip.shape[1]*scale)
        if y + sh > mtop + tb_h:
            break
        s = Image.fromarray(strip).resize((sw, sh), Image.LANCZOS)
        page.paste(s, (mx, y))
        y += sh
        placed += 1
    page.save(f"{D}/out_trim_page.png")
    print(f"placed {placed} lines on one trim page (of {len(cuts)-1} available)")

if __name__ == "__main__":
    main()
