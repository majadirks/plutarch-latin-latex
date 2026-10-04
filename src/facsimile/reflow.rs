//! Global-scale selection, strip extraction, and composition of per-trim-page
//! body images from the detected line-bands.
//!
//! One global scale is chosen for the whole work (so type size does not breathe
//! page to page), mapping a robust column width to the trim text-block width. Every
//! composed body canvas is the same fixed pixel width — the global measure — so
//! that `\includegraphics[width=\textwidth]` keeps the type size uniform across
//! pages; canvas height is ragged on a unit's last page.

use anyhow::Result;
use image::imageops::FilterType;
use image::{GrayImage, Luma};

use crate::facsimile::columns::{crop_column, detect_latin_column, INK_THRESHOLD};
use crate::facsimile::lines::{cut_points, drop_furniture, ink_profile_rows, line_bands, Band};
use crate::facsimile::metrics::{
    band_ink_span, classify_band, is_narrow, line_metrics, BandClass, LineMetrics,
};
use crate::facsimile::units::UnitRange;

/// Default DPI the body column is rendered and composed at.
pub const BODY_DPI: u32 = 300;

/// Minimum per-row ink for a row to count as texty when banding a column.
const LINE_MIN_INK: u32 = 3;

/// A composed page's text block must be as wide as the widest line we size for;
/// lines are sized to this percentile of detected column widths so a few spuriously
/// over-wide detections do not shrink every page.
const SCALE_PERCENTILE: f32 = 95.0;

/// A column crop whose ink spans at least this fraction of its width is a
/// full-measure body line (not a centered title / short cross-reference).
const BODY_LINE_SPAN_FRAC: f32 = 0.8;

/// Minimum dark-pixel area for a strip, measured in source-column pixels.
const SPECK_MIN_INK: u64 = 8;
/// Ink-area floor relative to the typical line's bounding rectangle.
const SPECK_INK_FRAC: f64 = 0.02;

/// The negligible-ink threshold; absent metrics disable the filter.
fn minimum_ink(m: &LineMetrics) -> Option<u64> {
    (!m.is_sentinel())
        .then(|| (SPECK_INK_FRAC * m.height_med as f64 * m.width_med as f64).ceil() as u64)
        .map(|ink| ink.max(SPECK_MIN_INK))
}

// Trim geometry, in centimetres, matching the `geometry` options the facsimile
// inner `.tex` uses (`inner_preamble` with the running head on): paper 10.5×16 cm,
// `inner=1`, `outer=1` → `\textwidth = 8.5 cm`; `top=1.5` (above the running head),
// `bottom=1.5`, and the head band (`headheight`+`headsep`) reserved for it. These
// are the real page margins (binding-safe); a golden test keeps them in sync with
// the preamble strings.
const PAPER_W_CM: f32 = 10.5;
const PAPER_H_CM: f32 = 16.0;
/// Inner (spine-side) page margin.
pub const INNER_CM: f32 = 1.0;
/// Outer (fore-edge) page margin.
pub const OUTER_CM: f32 = 1.0;
/// Margin above the running head.
pub const TOP_CM: f32 = 1.5;
/// Margin below the text.
pub const BOTTOM_CM: f32 = 1.5;
/// Head-band height reserved for the running head (see `inner_preamble`).
pub const HEADHEIGHT_CM: f32 = 0.5;
/// Gap between the running head and the body (see `inner_preamble`).
pub const HEADSEP_CM: f32 = 0.4;

// Optional safety insets baked *inside* the text block, on top of the page
// geometry margins. Zero by default: the page geometry already provides generous
// binding-safe margins, so the composed image fills the full measure. They remain
// tunable if a tighter crop ever needs extra breathing room.
/// Left/right safety inset.
const SAFETY_X_CM: f32 = 0.0;
/// Top/bottom safety inset.
const SAFETY_Y_CM: f32 = 0.0;

/// Bottom-only slack subtracted from the per-canvas height budget so a full page
/// keeps a visible lower margin. Unlike `SAFETY_Y_CM` (applied symmetrically, which
/// would also push type down from the top), this trims only the height budget, so
/// the type size is unchanged and the measure still maps to the full width.
pub const BODY_BOTTOM_SLACK_CM: f32 = 0.3;

/// Vertical space reserved on a unit's first page for the inline chapter header
/// (about one `\large` line plus a `\bigskip`). Subtracted from the first canvas's
/// height budget so the header and the first composed image fit the page together.
/// Tuned after a real-eyes check.
pub const HEADER_ALLOWANCE_CM: f32 = 1.2;

/// The trim text block in pixels at a given body DPI. `tb_w`/`tb_h` are the full
/// text block (width maps to `\textwidth`); `pad_x`/`pad_y` are the safety insets
/// the pasted type is kept inside, so the usable measure is `content_width`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrimGeom {
    pub dpi: u32,
    pub tb_w: u32,
    pub tb_h: u32,
    pub pad_x: u32,
    pub pad_y: u32,
}

impl TrimGeom {
    /// The geometry at `dpi`, derived from the shared centimetre constants.
    pub fn new(dpi: u32) -> Self {
        let px = |cm: f32| (cm / 2.54 * dpi as f32).round() as u32;
        let text_w = PAPER_W_CM - INNER_CM - OUTER_CM;
        let text_h = PAPER_H_CM - TOP_CM - HEADHEIGHT_CM - HEADSEP_CM - BOTTOM_CM;
        TrimGeom {
            dpi,
            tb_w: px(text_w),
            tb_h: px(text_h),
            pad_x: px(SAFETY_X_CM),
            pad_y: px(SAFETY_Y_CM),
        }
    }

    /// The usable text measure in px after the left/right safety insets. Strips are
    /// scaled to this width, so [`global_scale`] is called with it (not `tb_w`).
    pub fn content_width(&self) -> u32 {
        self.tb_w.saturating_sub(2 * self.pad_x).max(1)
    }

    /// The usable vertical budget in px for strips after the top/bottom insets.
    pub fn content_height(&self) -> u32 {
        self.tb_h.saturating_sub(2 * self.pad_y).max(1)
    }

    /// The per-canvas height budget: the usable content height less the bottom
    /// slack, so a fully packed page keeps a visible lower margin. Every composed
    /// canvas is kept at or below this, so no page overflows its bottom margin.
    pub fn body_budget(&self) -> u32 {
        let slack = (BODY_BOTTOM_SLACK_CM / 2.54 * self.dpi as f32).round() as u32;
        self.content_height().saturating_sub(slack).max(1)
    }

    /// The first canvas's height budget: the body budget less the inline chapter
    /// header's allowance, so a unit's first page fits the header above its image.
    pub fn first_body_budget(&self) -> u32 {
        let header = (HEADER_ALLOWANCE_CM / 2.54 * self.dpi as f32).round() as u32;
        self.body_budget().saturating_sub(header).max(1)
    }
}

impl Default for TrimGeom {
    fn default() -> Self {
        TrimGeom::new(BODY_DPI)
    }
}

/// One rendered unit page: its cropped Latin-column image and the body line-bands
/// kept for it.
pub type PageBands = (GrayImage, Vec<Band>);

/// One line-strip of a page's column: which composed-page-source image it comes
/// from, the vertical `[top, bottom)` extent (a cut-point interval, so it carries
/// the line plus its surrounding half-gaps), and the class of the underlying line
/// band (so the compose step knows which strips are tall/merged and may be split to
/// fill a partial canvas).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Strip {
    pub page_img_index: usize,
    pub band: Band,
    pub class: BandClass,
}

