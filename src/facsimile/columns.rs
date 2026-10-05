//! Latin-column detection from a page's vertical ink-projection profile.
//!
//! A body page is Greek on the left and Latin on the right, with marginal
//! section-reference letters sitting in the gutter across a whitespace gap from the
//! Latin text margin. Summing dark pixels per column gives a profile with deep
//! valleys at the margins and the gutter and plateaus over each text column; the
//! Latin column is the dense block right of the gutter, and backing off to its own
//! left-margin whitespace excludes the sparse marginal letters.

use anyhow::{bail, Result};
use image::GrayImage;

/// Pixels at or below this luminance count as ink.
pub const INK_THRESHOLD: u8 = 170;

/// Box-smoothing window (columns) applied to the raw profile before thresholds.
const SMOOTH_WINDOW: usize = 9;

/// The detected Latin text column: `left` inclusive, `right` exclusive, in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Column {
    pub left: u32,
    pub right: u32,
}

/// Dark-pixel count per image column (vertical ink-projection profile).
pub fn ink_profile_cols(img: &GrayImage, thr: u8) -> Vec<u32> {
    let (w, h) = img.dimensions();
    let mut profile = vec![0u32; w as usize];
    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y).0[0] <= thr {
                profile[x as usize] += 1;
            }
        }
    }
    profile
}

/// Box-smooth a profile with a centered window of width `k`, zero-padding at the
/// edges (matching numpy's `convolve(..., mode="same")` with a `ones(k)/k` kernel).
fn smooth(profile: &[u32], k: usize) -> Vec<f32> {
    let n = profile.len();
    if n == 0 || k == 0 {
        return profile.iter().map(|&v| v as f32).collect();
    }
    let half = k / 2;
    let mut out = vec![0.0f32; n];
    for (i, slot) in out.iter_mut().enumerate() {
        let lo = i.saturating_sub(half);
        let hi = (i + half + 1).min(n);
        let sum: u32 = profile[lo..hi].iter().sum();
        *slot = sum as f32 / k as f32;
    }
    out
}

/// Detect the dense Latin text block, excluding the Greek column and the gutter
/// marginal letters.
pub fn detect_latin_column(img: &GrayImage) -> Result<Column> {
    let (w, h) = img.dimensions();
    if w == 0 || h == 0 {
        bail!("empty image; cannot detect Latin column");
    }
    let profile = ink_profile_cols(img, INK_THRESHOLD);
    let col = smooth(&profile, SMOOTH_WINDOW);
    let cmax = col.iter().copied().fold(0.0f32, f32::max);
    if cmax <= 0.0 {
        bail!("no ink detected; cannot find Latin column");
    }
    let wlen = w as usize;
    let wf = w as f32;

    // Gutter = lowest-ink column in the central band, where the Greek/Latin divide
    // sits.
    let lo = ((0.40 * wf) as usize).min(wlen.saturating_sub(1));
    let hi = ((0.62 * wf) as usize).clamp(lo + 1, wlen);
    let mut gutter = lo;
    let mut gmin = col[lo];
    for (x, &v) in col.iter().enumerate().take(hi).skip(lo) {
        if v < gmin {
            gmin = v;
            gutter = x;
        }
    }

    // Walk right to a column firmly inside the dense Latin body, then back off left
    // to the text block's own left-margin whitespace; the sparse marginal letters
    // live across a gap from this margin, so they are excluded.
    let mut x = gutter;
    while x < wlen - 1 && col[x] < 0.35 * cmax {
        x += 1;
    }
    while x > gutter && col[x] > 0.20 * cmax {
        x -= 1;
    }
    let latin_left = (x + 1) as u32;

    // Right edge: inward from the page's right until the first dense column.
    let mut x = wlen - 1;
    while x as u32 > latin_left && col[x] < 0.18 * cmax {
        x -= 1;
    }
    let latin_right = (x as u32 + 8).min(w);

    // Exclude a thin marginal line-number rule/tick column at the left when it is
    // detached from the body by a whitespace gap; it would otherwise bridge the
    // inter-line gaps and merge body lines into giant bands.
    let latin_left = exclude_left_ticks(&col, latin_left, latin_right, cmax);

    Ok(exclude_isolated_left_divider(
        img,
        Column {
            left: latin_left,
            right: latin_right.max(latin_left + 1).min(w),
        },
    ))
}

