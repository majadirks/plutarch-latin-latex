//! Intact editorial lines from the bounded Praefatio corrigenda regions.

use anyhow::{bail, Result};
use image::{GrayImage, Luma};

use super::frontmatter::PraefatioRegions;
use super::lines::{cut_points, line_bands, Band};
use super::metrics::{line_metrics, median_u32};
use super::reflow::{place_line_images, resize_line_image, TrimGeom, BODY_DPI};

/// Maximum blank-row gap joining isolated accents to their printed baseline.
const ACCENT_JOIN_GAP_FRAC: f64 = 0.001;

#[derive(Debug, Clone)]
pub(crate) struct ErrataLine {
    pub image: GrayImage,
    pub source_page: u32,
    pub line_index: usize,
    /// Full-page crop before the first corrigendum is relocated horizontally.
    pub source_rect: (u32, u32, u32, u32),
}

fn group_accents(raw: &[Band], gap: u32) -> Vec<Band> {
    let mut grouped: Vec<Band> = Vec::new();
    for &band in raw {
        if let Some(last) = grouped.last_mut() {
            if band.top.saturating_sub(last.bottom) <= gap {
                last.bottom = band.bottom;
                continue;
            }
        }
        grouped.push(band);
    }
    grouped
}

/// Reviewed intervals must cover every dark row exactly once.
fn validate_intervals(region: &GrayImage, bands: &[Band], page: u32) -> Result<()> {
    if bands.is_empty() {
        bail!("praefatio source 1, source page {page}: empty line extraction");
    }
    for (i, band) in bands.iter().enumerate() {
        if band.top >= band.bottom
            || band.bottom > region.height()
            || (i > 0 && bands[i - 1].bottom > band.top)
        {
            bail!("praefatio source 1, source page {page}, line {}: overlapping or invalid ink interval", i+1);
        }
    }
    for raw in line_bands(region, 0) {
        for y in raw.top..raw.bottom {
            if bands.iter().filter(|b| b.top <= y && y < b.bottom).count() != 1 {
                bail!("praefatio source 1, source page {page}: uncovered ink at row {y}");
            }
        }
    }
    Ok(())
}

pub(crate) fn extract_errata_lines(
    page: u32,
    image: &GrayImage,
    regions: PraefatioRegions,
) -> Result<Vec<ErrataLine>> {
    let (x, body_top, width, height) = regions.body;
    if width == 0
        || height == 0
        || x.checked_add(width).is_none_or(|r| r > image.width())
        || body_top
            .checked_add(height)
            .is_none_or(|b| b > image.height())
    {
        bail!("praefatio source 1, source page {page}: invalid extraction rectangle");
    }
    let top = if page == 15 {
        regions
            .transition
            .ok_or_else(|| anyhow::anyhow!("praefatio source page 15: missing transition"))?
            .0
    } else if page == 16 {
        body_top
    } else {
        bail!("praefatio source page {page}: not an errata page");
    };
    if top < body_top || top >= body_top + height {
        bail!("praefatio source page {page}: transition outside body");
    }
    let mut region =
        image::imageops::crop_imm(image, x, top, width, body_top + height - top).to_image();
    if let Some((start, end, split)) = regions.transition {
        if start != top || end > body_top + height || split <= x || split >= x + width {
            bail!("praefatio source page {page}: invalid horizontal transition");
        }
        for y in 0..end - top {
            for xx in 0..split - x {
                region.put_pixel(xx, y, Luma([255]));
            }
        }
    }
    let raw = line_bands(&region, 0);
    let gap = (image.height() as f64 * ACCENT_JOIN_GAP_FRAC).round() as u32;
    let bands = group_accents(&raw, gap);
    validate_intervals(&region, &bands, page)?;
    let metrics = line_metrics(&[(region.clone(), bands.clone())]);
    if bands
        .iter()
        .any(|b| b.bottom - b.top > metrics.height_med * 2)
    {
        bail!(
            "praefatio source page {page}: uncertain merged baseline; reviewed intervals required"
        );
    }
    let cuts = cut_points(&bands);
    let mut result = Vec::new();
    for (i, pair) in cuts.windows(2).enumerate() {
        let (crop_x, crop_w) = if page == 15 && i == 0 {
            let split = regions.transition.unwrap().2;
            (split, x + width - split)
        } else {
            (x, width)
        };
        let strip =
            image::imageops::crop_imm(&region, crop_x - x, pair[0], crop_w, pair[1] - pair[0])
                .to_image();
        let mut canvas = GrayImage::from_pixel(width, strip.height(), Luma([255]));
        image::imageops::overlay(&mut canvas, &strip, 0, 0);
        result.push(ErrataLine {
            image: canvas,
            source_page: page,
            line_index: i + 1,
            source_rect: (crop_x, top + pair[0], crop_w, pair[1] - pair[0]),
        });
    }
    Ok(result)
}