/// Choose one global scale mapping a robust (95th-percentile) column width to the
/// trim text-block width, so that width maps exactly to the measure.
pub fn global_scale(col_widths: &[u32], tb_w_px: u32) -> f32 {
    let p95 = percentile(col_widths, SCALE_PERCENTILE);
    if p95 == 0 {
        return 1.0;
    }
    tb_w_px as f32 / p95 as f32
}

/// Nearest-rank percentile of a width set.
fn percentile(widths: &[u32], p: f32) -> u32 {
    if widths.is_empty() {
        return 0;
    }
    let mut sorted = widths.to_vec();
    sorted.sort_unstable();
    let rank = (p / 100.0 * sorted.len() as f32).ceil() as usize;
    let idx = rank.saturating_sub(1).min(sorted.len() - 1);
    sorted[idx]
}

/// Render, column-detect, crop, band, and de-furniture every source page in a
/// unit's range, returning one `(column image, body bands)` per page plus the
/// unit's robust [`LineMetrics`].
///
/// Two passes, because the metrics must summarize body lines from **every** page
/// (furniture already removed) before the boundary trims — which depend on them —
/// can run. Pass A bands every page (`render_and_band`, which drops furniture and,
/// on interior pages, a bled running head) without any boundary trim. The metrics
/// are taken over those bands. Pass B then applies the first page's top-trim (the
/// previous unit's tail + the unit's own display title) and, when the unit shares
/// its last page with the next unit, that page's bottom-trim. A page that fails to
/// render or detect is counted and warned, never silently dropped; a unit with no
/// usable band yields the sentinel metrics.
pub fn unit_body_strips(
    range: &UnitRange,
    render_fn: impl Fn(usize, u32) -> Result<GrayImage>,
) -> Result<(Vec<PageBands>, LineMetrics)> {
    let source = range.unit.source;

    // Pass A — band every page; no boundary trim yet.
    let mut pages: Vec<(GrayImage, Vec<Band>)> = Vec::new();
    let mut source_pages = Vec::new();
    let mut skipped = 0u32;
    for page in range.unit.start_page..=range.end_page {
        match render_and_band(source, page, &render_fn) {
            Ok((col, bands)) => {
                pages.push((col, bands));
                source_pages.push(page);
            }
            Err(e) => {
                skipped += 1;
                eprintln!(
                    "warning: unit {:?} source {source} page {page}: {e:#} (skipped)",
                    range.unit.id
                );
            }
        }
    }
    if skipped > 0 {
        eprintln!(
            "warning: unit {:?}: {skipped} source page(s) skipped",
            range.unit.id
        );
    }

    let metrics = line_metrics(&pages);

    // Pass B — boundary and furniture trims, now that the metrics exist. The first
    // page drops its leading opening bands; every interior page drops a bled running
    // head / signature; a shared last page drops from the next unit's title down.
    // When the unit's one page is both first and shared-last, both boundary trims
    // apply to it.
    for (page, (col, bands)) in source_pages.into_iter().zip(pages.iter_mut()) {
        if page == range.unit.start_page {
            *bands = drop_opening_bands(
                col,
                bands,
                range.unit.skip_top_bands,
                range.open_body_frac,
                &metrics,
            );
        } else {
            *bands = drop_interior_running_head(col, bands, &metrics);
        }
        if range.shared_tail && page == range.end_page {
            *bands = drop_shared_tail(col, bands, range.tail_cut_frac, &metrics);
        }
    }

    Ok((pages, metrics))
}

/// Render one page, crop its Latin column, band it, and drop zone furniture. The
/// leading / trailing bled-head and signature drops run later in Pass B (they need
/// the unit's line metrics), so this leaves them in place.
fn render_and_band(
    source: usize,
    page: u32,
    render_fn: &impl Fn(usize, u32) -> Result<GrayImage>,
) -> Result<(GrayImage, Vec<Band>)> {
    let page_img = render_fn(source, page)?;
    let column = detect_latin_column(&page_img)?;
    let cropped = crop_column(&page_img, &column);
    let bands = line_bands(&cropped, LINE_MIN_INK);
    let bands = drop_furniture(&bands, cropped.height());
    Ok((cropped, bands))
}

/// Drop a bled-through running head or bare folio/signature that survived zone
/// filtering on an interior page: a **leading** band that is both short (ink span
/// under a full body line) and detached (separated from the next band by an
/// above-median gap), and a **trailing** band at the foot that is short *or* a
/// metric-class outlier (`Short`/`Narrow`) *and* detached (a bare page number or
/// signature numeral like a stray "2"). A band must be BOTH outlier AND detached to
/// be dropped, so a legitimately short first line or a full-measure last line is
/// kept.
fn drop_interior_running_head(col: &GrayImage, bands: &[Band], m: &LineMetrics) -> Vec<Band> {
    if bands.len() < 3 {
        return bands.to_vec();
    }
    let short = BODY_LINE_SPAN_FRAC * col.width() as f32;
    let gaps: Vec<i64> = bands
        .windows(2)
        .map(|w| w[1].top as i64 - w[0].bottom as i64)
        .collect();
    let med = median_gap(&gaps);
    let mut lo = 0usize;
    if (band_ink_span(col, &bands[0]) as f32) < short && gaps[0] as f32 > med {
        lo = 1;
    }
    drop_trailing_furniture(col, &bands[lo..], m)
}

/// Drop a short or narrow final band detached by an above-median gap. The
/// full-measure threshold also identifies furniture when metrics are unavailable.
fn drop_trailing_furniture(col: &GrayImage, bands: &[Band], m: &LineMetrics) -> Vec<Band> {
    if bands.len() < 3 {
        return bands.to_vec();
    }
    let gaps: Vec<i64> = bands
        .windows(2)
        .map(|w| w[1].top as i64 - w[0].bottom as i64)
        .collect();
    let last = bands.len() - 1;
    let outlier = (band_ink_span(col, &bands[last]) as f32)
        < BODY_LINE_SPAN_FRAC * col.width() as f32
        || matches!(
            classify_band(col, &bands[last], m),
            BandClass::Short | BandClass::Narrow
        );
    let hi = if outlier && gaps[last - 1] as f32 > median_gap(&gaps) {
        last
    } else {
        bands.len()
    };
    bands[..hi].to_vec()
}

/// Median of inter-band gaps, as f32.
fn median_gap(gaps: &[i64]) -> f32 {
    if gaps.is_empty() {
        return 0.0;
    }
    let mut sorted = gaps.to_vec();
    sorted.sort_unstable();
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[mid - 1] + sorted[mid]) as f32 / 2.0
    } else {
        sorted[mid] as f32
    }
}

/// Map a page-height fraction to a column row, less half a median line of
/// tolerance. Missing metrics or invalid fractions leave the heuristic in charge.
fn frac_row(frac: f32, col_h: u32, m: &LineMetrics) -> Option<u32> {
    if m.is_sentinel() || !frac.is_finite() || !(0.0..=1.0).contains(&frac) {
        return None;
    }
    Some(((frac * col_h as f32).round() as u32).saturating_sub(m.height_med / 2))
}

