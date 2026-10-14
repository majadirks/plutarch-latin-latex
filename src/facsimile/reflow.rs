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

use crate::facsimile::columns::{crop_column, detect_latin_column, Column, INK_THRESHOLD};
use crate::facsimile::lines::{cut_points, drop_furniture, ink_profile_rows, line_bands, Band};
use crate::facsimile::metrics::{
    band_ink_bounds, band_ink_coverage, band_ink_span, classify_band, is_narrow, line_metrics,
    median_u32, BandClass, LineMetrics,
};
use crate::facsimile::units::UnitRange;

/// Default DPI the body column is rendered and composed at.
pub const BODY_DPI: u32 = 300;

/// Verified furniture regions in normalized, half-open full-source coordinates.
#[derive(Clone, Copy)]
struct SourceFurnitureMask {
    source: usize,
    page: u32,
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

const SOURCE_SHEARED_FURNITURE_MASKS: &[SourceFurnitureMask] = &[
    // Sheared-page gutter fragments outside the reviewed Latin text.
    SourceFurnitureMask {
        source: 0,
        page: 34,
        left: 945.0 / 1982.0,
        top: 1900.0 / 3040.0,
        right: 975.0 / 1982.0,
        bottom: 2651.0 / 3040.0,
    },
    SourceFurnitureMask {
        source: 0,
        page: 448,
        left: 909.0 / 1988.0,
        top: 315.0 / 3068.0,
        right: 929.0 / 1988.0,
        bottom: 1050.0 / 3068.0,
    },
    SourceFurnitureMask {
        source: 1,
        page: 266,
        left: 880.0 / 1878.0,
        top: 270.0 / 2896.0,
        right: 900.0 / 1878.0,
        bottom: 700.0 / 2896.0,
    },
];

const SOURCE_FURNITURE_MASKS: &[SourceFurnitureMask] = &[
    // Source-side vertical rules connect the Latin rows on these two pages.
    SourceFurnitureMask {
        source: 0,
        page: 17,
        left: 1043.0 / 2006.0,
        top: 270.0 / 2975.0,
        right: 1053.0 / 2006.0,
        bottom: 2655.0 / 2975.0,
    },
    SourceFurnitureMask {
        source: 0,
        page: 59,
        left: 1018.0 / 2002.0,
        top: 410.0 / 3005.0,
        right: 1028.0 / 2002.0,
        bottom: 2120.0 / 3005.0,
    },
    // The Themistocles source running head and folio sit above the first body row.
    SourceFurnitureMask {
        source: 0,
        page: 166,
        left: 900.0 / 1988.0,
        top: 210.0 / 2993.0,
        right: 1750.0 / 1988.0,
        bottom: 280.0 / 2993.0,
    },
    // Narrow right scan edges join unrelated Latin rows on these source pages.
    SourceFurnitureMask {
        source: 0,
        page: 229,
        left: 1932.0 / 1947.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    SourceFurnitureMask {
        source: 0,
        page: 257,
        left: 1995.0 / 2010.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    SourceFurnitureMask {
        source: 0,
        page: 276,
        left: 1971.0 / 1986.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    SourceFurnitureMask {
        source: 0,
        page: 292,
        left: 1979.0 / 1994.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    SourceFurnitureMask {
        source: 0,
        page: 297,
        left: 1885.0 / 1900.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    SourceFurnitureMask {
        source: 0,
        page: 310,
        left: 1975.0 / 1990.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    SourceFurnitureMask {
        source: 0,
        page: 329,
        left: 1864.0 / 1879.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Pale digitization watermark below the final Latin line.
    SourceFurnitureMask {
        source: 0,
        page: 233,
        left: 1180.0 / 1954.0,
        top: 2870.0 / 3005.0,
        right: 1605.0 / 1954.0,
        bottom: 2980.0 / 3005.0,
    },
    // Bottom scan edge below all Latin.
    SourceFurnitureMask {
        source: 0,
        page: 233,
        left: 0.0 / 1954.0,
        top: 2980.0 / 3005.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Right scan edge beyond the Latin measure.
    SourceFurnitureMask {
        source: 0,
        page: 233,
        left: 1920.0 / 1954.0,
        top: 0.0 / 3005.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Pale digitization watermark below the final Latin line.
    SourceFurnitureMask {
        source: 0,
        page: 235,
        left: 1180.0 / 1969.0,
        top: 2880.0 / 3013.0,
        right: 1605.0 / 1969.0,
        bottom: 2985.0 / 3013.0,
    },
    // Bottom scan rule below all Latin.
    SourceFurnitureMask {
        source: 0,
        page: 235,
        left: 0.0 / 1969.0,
        top: 2985.0 / 3013.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Right scan noise connecting body and footer bands.
    SourceFurnitureMask {
        source: 0,
        page: 235,
        left: 1920.0 / 1969.0,
        top: 0.0 / 3013.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Running head above the first Latin line and comparison title.
    SourceFurnitureMask {
        source: 0,
        page: 240,
        left: 550.0 / 1978.0,
        top: 210.0 / 2980.0,
        right: 1260.0 / 1978.0,
        bottom: 270.0 / 2980.0,
    },
    // Detached folio above the first Latin line.
    SourceFurnitureMask {
        source: 0,
        page: 240,
        left: 1630.0 / 1978.0,
        top: 210.0 / 2980.0,
        right: 1730.0 / 1978.0,
        bottom: 275.0 / 2980.0,
    },
    // Running head above the Coriolanus continuation.
    SourceFurnitureMask {
        source: 0,
        page: 284,
        left: 680.0 / 2002.0,
        top: 190.0 / 2988.0,
        right: 1150.0 / 2002.0,
        bottom: 245.0 / 2988.0,
    },
    // Detached folio above the first Latin line.
    SourceFurnitureMask {
        source: 0,
        page: 284,
        left: 1630.0 / 2002.0,
        top: 190.0 / 2988.0,
        right: 1730.0 / 2002.0,
        bottom: 245.0 / 2988.0,
    },
    // Right scan noise joining header and early body bands.
    SourceFurnitureMask {
        source: 0,
        page: 284,
        left: 1970.0 / 2002.0,
        top: 0.0 / 2988.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Local divider and foreign-column fragments beside comparison opening.
    SourceFurnitureMask {
        source: 0,
        page: 56,
        left: 946.0 / 2012.0,
        top: 2445.0 / 2998.0,
        right: 1000.0 / 2012.0,
        bottom: 2630.0 / 2998.0,
    },
    // Google watermark below the final Latin line.
    SourceFurnitureMask {
        source: 0,
        page: 287,
        left: 0.6,
        top: 0.9042017,
        right: 0.8578947,
        bottom: 1.0,
    },
    // Coriolanus running head and folio above the first body line.
    SourceFurnitureMask {
        source: 0,
        page: 288,
        left: 0.0909091,
        top: 0.0687,
        right: 0.8737374,
        bottom: 0.0931635,
    },
    // Camillus running head and folio above the first body line.
    SourceFurnitureMask {
        source: 0,
        page: 182,
        left: 0.08612,
        top: 0.071022,
        right: 0.962513,
        bottom: 0.095142,
    },
    // Coriolanus running head and folio above the first body line.
    SourceFurnitureMask {
        source: 0,
        page: 290,
        left: 0.08612,
        top: 0.070998,
        right: 0.962513,
        bottom: 0.093101,
    },
    // Google watermark below the final Timoleon line.
    SourceFurnitureMask {
        source: 0,
        page: 299,
        left: 0.6,
        top: 0.8775,
        right: 0.8632,
        bottom: 1.0,
    },
    // Greek Coriolanus running head above the chapter XXII body.
    SourceFurnitureMask {
        source: 0,
        page: 281,
        left: 0.085729,
        top: 0.061533,
        right: 0.958932,
        bottom: 0.080699,
    },
    // Timoleon running head and folio above the first body line.
    SourceFurnitureMask {
        source: 0,
        page: 298,
        left: 0.08593,
        top: 0.064386,
        right: 0.962814,
        bottom: 0.084172,
    },
    // Timoleon running head and folio above the first body line.
    SourceFurnitureMask {
        source: 0,
        page: 308,
        left: 0.085586,
        top: 0.064386,
        right: 0.962963,
        bottom: 0.083501,
    },
    // Lysander running head and folio above the first body line.
    SourceFurnitureMask {
        source: 0,
        page: 550,
        left: 0.085859,
        top: 0.068265,
        right: 0.962626,
        bottom: 0.089244,
    },
    // Demosthenes running head above the first body line.
    SourceFurnitureMask {
        source: 1,
        page: 441,
        left: 0.074508,
        top: 0.041549,
        right: 0.904737,
        bottom: 0.062676,
    },
    // Far-right vertical rule beyond the Latin measure.
    SourceFurnitureMask {
        source: 1,
        page: 441,
        left: 0.987759,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Antonius running head and folio above the split-word continuation.
    SourceFurnitureMask {
        source: 1,
        page: 539,
        left: 0.076754,
        top: 0.044397,
        right: 0.926535,
        bottom: 0.067653,
    },
    // Cato running head and folio above the first body line.
    SourceFurnitureMask {
        source: 1,
        page: 337,
        left: 0.084507,
        top: 0.047216,
        right: 0.96208,
        bottom: 0.069063,
    },
    // End-of-Life ornament below the complete Marius ending.
    SourceFurnitureMask {
        source: 0,
        page: 530,
        left: 0.070301,
        top: 0.387735,
        right: 0.609272,
        bottom: 0.40382,
    },
    // Far-right scan edge beyond the retained Latin measure.
    SourceFurnitureMask {
        source: 0,
        page: 281,
        left: 1900.0 / 1948.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Far-right scan edge beyond the retained Latin measure.
    SourceFurnitureMask {
        source: 0,
        page: 287,
        left: 1875.0 / 1900.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Far-right scan edge beyond the retained Latin measure.
    SourceFurnitureMask {
        source: 0,
        page: 296,
        left: 1965.0 / 1990.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Far-right scan edge beyond the retained Latin measure.
    SourceFurnitureMask {
        source: 0,
        page: 298,
        left: 1965.0 / 1990.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Far-right scan edge beyond the retained Latin measure.
    SourceFurnitureMask {
        source: 0,
        page: 299,
        left: 1875.0 / 1900.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Far-right scan edge beyond the retained Latin measure.
    SourceFurnitureMask {
        source: 0,
        page: 304,
        left: 1963.0 / 1988.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Far-right scan edge beyond the retained Latin measure.
    SourceFurnitureMask {
        source: 0,
        page: 306,
        left: 1976.0 / 2001.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Far-right scan edge beyond the retained Latin measure.
    SourceFurnitureMask {
        source: 0,
        page: 308,
        left: 1973.0 / 1998.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Far-right scan edge beyond the retained Latin measure.
    SourceFurnitureMask {
        source: 0,
        page: 470,
        left: 1991.0 / 2016.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Far-right scan edge beyond the retained Latin measure.
    SourceFurnitureMask {
        source: 1,
        page: 361,
        left: 1785.0 / 1810.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Far-right scan edge beyond the retained Latin measure.
    SourceFurnitureMask {
        source: 1,
        page: 539,
        left: 1799.0 / 1824.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Far-right scan edge beyond the retained Latin measure.
    SourceFurnitureMask {
        source: 1,
        page: 625,
        left: 1791.0 / 1816.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Scan-edge rule beyond the reviewed Latin measure.
    SourceFurnitureMask {
        source: 1,
        page: 375,
        left: 1745.0 / 1797.0,
        top: 0.0 / 2836.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Pale watermark below the final Latin line.
    SourceFurnitureMask {
        source: 0,
        page: 321,
        left: 1188.0 / 1893.0,
        top: 2834.0 / 2960.0,
        right: 1595.0 / 1893.0,
        bottom: 2922.0 / 2960.0,
    },
    // Scan-edge rule beyond the reviewed Latin measure.
    SourceFurnitureMask {
        source: 0,
        page: 321,
        left: 1875.0 / 1893.0,
        top: 0.0 / 2960.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Pale watermark below the final Latin line.
    SourceFurnitureMask {
        source: 1,
        page: 603,
        left: 1216.0 / 1834.0,
        top: 2711.0 / 2831.0,
        right: 1595.0 / 1834.0,
        bottom: 2793.0 / 2831.0,
    },
    // Scan-edge rule beyond the reviewed Latin measure.
    SourceFurnitureMask {
        source: 1,
        page: 603,
        left: 1805.0 / 1834.0,
        top: 0.0 / 2831.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Pale watermark below the final Latin line.
    SourceFurnitureMask {
        source: 1,
        page: 623,
        left: 1216.0 / 1810.0,
        top: 2723.0 / 2843.0,
        right: 1595.0 / 1810.0,
        bottom: 2805.0 / 2843.0,
    },
    // Scan-edge rule beyond the reviewed Latin measure.
    SourceFurnitureMask {
        source: 1,
        page: 623,
        left: 1790.0 / 1810.0,
        top: 0.0 / 2843.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Detached folio above the first Latin body line.
    SourceFurnitureMask {
        source: 0,
        page: 448,
        left: 1670.0 / 1988.0,
        top: 256.0 / 3007.0,
        right: 1760.0 / 1988.0,
        bottom: 296.0 / 3007.0,
    },
    // Detached folio above the first Latin body line.
    SourceFurnitureMask {
        source: 0,
        page: 528,
        left: 1640.0 / 1958.0,
        top: 235.0 / 2988.0,
        right: 1740.0 / 1958.0,
        bottom: 286.0 / 2988.0,
    },
    // Detached folio above the first Latin body line.
    SourceFurnitureMask {
        source: 0,
        page: 432,
        left: 1640.0 / 1948.0,
        top: 230.0 / 2980.0,
        right: 1740.0 / 1948.0,
        bottom: 270.0 / 2980.0,
    },
    // Isolated right divider beyond the reviewed Latin body.
    SourceFurnitureMask {
        source: 1,
        page: 391,
        left: 1790.0 / 1825.0,
        top: 0.0 / 2825.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Isolated right divider beyond the reviewed Latin body.
    SourceFurnitureMask {
        source: 1,
        page: 393,
        left: 1850.0 / 1887.0,
        top: 0.0 / 2832.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Isolated right divider beyond the reviewed Latin body.
    SourceFurnitureMask {
        source: 1,
        page: 437,
        left: 1800.0 / 1841.0,
        top: 0.0 / 2838.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Isolated right divider beyond the reviewed Latin body.
    SourceFurnitureMask {
        source: 1,
        page: 443,
        left: 1800.0 / 1846.0,
        top: 0.0 / 2838.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Isolated right divider beyond the reviewed Latin body.
    SourceFurnitureMask {
        source: 1,
        page: 407,
        left: 1840.0 / 1889.0,
        top: 0.0 / 2830.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Isolated right divider beyond the reviewed Latin body.
    SourceFurnitureMask {
        source: 1,
        page: 411,
        left: 1830.0 / 1883.0,
        top: 0.0 / 2830.0,
        right: 1.0,
        bottom: 1.0,
    },
    // Left Greek/Latin divider streaks that merge every Latin row into one band on
    // these pages (Theseus p23/p30, Romulus p41), causing early page breaks and the
    // horizontal split of the genethliacus line. The streak sits in the gutter left
    // of the Latin glyphs, which begin well right of the mask.
    SourceFurnitureMask {
        source: 0,
        page: 23,
        left: 1047.0 / 2007.0,
        top: 0.0,
        right: 1065.0 / 2007.0,
        bottom: 1.0,
    },
    SourceFurnitureMask {
        source: 0,
        page: 30,
        left: 936.0 / 1984.0,
        top: 0.0,
        right: 955.0 / 1984.0,
        bottom: 1.0,
    },
    SourceFurnitureMask {
        source: 0,
        page: 41,
        left: 1037.0 / 1990.0,
        top: 0.0,
        right: 1072.0 / 1990.0,
        bottom: 1.0,
    },
    // Right scan edge joins the Latin rows at the foot of Timoleon p315, splitting
    // "... operam suam nava-/vit ...".
    SourceFurnitureMask {
        source: 0,
        page: 315,
        left: 1880.0 / 1883.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
    // T2a: the right scan edge below the final Coriolanus line on p289 extends the
    // last Latin band ("... exclamans : O ma-", rows 2559-2586) down to row 2728,
    // so the "O ma-" strip carries ~140 px of trailing whitespace that renders as the
    // inflated gap before "ter, ut mecum egisti!" (first line of p290). The streak
    // sits at the extreme right margin, clear of the Latin (which ends near x=1826).
    SourceFurnitureMask {
        source: 0,
        page: 289,
        left: 1904.0 / 1913.0,
        top: 0.0,
        right: 1.0,
        bottom: 1.0,
    },
];

fn apply_source_furniture_mask(mask: &SourceFurnitureMask, img: &mut GrayImage) {
    let coords = [mask.left, mask.top, mask.right, mask.bottom];
    if coords.iter().any(|v| !v.is_finite()) || mask.left >= mask.right || mask.top >= mask.bottom {
        return;
    }
    let x = |v: f64| (v.clamp(0.0, 1.0) * img.width() as f64).round() as u32;
    let y = |v: f64| (v.clamp(0.0, 1.0) * img.height() as f64).round() as u32;
    let (left, right, top, bottom) = (x(mask.left), x(mask.right), y(mask.top), y(mask.bottom));
    for yy in top..bottom {
        for xx in left..right {
            img.put_pixel(xx, yy, Luma([255]));
        }
    }
}

fn apply_raw_source_mask_to_sheared(
    mask: &SourceFurnitureMask,
    shear: &SourceShear,
    img: &mut GrayImage,
) {
    let coords = [mask.left, mask.top, mask.right, mask.bottom];
    if coords.iter().any(|v| !v.is_finite()) || mask.left >= mask.right || mask.top >= mask.bottom {
        return;
    }
    let (width, height) = shear.raw_dims;
    let x = |v: f64| (v.clamp(0.0, 1.0) * f64::from(width)).round() as u32;
    let y = |v: f64| (v.clamp(0.0, 1.0) * f64::from(height)).round() as u32;
    let (left, right, top, bottom) = (x(mask.left), x(mask.right), y(mask.top), y(mask.bottom));
    let (_, top_pad) = source_shear_shifts(width, shear.x0_px, shear.baseline_slope);
    for xx in left..right {
        let offset = source_shear_shift(xx, shear.x0_px, shear.baseline_slope) + i64::from(top_pad);
        for yy in top..bottom {
            let dst_y =
                u32::try_from(i64::from(yy) + offset).expect("masked row inside shear padding");
            img.put_pixel(xx, dst_y, Luma([255]));
        }
    }
}

fn mask_known_source_furniture(source: usize, page: u32, img: &mut GrayImage) {
    let shear = SOURCE_SHEARS
        .iter()
        .find(|entry| entry.source == source && entry.page == page)
        .filter(|shear| {
            let (shifts, top_pad) =
                source_shear_shifts(shear.raw_dims.0, shear.x0_px, shear.baseline_slope);
            let bottom_pad = u32::try_from(shifts.into_iter().max().unwrap_or(0).max(0)).unwrap();
            img.dimensions() == (shear.raw_dims.0, shear.raw_dims.1 + top_pad + bottom_pad)
        });
    for mask in SOURCE_FURNITURE_MASKS
        .iter()
        .filter(|m| m.source == source && m.page == page)
    {
        if let Some(shear) = shear {
            apply_raw_source_mask_to_sheared(mask, shear, img);
        } else {
            apply_source_furniture_mask(mask, img);
        }
    }
    if shear.is_some() {
        for mask in SOURCE_SHEARED_FURNITURE_MASKS
            .iter()
            .filter(|m| m.source == source && m.page == page)
        {
            apply_source_furniture_mask(mask, img);
        }
    }
}

/// Verified bleed-through / compaction regions in normalized, half-open full-source
/// row coordinates. Unlike a furniture mask (which only whitens ink), an excision
/// deletes the rows entirely and shifts the rows below up, closing the vertical
/// space the region occupied. Records for one `(source, page)` must be sorted
/// ascending and non-overlapping.
#[derive(Clone, Copy)]
struct SourceRowExcision {
    source: usize,
    page: u32,
    top: f64,
    bottom: f64,
}

const SOURCE_ROW_EXCISIONS: &[SourceRowExcision] = &[
    // Romulus II->III: remove the blank parallel-column alignment space between
    // the preceding Latin line and "III. Ceterum" after the p34 shear. The body
    // bands end at row 1878 and resume at 2149; this leaves 14 blank rows on
    // either side of the splice and preserves every Latin pixel.
    SourceRowExcision {
        source: 0,
        page: 34,
        top: 1892.0 / 3040.0,
        bottom: 2135.0 / 3040.0,
    },
    // Theseus §17/§18: five faint bled-through lines stand alone between the line
    // ending "... tradit agi." and "XVIII. Sorte ducta ...". Delete the bleed-only
    // slab and close the space, leaving one ordinary inter-line gap. On the 789×2975
    // production crop the real text ends at row 2008 and resumes at row 2240; the
    // intervening bands are all faint bleed (ink/row ≤ 58 vs ≥ 250 for real lines).
    SourceRowExcision {
        source: 0,
        page: 21,
        top: 2012.0 / 2975.0,
        bottom: 2237.0 / 2975.0,
    },
    // Romulus VI→VII: the Latin column finishes chapter VI ("... Hinc adolescentibus
    // magnum nomen.") well above where the parallel Greek column reaches "VII.", so the
    // Latin sits blank until the next chapter aligns. That alignment whitespace is
    // meaningless in a Latin-only reader. Delete the ink-free slab (max ink 2-3 per row
    // on the 2978-row raster; real text ends at row 2408 and resumes at 2602), leaving
    // one ordinary inter-line gap.
    SourceRowExcision {
        source: 0,
        page: 36,
        top: 2414.0 / 2978.0,
        bottom: 2596.0 / 2978.0,
    },
    // Romulus VII→VIII: the same parallel-alignment blank between "... remotis arbitris
    // de his ageret." (ends row 2298) and "VIII. At Faustulus ..." (resumes row 2496) on
    // the 2990-row raster; the slab is ink-free. Close it to one inter-line gap.
    SourceRowExcision {
        source: 0,
        page: 37,
        top: 2304.0 / 2990.0,
        bottom: 2490.0 / 2990.0,
    },
];

/// The original-image row interval `[top, bottom)` an excision deletes, or `None`
/// for a non-finite, empty, or out-of-range record.
fn excision_rows(excision: &SourceRowExcision, height: u32) -> Option<(u32, u32)> {
    if !excision.top.is_finite() || !excision.bottom.is_finite() || excision.top >= excision.bottom
    {
        return None;
    }
    let row = |v: f64| (v.clamp(0.0, 1.0) * height as f64).round() as u32;
    let (top, bottom) = (
        row(excision.top).min(height),
        row(excision.bottom).min(height),
    );
    (top < bottom).then_some((top, bottom))
}

/// Copy every row of `img` except those in the sorted, non-overlapping half-open
/// `deleted` intervals (original-image coordinates) into a new image, shifting the
/// surviving rows below each deletion up. An empty deletion set is a no-op clone.
fn splice_out_rows(img: &GrayImage, deleted: &[(u32, u32)]) -> GrayImage {
    let (w, h) = img.dimensions();
    let total: u32 = deleted
        .iter()
        .map(|&(t, b)| b.min(h).saturating_sub(t.min(h)))
        .sum();
    if total == 0 {
        return img.clone();
    }
    let mut out = GrayImage::from_pixel(w, h - total, Luma([255]));
    let mut src = 0u32;
    let mut dst = 0u32;
    for &(t, b) in deleted {
        let (t, b) = (t.min(h), b.min(h));
        if t > src {
            let span = t - src;
            let keep = image::imageops::crop_imm(img, 0, src, w, span).to_image();
            image::imageops::overlay(&mut out, &keep, 0, dst as i64);
            dst += span;
        }
        src = src.max(b);
    }
    if src < h {
        let keep = image::imageops::crop_imm(img, 0, src, w, h - src).to_image();
        image::imageops::overlay(&mut out, &keep, 0, dst as i64);
    }
    out
}

/// Delete one excision's rows from `img`, returning a new, shorter image (or an
/// unchanged clone for a degenerate record). Used by the synthetic primitive tests;
/// production dispatches through [`excise_known_source_rows`].
#[cfg(test)]
fn apply_source_row_excision(excision: &SourceRowExcision, img: &GrayImage) -> GrayImage {
    match excision_rows(excision, img.height()) {
        Some(rows) => splice_out_rows(img, &[rows]),
        None => img.clone(),
    }
}

/// Delete every configured bleed-through slab for `(source, page)` from `img` in one
/// pass, closing the vertical space it occupied. Intervals are converted against the
/// original height and must be disjoint; with no record the image is returned as-is.
fn excise_known_source_rows(source: usize, page: u32, img: GrayImage) -> GrayImage {
    let height = img.height();
    let mut intervals: Vec<(u32, u32)> = SOURCE_ROW_EXCISIONS
        .iter()
        .filter(|e| e.source == source && e.page == page)
        .filter_map(|e| excision_rows(e, height))
        .collect();
    if intervals.is_empty() {
        return img;
    }
    intervals.sort_unstable();
    splice_out_rows(&img, &intervals)
}

/// A reviewed per-column shear that straightens a uniformly tilted source scan
/// before masking and banding. The scan was photographed at a slight angle, so its
/// baselines are straight lines with a small constant slope; shifting each column
/// vertically by an integer number of rows makes them horizontal again. Because
/// every pixel moves a whole number of rows **within its own column**, the glyph
/// pixel values are translated, never interpolated or resampled. The shear spans the
/// full page width (Greek column, divider, and Latin column together) under the
/// premise that the defect is a single uniform rotation of the whole scan, so one
/// slope straightens every baseline.
struct SourceShear {
    source: usize,
    page: u32,
    raw_dims: (u32, u32),
    fingerprint_region: [u32; 4],
    fingerprint: &'static str,
    /// Full-page raw x the shift pivots around; its exact value only slides the
    /// straightened raster by a constant absorbed by `top_pad`.
    x0_px: u32,
    /// Measured change in baseline y per x pixel. The applied shift negates it.
    baseline_slope: f64,
}

impl SourceShear {
    /// A raw page matches when its dimensions and distinctive raw pixels are the
    /// reviewed ones. The fingerprint is checked on the untransformed image, so a
    /// same-sized but changed render does not trigger the shear.
    fn matches(&self, raw: &GrayImage) -> bool {
        self.raw_dims == raw.dimensions()
            && matches_source_region_fingerprint(raw, self.fingerprint_region, self.fingerprint)
    }
}

const SOURCE_SHEARS: &[SourceShear] = &[
    SourceShear {
        source: 0,
        page: 34,
        raw_dims: (1982, 3008),
        fingerprint_region: [1100, 600, 1700, 750],
        fingerprint: "6a72bab3264a4a88ccdb988d4d138a383f7dcf140366be63192fd99740a3f204",
        x0_px: 1400,
        baseline_slope: -0.016079632465543646,
    },
    SourceShear {
        source: 0,
        page: 448,
        raw_dims: (1988, 3007),
        fingerprint_region: [1100, 600, 1700, 750],
        fingerprint: "852199dd3ab488b25f83a7d74e3beeb99d03a8148961068de1ccba083b477ccf",
        x0_px: 1400,
        baseline_slope: 0.03048780487804878,
    },
    SourceShear {
        source: 1,
        page: 266,
        raw_dims: (1878, 2825),
        fingerprint_region: [1050, 600, 1650, 750],
        fingerprint: "fcf1065d83486979a6e6f9d4ddbcf50f0e20b107cdd7aafa48f507246edc5d2c",
        x0_px: 1350,
        baseline_slope: 0.03745928338762215,
    },
    SourceShear {
        source: 1,
        page: 398,
        raw_dims: (1840, 2835),
        fingerprint_region: [1050, 600, 1650, 750],
        fingerprint: "680397c87b37fb70c0a520f82755d86b0a186a93b1d201a303c7bbc3281cd660",
        x0_px: 1350,
        baseline_slope: 0.03175895765472313,
    },
];

/// The integer vertical shift applied to raw column `x` for a shear centered at
/// `x0_px` with measured `baseline_slope`. The shift negates the baseline's rise so
/// a baseline that descends to the right is lifted back level.
fn source_shear_shift(x: u32, x0_px: u32, baseline_slope: f64) -> i64 {
    (-((f64::from(x) - f64::from(x0_px)) * baseline_slope)).round() as i64
}

/// Every column's signed shift plus the top padding (`-min_shift`, with zero
/// included in the minimum) that keeps all shifted pixels at non-negative rows.
fn source_shear_shifts(width: u32, x0_px: u32, baseline_slope: f64) -> (Vec<i64>, u32) {
    let shifts: Vec<i64> = (0..width)
        .map(|x| source_shear_shift(x, x0_px, baseline_slope))
        .collect();
    let min_shift = shifts.iter().copied().min().unwrap_or(0).min(0);
    let top_pad = u32::try_from(-min_shift).expect("non-negative top padding");
    (shifts, top_pad)
}

/// Straighten a uniformly tilted scan by shifting each column vertically by an
/// integer number of rows. The destination is padded top and bottom so no pixel is
/// clipped: its height is `raw_height + max_shift - min_shift` (zero included in
/// both bounds), and every raw pixel `(x, y)` lands once at
/// `(x, y + shift(x) + top_pad)`. Vacated cells stay white, so each output column
/// holds the input column's exact pixels plus white padding.
fn shear_image(raw: &GrayImage, x0_px: u32, baseline_slope: f64) -> GrayImage {
    let (width, height) = raw.dimensions();
    let (shifts, top_pad) = source_shear_shifts(width, x0_px, baseline_slope);
    let bottom_pad = u32::try_from(shifts.iter().copied().max().unwrap_or(0).max(0))
        .expect("non-negative bottom padding");
    let out_height = height
        .checked_add(top_pad)
        .and_then(|h| h.checked_add(bottom_pad))
        .expect("sheared height fits u32");
    let mut out = GrayImage::from_pixel(width, out_height, Luma([255]));
    for x in 0..width {
        let shift = shifts[x as usize];
        for y in 0..height {
            let dst = i64::from(y) + shift + i64::from(top_pad);
            let dst = u32::try_from(dst).expect("shifted row within padded height");
            out.put_pixel(x, dst, *raw.get_pixel(x, y));
        }
    }
    out
}

/// Straighten a reviewed tilted page before any other source transform. Returns the
/// input untouched unless a `SOURCE_SHEARS` entry matches source, page, raw
/// dimensions, and the raw-pixel fingerprint.
fn apply_known_source_shear(source: usize, page: u32, raw: GrayImage) -> GrayImage {
    match SOURCE_SHEARS
        .iter()
        .find(|s| s.source == source && s.page == page && s.matches(&raw))
    {
        Some(shear) => shear_image(&raw, shear.x0_px, shear.baseline_slope),
        None => raw,
    }
}

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

/// Exclude reviewed whole-page repetitions within their owning unit.
fn skip_known_duplicate_source_page(unit_id: &str, source: usize, page: u32) -> bool {
    matches!(
        (unit_id, source, page),
        ("comparatio_sertorii_et_eumenis", 1, 103 | 104)
            | ("agis", 1, 369 | 370 | 377 | 378)
            | ("cato_minor", 1, 339..=348 | 361 | 362)
            | ("agesilaus", 1, 119..=122)
            | ("alexander", 1, 213 | 214)
            | ("phocion", 1, 297 | 298)
            | ("cicero", 1, 465 | 466)
            | ("antonius", 1, 523 | 524)
            | ("aratus", 1, 673 | 674)
            | ("otho", 1, 701 | 702)
    )
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
        if skip_known_duplicate_source_page(&range.unit.id, source, page) {
            continue;
        }
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
        let observed = bands.clone();
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
        *bands = retain_known_source_body_bands(
            Some(&range.unit.id),
            source,
            page,
            col.dimensions(),
            &observed,
            bands,
        );
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
    let mut page_img = apply_known_source_shear(source, page, page_img);
    mask_known_source_furniture(source, page, &mut page_img);
    let page_img = excise_known_source_rows(source, page, page_img);
    let column = production_source_column(source, page, &page_img)?;
    let cropped = crop_column(&page_img, &column);
    let (raw, bands) = band_source_column(source, page, &cropped);
    let bands =
        retain_known_source_body_bands(None, source, page, cropped.dimensions(), &raw, &bands);
    Ok((cropped, bands))
}

/// Column bounds on a raster after edition-specific furniture masking.
fn production_source_column(source: usize, page: u32, masked: &GrayImage) -> Result<Column> {
    #[cfg(test)]
    if source == 1 && page == 562 && masked.dimensions() == (1850, 2835) {
        if let Some(column) = AUDIT_ANTONIUS_CROP_OVERRIDE.with(|slot| slot.get()) {
            return Ok(column);
        }
    }
    Ok(preserve_known_body_crop_edge(
        source,
        page,
        masked,
        detect_latin_column(masked)?,
    ))
}

/// Raw line evidence and furniture-filtered bands on an owned Latin column.
fn band_source_column(source: usize, page: u32, cropped: &GrayImage) -> (Vec<Band>, Vec<Band>) {
    let reviewed_text_window = match (source, page, cropped.dimensions()) {
        (0, 34, (828, 2797))
            if matches_source_region_fingerprint(
                cropped,
                [100, 600, 600, 750],
                "b178043dee776247891c4b54e8ad38b3a12e4b91a9b2619e51cebf5d4fde92e5",
            ) =>
        {
            Some((60, 798))
        }
        (0, 448, (1079, 3068))
            if matches_source_region_fingerprint(
                cropped,
                [100, 600, 600, 750],
                "d8fae0659e8f02b3d9204b2335552d853757608cced3da72755b5f5a6c9f2e4d",
            ) =>
        {
            Some((60, 1049))
        }
        _ => None,
    };
    let raw = if let Some((left, right)) = reviewed_text_window {
        let text_view =
            image::imageops::crop_imm(cropped, left, 0, right - left, cropped.height()).to_image();
        line_bands(&text_view, LINE_MIN_INK)
    } else if source == 0
        && page == 17
        && cropped.dimensions() == (840, 2975)
        && matches_source_region_fingerprint(
            cropped,
            [0, 672, 40, 2651],
            "cca64e5b02b7f0aba9fc096b1856c223e13da07ab18e3725058a809052f6eb59",
        )
    {
        let text_view = image::imageops::crop_imm(cropped, 40, 0, 800, cropped.height()).to_image();
        line_bands(&text_view, LINE_MIN_INK)
    } else {
        line_bands(cropped, LINE_MIN_INK)
    };
    let bands = coalesce_known_body_fragments(source, page, cropped.dimensions(), &raw);
    let kept = drop_furniture(&bands, cropped.height());
    let kept = retain_early_aligned_body_bands(cropped, &bands, &kept);
    (raw, kept)
}

/// Preserve reviewed edge glyphs for exact production-raster detector signatures.
fn preserve_known_body_crop_edge(
    source: usize,
    page: u32,
    raster: &GrayImage,
    column: Column,
) -> Column {
    match (source, page, raster.dimensions(), column.left, column.right) {
        (0, 34, (1982, 2797), 950, 1774)
            if matches_source_region_fingerprint(
                raster,
                [1045, 600, 1545, 750],
                "b178043dee776247891c4b54e8ad38b3a12e4b91a9b2619e51cebf5d4fde92e5",
            ) =>
        {
            Column {
                left: 945,
                right: 1773,
            }
        }
        (0, 448, (1988, 3068), 915, 1988)
            if matches_source_region_fingerprint(
                raster,
                [1009, 600, 1509, 750],
                "d8fae0659e8f02b3d9204b2335552d853757608cced3da72755b5f5a6c9f2e4d",
            ) =>
        {
            Column {
                left: 909,
                right: 1988,
            }
        }
        (1, 266, (1878, 2896), 880 | 881, 1708)
            if matches_source_region_fingerprint(
                raster,
                [980, 600, 1480, 750],
                "97191be5be83c45fd48d15fbedff911430d8ea5fd8a1a0d66cd1243edc54df3d",
            ) =>
        {
            Column {
                left: 880,
                right: 1730,
            }
        }
        (0, 432, (1948, 2980), 907, 1710) => Column {
            left: 907,
            right: 1711,
        },
        (1, 623, (1810, 2843), 982, 1771) => Column {
            left: 976,
            right: 1771,
        },
        (0, 38, (1988, 3000), 929, 1763) => Column {
            left: 932,
            right: 1774,
        },
        (0, 39, (2004, 2989), 1101, 1892) => Column {
            left: 1085,
            right: 1892,
        },
        (1, 252, (1874, 2831), 891, 1706) => Column {
            left: 891,
            right: 1722,
        },
        (0, 326, (1966, 2982), 922, 1712) => Column {
            left: 921,
            right: 1712,
        },
        (0, 535, (1978, 2969), 1049, 1845) => Column {
            left: 1045,
            right: 1845,
        },
        (0, 377, (1952, 2957), 1048, 1856) => Column {
            left: 1048,
            right: 1857,
        },
        (0, 318, (1937, 2988), 900, 1691) => Column {
            left: 899,
            right: 1691,
        },
        (1, 352, (1830, 2842), 879, 1683) => Column {
            left: 876,
            right: 1683,
        },
        (1, 566, (1813, 2838), 892, 1697) => Column {
            left: 888,
            right: 1697,
        },
        (1, 580, (1816, 2835), 892, 1704) => Column {
            left: 889,
            right: 1704,
        },
        (1, 619, (1822, 2824), 1017, 1822) => Column {
            left: 1014,
            right: 1822,
        },
        (1, 445, (1844, 2842), 1017, 1844) => Column {
            left: 1015,
            right: 1844,
        },
        (1, 562, (1850, 2835), 906, 1721) => Column {
            left: 876,
            right: 1721,
        },
        _ => column,
    }
}

struct SourceBodyBand {
    source: usize,
    page: u32,
    unit_id: &'static str,
    dimensions: (u32, u32),
    observed: Band,
    protected: Band,
}

#[cfg(test)]
thread_local! {
    static AUDIT_EXTRA_BODY_BANDS: std::cell::Cell<&'static [SourceBodyBand]> = const { std::cell::Cell::new(&[]) };
    static AUDIT_DISABLE_ANTONIUS_WHOLE_STRIP: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static AUDIT_ANTONIUS_CROP_OVERRIDE: std::cell::Cell<Option<Column>> = const { std::cell::Cell::new(None) };
}

const SOURCE_BODY_BANDS: &[SourceBodyBand] = &[
    SourceBodyBand {
        source: 1,
        page: 110,
        unit_id: "agesilaus",
        dimensions: (798, 2834),
        observed: Band {
            top: 211,
            bottom: 248,
        },
        protected: Band {
            top: 211,
            bottom: 249,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 251,
        unit_id: "alcibiades",
        dimensions: (795, 2975),
        observed: Band {
            top: 2556,
            bottom: 2588,
        },
        protected: Band {
            top: 2556,
            bottom: 2589,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 263,
        unit_id: "alcibiades",
        dimensions: (799, 2989),
        observed: Band {
            top: 2578,
            bottom: 2607,
        },
        protected: Band {
            top: 2578,
            bottom: 2607,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 231,
        unit_id: "alexander",
        dimensions: (810, 2844),
        observed: Band {
            top: 208,
            bottom: 247,
        },
        protected: Band {
            top: 207,
            bottom: 247,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 623,
        unit_id: "brutus",
        dimensions: (795, 2843),
        observed: Band {
            top: 222,
            bottom: 254,
        },
        protected: Band {
            top: 221,
            bottom: 254,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 623,
        unit_id: "brutus",
        dimensions: (795, 2843),
        observed: Band {
            top: 266,
            bottom: 297,
        },
        protected: Band {
            top: 266,
            bottom: 297,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 623,
        unit_id: "brutus",
        dimensions: (795, 2843),
        observed: Band {
            top: 311,
            bottom: 341,
        },
        protected: Band {
            top: 310,
            bottom: 344,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 624,
        unit_id: "brutus",
        dimensions: (786, 2830),
        observed: Band {
            top: 200,
            bottom: 231,
        },
        protected: Band {
            top: 199,
            bottom: 231,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 327,
        unit_id: "cato_minor",
        dimensions: (795, 2834),
        observed: Band {
            top: 2509,
            bottom: 2540,
        },
        protected: Band {
            top: 2509,
            bottom: 2540,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 331,
        unit_id: "cato_minor",
        dimensions: (792, 2838),
        observed: Band {
            top: 170,
            bottom: 207,
        },
        protected: Band {
            top: 170,
            bottom: 208,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 391,
        unit_id: "cleomenes",
        dimensions: (782, 2825),
        observed: Band {
            top: 196,
            bottom: 230,
        },
        protected: Band {
            top: 196,
            bottom: 230,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 393,
        unit_id: "cleomenes",
        dimensions: (789, 2832),
        observed: Band {
            top: 208,
            bottom: 244,
        },
        protected: Band {
            top: 207,
            bottom: 245,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 67,
        unit_id: "crassus",
        dimensions: (795, 2830),
        observed: Band {
            top: 216,
            bottom: 249,
        },
        protected: Band {
            top: 216,
            bottom: 251,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 437,
        unit_id: "demosthenes",
        dimensions: (788, 2838),
        observed: Band {
            top: 203,
            bottom: 235,
        },
        protected: Band {
            top: 203,
            bottom: 236,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 437,
        unit_id: "demosthenes",
        dimensions: (788, 2838),
        observed: Band {
            top: 252,
            bottom: 284,
        },
        protected: Band {
            top: 252,
            bottom: 284,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 437,
        unit_id: "demosthenes",
        dimensions: (788, 2838),
        observed: Band {
            top: 299,
            bottom: 331,
        },
        protected: Band {
            top: 299,
            bottom: 331,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 437,
        unit_id: "demosthenes",
        dimensions: (788, 2838),
        observed: Band {
            top: 348,
            bottom: 378,
        },
        protected: Band {
            top: 347,
            bottom: 378,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 438,
        unit_id: "demosthenes",
        dimensions: (794, 2844),
        observed: Band {
            top: 2520,
            bottom: 2553,
        },
        protected: Band {
            top: 2520,
            bottom: 2556,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 443,
        unit_id: "demosthenes",
        dimensions: (807, 2838),
        observed: Band {
            top: 197,
            bottom: 229,
        },
        protected: Band {
            top: 196,
            bottom: 229,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 443,
        unit_id: "demosthenes",
        dimensions: (807, 2838),
        observed: Band {
            top: 246,
            bottom: 278,
        },
        protected: Band {
            top: 246,
            bottom: 278,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 577,
        unit_id: "dio",
        dimensions: (792, 2843),
        observed: Band {
            top: 199,
            bottom: 230,
        },
        protected: Band {
            top: 199,
            bottom: 230,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 579,
        unit_id: "dio",
        dimensions: (798, 2832),
        observed: Band {
            top: 210,
            bottom: 237,
        },
        protected: Band {
            top: 210,
            bottom: 237,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 91,
        unit_id: "eumenes",
        dimensions: (789, 2836),
        observed: Band {
            top: 208,
            bottom: 239,
        },
        protected: Band {
            top: 207,
            bottom: 239,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 91,
        unit_id: "eumenes",
        dimensions: (789, 2836),
        observed: Band {
            top: 259,
            bottom: 289,
        },
        protected: Band {
            top: 259,
            bottom: 290,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 228,
        unit_id: "fabius_maximus",
        dimensions: (822, 3004),
        observed: Band {
            top: 262,
            bottom: 297,
        },
        protected: Band {
            top: 261,
            bottom: 297,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 252,
        unit_id: "iulius_caesar",
        dimensions: (831, 2831),
        observed: Band {
            top: 240,
            bottom: 744,
        },
        protected: Band {
            top: 239,
            bottom: 745,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 607,
        unit_id: "lucullus",
        dimensions: (825, 2982),
        observed: Band {
            top: 248,
            bottom: 287,
        },
        protected: Band {
            top: 247,
            bottom: 288,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 79,
        unit_id: "lycurgus",
        dimensions: (810, 3005),
        observed: Band {
            top: 280,
            bottom: 313,
        },
        protected: Band {
            top: 280,
            bottom: 313,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 292,
        unit_id: "marcius_coriolanus",
        dimensions: (1035, 2994),
        observed: Band {
            top: 280,
            bottom: 312,
        },
        protected: Band {
            top: 280,
            bottom: 312,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 292,
        unit_id: "marcius_coriolanus",
        dimensions: (1035, 2994),
        observed: Band {
            top: 335,
            bottom: 368,
        },
        protected: Band {
            top: 334,
            bottom: 369,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 25,
        unit_id: "nicias",
        dimensions: (812, 2832),
        observed: Band {
            top: 2508,
            bottom: 2544,
        },
        protected: Band {
            top: 2508,
            bottom: 2544,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 100,
        unit_id: "numa",
        dimensions: (798, 2976),
        observed: Band {
            top: 260,
            bottom: 293,
        },
        protected: Band {
            top: 260,
            bottom: 293,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 200,
        unit_id: "pericles",
        dimensions: (793, 2986),
        observed: Band {
            top: 280,
            bottom: 312,
        },
        protected: Band {
            top: 280,
            bottom: 312,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 448,
        unit_id: "philopoemen",
        dimensions: (1079, 3068),
        observed: Band {
            top: 330,
            bottom: 366,
        },
        protected: Band {
            top: 330,
            bottom: 366,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 294,
        unit_id: "phocion",
        dimensions: (794, 2839),
        observed: Band {
            top: 210,
            bottom: 241,
        },
        protected: Band {
            top: 210,
            bottom: 241,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 299,
        unit_id: "phocion",
        dimensions: (791, 2832),
        observed: Band {
            top: 178,
            bottom: 209,
        },
        protected: Band {
            top: 177,
            bottom: 210,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 301,
        unit_id: "phocion",
        dimensions: (787, 2836),
        observed: Band {
            top: 192,
            bottom: 220,
        },
        protected: Band {
            top: 192,
            bottom: 220,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 303,
        unit_id: "phocion",
        dimensions: (860, 2836),
        observed: Band {
            top: 204,
            bottom: 235,
        },
        protected: Band {
            top: 203,
            bottom: 235,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 475,
        unit_id: "pyrrhus",
        dimensions: (795, 2962),
        observed: Band {
            top: 2556,
            bottom: 2589,
        },
        protected: Band {
            top: 2555,
            bottom: 2590,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 36,
        unit_id: "romulus",
        dimensions: (795, 2978),
        observed: Band {
            top: 2602,
            bottom: 2633,
        },
        protected: Band {
            top: 2601,
            bottom: 2633,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 38,
        unit_id: "romulus",
        dimensions: (842, 3000),
        observed: Band {
            top: 2451,
            bottom: 2651,
        },
        protected: Band {
            top: 2451,
            bottom: 2651,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 108,
        unit_id: "solon",
        dimensions: (796, 2975),
        observed: Band {
            top: 2554,
            bottom: 2591,
        },
        protected: Band {
            top: 2553,
            bottom: 2592,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 115,
        unit_id: "solon",
        dimensions: (986, 3020),
        observed: Band {
            top: 240,
            bottom: 325,
        },
        protected: Band {
            top: 240,
            bottom: 326,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 15,
        unit_id: "theseus",
        dimensions: (801, 2973),
        observed: Band {
            top: 2368,
            bottom: 2630,
        },
        protected: Band {
            top: 2367,
            bottom: 2632,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 25,
        unit_id: "theseus",
        dimensions: (793, 2986),
        observed: Band {
            top: 2592,
            bottom: 2622,
        },
        protected: Band {
            top: 2592,
            bottom: 2622,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 407,
        unit_id: "tiberius_gracchus",
        dimensions: (796, 2830),
        observed: Band {
            top: 186,
            bottom: 218,
        },
        protected: Band {
            top: 186,
            bottom: 218,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 411,
        unit_id: "tiberius_gracchus",
        dimensions: (785, 2830),
        observed: Band {
            top: 179,
            bottom: 211,
        },
        protected: Band {
            top: 179,
            bottom: 211,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 414,
        unit_id: "tiberius_gracchus",
        dimensions: (794, 2844),
        observed: Band {
            top: 247,
            bottom: 320,
        },
        protected: Band {
            top: 247,
            bottom: 320,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 326,
        unit_id: "aemilius_paulus",
        dimensions: (791, 2982),
        observed: Band {
            top: 273,
            bottom: 304,
        },
        protected: Band {
            top: 273,
            bottom: 304,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 519,
        unit_id: "caius_marius",
        dimensions: (818, 2973),
        observed: Band {
            top: 246,
            bottom: 292,
        },
        protected: Band {
            top: 246,
            bottom: 292,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 44,
        unit_id: "crassus",
        dimensions: (789, 2822),
        observed: Band {
            top: 191,
            bottom: 224,
        },
        protected: Band {
            top: 191,
            bottom: 224,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 631,
        unit_id: "lucullus",
        dimensions: (795, 2978),
        observed: Band {
            top: 252,
            bottom: 287,
        },
        protected: Band {
            top: 252,
            bottom: 287,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 535,
        unit_id: "lysander",
        dimensions: (800, 2969),
        observed: Band {
            top: 245,
            bottom: 278,
        },
        protected: Band {
            top: 245,
            bottom: 278,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 535,
        unit_id: "lysander",
        dimensions: (800, 2969),
        observed: Band {
            top: 306,
            bottom: 339,
        },
        protected: Band {
            top: 306,
            bottom: 339,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 377,
        unit_id: "marcellus",
        dimensions: (809, 2957),
        observed: Band {
            top: 249,
            bottom: 282,
        },
        protected: Band {
            top: 249,
            bottom: 282,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 25,
        unit_id: "nicias",
        dimensions: (812, 2832),
        observed: Band {
            top: 204,
            bottom: 247,
        },
        protected: Band {
            top: 204,
            bottom: 247,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 318,
        unit_id: "timoleon",
        dimensions: (792, 2988),
        observed: Band {
            top: 263,
            bottom: 295,
        },
        protected: Band {
            top: 263,
            bottom: 295,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 110,
        unit_id: "agesilaus",
        dimensions: (798, 2834),
        observed: Band {
            top: 2519,
            bottom: 2558,
        },
        protected: Band {
            top: 2519,
            bottom: 2558,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 633,
        unit_id: "artoxerxes",
        dimensions: (786, 2826),
        observed: Band {
            top: 2522,
            bottom: 2554,
        },
        protected: Band {
            top: 2522,
            bottom: 2554,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 352,
        unit_id: "cato_minor",
        dimensions: (807, 2842),
        observed: Band {
            top: 2526,
            bottom: 2564,
        },
        protected: Band {
            top: 2526,
            bottom: 2564,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 566,
        unit_id: "dio",
        dimensions: (809, 2838),
        observed: Band {
            top: 2540,
            bottom: 2577,
        },
        protected: Band {
            top: 2540,
            bottom: 2577,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 580,
        unit_id: "dio",
        dimensions: (815, 2835),
        observed: Band {
            top: 2550,
            bottom: 2597,
        },
        protected: Band {
            top: 2550,
            bottom: 2597,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 242,
        unit_id: "alcibiades",
        dimensions: (787, 2981),
        observed: Band {
            top: 2598,
            bottom: 2623,
        },
        protected: Band {
            top: 2598,
            bottom: 2623,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 214,
        unit_id: "pericles",
        dimensions: (802, 2988),
        observed: Band {
            top: 2599,
            bottom: 2628,
        },
        protected: Band {
            top: 2599,
            bottom: 2628,
        },
    },
    SourceBodyBand {
        source: 0,
        page: 215,
        unit_id: "pericles",
        dimensions: (796, 2974),
        observed: Band {
            top: 249,
            bottom: 272,
        },
        protected: Band {
            top: 249,
            bottom: 272,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 619,
        unit_id: "brutus",
        dimensions: (808, 2824),
        observed: Band {
            top: 120,
            bottom: 225,
        },
        protected: Band {
            top: 193,
            bottom: 225,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 329,
        unit_id: "cato_minor",
        dimensions: (839, 2831),
        observed: Band {
            top: 40,
            bottom: 210,
        },
        protected: Band {
            top: 180,
            bottom: 210,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 445,
        unit_id: "demosthenes",
        dimensions: (829, 2842),
        observed: Band {
            top: 44,
            bottom: 226,
        },
        protected: Band {
            top: 197,
            bottom: 226,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 291,
        unit_id: "phocion",
        dimensions: (873, 2836),
        observed: Band {
            top: 173,
            bottom: 239,
        },
        protected: Band {
            top: 206,
            bottom: 239,
        },
    },
    SourceBodyBand {
        source: 1,
        page: 562,
        unit_id: "antonius",
        dimensions: (845, 2835),
        observed: Band {
            top: 249,
            bottom: 1356,
        },
        protected: Band {
            top: 249,
            bottom: 1356,
        },
    },
];

/// Keep verified source body intervals when their exact band signature survives.
/// Pass B requires the owning unit so neighboring shared-page Latin stays excluded.
fn retain_known_source_body_bands(
    owner: Option<&str>,
    source: usize,
    page: u32,
    dimensions: (u32, u32),
    observed: &[Band],
    kept: &[Band],
) -> Vec<Band> {
    let mut result = kept.to_vec();
    let mut matched = false;
    let records = SOURCE_BODY_BANDS.iter();
    #[cfg(test)]
    let extra = AUDIT_EXTRA_BODY_BANDS.with(|slot| slot.get());
    #[cfg(test)]
    let records = records.chain(extra.iter());
    for record in records.filter(|r| {
        r.source == source
            && r.page == page
            && r.dimensions == dimensions
            && owner.is_none_or(|id| id == r.unit_id)
    }) {
        if observed
            .iter()
            .any(|b| *b == record.observed || *b == record.protected)
        {
            result.push(record.protected);
            matched = true;
        }
    }
    if !matched {
        return result;
    }
    merge_source_body_bands(result)
}

fn merge_source_body_bands(mut result: Vec<Band>) -> Vec<Band> {
    result.sort_by_key(|b| b.top);
    let mut merged: Vec<Band> = Vec::new();
    for band in result {
        if let Some(last) = merged.last_mut().filter(|last| band.top <= last.bottom) {
            last.bottom = last.bottom.max(band.bottom);
        } else {
            merged.push(band);
        }
    }
    merged
}

/// Join exact adjacent body fragments without changing their source pixels.
fn coalesce_known_body_fragments(
    source: usize,
    page: u32,
    dimensions: (u32, u32),
    bands: &[Band],
) -> Vec<Band> {
    let joins: &[([u32; 2], [u32; 2], [u32; 2])] = match (source, page, dimensions) {
        (0, 321, (764, 2960)) => &[([2529, 2533], [2534, 2572], [2528, 2572])],
        (1, 375, (752, 2836)) => &[
            ([2209, 2245], [2247, 2251], [2209, 2251]),
            ([2315, 2319], [2321, 2357], [2315, 2357]),
        ],
        (1, 391, (782, 2825)) => &[([1192, 1195], [1196, 1231], [1192, 1231])],
        (1, 393, (789, 2832)) => &[([928, 959], [961, 962], [928, 962])],
        (1, 443, (807, 2838)) => &[([2153, 2179], [2180, 2185], [2153, 2185])],
        _ => &[],
    };
    let mut result = Vec::new();
    let mut i = 0;
    while i < bands.len() {
        let joined = joins.iter().find(|(a, b, _)| {
            i + 1 < bands.len()
                && [bands[i].top, bands[i].bottom] == *a
                && [bands[i + 1].top, bands[i + 1].bottom] == *b
        });
        if let Some((_, _, [top, bottom])) = joined {
            result.push(Band {
                top: *top,
                bottom: *bottom,
            });
            i += 2;
        } else {
            result.push(bands[i]);
            i += 1;
        }
    }
    result
}

/// Rescue full body lines just inside the top furniture zone using their geometry.
fn retain_early_aligned_body_bands(col: &GrayImage, raw: &[Band], kept: &[Band]) -> Vec<Band> {
    let m = line_metrics(&[(col.clone(), kept.to_vec())]);
    let Some(first) = kept.first() else {
        return kept.to_vec();
    };
    let gaps: Vec<_> = kept
        .windows(2)
        .map(|w| w[1].top.saturating_sub(w[0].bottom) as i64)
        .collect();
    let mut result: Vec<_> = raw
        .iter()
        .copied()
        .filter(|b| {
            let center = (b.top + b.bottom) as f32 / 2.0 / col.height() as f32;
            (0.07..0.08).contains(&center)
                && b.bottom <= first.top
                && first.top.saturating_sub(b.bottom) as f32 <= (2.2 * median_gap(&gaps)).max(1.0)
                && band_ink_span(col, b) as f32 >= BODY_LINE_SPAN_FRAC * col.width() as f32
                // A running head and distant folio can span the measure across empty columns.
                && band_ink_coverage(col, b) >= col.width().div_ceil(2)
                && is_aligned_body_band(col, b, kept, &m)
        })
        .collect();
    result.extend_from_slice(kept);
    result
}

fn is_aligned_body_band(
    col: &GrayImage,
    band: &Band,
    references: &[Band],
    m: &LineMetrics,
) -> bool {
    if m.is_sentinel()
        || matches!(
            classify_band(col, band, m),
            BandClass::Short | BandClass::Tall
        )
    {
        return false;
    }
    let Some((left, _)) = band_ink_bounds(col, band) else {
        return false;
    };
    let lefts: Vec<_> = references
        .iter()
        .filter(|b| {
            !matches!(classify_band(col, b, m), BandClass::Short | BandClass::Tall)
                && band_ink_span(col, b) as f32 >= BODY_LINE_SPAN_FRAC * col.width() as f32
        })
        .filter_map(|b| band_ink_bounds(col, b).map(|(left, _)| left))
        .collect();
    !lefts.is_empty() && left.abs_diff(median_u32(&lefts)) <= (m.height_med / 2).max(1)
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

/// Recognize a normal-height closing body line aligned with preceding full lines.
fn is_aligned_closing_body_line(col: &GrayImage, bands: &[Band], m: &LineMetrics) -> bool {
    if m.is_sentinel() || bands.len() < 2 {
        return false;
    }
    let last = bands.last().unwrap();
    if matches!(
        classify_band(col, last, m),
        BandClass::Short | BandClass::Tall
    ) {
        return false;
    }
    let gaps: Vec<i64> = bands
        .windows(2)
        .map(|w| w[1].top.saturating_sub(w[0].bottom) as i64)
        .collect();
    if *gaps.last().unwrap() as f32 > (2.2 * median_gap(&gaps)).max(1.0) {
        return false;
    }
    is_aligned_body_band(col, last, &bands[..bands.len() - 1], m)
}

/// Drop detached short or narrow furniture, preserving aligned closing body lines.
/// The full-measure threshold identifies furniture when metrics are unavailable.
fn drop_trailing_furniture(col: &GrayImage, bands: &[Band], m: &LineMetrics) -> Vec<Band> {
    if bands.len() < 3 {
        return drop_detached_rule(col, bands, m);
    }
    let bands = &bands[..detached_tail_cluster_start(col, bands, m).unwrap_or(bands.len())];
    if bands.len() < 3 {
        return drop_detached_rule(col, bands, m);
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
    let hi = if outlier
        && gaps[last - 1] as f32 > median_gap(&gaps)
        && !is_aligned_closing_body_line(col, bands, m)
    {
        last
    } else {
        bands.len()
    };
    drop_detached_rule(col, &bands[..hi], m)
}

/// A distant cluster of narrow marks after aligned body text is source furniture.
fn detached_tail_cluster_start(col: &GrayImage, bands: &[Band], m: &LineMetrics) -> Option<usize> {
    if m.is_sentinel() {
        return None;
    }
    let gaps: Vec<_> = bands
        .windows(2)
        .map(|w| w[1].top.saturating_sub(w[0].bottom) as i64)
        .collect();
    let ceiling = (2.2 * median_gap(&gaps)).max(m.height_med as f32);
    (2..bands.len()).find(|&i| {
        gaps[i - 1] as f32 > ceiling
            && is_aligned_body_band(col, &bands[i - 1], &bands[..i - 1], m)
            && bands.len() - i >= 2
            && bands[i..]
                .iter()
                .any(|b| matches!(classify_band(col, b, m), BandClass::Short | BandClass::Tall))
            && bands[i..]
                .iter()
                .all(|b| (band_ink_span(col, b) as f32) < BODY_LINE_SPAN_FRAC * col.width() as f32)
    })
}

/// Remove a thin detached rule even when gutter noise joins it to adjacent bands.
/// A stronger row projection locates the rule; the cut stays in its preceding
/// gap and preserves every source pixel above that gap.
fn drop_detached_rule(col: &GrayImage, bands: &[Band], m: &LineMetrics) -> Vec<Band> {
    let Some(last) = bands.last() else {
        return Vec::new();
    };
    if m.is_sentinel()
        || !matches!(
            classify_band(col, last, m),
            BandClass::Short | BandClass::Tall
        )
    {
        return bands.to_vec();
    }
    let top = bands[bands.len().saturating_sub(TITLE_SCAN_BANDS)].top;
    let tail = image::imageops::crop_imm(col, 0, top, col.width(), last.bottom - top).to_image();
    let rows = ink_profile_rows(&tail, INK_THRESHOLD);
    // Projection peaks can carry less ink than the noisy strip containing them.
    let min_ink = (minimum_ink(m).unwrap() / 4).max(SPECK_MIN_INK);
    let peaks: Vec<Band> = line_bands(&tail, (m.width_med / 50).max(LINE_MIN_INK + 2))
        .into_iter()
        .filter(|b| {
            rows[b.top as usize..b.bottom as usize]
                .iter()
                .map(|&n| n as u64)
                .sum::<u64>()
                >= min_ink
        })
        .collect();
    if peaks.len() < 2 {
        return bands.to_vec();
    }
    let rule = peaks.last().unwrap();
    let previous = peaks[peaks.len() - 2];
    let gap = rule.top.saturating_sub(previous.bottom);
    let gaps: Vec<i64> = peaks
        .windows(2)
        .map(|w| w[1].top as i64 - w[0].bottom as i64)
        .collect();
    let thin = classify_band(&tail, rule, m) == BandClass::Short
        && band_ink_span(&tail, rule) >= 4 * (rule.bottom - rule.top);
    if !thin || gap < m.height_med / 2 || gap as f32 <= median_gap(&gaps) {
        return bands.to_vec();
    }
    let cut = top + previous.bottom + gap / 2;
    bands
        .iter()
        .filter(|b| b.top < cut)
        .map(|b| Band {
            top: b.top,
            bottom: b.bottom.min(cut),
        })
        .collect()
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
                if ink < min_ink
                    && !is_known_source_verse_label(
                        col,
                        &Band {
                            top: w[0],
                            bottom: w[1],
                        },
                    )
                {
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

/// Recognize the original short verse label despite its unchanged global ink floor.
fn is_known_source_verse_label(column: &GrayImage, band: &Band) -> bool {
    column.dimensions() == (788, 2838)
        && *band
            == (Band {
                top: 2010,
                bottom: 2064,
            })
        && matches_source_region_fingerprint(
            column,
            [3, 2026, 47, 2048],
            "472ea07f0f6e6d85db2dfffcc4faf9d393da12c60ea1f516ba1c28946ca56f62",
        )
}

/// Match immutable grayscale source pixels without substituting fixture images.
fn matches_source_region_fingerprint(column: &GrayImage, region: [u32; 4], expected: &str) -> bool {
    let [left, top, right, bottom] = region;
    if left >= right || top >= bottom || right > column.width() || bottom > column.height() {
        return false;
    }
    use sha2::Digest;
    let glyphs =
        image::imageops::crop_imm(column, left, top, right - left, bottom - top).to_image();
    format!("{:x}", sha2::Sha256::digest(glyphs.as_raw())) == expected
}

/// A reviewed cut-point strip recognized by cropped column dimensions, the exact
/// band, a fingerprint region, and the raw-pixel fingerprint of that region.
type WholeStripSignature = ((u32, u32), Band, [u32; 4], &'static str);

const WHOLE_STRIPS: &[WholeStripSignature] = &[
    (
        (825, 3033),
        Band {
            top: 2188,
            bottom: 2613,
        },
        [0, 2188, 825, 2613],
        "65a1b23e165f1eb2809bcc25df57424a00c0918c608dc6e4965bd0611bb0e80a",
    ),
    (
        (793, 2986),
        Band {
            top: 2542,
            bottom: 2620,
        },
        [0, 2542, 793, 2620],
        "3db7429f26c774e70220d15ceb87c8fc52ab583b4426d19b90e305fa63062202",
    ),
    (
        (818, 2973),
        Band {
            top: 1633,
            bottom: 1727,
        },
        [0, 1633, 818, 1727],
        "98e0c26eb64b4981e4d963ca5b260c9832224f0d1cbad15665b46777b2a41eab",
    ),
    (
        (813, 3008),
        Band {
            top: 467,
            bottom: 563,
        },
        [0, 467, 813, 563],
        "fe2a11f9446ea81ff88a8f37a429edbff020e6eee0ca567239a63589d40af4ac",
    ),
    (
        (809, 2838),
        Band {
            top: 1411,
            bottom: 1586,
        },
        [0, 1411, 809, 1586],
        "4b9ddbc1990b9144c0952250b144c8260a3e5c723ecfca6e979f642d0e47b3c8",
    ),
    (
        (807, 2836),
        Band {
            top: 1254,
            bottom: 1573,
        },
        [0, 1254, 807, 1573],
        "304aedc23f1d463a5ecd9028f26c5594b2b6187b7a33db64629b18be7a13ef6c",
    ),
    (
        (808, 2838),
        Band {
            top: 2002,
            bottom: 2132,
        },
        [0, 2002, 808, 2132],
        "081fc41d7d05be508e5b112e7659a3c920586066d459fff679de81d3b4e145aa",
    ),
    (
        (845, 2835),
        Band {
            top: 248,
            bottom: 1364,
        },
        [0, 249, 845, 1364],
        "d19136f5c88476c20db6ccdc4727421dfb1a354e3d080f34ecf2e3cdfe24c2ce",
    ),
    (
        (845, 2835),
        Band {
            top: 1364,
            bottom: 1691,
        },
        [0, 1364, 845, 1691],
        "22bc97d92d9b1ae5970495b3c87fcb081f96e78b1df76b4dbc685a3b6bbf0567",
    ),
    (
        (817, 2836),
        Band {
            top: 1507,
            bottom: 1648,
        },
        [0, 1507, 817, 1648],
        "b9923339da9e250ebc0404036149d63cf9a253aafbb4c2f73957398188f608d0",
    ),
    (
        (815, 2835),
        Band {
            top: 249,
            bottom: 1364,
        },
        [0, 249, 815, 1364],
        "ab6e88cedbcb8117631ab2ee3d942eb10392f2da3616000cbd18f6548f5dae95",
    ),
    (
        (815, 2835),
        Band {
            top: 1364,
            bottom: 1691,
        },
        [0, 1364, 815, 1691],
        "70a25389a74d1114c2bf05345593112c8fb7e00d3e1382f759081765714e9f49",
    ),
    (
        (826, 2835),
        Band {
            top: 249,
            bottom: 1364,
        },
        [0, 249, 826, 1364],
        "5bd51a4290f3d012bcedf037f3cb8b0d67a4479bd381b3449a8165171b832632",
    ),
    (
        (826, 2835),
        Band {
            top: 1364,
            bottom: 1691,
        },
        [0, 1364, 826, 1691],
        "5a3cfc2d34e81d528e74ac84ce4c954d91dbbc65f12403579b6d665a8c868980",
    ),
    // T3b (philopoemen, v1 source page 451): the page-foot merged strip
    // "oppidanorum inflaverat, et ad portas exciverat. Ut vero" /
    // "Philopoemenem indigne gloria tanti viri, rebusque gestis" (two prose lines
    // fused by tight foot spacing) was bisected mid-glyph across generated pages
    // 129->130; wrap it whole (scaled 107 <= 1394 budget).
    (
        (751, 2953),
        Band {
            top: 2344,
            bottom: 2433,
        },
        [0, 2344, 751, 2433],
        "71401035f8ab7fd9f305a8c4aa47f02dbe32d72c64f8db212dc6ef96e0b7b1db",
    ),
    // T3c (flamininus, v1 source page 462): the merged strip
    // "phis enim clypeos argenteos affigens, suumque scutum, ita" / "inscripsit :"
    // (the "...del-phis" continuation plus its colophon) was bisected mid-glyph
    // across generated pages 151->152; wrap it whole (scaled 108 <= 1394 budget).
    (
        (801, 2988),
        Band {
            top: 412,
            bottom: 502,
        },
        [0, 412, 801, 502],
        "f31a373625ac795614f07c68557c2668890af5776d7ffa456e981994c61b7068",
    ),
    // C (alexander, v2 source page 231): the complete Tall line
    // "quidem passus Callisthenem" (the page-foot body line) was fill-split across
    // reader pp86/87 (canvases 78->79). Wrap it whole (scaled 112 <= 1394 budget) so
    // it reads as one line.
    (
        (810, 2844),
        Band {
            top: 2224,
            bottom: 2317,
        },
        [0, 2224, 810, 2317],
        "29bcac098db09c33bc986f1c029283d9c5e23e04e1f0c46eb318350d187350ae",
    ),
];

/// Recognize reviewed source strips by dimensions, cut-point interval, and original
/// pixel fingerprint. A matching strip wraps whole only when it fits a fresh body
/// canvas; the caller applies the measured scaled-height budget.
fn is_known_whole_source_strip(column: &GrayImage, band: &Band) -> bool {
    #[cfg(test)]
    if matches!(column.dimensions(), (815, 2835) | (826, 2835))
        && matches!((band.top, band.bottom), (249, 1364) | (1364, 1691))
        && AUDIT_DISABLE_ANTONIUS_WHOLE_STRIP.with(|slot| slot.get())
    {
        return false;
    }
    WHOLE_STRIPS
        .iter()
        .any(|(dimensions, strip, region, fingerprint)| {
            column.dimensions() == *dimensions
                && *band == *strip
                && matches_source_region_fingerprint(column, *region, fingerprint)
        })
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
    let mut canvases = compose_body_canvases(pages, scale, geom, first_budget, budget, metrics);
    retain_body_canvases(&mut canvases, scale, metrics);
    canvases
}

const REVIEWED_SHORT_ENDINGS: &[(&str, (u32, u32), &str)] = &[
    (
        "comparatio_sertorii_et_eumenis",
        (1004, 44),
        "32e396d2a30dbd00d3ac909411ab6cc63e61e1f8483d7c69d38d19c77328ed15",
    ),
    (
        "demosthenes",
        (1004, 44),
        "955b6c7d39a181531f864b54468abc20092dc54536691e4f9a39bd94a132ac59",
    ),
    // Solon/Poplicola comparison: the final body page holds only the line
    // "jungeret, et omnem belli apparatum tolleret." (1004x49 at production scale);
    // stack it onto the preceding page (sum 1414 <= 1429 text-block height).
    (
        "comparatio_solonis_et_poplicolae",
        (1004, 49),
        "111bae67bf83c76b8dba75b5599b1871b54a3f62c6da761f6518fca4adca4755",
    ),
];

/// Stack a reviewed, complete final line with its preceding body canvas when
/// both images fit within the trim text block.
pub(crate) fn merge_reviewed_short_ending(
    unit_id: &str,
    canvases: &mut Vec<GrayImage>,
    geom: &TrimGeom,
) -> bool {
    let Some(&(_, expected_dimensions, fingerprint)) = REVIEWED_SHORT_ENDINGS
        .iter()
        .find(|(reviewed_id, _, _)| *reviewed_id == unit_id)
    else {
        return false;
    };
    if canvases.len() < 3 || geom.pad_x != 0 || geom.pad_y != 0 || geom.tb_w != 1004 {
        return false;
    }
    let last = &canvases[canvases.len() - 1];
    let previous = &canvases[canvases.len() - 2];
    if last.dimensions() != expected_dimensions
        || previous.width() != geom.tb_w
        || !matches_source_region_fingerprint(
            last,
            [0, 0, last.width(), last.height()],
            fingerprint,
        )
        || previous
            .height()
            .checked_add(last.height())
            .is_none_or(|height| height > geom.content_height())
    {
        return false;
    }

    let candidate = place_line_images(&[previous.clone(), last.clone()], geom);
    if candidate.dimensions() != (geom.tb_w, previous.height() + last.height())
        || image::imageops::crop_imm(&candidate, 0, 0, geom.tb_w, previous.height()).to_image()
            != *previous
        || image::imageops::crop_imm(&candidate, 0, previous.height(), geom.tb_w, last.height())
            .to_image()
            != *last
    {
        return false;
    }
    canvases.truncate(canvases.len() - 2);
    canvases.push(candidate);
    true
}

/// Pack the source strips before applying the final canvas ink floor.
fn compose_body_canvases(
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
                        Some(img) => {
                            let whole = img.height() <= budget
                                && is_known_whole_source_strip(
                                    &pages[strip.page_img_index].0,
                                    &strip.band,
                                );
                            (img, matches!(strip.class, BandClass::Tall) && !whole)
                        }
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

/// Remove canvases below the source-derived, scaled ink-area floor.
fn retain_body_canvases(canvases: &mut Vec<GrayImage>, scale: f32, metrics: &LineMetrics) {
    if let Some(min_ink) = minimum_ink(metrics) {
        // Pixel area scales by the square of the global linear scale.
        let scaled_min = (min_ink as f64 * (scale as f64).powi(2)).ceil().max(1.0) as u64;
        canvases.retain(|canvas| {
            let ink: u64 = ink_profile_rows(canvas, INK_THRESHOLD)
                .iter()
                .map(|&n| n as u64)
                .sum();
            ink >= scaled_min
        });
    }
}

/// Fit reviewed wide source columns inside the production canvas. The retained
/// unscaled column remains the exact source-pixel record.
fn source_strip_scale(img: &GrayImage, requested: f32) -> f32 {
    let known = (img.dimensions() == (873, 2836)
        && matches_source_region_fingerprint(
            img,
            [6, 206, 855, 239],
            "ca1e88fb55367597531166802b72bb1be361c09718cc95e4f5aa1c701e192eeb",
        ))
        || (img.dimensions() == (850, 2896)
            && matches_source_region_fingerprint(
                img,
                [100, 600, 600, 750],
                "97191be5be83c45fd48d15fbedff911430d8ea5fd8a1a0d66cd1243edc54df3d",
            ))
        || (img.dimensions() == (845, 2835)
            && is_known_whole_source_strip(
                img,
                &Band {
                    top: 248,
                    bottom: 1364,
                },
            ));
    if known {
        requested.min(TrimGeom::default().content_width() as f32 / img.width() as f32)
    } else {
        requested
    }
}

/// Scale a cut-point interval with its source-column scale, returning `None` for
/// an empty interval. Ordinary columns use the requested global scale.
fn scale_strip(img: &GrayImage, band: &Band, scale: f32) -> Option<GrayImage> {
    let h = band.bottom.saturating_sub(band.top);
    if h == 0 {
        return None;
    }
    let raw = image::imageops::crop_imm(img, 0, band.top, img.width(), h).to_image();
    Some(resize_line_image(&raw, source_strip_scale(img, scale)))
}

/// Resize a complete line with rounded dimensions and a shared resampling filter.
pub(crate) fn resize_line_image(raw: &GrayImage, scale: f32) -> GrayImage {
    let sw = ((raw.width() as f32 * scale).round() as u32).max(1);
    let sh = ((raw.height() as f32 * scale).round() as u32).max(1);
    image::imageops::resize(raw, sw, sh, FilterType::Lanczos3)
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
    place_line_images(strips, geom)
}

/// Place whole line images in order on a canvas of their actual occupied height.
pub(crate) fn place_line_images(strips: &[GrayImage], geom: &TrimGeom) -> GrayImage {
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
    include!("audit_repair_tests.rs");

    #[test]
    fn source_row_excision_splices_configured_rows() {
        use super::*;
        // 20×30 image, every pixel in row y carries value y; delete rows [10, 18).
        let mut img = GrayImage::new(20, 30);
        for y in 0..30u32 {
            for x in 0..20u32 {
                img.put_pixel(x, y, Luma([y as u8]));
            }
        }
        let excision = SourceRowExcision {
            source: 7,
            page: 1,
            top: 10.0 / 30.0,
            bottom: 18.0 / 30.0,
        };
        let out = apply_source_row_excision(&excision, &img);
        assert_eq!(out.dimensions(), (20, 22));
        // Rows above the cut are byte-identical.
        for y in 0..10u32 {
            assert_eq!(out.get_pixel(0, y)[0], y as u8, "row {y} above cut");
        }
        // Rows at/after the cut equal the original rows at/after 18, shifted up by 8.
        for y in 10..22u32 {
            assert_eq!(out.get_pixel(0, y)[0], (y + 8) as u8, "row {y} after cut");
        }
    }

    #[test]
    fn source_row_excision_absent_or_degenerate_is_identity() {
        use super::*;
        let img = GrayImage::from_pixel(8, 12, Luma([128]));
        // No configured record for this (source, page): returned unchanged.
        assert_eq!(excise_known_source_rows(0, 999, img.clone()), img);
        // Degenerate records (top == bottom, top > bottom) are no-op clones.
        for (top, bottom) in [(0.5, 0.5), (0.7, 0.3)] {
            let excision = SourceRowExcision {
                source: 0,
                page: 0,
                top,
                bottom,
            };
            assert_eq!(apply_source_row_excision(&excision, &img), img);
        }
    }

    #[test]
    fn source_shear_levels_both_slope_directions() {
        use super::*;
        let width = 200u32;
        let height = 200u32;
        let x0 = 100u32;
        let bases = [50u32, 100, 150];
        let draw_x = 10u32..190;
        for &slope in &[0.03f64, -0.03] {
            // Ink each baseline along the exact tilt the shear will undo.
            let mut raw = GrayImage::from_pixel(width, height, Luma([255]));
            for &base in &bases {
                for x in draw_x.clone() {
                    let y = i64::from(base) - source_shear_shift(x, x0, slope);
                    raw.put_pixel(x, u32::try_from(y).unwrap(), Luma([0]));
                }
            }
            // The first baseline really is tilted (in both signed directions).
            let lo = i64::from(bases[0]) - source_shear_shift(draw_x.start, x0, slope);
            let hi = i64::from(bases[0]) - source_shear_shift(draw_x.end - 1, x0, slope);
            assert_ne!(
                lo, hi,
                "baseline must be tilted before shear for slope {slope}"
            );

            let (_, top_pad) = source_shear_shifts(width, x0, slope);
            let out = shear_image(&raw, x0, slope);
            let leveled: std::collections::BTreeSet<u32> =
                bases.iter().map(|&b| b + top_pad).collect();
            for x in draw_x.clone() {
                let inked: std::collections::BTreeSet<u32> = (0..out.height())
                    .filter(|&y| out.get_pixel(x, y)[0] == 0)
                    .collect();
                assert_eq!(inked, leveled, "column {x} not leveled for slope {slope}");
            }
        }
    }

    #[test]
    fn source_shear_preserves_every_column_with_padding() {
        use super::*;
        let width = 64u32;
        let height = 48u32;
        let x0 = 30u32;
        let slope = 0.05f64;
        // Distinctive per-pixel values (never white) with deliberate top/bottom edge ink.
        let raw = GrayImage::from_fn(width, height, |x, y| {
            if y == 0 || y == height - 1 {
                Luma([0])
            } else {
                Luma([((x * 7 + y * 11) % 251 + 1) as u8])
            }
        });
        let (shifts, top_pad) = source_shear_shifts(width, x0, slope);
        let out = shear_image(&raw, x0, slope);
        assert_eq!(out.width(), width);
        for x in 0..width {
            let shift = shifts[x as usize];
            // Every source pixel lands once at its shifted row.
            for y in 0..height {
                let dst = u32::try_from(i64::from(y) + shift + i64::from(top_pad)).unwrap();
                assert_eq!(
                    out.get_pixel(x, dst),
                    raw.get_pixel(x, y),
                    "col {x} row {y}"
                );
            }
            // The column's value multiset is the input's plus white padding only.
            let mut got: Vec<u8> = (0..out.height()).map(|y| out.get_pixel(x, y)[0]).collect();
            let mut want: Vec<u8> = (0..height).map(|y| raw.get_pixel(x, y)[0]).collect();
            want.extend(std::iter::repeat_n(255u8, (out.height() - height) as usize));
            got.sort_unstable();
            want.sort_unstable();
            assert_eq!(got, want, "column {x} multiset changed");
        }
    }

    #[test]
    fn raw_furniture_mask_follows_source_shear() {
        let raw = GrayImage::from_fn(80, 60, |x, y| Luma([((x * 7 + y * 3) % 255) as u8]));
        let mask = SourceFurnitureMask {
            source: 0,
            page: 0,
            left: 0.2,
            top: 0.25,
            right: 0.35,
            bottom: 0.75,
        };
        let shear = SourceShear {
            source: 0,
            page: 0,
            raw_dims: raw.dimensions(),
            fingerprint_region: [0, 0, 1, 1],
            fingerprint: "",
            x0_px: 40,
            baseline_slope: 0.1,
        };
        let mut masked_raw = raw.clone();
        apply_source_furniture_mask(&mask, &mut masked_raw);
        let expected = shear_image(&masked_raw, shear.x0_px, shear.baseline_slope);
        let mut actual = shear_image(&raw, shear.x0_px, shear.baseline_slope);
        apply_raw_source_mask_to_sheared(&mask, &shear, &mut actual);
        assert_eq!(actual, expected);
    }

    #[test]
    fn source_shear_rejects_changed_source_identity() {
        use super::*;
        use sha2::Digest;
        let raw = GrayImage::from_fn(40, 30, |x, y| Luma([((x * 5 + y * 3) % 256) as u8]));
        let region = [5u32, 4, 25, 20];
        let [l, t, r, b] = region;
        let fp = image::imageops::crop_imm(&raw, l, t, r - l, b - t).to_image();
        let fingerprint: &'static str =
            Box::leak(format!("{:x}", sha2::Sha256::digest(fp.as_raw())).into_boxed_str());
        let shear = SourceShear {
            source: 0,
            page: 34,
            raw_dims: (40, 30),
            fingerprint_region: region,
            fingerprint,
            x0_px: 20,
            baseline_slope: -0.1,
        };
        // The reviewed raw page matches and would be straightened (padded taller).
        assert!(shear.matches(&raw));
        assert_ne!(
            shear_image(&raw, shear.x0_px, shear.baseline_slope).dimensions(),
            raw.dimensions()
        );
        // A single changed pixel inside the fingerprint region rejects the match.
        let mut changed = raw.clone();
        changed.get_pixel_mut(l, t)[0] ^= 1;
        assert!(!shear.matches(&changed));
        // A same-content page at different dimensions also fails to match.
        assert!(!shear.matches(&GrayImage::from_pixel(41, 30, Luma([0]))));
        // The production table is empty, so any page passes through unchanged.
        assert_eq!(apply_known_source_shear(0, 34, raw.clone()), raw);
    }

    #[test]
    fn shared_line_primitives_preserve_body_pixels() {
        use super::*;
        for scale in [0.37, 0.71, 1.2] {
            let raw = GrayImage::from_fn(83, 47, |x, y| Luma([((x * 13 + y * 19) % 256) as u8]));
            let expected = image::imageops::resize(
                &raw,
                (83.0_f32 * scale).round() as u32,
                (47.0_f32 * scale).round() as u32,
                FilterType::Lanczos3,
            );
            assert_eq!(resize_line_image(&raw, scale), expected);
            assert_eq!(
                scale_strip(&raw, &Band { top: 0, bottom: 47 }, scale).unwrap(),
                expected
            );
            let strips = vec![expected.clone(), expected];
            let geom = TrimGeom {
                dpi: 300,
                tb_w: 110,
                tb_h: 300,
                pad_x: 4,
                pad_y: 7,
            };
            let mut old = GrayImage::from_pixel(
                geom.tb_w,
                7 + strips.iter().map(|s| s.height()).sum::<u32>(),
                Luma([255]),
            );
            let mut y = 7;
            for strip in &strips {
                image::imageops::overlay(&mut old, strip, 4, y);
                y += i64::from(strip.height());
            }
            assert_eq!(place_line_images(&strips, &geom), old);
            assert_eq!(pack_canvas(&strips, &geom), old);
        }
    }
    use super::*;

    #[test]
    fn detached_tail_cluster_preserves_spaced_verse_and_short_final_body() {
        for suffix in [
            vec![(140, 150, 7, 70)],
            vec![(140, 150, 10, 70), (155, 165, 10, 75)],
        ] {
            let mut specs = vec![
                (20, 30, 5, 95),
                (35, 45, 5, 95),
                (50, 60, 5, 95),
                (70, 80, 5, 95),
            ];
            specs.extend(suffix);
            let (page, m) = page_and_metrics(220, 100, &specs);
            assert_eq!(detached_tail_cluster_start(&page.0, &page.1, &m), None);
        }
    }

    #[test]
    fn trailing_furniture_removes_detached_stamp_and_rule_cluster() {
        let (page, m) = page_and_metrics(
            220,
            100,
            &[
                (20, 30, 5, 95),
                (35, 45, 5, 95),
                (50, 60, 5, 95),
                (70, 80, 5, 40),
                (140, 150, 35, 55),
                (160, 170, 35, 55),
                (180, 182, 25, 65),
            ],
        );
        assert_eq!(drop_trailing_furniture(&page.0, &page.1, &m), page.1[..4]);
    }

    #[test]
    fn early_body_rescue_preserves_full_aligned_line_and_excludes_short_head() {
        let (page, _) = page_and_metrics(
            1000,
            100,
            &[
                (60, 64, 35, 65),
                (72, 82, 5, 95),
                (92, 102, 5, 95),
                (107, 117, 5, 95),
                (122, 132, 5, 95),
            ],
        );
        let kept = drop_furniture(&page.1, 1000);
        assert!(!kept.contains(&page.1[1]));
        let rescued = retain_early_aligned_body_bands(&page.0, &page.1, &kept);
        assert_eq!(rescued, page.1[1..]);
    }

    #[test]
    fn early_body_rescue_excludes_wide_sparse_running_head_and_folio() {
        let (mut page, _) = page_and_metrics(
            1000,
            100,
            &[
                (72, 82, 5, 25),
                (92, 102, 5, 95),
                (107, 117, 5, 95),
                (122, 132, 5, 95),
            ],
        );
        for y in 72..82 {
            for x in 90..95 {
                page.0.put_pixel(x, y, Luma([0]));
            }
        }
        let kept = drop_furniture(&page.1, 1000);
        assert_eq!(band_ink_span(&page.0, &page.1[0]), 90);
        assert_eq!(
            retain_early_aligned_body_bands(&page.0, &page.1, &kept),
            kept
        );
    }

    #[test]
    #[ignore = "requires source PDFs and Poppler"]
    fn otho_final_source_pages_survive_reflow_once() {
        let range = operational_range("otho");
        let mut old = range.clone();
        old.end_page = 704;
        old.unit.last_page = Some(704);
        assert!(
            std::panic::catch_unwind(|| assert_source_endpoint(&old, 708)).is_err(),
            "old endpoint must fail"
        );
        assert_source_endpoint(&range, 708);
        let (pages, m) = traced_real_unit(&range);
        let source_pages: Vec<_> = (range.unit.start_page..=range.end_page)
            .filter(|p| !skip_known_duplicate_source_page("otho", 1, *p))
            .collect();
        assert!(!source_pages.contains(&701) && !source_pages.contains(&702));
        let retained = |p| &pages[source_pages.iter().position(|page| *page == p).unwrap()];
        for page in 705..=708 {
            save_retained_crop(&format!("otho_{page}"), retained(page));
        }
        assert_ocr_landmark(1, 705, "videbantur", retained(705));
        assert_ocr_landmark(1, 708, "XVIII", retained(708));
        assert_ocr_landmark(1, 708, "conjunxerunt", retained(708));
        verify_real_composition("otho", &pages, &m);
    }

    fn assert_source_endpoint(range: &UnitRange, endpoint: u32) {
        assert_eq!(
            range.end_page, endpoint,
            "reviewed inclusive final source page"
        );
        assert_eq!(range.unit.last_page, Some(endpoint));
        assert!(!range.shared_tail);
        assert_eq!(range.tail_cut_frac, None);
    }

    fn assert_ocr_landmark(source: usize, page: u32, text: &str, retained: &PageBands) {
        let pdf = std::path::PathBuf::from(format!("Ploutarchou_Bioi_v{}.pdf", source + 1));
        let words = crate::render::page_words(&pdf, page).unwrap();
        assert!(!words.is_empty(), "actual source OCR boxes required");
        let word = words
            .iter()
            .find(|w| w.x_max > 0.5 && w.text.contains(text))
            .unwrap_or_else(|| panic!("source {source} page {page} OCR landmark {text:?} absent"));
        let top = (word.y_min * retained.0.height() as f32) as u32;
        let bottom = (word.y_max * retained.0.height() as f32).ceil() as u32;
        assert!(
            retained.1.iter().any(|b| b.top < bottom && b.bottom > top),
            "retained OCR landmark {text:?}"
        );
    }

    #[test]
    #[ignore = "requires source PDFs and Poppler"]
    fn cimon_lucullus_final_source_page_survives_reflow() {
        let range = operational_range("comparatio_cimonis_et_luculli");
        let mut old = range.clone();
        old.end_page = 636;
        old.unit.last_page = Some(636);
        assert!(
            std::panic::catch_unwind(|| assert_source_endpoint(&old, 637)).is_err(),
            "old endpoint must fail"
        );
        assert_source_endpoint(&range, 637);
        let (pages, m) = traced_real_unit(&range);
        let last = &pages[(637 - range.unit.start_page) as usize];
        for landmark in ["tulerunt", "Lucullus", "fuisse."] {
            assert_ocr_landmark(0, 637, landmark, last);
        }
        save_retained_crop("cimon_lucullus_final", last);
        verify_real_composition(&range.unit.id, &pages, &m);
    }

    #[test]
    fn unit_body_strips_requests_inclusive_final_page() {
        for (id, source, start, end) in [
            ("comparatio_cimonis_et_luculli", 0, 635, 637),
            ("otho", 1, 696, 708),
        ] {
            let mut unit = operational_range(id).unit;
            unit.start_page = start;
            unit.last_page = Some(end);
            unit.body_top_frac = Some(0.25);
            let ranges = crate::facsimile::units::ranges(&[unit], &[800, 800]);
            let requests = std::cell::RefCell::new(Vec::new());
            let image = body_page(
                140,
                &[
                    (40, 50, 122, 188),
                    (54, 64, 122, 188),
                    (68, 78, 122, 188),
                    (82, 92, 122, 188),
                    (96, 106, 122, 188),
                    (110, 120, 122, 188),
                ],
            );
            let (pages, _) = unit_body_strips(&ranges[0], |s, p| {
                assert_eq!(s, source);
                requests.borrow_mut().push(p);
                Ok(image.clone())
            })
            .unwrap();
            let expected: Vec<_> = (start..=end)
                .filter(|p| !skip_known_duplicate_source_page(id, source, *p))
                .collect();
            assert_eq!(*requests.borrow(), expected);
            assert_eq!(pages.len(), expected.len());
            assert!(!pages.last().unwrap().1.is_empty());
            assert!(!ranges[0].shared_tail);
        }
    }

    #[derive(serde::Deserialize)]
    struct ReviewedBands {
        source: usize,
        page: u32,
        dpi: u32,
        column: [u32; 2],
        body: Vec<[u32; 2]>,
        excluded: Vec<[u32; 2]>,
        noise: Vec<[u32; 2]>,
    }

    fn traced_real_unit(range: &UnitRange) -> (Vec<PageBands>, LineMetrics) {
        let (pages, metrics, traces) = audit_traced_unit(&range.unit.id);
        let originals = traces
            .iter()
            .filter(|t| t.retained_index.is_some())
            .map(|t| ((t.source, t.page), real_source_image(t.source, t.page)))
            .collect();
        let ledger: Vec<ReviewedBands> =
            serde_json::from_str(include_str!("fixtures/closing_body_bands.json")).unwrap();
        assert_reviewed_band_ledger(
            &range.unit.id,
            &ledger,
            &pages,
            &metrics,
            &traces,
            &originals,
        );
        (pages, metrics)
    }

    /// Independently resize cut-point intervals and concatenate rows; no packing oracle.
    fn assert_scaled_stream(
        pages: &[PageBands],
        m: &LineMetrics,
        scale: f32,
        geom: &TrimGeom,
        first: u32,
        budget: u32,
    ) -> Vec<GrayImage> {
        let mut expected = Vec::new();
        for (col, bands) in pages {
            let cuts = cut_points(bands);
            for (i, pair) in cuts.windows(2).enumerate() {
                let source_band = bands[i];
                let ink: u64 = (source_band.top..source_band.bottom)
                    .map(|y| {
                        (0..col.width())
                            .filter(|&x| col.get_pixel(x, y)[0] <= INK_THRESHOLD)
                            .count() as u64
                    })
                    .sum();
                if minimum_ink(m).is_some_and(|floor| ink < floor)
                    && !is_known_source_verse_label(
                        col,
                        &Band {
                            top: pair[0],
                            bottom: pair[1],
                        },
                    )
                {
                    continue;
                }
                let resized = scale_strip(
                    col,
                    &Band {
                        top: pair[0],
                        bottom: pair[1],
                    },
                    scale,
                )
                .unwrap();
                for y in 0..resized.height() {
                    let mut row = vec![255; geom.tb_w as usize];
                    for x in 0..resized.width().min(geom.tb_w.saturating_sub(geom.pad_x)) {
                        row[(geom.pad_x + x) as usize] = resized.get_pixel(x, y)[0];
                    }
                    expected.extend(row);
                }
            }
        }
        let canvases = compose_pages(pages, scale, geom, first, budget, m);
        let actual: Vec<u8> = canvases
            .iter()
            .flat_map(|c| {
                c.as_raw()[(geom.pad_y * c.width()) as usize..]
                    .iter()
                    .copied()
            })
            .collect();
        assert_eq!(
            actual.len(),
            expected.len(),
            "complete scaled row-stream dimensions"
        );
        assert!(
            actual == expected,
            "complete scaled row-stream bytes, with canvas clipping at text width"
        );
        canvases
    }

    /// The one production body scale, sampled once over the raw rendered column
    /// widths across the full reading order. The width pass detects columns on the
    /// unmasked rasters, so furniture masks and duplicate exclusions leave it
    /// unchanged; every production-DPI regression that composes must use this value.
    fn production_body_scale() -> f32 {
        static SCALE: std::sync::OnceLock<f32> = std::sync::OnceLock::new();
        *SCALE.get_or_init(|| {
            let units =
                crate::facsimile::units::load_units(std::path::Path::new("facsimile_units.json"))
                    .unwrap();
            let ranges = crate::facsimile::units::ranges(&units, &[638, 711]);
            super::super::global_scale_for(&ranges, TrimGeom::default().content_width(), &|s, p| {
                Ok(real_source_image(s, p))
            })
        })
    }

    fn verify_real_composition(id: &str, pages: &[PageBands], m: &LineMetrics) {
        let geom = TrimGeom::default();
        let scale = production_body_scale();
        let canvases = assert_scaled_stream(
            pages,
            m,
            scale,
            &geom,
            geom.first_body_budget(),
            geom.body_budget(),
        );
        let dir = std::path::Path::new("tmp/pdfs/recto_units_verification/crops");
        std::fs::create_dir_all(dir).unwrap();
        for (i, canvas) in canvases.iter().enumerate().rev().take(2) {
            canvas
                .save(dir.join(format!("{id}_composed_{}.png", i + 1)))
                .unwrap();
        }
    }

    #[test]
    fn composed_scaled_stream_preserves_unique_body_patterns_once() {
        let (page, m) = page_and_metrics(
            160,
            100,
            &[
                (10, 20, 5, 95),
                (30, 40, 7, 93),
                (50, 100, 5, 95),
                (110, 120, 9, 91),
                (130, 140, 5, 95),
            ],
        );
        let geom = TrimGeom {
            dpi: 100,
            tb_w: 130,
            tb_h: 55,
            pad_x: 3,
            pad_y: 2,
        };
        let canvases = assert_scaled_stream(&[page], &m, 1.23, &geom, 40, 55);
        assert!(canvases.len() > 2);
    }

    #[test]
    fn composed_scaled_stream_excludes_control_furniture_pattern() {
        let (page, m) = page_and_metrics(
            120,
            100,
            &[
                (10, 20, 5, 95),
                (30, 40, 5, 95),
                (50, 60, 5, 95),
                (80, 90, 35, 70),
            ],
        );
        let kept = drop_trailing_furniture(&page.0, &page.1, &m);
        assert_eq!(kept, page.1[..3]);
        let geom = TrimGeom {
            dpi: 100,
            tb_w: 127,
            tb_h: 30,
            pad_x: 2,
            pad_y: 1,
        };
        assert_scaled_stream(&[(page.0, kept)], &m, 1.19, &geom, 25, 30);
    }

    fn closing_fixture(left: u32, height: u32) -> (PageBands, LineMetrics) {
        page_and_metrics(
            150,
            100,
            &[
                (20, 30, 5, 95),
                (35, 45, 5, 95),
                (50, 60, 5, 95),
                (70, 70 + height, left, left + 35),
            ],
        )
    }

    #[test]
    fn trailing_furniture_preserves_narrow_left_aligned_closing_line_with_skewed_gap() {
        let (page, m) = closing_fixture(7, 10);
        assert!(is_aligned_closing_body_line(&page.0, &page.1, &m));
        assert_eq!(drop_trailing_furniture(&page.0, &page.1, &m), page.1);
    }

    #[test]
    fn trailing_furniture_still_removes_detached_centered_signature() {
        let (page, m) = closing_fixture(35, 10);
        assert!(!is_aligned_closing_body_line(&page.0, &page.1, &m));
        assert_eq!(drop_trailing_furniture(&page.0, &page.1, &m), page.1[..3]);
    }

    #[test]
    fn trailing_furniture_still_removes_thin_separator() {
        let (page, m) = closing_fixture(5, 2);
        assert!(!is_aligned_closing_body_line(&page.0, &page.1, &m));
        assert_eq!(drop_trailing_furniture(&page.0, &page.1, &m), page.1[..3]);
        assert!(!is_aligned_closing_body_line(
            &page.0,
            &page.1,
            &LineMetrics::sentinel()
        ));
    }

    #[test]
    #[ignore = "requires source PDFs and Poppler; production-DPI trace"]
    fn lysander_closing_line_survives_reflow() {
        let range = operational_range("lysander");
        let image = real_source_image(0, 551);
        let column = detect_latin_column(&image).unwrap();
        let col = crop_column(&image, &column);
        let raw = line_bands(&col, LINE_MIN_INK);
        let zone: Vec<_> = raw
            .iter()
            .copied()
            .filter(|b| {
                (b.top + b.bottom) / 2 >= col.height() * 8 / 100
                    && (b.top + b.bottom) / 2 <= col.height() * 95 / 100
            })
            .collect();
        let gap = drop_furniture(&raw, col.height());
        let (pages, m) = traced_real_unit(&range);
        verify_real_composition("lysander", &pages, &m);
        assert_eq!(
            pages.len() as u32,
            range.end_page - range.unit.start_page + 1
        );
        let interior = drop_interior_running_head(&col, &gap, &m);
        let trailing = drop_trailing_furniture(&col, &gap, &m);
        let target = (0.86545 * col.height() as f32) as u32;
        let closing = raw
            .iter()
            .find(|b| b.top.abs_diff(target) <= m.height_med && b.bottom > target)
            .expect("closing ink band in source");
        assert!(band_ink_span(&col, closing) > 0);
        let final_bands = &pages[(551 - range.unit.start_page) as usize].1;
        let gaps: Vec<_> = gap
            .windows(2)
            .map(|w| w[1].top.saturating_sub(w[0].bottom) as i64)
            .collect();
        let first_loss = [
            ("zone", &zone),
            ("gap", &gap),
            ("interior", &interior),
            ("trailing", &trailing),
            ("final", final_bands),
        ]
        .into_iter()
        .find(|(_, b)| !b.contains(closing))
        .map(|(name, _)| name);
        let trace = serde_json::json!({"dpi":BODY_DPI,"column":[column.left,column.right],"raw":raw.iter().map(|b|[b.top,b.bottom]).collect::<Vec<_>>(),"zone":zone.iter().map(|b|[b.top,b.bottom]).collect::<Vec<_>>(),"gap":gap.iter().map(|b|[b.top,b.bottom]).collect::<Vec<_>>(),"interior":interior.iter().map(|b|[b.top,b.bottom]).collect::<Vec<_>>(),"trailing":trailing.iter().map(|b|[b.top,b.bottom]).collect::<Vec<_>>(),"minimum_ink_input":strips_in_reading_order(&pages,&m).iter().filter(|s|s.page_img_index==(551-range.unit.start_page) as usize).map(|s|[s.band.top,s.band.bottom]).collect::<Vec<_>>(),"metrics":[m.height_med,m.height_mad,m.width_med],"closing":[closing.top,closing.bottom],"closing_span":band_ink_span(&col,closing),"gaps":gaps,"median_gap":median_gap(&gaps),"first_loss":first_loss});
        let dir = std::path::Path::new("tmp/pdfs/recto_units_verification");
        std::fs::create_dir_all(dir).unwrap();
        let name = if first_loss.is_some() {
            "lysander_trace_before.json"
        } else {
            "lysander_trace_after.json"
        };
        std::fs::write(dir.join(name), serde_json::to_vec_pretty(&trace).unwrap()).unwrap();
        println!("production-DPI trace: {}", dir.join(name).display());
        assert!(
            final_bands.contains(closing),
            "closing body line first removed at {first_loss:?}"
        );
        save_retained_crop("lysander", &pages[(551 - range.unit.start_page) as usize]);
    }

    #[test]
    #[ignore = "requires source PDFs, Poppler, and production-DPI renders"]
    fn facsimile_artifact_source_trace() {
        let dir = std::path::Path::new("tmp/pdfs/facsimile_artifact_cleanup");
        std::fs::create_dir_all(dir).unwrap();
        artifact_mask_measurements(dir);
        let bands_json =
            |bands: &[Band]| bands.iter().map(|b| [b.top, b.bottom]).collect::<Vec<_>>();
        for id in [
            "romulus",
            "comparatio_thesei_et_romuli",
            "fabius_maximus",
            "comparatio_periclis_et_fabii",
            "marcius_coriolanus",
        ] {
            let range = operational_range(id);
            let before = std::cell::RefCell::new(Vec::new());
            let (pages, metrics) = unit_body_strips(&range, |source, page| {
                let image = real_source_image(source, page);
                let edges = detect_latin_column(&image).unwrap();
                assert!(edges.left < edges.right && edges.right <= image.width());
                let col = crop_column(&image, &edges);
                let raw = line_bands(&col, LINE_MIN_INK);
                let (_, pass_a) = render_and_band(source, page, &|s, p| {
                    assert_eq!((s, p), (source, page));
                    Ok(image.clone())
                })
                .unwrap();
                col.save(dir.join(format!("{id}-s{source}-p{page:04}-column.png")))
                    .unwrap();
                before.borrow_mut().push(serde_json::json!({
                    "source": source, "page": page, "source_dimensions": image.dimensions(),
                    "crop_edges": [edges.left, edges.right], "raw_bands": bands_json(&raw),
                    "pass_a_bands": bands_json(&pass_a),
                }));
                Ok(image)
            })
            .unwrap();
            assert_eq!(
                pages.len(),
                (range.end_page - range.unit.start_page + 1) as usize
            );
            let geom = TrimGeom::default();
            let widths = pages.iter().map(|p| p.0.width()).collect::<Vec<_>>();
            let scale = global_scale(&widths, geom.content_width());
            let canvases = compose_pages(
                &pages,
                scale,
                &geom,
                geom.first_body_budget(),
                geom.body_budget(),
                &metrics,
            );
            let mut canvas_end = 0u64;
            let canvas_ranges = canvases
                .iter()
                .enumerate()
                .map(|(index, canvas)| {
                    canvas
                        .save(dir.join(format!("{id}-canvas-{index:03}.png")))
                        .unwrap();
                    let start = canvas_end;
                    canvas_end += u64::from(canvas.height());
                    (index, start, canvas_end)
                })
                .collect::<Vec<_>>();
            let mut stream_row = 0u64;
            let strips = strips_in_reading_order(&pages, &metrics).iter().map(|strip| {
                let image = &pages[strip.page_img_index].0;
                assert!(strip.band.top < strip.band.bottom && strip.band.bottom <= image.height());
                let scaled = scale_strip(image, &strip.band, scale).unwrap();
                let start = stream_row;
                stream_row += u64::from(scaled.height());
                serde_json::json!({
                    "source": range.unit.source,
                    "page": range.unit.start_page + strip.page_img_index as u32,
                    "interval": [strip.band.top, strip.band.bottom],
                    "scaled_stream_rows": [start, stream_row],
                    "canvas_indices": canvas_ranges.iter().filter(|(_, lo, hi)| *lo < stream_row && *hi > start).map(|(i, _, _)| *i).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>();
            assert_eq!(
                stream_row, canvas_end,
                "composition preserves the traced row stream"
            );
            let mut records = before.into_inner();
            for (record, (_, bands)) in records.iter_mut().zip(&pages) {
                record["pass_b_bands"] = serde_json::json!(bands_json(bands));
            }
            let path = dir.join(format!("{id}.json"));
            std::fs::write(
                &path,
                serde_json::to_string_pretty(&serde_json::json!({
                    "dpi": BODY_DPI, "scale": scale,
                    "scale_scope": "diagnostic unit column widths",
                    "pages": records, "strips": strips,
                }))
                .unwrap(),
            )
            .unwrap();
            println!("artifact trace: {}", path.display());
        }
    }

    fn artifact_mask_measurements(dir: &std::path::Path) {
        let mut evidence = Vec::new();
        for (page, rectangles) in [
            (
                233,
                vec![
                    [1180, 2870, 1605, 2980],
                    [0, 2980, 1954, 3005],
                    [1920, 0, 1954, 3005],
                ],
            ),
            (
                235,
                vec![
                    [1180, 2880, 1605, 2985],
                    [0, 2985, 1969, 3013],
                    [1920, 0, 1969, 3013],
                ],
            ),
            (240, vec![[550, 210, 1260, 270], [1630, 210, 1730, 275]]),
            (
                284,
                vec![
                    [680, 190, 1150, 245],
                    [1630, 190, 1730, 245],
                    [1970, 0, 2002, 2988],
                ],
            ),
            (56, vec![[946, 2445, 1000, 2630]]),
        ] {
            let original = real_source_image(0, page);
            let column = detect_latin_column(&original).unwrap();
            let raw_col = crop_column(&original, &column);
            let raw = line_bands(&raw_col, LINE_MIN_INK);
            let mut cleaned = original.clone();
            for &[left, top, right, bottom] in &rectangles {
                assert!(left < right && right <= original.width());
                assert!(top < bottom && bottom <= original.height());
                for y in top..bottom {
                    for x in left..right {
                        cleaned.put_pixel(x, y, Luma([255]));
                    }
                }
                image::imageops::crop_imm(&original, left, top, right - left, bottom - top)
                    .to_image()
                    .save(dir.join(format!("s0-p{page:04}-rect-{left}-{top}.png")))
                    .unwrap();
            }
            // Fixed crop edges isolate the effect of the candidate masks on banding.
            let cleaned_col = crop_column(&cleaned, &column);
            let rebanded = line_bands(&cleaned_col, LINE_MIN_INK);
            let kept = drop_furniture(&rebanded, cleaned_col.height());
            let kept = retain_early_aligned_body_bands(&cleaned_col, &rebanded, &kept);
            let metrics = line_metrics(&[(cleaned_col.clone(), kept.clone())]);
            let kept = drop_interior_running_head(&cleaned_col, &kept, &metrics);
            let bands_json =
                |bands: &[Band]| bands.iter().map(|b| [b.top, b.bottom]).collect::<Vec<_>>();
            let gaps = rebanded
                .windows(2)
                .filter_map(|w| {
                    let gap = w[1].top.saturating_sub(w[0].bottom);
                    (gap > 0).then_some(gap)
                })
                .collect::<Vec<_>>();
            cleaned_col
                .save(dir.join(format!("s0-p{page:04}-candidate-masked-column.png")))
                .unwrap();
            evidence.push(serde_json::json!({
                "source": 0, "page": page, "dimensions": original.dimensions(),
                "crop_edges": [column.left, column.right], "candidate_rectangles": rectangles,
                "raw_bands": bands_json(&raw), "masked_raw_bands": bands_json(&rebanded),
                "masked_positive_gap_median": median_u32(&gaps),
                "masked_cut_points": cut_points(&rebanded),
                "masked_retained_bands": bands_json(&kept),
                "masked_retained_cut_points": cut_points(&kept),
            }));
        }
        std::fs::write(
            dir.join("candidate-mask-measurements.json"),
            serde_json::to_string_pretty(&evidence).unwrap(),
        )
        .unwrap();
        let mut components = Vec::new();
        for page in [51, 56, 59] {
            let image = real_source_image(0, page);
            let column = detect_latin_column(&image).unwrap();
            let width = 48u32.min(column.right - column.left);
            let height = image.height();
            let mut visited = vec![false; (width * height) as usize];
            for y in 0..height {
                for x in 0..width {
                    let index = (y * width + x) as usize;
                    if visited[index] || image.get_pixel(column.left + x, y).0[0] > INK_THRESHOLD {
                        continue;
                    }
                    visited[index] = true;
                    let mut pending = vec![(x, y)];
                    let (mut left, mut right, mut top, mut bottom) = (x, x + 1, y, y + 1);
                    while let Some((cx, cy)) = pending.pop() {
                        left = left.min(cx);
                        right = right.max(cx + 1);
                        top = top.min(cy);
                        bottom = bottom.max(cy + 1);
                        for ny in cy.saturating_sub(1)..=(cy + 1).min(height - 1) {
                            for nx in cx.saturating_sub(1)..=(cx + 1).min(width - 1) {
                                let index = (ny * width + nx) as usize;
                                if !visited[index]
                                    && image.get_pixel(column.left + nx, ny).0[0] <= INK_THRESHOLD
                                {
                                    visited[index] = true;
                                    pending.push((nx, ny));
                                }
                            }
                        }
                    }
                    if bottom - top >= 150.max(height / 20) {
                        components.push(serde_json::json!({
                            "source": 0, "page": page, "crop_edges": [column.left, column.right],
                            "component_rectangle": [column.left + left, top, column.left + right, bottom],
                            "touches_search_right_edge": right == width,
                        }));
                    }
                }
            }
        }
        std::fs::write(
            dir.join("divider-components.json"),
            serde_json::to_string_pretty(&components).unwrap(),
        )
        .unwrap();
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn reported_divider_pages_preserve_latin() {
        for page in [51, 56, 59] {
            let img = real_source_image(0, page);
            let cleaned = assert_real_mask_preservation(page);
            if page == 56 {
                let c = detect_latin_column(&cleaned).unwrap();
                let cropped = crop_column(&cleaned, &c);
                let bands = line_bands(&cropped, LINE_MIN_INK);
                for expected in [
                    Band {
                        top: 2470,
                        bottom: 2518,
                    },
                    Band {
                        top: 2525,
                        bottom: 2572,
                    },
                    Band {
                        top: 2577,
                        bottom: 2630,
                    },
                ] {
                    assert!(bands.contains(&expected));
                }
                image::imageops::crop_imm(&cropped, 0, 2445, cropped.width(), 200)
                    .to_image()
                    .save("tmp/pdfs/facsimile_artifact_cleanup/opening-56-production.png")
                    .unwrap();
            }
            let c = detect_latin_column(&img).unwrap();
            let ledger: serde_json::Value =
                serde_json::from_str(include_str!("fixtures/artifact_source_trace.json")).unwrap();
            let expected = ledger["divider_components"]
                .as_array()
                .unwrap()
                .iter()
                .find(|entry| entry["page"].as_u64() == Some(page as u64))
                .unwrap();
            assert_eq!(c.left, expected["crop_edges"][0].as_u64().unwrap() as u32);
            assert_eq!(c.right, expected["crop_edges"][1].as_u64().unwrap() as u32);
            let cropped = crop_column(&img, &c);
            assert!(c.left < c.right);
            assert_eq!(cropped.height(), img.height());
            for y in 0..img.height() {
                for x in c.left..c.right {
                    assert_eq!(img.get_pixel(x, y), cropped.get_pixel(x - c.left, y));
                }
            }
            save_retained_crop(
                &format!("divider-{page}"),
                &(cropped.clone(), line_bands(&cropped, LINE_MIN_INK)),
            );
        }
    }
    fn small_mask() -> SourceFurnitureMask {
        SourceFurnitureMask {
            source: 0,
            page: 233,
            left: 0.2,
            top: 0.4,
            right: 0.4,
            bottom: 0.6,
        }
    }
    #[test]
    fn known_source_furniture_masks_only_matching_page() {
        let img = GrayImage::from_pixel(512, 512, Luma([200]));
        for (source, page) in [(1, 233), (0, 232)] {
            let mut cleaned = img.clone();
            mask_known_source_furniture(source, page, &mut cleaned);
            assert_eq!(cleaned, img);
        }
        let mut cleaned = img.clone();
        mask_known_source_furniture(0, 233, &mut cleaned);
        assert_ne!(cleaned, img);
        let identities: std::collections::BTreeSet<_> = SOURCE_FURNITURE_MASKS
            .iter()
            .map(|m| (m.source, m.page))
            .collect();
        for (source, page) in identities {
            let mut cleaned = img.clone();
            mask_known_source_furniture(source, page, &mut cleaned);
            assert_ne!(cleaned, img, "installed identity {source}/{page}");
            for (other_source, other_page) in [(2, page), (source, 0)] {
                let mut cleaned = img.clone();
                mask_known_source_furniture(other_source, other_page, &mut cleaned);
                assert_eq!(cleaned, img);
            }
        }
    }
    #[test]
    fn known_source_furniture_preserves_pixels_outside_rectangle() {
        let mut img = GrayImage::from_pixel(100, 100, Luma([90]));
        apply_source_furniture_mask(&small_mask(), &mut img);
        for y in 0..100 {
            for x in 0..100 {
                assert_eq!(
                    img.get_pixel(x, y).0[0],
                    if (20..40).contains(&x) && (40..60).contains(&y) {
                        255
                    } else {
                        90
                    }
                );
            }
        }
    }
    #[test]
    fn known_source_furniture_removes_pale_watermark_pixels() {
        let mut img = GrayImage::from_pixel(100, 100, Luma([220]));
        mask_known_source_furniture(0, 233, &mut img);
        assert_eq!(img.get_pixel(70, 97).0[0], 255);
        assert_eq!(img.get_pixel(70, 80).0[0], 220);
    }
    #[test]
    fn source_furniture_mask_bounds_are_safe() {
        for mask in SOURCE_FURNITURE_MASKS {
            assert!(mask.left >= 0.0 && mask.right <= 1.0 && mask.left < mask.right);
            assert!(mask.top >= 0.0 && mask.bottom <= 1.0 && mask.top < mask.bottom);
        }
        let original = GrayImage::from_pixel(10, 10, Luma([0]));
        for (left, right) in [(f64::NAN, 0.5), (0.8, 0.2), (0.3, 0.3)] {
            let mut img = original.clone();
            apply_source_furniture_mask(
                &SourceFurnitureMask {
                    left,
                    right,
                    ..small_mask()
                },
                &mut img,
            );
            assert_eq!(img, original);
        }
        let mut img = original;
        apply_source_furniture_mask(
            &SourceFurnitureMask {
                left: -2.0,
                top: -1.0,
                right: 2.0,
                bottom: 2.0,
                ..small_mask()
            },
            &mut img,
        );
        assert!(img.pixels().all(|p| p.0[0] == 255));
    }
    #[test]
    fn masked_furniture_cannot_return_in_strip_padding() {
        let mut img = column(
            100,
            100,
            &[(10, 20, 0, 90), (50, 60, 0, 90), (80, 90, 0, 90)],
        );
        img.put_pixel(30, 45, Luma([220]));
        let original = img.clone();
        apply_source_furniture_mask(
            &SourceFurnitureMask {
                bottom: 0.5,
                ..small_mask()
            },
            &mut img,
        );
        let bands = line_bands(&img, LINE_MIN_INK);
        let pages = vec![(img, bands)];
        let m = line_metrics(&pages);
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 100,
            tb_h: 200,
            pad_x: 0,
            pad_y: 0,
        };
        let output = compose_pages(&pages, 1.0, &geom, 200, 200, &m);
        assert_eq!(output.len(), 1);
        assert!(output[0].pixels().all(|p| p.0[0] != 220));
        assert_eq!(output[0].get_pixel(30, 35).0[0], 255);
        for y in [10..20, 50..60, 80..90].into_iter().flatten() {
            for x in 0..100 {
                assert_eq!(output[0].get_pixel(x, y - 10), original.get_pixel(x, y));
            }
        }
    }

    /// Compare full-resolution pixels, including faint type, outside verified masks.
    fn assert_real_mask_preservation(page: u32) -> GrayImage {
        let original = real_source_image(0, page);
        let mut cleaned = original.clone();
        mask_known_source_furniture(0, page, &mut cleaned);
        let mut erased = vec![false; (original.width() * original.height()) as usize];
        for mask in SOURCE_FURNITURE_MASKS
            .iter()
            .filter(|m| m.source == 0 && m.page == page)
        {
            let x = |v: f64| (v * original.width() as f64).round() as u32;
            let y = |v: f64| (v * original.height() as f64).round() as u32;
            for yy in y(mask.top)..y(mask.bottom) {
                for xx in x(mask.left)..x(mask.right) {
                    erased[(yy * original.width() + xx) as usize] = true;
                }
            }
        }
        for (i, (before, after)) in original.pixels().zip(cleaned.pixels()).enumerate() {
            if erased[i] {
                assert_eq!(after.0[0], 255);
            } else {
                assert_eq!(before, after, "page {page} pixel {i}");
            }
        }
        let dir = std::path::Path::new("tmp/pdfs/facsimile_artifact_cleanup");
        std::fs::create_dir_all(dir).unwrap();
        cleaned
            .save(dir.join(format!("s0-p{page:04}-production-cleaned.png")))
            .unwrap();
        cleaned
    }
    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn reported_watermark_pages_preserve_adjacent_lines() {
        let range = operational_range("fabius_maximus");
        let (unit_pages, metrics) =
            unit_body_strips(&range, |s, p| Ok(real_source_image(s, p))).unwrap();
        let unit_strips = strips_in_reading_order(&unit_pages, &metrics);
        for (page, end, next, first) in [
            (
                233,
                2624,
                234,
                Band {
                    top: 266,
                    bottom: 306,
                },
            ),
            (
                235,
                2622,
                236,
                Band {
                    top: 270,
                    bottom: 301,
                },
            ),
        ] {
            let cleaned = assert_real_mask_preservation(page);
            let (col, bands) = render_and_band(0, page, &|_, _| Ok(cleaned.clone())).unwrap();
            assert_eq!(bands.last().unwrap().bottom, end);
            assert_eq!(*cut_points(&bands).last().unwrap(), end);
            let (_, following) =
                render_and_band(0, next, &|s, p| Ok(real_source_image(s, p))).unwrap();
            assert!(following.contains(&first));
            assert!(col.height() > end);
            let index = (page - range.unit.start_page) as usize;
            let current = &unit_pages[index];
            let following = &unit_pages[index + 1];
            assert_eq!(current.1.last().unwrap().bottom, end);
            assert!(following.1.contains(&first));
            let copied = unit_strips
                .iter()
                .filter(|s| s.page_img_index == index)
                .collect::<Vec<_>>();
            assert_eq!(copied.last().unwrap().band.bottom, end);
            assert!(copied.iter().all(|s| s.band.bottom <= end));
            let old_end = if page == 233 { 2953 } else { 3013 };
            assert_eq!(old_end - end, if page == 233 { 329 } else { 391 });
            let gaps = current
                .1
                .windows(2)
                .map(|w| w[1].top - w[0].bottom)
                .filter(|&g| g > 0)
                .collect::<Vec<_>>();
            assert_eq!(median_u32(&gaps), if page == 233 { 14 } else { 18 });
            let original_next = real_source_image(0, next);
            let next_column = detect_latin_column(&original_next).unwrap();
            for y in first.top..first.bottom {
                for x in 0..following.0.width() {
                    assert_eq!(
                        following.0.get_pixel(x, y),
                        original_next.get_pixel(x + next_column.left, y)
                    );
                }
            }
            let seam_pages = vec![
                (
                    current.0.clone(),
                    current
                        .1
                        .iter()
                        .rev()
                        .take(2)
                        .copied()
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect(),
                ),
                (
                    following.0.clone(),
                    following.1.iter().take(2).copied().collect(),
                ),
            ];
            let geom = TrimGeom {
                dpi: 300,
                tb_w: current.0.width().max(following.0.width()),
                tb_h: 2000,
                pad_x: 0,
                pad_y: 0,
            };
            let seam = assert_scaled_stream(&seam_pages, &metrics, 1.0, &geom, 2000, 2000);
            let dir = std::path::Path::new("tmp/pdfs/facsimile_artifact_cleanup");
            seam[0]
                .save(dir.join(format!("watermark-{page}-{next}-after-seam.png")))
                .unwrap();
            let original = real_source_image(0, page);
            let original_column = detect_latin_column(&original).unwrap();
            let before_pages = vec![
                (
                    crop_column(&original, &original_column),
                    vec![Band {
                        top: end - 100,
                        bottom: old_end,
                    }],
                ),
                (following.0.clone(), vec![first]),
            ];
            let before = compose_pages(&before_pages, 1.0, &geom, 2000, 2000, &metrics);
            before[0]
                .save(dir.join(format!("watermark-{page}-{next}-before-seam.png")))
                .unwrap();
            println!(
                "source {page}: endpoint {old_end} -> {end}, {} rows removed, zero footer padding",
                old_end - end
            );
        }
    }
    fn assert_header_units(page: u32, ids: &[&str], first: Band) {
        assert_real_mask_preservation(page);
        for id in ids {
            let range = operational_range(id);
            let (pages, _) = unit_body_strips(&range, |s, p| Ok(real_source_image(s, p))).unwrap();
            let retained = &pages[(page - range.unit.start_page) as usize];
            assert!(retained.1.iter().all(|b| b.top >= first.top));
            if page != range.unit.start_page {
                assert!(retained.1.contains(&first));
            }
            save_retained_crop(&format!("{id}-header-{page}"), retained);
        }
    }
    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn fabius_shared_page_header_is_excluded_from_both_units() {
        assert_header_units(
            240,
            &["fabius_maximus", "comparatio_periclis_et_fabii"],
            Band {
                top: 291,
                bottom: 327,
            },
        );
    }
    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn coriolanus_source_284_header_is_excluded() {
        assert_header_units(
            284,
            &["marcius_coriolanus"],
            Band {
                top: 270,
                bottom: 302,
            },
        );
    }
    fn masked_tail_fixture(
        bridge_width: u32,
        watermark_shade: u8,
    ) -> (GrayImage, Vec<PageBands>, LineMetrics) {
        let mut raw = column(
            400,
            200,
            &[
                (60, 80, 20, 170),
                (100, 120, 20, 170),
                (170, 190, 20, 170),
                (220, 240, 20, 170),
            ],
        );
        for y in 220..400 {
            for x in 200 - bridge_width..200 {
                raw.put_pixel(x, y, Luma([0]));
            }
        }
        for y in 390..400 {
            for x in 0..200 {
                raw.put_pixel(x, y, Luma([0]));
            }
        }
        for y in 280..300 {
            for x in 100..160 {
                raw.put_pixel(x, y, Luma([watermark_shade]));
            }
        }
        let mut cleaned = raw.clone();
        for mask in [
            SourceFurnitureMask {
                left: 0.98,
                top: 0.0,
                right: 1.0,
                bottom: 1.0,
                ..small_mask()
            },
            SourceFurnitureMask {
                left: 0.0,
                top: 0.975,
                right: 1.0,
                bottom: 1.0,
                ..small_mask()
            },
            SourceFurnitureMask {
                left: 0.5,
                top: 0.7,
                right: 0.8,
                bottom: 0.75,
                ..small_mask()
            },
        ] {
            apply_source_furniture_mask(&mask, &mut cleaned);
        }
        let bands = line_bands(&cleaned, LINE_MIN_INK);
        let kept = drop_furniture(&bands, cleaned.height());
        let bands = retain_early_aligned_body_bands(&cleaned, &bands, &kept);
        let pages = vec![(cleaned, bands)];
        let m = line_metrics(&pages);
        (raw, pages, m)
    }
    #[test]
    fn masked_tail_cut_points_exclude_furniture_rows() {
        for (bridge_width, shade) in [(4, 220), (2, 230), (3, 210)] {
            let (raw, pages, m) = masked_tail_fixture(bridge_width, shade);
            assert_eq!(line_bands(&raw, LINE_MIN_INK).last().unwrap().bottom, 400);
            assert_eq!(
                pages[0].1,
                vec![
                    Band {
                        top: 60,
                        bottom: 80
                    },
                    Band {
                        top: 100,
                        bottom: 120
                    },
                    Band {
                        top: 170,
                        bottom: 190
                    },
                    Band {
                        top: 220,
                        bottom: 240
                    }
                ]
            );
            let strips = strips_in_reading_order(&pages, &m);
            assert_eq!(strips.last().unwrap().band.bottom, 240);
            assert_eq!(
                strips
                    .iter()
                    .map(|s| s.band.bottom - s.band.top)
                    .sum::<u32>(),
                180
            );
        }
    }
    #[test]
    fn watermark_cleanup_removes_pixels_and_space() {
        let (raw, pages, m) = masked_tail_fixture(4, 220);
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 200,
            tb_h: 500,
            pad_x: 0,
            pad_y: 0,
        };
        let output = compose_pages(&pages, 1.0, &geom, 500, 500, &m);
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].height(), 180);
        assert_eq!(400 - 60 - output[0].height(), 160);
        assert!(output[0].pixels().all(|p| p.0[0] != 220));
        for y in 60..240 {
            for x in 0..196 {
                assert_eq!(output[0].get_pixel(x, y - 60), raw.get_pixel(x, y));
            }
        }
    }
    #[test]
    fn watermark_cleanup_preserves_unrelated_paragraph_gap() {
        let (raw, pages, m) = masked_tail_fixture(4, 220);
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 200,
            tb_h: 500,
            pad_x: 0,
            pad_y: 0,
        };
        let output = compose_pages(&pages, 1.0, &geom, 500, 500, &m);
        assert_eq!(pages[0].1[2].top - pages[0].1[1].bottom, 50);
        for y in 120..170 {
            for x in 0..196 {
                assert_eq!(output[0].get_pixel(x, y - 60), raw.get_pixel(x, y));
            }
        }
    }
    fn operational_range(id: &str) -> UnitRange {
        let units =
            crate::facsimile::units::load_units(std::path::Path::new("facsimile_units.json"))
                .unwrap();
        crate::facsimile::units::ranges(&units, &[638, 711])
            .into_iter()
            .find(|r| r.unit.id == id)
            .unwrap()
    }

    fn real_source_image(source: usize, page: u32) -> GrayImage {
        static POPPLER: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let pdf = std::path::PathBuf::from(format!("Ploutarchou_Bioi_v{}.pdf", source + 1));
        assert!(pdf.is_file(), "required source PDF {}", pdf.display());
        assert!(
            *POPPLER.get_or_init(crate::render::pdftotext_available),
            "Poppler is required"
        );
        let cache = std::path::Path::new("build_facsimile/render_cache");
        let png = cache.join(format!(
            "Ploutarchou_Bioi_v{}-p{page:04}-{BODY_DPI}dpi.png",
            source + 1
        ));
        let png = if png.exists() {
            png
        } else {
            crate::render::render_page(&pdf, page, BODY_DPI, cache).expect("production-DPI render")
        };
        image::open(png).expect("decode source raster").to_luma8()
    }

    /// Diagnostic (not a regression): band every Tier-B dropped-line source page and
    /// print the raw `line_bands` output near the audit box, with ink-span, survival
    /// through `render_and_band`, and a saved crop per candidate band. Supplies the
    /// `(dimensions, observed)` coordinates for `SOURCE_BODY_BANDS` repair entries.
    #[test]
    #[ignore]
    fn extract_tier_b_source_bands() {
        measure_tier_b_source_bands();
    }

    fn save_retained_crop(id: &str, page: &PageBands) {
        let dir = std::path::Path::new("tmp/pdfs/recto_units_verification/crops");
        std::fs::create_dir_all(dir).unwrap();
        let first = page.1.first().expect("retained body").top;
        let last = page.1.last().unwrap().bottom;
        let path = dir.join(format!("{id}.png"));
        image::imageops::crop_imm(&page.0, 0, first, page.0.width(), last - first)
            .to_image()
            .save(&path)
            .unwrap();
        println!("retained crop: {}", path.display());
    }

    /// A test-only record of one source page after the inner-tomi repairs (duplicate
    /// exclusion, furniture masking, and interior-whitespace compaction). Every row is
    /// an **original** (full-source) coordinate; the retained-image row is derived with
    /// [`RetainedSourcePageTrace::map_row`]. A skipped page (a verified whole-page
    /// duplicate) has `retained_index: None` and contributes no retained image, so its
    /// index must never be reconstructed as `page - start_page`.
    #[derive(Clone, serde::Serialize)]
    struct RetainedSourcePageTrace {
        source: usize,
        page: u32,
        /// Position in the retained `pages` vector, or `None` for a skipped page.
        retained_index: Option<usize>,
        /// Original `[left, right)` Latin-column crop edges.
        crop: [u32; 2],
        /// Retained body bands in original-source row coordinates.
        bands: Vec<[u32; 2]>,
        /// Half-open `[top, bottom)` row intervals deleted by masking/compaction, in
        /// original-source coordinates. Must be sorted and non-overlapping.
        deleted_rows: Vec<[u32; 2]>,
    }

    impl RetainedSourcePageTrace {
        /// Map an original-source row to its row in the retained (compacted) image, or
        /// `None` when the row itself was deleted. Subtracts the total height of every
        /// deletion interval ending at or before `row`; a row inside a deletion
        /// interval has no mapped coordinate. `deleted_rows` must be sorted ascending.
        fn map_row(&self, row: u32) -> Option<u32> {
            let mut shift = 0u32;
            for &[top, bottom] in &self.deleted_rows {
                if bottom <= row {
                    shift += bottom - top;
                } else if top <= row {
                    return None;
                } else {
                    break;
                }
            }
            Some(row - shift)
        }
    }

    /// Look a retained page up by identity. The one shared lookup used by the
    /// duplication, mask, and compaction regressions — never `page - start_page`.
    fn retained_page(
        traces: &[RetainedSourcePageTrace],
        source: usize,
        page: u32,
    ) -> Option<&RetainedSourcePageTrace> {
        traces.iter().find(|t| t.source == source && t.page == page)
    }

    #[test]
    fn repair_trace_maps_skipped_pages_and_deleted_rows() {
        let traces = vec![
            // A retained page with no deletions: an identity mapping.
            RetainedSourcePageTrace {
                source: 1,
                page: 17,
                retained_index: Some(0),
                crop: [100, 900],
                bands: vec![[200, 240]],
                deleted_rows: vec![],
            },
            // A verified duplicate whole page, skipped: no retained index at all.
            RetainedSourcePageTrace {
                source: 1,
                page: 21,
                retained_index: None,
                crop: [100, 900],
                bands: vec![],
                deleted_rows: vec![],
            },
            // An interior watermark band and a footer tail deleted in original coords.
            RetainedSourcePageTrace {
                source: 0,
                page: 227,
                retained_index: Some(1),
                crop: [1040, 1950],
                bands: vec![[300, 340], [1800, 1840]],
                deleted_rows: vec![[1500, 1700], [2600, 2900]],
            },
        ];
        // A skipped page is found by identity and carries no retained index.
        assert_eq!(retained_page(&traces, 1, 21).unwrap().retained_index, None);
        // Identity mapping for a page with no deletions.
        let clean = retained_page(&traces, 1, 17).unwrap();
        assert_eq!(clean.map_row(220), Some(220));
        // Deleted-row semantics on the masked page.
        let p = retained_page(&traces, 0, 227).unwrap();
        assert_eq!(p.map_row(300), Some(300));
        assert_eq!(p.map_row(1499), Some(1499));
        assert_eq!(p.map_row(1500), None, "inclusive top of a deletion");
        assert_eq!(p.map_row(1600), None);
        assert_eq!(p.map_row(1699), None);
        assert_eq!(p.map_row(1700), Some(1500), "shift by the 200-row deletion");
        assert_eq!(p.map_row(1800), Some(1600));
        assert_eq!(p.map_row(2599), Some(2399));
        assert_eq!(p.map_row(2700), None, "inside the second deletion");
        assert_eq!(p.map_row(2900), Some(2400), "shift by 200 + 300 rows");
        // Every field participates (crop/bands are read by the evidence tracer).
        assert_eq!(p.crop, [1040, 1950]);
        assert_eq!(p.bands.len(), 2);
        assert_eq!(p.page, 227);
    }

    #[test]
    fn inner_tomi_repair_ledger_has_expected_shape() {
        let ledger: serde_json::Value =
            serde_json::from_str(include_str!("fixtures/inner_tomi_repair_source_trace.json"))
                .expect("committed repair ledger parses");
        assert!(ledger["description"].is_string());
        assert!(ledger["schema"]["repair_record"].is_array());
        assert!(
            ledger["repairs"].is_array(),
            "repairs is the list of reviewed, frozen repair records"
        );
        // Every present repair record must carry the mandatory diagnostic keys, so a
        // guessed or half-filled record cannot slip in unnoticed.
        for repair in ledger["repairs"].as_array().unwrap() {
            for key in [
                "finding_ids",
                "unit_id",
                "source",
                "source_page",
                "disposition",
                "preservation_boundary",
            ] {
                assert!(!repair[key].is_null(), "repair record missing {key}");
            }
        }
    }

    fn repair_ledger() -> serde_json::Value {
        serde_json::from_str(include_str!("fixtures/inner_tomi_repair_source_trace.json")).unwrap()
    }

    fn backlog_repair_ledger() -> serde_json::Value {
        serde_json::from_str(include_str!("fixtures/backlog_artifact_source_trace.json")).unwrap()
    }

    fn tomi_gap_split_repair_ledger() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "fixtures/tomi_gap_split_repairs_source_trace.json"
        ))
        .unwrap()
    }

    /// A JSON value is a lowercase/uppercase 64-character SHA-256 hex digest.
    fn ledger_is_hex64(value: &serde_json::Value) -> bool {
        value
            .as_str()
            .is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
    }

    /// A JSON value is an array of exactly `n` non-negative integers.
    fn ledger_is_u_array(value: &serde_json::Value, n: usize) -> bool {
        value
            .as_array()
            .is_some_and(|a| a.len() == n && a.iter().all(|v| v.as_u64().is_some()))
    }

    #[test]
    fn tomi_gap_split_ledger_has_expected_shape() {
        let ledger = tomi_gap_split_repair_ledger();
        assert_eq!(ledger["schema_version"], 1);
        assert!(ledger["description"]
            .as_str()
            .is_some_and(|d| !d.is_empty()));
        let repairs = ledger["repairs"].as_array().unwrap();
        assert!(!repairs.is_empty());
        for repair in repairs {
            // Every installed record shares source identity, raw dimensions, and
            // reviewed Latin regions regardless of disposition.
            assert!(repair["unit_id"].as_str().is_some_and(|id| !id.is_empty()));
            assert!(matches!(repair["source"].as_u64(), Some(0 | 1)));
            assert!(repair["source_page"].as_u64().is_some());
            assert!(ledger_is_u_array(&repair["raster"], 2));
            assert!(repair["preserved_latin_regions"]
                .as_array()
                .is_some_and(|regions| !regions.is_empty()));

            match repair["disposition"].as_str().unwrap() {
                "mask" => {
                    assert_eq!(repair["status"], "repaired");
                    assert!(repair["backlog_id"]
                        .as_str()
                        .is_some_and(|id| !id.is_empty()));
                    let rects = repair["mask_rectangles"].as_array().unwrap();
                    assert!(!rects.is_empty());
                    for rect in rects {
                        let r: Vec<f64> = rect
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v.as_f64().unwrap())
                            .collect();
                        assert_eq!(r.len(), 4);
                        assert!(r[0] >= 0.0 && r[2] <= 1.0 && r[0] < r[2]);
                        assert!(r[1] >= 0.0 && r[3] <= 1.0 && r[1] < r[3]);
                    }
                    // Fractions are raw full-source coordinates unless marked sheared.
                    if let Some(space) = repair["coordinate_space"].as_str() {
                        assert!(matches!(space, "raw" | "sheared"));
                        if space == "sheared" {
                            assert!(ledger_is_u_array(&repair["sheared_raster"], 2));
                        }
                    }
                }
                "shear" => {
                    assert_eq!(repair["status"], "repaired");
                    assert!(ledger_is_hex64(&repair["fingerprint"]));
                    assert!(ledger_is_u_array(&repair["fingerprint_region"], 4));
                    let gaps = &repair["matched_gaps"];
                    assert!(gaps["left_x"].as_u64().is_some());
                    assert!(gaps["right_x"].as_u64().is_some());
                    assert!(ledger_is_u_array(&gaps["left_rows"], 2));
                    assert!(ledger_is_u_array(&gaps["right_rows"], 2));
                    assert!(repair["x0_px"].as_u64().is_some());
                    assert!(repair["baseline_slope"].as_f64().is_some());
                    assert_eq!(repair["shift_range"].as_array().unwrap().len(), 2);
                    assert!(repair["top_pad"].as_u64().is_some());
                    assert!(ledger_is_u_array(&repair["crop_edges"], 2));
                    assert!(repair["expected_bands"]
                        .as_array()
                        .is_some_and(|bands| !bands.is_empty()
                            && bands.iter().all(|band| ledger_is_u_array(band, 2))));
                }
                "row_excision" => {
                    assert_eq!(repair["status"], "repaired");
                    assert_eq!(repair["coordinate_space"], "sheared");
                    assert!(ledger_is_u_array(&repair["sheared_raster"], 2));
                    assert!(ledger_is_u_array(&repair["deleted_rows"], 2));
                    assert!(ledger_is_u_array(&repair["column_dims"], 2));
                    let top = repair["top"].as_f64().unwrap();
                    let bottom = repair["bottom"].as_f64().unwrap();
                    assert!(0.0 <= top && top < bottom && bottom <= 1.0);
                    assert!(repair["gap_before_rows"].as_u64().is_some());
                    assert!(repair["gap_after_rows"].as_u64().is_some());
                }
                "whole_strip" => {
                    assert_eq!(repair["status"], "repaired");
                    assert!(ledger_is_u_array(&repair["column_dims"], 2));
                    assert!(ledger_is_u_array(&repair["composer_strip"], 2));
                    assert!(ledger_is_u_array(&repair["strip_fingerprint_region"], 4));
                    assert!(ledger_is_hex64(&repair["strip_fingerprint"]));
                }
                "source_body_band" => {
                    assert_eq!(repair["status"], "repaired");
                    assert!(ledger_is_u_array(&repair["column_dims"], 2));
                    assert!(ledger_is_u_array(&repair["observed"], 2));
                    assert!(ledger_is_u_array(&repair["protected"], 2));
                }
                "word_patch" => {
                    assert_eq!(repair["status"], "repaired");
                    assert!(ledger_is_u_array(&repair["src_rect"], 4));
                    assert!(ledger_is_u_array(&repair["dest_anchor"], 2));
                    if !repair["excise_rect"].is_null() {
                        assert!(ledger_is_u_array(&repair["excise_rect"], 4));
                    }
                    assert!(ledger_is_hex64(&repair["fingerprint"]));
                }
                "source_spacing_retained" => {
                    // Not a repair: a measured, justified decision to leave the gap.
                    assert!(repair["source_gap_px"].as_array().is_some());
                    assert!(!repair["reader_gap_px"].is_null());
                    assert!(repair["reason"].as_str().is_some_and(|r| !r.is_empty()));
                }
                other => panic!("unknown disposition {other}"),
            }
        }
    }

    /// Reviewed rectangles grouped by source identity, including separate repairs on one page.
    fn reviewed_page_masks(
        ledger: &serde_json::Value,
    ) -> std::collections::BTreeMap<(usize, u32), Vec<[f64; 4]>> {
        let mut pages = std::collections::BTreeMap::new();
        for repair in ledger["repairs"].as_array().unwrap() {
            if repair["disposition"] != "mask"
                || repair["coordinate_space"] == "sheared"
                || (repair["status"].is_string() && repair["status"] != "repaired")
            {
                continue;
            }
            let identity = (
                repair["source"].as_u64().unwrap() as usize,
                repair["source_page"].as_u64().unwrap() as u32,
            );
            let rectangles: &mut Vec<[f64; 4]> = pages.entry(identity).or_default();
            for rectangle in repair["mask_rectangles"].as_array().unwrap() {
                rectangles.push(std::array::from_fn(|i| rectangle[i].as_f64().unwrap()));
            }
        }
        pages
    }

    fn all_reviewed_page_masks(
        ledgers: &[&serde_json::Value],
    ) -> std::collections::BTreeMap<(usize, u32), Vec<[f64; 4]>> {
        let mut pages = std::collections::BTreeMap::new();
        let mut rasters = std::collections::BTreeMap::new();
        for ledger in ledgers {
            for repair in ledger["repairs"].as_array().unwrap() {
                if repair["disposition"] != "mask"
                    || repair["coordinate_space"] == "sheared"
                    || (repair["status"].is_string() && repair["status"] != "repaired")
                {
                    continue;
                }
                let identity = (
                    repair["source"].as_u64().unwrap() as usize,
                    repair["source_page"].as_u64().unwrap() as u32,
                );
                let raster = [
                    repair["raster"][0].as_u64().unwrap(),
                    repair["raster"][1].as_u64().unwrap(),
                ];
                if let Some(previous) = rasters.insert(identity, raster) {
                    assert_eq!(
                        previous, raster,
                        "conflicting reviewed raster for {identity:?}"
                    );
                }
                if repair["status"] == "repaired" {
                    assert!(repair["preserved_latin_regions"]
                        .as_array()
                        .is_some_and(|regions| !regions.is_empty()));
                }
            }
            for (identity, rectangles) in reviewed_page_masks(ledger) {
                let entry: &mut Vec<[f64; 4]> = pages.entry(identity).or_default();
                for rectangle in rectangles {
                    if !entry.contains(&rectangle) {
                        entry.push(rectangle);
                    }
                }
            }
        }
        pages
    }

    #[test]
    fn backlog_mask_union_includes_shared_page_repairs() {
        let historical = serde_json::json!({"repairs":[
            {"source":0,"source_page":17,"raster":[100,200],"disposition":"mask",
             "mask_rectangles":[[0.1,0.1,0.2,0.2]]}
        ]});
        let backlog = serde_json::json!({"repairs":[
            {"source":0,"source_page":17,"raster":[100,200],"disposition":"mask",
             "status":"repaired","preserved_latin_regions":[[30,30,40,40]],
             "mask_rectangles":[[0.1,0.1,0.2,0.2],[0.3,0.3,0.4,0.4]]},
            {"source":1,"source_page":4,"raster":[50,60],"disposition":"mask",
             "status":"repaired","preserved_latin_regions":[[10,10,20,20]],
             "mask_rectangles":[[0.5,0.5,0.6,0.6]]}
        ]});
        let masks = all_reviewed_page_masks(&[&historical, &backlog]);
        assert_eq!(masks[&(0, 17)].len(), 2);
        assert_eq!(masks[&(1, 4)].len(), 1);
        let conflict = serde_json::json!({"repairs":[
            {"source":0,"source_page":17,"raster":[101,200],"disposition":"mask",
             "status":"repaired","preserved_latin_regions":[[30,30,40,40]],
             "mask_rectangles":[[0.3,0.3,0.4,0.4]]}
        ]});
        assert!(
            std::panic::catch_unwind(|| all_reviewed_page_masks(&[&historical, &conflict]))
                .is_err()
        );
    }

    #[test]
    fn backlog_mask_union_excludes_uninstalled_repairs() {
        let backlog = serde_json::json!({"repairs":[
            {"source":0,"source_page":17,"raster":[100,200],"disposition":"mask",
             "status":"planned","mask_rectangles":[[0.1,0.1,0.2,0.2]]},
            {"source":0,"source_page":18,"raster":[100,200],"disposition":"mask",
             "status":"unresolved","mask_rectangles":[[0.1,0.1,0.2,0.2]]}
        ]});
        assert!(all_reviewed_page_masks(&[&backlog]).is_empty());
        let missing_preservation = serde_json::json!({"repairs":[
            {"source":0,"source_page":17,"raster":[100,200],"disposition":"mask",
             "status":"repaired","mask_rectangles":[[0.1,0.1,0.2,0.2]]}
        ]});
        assert!(
            std::panic::catch_unwind(|| all_reviewed_page_masks(&[&missing_preservation])).is_err()
        );
    }

    fn current_reviewed_page_expectation(
        backlog: &serde_json::Value,
        source: usize,
        page: u32,
        historical: &serde_json::Value,
        field: &str,
    ) -> serde_json::Value {
        let entries: Vec<_> = backlog["current_page_expectations"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|entry| {
                entry["source"].as_u64() == Some(source as u64)
                    && entry["source_page"].as_u64() == Some(page as u64)
            })
            .collect();
        assert!(
            entries.len() <= 1,
            "conflicting current expectations for {source}:{page}"
        );
        let installed = backlog["repairs"].as_array().unwrap().iter().any(|repair| {
            repair["source"].as_u64() == Some(source as u64)
                && repair["source_page"].as_u64() == Some(page as u64)
                && repair["disposition"] == "mask"
                && repair["status"] == "repaired"
        });
        if installed {
            let entry = entries
                .first()
                .expect("installed mask needs current page expectations");
            assert!(entry["expected_crop_edges"].is_array());
            assert!(entry["expected_pass_a_bands"].is_array());
        }
        entries
            .first()
            .map(|entry| &entry[field])
            .filter(|value| !value.is_null())
            .cloned()
            .unwrap_or_else(|| historical[field].clone())
    }

    #[test]
    fn backlog_page_expectations_reject_conflicts() {
        let historical = serde_json::json!({"expected_crop_edges":[10,90],
            "expected_pass_a_bands":[[20,30]],"expected_first_band":[20,30]});
        let current = serde_json::json!({"repairs":[{"source":0,"source_page":17,
            "disposition":"mask","status":"repaired"}],
            "current_page_expectations":[{"source":0,"source_page":17,
            "expected_crop_edges":[11,90],"expected_pass_a_bands":[[21,31]],
            "expected_first_band":[21,31]}]});
        assert_eq!(
            current_reviewed_page_expectation(&current, 0, 17, &historical, "expected_crop_edges"),
            serde_json::json!([11, 90])
        );
        assert_eq!(
            current_reviewed_page_expectation(&current, 0, 17, &historical, "expected_first_band"),
            serde_json::json!([21, 31])
        );
        assert_eq!(
            current_reviewed_page_expectation(&current, 0, 18, &historical, "expected_first_band"),
            serde_json::json!([20, 30])
        );
        let duplicate = serde_json::json!({"repairs":[],"current_page_expectations":[
            {"source":0,"source_page":17},{"source":0,"source_page":17}]});
        assert!(
            std::panic::catch_unwind(|| current_reviewed_page_expectation(
                &duplicate,
                0,
                17,
                &historical,
                "expected_first_band"
            ))
            .is_err()
        );
        let missing = serde_json::json!({"repairs":[{"source":0,"source_page":17,
            "disposition":"mask","status":"repaired"}],"current_page_expectations":[]});
        assert!(
            std::panic::catch_unwind(|| current_reviewed_page_expectation(
                &missing,
                0,
                17,
                &historical,
                "expected_first_band"
            ))
            .is_err()
        );
    }

    #[test]
    fn known_duplicate_page_exclusions_are_identity_scoped() {
        let ledger = repair_ledger();
        for repair in ledger["repairs"].as_array().unwrap() {
            if repair["disposition"] != "duplicate_page" {
                continue;
            }
            let id = repair["unit_id"].as_str().unwrap();
            let source = repair["source"].as_u64().unwrap() as usize;
            for page in repair["skip_pages"].as_array().unwrap() {
                let page = page.as_u64().unwrap() as u32;
                assert!(skip_known_duplicate_source_page(id, source, page));
                assert!(!skip_known_duplicate_source_page(id, 0, page));
                assert!(!skip_known_duplicate_source_page("unrelated", source, page));
            }
            for page in repair["retain_sequence"].as_array().unwrap() {
                assert!(!skip_known_duplicate_source_page(
                    id,
                    source,
                    page.as_u64().unwrap() as u32
                ));
            }
        }
    }

    #[test]
    fn inner_tomi_masks_match_reviewed_ledger() {
        let historical = repair_ledger();
        let backlog = backlog_repair_ledger();
        let tomi = tomi_gap_split_repair_ledger();
        for ((source, page), rectangles) in all_reviewed_page_masks(&[&historical, &backlog, &tomi])
        {
            let record = [&historical, &backlog, &tomi]
                .into_iter()
                .flat_map(|ledger| ledger["repairs"].as_array().unwrap())
                .find(|r| {
                    r["disposition"] == "mask"
                        && r["source"].as_u64() == Some(source as u64)
                        && r["source_page"].as_u64() == Some(page as u64)
                })
                .unwrap();
            let raster = [
                record["raster"][0].as_u64().unwrap(),
                record["raster"][1].as_u64().unwrap(),
            ];
            let pixels = |rect: [f64; 4]| -> [u32; 4] {
                std::array::from_fn(|i| (rect[i] * raster[i % 2] as f64).round() as u32)
            };
            let masks: Vec<_> = SOURCE_FURNITURE_MASKS
                .iter()
                .filter(|mask| mask.source == source && mask.page == page)
                .collect();
            assert_eq!(masks.len(), rectangles.len());
            for rectangle in rectangles {
                assert!(
                    masks.iter().any(|mask| {
                        let actual = [mask.left, mask.top, mask.right, mask.bottom];
                        actual
                            .iter()
                            .zip(rectangle)
                            .all(|(a, b)| (a - b).abs() <= 4.0 * f64::EPSILON)
                            && pixels(actual) == pixels(rectangle)
                    }),
                    "source {source} page {page}: every reviewed rectangle is installed"
                );
            }
        }
    }

    #[test]
    fn inner_tomi_sheared_masks_match_reviewed_ledger() {
        let ledger = tomi_gap_split_repair_ledger();
        for record in repairs_with_disposition(&ledger, "mask")
            .into_iter()
            .filter(|record| record["coordinate_space"] == "sheared")
        {
            let source = record["source"].as_u64().unwrap() as usize;
            let page = record["source_page"].as_u64().unwrap() as u32;
            let dims = (
                record["sheared_raster"][0].as_u64().unwrap() as u32,
                record["sheared_raster"][1].as_u64().unwrap() as u32,
            );
            let shear = SOURCE_SHEARS
                .iter()
                .find(|entry| entry.source == source && entry.page == page)
                .expect("sheared mask needs an installed source shear");
            let (shifts, top_pad) =
                source_shear_shifts(shear.raw_dims.0, shear.x0_px, shear.baseline_slope);
            let bottom_pad = u32::try_from(shifts.into_iter().max().unwrap_or(0).max(0)).unwrap();
            assert_eq!(
                dims,
                (shear.raw_dims.0, shear.raw_dims.1 + top_pad + bottom_pad)
            );
            let masks: Vec<_> = SOURCE_SHEARED_FURNITURE_MASKS
                .iter()
                .filter(|mask| mask.source == source && mask.page == page)
                .collect();
            let rectangles = record["mask_rectangles"].as_array().unwrap();
            assert_eq!(masks.len(), rectangles.len());
            for rectangle in rectangles {
                let expected: [f64; 4] = std::array::from_fn(|i| rectangle[i].as_f64().unwrap());
                let pixels = |r: [f64; 4]| -> [u32; 4] {
                    std::array::from_fn(|i| {
                        (r[i] * f64::from(if i % 2 == 0 { dims.0 } else { dims.1 })).round() as u32
                    })
                };
                assert!(masks.iter().any(|mask| {
                    let actual = [mask.left, mask.top, mask.right, mask.bottom];
                    actual
                        .iter()
                        .zip(expected)
                        .all(|(a, b)| (a - b).abs() <= 4.0 * f64::EPSILON)
                        && pixels(actual) == pixels(expected)
                }));
            }
        }
    }

    /// Installed `repairs` records of one disposition across the shared ledger.
    fn repairs_with_disposition<'a>(
        ledger: &'a serde_json::Value,
        disposition: &str,
    ) -> Vec<&'a serde_json::Value> {
        ledger["repairs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["disposition"] == disposition && r["status"] == "repaired")
            .collect()
    }

    #[test]
    fn inner_tomi_shears_match_reviewed_ledger() {
        let tomi = tomi_gap_split_repair_ledger();
        let records = repairs_with_disposition(&tomi, "shear");
        // Every reviewed shear record is installed with identical geometry.
        for record in &records {
            let source = record["source"].as_u64().unwrap() as usize;
            let page = record["source_page"].as_u64().unwrap() as u32;
            let raw_dims = (
                record["raster"][0].as_u64().unwrap() as u32,
                record["raster"][1].as_u64().unwrap() as u32,
            );
            let entry = SOURCE_SHEARS
                .iter()
                .find(|s| s.source == source && s.page == page)
                .unwrap_or_else(|| panic!("shear installed for source {source} page {page}"));
            assert_eq!(entry.raw_dims, raw_dims);
            assert_eq!(u64::from(entry.x0_px), record["x0_px"].as_u64().unwrap());
            assert_eq!(
                entry.baseline_slope,
                record["baseline_slope"].as_f64().unwrap()
            );
            assert_eq!(entry.fingerprint, record["fingerprint"].as_str().unwrap());
            let region: [u32; 4] =
                std::array::from_fn(|i| record["fingerprint_region"][i].as_u64().unwrap() as u32);
            assert_eq!(entry.fingerprint_region, region);
        }
        // SOURCE_SHEARS is exclusive to this work, so every installed shear is reviewed.
        for entry in SOURCE_SHEARS {
            assert!(
                records.iter().any(|r| {
                    r["source"].as_u64() == Some(entry.source as u64)
                        && r["source_page"].as_u64() == Some(u64::from(entry.page))
                }),
                "installed shear source {} page {} has a reviewed ledger record",
                entry.source,
                entry.page
            );
        }
    }

    #[test]
    fn inner_tomi_whole_strips_match_reviewed_ledger() {
        let tomi = tomi_gap_split_repair_ledger();
        // Every reviewed whole-strip record is backed by a WHOLE_STRIPS signature with
        // identical cropped column dimensions, band, fingerprint region, and digest.
        for record in repairs_with_disposition(&tomi, "whole_strip") {
            let dims = (
                record["column_dims"][0].as_u64().unwrap() as u32,
                record["column_dims"][1].as_u64().unwrap() as u32,
            );
            let band = Band {
                top: record["composer_strip"][0].as_u64().unwrap() as u32,
                bottom: record["composer_strip"][1].as_u64().unwrap() as u32,
            };
            let region: [u32; 4] = std::array::from_fn(|i| {
                record["strip_fingerprint_region"][i].as_u64().unwrap() as u32
            });
            let fingerprint = record["strip_fingerprint"].as_str().unwrap();
            assert!(
                WHOLE_STRIPS.iter().any(|(d, b, r, f)| {
                    *d == dims && *b == band && *r == region && *f == fingerprint
                }),
                "reviewed whole strip source {} page {} is installed in WHOLE_STRIPS",
                record["source"],
                record["source_page"]
            );
        }
    }

    #[test]
    fn inner_tomi_row_excisions_match_reviewed_ledger() {
        let tomi = tomi_gap_split_repair_ledger();
        let records = repairs_with_disposition(&tomi, "row_excision");
        assert_eq!(records.len(), 1, "one reviewed Tomi row excision");
        let record = records[0];
        assert_eq!(record["source"], 0);
        assert_eq!(record["source_page"], 34);
        assert_eq!(record["raster"], serde_json::json!([1982, 3008]));
        let entry = SOURCE_ROW_EXCISIONS
            .iter()
            .find(|e| e.source == 0 && e.page == 34)
            .expect("Romulus p34 row excision installed");
        assert_eq!(entry.top, record["top"].as_f64().unwrap());
        assert_eq!(entry.bottom, record["bottom"].as_f64().unwrap());
        let sheared_height = record["sheared_raster"][1].as_u64().unwrap() as u32;
        let (top, bottom) = excision_rows(entry, sheared_height).unwrap();
        assert_eq!(record["deleted_rows"], serde_json::json!([top, bottom]));
        assert_eq!(
            record["column_dims"],
            serde_json::json!([828, sheared_height - (bottom - top)])
        );
        assert_eq!(record["gap_before_rows"], 271);
        assert_eq!(record["gap_after_rows"], 271 - (bottom - top));
        assert_eq!(
            SOURCE_ROW_EXCISIONS
                .iter()
                .filter(|e| e.source == 0 && e.page == 34)
                .count(),
            1
        );
    }

    #[test]
    fn inner_tomi_source_body_bands_match_reviewed_ledger() {
        let tomi = tomi_gap_split_repair_ledger();
        let records = repairs_with_disposition(&tomi, "source_body_band");
        assert!(
            !records.is_empty(),
            "the ledger installs a source_body_band repair"
        );
        // Every reviewed source-body-band record is backed by a SOURCE_BODY_BANDS entry
        // with identical identity, cropped column dimensions, and observed/protected band.
        for record in &records {
            let source = record["source"].as_u64().unwrap() as usize;
            let page = record["source_page"].as_u64().unwrap() as u32;
            let unit_id = record["unit_id"].as_str().unwrap();
            let dims = (
                record["column_dims"][0].as_u64().unwrap() as u32,
                record["column_dims"][1].as_u64().unwrap() as u32,
            );
            let observed = Band {
                top: record["observed"][0].as_u64().unwrap() as u32,
                bottom: record["observed"][1].as_u64().unwrap() as u32,
            };
            let protected = Band {
                top: record["protected"][0].as_u64().unwrap() as u32,
                bottom: record["protected"][1].as_u64().unwrap() as u32,
            };
            assert!(
                SOURCE_BODY_BANDS.iter().any(|b| {
                    b.source == source
                        && b.page == page
                        && b.unit_id == unit_id
                        && b.dimensions == dims
                        && b.observed == observed
                        && b.protected == protected
                }),
                "reviewed source body band source {source} page {page} is installed in SOURCE_BODY_BANDS"
            );
        }
        // The installed Philopoemen p448 rescue is reviewed here. SOURCE_BODY_BANDS also
        // carries entries from unrelated work, so only this branch's page is required to
        // round-trip back to a ledger record.
        let entry = SOURCE_BODY_BANDS
            .iter()
            .find(|b| b.source == 0 && b.page == 448)
            .expect("Philopoemen p448 body-band rescue installed");
        assert!(
            records.iter().any(|record| {
                record["source"].as_u64() == Some(entry.source as u64)
                    && record["source_page"].as_u64() == Some(u64::from(entry.page))
                    && record["unit_id"].as_str() == Some(entry.unit_id)
                    && record["column_dims"][0].as_u64() == Some(u64::from(entry.dimensions.0))
                    && record["column_dims"][1].as_u64() == Some(u64::from(entry.dimensions.1))
                    && record["observed"][0].as_u64() == Some(u64::from(entry.observed.top))
                    && record["observed"][1].as_u64() == Some(u64::from(entry.observed.bottom))
                    && record["protected"][0].as_u64() == Some(u64::from(entry.protected.top))
                    && record["protected"][1].as_u64() == Some(u64::from(entry.protected.bottom))
            }),
            "the installed Philopoemen p448 body-band entry has a reviewed ledger record"
        );
    }

    /// A reviewed Latin region `[x0, y0, x1, y1]` from a ledger record.
    fn ledger_region(value: &serde_json::Value) -> [u32; 4] {
        std::array::from_fn(|i| value[i].as_u64().unwrap() as u32)
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn inner_tomi_shears_preserve_latin() {
        let tomi = tomi_gap_split_repair_ledger();
        for record in repairs_with_disposition(&tomi, "shear") {
            let source = record["source"].as_u64().unwrap() as usize;
            let page = record["source_page"].as_u64().unwrap() as u32;
            let raw = real_source_image(source, page);
            let (cropped, _) =
                render_and_band(source, page, &|s, p| Ok(real_source_image(s, p))).unwrap();
            let x0 = record["x0_px"].as_u64().unwrap() as u32;
            let slope = record["baseline_slope"].as_f64().unwrap();
            let top_pad = record["top_pad"].as_u64().unwrap() as u32;
            let crop_left = record["crop_edges"][0].as_u64().unwrap() as u32;
            // The reviewed shear pages carry no row excisions.
            assert!(!SOURCE_ROW_EXCISIONS
                .iter()
                .any(|e| e.source == source && e.page == page));
            for region in record["preserved_latin_regions"].as_array().unwrap() {
                assert_sheared_source_region_pixels_survive(
                    &raw,
                    &cropped,
                    x0,
                    slope,
                    top_pad,
                    crop_left,
                    &[],
                    ledger_region(region),
                );
            }
        }
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn inner_tomi_whole_strips_preserve_latin() {
        let tomi = tomi_gap_split_repair_ledger();
        for record in repairs_with_disposition(&tomi, "whole_strip") {
            let source = record["source"].as_u64().unwrap() as usize;
            let page = record["source_page"].as_u64().unwrap() as u32;
            let (pages, _, traces) = audit_traced_unit(record["unit_id"].as_str().unwrap());
            let trace = retained_page(&traces, source, page).unwrap();
            let index = trace.retained_index.unwrap();
            let original = real_source_image(source, page);
            for region in record["preserved_latin_regions"].as_array().unwrap() {
                assert_source_region_pixels_survive(
                    &original,
                    &pages[index].0,
                    trace,
                    ledger_region(region),
                );
            }
        }
    }

    #[test]
    fn inner_tomi_masking_restores_separate_line_bands() {
        let mut original = column(
            120,
            100,
            &[(20, 30, 5, 90), (50, 60, 5, 90), (80, 90, 5, 90)],
        );
        for y in 20..90 {
            for x in 96..100 {
                original.put_pixel(x, y, Luma([0]));
            }
        }
        assert_eq!(line_bands(&original, LINE_MIN_INK).len(), 1);
        let mut cleaned = original.clone();
        apply_source_furniture_mask(
            &SourceFurnitureMask {
                source: 1,
                page: 441,
                left: 0.96,
                top: 0.0,
                right: 1.0,
                bottom: 1.0,
            },
            &mut cleaned,
        );
        assert_eq!(
            line_bands(&cleaned, LINE_MIN_INK),
            vec![
                Band {
                    top: 20,
                    bottom: 30
                },
                Band {
                    top: 50,
                    bottom: 60
                },
                Band {
                    top: 80,
                    bottom: 90
                },
            ]
        );
        for y in 0..120 {
            for x in 0..96 {
                assert_eq!(original.get_pixel(x, y), cleaned.get_pixel(x, y));
            }
        }
    }

    fn assert_reviewed_duplicate_sequence(id: &str) {
        let ledger = repair_ledger();
        let repairs: Vec<_> = ledger["repairs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["unit_id"] == id && r["disposition"] == "duplicate_page")
            .collect();
        assert!(!repairs.is_empty());
        let skipped: std::collections::BTreeSet<_> = repairs
            .iter()
            .flat_map(|r| r["skip_pages"].as_array().unwrap())
            .map(|p| p.as_u64().unwrap() as u32)
            .collect();
        let range = operational_range(id);
        let (pages, metrics, traces) = audit_traced_unit(id);
        let sequence: Vec<_> = traces
            .iter()
            .filter(|t| t.retained_index.is_some())
            .map(|t| t.page)
            .collect();
        let expected: Vec<_> = (range.unit.start_page..=range.end_page)
            .filter(|p| !skipped.contains(p))
            .collect();
        assert_eq!(sequence, expected);
        assert_eq!(pages.len(), sequence.len());
        let diagnostic = audit_repair_ledger();
        let mut reviewed = Vec::new();
        for repair in &repairs {
            let seam: Vec<_> = repair["retain_sequence"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p.as_u64().unwrap() as u32)
                .collect();
            assert!(sequence.windows(seam.len()).any(|w| w == seam));
            for p in repair["skip_pages"].as_array().unwrap() {
                assert!(!sequence.contains(&(p.as_u64().unwrap() as u32)));
            }
            let evidence = if repair["reviewed_lines"].is_array() {
                *repair
            } else {
                diagnostic["existing_duplicate_evidence"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["unit_id"] == id && r["finding_ids"] == repair["finding_ids"])
                    .unwrap()
            };
            let lines: Vec<ReviewedSourceLine> =
                serde_json::from_value(evidence["reviewed_lines"].clone()).unwrap();
            assert!(!lines.is_empty());
            reviewed.extend(lines);
        }
        reviewed.sort_by_key(|l| (l.source_page, l.region[1]));
        reviewed.dedup_by_key(|l| (l.source_page, l.region));
        let originals = reviewed
            .iter()
            .map(|l| (l.source, l.source_page))
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|(source, page)| ((source, page), real_source_image(source, page)))
            .collect();
        assert_reviewed_source_lines_survive(
            &reviewed,
            &ReviewedLineContext {
                traces: &traces,
                originals: &originals,
                pages: &pages,
                metrics: &metrics,
                scale: production_body_scale(),
                geom: TrimGeom::default(),
            },
            false,
        );
        for (index, (&page, (column, bands))) in sequence.iter().zip(&pages).enumerate() {
            let raw = real_source_image(range.unit.source, page);
            // Compare retained bands at the source coordinates used by production,
            // including reviewed crop edges that preserve marginal glyphs.
            let mut masked = raw.clone();
            mask_known_source_furniture(range.unit.source, page, &mut masked);
            let crop = production_source_column(range.unit.source, page, &masked).unwrap();
            let trace = RetainedSourcePageTrace {
                source: range.unit.source,
                page,
                retained_index: Some(index),
                crop: [crop.left, crop.right],
                bands: bands.iter().map(|b| [b.top, b.bottom]).collect(),
                deleted_rows: Vec::new(),
            };
            for band in bands {
                for y in band.top..band.bottom {
                    for x in 0..column.width() {
                        assert_eq!(
                            column.get_pixel(x, trace.map_row(y).unwrap()),
                            raw.get_pixel(crop.left + x, y),
                            "unit {id} source page {page} pixel {x},{y}"
                        );
                    }
                }
            }
        }
        if id == "comparatio_sertorii_et_eumenis" {
            let ending = operational_range("eumenes");
            assert_eq!(ending.end_page, range.unit.start_page);
            let (ending_pages, _) =
                unit_body_strips(&ending, |s, p| Ok(real_source_image(s, p))).unwrap();
            assert!(!ending_pages.last().unwrap().1.is_empty());
        }
        println!(
            "{id}: reviewed retained sequence {sequence:?}; join {:?}",
            repairs
                .iter()
                .map(|r| &r["adjacent_prose"])
                .collect::<Vec<_>>()
        );
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn eumenes_comparison_prose_is_complete_and_unique() {
        assert_reviewed_duplicate_sequence("comparatio_sertorii_et_eumenis");
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn agis_repeated_passage_survives_once() {
        assert_reviewed_duplicate_sequence("agis");
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn cato_minor_repeated_folios_survive_once() {
        assert_reviewed_duplicate_sequence("cato_minor");
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn agesilaus_second_scan_survives_once() {
        assert_reviewed_duplicate_sequence("agesilaus");
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn alexander_second_scan_survives_once() {
        assert_reviewed_duplicate_sequence("alexander");
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn phocion_second_scan_survives_once() {
        assert_reviewed_duplicate_sequence("phocion");
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn cicero_second_scan_survives_once() {
        assert_reviewed_duplicate_sequence("cicero");
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn antonius_second_scan_survives_once() {
        assert_reviewed_duplicate_sequence("antonius");
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn aratus_second_scan_survives_once() {
        assert_reviewed_duplicate_sequence("aratus");
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn otho_second_scan_survives_once() {
        assert_reviewed_duplicate_sequence("otho");
    }

    #[test]
    #[ignore = "requires all source rasters; writes production-scale repair evidence"]
    fn inner_tomi_post_mask_composition_evidence() {
        let dir = std::path::Path::new("tmp/pdfs/inner_tomi_repairs/post_mask");
        std::fs::create_dir_all(dir).unwrap();
        let units =
            crate::facsimile::units::load_units(std::path::Path::new("facsimile_units.json"))
                .unwrap();
        let ranges = crate::facsimile::units::ranges(&units, &[638, 711]);
        let geom = TrimGeom::default();
        let scale = super::super::global_scale_for(&ranges, geom.content_width(), &|s, p| {
            Ok(real_source_image(s, p))
        });
        let sampling: Vec<_> = ranges.iter().map(|r| serde_json::json!({"unit_id":r.unit.id,"source":r.unit.source,"start_page":r.unit.start_page,"end_page":r.end_page})).collect();
        let mut evidence = Vec::new();
        for id in [
            "marcius_coriolanus",
            "timoleon",
            "caius_marius",
            "brutus",
            "theseus",
            "pyrrhus",
            "comparatio_sertorii_et_eumenis",
            "agis",
            "camillus",
            "lysander",
            "demosthenes",
            "antonius",
            "cato_minor",
        ] {
            let range = operational_range(id);
            let sequence = std::cell::RefCell::new(Vec::new());
            let (pages, metrics) = unit_body_strips(&range, |s, p| {
                sequence.borrow_mut().push(p);
                Ok(real_source_image(s, p))
            })
            .unwrap();
            let sequence = sequence.into_inner();
            let canvases = compose_pages(
                &pages,
                scale,
                &geom,
                geom.first_body_budget(),
                geom.body_budget(),
                &metrics,
            );
            let mut stream_top = 0;
            let mut scaled_strips = Vec::new();
            for strip in strips_in_reading_order(&pages, &metrics) {
                let scaled =
                    scale_strip(&pages[strip.page_img_index].0, &strip.band, scale).unwrap();
                let stream_bottom = stream_top + scaled.height();
                scaled_strips.push((strip, scaled, stream_top, stream_bottom));
                stream_top = stream_bottom;
            }
            let mut canvas_bottom = 0;
            let mut seams = Vec::new();
            for (index, canvas) in canvases
                .iter()
                .enumerate()
                .take(canvases.len().saturating_sub(1))
            {
                canvas_bottom += canvas.height() - geom.pad_y;
                if let Some((strip, scaled, top, _)) = scaled_strips
                    .iter()
                    .find(|(_, _, top, bottom)| *top < canvas_bottom && canvas_bottom < *bottom)
                {
                    let row = canvas_bottom - top;
                    let rows = ink_profile_rows(scaled, INK_THRESHOLD);
                    seams.push(serde_json::json!({"canvas_before":index+1,"canvas_after":index+2,
                        "source_page":sequence[strip.page_img_index],"strip":[strip.band.top,strip.band.bottom],
                        "class":format!("{:?}",strip.class),"scaled_cut_row":row,
                        "source_row_estimate":strip.band.top as f32 + row as f32 / scale,
                        "ink_before_cut":rows[(row-1) as usize],"ink_after_cut":rows[row as usize]}));
                }
            }
            assert_eq!(
                canvases,
                assert_scaled_stream(
                    &pages,
                    &metrics,
                    scale,
                    &geom,
                    geom.first_body_budget(),
                    geom.body_budget()
                ),
                "{id}: production locators trace the complete scaled source row stream"
            );
            for (index, canvas) in canvases.iter().enumerate() {
                canvas
                    .save(dir.join(format!("{id}-canvas-{:03}.png", index + 1)))
                    .unwrap();
            }
            let mut canvas_top = 0;
            let canvas_ranges: Vec<_> = canvases
                .iter()
                .enumerate()
                .map(|(index, canvas)| {
                    let top = canvas_top;
                    canvas_top += canvas.height() - geom.pad_y;
                    (index + 1, top, canvas_top)
                })
                .collect();
            let strips: Vec<_> = scaled_strips.iter().map(|(strip, _, top, bottom)| serde_json::json!({
                "source_page":sequence[strip.page_img_index],"top":strip.band.top,"bottom":strip.band.bottom,
                "class":format!("{:?}",strip.class),"scaled_stream_rows":[top,bottom],
                "canvas_indices":canvas_ranges.iter().filter(|(_,lo,hi)|lo<bottom && hi>top).map(|(index,_,_)|index).collect::<Vec<_>>()
            })).collect();
            let bands: Vec<_> = sequence.iter().zip(&pages).map(|(p,(_,bands))|serde_json::json!({"source_page":p,"bands":bands.iter().map(|b|[b.top,b.bottom]).collect::<Vec<_>>()})).collect();
            evidence.push(serde_json::json!({"unit_id":id,"source":range.unit.source,"sequence":sequence,"bands":bands,"strips":strips,"canvas_count":canvases.len(),"within_strip_seams":seams}));
        }
        let result =
            serde_json::json!({"scale":scale,"width_sampling_ranges":sampling,"units":evidence});
        std::fs::write(
            dir.join("composition.json"),
            serde_json::to_vec_pretty(&result).unwrap(),
        )
        .unwrap();
        println!(
            "production scale {scale}; composition evidence {}",
            dir.display()
        );
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn marius_terminal_furniture_emits_no_body_canvas() {
        let range = operational_range("caius_marius");
        let (pages, metrics) =
            unit_body_strips(&range, |s, p| Ok(real_source_image(s, p))).unwrap();
        let ledger = repair_ledger();
        let repair = ledger["repairs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["finding_ids"][0] == "O5")
            .unwrap();
        let page = repair["source_page"].as_u64().unwrap() as u32;
        let retained = &pages[(page - range.unit.start_page) as usize];
        let top = (repair["mask_rectangles"][0][1].as_f64().unwrap() * retained.0.height() as f64)
            .round() as u32;
        assert!(!retained.1.is_empty(), "complete Marius ending retained");
        assert!(
            retained.1.iter().all(|b| b.bottom < top),
            "ornament contributes no retained band"
        );
        let retained_index = (page - range.unit.start_page) as usize;
        assert!(
            strips_in_reading_order(&pages, &metrics)
                .iter()
                .filter(|strip| strip.page_img_index == retained_index)
                .all(|strip| strip.band.bottom < top),
            "production extraction copies no ornament or terminal blank region"
        );
        save_retained_crop("marius-repaired-ending", retained);
    }

    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn inner_tomi_known_masks_preserve_latin() {
        let historical = repair_ledger();
        let backlog = backlog_repair_ledger();
        let tomi = tomi_gap_split_repair_ledger();
        let page_masks = all_reviewed_page_masks(&[&historical, &backlog, &tomi]);
        let dir = std::path::Path::new("tmp/pdfs/inner_tomi_repairs/masks");
        std::fs::create_dir_all(dir).unwrap();
        let mut measurements = Vec::new();
        for &(source, page) in page_masks.keys() {
            let records: Vec<_> = [&historical, &backlog, &tomi]
                .into_iter()
                .flat_map(|ledger| ledger["repairs"].as_array().unwrap())
                .filter(|repair| {
                    repair["disposition"] == "mask"
                        && repair["source"].as_u64() == Some(source as u64)
                        && repair["source_page"].as_u64() == Some(page as u64)
                        && (!repair["status"].is_string() || repair["status"] == "repaired")
                })
                .collect();
            let repair = records[0];
            let original = real_source_image(source, page);
            assert_eq!(
                [original.width() as u64, original.height() as u64],
                [
                    repair["raster"][0].as_u64().unwrap(),
                    repair["raster"][1].as_u64().unwrap()
                ]
            );
            let mut cleaned = original.clone();
            mask_known_source_furniture(source, page, &mut cleaned);
            let rectangles: Vec<[u32; 4]> = page_masks[&(source, page)]
                .iter()
                .map(|r| {
                    std::array::from_fn(|i| {
                        (r[i]
                            * if i % 2 == 0 {
                                original.width()
                            } else {
                                original.height()
                            } as f64)
                            .round() as u32
                    })
                })
                .collect();
            for (x, y, pixel) in original.enumerate_pixels() {
                let masked = rectangles
                    .iter()
                    .any(|a| (a[0]..a[2]).contains(&x) && (a[1]..a[3]).contains(&y));
                assert_eq!(
                    cleaned.get_pixel(x, y),
                    if masked { &Luma([255]) } else { pixel },
                    "source {source} page {page} pixel {x},{y}"
                );
            }
            // Feed the untouched raw page so render_and_band applies its own shear
            // and masking; a sheared page masked here would be transformed twice.
            let (_, bands) = render_and_band(source, page, &|_, _| Ok(original.clone())).unwrap();
            let raw_column = detect_latin_column(&original).unwrap();
            let raw_crop = crop_column(&original, &raw_column);
            let before = drop_furniture(&line_bands(&raw_crop, LINE_MIN_INK), original.height());
            let crop = preserve_known_body_crop_edge(
                source,
                page,
                &cleaned,
                detect_latin_column(&cleaned).unwrap(),
            );
            for repair in records {
                let expected_crop = current_reviewed_page_expectation(
                    &backlog,
                    source,
                    page,
                    repair,
                    "expected_crop_edges",
                );
                let expected_bands = current_reviewed_page_expectation(
                    &backlog,
                    source,
                    page,
                    repair,
                    "expected_pass_a_bands",
                );
                if let Some(expected) = expected_crop.as_array() {
                    assert_eq!(
                        vec![crop.left as u64, crop.right as u64],
                        expected
                            .iter()
                            .map(|v| v.as_u64().unwrap())
                            .collect::<Vec<_>>()
                    );
                    let expected: Vec<_> = expected_bands
                        .as_array()
                        .unwrap()
                        .iter()
                        .map(ledger_band)
                        .collect();
                    assert_eq!(
                        band_source_column(source, page, &crop_column(&cleaned, &crop)).1,
                        expected,
                        "source {source} page {page}: independently reviewed Pass A"
                    );
                }
                let trace = RetainedSourcePageTrace {
                    source,
                    page,
                    retained_index: None,
                    crop: [0, original.width()],
                    bands: vec![],
                    deleted_rows: vec![],
                };
                if expected_crop.is_array() {
                    assert!(repair["preserved_latin_regions"].is_array());
                }
                for region in repair["preserved_latin_regions"]
                    .as_array()
                    .into_iter()
                    .flatten()
                {
                    assert_source_region_pixels_survive(
                        &original,
                        &cleaned,
                        &trace,
                        std::array::from_fn(|i| region[i].as_u64().unwrap() as u32),
                    );
                }
                if repair["finding_ids"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|id| {
                        ["H1", "H2", "MARIUS_HEAD", "CATO_MAJOR_FOLIO"]
                            .iter()
                            .any(|name| id == name)
                    })
                {
                    let (pages, metrics, traces) =
                        audit_traced_unit(repair["unit_id"].as_str().unwrap());
                    let index = retained_page(&traces, source, page)
                        .unwrap()
                        .retained_index
                        .unwrap();
                    let first = ledger_band(&current_reviewed_page_expectation(
                        &backlog,
                        source,
                        page,
                        repair,
                        "expected_first_band",
                    ));
                    assert_eq!(
                        pages[index].1.first(),
                        Some(&first),
                        "cleaned head leaves the reviewed first body band"
                    );
                    assert!(strips_in_reading_order(&pages, &metrics)
                        .iter()
                        .filter(|s| s.page_img_index == index)
                        .all(|s| s.band.top >= first.top));
                }
            }
            let copied_span = |bands: &[Band]| {
                bands
                    .last()
                    .zip(bands.first())
                    .map_or(0, |(last, first)| last.bottom - first.top)
            };
            measurements.push(serde_json::json!({"source":source,"source_page":page,
                "crop_before":[raw_column.left,raw_column.right],"crop_after":[crop.left,crop.right],
                "bands_before_zone_filter":line_bands(&raw_crop,LINE_MIN_INK).iter().map(|b|[b.top,b.bottom]).collect::<Vec<_>>(),
                "bands_before_zone_filter_retained":before.iter().map(|b|[b.top,b.bottom]).collect::<Vec<_>>(),
                "bands_after_zone_filter":bands.iter().map(|b|[b.top,b.bottom]).collect::<Vec<_>>(),
                "copied_span_before_zone_banding":copied_span(&before),"copied_span_after_zone_banding":copied_span(&bands)}));
            original
                .save(dir.join(format!("s{source}-p{page:04}-raw.png")))
                .unwrap();
            cleaned
                .save(dir.join(format!("s{source}-p{page:04}-masked.png")))
                .unwrap();
            println!(
                "source {source} page {page}: copied span before/after zone banding {}/{}",
                copied_span(&before),
                copied_span(&bands)
            );
        }
        std::fs::write(
            dir.join("measurements.json"),
            serde_json::to_vec_pretty(&measurements).unwrap(),
        )
        .unwrap();
    }

    /// The frozen right-margin repair record for a source page: the full-height mask
    /// that restores the Latin column and trims the connecting scan-edge ink.
    fn margin_repair_record(
        ledger: &serde_json::Value,
        source: usize,
        page: u32,
    ) -> serde_json::Value {
        ledger["repairs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| {
                r["repair_role"] == "margin"
                    && r["source"].as_u64() == Some(source as u64)
                    && r["source_page"].as_u64() == Some(page as u64)
            })
            .unwrap_or_else(|| panic!("margin repair record for source {source} page {page}"))
            .clone()
    }

    fn ledger_band(value: &serde_json::Value) -> Band {
        Band {
            top: value[0].as_u64().unwrap() as u32,
            bottom: value[1].as_u64().unwrap() as u32,
        }
    }

    fn ledger_crop_edges(record: &serde_json::Value) -> [u32; 2] {
        [
            record["crop_edges"][0].as_u64().unwrap() as u32,
            record["crop_edges"][1].as_u64().unwrap() as u32,
        ]
    }

    /// The production crop (all masks applied) restores the ledger's Latin column: the
    /// scan-edge rule is no longer mistaken for the body. On the two pages that also
    /// carry a header mask (281/298), the header+margin union shifts the detected left
    /// edge by one pixel from the margin-only preview the ledger crop was measured
    /// against; the right edge and column width are unchanged.
    fn assert_restored_column(left: u32, right: u32, edges: [u32; 2], page: u32) {
        assert_eq!(
            right, edges[1],
            "page {page} restored Latin column right edge"
        );
        assert!(
            left.abs_diff(edges[0]) <= 1,
            "page {page} restored Latin column left edge {left} vs ledger {}",
            edges[0]
        );
    }

    /// A unit with no duplicate exclusions renders one retained page per source page,
    /// so the retained vector indexes directly by `page - start_page`.
    fn unit_retained_page(pages: &[PageBands], range: &UnitRange, page: u32) -> usize {
        assert_eq!(
            pages.len() as u32,
            range.end_page - range.unit.start_page + 1,
            "unit {:?} renders every source page (no exclusions)",
            range.unit.id
        );
        (page - range.unit.start_page) as usize
    }

    /// Every retained body pixel equals the raw source at its original column/row. The
    /// retained column is cropped from the masked image left of the right-margin mask,
    /// so an identity source comparison proves no Latin glyph was altered or erased.
    fn assert_retained_bands_match_source(retained: &PageBands, source: usize, page: u32) {
        let raw = real_source_image(source, page);
        let mut masked = raw.clone();
        mask_known_source_furniture(source, page, &mut masked);
        let crop = preserve_known_body_crop_edge(
            source,
            page,
            &masked,
            detect_latin_column(&masked).unwrap(),
        );
        for band in &retained.1 {
            for y in band.top..band.bottom {
                for x in 0..retained.0.width() {
                    assert_eq!(
                        retained.0.get_pixel(x, y),
                        raw.get_pixel(crop.left + x, y),
                        "source {source} page {page} retained pixel {x},{y}"
                    );
                }
            }
        }
    }

    /// A strip paired with its `[top, bottom)` rows in the scaled composed stream.
    type StripRange = (Strip, u32, u32);
    /// A one-based canvas index paired with its `[top, bottom)` rows in that stream.
    type CanvasRange = (usize, u32, u32);

    /// Scaled-stream row ranges of each strip and of each composed canvas, in one
    /// contiguous coordinate system, so a strip can be located within the canvases it
    /// touches. Mirrors the mapping saved by `inner_tomi_post_mask_composition_evidence`.
    fn strip_and_canvas_ranges(
        pages: &[PageBands],
        metrics: &LineMetrics,
        scale: f32,
        geom: &TrimGeom,
    ) -> (Vec<StripRange>, Vec<CanvasRange>, Vec<GrayImage>) {
        let canvases = compose_pages(
            pages,
            scale,
            geom,
            geom.first_body_budget(),
            geom.body_budget(),
            metrics,
        );
        ranges_for_composed_canvases(pages, metrics, scale, geom, canvases)
    }

    /// Map strips into an already verified set of composed canvases.
    fn ranges_for_composed_canvases(
        pages: &[PageBands],
        metrics: &LineMetrics,
        scale: f32,
        geom: &TrimGeom,
        canvases: Vec<GrayImage>,
    ) -> (Vec<StripRange>, Vec<CanvasRange>, Vec<GrayImage>) {
        let mut stream_top = 0;
        let mut strips = Vec::new();
        for strip in strips_in_reading_order(pages, metrics) {
            let scaled = scale_strip(&pages[strip.page_img_index].0, &strip.band, scale).unwrap();
            let stream_bottom = stream_top + scaled.height();
            strips.push((strip, stream_top, stream_bottom));
            stream_top = stream_bottom;
        }
        let mut canvas_top = 0;
        let canvas_ranges = canvases
            .iter()
            .enumerate()
            .map(|(index, canvas)| {
                let top = canvas_top;
                canvas_top += canvas.height() - geom.pad_y;
                (index + 1, top, canvas_top)
            })
            .collect();
        (strips, canvas_ranges, canvases)
    }

    /// Reviewed body-watermark tails. The right-margin masks delete the
    /// scan-edge ink that previously merged into the footer and dragged the copied
    /// region far past the last Latin line. Both the pixels and that copied space must
    /// go, and the following page's first line must survive.
    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn inner_tomi_watermark_cleanup_removes_pixels_and_space() {
        let ledger = repair_ledger();
        let dir = std::path::Path::new("tmp/pdfs/inner_tomi_repairs/watermark_cleanup");
        std::fs::create_dir_all(dir).unwrap();
        let mut units = std::collections::BTreeMap::new();
        let mut measurements = Vec::new();
        // Frozen copied endpoints and line regions bind the source seams.
        for (unit_id, source, page, next, old_copied) in [
            ("marcius_coriolanus", 0usize, 287u32, 288u32, 2940u32),
            ("timoleon", 0, 299, 300, 2922),
            ("aemilius_paulus", 0, 321, 322, 2881),
            ("brutus", 1, 603, 604, 2735),
            ("brutus", 1, 623, 624, 2735),
        ] {
            let record = if [321, 603, 623].contains(&page) {
                ledger["repairs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| {
                        r["disposition"] == "mask"
                            && r["source"].as_u64() == Some(source as u64)
                            && r["source_page"].as_u64() == Some(page as u64)
                    })
                    .unwrap()
                    .clone()
            } else {
                margin_repair_record(&ledger, source, page)
            };
            let end = ledger_band(&record["expected_last_band"]).bottom;

            // Masked zone banding ends at the last Latin line, not the watermark tail.
            let (_, bands) = render_and_band(source, page, &|s, p| Ok(real_source_image(s, p)))
                .unwrap_or_else(|e| panic!("render page {page}: {e:#}"));
            assert_eq!(
                bands.last().unwrap().bottom,
                end,
                "source {page} zone endpoint"
            );
            assert_eq!(*cut_points(&bands).last().unwrap(), end);
            assert!(old_copied > end, "copied region shrinks past the watermark");

            // The full unit keeps the same endpoint and copies nothing below it.
            let (pages, metrics, traces) = units
                .entry(unit_id)
                .or_insert_with(|| audit_traced_unit(unit_id));
            let index = retained_page(traces, source, page)
                .unwrap()
                .retained_index
                .unwrap();
            let current = &pages[index];
            assert_eq!(current.1.last().unwrap().bottom, end, "full-unit endpoint");
            let page_strips: Vec<_> = strips_in_reading_order(pages, metrics)
                .into_iter()
                .filter(|s| s.page_img_index == index)
                .collect();
            assert!(
                page_strips.iter().all(|s| s.band.bottom <= end),
                "no strip copied below the masked watermark endpoint"
            );
            assert_eq!(page_strips.last().unwrap().band.bottom, end);
            assert_retained_bands_match_source(current, source, page);

            // The following page's first body line is intact and faithful.
            let following = &pages[index + 1];
            assert!(!following.1.is_empty(), "page {next} retains its Latin");
            assert_retained_bands_match_source(following, source, next);
            if record["watermark_evidence"].is_object() {
                let evidence = &record["watermark_evidence"];
                let before: Vec<_> = evidence["baseline_strips"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(ledger_band)
                    .collect();
                let before_tail: u32 = before
                    .iter()
                    .map(|b| b.bottom.saturating_sub(b.top.max(end)))
                    .sum();
                let after_tail: u32 = page_strips
                    .iter()
                    .map(|s| s.band.bottom.saturating_sub(s.band.top.max(end)))
                    .sum();
                assert_eq!(
                    before_tail as u64,
                    evidence["copied_tail_height"]["before"].as_u64().unwrap()
                );
                assert_eq!(after_tail, 0, "zero copied furniture tail rows");
                let copied: u32 = page_strips.iter().map(|s| s.band.bottom - s.band.top).sum();
                let fragment_rows: u32 = evidence["fragment_preservation_copied_rows"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .chain(
                        evidence["restoration_copied_rows"]
                            .as_array()
                            .into_iter()
                            .flatten(),
                    )
                    .map(|r| {
                        let band = ledger_band(r);
                        assert!(band.bottom <= end);
                        for row in band.top..band.bottom {
                            assert_eq!(
                                page_strips
                                    .iter()
                                    .filter(|s| s.band.top <= row && row < s.band.bottom)
                                    .count(),
                                1
                            );
                        }
                        band.bottom - band.top
                    })
                    .sum();
                assert_eq!(
                    copied as u64,
                    record["copied_height"]["after"].as_u64().unwrap() + fragment_rows as u64
                );
                let lines: Vec<ReviewedSourceLine> =
                    serde_json::from_value(evidence["following_reviewed_lines"].clone()).unwrap();
                if evidence["following_first_source_line_required"] == true {
                    assert_eq!(lines[0].region[1], following.1[0].top);
                } else {
                    assert!(
                        lines
                            .iter()
                            .all(|l| l.source_page != 624 || l.region[1] >= 246),
                        "C5 remains independently pending"
                    );
                }
                let originals = lines
                    .iter()
                    .map(|l| {
                        (
                            (l.source, l.source_page),
                            real_source_image(l.source, l.source_page),
                        )
                    })
                    .collect();
                assert_reviewed_source_lines_survive(
                    &lines,
                    &ReviewedLineContext {
                        traces,
                        originals: &originals,
                        pages,
                        metrics,
                        scale: production_body_scale(),
                        geom: TrimGeom::default(),
                    },
                    false,
                );
                measurements.push(serde_json::json!({"source":source,"source_page":page,"copied_height_before":record["copied_height"]["before"],
                    "copied_height_after":copied,"copied_tail_before":before_tail,"copied_tail_after":after_tail}));
            }

            // The composed seam at production scale follows the stream with no footer
            // padding: the complete row stream equals the concatenated resized strips.
            let seam_pages = vec![
                (
                    current.0.clone(),
                    current.1.iter().rev().take(2).rev().copied().collect(),
                ),
                (
                    following.0.clone(),
                    following.1.iter().take(2).copied().collect(),
                ),
            ];
            let geom = TrimGeom::default();
            let seam = assert_scaled_stream(
                &seam_pages,
                metrics,
                production_body_scale(),
                &geom,
                4000,
                4000,
            );
            seam[0]
                .save(dir.join(format!("watermark-{page}-{next}-seam.png")))
                .unwrap();
            println!(
                "source {page}: copied endpoint {old_copied} -> {end}, {} rows removed",
                old_copied - end
            );
        }
        std::fs::write(
            dir.join("measurements.json"),
            serde_json::to_vec_pretty(&measurements).unwrap(),
        )
        .unwrap();
    }

    /// G1/G2/G3/G6: pages where column detection had locked onto a narrow right-margin
    /// rule and dropped the Latin column (or, for Pyrrhus's shared opening, copied a
    /// long furniture tail). The masks restore the real Latin crop; every retained line
    /// must be faithful to the source and the opening/tail endpoints must match review.
    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn inner_tomi_severe_gaps_preserve_all_lines() {
        let ledger = repair_ledger();
        let dir = std::path::Path::new("tmp/pdfs/inner_tomi_repairs/severe_gaps");
        std::fs::create_dir_all(dir).unwrap();
        for (unit_id, source, page, neighbors) in [
            ("timoleon", 0usize, 296u32, &[295u32, 297][..]),
            ("timoleon", 0, 304, &[305]),
            ("pyrrhus", 0, 470, &[471]),
            ("brutus", 1, 625, &[624, 626]),
        ] {
            let record = margin_repair_record(&ledger, source, page);
            let crop_edges = ledger_crop_edges(&record);
            let first = ledger_band(&record["expected_first_band"]);
            let last = ledger_band(&record["expected_last_band"]);

            // Masking restores the Latin column; the narrow scan-edge rule is gone.
            let mut masked = real_source_image(source, page);
            mask_known_source_furniture(source, page, &mut masked);
            let crop = detect_latin_column(&masked).unwrap();
            assert_restored_column(crop.left, crop.right, crop_edges, page);

            let range = operational_range(unit_id);
            let (pages, _) = unit_body_strips(&range, |s, p| Ok(real_source_image(s, p))).unwrap();
            let index = unit_retained_page(&pages, &range, page);
            let retained = &pages[index];
            assert_eq!(
                *retained.1.first().unwrap(),
                first,
                "source {page} first line"
            );
            assert_eq!(*retained.1.last().unwrap(), last, "source {page} last line");
            // The former copied tail on Pyrrhus's opening page is gone.
            if page == 470 {
                assert!(
                    last.bottom < 2782,
                    "copied tail trimmed below the former row 2782"
                );
            }
            assert_retained_bands_match_source(retained, source, page);
            save_retained_crop(&format!("severe-gap-{source}-{page}"), retained);

            // Each adjacent page in the join retains its own Latin faithfully.
            for &neighbor in neighbors {
                let n_index = unit_retained_page(&pages, &range, neighbor);
                assert!(
                    !pages[n_index].1.is_empty(),
                    "join neighbor page {neighbor} retains its Latin"
                );
                assert_retained_bands_match_source(&pages[n_index], source, neighbor);
            }
            println!(
                "source {page}: restored crop {crop_edges:?}, {} retained lines {}..{}",
                retained.1.len(),
                first.top,
                last.bottom
            );
            let _ = dir;
        }
    }

    /// O8B: a scan-edge rule on source 306 had merged three Timoleon lines into one
    /// tall strip that the packer cut mid-line across a canvas boundary. The margin
    /// mask restores separate line bands; each original line must now compose whole,
    /// on a single canvas, at the production scale.
    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn timoleon_reported_page_break_preserves_whole_lines() {
        let ledger = repair_ledger();
        let sliced = &ledger["post_rebuild_review"]["sliced_line"];
        let source = sliced["source"].as_u64().unwrap() as usize;
        let page = sliced["source_page"].as_u64().unwrap() as u32;
        let merged = ledger_band(&sliced["strip"]);

        let range = operational_range("timoleon");
        let (pages, metrics) =
            unit_body_strips(&range, |s, p| Ok(real_source_image(s, p))).unwrap();
        let index = unit_retained_page(&pages, &range, page);
        let retained = &pages[index];

        // The former merged interval is now covered by at least two separate lines.
        let within: Vec<_> = retained
            .1
            .iter()
            .filter(|b| b.top < merged.bottom && b.bottom > merged.top)
            .collect();
        assert!(
            within.len() >= 2,
            "merged strip {merged:?} split into separate line bands"
        );
        assert!(
            !retained
                .1
                .iter()
                .any(|b| b.top <= merged.top && b.bottom >= merged.bottom),
            "no single retained band spans the former merge"
        );
        assert_retained_bands_match_source(retained, source, page);

        // At production scale, no source-306 line is a tall (splittable) strip, and
        // each composes entirely within one canvas.
        let scale = production_body_scale();
        let geom = TrimGeom::default();
        let (strips, canvas_ranges, canvases) =
            strip_and_canvas_ranges(&pages, &metrics, scale, &geom);
        assert_eq!(
            canvases,
            assert_scaled_stream(
                &pages,
                &metrics,
                scale,
                &geom,
                geom.first_body_budget(),
                geom.body_budget()
            ),
            "composed stream is the concatenated resized strips"
        );
        let page_strips: Vec<_> = strips
            .iter()
            .filter(|(s, _, _)| s.page_img_index == index)
            .collect();
        assert!(!page_strips.is_empty());
        for (strip, top, bottom) in &page_strips {
            assert!(
                !matches!(strip.class, BandClass::Tall),
                "source {page} line {:?} is an ordinary (whole-wrapping) strip",
                strip.band
            );
            let touched: Vec<_> = canvas_ranges
                .iter()
                .filter(|(_, lo, hi)| lo < bottom && hi > top)
                .collect();
            assert_eq!(
                touched.len(),
                1,
                "source {page} line {:?} composes whole on one canvas, touched {touched:?}",
                strip.band
            );
        }
        let dir = std::path::Path::new("tmp/pdfs/inner_tomi_repairs/timoleon_seam");
        std::fs::create_dir_all(dir).unwrap();
        let touched: std::collections::BTreeSet<usize> = page_strips
            .iter()
            .flat_map(|(_, top, bottom)| {
                canvas_ranges
                    .iter()
                    .filter(move |(_, lo, hi)| lo < bottom && hi > top)
                    .map(|(i, _, _)| *i)
            })
            .collect();
        for i in touched {
            canvases[i - 1]
                .save(dir.join(format!("timoleon-canvas-{i:03}.png")))
                .unwrap();
        }
        println!(
            "source {page}: {} restored lines over former merge {merged:?}, each whole on one canvas",
            within.len()
        );
    }

    /// Timoleon and Brutus previously produced near-blank canvases whose only content
    /// was a right-margin rule that column detection had mistaken for the body. With
    /// the rules masked, the Latin columns are restored, no strip derives from a rule
    /// region, and every composed canvas carries real body ink. Blank recto separators
    /// are inserted later by assembly and are not produced here.
    #[test]
    #[ignore = "requires source PDFs and production-DPI renders"]
    fn timoleon_and_brutus_have_no_furniture_only_body_canvases() {
        let ledger = repair_ledger();
        let scale = production_body_scale();
        let geom = TrimGeom::default();
        let dir = std::path::Path::new("tmp/pdfs/inner_tomi_repairs/furniture_only");
        std::fs::create_dir_all(dir).unwrap();
        for (unit_id, source, masked_pages) in [
            ("timoleon", 0usize, &[296u32, 298, 304, 306, 308][..]),
            ("brutus", 1, &[625]),
        ] {
            // Every masked furniture page now resolves to its Latin column, so the
            // right-margin rule contributes no band and therefore no strip.
            for &page in masked_pages {
                let record = margin_repair_record(&ledger, source, page);
                let crop_edges = ledger_crop_edges(&record);
                let mut masked = real_source_image(source, page);
                mask_known_source_furniture(source, page, &mut masked);
                let crop = detect_latin_column(&masked).unwrap();
                assert_restored_column(crop.left, crop.right, crop_edges, page);
            }

            let range = operational_range(unit_id);
            let (pages, metrics) =
                unit_body_strips(&range, |s, p| Ok(real_source_image(s, p))).unwrap();
            // No retained band on a masked page lies in or beyond the masked rule.
            for &page in masked_pages {
                let index = unit_retained_page(&pages, &range, page);
                assert!(!pages[index].1.is_empty(), "page {page} retains Latin");
                assert_retained_bands_match_source(&pages[index], source, page);
            }

            let canvases = compose_pages(
                &pages,
                scale,
                &geom,
                geom.first_body_budget(),
                geom.body_budget(),
                &metrics,
            );
            assert!(!canvases.is_empty());
            // Every composed canvas carries at least a full scaled line of body ink:
            // compose drops anything fainter, so none is a furniture-only strip.
            let floor = minimum_ink(&metrics)
                .map(|m| (m as f64 * (scale as f64).powi(2)).ceil().max(1.0) as u64)
                .unwrap_or(1);
            for (i, canvas) in canvases.iter().enumerate() {
                let ink: u64 = ink_profile_rows(canvas, INK_THRESHOLD)
                    .iter()
                    .map(|&n| n as u64)
                    .sum();
                assert!(
                    ink >= floor,
                    "{unit_id} canvas {} carries body ink {ink} >= {floor}",
                    i + 1
                );
            }
            canvases
                .last()
                .unwrap()
                .save(dir.join(format!("{unit_id}-final-canvas.png")))
                .unwrap();
            println!(
                "{unit_id}: {} body canvases, none furniture-only (ink floor {floor})",
                canvases.len()
            );
        }
    }

    /// Save original-coordinate traces and column crops for a representative source
    /// window using production masking and banding. These windows contain no deleted
    /// rows, so every retained row maps to itself.
    fn save_representative_repair_trace(
        group: &str,
        pages: &[(usize, u32)],
    ) -> Vec<RetainedSourcePageTrace> {
        let dir = std::path::Path::new("tmp/pdfs/inner_tomi_repairs");
        std::fs::create_dir_all(dir).unwrap();
        let mut traces = Vec::new();
        for (retained_index, &(source, page)) in pages.iter().enumerate() {
            let image = real_source_image(source, page);
            let column = detect_latin_column(&image).unwrap();
            let (col, bands) = render_and_band(source, page, &|_, _| Ok(image.clone())).unwrap();
            col.save(dir.join(format!("{group}-s{source}-p{page:04}-column.png")))
                .unwrap();
            let trace = RetainedSourcePageTrace {
                source,
                page,
                retained_index: Some(retained_index),
                crop: [column.left, column.right],
                bands: bands.iter().map(|b| [b.top, b.bottom]).collect(),
                deleted_rows: Vec::new(),
            };
            // Without row deletions, retained bands use original source rows.
            for b in &trace.bands {
                assert_eq!(trace.map_row(b[0]), Some(b[0]));
            }
            traces.push(trace);
        }
        let path = dir.join(format!("{group}.json"));
        std::fs::write(&path, serde_json::to_vec_pretty(&traces).unwrap()).unwrap();
        println!("representative repair trace: {}", path.display());
        traces
    }

    #[test]
    #[ignore = "requires source PDFs and Poppler; populates D1/D2 diagnosis"]
    fn inner_tomi_duplicate_source_trace() {
        // Trace the Agis opening and the shared Eumenes/comparison seam.
        let traces = save_representative_repair_trace("duplicate", &[(1, 88), (1, 102), (1, 363)]);
        assert_eq!(
            retained_page(&traces, 1, 363).unwrap().retained_index,
            Some(2)
        );
    }

    #[test]
    #[ignore = "requires source PDFs and Poppler; populates Timoleon/Coriolanus diagnosis"]
    fn inner_tomi_timoleon_coriolanus_source_trace() {
        let traces = save_representative_repair_trace("timoleon_coriolanus", &[(0, 268), (0, 295)]);
        assert!(retained_page(&traces, 0, 295).is_some());
    }

    #[test]
    #[ignore = "requires source PDFs and Poppler; populates Brutus/remaining diagnosis"]
    fn inner_tomi_remaining_source_trace() {
        let traces = save_representative_repair_trace("remaining", &[(1, 596)]);
        assert!(retained_page(&traces, 1, 596).is_some());
    }

    fn two_line_boundary_fixture() -> (Vec<UnitRange>, GrayImage) {
        let mut cato = operational_range("cato_major").unit;
        let mut comparison = operational_range("comparatio_aristidis_et_catonis").unit;
        cato.start_page = 434;
        cato.body_top_frac = Some(0.40);
        comparison.last_page = Some(434);
        let ranges = crate::facsimile::units::ranges(&[cato, comparison], &[434]);
        let mut img = body_page(
            1000,
            &[
                (400, 410, 122, 188),
                (420, 430, 122, 188),
                (440, 450, 122, 188),
                (470, 480, 122, 188),
                (510, 512, 140, 170),
                (549, 560, 140, 180),
                (578, 589, 140, 180),
                (609, 619, 122, 188),
                (629, 639, 122, 188),
                (649, 659, 122, 188),
            ],
        );
        // Match the Greek column's total ink to the sparse Latin fixture.
        for y in 0..img.height() {
            let ink = (120..190).any(|x| img.get_pixel(x, y)[0] == 0);
            for x in 10..80 {
                img.put_pixel(x, y, Luma([if ink { 0 } else { 255 }]));
            }
        }
        (ranges, img)
    }

    #[test]
    fn shared_tail_excludes_both_lines_of_comparatio_title() {
        let (ranges, img) = two_line_boundary_fixture();
        assert_eq!(ranges[0].tail_cut_frac, Some(0.5491843));
        let (pages, _) = unit_body_strips(&ranges[0], |_, _| Ok(img.clone())).unwrap();
        assert_eq!(
            pages[0].1,
            vec![
                Band {
                    top: 400,
                    bottom: 410
                },
                Band {
                    top: 420,
                    bottom: 430
                },
                Band {
                    top: 440,
                    bottom: 450
                },
                Band {
                    top: 470,
                    bottom: 480
                }
            ]
        );
    }

    #[test]
    fn opening_fraction_preserves_comparatio_first_body_line() {
        let (ranges, img) = two_line_boundary_fixture();
        let (pages, _) = unit_body_strips(&ranges[1], |_, _| Ok(img.clone())).unwrap();
        assert_eq!(
            pages[0].1,
            vec![
                Band {
                    top: 609,
                    bottom: 619
                },
                Band {
                    top: 629,
                    bottom: 639
                },
                Band {
                    top: 649,
                    bottom: 659
                }
            ]
        );
    }

    #[test]
    #[ignore = "requires source PDFs and Poppler"]
    fn cato_comparatio_source_boundary_excludes_duplicate_title() {
        for id in ["cato_major", "comparatio_aristidis_et_catonis"] {
            let range = operational_range(id);
            let (pages, m) = traced_real_unit(&range);
            verify_real_composition(id, &pages, &m);
            assert_eq!(
                pages.len() as u32,
                range.end_page - range.unit.start_page + 1
            );
            let page = &pages[(434 - range.unit.start_page) as usize];
            let h = page.0.height() as f32;
            if id == "cato_major" {
                assert!((page.1.last().unwrap().bottom as f32) < 0.5491843 * h);
                assert!((page.1.last().unwrap().bottom as f32) > 0.49 * h);
            } else {
                assert!((page.1[0].top as f32) > 0.59 * h);
                assert!((page.1[0].top as f32) < 0.62 * h);
            }
            save_retained_crop(id, page);
        }
    }

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
    fn compose_drops_near_blank_split_canvases() {
        let img = column(100, 100, &[(0, 8, 5, 95)]);
        let pages = [(
            img,
            vec![Band {
                top: 0,
                bottom: 100,
            }],
        )];
        let m = LineMetrics {
            height_med: 8,
            height_mad: 0,
            width_med: 90,
        };
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 100,
            tb_h: 100,
            pad_x: 0,
            pad_y: 0,
        };
        let canvases = compose_pages(&pages, 1.0, &geom, 20, 20, &m);
        assert_eq!(canvases.len(), 1);
        let ink: u64 = ink_profile_rows(&canvases[0], INK_THRESHOLD)
            .iter()
            .map(|&n| n as u64)
            .sum();
        assert_eq!(ink, 8 * 90);
    }

    #[test]
    fn compose_canvas_ink_floor_tracks_global_scale() {
        let (page, m) = page_and_metrics(8, 100, &[(0, 8, 5, 95)]);
        let geom = TrimGeom {
            dpi: 300,
            tb_w: 10,
            tb_h: 100,
            pad_x: 0,
            pad_y: 0,
        };
        // After shrinking, the real line has less ink than the source-pixel floor.
        let canvases = compose_pages(&[page], 0.1, &geom, 20, 20, &m);
        assert_eq!(canvases.len(), 1);
        assert!(
            ink_profile_rows(&canvases[0], INK_THRESHOLD)
                .iter()
                .sum::<u32>()
                > 0
        );
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
    fn shared_tail_frac_removes_rule_connected_to_gutter_noise() {
        let mut img = column(
            150,
            100,
            &[
                (0, 8, 5, 95),
                (12, 20, 5, 95),
                (24, 32, 5, 95),
                (60, 62, 40, 60),
                (100, 110, 5, 95),
                (125, 133, 5, 95),
            ],
        );
        for y in 0..78 {
            for x in 0..5 {
                img.put_pixel(x, y, Luma([0]));
            }
        }
        let bands = line_bands(&img, LINE_MIN_INK);
        let m = LineMetrics {
            height_med: 8,
            height_mad: 0,
            width_med: 90,
        };
        assert_eq!(bands[0], Band { top: 0, bottom: 78 });
        let kept = drop_shared_tail(&img, &bands, Some(100.0 / 150.0), &m);
        assert_eq!(kept, vec![Band { top: 0, bottom: 46 }]);
        assert!(
            kept[0].bottom > 32,
            "the previous unit's final line must survive"
        );
    }

    #[test]
    fn trailing_rule_cleanup_preserves_real_line_and_sentinel() {
        let mut img = column(
            90,
            100,
            &[
                (0, 8, 5, 95),
                (12, 20, 5, 95),
                (24, 32, 5, 95),
                (60, 68, 40, 60),
            ],
        );
        for y in 0..78 {
            for x in 0..5 {
                img.put_pixel(x, y, Luma([0]));
            }
        }
        let bands = line_bands(&img, LINE_MIN_INK);
        let m = LineMetrics {
            height_med: 8,
            height_mad: 0,
            width_med: 90,
        };
        assert_eq!(drop_trailing_furniture(&img, &bands, &m), bands);
        assert_eq!(
            drop_trailing_furniture(&img, &bands, &LineMetrics::sentinel()),
            bands
        );
    }

    #[test]
    fn shared_tail_frac_removes_rule_smaller_than_strip_ink_floor() {
        let mut img = column(
            400,
            1066,
            &[
                (0, 33, 80, 864),
                (45, 78, 80, 864),
                (90, 123, 80, 864),
                (250, 257, 480, 544),
                (350, 360, 80, 864),
            ],
        );
        for y in 0..305 {
            for x in 0..4 {
                img.put_pixel(x, y, Luma([0]));
            }
        }
        let bands = line_bands(&img, LINE_MIN_INK);
        let m = LineMetrics {
            height_med: 33,
            height_mad: 1,
            width_med: 784,
        };
        assert!(7 * 64 < minimum_ink(&m).unwrap());
        let kept = drop_shared_tail(&img, &bands, Some(350.0 / 400.0), &m);
        assert_eq!(
            kept,
            vec![Band {
                top: 0,
                bottom: 186
            }]
        );
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
