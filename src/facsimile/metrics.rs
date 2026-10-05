//! Robust per-unit line metrics and band classification.
//!
//! A unit's body lines share one font, one leading, and one measure, so their
//! heights and ink-span widths cluster tightly. The **median** height/width with
//! the **median absolute deviation (MAD)** of the heights gives a central estimate
//! that a few merged ("giant") or partial bands cannot skew. Classifying each band
//! against that estimate gives one shared definition of "tall" / "short" / "narrow"
//! that the fill-split (reflow), title detection, and furniture drop all key off,
//! instead of each inventing its own threshold.

use image::GrayImage;

use crate::facsimile::columns::INK_THRESHOLD;
use crate::facsimile::lines::Band;

/// A band is **tall** (a merged giant band) when its height exceeds the median by
/// this many MADs. Tunable.
pub const TALL_MAD_K: f32 = 3.0;
/// …and also at least this multiple of the median height, so a tight distribution
/// with a near-zero MAD does not over-flag ordinary lines. Tunable.
pub const TALL_MIN_FACTOR: f32 = 1.6;
/// A band is **short** when its height is under this multiple of the median. Tunable.
pub const SHORT_FACTOR: f32 = 0.5;
/// A band is **narrow** when its ink span is under this multiple of the median
/// width (a centered title or a short cross-reference line). Tunable.
pub const NARROW_FACTOR: f32 = 0.6;

/// Robust typical line dimensions for a unit's pages, in original (source-column)
/// pixels. `height_med == 0` is the sentinel for "no usable bands"; callers then
/// fall back to their non-metric behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineMetrics {
    pub height_med: u32,
    pub height_mad: u32,
    pub width_med: u32,
}

impl LineMetrics {
    /// The sentinel returned when no band is available.
    pub fn sentinel() -> Self {
        LineMetrics {
            height_med: 0,
            height_mad: 0,
            width_med: 0,
        }
    }

    /// Whether this is the no-usable-bands sentinel.
    pub fn is_sentinel(&self) -> bool {
        self.height_med == 0
    }
}

/// How a band compares to the unit's typical line. `Narrow` and the height classes
/// can co-occur; [`classify_band`] returns the dominant class while [`is_narrow`]
/// exposes the width test on its own for callers that combine them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandClass {
    Normal,
    Tall,
    Short,
    Narrow,
}

/// Compute robust typical line dimensions over every band of every page: the median
/// band height and ink-span width, plus the MAD of the heights. Empty input yields
/// the [`LineMetrics::sentinel`].
pub fn line_metrics(pages: &[(GrayImage, Vec<Band>)]) -> LineMetrics {
    let mut heights: Vec<u32> = Vec::new();
    let mut widths: Vec<u32> = Vec::new();
    for (col, bands) in pages {
        for band in bands {
            heights.push(band.bottom.saturating_sub(band.top));
            widths.push(band_ink_span(col, band));
        }
    }
    if heights.is_empty() {
        return LineMetrics::sentinel();
    }
    let height_med = median_u32(&heights);
    let width_med = median_u32(&widths);
    let devs: Vec<u32> = heights.iter().map(|&h| h.abs_diff(height_med)).collect();
    let height_mad = median_u32(&devs);
    LineMetrics {
        height_med: height_med.max(1),
        height_mad,
        width_med,
    }
}

/// Classify a band against the unit's typical line: `Tall` (merged lines), `Short`,
/// `Narrow` (centered / short measure), or `Normal`. On the sentinel metrics every
/// band is `Normal`, so callers fall back to their non-metric behavior.
pub fn classify_band(col: &GrayImage, band: &Band, m: &LineMetrics) -> BandClass {
    if m.is_sentinel() {
        return BandClass::Normal;
    }
    let h = band.bottom.saturating_sub(band.top) as f32;
    let med = m.height_med as f32;
    let mad = m.height_mad as f32;
    if h > med + TALL_MAD_K * mad && h > med * TALL_MIN_FACTOR {
        BandClass::Tall
    } else if h < med * SHORT_FACTOR {
        BandClass::Short
    } else if is_narrow(col, band, m) {
        BandClass::Narrow
    } else {
        BandClass::Normal
    }
}

/// Whether a band's horizontal ink span is well under the typical measure (a
/// centered title or a short cross-reference line). Always `false` on the sentinel.
pub fn is_narrow(col: &GrayImage, band: &Band, m: &LineMetrics) -> bool {
    if m.is_sentinel() {
        return false;
    }
    (band_ink_span(col, band) as f32) < m.width_med as f32 * NARROW_FACTOR
}

/// The horizontal extent (leftmost to rightmost ink column) of a band's rows.
pub(crate) fn band_ink_span(col: &GrayImage, band: &Band) -> u32 {
    band_ink_bounds(col, band).map_or(0, |(left, right)| right - left)
}