/// Drop the leading opening bands of a unit's first page. The `skip_top_bands`
/// override, when set, drops exactly that many bands. A recorded body fraction
/// keeps bands from the first body line downward. Without a usable fraction, if the source's
/// printed display title is detected among the leading bands, every band up to and
/// including it is dropped (the previous unit's tail on a shared page plus the
/// printed title); failing that, every band above the first full-measure body line
/// (the work/Life titles and edition cross-reference are narrower than the measure).
fn drop_opening_bands(
    col: &GrayImage,
    bands: &[Band],
    skip_top_bands: Option<u32>,
    open_body_frac: Option<f32>,
    m: &LineMetrics,
) -> Vec<Band> {
    match skip_top_bands {
        Some(n) => bands.iter().skip(n as usize).copied().collect(),
        None => {
            if let Some(row) = open_body_frac.and_then(|frac| frac_row(frac, col.height(), m)) {
                return bands
                    .iter()
                    .copied()
                    .filter(|b| b.top + (b.bottom - b.top) / 2 >= row)
                    .collect();
            }
            if let Some(i) = find_title_band(col, bands, m) {
                return bands[i + 1..].to_vec();
            }
            let threshold = BODY_LINE_SPAN_FRAC * col.width() as f32;
            match bands
                .iter()
                .position(|b| band_ink_span(col, b) as f32 >= threshold)
            {
                Some(i) => bands[i..].to_vec(),
                None => bands.to_vec(),
            }
        }
    }
}

/// On a unit's shared last page, keep only the bands **above** the next unit's
/// printed display title (dropping the title and the next unit's body below it).
/// A recorded title fraction drives the cut and a detached trailing separator is
/// removed. Without a usable fraction, the title heuristic supplies the cut; if
/// that finds no title, the page contributes no bands to this unit.
fn drop_shared_tail(
    col: &GrayImage,
    bands: &[Band],
    tail_cut_frac: Option<f32>,
    m: &LineMetrics,
) -> Vec<Band> {
    if let Some(row) = tail_cut_frac.and_then(|frac| frac_row(frac, col.height(), m)) {
        let kept: Vec<Band> = bands
            .iter()
            .copied()
            .filter(|b| b.top + (b.bottom - b.top) / 2 < row)
            .collect();
        return drop_trailing_furniture(col, &kept, m);
    }
    match find_title_band_from(col, bands, m) {
        Some(i) => bands[..i].to_vec(),
        None => Vec::new(),
    }
}

/// Leading/trailing bands scanned for a display title.
const TITLE_SCAN_BANDS: usize = 6;

/// The leading band indices the optional line review should examine: the boundary
/// bands the heuristics are least sure about — a `Tall` (merged) band or a `Narrow`
/// band (a possible title / furniture) within the leading window. Ordinary full-
/// measure body lines are not candidates.
pub(crate) fn review_candidates(col: &GrayImage, bands: &[Band], m: &LineMetrics) -> Vec<usize> {
    let hi = bands.len().min(TITLE_SCAN_BANDS);
    (0..hi)
        .filter(|&i| {
            matches!(classify_band(col, &bands[i], m), BandClass::Tall)
                || is_narrow(col, &bands[i], m)
        })
        .collect()
}

/// The index of a unit's own printed display title among the **leading** bands, or
/// `None` on a clean top-of-page start. A display title is **narrow** (centered,
/// ink-span well under the measure) and **isolated** (an above-median gap on at
/// least one side); body lines are full-measure. The earliest such band in the
/// leading window is returned, so the previous unit's tail (on a shared page) and
/// this unit's own title are dropped above it. Heuristic; tunable.
fn find_title_band(col: &GrayImage, bands: &[Band], m: &LineMetrics) -> Option<usize> {
    let hi = bands.len().min(TITLE_SCAN_BANDS);
    scan_title_band(col, bands, m, 0..hi)
}

/// The index of the next unit's printed display title among the **trailing** bands
/// of a shared page, or `None` when the page has no following-unit title. Mirrors
/// [`find_title_band`] but scans the trailing window, so the previous unit keeps the
/// bands above it.
fn find_title_band_from(col: &GrayImage, bands: &[Band], m: &LineMetrics) -> Option<usize> {
    let lo = bands.len().saturating_sub(TITLE_SCAN_BANDS);
    scan_title_band(col, bands, m, lo..bands.len())
}

/// The earliest narrow, isolated band whose index lies in `range` — the display-title
/// heuristic shared by [`find_title_band`] and [`find_title_band_from`]. Returns
/// `None` when no band in the window is both narrow and set off by an above-median
/// gap, or when the metrics are the sentinel (every band reads `Normal`).
fn scan_title_band(
    col: &GrayImage,
    bands: &[Band],
    m: &LineMetrics,
    range: std::ops::Range<usize>,
) -> Option<usize> {
    if bands.len() < 2 || m.is_sentinel() {
        return None;
    }
    let gaps: Vec<i64> = bands
        .windows(2)
        .map(|w| w[1].top as i64 - w[0].bottom as i64)
        .collect();
    let med = median_gap(&gaps);
    for i in range {
        if i >= bands.len() {
            break;
        }
        if !is_narrow(col, &bands[i], m) {
            continue;
        }
        let gap_before = if i == 0 {
            bands[0].top as f32
        } else {
            gaps[i - 1] as f32
        };
        let gap_after = if i + 1 < bands.len() {
            gaps[i] as f32
        } else {
            0.0
        };
        if gap_before > med || gap_after > med {
            return Some(i);
        }
    }
    None
}

/// Flatten a unit's pages into line-strips in reading order, each a cut-point
/// interval so inter-line spacing is preserved and no line is bisected. Each strip
/// also carries the class of its underlying line band (classified against `m`), so
/// the compose step can tell a merged "tall" strip (splittable to fill a page) from
/// an ordinary line (which wraps whole). The `j`-th cut-point interval on a page
/// corresponds to that page's `j`-th band, so the two zip directly.
fn strips_in_reading_order(pages: &[(GrayImage, Vec<Band>)], m: &LineMetrics) -> Vec<Strip> {
    let mut strips = Vec::new();
    let min_ink = minimum_ink(m);
    for (i, (col, bands)) in pages.iter().enumerate() {
        let rows = min_ink.map(|_| ink_profile_rows(col, INK_THRESHOLD));
        let cuts = cut_points(bands);
        for (j, w) in cuts.windows(2).enumerate() {
            if let (Some(min_ink), Some(rows)) = (min_ink, &rows) {
                let band = &bands[j];
                let ink: u64 = rows[band.top as usize..band.bottom as usize]
                    .iter()
                    .map(|&ink| ink as u64)
                    .sum();
                if ink < min_ink {
                    continue;
                }
            }
            strips.push(Strip {
                page_img_index: i,
                band: Band {
                    top: w[0],
                    bottom: w[1],
                },
                class: classify_band(col, &bands[j], m),
            });
        }
    }
    strips
}

/// Fraction of a typical scaled line used as the minimum fill-split piece, so a cut
/// that fills a partial canvas is never a sliver. Tunable.
const MIN_PIECE_FACTOR: f32 = 0.5;
/// Absolute floor (scaled px) for the minimum fill-split piece, used when the metrics
/// carry no usable line height.
const MIN_PIECE_FLOOR: u32 = 8;