/// Exclude a connected narrow rule only when the discarded pixels are isolated.
fn exclude_isolated_left_divider(img: &GrayImage, column: Column) -> Column {
    let width = 48.min(column.right.saturating_sub(column.left));
    let height = img.height();
    if width == 0 || height == 0 {
        return column;
    }
    let mut visited = vec![false; (width * height) as usize];
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) as usize;
            if visited[index] || img.get_pixel(column.left + x, y).0[0] > INK_THRESHOLD {
                continue;
            }
            visited[index] = true;
            let mut pending = vec![(x, y)];
            let (mut left, mut right, mut top, mut bottom) = (x, x + 1, y, y + 1);
            let mut count = 0;
            while let Some((cx, cy)) = pending.pop() {
                count += 1;
                left = left.min(cx);
                right = right.max(cx + 1);
                top = top.min(cy);
                bottom = bottom.max(cy + 1);
                for ny in cy.saturating_sub(1)..=(cy + 1).min(height - 1) {
                    for nx in cx.saturating_sub(1)..=(cx + 1).min(width - 1) {
                        let i = (ny * width + nx) as usize;
                        if !visited[i] && img.get_pixel(column.left + nx, ny).0[0] <= INK_THRESHOLD
                        {
                            visited[i] = true;
                            pending.push((nx, ny));
                        }
                    }
                }
            }
            if bottom - top < 150.max(height / 20) || right - left > 12 || right == width {
                continue;
            }
            let edge = column.left + right;
            if edge + 8 >= column.right {
                continue;
            }
            if (edge..edge + 8)
                .any(|xx| (top..bottom).any(|yy| img.get_pixel(xx, yy).0[0] <= INK_THRESHOLD))
            {
                continue;
            }
            // Every discarded dark pixel must belong to this one component.
            let discarded = (0..height)
                .map(|yy| {
                    (column.left..edge)
                        .filter(|&xx| img.get_pixel(xx, yy).0[0] <= INK_THRESHOLD)
                        .count()
                })
                .sum::<usize>();
            if discarded != count {
                continue;
            }
            let retained = Column {
                left: edge,
                right: column.right,
            };
            if crate::facsimile::lines::line_bands(&crop_column(img, &retained), 3).len() >= 3 {
                return retained;
            }
        }
    }
    column
}

/// A left stripe this many times narrower than the whole detected block counts as a
/// thin marginal rule/tick column (not the body's own first glyphs).
const TICK_WIDTH_DIVISOR: usize = 8;

/// Advance the left edge past a thin, dense vertical stripe (a marginal line-number
/// rule/tick column) when it is detached from the dense body by a whitespace gap.
/// Such a stripe spans the inter-line gaps with ink and would merge body lines into
/// giant bands. Conservative: only a narrow stripe set off by a clear gap from a
/// following dense body is trimmed, so on a clean page the body's own left edge — and
/// thus the column width that feeds the global scale — is unchanged.
fn exclude_left_ticks(col: &[f32], left: u32, right: u32, cmax: f32) -> u32 {
    let dense = 0.35 * cmax;
    let white = 0.05 * cmax;
    let (l, r) = (left as usize, (right as usize).min(col.len()));
    if l >= r {
        return left;
    }
    // Skip any sub-dense ramp at the far left, then the dense stripe [s, i).
    let mut s = l;
    while s < r && col[s] < dense {
        s += 1;
    }
    if s >= r {
        return left; // no dense region at all
    }
    let mut i = s;
    while i < r && col[i] >= dense {
        i += 1;
    }
    let stripe_w = i - s;
    // The gap [i, j) runs until the body's dense columns resume.
    let mut j = i;
    while j < r && col[j] < dense {
        j += 1;
    }
    let body_follows = j < r;
    // A real whitespace separation (not just a dip) must sit in the gap, the stripe
    // must be thin, and a body must follow — otherwise this is the body itself.
    let gap_is_white = (i..j).any(|x| col[x] < white);
    let thin = stripe_w * TICK_WIDTH_DIVISOR <= (r - l);
    if body_follows && gap_is_white && thin {
        j as u32
    } else {
        left
    }
}