/// Add the ordinary half-gap at the single source-page join.
pub(crate) fn join_errata_pages(
    mut first: Vec<ErrataLine>,
    mut second: Vec<ErrataLine>,
) -> Result<Vec<ErrataLine>> {
    if first.is_empty() || second.is_empty() {
        bail!("praefatio source pages 15â€“16: missing errata lines");
    }
    let mut gaps = Vec::new();
    for page in [&first, &second] {
        for pair in page.windows(2) {
            let a = line_bands(&pair[0].image, 0);
            let b = line_bands(&pair[1].image, 0);
            gaps.push(pair[0].image.height() - a.last().unwrap().bottom + b[0].top);
        }
    }
    let gap = median_u32(&gaps);
    let last = first.last_mut().unwrap();
    let mut padded = GrayImage::from_pixel(
        last.image.width(),
        last.image.height() + gap / 2,
        Luma([255]),
    );
    image::imageops::overlay(&mut padded, &last.image, 0, 0);
    last.image = padded;
    let initial = &mut second[0];
    let mut padded = GrayImage::from_pixel(
        initial.image.width(),
        initial.image.height() + gap - gap / 2,
        Luma([255]),
    );
    image::imageops::overlay(&mut padded, &initial.image, 0, (gap - gap / 2) as i64);
    initial.image = padded;
    first.extend(second);
    Ok(first)
}

/// Minimum page count, then minimum load range, then earliest break vector.
pub(crate) fn balanced_errata_breaks(
    heights: &[u32],
    pad_y: u32,
    budget: u32,
) -> Result<Vec<usize>> {
    if heights.is_empty() {
        bail!("praefatio: no errata lines to compose");
    }
    let mut prefix = vec![0u64];
    let mut groups = 1;
    let mut current = u64::from(pad_y);
    for &height in heights {
        if height == 0 || u64::from(height) + u64::from(pad_y) > u64::from(budget) {
            bail!("praefatio: line height {height} cannot fit budget {budget}");
        }
        prefix.push(prefix.last().unwrap() + u64::from(height));
        if current + u64::from(height) > u64::from(budget) {
            groups += 1;
            current = u64::from(pad_y);
        }
        current += u64::from(height);
    }
    let n = heights.len();
    let load = |start: usize, end: usize| prefix[end] - prefix[start] + u64::from(pad_y);
    let mut loads = Vec::new();
    for start in 0..n {
        for end in start + 1..=n {
            let value = load(start, end);
            if value <= u64::from(budget) {
                loads.push(value);
            }
        }
    }
    loads.sort_unstable();
    loads.dedup();
    let mut ranges = Vec::new();
    for (i, &low) in loads.iter().enumerate() {
        for &high in &loads[i..] {
            ranges.push((high - low, low, high));
        }
    }
    ranges.sort_unstable();
    let mut best: Option<Vec<usize>> = None;
    let mut best_difference = None;
    for (difference, low, high) in ranges {
        if best_difference.is_some_and(|d| difference > d) {
            break;
        }
        let mut can = vec![vec![false; n + 1]; groups + 1];
        can[0][n] = true;
        for remaining in 1..=groups {
            for start in (0..n).rev() {
                can[remaining][start] = (start + 1..=n).any(|end| {
                    let value = load(start, end);
                    value >= low && value <= high && can[remaining - 1][end]
                });
            }
        }
        if !can[groups][0] {
            continue;
        }
        let mut ends = Vec::new();
        let mut start = 0;
        for remaining in (1..=groups).rev() {
            let end = (start + 1..=n)
                .find(|&end| {
                    let value = load(start, end);
                    value >= low && value <= high && can[remaining - 1][end]
                })
                .unwrap();
            ends.push(end);
            start = end;
        }
        if best.as_ref().is_none_or(|b| ends < *b) {
            best = Some(ends);
        }
        best_difference = Some(difference);
    }
    best.ok_or_else(|| anyhow::anyhow!("praefatio: no feasible whole-line partition"))
}