/// Number of horizontal coordinates containing ink in a band's rows.
pub(crate) fn band_ink_coverage(col: &GrayImage, band: &Band) -> u32 {
    (0..col.width())
        .filter(|&x| {
            (band.top..band.bottom.min(col.height()))
                .any(|y| col.get_pixel(x, y)[0] <= INK_THRESHOLD)
        })
        .count() as u32
}

/// Leftmost and exclusive rightmost ink coordinates in a band's rows.
pub(crate) fn band_ink_bounds(col: &GrayImage, band: &Band) -> Option<(u32, u32)> {
    let w = col.width();
    let mut left = None;
    let mut right = 0u32;
    for y in band.top..band.bottom.min(col.height()) {
        for x in 0..w {
            if col.get_pixel(x, y).0[0] <= INK_THRESHOLD {
                left = Some(left.map_or(x, |l: u32| l.min(x)));
                right = right.max(x);
            }
        }
    }
    left.map(|l| (l, right + 1))
}

/// Median of a non-empty `u32` slice, rounding a two-element midpoint down.
pub(crate) fn median_u32(values: &[u32]) -> u32 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        ((sorted[mid - 1] as u64 + sorted[mid] as u64) / 2) as u32
    } else {
        sorted[mid]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Luma;

    #[test]
    fn band_ink_coverage_excludes_whitespace_between_head_and_folio() {
        let col = column(40, 100, &[(10, 20, 5, 25), (10, 20, 90, 95)]);
        let band = Band {
            top: 10,
            bottom: 20,
        };
        assert_eq!(band_ink_span(&col, &band), 90);
        assert_eq!(band_ink_coverage(&col, &band), 25);
        assert_eq!(
            band_ink_coverage(
                &col,
                &Band {
                    top: 25,
                    bottom: 35
                }
            ),
            0
        );
    }

    #[test]
    fn band_ink_bounds_preserves_span_contract() {
        let col = column(40, 100, &[(10, 20, 5, 75)]);
        let body = Band {
            top: 10,
            bottom: 20,
        };
        assert_eq!(band_ink_bounds(&col, &body), Some((5, 75)));
        assert_eq!(band_ink_span(&col, &body), 70);
        let empty = Band {
            top: 25,
            bottom: 35,
        };
        assert_eq!(band_ink_bounds(&col, &empty), None);
        assert_eq!(band_ink_span(&col, &empty), 0);
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
    fn line_metrics_medians_ignore_outliers() {
        // ~20 uniform 8px-tall full-measure bands, plus one giant and one tiny band.
        let w = 100;
        let mut specs: Vec<(u32, u32, u32, u32)> = Vec::new();
        let mut y = 0u32;
        for _ in 0..20 {
            specs.push((y, y + 8, 5, 95));
            y += 12;
        }
        specs.push((y, y + 40, 5, 95)); // giant
        y += 44;
        specs.push((y, y + 2, 5, 95)); // tiny
        let img = column(y + 4, w, &specs);
        let bands = crate::facsimile::lines::line_bands(&img, 3);
        let m = line_metrics(&[(img, bands)]);
        // The median height tracks the uniform 8px lines, unmoved by the two outliers.
        assert!(
            (7..=9).contains(&m.height_med),
            "height_med {} not near 8",
            m.height_med
        );
    }

    #[test]
    fn classify_band_flags_tall_short_narrow() {
        // Uniform body (8px, full measure), a merged double-height band, a half-height
        // band, and a narrow centered band.
        let w = 100;
        let img = column(
            200,
            w,
            &[
                (0, 8, 5, 95),      // body
                (12, 20, 5, 95),    // body
                (24, 32, 5, 95),    // body
                (36, 44, 5, 95),    // body
                (48, 56, 5, 95),    // body
                (60, 84, 5, 95),    // tall: 24px
                (90, 93, 5, 95),    // short: 3px
                (100, 108, 42, 58), // narrow: 8px but span 16
            ],
        );
        let bands = crate::facsimile::lines::line_bands(&img, 3);
        let m = line_metrics(&[(img.clone(), bands.clone())]);
        let class_of = |top: u32| {
            let b = bands.iter().find(|b| b.top == top).copied().unwrap();
            classify_band(&img, &b, &m)
        };
        assert_eq!(class_of(0), BandClass::Normal);
        assert_eq!(class_of(60), BandClass::Tall);
        assert_eq!(class_of(90), BandClass::Short);
        assert_eq!(class_of(100), BandClass::Narrow);
    }

    #[test]
    fn sentinel_metrics_classify_everything_normal() {
        let w = 50;
        let img = column(20, w, &[(0, 8, 5, 45)]);
        let bands = crate::facsimile::lines::line_bands(&img, 3);
        let m = LineMetrics::sentinel();
        assert!(m.is_sentinel());
        assert_eq!(classify_band(&img, &bands[0], &m), BandClass::Normal);
        assert!(!is_narrow(&img, &bands[0], &m));
    }
}