/// Crop a page image to the full-height Latin column.
pub fn crop_column(img: &GrayImage, c: &Column) -> GrayImage {
    let (w, h) = img.dimensions();
    let left = c.left.min(w);
    let right = c.right.min(w).max(left + 1).min(w);
    let width = right.saturating_sub(left).max(1);
    image::imageops::crop_imm(img, left, 0, width, h).to_image()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Luma;

    fn divider_fixture() -> (GrayImage, Column) {
        let mut img = page(200, 400, &[], &[]);
        for y in 50..350 {
            img.put_pixel(102, y, Luma([0]));
        }
        for top in [70, 150, 230] {
            for y in top..top + 15 {
                for x in 120..190 {
                    img.put_pixel(x, y, Luma([0]));
                }
            }
        }
        (
            img,
            Column {
                left: 100,
                right: 195,
            },
        )
    }
    #[test]
    fn isolated_left_divider_is_excluded() {
        let (img, c) = divider_fixture();
        let out = exclude_isolated_left_divider(&img, c);
        assert_eq!(
            out,
            Column {
                left: 103,
                right: 195
            }
        );
        assert_eq!(
            crop_column(&img, &out).get_pixel(120 - out.left, 70).0[0],
            0
        );
    }
    #[test]
    fn partial_divider_with_lower_left_body_is_preserved() {
        let (mut img, c) = divider_fixture();
        img.put_pixel(101, 370, Luma([0]));
        assert_eq!(exclude_isolated_left_divider(&img, c), c);
    }
    #[test]
    fn divider_touching_glyph_is_preserved() {
        let (mut img, c) = divider_fixture();
        for x in 103..121 {
            img.put_pixel(x, 70, Luma([0]));
        }
        assert_eq!(exclude_isolated_left_divider(&img, c), c);
    }
    #[test]
    fn short_vertical_glyph_is_preserved() {
        let (mut img, c) = divider_fixture();
        for y in 100..350 {
            img.put_pixel(102, y, Luma([255]));
        }
        assert_eq!(exclude_isolated_left_divider(&img, c), c);
    }
    #[test]
    fn ambiguous_left_noise_is_preserved() {
        let (mut img, c) = divider_fixture();
        img.put_pixel(104, 60, Luma([0]));
        assert_eq!(exclude_isolated_left_divider(&img, c), c);
    }
    #[test]
    fn clean_column_detection_is_unchanged() {
        let (mut img, c) = divider_fixture();
        for y in 50..350 {
            img.put_pixel(102, y, Luma([255]));
        }
        assert_eq!(exclude_isolated_left_divider(&img, c), c);
    }
    /// A synthetic page: white background, with the given `[left,right)` x-bands
    /// filled dark over their full height, plus optional sparse dark marks at
    /// `(x, y)` single pixels (marginal letters).
    fn page(w: u32, h: u32, dark_bands: &[(u32, u32)], marks: &[(u32, u32)]) -> GrayImage {
        let mut img = GrayImage::from_pixel(w, h, Luma([255]));
        for &(l, r) in dark_bands {
            for y in 0..h {
                for x in l..r {
                    img.put_pixel(x, y, Luma([0]));
                }
            }
        }
        for &(x, y) in marks {
            img.put_pixel(x, y, Luma([0]));
        }
        img
    }

    #[test]
    fn detect_column_finds_right_block_of_two() {
        // Greek block [10,80), gutter [80,120), Latin block [120,190).
        let img = page(200, 100, &[(10, 80), (120, 190)], &[]);
        let col = detect_latin_column(&img).unwrap();
        // The detected column sits right of the gutter and covers the Latin block.
        assert!(
            col.left > 100,
            "left {} leaked into the gutter/Greek",
            col.left
        );
        assert!(
            col.left <= 122,
            "left {} missed the Latin block start",
            col.left
        );
        assert!(
            col.right >= 189,
            "right {} clipped the Latin block",
            col.right
        );
        assert!(col.right <= 200);
    }

    #[test]
    fn detect_column_excludes_sparse_left_marks() {
        // A few isolated dark cells just left of the Latin column (gutter marginal
        // letters) must not pull the left edge back to them.
        let marks: Vec<(u32, u32)> = (40..49).map(|y| (108u32, y)).collect();
        let img = page(200, 100, &[(10, 80), (120, 190)], &marks);
        let col = detect_latin_column(&img).unwrap();
        assert!(
            col.left > 112,
            "left {} swept in the sparse marginal marks at x=108",
            col.left
        );
    }

    #[test]
    fn detect_column_handles_width_drift() {
        // The Latin block shifted right by 10px is still detected tightly.
        let img = page(200, 100, &[(10, 80), (130, 195)], &[]);
        let col = detect_latin_column(&img).unwrap();
        assert!(col.left > 110 && col.left <= 132, "left {}", col.left);
        assert!(col.right >= 194, "right {}", col.right);
    }

    #[test]
    fn crop_column_extracts_the_block() {
        let img = page(200, 100, &[(10, 80), (120, 190)], &[]);
        let col = detect_latin_column(&img).unwrap();
        let cropped = crop_column(&img, &col);
        assert_eq!(cropped.width(), col.right - col.left);
        assert_eq!(cropped.height(), 100);
    }

    #[test]
    fn crop_excludes_left_tick_column() {
        // A full-height rule at [118,124) (a marginal line-number column, wide enough
        // to survive smoothing as dense), detached by whitespace [124,135) from the
        // dense Latin body [135,195).
        let img = page(200, 100, &[(10, 80), (118, 124), (135, 195)], &[]);
        let col = detect_latin_column(&img).unwrap();
        // The crop begins past the detached rule (and its gap), not at the rule.
        assert!(
            col.left > 124,
            "left {} did not exclude the tick rule at 118-123",
            col.left
        );
        assert!(col.right >= 194, "right {} clipped the body", col.right);
    }

    #[test]
    fn left_tick_trim_preserves_column_width() {
        // The same body with and without the detached rule yields nearly the same
        // detected width, so the global scale is unaffected by the trim.
        let with_rule = page(200, 100, &[(10, 80), (118, 124), (135, 195)], &[]);
        let without = page(200, 100, &[(10, 80), (135, 195)], &[]);
        let a = detect_latin_column(&with_rule).unwrap();
        let b = detect_latin_column(&without).unwrap();
        let wa = a.right - a.left;
        let wb = b.right - b.left;
        assert!(wa.abs_diff(wb) <= 4, "width drifted: {wa} vs {wb}");
    }
}
