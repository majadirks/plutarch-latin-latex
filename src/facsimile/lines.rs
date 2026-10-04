//! Line-band detection and source-furniture removal from a cropped column's
//! horizontal ink-projection profile.
//!
//! Summing dark pixels per row gives a profile that is near-zero in inter-line
//! gaps and high over glyph rows, yielding clean line bands. The running head and
//! page number live in the top zone and the "Digitized by Google" watermark in the
//! bottom zone; both are excluded, with a gap-outlier pass as belt-and-suspenders.

use image::GrayImage;

use crate::facsimile::columns::INK_THRESHOLD;

/// A contiguous run of texty rows: `top` inclusive, `bottom` exclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Band {
    pub top: u32,
    pub bottom: u32,
}

/// Fraction of page height at the top treated as furniture (running head / folio).
/// Raised from the original 6.5 % so a running head that bleeds just below the old
/// zone is dropped; tuned so it does not clip the first body line on normal pages.
const TOP_FURNITURE_FRAC: f32 = 0.08;
/// Fraction of page height below which content is body (the rest is the bottom
/// watermark belt).
const BOTTOM_BODY_FRAC: f32 = 0.95;
/// A leading/trailing gap this many times the median inter-line gap marks a
/// detached furniture band.
const GAP_OUTLIER_FACTOR: f32 = 2.2;

/// Dark-pixel count per row (horizontal ink-projection profile). Exposed so the
/// reflow splitter can find a low-ink row to cut an oversized strip at.
pub(crate) fn ink_profile_rows(img: &GrayImage, thr: u8) -> Vec<u32> {
    let (w, h) = img.dimensions();
    let mut rows = vec![0u32; h as usize];
    for y in 0..h {
        let mut count = 0;
        for x in 0..w {
            if img.get_pixel(x, y).0[0] <= thr {
                count += 1;
            }
        }
        rows[y as usize] = count;
    }
    rows
}

/// Split a cropped column into text bands: maximal runs of rows whose ink exceeds
/// `min_ink`.
pub fn line_bands(col: &GrayImage, min_ink: u32) -> Vec<Band> {
    let rows = ink_profile_rows(col, INK_THRESHOLD);
    let h = rows.len();
    let mut bands = Vec::new();
    let mut y = 0;
    while y < h {
        if rows[y] > min_ink {
            let start = y;
            while y < h && rows[y] > min_ink {
                y += 1;
            }
            bands.push(Band {
                top: start as u32,
                bottom: y as u32,
            });
        } else {
            y += 1;
        }
    }
    bands
}

/// Drop source furniture: keep only bands whose center lies in the body zone (top
/// 6.5% / bottom 5% of page height excluded), then drop a leading or trailing band
/// separated from the body by an abnormally large gap.
pub fn drop_furniture(bands: &[Band], page_h: u32) -> Vec<Band> {
    let top_zone = (TOP_FURNITURE_FRAC * page_h as f32) as u32;
    let bot_zone = (BOTTOM_BODY_FRAC * page_h as f32) as u32;
    let kept: Vec<Band> = bands
        .iter()
        .copied()
        .filter(|b| {
            let center = (b.top + b.bottom) / 2;
            center >= top_zone && center <= bot_zone
        })
        .collect();
    drop_gap_outliers(kept)
}

/// Drop a detached first/last band whose adjoining gap far exceeds the median
/// inter-line gap.
fn drop_gap_outliers(bands: Vec<Band>) -> Vec<Band> {
    if bands.len() < 3 {
        return bands;
    }
    let mut gaps: Vec<i64> = bands
        .windows(2)
        .map(|w| w[1].top as i64 - w[0].bottom as i64)
        .collect();
    let med = median(&gaps);
    let mut out = bands;
    if !gaps.is_empty() && gaps[0] as f32 > GAP_OUTLIER_FACTOR * med {
        out.remove(0);
        gaps.remove(0);
    }
    if let Some(&last_gap) = gaps.last() {
        if last_gap as f32 > GAP_OUTLIER_FACTOR * med {
            out.pop();
        }
    }
    out
}

/// Median of a non-empty slice, as f32.
fn median(values: &[i64]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let mid = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[mid - 1] + sorted[mid]) as f32 / 2.0
    } else {
        sorted[mid] as f32
    }
}