pub(crate) fn compose_errata_pages(
    lines: &[ErrataLine],
    geom: &TrimGeom,
) -> Result<Vec<GrayImage>> {
    let width = lines.iter().map(|l| l.image.width()).max().unwrap_or(0);
    if width == 0 {
        bail!("praefatio: empty source column");
    }
    for line in lines {
        if line.image.width() == 0 || line.image.height() == 0 {
            bail!(
                "praefatio source 1, source page {}, line {}: empty line image",
                line.source_page,
                line.line_index
            );
        }
    }
    let scale = geom.content_width() as f32 / width as f32;
    let strips: Vec<_> = lines
        .iter()
        .map(|l| resize_line_image(&l.image, scale))
        .collect();
    for (line, strip) in lines.iter().zip(&strips) {
        if strip.height().saturating_add(geom.pad_y) > geom.body_budget()
            || strip.width() > geom.content_width()
        {
            bail!("praefatio source 1, source page {}, line {} ({:?}): scaled line cannot fit the image budget", line.source_page, line.line_index, line.source_rect);
        }
    }
    let ends = balanced_errata_breaks(
        &strips.iter().map(|s| s.height()).collect::<Vec<_>>(),
        geom.pad_y,
        geom.body_budget(),
    )?;
    let mut start = 0;
    Ok(ends
        .into_iter()
        .map(|end| {
            let canvas = place_line_images(&strips[start..end], geom);
            start = end;
            canvas
        })
        .collect())
}