/// Compose a unit's pages into trim-page body images: scale every strip by the one
/// global `scale`, pack them top-to-bottom onto fixed-width (`tb_w`) canvases, inset
/// by the safety margins, wrapping to a new canvas at a strip boundary. The unit's
/// **first** emitted canvas uses `first_budget` (reduced to leave room for the
/// inline chapter header); every later canvas uses `budget`. The last canvas is
/// ragged in height.
///
/// Greedy fill: a strip that does not fit the remaining space is, when it is a
/// **tall** (merged) strip and the remaining space is at least a half-line, cut at a
/// low-ink row to **fill the current canvas** before wrapping; its remainder flows
/// on. An ordinary line that merely does not fit wraps whole (never bisected). A
/// strip taller than an entire budget (even on a fresh canvas) falls back to
/// [`split_tall_strip`]. The classification is taken from the original band against
/// the original `metrics`, so it is scale-invariant; the cutting is all in scaled
/// (trim) px. **Every returned canvas has height ≤ its budget** — no page overflows,
/// and no page closes early leaving a large mid-unit blank.
pub fn compose_pages(
    pages: &[(GrayImage, Vec<Band>)],
    scale: f32,
    geom: &TrimGeom,
    first_budget: u32,
    budget: u32,
    metrics: &LineMetrics,
) -> Vec<GrayImage> {
    let strips = strips_in_reading_order(pages, metrics);
    let min_piece = min_piece_px(metrics, scale);

    let mut canvases: Vec<GrayImage> = Vec::new();
    let mut current: Vec<GrayImage> = Vec::new();
    let mut used_h = 0u32;
    // Scale each strip lazily and feed it to the packer; a fill-split pushes the
    // strip's remainder back as `pending` so it is placed before the next strip.
    let mut pending: Option<(GrayImage, bool)> = None;
    let mut iter = strips.iter();
    loop {
        let (atom, tall) = match pending.take() {
            Some(x) => x,
            None => match iter.next() {
                Some(strip) => {
                    match scale_strip(&pages[strip.page_img_index].0, &strip.band, scale) {
                        Some(img) => (img, matches!(strip.class, BandClass::Tall)),
                        None => continue,
                    }
                }
                None => break,
            },
        };

        let b = if canvases.is_empty() {
            first_budget
        } else {
            budget
        };
        let remaining = b.saturating_sub(used_h);
        let h = atom.height();

        if h <= remaining {
            used_h += h;
            current.push(atom);
            continue;
        }

        // The strip does not fit the remaining space. A tall (merged) strip with
        // room for at least a half-line is cut at a low-ink row to fill the page.
        if tall && remaining >= min_piece {
            if let Some(cut) = lowest_ink_row(&atom, min_piece, remaining) {
                let w = atom.width();
                let piece = image::imageops::crop_imm(&atom, 0, 0, w, cut).to_image();
                let rem =
                    image::imageops::crop_imm(&atom, 0, cut, w, atom.height() - cut).to_image();
                current.push(piece);
                canvases.push(pack_canvas(&current, geom));
                current.clear();
                used_h = 0;
                pending = Some((rem, tall));
                continue;
            }
        }

        // Not splittable here. If the canvas has content, close it and retry the
        // whole strip on a fresh canvas.
        if !current.is_empty() {
            canvases.push(pack_canvas(&current, geom));
            current.clear();
            used_h = 0;
            pending = Some((atom, tall));
            continue;
        }

        // A fresh canvas still cannot hold the strip (taller than a whole budget):
        // split it at low-ink rows into budget-sized pieces so the invariant holds.
        for piece in split_tall_strip(&atom, b) {
            let ph = piece.height();
            if used_h + ph > b && !current.is_empty() {
                canvases.push(pack_canvas(&current, geom));
                current.clear();
                used_h = 0;
            }
            used_h += ph;
            current.push(piece);
        }
    }
    if !current.is_empty() {
        canvases.push(pack_canvas(&current, geom));
    }
    canvases
}

/// Scale a strip (a cut-point interval of a page column) by the global `scale`,
/// returning `None` for an empty interval.
fn scale_strip(img: &GrayImage, band: &Band, scale: f32) -> Option<GrayImage> {
    let h = band.bottom.saturating_sub(band.top);
    if h == 0 {
        return None;
    }
    let raw = image::imageops::crop_imm(img, 0, band.top, img.width(), h).to_image();
    let sw = ((raw.width() as f32 * scale).round() as u32).max(1);
    let sh = ((raw.height() as f32 * scale).round() as u32).max(1);
    Some(image::imageops::resize(&raw, sw, sh, FilterType::Lanczos3))
}

/// The minimum fill-split piece height in scaled (trim) px: half a scaled typical
/// line, or an absolute floor when the metrics carry no usable line height.
fn min_piece_px(metrics: &LineMetrics, scale: f32) -> u32 {
    if metrics.is_sentinel() {
        return MIN_PIECE_FLOOR;
    }
    ((MIN_PIECE_FACTOR * metrics.height_med as f32 * scale).round() as u32).max(MIN_PIECE_FLOOR)
}

/// Split a scaled strip taller than `max_h` into sub-strips each ≤ `max_h`, cutting
/// preferentially at an internal low-ink row (so a cut never bisects a glyph row)
/// and falling back to a hard cut at `max_h` when the window has no clear gap. The
/// cut is sought in the lower half of each `max_h` window so pieces are not
/// needlessly short.
fn split_tall_strip(strip: &GrayImage, max_h: u32) -> Vec<GrayImage> {
    let h = strip.height();
    if h <= max_h || max_h == 0 {
        return vec![strip.clone()];
    }
    let rows = ink_profile_rows(strip, INK_THRESHOLD);
    let mut pieces = Vec::new();
    let mut top = 0u32;
    while h - top > max_h {
        let hi = top + max_h; // exclusive upper bound for the cut
        let lo = (top + (max_h / 2)).max(top + 1);
        // Lowest-ink row in [lo, hi); ties resolve to the latest row so pieces fill
        // as much of the budget as possible.
        let cut = lowest_ink_row_in(&rows, lo, hi)
            .unwrap_or_else(|| hi.saturating_sub(1))
            .max(top + 1);
        pieces.push(image::imageops::crop_imm(strip, 0, top, strip.width(), cut - top).to_image());
        top = cut;
    }
    if top < h {
        pieces.push(image::imageops::crop_imm(strip, 0, top, strip.width(), h - top).to_image());
    }
    pieces
}

/// The lowest-ink row in `[lo, hi)` of a strip (ties resolve to the latest row, so a
/// fill cut uses as much of the budget as possible), or `None` for an empty window.
fn lowest_ink_row(strip: &GrayImage, lo: u32, hi: u32) -> Option<u32> {
    let rows = ink_profile_rows(strip, INK_THRESHOLD);
    lowest_ink_row_in(&rows, lo, hi.min(strip.height()))
}

/// The lowest-ink row in `[lo, hi)` of a precomputed row profile, ties to the latest.
fn lowest_ink_row_in(rows: &[u32], lo: u32, hi: u32) -> Option<u32> {
    let hi = hi.min(rows.len() as u32);
    if lo >= hi {
        return None;
    }
    let mut cut = None;
    let mut best = u32::MAX;
    for y in lo..hi {
        let ink = rows[y as usize];
        if ink <= best {
            best = ink;
            cut = Some(y);
        }
    }
    cut
}

/// Paste scaled strips top-to-bottom, left-aligned, onto one fixed-width (`tb_w`)
/// white canvas, inset from the top by `pad_y` and from the left by `pad_x` so the
/// type keeps a safety margin inside the text block. Canvas height is the top inset
/// plus the strip heights; the bottom margin falls out of the frame below.
fn pack_canvas(strips: &[GrayImage], geom: &TrimGeom) -> GrayImage {
    let content_h: u32 = strips.iter().map(|s| s.height()).sum();
    let height = geom.pad_y + content_h;
    let mut canvas = GrayImage::from_pixel(geom.tb_w, height.max(1), Luma([255]));
    let mut y = geom.pad_y as i64;
    for strip in strips {
        image::imageops::overlay(&mut canvas, strip, geom.pad_x as i64, y);
        y += strip.height() as i64;
    }
    canvas
}