/// The break points between successive strips: the first band's top, each
/// inter-line gap midpoint, and the last band's bottom. The interior breaks (gap
/// midpoints) fall in zero-ink rows, so a reflow break never bisects a line.
pub fn cut_points(bands: &[Band]) -> Vec<u32> {
    if bands.is_empty() {
        return Vec::new();
    }
    let mut cuts = Vec::with_capacity(bands.len() + 1);
    cuts.push(bands[0].top);
    for w in bands.windows(2) {
        cuts.push((w[0].bottom + w[1].top) / 2);
    }
    cuts.push(bands[bands.len() - 1].bottom);
    cuts
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Luma;

    /// A synthetic column image: white, with the given `[top,bottom)` row-bands
    /// filled dark across the full width.
    fn col_image(h: u32, bands: &[(u32, u32)]) -> GrayImage {
        let w = 50;
        let mut img = GrayImage::from_pixel(w, h, Luma([255]));
        for &(t, b) in bands {
            for y in t..b {
                for x in 5..45 {
                    img.put_pixel(x, y, Luma([0]));
                }
            }
        }
        img
    }

    #[test]
    fn line_bands_splits_on_gaps() {
        let img = col_image(100, &[(10, 20), (30, 40), (50, 60)]);
        let bands = line_bands(&img, 3);
        assert_eq!(bands.len(), 3);
        assert_eq!(
            bands[0],
            Band {
                top: 10,
                bottom: 20
            }
        );
        assert_eq!(
            bands[1],
            Band {
                top: 30,
                bottom: 40
            }
        );
        assert_eq!(
            bands[2],
            Band {
                top: 50,
                bottom: 60
            }
        );
    }

    #[test]
    fn drop_furniture_removes_top_and_bottom_zones() {
        let page_h = 100;
        // Furniture at top (center 2) and bottom (center 98), body in between.
        let bands = vec![
            Band { top: 1, bottom: 3 },
            Band {
                top: 10,
                bottom: 20,
            },
            Band {
                top: 40,
                bottom: 50,
            },
            Band {
                top: 70,
                bottom: 80,
            },
            Band {
                top: 97,
                bottom: 99,
            },
        ];
        let kept = drop_furniture(&bands, page_h);
        assert_eq!(kept.len(), 3);
        let top_zone = (TOP_FURNITURE_FRAC * page_h as f32) as u32;
        let bot_zone = (BOTTOM_BODY_FRAC * page_h as f32) as u32;
        for b in &kept {
            let center = (b.top + b.bottom) / 2;
            assert!(center >= top_zone && center <= bot_zone);
        }
    }

    #[test]
    fn raised_top_zone_drops_bled_head_between_old_and_new_zone() {
        let page_h = 1000;
        // A band centered at 7 % of height sits below the old 6.5 % zone but inside
        // the raised ~8 % zone, so it is now dropped; a body line at 10 % survives.
        let bands = vec![
            Band {
                top: 68,
                bottom: 72,
            }, // center 70 = 7 %
            Band {
                top: 98,
                bottom: 102,
            }, // center 100 = 10 %
            Band {
                top: 300,
                bottom: 320,
            },
            Band {
                top: 600,
                bottom: 620,
            },
        ];
        let kept = drop_furniture(&bands, page_h);
        assert!(kept.iter().all(|b| (b.top + b.bottom) / 2 >= 80));
        assert_eq!(kept[0].top, 98);
    }

    #[test]
    fn drop_furniture_removes_detached_leading_band() {
        // A leading band separated by a huge gap from the three evenly spaced body
        // bands is dropped by the gap-outlier pass.
        let bands = vec![
            Band {
                top: 10,
                bottom: 15,
            },
            Band {
                top: 60,
                bottom: 70,
            },
            Band {
                top: 80,
                bottom: 90,
            },
            Band {
                top: 100,
                bottom: 110,
            },
        ];
        let kept = drop_furniture(&bands, 200);
        assert_eq!(kept.len(), 3);
        assert_eq!(kept[0].top, 60);
    }

    #[test]
    fn cut_points_lie_in_gaps() {
        let img = col_image(100, &[(10, 20), (40, 50), (70, 80)]);
        let bands = line_bands(&img, 3);
        let cuts = cut_points(&bands);
        assert_eq!(cuts.len(), bands.len() + 1);
        assert_eq!(*cuts.first().unwrap(), 10);
        assert_eq!(*cuts.last().unwrap(), 80);

        // Every interior break (a strip boundary between two lines) is a zero-ink
        // row, so no line is bisected.
        let rows = ink_profile_rows(&img, INK_THRESHOLD);
        for &c in &cuts[1..cuts.len() - 1] {
            assert_eq!(rows[c as usize], 0, "interior cut {c} is not in a gap");
        }
    }
}