pub(crate) fn render_praefatio_errata(
    pdf: &crate::render::Pdf,
    out_dir: &std::path::Path,
) -> Result<Vec<String>> {
    let mut pages = Vec::new();
    for page in [15, 16] {
        let png = crate::render::render_page(
            &pdf.path,
            page,
            crate::render::TRANSCRIBE_DPI,
            &crate::pipeline::scratch_dir(),
        )?;
        let image = image::open(png)?.to_luma8();
        let regions = super::frontmatter::praefatio_regions(image.width(), image.height(), page)?;
        pages.push(extract_errata_lines(page, &image, regions)?);
    }
    let second = pages.pop().unwrap();
    let lines = join_errata_pages(pages.pop().unwrap(), second)?;
    let geom = TrimGeom::new(BODY_DPI);
    let canvases = compose_errata_pages(&lines, &geom)?;
    eprintln!(
        "  Praefatio: {} lines, scale {:.6}, image heights {:?}",
        lines.len(),
        geom.content_width() as f32 / lines.iter().map(|l| l.image.width()).max().unwrap() as f32,
        canvases.iter().map(|c| c.height()).collect::<Vec<_>>()
    );
    std::fs::create_dir_all(out_dir)?;
    canvases
        .iter()
        .enumerate()
        .map(|(i, canvas)| {
            let name = format!("front_praefatio_errata_{:03}.png", i + 1);
            canvas.save(out_dir.join(&name))?;
            Ok(name)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::columns::INK_THRESHOLD;
    use super::super::frontmatter::praefatio_regions;
    use super::*;

    fn packing_lines(heights: &[u32], widths: &[u32]) -> Vec<ErrataLine> {
        heights
            .iter()
            .enumerate()
            .map(|(i, &h)| ErrataLine {
                image: GrayImage::from_pixel(widths[i], h, Luma([20 + i as u8])),
                source_page: if i < 2 { 15 } else { 16 },
                line_index: i + 1,
                source_rect: (0, 0, widths[i], h),
            })
            .collect()
    }

    fn small_geom() -> TrimGeom {
        TrimGeom {
            dpi: 1,
            tb_w: 20,
            tb_h: 31,
            pad_x: 0,
            pad_y: 1,
        }
    }

    #[test]
    fn praefatio_errata_uses_one_scale() {
        let lines = packing_lines(&[10, 10], &[40, 20]);
        let canvases = compose_errata_pages(&lines, &small_geom()).unwrap();
        assert_eq!(canvases.len(), 1);
        assert_eq!(canvases[0].dimensions(), (20, 11));
        assert_eq!(canvases[0].get_pixel(15, 8)[0], 255);
        assert_eq!(canvases[0].get_pixel(5, 8)[0], 21);
    }

    #[test]
    fn praefatio_errata_balances_pages_without_splitting_lines() {
        assert_eq!(
            balanced_errata_breaks(&[9, 9, 9, 9, 9], 1, 31).unwrap(),
            vec![2, 5]
        );
        let canvases =
            compose_errata_pages(&packing_lines(&[9; 5], &[20; 5]), &small_geom()).unwrap();
        assert_eq!(
            canvases.iter().map(|c| c.height()).collect::<Vec<_>>(),
            [19, 28]
        );
    }

    #[test]
    fn praefatio_errata_packs_across_source_page_boundary() {
        let canvases =
            compose_errata_pages(&packing_lines(&[5; 4], &[20; 4]), &small_geom()).unwrap();
        assert_eq!(canvases.len(), 1);
        for i in 0..4 {
            assert_eq!(canvases[0].get_pixel(0, 1 + i * 5)[0], 20 + i as u8);
        }
    }

    #[test]
    fn praefatio_errata_has_no_empty_canvas() {
        assert!(compose_errata_pages(&[], &small_geom()).is_err());
        assert!(compose_errata_pages(&packing_lines(&[0], &[20]), &small_geom()).is_err());
        assert_eq!(balanced_errata_breaks(&[30, 30], 1, 31).unwrap(), [1, 2]);
    }

    #[test]
    fn praefatio_errata_rejects_oversized_line() {
        let err = compose_errata_pages(&packing_lines(&[31], &[20]), &small_geom()).unwrap_err();
        assert!(err.to_string().contains("source page 15, line 1"));
    }

    #[test]
    fn praefatio_errata_budget_uses_rounded_strip_heights() {
        let mut geom = small_geom();
        geom.tb_h = 4;
        geom.pad_y = 0;
        let canvases = compose_errata_pages(&packing_lines(&[3; 3], &[40; 3]), &geom).unwrap();
        assert_eq!(
            canvases.iter().map(|c| c.height()).collect::<Vec<_>>(),
            [2, 4]
        );
    }

    #[test]
    fn praefatio_errata_partition_matches_exhaustive_optimum() {
        for n in 1..=7 {
            for seed in 0..40 {
                let heights: Vec<_> = (0..n)
                    .map(|i| 1 + ((seed * 7 + i * 3 + i * i) % 9) as u32)
                    .collect();
                let mut optimum = None;
                for mask in 0..(1usize << (n - 1)) {
                    let mut ends: Vec<_> =
                        (1..n).filter(|&end| mask & (1 << (end - 1)) != 0).collect();
                    ends.push(n);
                    let mut start = 0;
                    let loads: Vec<_> = ends
                        .iter()
                        .map(|&end| {
                            let value = 1 + heights[start..end].iter().sum::<u32>();
                            start = end;
                            value
                        })
                        .collect();
                    if loads.iter().any(|&l| l > 15) {
                        continue;
                    }
                    let candidate = (
                        ends.len(),
                        loads.iter().max().unwrap() - loads.iter().min().unwrap(),
                        ends,
                    );
                    if optimum.as_ref().is_none_or(|b| candidate < *b) {
                        optimum = Some(candidate);
                    }
                }
                assert_eq!(
                    balanced_errata_breaks(&heights, 1, 15).unwrap(),
                    optimum.unwrap().2,
                    "{heights:?}"
                );
            }
        }
    }

    fn fixture(page: u32) -> (GrayImage, PraefatioRegions) {
        let mut im = GrayImage::from_pixel(200, 1000, Luma([255]));
        let regions = PraefatioRegions {
            body: (20, 80, 160, 800),
            transition: (page == 15).then_some((500, 530, 100)),
        };
        let start = if page == 15 { 500 } else { 100 };
        for (i, top) in [start, start + 40, start + 80].into_iter().enumerate() {
            let left = if page == 15 && i == 0 { 110 } else { 30 };
            for y in top + 5..top + 20 {
                for x in left..left + 20 {
                    im.put_pixel(x, y, Luma([0]));
                }
            }
            im.put_pixel(left, top + 3, Luma([0]));
        }
        im.put_pixel(40, 30, Luma([0]));
        im.put_pixel(40, 950, Luma([0]));
        if page == 15 {
            for x in 30..90 {
                im.put_pixel(x, 510, Luma([0]));
            }
        }
        (im, regions)
    }
    fn dark(im: &GrayImage) -> usize {
        im.pixels().filter(|p| p[0] <= INK_THRESHOLD).count()
    }

    #[test]
    fn praefatio_first_errata_line_excludes_ages() {
        let (im, r) = fixture(15);
        let lines = extract_errata_lines(15, &im, r).unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].source_rect.0, 100);
        assert_eq!(dark(&lines[0].image), 301);
        assert_eq!(lines[0].image.get_pixel(10, 0)[0], 0);
    }
    #[test]
    fn praefatio_line_extraction_preserves_accents_and_short_lines() {
        let (im, r) = fixture(16);
        let lines = extract_errata_lines(16, &im, r).unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines.iter().map(|l| dark(&l.image)).sum::<usize>(), 903);
        assert!(lines.iter().all(|l| dark(&l.image) == 301));
    }
    #[test]
    fn praefatio_errata_preserves_source_page_order() {
        let (a, r) = fixture(15);
        let (b, s) = fixture(16);
        let joined = join_errata_pages(
            extract_errata_lines(15, &a, r).unwrap(),
            extract_errata_lines(16, &b, s).unwrap(),
        )
        .unwrap();
        assert_eq!(
            joined
                .iter()
                .map(|l| (l.source_page, l.line_index))
                .collect::<Vec<_>>(),
            [(15, 1), (15, 2), (15, 3), (16, 1), (16, 2), (16, 3)]
        );
        assert_eq!(joined.iter().map(|l| dark(&l.image)).sum::<usize>(), 1806);
    }
    #[test]
    fn praefatio_reviewed_intervals_reject_overlap_and_uncovered_ink() {
        let im = GrayImage::from_pixel(2, 20, Luma([0]));
        assert!(validate_intervals(
            &im,
            &[
                Band { top: 0, bottom: 15 },
                Band {
                    top: 10,
                    bottom: 20
                }
            ],
            16
        )
        .is_err());
        assert!(validate_intervals(&im, &[Band { top: 0, bottom: 19 }], 16).is_err());
        validate_intervals(&im, &[Band { top: 0, bottom: 20 }], 16).unwrap();
    }
    #[test]
    fn praefatio_page_join_adds_one_normal_gap() {
        let (a, r) = fixture(15);
        let (b, s) = fixture(16);
        let first = extract_errata_lines(15, &a, r).unwrap();
        let second = extract_errata_lines(16, &b, s).unwrap();
        let before: u32 = first.iter().chain(&second).map(|l| l.image.height()).sum();
        let joined = join_errata_pages(first, second).unwrap();
        assert_eq!(
            joined.iter().map(|l| l.image.height()).sum::<u32>() - before,
            23
        );
    }

    #[test]
    #[ignore = "requires source PDF and Poppler; writes crop inventory for visual review"]
    fn praefatio_source_line_inventory() {
        let out = std::path::Path::new("tmp/praefatio/lines");
        std::fs::create_dir_all(out).unwrap();
        let mut inventory = String::new();
        for page in [15, 16] {
            let png = crate::render::render_page(
                std::path::Path::new("Ploutarchou_Bioi_v2.pdf"),
                page,
                crate::render::TRANSCRIBE_DPI,
                out,
            )
            .unwrap();
            let im = image::open(png).unwrap().to_luma8();
            let regions = praefatio_regions(im.width(), im.height(), page).unwrap();
            let lines = extract_errata_lines(page, &im, regions).unwrap();
            println!("p{page}: {} lines", lines.len());
            for line in lines {
                let name = format!("p{page}-{:02}.png", line.line_index);
                line.image.save(out.join(&name)).unwrap();
                inventory.push_str(&format!(
                    "{name}: {:?}; automatic accent grouping\n",
                    line.source_rect
                ));
            }
        }
        std::fs::write(out.join("inventory.txt"), inventory).unwrap();
    }
}