/// The file name of a unit's `n`-th (1-based) composed body image.
pub fn body_image_name(unit_id: &str, n: usize) -> String {
    format!("facsimile_{unit_id}_{n}.png")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frac_row_maps_full_height_with_line_tolerance() {
        let m = LineMetrics {
            height_med: 8,
            height_mad: 0,
            width_med: 90,
        };
        assert_eq!(frac_row(0.39, 1000, &m), Some(386));
        assert_eq!(frac_row(0.0, 1000, &m), Some(0));
        assert_eq!(frac_row(0.39, 1000, &LineMetrics::sentinel()), None);
        for frac in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
            assert_eq!(frac_row(frac, 1000, &m), None);
        }
    }

    /// A synthetic column image with the given row-bands filled dark across
    /// `[left, right)`; the rest is white.
    fn column(h: u32, w: u32, bands: &[(u32, u32, u32, u32)]) -> GrayImage {
        let mut img = GrayImage::from_pixel(w, h, Luma([255]));
        for &(top, bottom, left, right) in bands {
            for y in top..bottom {
                for x in left..right {
                    img.put_pixel(x, y, Luma([0]));
                }
            }
        }
        img
    }

    #[test]
    fn trim_geom_width_is_textwidth_in_px() {
        let geom = TrimGeom::new(300);
        // 8.5 cm / 2.54 * 300 ≈ 1004 px.
        assert!((1002..=1006).contains(&geom.tb_w), "tb_w {}", geom.tb_w);
        assert!(geom.tb_h > geom.tb_w, "a page is taller than it is wide");
        // No internal inset by default: the page geometry provides the margins, so
        // the composed image fills the full text measure.
        assert_eq!(geom.pad_x, 0);
        assert_eq!(geom.pad_y, 0);
        assert_eq!(geom.content_width(), geom.tb_w);
        assert_eq!(geom.content_height(), geom.tb_h);
    }

    #[test]
    fn global_scale_maps_p95_to_measure() {
        // Nineteen measures at 100 and one spurious over-wide 200: p95 = 100, so the
        // typographic measure (100) maps exactly to tb_w and the outlier is ignored.
        let mut widths = vec![100u32; 19];
        widths.push(200);
        let scale = global_scale(&widths, 950);
        assert_eq!((scale * 100.0).round() as u32, 950);
    }

    #[test]
    fn global_scale_never_overflows_widest() {
        // With no spurious outlier (max == p95), the widest column fits the measure
        // exactly after scaling, so no line overflows.
        let widths = vec![80, 90, 95, 100, 100, 100];
        let tb_w = 1000;
        let scale = global_scale(&widths, tb_w);
        let widest = *widths.iter().max().unwrap();
        assert!((scale * widest as f32) <= tb_w as f32 + 0.5);
    }

    #[test]
    fn first_page_excludes_title_and_crossref_bands() {
        // Two narrow centered bands (work/Life title, cross-ref) over three
        // full-measure body lines; the heuristic drops the two narrow ones.
        let w = 100;
        let img = column(
            100,
            w,
            &[
                (5, 10, 40, 60),  // narrow centered title
                (15, 20, 42, 58), // narrow centered cross-ref
                (30, 38, 5, 95),  // body line (full measure)
                (45, 53, 5, 95),  // body line
                (60, 68, 5, 95),  // body line
            ],
        );
        let bands = line_bands(&img, 3);
        assert_eq!(bands.len(), 5);
        // With sentinel metrics the title heuristic is skipped, so this exercises the
        // full-measure-line fallback: both narrow leading bands are dropped.
        let kept = drop_opening_bands(&img, &bands, None, None, &LineMetrics::sentinel());
        assert_eq!(kept.len(), 3);
        assert_eq!(kept[0].top, 30);
    }

    #[test]
    fn skip_top_bands_override_drops_exact_count() {
        let w = 100;
        let img = column(
            100,
            w,
            &[
                (5, 10, 5, 95),
                (20, 28, 5, 95),
                (35, 43, 5, 95),
                (50, 58, 5, 95),
            ],
        );
        let bands = line_bands(&img, 3);
        assert_eq!(bands.len(), 4);
        let kept = drop_opening_bands(&img, &bands, Some(3), Some(0.0), &LineMetrics::sentinel());
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].top, 50);
    }

    /// One page: a column with `n` evenly spaced full-width bands of equal height.
    fn page_with_lines(n: u32) -> (GrayImage, Vec<Band>) {
        let w = 100;
        let line_h = 8;
        let gap = 4;
        let mut specs = Vec::new();
        let mut y = gap;
        for _ in 0..n {
            specs.push((y, y + line_h, 5, 95));
            y += line_h + gap;
        }
        let img = column(y, w, &specs);
        let bands = line_bands(&img, 3);
        (img, bands)
    }

    #[test]
    fn compose_drops_speck_trailing_canvas() {
        let mut page = page_with_lines(3);
        let top = page.0.height() - 1;
        for x in 5..8 {
            page.0.put_pixel(x, top, Luma([0]));
        }
        page.1.push(Band {
            top,
            bottom: top + 1,
        });
        let m = line_metrics(std::slice::from_ref(&page));
        let strips = strips_in_reading_order(std::slice::from_ref(&page), &m);
        assert_eq!(strips.len(), 3);
        assert_eq!(
            strips_in_reading_order(std::slice::from_ref(&page), &LineMetrics::sentinel()).len(),
            4
        );
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 100,
            tb_h: 100,
            pad_x: 0,
            pad_y: 0,
        };
        let canvases = compose_pages(&[page], 1.0, &geom, 12, 12, &m);
        assert_eq!(canvases.len(), 3);
        for canvas in canvases {
            let ink: u64 = ink_profile_rows(&canvas, INK_THRESHOLD)
                .iter()
                .map(|&n| n as u64)
                .sum();
            assert_eq!(ink, 8 * 90);
        }
    }

    #[test]
    fn compose_keeps_short_real_last_line() {
        let mut page = page_with_lines(3);
        let last = *page.1.last().unwrap();
        for y in last.top..last.bottom {
            for x in 25..95 {
                page.0.put_pixel(x, y, Luma([255]));
            }
        }
        let m = line_metrics(std::slice::from_ref(&page));
        assert_eq!(
            strips_in_reading_order(std::slice::from_ref(&page), &m).len(),
            3
        );
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 100,
            tb_h: 100,
            pad_x: 0,
            pad_y: 0,
        };
        let canvases = compose_pages(&[page], 1.0, &geom, 12, 12, &m);
        assert_eq!(canvases.len(), 3);
        let ink: u64 = ink_profile_rows(canvases.last().unwrap(), INK_THRESHOLD)
            .iter()
            .map(|&n| n as u64)
            .sum();
        assert_eq!(ink, 8 * 20);
    }

    #[test]
    fn compose_canvases_share_fixed_width() {
        let pages = vec![page_with_lines(6)];
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 300,
            tb_h: 40,
            pad_x: 0,
            pad_y: 0,
        };
        let scale = 1.0;
        let b = geom.content_height();
        let canvases = compose_pages(&pages, scale, &geom, b, b, &LineMetrics::sentinel());
        assert!(canvases.len() > 1, "strips should wrap across pages");
        for c in &canvases {
            assert_eq!(c.width(), geom.tb_w);
        }
    }

    #[test]
    fn compose_fills_page_then_wraps() {
        // Each strip (cut-point interval) is 12 px tall at scale 1.0; tb_h=24 fits
        // exactly two strips, so six strips → three canvases.
        let pages = vec![page_with_lines(6)];
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 300,
            tb_h: 24,
            pad_x: 0,
            pad_y: 0,
        };
        let b = geom.content_height();
        let canvases = compose_pages(&pages, 1.0, &geom, b, b, &LineMetrics::sentinel());
        assert_eq!(canvases.len(), 3);
    }

    #[test]
    fn compose_breaks_only_at_cut_points() {
        // Canvas heights must be exact sums of whole strip heights: no strip is
        // split across a page break.
        let pages = vec![page_with_lines(5)];
        let strips = strips_in_reading_order(&pages, &LineMetrics::sentinel());
        let scale = 1.0;
        let strip_heights: Vec<u32> = strips
            .iter()
            .map(|s| ((s.band.bottom - s.band.top) as f32 * scale).round() as u32)
            .collect();
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 300,
            tb_h: 24,
            pad_x: 0,
            pad_y: 0,
        };
        let b = geom.content_height();
        let canvases = compose_pages(&pages, scale, &geom, b, b, &LineMetrics::sentinel());

        // Every canvas height is a sum of a consecutive run of strip heights.
        let mut i = 0usize;
        for c in &canvases {
            let mut acc = 0u32;
            while i < strip_heights.len() && acc + strip_heights[i] <= c.height() {
                acc += strip_heights[i];
                i += 1;
            }
            assert_eq!(acc, c.height(), "canvas height not a whole-strip sum");
        }
        assert_eq!(i, strip_heights.len(), "all strips placed");
    }

    #[test]
    fn compose_insets_content_with_safety_margins() {
        // With non-zero safety insets, the top `pad_y` rows and left `pad_x`
        // columns of a canvas are white, keeping type off the trim and binding.
        let pages = vec![page_with_lines(3)];
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 200,
            tb_h: 400,
            pad_x: 20,
            pad_y: 15,
        };
        let b = geom.content_height();
        let canvases = compose_pages(&pages, 1.0, &geom, b, b, &LineMetrics::sentinel());
        assert_eq!(canvases.len(), 1);
        let c = &canvases[0];
        assert_eq!(c.width(), geom.tb_w);
        // Top inset band is entirely white.
        for y in 0..geom.pad_y {
            for x in 0..c.width() {
                assert_eq!(c.get_pixel(x, y).0[0], 255, "top inset not white");
            }
        }
        // Left inset band is entirely white.
        for y in 0..c.height() {
            for x in 0..geom.pad_x {
                assert_eq!(c.get_pixel(x, y).0[0], 255, "left inset not white");
            }
        }
        // There is ink somewhere past the insets (content was placed).
        let has_ink = (geom.pad_x..c.width())
            .flat_map(|x| (geom.pad_y..c.height()).map(move |y| (x, y)))
            .any(|(x, y)| c.get_pixel(x, y).0[0] < 128);
        assert!(has_ink, "no content placed after insets");
    }

    #[test]
    fn compose_never_exceeds_budget() {
        // One strip far taller than the budget: it must be split so every returned
        // canvas is within budget.
        let w = 100;
        let img = column(500, w, &[(0, 500, 5, 95)]); // one 500px-tall solid band
        let bands = line_bands(&img, 3);
        assert_eq!(bands.len(), 1);
        let pages = vec![(img, bands)];
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 100,
            tb_h: 120,
            pad_x: 0,
            pad_y: 0,
        };
        let budget = 120;
        let canvases = compose_pages(&pages, 1.0, &geom, budget, budget, &LineMetrics::sentinel());
        assert!(canvases.len() >= 4, "a 500px strip must split into pieces");
        for c in &canvases {
            assert!(
                c.height() <= budget,
                "canvas height {} > budget",
                c.height()
            );
        }
    }

    #[test]
    fn oversized_strip_split_at_low_ink() {
        // A tall strip with a clear whitespace row near the budget boundary is cut
        // there, not mid-glyph. Ink in [0,40) and [60,100); a gap at [40,60).
        let w = 50;
        let img = column(100, w, &[(0, 40, 5, 45), (60, 100, 5, 45)]);
        let pieces = split_tall_strip(&img, 70);
        assert_eq!(pieces.len(), 2);
        // The first piece ends inside the whitespace gap [40,60), so no glyph row is
        // bisected: its bottom rows are white.
        let first = &pieces[0];
        let cut = first.height();
        assert!((41..=60).contains(&cut), "cut {cut} not in the gap");
    }

    #[test]
    fn drop_interior_running_head_removes_low_head() {
        // A short centered band (bled running head) detached by a large gap above
        // three full-measure body lines is dropped (models a bled `…COMPARATIO. 227`).
        let w = 100;
        let img = column(
            200,
            w,
            &[
                (5, 12, 40, 60),   // short head: span 20 < 0.8·100
                (60, 70, 5, 95),   // body line, big gap (48) before it
                (80, 90, 5, 95),   // body line
                (100, 110, 5, 95), // body line
            ],
        );
        let bands = line_bands(&img, 3);
        assert_eq!(bands.len(), 4);
        let kept = drop_interior_running_head(&img, &bands, &LineMetrics::sentinel());
        assert_eq!(kept.len(), 3);
        assert_eq!(kept[0].top, 60);
    }

    #[test]
    fn drop_interior_running_head_keeps_full_measure_first_line() {
        // A full-measure first body line at the same height, even with a large gap
        // after it, is kept: short AND detached are both required to drop.
        let w = 100;
        let img = column(
            200,
            w,
            &[
                (5, 15, 5, 95), // full-measure line: span 90
                (60, 70, 5, 95),
                (80, 90, 5, 95),
                (100, 110, 5, 95),
            ],
        );
        let bands = line_bands(&img, 3);
        let kept = drop_interior_running_head(&img, &bands, &LineMetrics::sentinel());
        assert_eq!(kept.len(), 4);
        assert_eq!(kept[0].top, 5);
    }

    #[test]
    fn drop_trailing_signature_numeral() {
        // Four full-measure body lines, then a short narrow numeral detached at the
        // foot (a bled signature like "2"): the numeral is dropped.
        let w = 100;
        let specs = [
            (0, 10, 5, 95),
            (14, 24, 5, 95),
            (28, 38, 5, 95),
            (42, 52, 5, 95),
            (90, 94, 45, 55), // tiny narrow numeral, big gap before it
        ];
        let (page, m) = page_and_metrics(100, w, &specs);
        let kept = drop_interior_running_head(&page.0, &page.1, &m);
        assert_eq!(kept.len(), 4);
        assert_eq!(kept.last().unwrap().top, 42);

        // A full-measure last line at the same foot position is kept.
        let specs2 = [
            (0, 10, 5, 95),
            (14, 24, 5, 95),
            (28, 38, 5, 95),
            (42, 52, 5, 95),
            (90, 100, 5, 95), // full-measure last line
        ];
        let (page2, m2) = page_and_metrics(100, w, &specs2);
        let kept2 = drop_interior_running_head(&page2.0, &page2.1, &m2);
        assert_eq!(kept2.len(), 5);
    }

    #[test]
    fn first_page_budget_leaves_room_for_header() {
        let geom = TrimGeom::new(300);
        // The first canvas gets less room than a later one, for the inline header.
        assert!(geom.first_body_budget() < geom.body_budget());
        assert!(geom.first_body_budget() > 0);
    }

    #[test]
    fn full_page_keeps_bottom_slack() {
        // The per-canvas budget leaves a bottom margin below the content height.
        let geom = TrimGeom::new(300);
        assert!(geom.body_budget() < geom.content_height());
        assert!(geom.body_budget() > 0);
        // A fully packed canvas stays within the reduced budget.
        let pages = vec![page_with_lines(40)];
        let b = geom.body_budget();
        let canvases = compose_pages(&pages, 1.0, &geom, b, b, &LineMetrics::sentinel());
        for c in &canvases {
            assert!(
                c.height() <= b,
                "canvas {} exceeds body budget {}",
                c.height(),
                b
            );
        }
    }

    /// A synthetic body page for `unit_body_strips`: a full-height Greek block on the
    /// left, a gutter, then a Latin block on the right whose rows are `lat_lines`
    /// (`(top, bottom, left, right)` in page coordinates). Mirrors the detection
    /// fixture in `columns.rs`, so `detect_latin_column` crops to the Latin block.
    fn body_page(h: u32, lat_lines: &[(u32, u32, u32, u32)]) -> GrayImage {
        let w = 200;
        let mut img = GrayImage::from_pixel(w, h, Luma([255]));
        for y in 0..h {
            for x in 10..80 {
                img.put_pixel(x, y, Luma([0]));
            }
        }
        for &(t, b, l, r) in lat_lines {
            for y in t..b {
                for x in l..r {
                    img.put_pixel(x, y, Luma([0]));
                }
            }
        }
        img
    }

    #[test]
    fn unit_body_strips_returns_metrics() {
        // A Latin column with a narrow, isolated display title over seven body lines.
        let mut lat = vec![(4u32, 12u32, 145u32, 165u32)]; // narrow centered title
        let mut y = 40;
        for _ in 0..7 {
            lat.push((y, y + 10, 122, 188));
            y += 14;
        }
        let img = body_page(140, &lat);
        let unit = crate::facsimile::units::Unit {
            id: "u".to_string(),
            latin_title: "U".to_string(),
            greek_title: "Τ".to_string(),
            kind: crate::tags::LifeKind::Life,
            group: 1,
            source: 0,
            start_page: 1,
            skip_top_bands: None,
            title_top_frac: None,
            body_top_frac: None,
            last_page: None,
            pinned: false,
            review_notes: None,
        };
        let range = UnitRange {
            unit,
            end_page: 3,
            shared_tail: false,
            open_body_frac: None,
            tail_cut_frac: None,
        };
        let (pages, metrics) = unit_body_strips(&range, |_, _| Ok(img.clone())).unwrap();
        assert_eq!(pages.len(), 3);
        assert!(!metrics.is_sentinel());
        assert!(metrics.height_med > 0);
        // The first page's opening (the narrow title) is dropped: its first kept band
        // is the first body line, well below the title's top at y≈4.
        assert!(pages[0].1[0].top > 20, "title not dropped on first page");

        // Boundary coordinates belong to source pages, even if rendering fails.
        let mut missing_boundaries = range.clone();
        missing_boundaries.open_body_frac = Some(0.8);
        missing_boundaries.tail_cut_frac = Some(0.2);
        missing_boundaries.shared_tail = true;
        let (surviving, _) = unit_body_strips(&missing_boundaries, |_, page| {
            if page == 2 {
                Ok(img.clone())
            } else {
                anyhow::bail!("unavailable page")
            }
        })
        .unwrap();
        assert_eq!(surviving.len(), 1);
        assert_eq!(surviving[0].1, pages[1].1);
    }

    /// A page whose bands are `specs`, plus the metrics computed over it.
    fn page_and_metrics(
        h: u32,
        w: u32,
        specs: &[(u32, u32, u32, u32)],
    ) -> ((GrayImage, Vec<Band>), LineMetrics) {
        let img = column(h, w, specs);
        let bands = line_bands(&img, 3);
        let m = line_metrics(std::slice::from_ref(&(img.clone(), bands.clone())));
        ((img, bands), m)
    }

    #[test]
    fn greedy_fill_splits_tall_strip_to_fill_page() {
        // Three normal lines then one tall (merged) band; a budget that leaves a
        // partial canvas after the normal lines must be filled by the tall strip.
        let w = 100;
        let specs = [
            (0, 8, 5, 95),
            (12, 20, 5, 95),
            (24, 32, 5, 95),
            (40, 160, 5, 95), // tall merged band
        ];
        let (page, m) = page_and_metrics(200, w, &specs);
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 100,
            tb_h: 60,
            pad_x: 0,
            pad_y: 0,
        };
        let b = 60;
        let canvases = compose_pages(&[page], 1.0, &geom, b, b, &m);
        assert!(
            canvases.len() >= 2,
            "remainder must flow onto later canvases"
        );
        for c in &canvases {
            assert!(
                c.height() <= b,
                "canvas {} exceeds budget {}",
                c.height(),
                b
            );
        }
        // The first canvas is filled to within one median line of the budget.
        assert!(
            canvases[0].height() >= b - m.height_med,
            "first canvas {} not filled toward budget {}",
            canvases[0].height(),
            b
        );
    }

    #[test]
    fn greedy_fill_keeps_single_line_whole() {
        // Normal lines only: a line that does not fit the remaining space wraps whole
        // (no canvas exceeds budget, and every canvas height is a whole-strip sum).
        let (page, m) = {
            let p = page_with_lines(6);
            let m = line_metrics(std::slice::from_ref(&p));
            (p, m)
        };
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 100,
            tb_h: 26,
            pad_x: 0,
            pad_y: 0,
        };
        let b = 26;
        let strips = strips_in_reading_order(std::slice::from_ref(&page), &m);
        let strip_heights: Vec<u32> = strips.iter().map(|s| s.band.bottom - s.band.top).collect();
        let canvases = compose_pages(&[page], 1.0, &geom, b, b, &m);
        let mut i = 0usize;
        for c in &canvases {
            assert!(
                c.height() <= b,
                "canvas {} exceeds budget {}",
                c.height(),
                b
            );
            let mut acc = 0u32;
            while i < strip_heights.len() && acc + strip_heights[i] <= c.height() {
                acc += strip_heights[i];
                i += 1;
            }
            assert_eq!(acc, c.height(), "a normal line was bisected");
        }
        assert_eq!(i, strip_heights.len(), "all strips placed");
    }

    #[test]
    fn no_large_blank_before_tall_strip() {
        // Several normal lines then a tall merged band: the canvas before the tall
        // strip's remainder is filled to within one median line of budget (no blank).
        let w = 100;
        let specs = [
            (0, 8, 5, 95),
            (12, 20, 5, 95),
            (24, 32, 5, 95),
            (36, 44, 5, 95),
            (52, 172, 5, 95), // tall merged band
        ];
        let (page, m) = page_and_metrics(180, w, &specs);
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 100,
            tb_h: 60,
            pad_x: 0,
            pad_y: 0,
        };
        let b = 60;
        let canvases = compose_pages(&[page], 1.0, &geom, b, b, &m);
        assert!(
            canvases[0].height() >= b - m.height_med,
            "blank before tall strip: canvas {} vs budget {}",
            canvases[0].height(),
            b
        );
        for c in &canvases {
            assert!(c.height() <= b);
        }
    }

    #[test]
    fn find_title_band_picks_centered_isolated_band() {
        // Full-measure tail lines, a wide gap, a narrow centered title, then body.
        let w = 100;
        let specs = [
            (0, 8, 5, 95),
            (12, 20, 5, 95),
            (24, 32, 5, 95),
            (60, 70, 40, 60), // narrow centered title (big gap before)
            (80, 88, 5, 95),
            (92, 100, 5, 95),
        ];
        let (page, m) = page_and_metrics(110, w, &specs);
        assert_eq!(find_title_band(&page.0, &page.1, &m), Some(3));
    }

    #[test]
    fn find_title_band_none_on_clean_page() {
        let w = 100;
        let specs = [
            (0, 8, 5, 95),
            (12, 20, 5, 95),
            (24, 32, 5, 95),
            (36, 44, 5, 95),
        ];
        let (page, m) = page_and_metrics(60, w, &specs);
        assert_eq!(find_title_band(&page.0, &page.1, &m), None);
    }

    #[test]
    fn top_trim_drops_previous_tail_and_title() {
        let w = 100;
        let specs = [
            (0, 8, 5, 95),
            (12, 20, 5, 95),
            (24, 32, 5, 95),
            (60, 70, 40, 60), // title
            (80, 88, 5, 95),
            (92, 100, 5, 95),
        ];
        let (page, m) = page_and_metrics(110, w, &specs);
        let kept = drop_opening_bands(&page.0, &page.1, None, None, &m);
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[0].top, 80);
    }

    fn opening_fraction_fixture(title_left: u32, title_right: u32) -> (PageBands, LineMetrics) {
        page_and_metrics(
            150,
            100,
            &[
                (0, 8, 5, 95),
                (12, 20, 5, 95),
                (24, 32, 5, 95),
                (50, 52, 40, 60),
                (70, 80, title_left, title_right),
                (100, 108, 5, 95),
                (112, 120, 5, 95),
                (124, 132, 5, 95),
            ],
        )
    }

    #[test]
    fn opening_frac_drops_prev_tail_separator_title() {
        let (page, m) = opening_fraction_fixture(40, 60);
        let kept = drop_opening_bands(&page.0, &page.1, None, Some(100.0 / 150.0), &m);
        assert_eq!(kept, page.1[5..]);
    }

    #[test]
    fn opening_frac_handles_wide_title() {
        let (page, m) = opening_fraction_fixture(5, 95);
        let kept = drop_opening_bands(&page.0, &page.1, None, Some(100.0 / 150.0), &m);
        assert_eq!(kept, page.1[5..]);
        // A hand-authored band count takes precedence even with usable metrics.
        assert_eq!(
            drop_opening_bands(&page.0, &page.1, Some(2), Some(100.0 / 150.0), &m),
            page.1[2..]
        );
    }

    #[test]
    fn opening_falls_back_to_heuristic_without_frac() {
        let (page, m) = page_and_metrics(
            110,
            100,
            &[
                (0, 8, 5, 95),
                (12, 20, 5, 95),
                (24, 32, 5, 95),
                (60, 70, 40, 60),
                (80, 88, 5, 95),
                (92, 100, 5, 95),
            ],
        );
        let kept = drop_opening_bands(&page.0, &page.1, None, None, &m);
        assert_eq!(kept, page.1[4..]);
        assert_eq!(
            drop_opening_bands(&page.0, &page.1, None, Some(f32::NAN), &m),
            kept
        );
        assert_eq!(
            drop_opening_bands(&page.0, &page.1, None, Some(0.8), &LineMetrics::sentinel()),
            drop_opening_bands(&page.0, &page.1, None, None, &LineMetrics::sentinel()),
        );
    }

    #[test]
    fn bottom_trim_keeps_previous_tail() {
        let w = 100;
        let specs = [
            (0, 8, 5, 95),
            (12, 20, 5, 95),
            (24, 32, 5, 95),  // prev body
            (60, 70, 40, 60), // next unit's title
            (80, 88, 5, 95),
            (92, 100, 5, 95), // next body
        ];
        let (page, m) = page_and_metrics(110, w, &specs);
        let kept = drop_shared_tail(&page.0, &page.1, None, &m);
        assert_eq!(kept.len(), 3);
        assert_eq!(kept[2].top, 24);
    }

    #[test]
    fn shared_tail_frac_keeps_prev_body_drops_separator() {
        let (page, m) = opening_fraction_fixture(5, 95);
        let kept = drop_shared_tail(&page.0, &page.1, Some(70.0 / 150.0), &m);
        assert_eq!(kept, page.1[..3]);
    }

    #[test]
    fn shared_tail_frac_keeps_short_attached_last_line() {
        let (page, m) = page_and_metrics(
            110,
            100,
            &[
                (0, 8, 5, 95),
                (12, 20, 5, 95),
                (24, 32, 40, 60),
                (60, 70, 5, 95),
                (80, 88, 5, 95),
            ],
        );
        assert_eq!(
            drop_shared_tail(&page.0, &page.1, Some(60.0 / 110.0), &m),
            page.1[..3]
        );
        assert_eq!(
            drop_shared_tail(&page.0, &page.1, Some(0.5), &LineMetrics::sentinel()),
            drop_shared_tail(&page.0, &page.1, None, &LineMetrics::sentinel()),
        );
    }

    #[test]
    fn single_page_fraction_trims_both_ends() {
        let (page, m) = page_and_metrics(
            180,
            100,
            &[
                (0, 8, 5, 95),
                (25, 35, 5, 95),
                (50, 58, 5, 95),
                (62, 70, 5, 95),
                (74, 82, 5, 95),
                (110, 112, 40, 60),
                (130, 140, 5, 95),
                (155, 163, 5, 95),
            ],
        );
        let after_top = drop_opening_bands(&page.0, &page.1, None, Some(50.0 / 180.0), &m);
        assert_eq!(
            drop_shared_tail(&page.0, &after_top, Some(130.0 / 180.0), &m),
            page.1[2..5]
        );
    }

    #[test]
    fn review_targets_only_outlier_bands() {
        // A narrow title, three normal body lines, and a tall merged band: only the
        // title (narrow) and the tall band are review candidates.
        let w = 100;
        let specs = [
            (0, 8, 40, 60),  // narrow title
            (12, 20, 5, 95), // body
            (24, 32, 5, 95), // body
            (36, 44, 5, 95), // body
            (50, 90, 5, 95), // tall merged band
        ];
        let (page, m) = page_and_metrics(100, w, &specs);
        assert_eq!(review_candidates(&page.0, &page.1, &m), vec![0, 4]);
    }

    #[test]
    fn single_page_unit_trims_both_ends() {
        // [prev tail][own title][own body][next title][next body] → keep [own body].
        let w = 100;
        let specs = [
            (0, 8, 5, 95),
            (12, 20, 5, 95),  // prev tail
            (48, 58, 40, 60), // own title
            (70, 78, 5, 95),
            (82, 90, 5, 95),    // own body
            (120, 130, 40, 60), // next title
            (140, 148, 5, 95),
            (152, 160, 5, 95), // next body
        ];
        let (page, m) = page_and_metrics(170, w, &specs);
        let after_top = drop_opening_bands(&page.0, &page.1, None, None, &m);
        let kept = drop_shared_tail(&page.0, &after_top, None, &m);
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[0].top, 70);
        assert_eq!(kept[1].top, 82);
    }
}
