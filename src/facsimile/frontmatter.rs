//! Front-matter transcription for the facsimile path. The body is reproduced as
//! pixels, while the Monitum and Praefatio Latin prose and derived title pages are
//! re-typeset to the trim measure. Corrigenda are reproduced from source pixels.
//! This reuses the
//! transcription pipeline's stages — classification, Sonnet/Haiku transcription,
//! Opus reconciliation — admitting **only** `FrontProse` pages, so no body page is
//! ever sent to the API.
//!
//! The result is cached in `plutarch.frontmatter.tags.json` with the same semantics
//! as the body cache: a complete, non-stale sidecar makes zero network calls, and
//! human proofreading corrections edited into it are never regenerated over.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::llm::batch::BatchBackend;
use crate::llm::instrument::UsageTracker;
use crate::llm::transcribe::{transcribe_all, DIVERGENCE_SIGNAL};
use crate::llm::Vision;
use crate::pages::{classify, is_boundary};
use crate::pipeline::{load_or_init, scratch_dir};
use crate::render::{render_page, Pdf, THUMBNAIL_DPI, TRANSCRIBE_DPI};
use crate::tags::{PageRole, Sidecar};
use crate::verify::reconcile_all_in_scope;

/// The committed, hand-editable front-matter cache.
pub const SIDECAR: &str = "plutarch.frontmatter.tags.json";

/// Read the existing prose cache for an offline build, preserving incomplete
/// records and every cached transcription. A stale cache cannot supply prose.
pub fn load_cached_front_matter(path: &Path, source_paths: &[PathBuf]) -> Result<Sidecar> {
    let sidecar = Sidecar::load(path)?;
    let sources: Vec<&Path> = source_paths.iter().map(PathBuf::as_path).collect();
    if sidecar.is_source_stale(&sources) {
        bail!("front-matter cache is stale; offline build cannot refresh it");
    }
    Ok(sidecar)
}

/// Whether the front-matter sidecar still needs an API call: a boundary page is
/// unclassified, or a non-image `FrontProse` page has no finalized transcription.
pub fn work_remains(sidecar: &Sidecar) -> bool {
    for (si, src) in sidecar.sources.iter().enumerate() {
        for page in 1..=src.page_count {
            if is_boundary(page, src.page_count) && sidecar.page(si, page).is_none() {
                return true;
            }
        }
    }
    sidecar.pages.iter().any(|p| {
        p.role == PageRole::FrontProse
            && !is_front_image_page(p.source, p.source_page)
            && p.result.is_none()
    })
}

/// Classify the boundary regions, then transcribe and reconcile **only** the
/// non-image `FrontProse` pages. Body pages are excluded by `in_scope`, so none is rendered or
/// sent; `render_thumb`/`render_full` are injected so tests run without poppler.
pub fn transcribe_front_matter(
    sidecar: &mut Sidecar,
    sidecar_path: &Path,
    tracker: &UsageTracker,
    vision: &dyn Vision,
    backend: &dyn BatchBackend,
    render_thumb: impl Fn(usize, u32) -> Result<Vec<u8>>,
    render_full: impl Fn(usize, u32) -> Result<Vec<u8>>,
) -> Result<()> {
    validate_praefatio_candidate_state(sidecar)?;
    let prepared =
        |source, page| prepare_front_prose_image(source, page, render_full(source, page)?);
    classify(vision, sidecar, tracker, &render_thumb)?;
    sidecar.save(sidecar_path)?;

    let front: HashSet<(usize, u32)> = sidecar
        .pages
        .iter()
        .filter(|p| p.role == PageRole::FrontProse && !is_front_image_page(p.source, p.source_page))
        .map(|p| (p.source, p.source_page))
        .collect();
    let in_scope = |source: usize, page: u32| front.contains(&(source, page));
    // Front matter transcribes no body pages; a body render here is a logic error.
    let render_body = |source: usize, page: u32| -> Result<Vec<u8>> {
        bail!("facsimile front matter must not render body page {source}:{page}")
    };

    transcribe_all(
        backend,
        sidecar,
        tracker,
        sidecar_path,
        None,
        in_scope,
        DIVERGENCE_SIGNAL,
        render_body,
        prepared,
    )?;
    reconcile_all_in_scope(
        backend,
        sidecar,
        tracker,
        sidecar_path,
        DIVERGENCE_SIGNAL,
        render_body,
        prepared,
        |_, _| Ok(None),
        in_scope,
    )?;
    Ok(())
}

/// Load or build the front-matter sidecar, transcribing the front matter when work
/// remains. `vision_factory` is called lazily — only when a boundary page is
/// unclassified or a non-image `FrontProse` page lacks a result — so a complete, non-stale
/// cache constructs no client and makes zero network calls.
pub fn run_front_matter(
    source_paths: &[PathBuf],
    retag: bool,
    tracker: &UsageTracker,
    vision_factory: impl FnOnce() -> Result<(Box<dyn Vision>, Box<dyn BatchBackend>)>,
) -> Result<Sidecar> {
    let sidecar_path = PathBuf::from(SIDECAR);
    let mut sidecar = load_or_init(&sidecar_path, source_paths, retag)?;
    validate_praefatio_candidate_state(&sidecar)?;
    if retag || work_remains(&sidecar) {
        let (vision, backend) = vision_factory()?;
        let paths = source_paths.to_vec();
        let render_thumb = |source: usize, page: u32| -> Result<Vec<u8>> {
            let png = render_page(&paths[source], page, THUMBNAIL_DPI, &scratch_dir())?;
            std::fs::read(&png).map_err(Into::into)
        };
        let render_full = |source: usize, page: u32| -> Result<Vec<u8>> {
            let png = render_page(&paths[source], page, TRANSCRIBE_DPI, &scratch_dir())?;
            std::fs::read(&png).map_err(Into::into)
        };
        transcribe_front_matter(
            &mut sidecar,
            &sidecar_path,
            tracker,
            vision.as_ref(),
            backend.as_ref(),
            render_thumb,
            render_full,
        )?;
    }
    Ok(sidecar)
}

/// Source page re-typeset through the Monitum prose sentinel, on Tomus I.
pub const MONITUM_PROSE_PAGES: [(usize, u32); 1] = [(0, 12)];
/// Printed phrase in the final prose block, before the embedded errata.
pub const MONITUM_PROSE_SENTINEL: &str = "mutanda esse significavit.";
/// Source pages reproduced entirely as images, excluded from transcription.
pub const MONITUM_IMAGE_PAGES: [(usize, u32); 1] = [(0, 13)];
/// Historical cache records spanning the complete source Praefatio.
#[cfg(test)]
pub const PRAEFATIO_PAGES: [(usize, u32); 4] = [(1, 14), (1, 15), (1, 16), (1, 17)];

pub(crate) const PRAEFATIO_OPENING_PAGES: [(usize, u32); 2] = [(1, 14), (1, 15)];
pub(crate) const PRAEFATIO_CLOSING_PAGES: [(usize, u32); 1] = [(1, 17)];
pub(crate) const PRAEFATIO_IMAGE_PAGES: [(usize, u32); 1] = [(1, 16)];
pub(crate) const PRAEFATIO_PROSE_SENTINEL: &str =
    "Ita, ut in duarum vitarum exemplis consistam, Ages.";

/// Editorial body bounds and the shared prose/corrigenda boundary, in pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PraefatioRegions {
    pub body: (u32, u32, u32, u32),
    pub transition: Option<(u32, u32, u32)>,
}

pub(crate) fn praefatio_regions(width: u32, height: u32, page: u32) -> Result<PraefatioRegions> {
    let (left, top, right, bottom) = match page {
        15 => (0.145, 0.074, 0.919, 0.907),
        16 => (0.078, 0.078, 0.852, 0.916),
        _ => bail!("praefatio source 1, source page {page}: no editorial region"),
    };
    let x = |fraction: f64| (width as f64 * fraction).round() as u32;
    let y = |fraction: f64| (height as f64 * fraction).round() as u32;
    let (l, t, r, b) = (x(left), y(top), x(right), y(bottom));
    if l >= r || t >= b || r > width || b > height {
        bail!("praefatio source 1, source page {page}: empty or invalid body bounds");
    }
    let transition =
        (page == 15).then(|| (y(1595.0 / 2827.0), y(1654.0 / 2827.0), x(795.0 / 1829.0)));
    if let Some((start, end, split)) = transition {
        if start < t || start >= end || end > b || split <= l || split >= r {
            bail!("praefatio source 1, source page {page}: invalid transition bounds");
        }
    }
    Ok(PraefatioRegions {
        body: (l, t, r - l, b - t),
        transition,
    })
}

/// Whether source pixels supply the page's editorial content instead of prose.
pub(crate) fn is_front_image_page(source: usize, page: u32) -> bool {
    MONITUM_IMAGE_PAGES.contains(&(source, page)) || PRAEFATIO_IMAGE_PAGES.contains(&(source, page))
}

pub(crate) fn prepare_front_prose_image(
    source: usize,
    page: u32,
    bytes: Vec<u8>,
) -> Result<Vec<u8>> {
    if (source, page) != (1, 15) {
        return Ok(bytes);
    }
    let mut image = image::load_from_memory(&bytes)?.to_rgba8();
    let regions = praefatio_regions(image.width(), image.height(), page)?;
    let (left, top, width, height) = regions.body;
    let (_, bottom, split) = regions.transition.unwrap();
    let transition_top = regions.transition.unwrap().0;
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        if x < left
            || x >= left + width
            || y < top
            || y >= top + height
            || y >= bottom
            || (y >= transition_top && x >= split)
        {
            *pixel = image::Rgba([255, 255, 255, 255]);
        }
    }
    let mut output = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image).write_to(&mut output, image::ImageFormat::Png)?;
    Ok(output.into_inner())
}

fn validate_praefatio_prose_boundary(doc: &crate::tags::PageDoc) -> Result<()> {
    let nonempty: Vec<_> = doc
        .blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| !b.text.trim().is_empty())
        .collect();
    let count: usize = doc
        .blocks
        .iter()
        .map(|b| b.text.matches(PRAEFATIO_PROSE_SENTINEL).count())
        .sum();
    if count != 1
        || !nonempty
            .last()
            .is_some_and(|(_, b)| b.text.ends_with(PRAEFATIO_PROSE_SENTINEL))
    {
        let block = doc
            .blocks
            .iter()
            .position(|b| b.text.contains(PRAEFATIO_PROSE_SENTINEL))
            .unwrap_or(0);
        bail!("praefatio source 1, source page 15, block {block}: expected a unique prose sentinel ending the final nonempty block; restore a reviewed finalized prose prefix in the cache, or deliberately regenerate using --retag. Preserve a copy first: --retag resets the entire front-matter sidecar and discards corrections and candidate evidence");
    }
    Ok(())
}

pub(crate) fn validate_praefatio_prose_results(sidecar: &Sidecar) -> Result<()> {
    for (source, page) in PRAEFATIO_OPENING_PAGES
        .into_iter()
        .chain(PRAEFATIO_CLOSING_PAGES)
    {
        if let Some(doc) = sidecar.page(source, page).and_then(|r| r.result.as_ref()) {
            crate::llm::transcribe::validate_pagedoc_text(doc)
                .with_context(|| format!("praefatio source {source}, source page {page}"))?;
            if page == 15 {
                validate_praefatio_prose_boundary(doc)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_praefatio_candidate_state(sidecar: &Sidecar) -> Result<()> {
    if let Some(record) = sidecar.page(1, 15).filter(|r| r.result.is_none()) {
        for (name, doc) in [("pass_a", &record.pass_a), ("pass_b", &record.pass_b)] {
            if let Some(doc) = doc {
                validate_praefatio_prose_boundary(doc)
                    .with_context(|| format!("legacy praefatio candidate {name}"))?;
            }
        }
    }
    Ok(())
}

pub(crate) fn validate_praefatio_results(sidecar: &Sidecar) -> Result<()> {
    validate_praefatio_prose_results(sidecar)
}
/// Edge fractions `(left, top, right, bottom)` cropped off each embedded front scan.
/// Tuned after a real-eyes check of the compiled PDFs; adjust if a watermark remnant
/// survives or type is over-cropped.
const QR_CROP: (f32, f32, f32, f32) = (0.04, 0.04, 0.04, 0.04);
const VOLUMEN_CROP: (f32, f32, f32, f32) = (0.08, 0.03, 0.08, 0.07);
const COLOPHON_CROP: (f32, f32, f32, f32) = (0.10, 0.33, 0.10, 0.47);
/// v1 p12 errata tail, below the prose and above the signature and folio.
const MONITUM_A_CROP: (f32, f32, f32, f32) = (0.13, 0.769, 0.185, 0.109);
/// v1 p13 errata, without the running head or ornament.
const MONITUM_B_CROP: (f32, f32, f32, f32) = (0.18, 0.094, 0.14, 0.350);

/// The embedded reproduction-notice (QR) scan for a source: source page 1.
pub fn front_qr_name(source: usize) -> String {
    format!("front_qr_s{source}.png")
}

/// The embedded engraved Volumen-title scan for a source.
pub fn front_volumen_name(source: usize) -> String {
    format!("front_volumen_s{source}.png")
}

/// The source page the engraved Volumen title sits on: v1 p10, v2 p12.
pub fn front_volumen_page(source: usize) -> u32 {
    if source == 0 {
        10
    } else {
        12
    }
}

/// The embedded printer's-colophon scan (v1 p9, Tomus I only).
pub fn front_colophon_name() -> &'static str {
    "front_colophon.png"
}

/// The embedded Monitum errata-tail scan (v1 p12), on Tomus I.
pub fn front_monitum_a_name() -> &'static str {
    "front_monitum_a.png"
}

/// The embedded Monitum errata scan (v1 p13), on Tomus I.
pub fn front_monitum_b_name() -> &'static str {
    "front_monitum_b.png"
}

/// The pixel sub-rectangle of a `w×h` image left after removing the given edge
/// fractions `(left, top, right, bottom)`.
fn crop_rect(w: u32, h: u32, frac: (f32, f32, f32, f32)) -> (u32, u32, u32, u32) {
    let (l, t, r, b) = frac;
    let x = (w as f32 * l).round() as u32;
    let y = (h as f32 * t).round() as u32;
    let cw = ((w as f32 * (1.0 - l - r)).round() as u32).max(1);
    let ch = ((h as f32 * (1.0 - t - b)).round() as u32).max(1);
    (
        x,
        y,
        cw.min(w.saturating_sub(x)).max(1),
        ch.min(h.saturating_sub(y)).max(1),
    )
}

/// Render one source page at transcription DPI, crop it to the fractional box, and
/// write the named cropped PNG into `out_dir`. The uncropped render goes to a
/// scratch dir so it does not linger among the build deliverables.
fn render_and_crop(
    pdf: &Pdf,
    page: u32,
    frac: (f32, f32, f32, f32),
    name: &str,
    out_dir: &Path,
) -> Result<()> {
    let png = render_page(&pdf.path, page, TRANSCRIBE_DPI, &scratch_dir())?;
    let img = image::open(&png).map_err(|e| anyhow::anyhow!("decoding {}: {e}", png.display()))?;
    let (x, y, cw, ch) = crop_rect(img.width(), img.height(), frac);
    img.crop_imm(x, y, cw, ch).save(out_dir.join(name))?;
    Ok(())
}

/// Render into `out_dir` the embedded front-matter scans a given tomus needs: the
/// reproduction-notice (QR) page on every tomus, the engraved Volumen title on the
/// first tomus of each source, and the printer's colophon and Monitum errata on Tomus I. Each is
/// cropped in Rust to a watermark-free box; no network is required.
pub fn render_front_scans(
    source: usize,
    is_first_of_source: bool,
    is_tomus_one: bool,
    pdf: &Pdf,
    out_dir: &Path,
) -> Result<()> {
    render_and_crop(pdf, 1, QR_CROP, &front_qr_name(source), out_dir)?;
    if is_first_of_source {
        render_and_crop(
            pdf,
            front_volumen_page(source),
            VOLUMEN_CROP,
            &front_volumen_name(source),
            out_dir,
        )?;
    }
    if is_tomus_one {
        // The colophon is v1 p9; Tomus I is always source 0, so `pdf` is the v1 PDF.
        render_and_crop(pdf, 9, COLOPHON_CROP, front_colophon_name(), out_dir)?;
        render_and_crop(pdf, 12, MONITUM_A_CROP, front_monitum_a_name(), out_dir)?;
        render_and_crop(pdf, 13, MONITUM_B_CROP, front_monitum_b_name(), out_dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn work_remains_ignores_praefatio_image_page() {
        assert!(is_front_image_page(1, 16));
        assert!(is_front_image_page(0, 13));
        assert!(!is_front_image_page(1, 15));
        let mut sc = committed_praefatio();
        sc.page_mut(1, 16).unwrap().result = None;
        assert!(!work_remains(&sc));
    }

    #[test]
    fn transcribe_and_reconcile_exclude_praefatio_image_page() {
        let pages = [(1, 14), (1, 15), (1, 16), (1, 17)];
        let scope: Vec<_> = pages
            .into_iter()
            .filter(|&(s, p)| !is_front_image_page(s, p))
            .collect();
        assert_eq!(scope, [(1, 14), (1, 15), (1, 17)]);
        let mut sc = committed_praefatio();
        sc.page_mut(1, 16).unwrap().result = None;
        sc.page_mut(1, 17).unwrap().result = None;
        sc.page_mut(1, 17).unwrap().pass_a = None;
        sc.page_mut(1, 17).unwrap().pass_b = None;
        let original = sc.page(1, 16).unwrap().clone();
        let backend = StubBackend::new();
        let vision = CountingVision {
            calls: RefCell::new(0),
        };
        let dir = tempfile::tempdir().unwrap();
        transcribe_front_matter(
            &mut sc,
            &dir.path().join(SIDECAR),
            &UsageTracker::new(),
            &vision,
            &backend,
            |_, _| panic!("unexpected classification"),
            |s, p| {
                assert_eq!((s, p), (1, 17));
                Ok(vec![4, 5, 6])
            },
        )
        .unwrap();
        assert_eq!(sc.page(1, 16).unwrap(), &original);
        assert!(backend
            .submitted
            .borrow()
            .iter()
            .all(|id| !id.ends_with("_16")));
    }

    #[test]
    fn praefatio_mask_preserves_prose_and_removes_errata() {
        let mut raw = image::RgbaImage::from_pixel(1829, 2827, image::Rgba([255; 4]));
        for (x, y) in [
            (790, 1620),
            (805, 1620),
            (400, 1700),
            (400, 100),
            (400, 2800),
            (400, 1000),
        ] {
            raw.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
        }
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(raw)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        let renderer = |s, p| prepare_front_prose_image(s, p, bytes.get_ref().clone());
        let masked = image::load_from_memory(&renderer(1, 15).unwrap())
            .unwrap()
            .to_rgba8();
        assert_eq!(masked.dimensions(), (1829, 2827));
        for (x, y) in [(790, 1620), (400, 1000)] {
            assert_eq!(masked.get_pixel(x, y).0, [0, 0, 0, 255]);
        }
        for (x, y) in [(805, 1620), (400, 1700), (400, 100), (400, 2800)] {
            assert_eq!(masked.get_pixel(x, y).0, [255; 4]);
        }
        assert_eq!(renderer(0, 12).unwrap(), *bytes.get_ref());
    }

    fn migrated_cache() -> Sidecar {
        let mut sc = committed_praefatio();
        let text = &mut sc.page_mut(1, 15).unwrap().result.as_mut().unwrap().blocks[0].text;
        let end = text.find(PRAEFATIO_PROSE_SENTINEL).unwrap() + PRAEFATIO_PROSE_SENTINEL.len();
        text.truncate(end);
        sc
    }

    #[test]
    fn praefatio_validation_rejects_unmigrated_mixed_result() {
        let mut legacy = committed_praefatio();
        legacy
            .page_mut(1, 15)
            .unwrap()
            .result
            .as_mut()
            .unwrap()
            .blocks[0]
            .text
            .push_str(" c. III, 5");
        let err = validate_praefatio_prose_results(&legacy).unwrap_err();
        assert!(format!("{err:#}").contains("source page 15, block 0"));
        validate_praefatio_prose_results(&migrated_cache()).unwrap();
    }

    #[test]
    fn praefatio_validation_ignores_image_only_result() {
        let mut sc = migrated_cache();
        sc.page_mut(1, 16).unwrap().result.as_mut().unwrap().blocks[0].text =
            "***END OF JSON***".into();
        validate_praefatio_prose_results(&sc).unwrap();
    }

    #[test]
    fn praefatio_missing_result_rejects_legacy_candidates_before_api() {
        let mut sc = committed_praefatio();
        sc.page_mut(1, 15).unwrap().result = None;
        let before = sc.clone();
        let err = validate_praefatio_candidate_state(&sc).unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("--retag") && message.contains("entire front-matter sidecar"));
        assert_eq!(sc, before);
        let backend = StubBackend::new();
        let vision = CountingVision {
            calls: RefCell::new(0),
        };
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SIDECAR);
        assert!(transcribe_front_matter(
            &mut sc,
            &path,
            &UsageTracker::new(),
            &vision,
            &backend,
            |_, _| panic!("legacy guard must precede classification"),
            |_, _| panic!("legacy guard must precede rendering")
        )
        .is_err());
        assert_eq!(*backend.calls.borrow(), 0);
        assert_eq!(sc, before);
        assert!(!path.exists());
        let record = sc.page_mut(1, 15).unwrap();
        record.pass_a = None;
        record.pass_b = None;
        validate_praefatio_candidate_state(&sc).unwrap();
    }

    #[test]
    fn praefatio_valid_result_preserves_candidates_when_other_page_needs_work() {
        let mut sc = migrated_cache();
        sc.page_mut(1, 17).unwrap().result = None;
        let before = sc.clone();
        validate_praefatio_candidate_state(&sc).unwrap();
        assert_eq!(sc, before);
        let record = sc.page(1, 15).unwrap().clone();
        let backend = StubBackend::new();
        let vision = CountingVision {
            calls: RefCell::new(0),
        };
        let dir = tempfile::tempdir().unwrap();
        transcribe_front_matter(
            &mut sc,
            &dir.path().join(SIDECAR),
            &UsageTracker::new(),
            &vision,
            &backend,
            |_, _| panic!("unexpected classification"),
            |s, p| {
                assert_eq!((s, p), (1, 17));
                Ok(vec![4, 5, 6])
            },
        )
        .unwrap();
        assert_eq!(sc.page(1, 15).unwrap(), &record);
    }

    #[test]
    fn praefatio_cache_opening_ends_at_ages() {
        let sc = committed_praefatio();
        let record = sc.page(1, 15).unwrap();
        let text = &record.result.as_ref().unwrap().blocks[0].text;
        assert!(text.ends_with(PRAEFATIO_PROSE_SENTINEL));
        assert!(record.pass_a.as_ref().unwrap().blocks[1]
            .text
            .starts_with(text));
        assert!(record.pass_a.as_ref().unwrap().blocks[1]
            .text
            .contains("Ages. c. III, 5"));
    }

    #[test]
    fn praefatio_complete_cache_never_constructs_vision_client() {
        let sources = [
            PathBuf::from("Ploutarchou_Bioi_v1.pdf"),
            PathBuf::from("Ploutarchou_Bioi_v2.pdf"),
        ];
        if sources.iter().any(|p| !p.exists()) {
            return;
        }
        let before = std::fs::read(SIDECAR).unwrap();
        let sc = run_front_matter(&sources, false, &UsageTracker::new(), || {
            panic!("complete cache must not construct a client")
        })
        .unwrap();
        assert!(!work_remains(&sc));
        validate_praefatio_results(&sc).unwrap();
        assert_eq!(std::fs::read(SIDECAR).unwrap(), before);
    }

    #[test]
    fn praefatio_both_api_stages_receive_identical_masked_images() {
        let mut sc = committed_praefatio();
        let rec = sc.page_mut(1, 15).unwrap();
        rec.result = None;
        rec.pass_a = None;
        rec.pass_b = None;
        let backend = StubBackend::new();
        let good = serde_json::to_string(&crate::tags::PageDoc {
            blocks: vec![crate::tags::Block {
                kind: crate::tags::BlockKind::Paragraph,
                roman: None,
                section_no: None,
                continues_previous: true,
                text: PRAEFATIO_PROSE_SENTINEL.into(),
            }],
        })
        .unwrap();
        backend
            .replies
            .borrow_mut()
            .insert("s_1_15".into(), format!(r#"{{"blocks":[{{"kind":"section","section_no":1,"text":"Synthetic opening"}},{{"kind":"section","section_no":3,"text":"{}"}}]}}"#, PRAEFATIO_PROSE_SENTINEL));
        backend.replies.borrow_mut().insert(
            "h_1_15".into(),
            good.replace("Ita, ut", "Aliud longe diversum et suspectum"),
        );
        backend.replies.borrow_mut().insert("o_1_15".into(), good);
        let raw = image::GrayImage::from_pixel(1829, 2827, image::Luma([0]));
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageLuma8(raw)
            .write_to(&mut png, image::ImageFormat::Png)
            .unwrap();
        let expected =
            crate::b64(&prepare_front_prose_image(1, 15, png.get_ref().clone()).unwrap());
        let vision = CountingVision {
            calls: RefCell::new(0),
        };
        let dir = tempfile::tempdir().unwrap();
        transcribe_front_matter(
            &mut sc,
            &dir.path().join(SIDECAR),
            &UsageTracker::new(),
            &vision,
            &backend,
            |_, _| panic!("unexpected classification"),
            |s, p| {
                assert_eq!((s, p), (1, 15));
                Ok(png.get_ref().clone())
            },
        )
        .unwrap();
        let requests = backend.requests.borrow();
        assert!(requests.iter().any(|r| r.custom_id == "o_1_15"));
        for request in requests.iter() {
            let images: Vec<_> = request
                .content
                .iter()
                .filter_map(|c| match c {
                    crate::llm::ContentBlock::Image { png_base64 } => Some(png_base64),
                    _ => None,
                })
                .collect();
            assert_eq!(images, vec![&expected]);
        }
        assert_eq!(
            sc.page(1, 15).unwrap().image_sha256,
            crate::render::hash_bytes(&prepare_front_prose_image(1, 15, png.into_inner()).unwrap())
        );
    }
    #[test]
    fn praefatio_regions_scale_with_render_dimensions() {
        let a = super::praefatio_regions(1829, 2827, 15).unwrap();
        let b = super::praefatio_regions(3658, 5654, 15).unwrap();
        assert_eq!(a.transition, Some((1595, 1654, 795)));
        assert_eq!(b.transition, Some((3190, 3308, 1590)));
        for page in [15, 16] {
            let region = super::praefatio_regions(1829, 2827, page).unwrap();
            let (x, y, w, h) = region.body;
            assert!(x + w <= 1829 && y + h <= 2827);
        }
        assert_eq!(super::PRAEFATIO_OPENING_PAGES, [(1, 14), (1, 15)]);
        assert_eq!(super::PRAEFATIO_CLOSING_PAGES, [(1, 17)]);
        assert_eq!(super::PRAEFATIO_IMAGE_PAGES, [(1, 16)]);
        assert!(super::PRAEFATIO_PROSE_SENTINEL.ends_with("Ages."));
    }

    #[test]
    fn praefatio_regions_reject_invalid_bounds() {
        for (w, h, page) in [(0, 2827, 15), (1829, 0, 16), (1, 1, 15), (1829, 2827, 14)] {
            assert!(super::praefatio_regions(w, h, page).is_err());
        }
    }
    #[test]
    fn praefatio_validation_rejects_contaminated_final_result() {
        let mut rec = record(1, 17, PageRole::FrontProse, true);
        rec.result.as_mut().unwrap().blocks[0].text = "***END OF JSON***".into();
        let err = validate_praefatio_results(&sidecar(17, vec![rec])).unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("source 1, source page 17"));
        assert!(message.contains("block 0"));
    }

    #[test]
    fn praefatio_validation_ignores_historical_passes() {
        let mut rec = record(1, 16, PageRole::FrontProse, true);
        rec.pass_a.as_mut().unwrap().blocks[0].text = "***END OF JSON***".into();
        validate_praefatio_results(&sidecar(17, vec![rec])).unwrap();
        validate_praefatio_results(&committed_praefatio()).unwrap();
    }
    fn committed_praefatio() -> crate::tags::Sidecar {
        crate::tags::Sidecar::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SIDECAR))
            .unwrap()
    }

    #[test]
    fn praefatio_cache_migration_preserves_prefix_and_candidates() {
        let mut original = committed_praefatio();
        let historical = original
            .page(1, 15)
            .unwrap()
            .pass_a
            .as_ref()
            .unwrap()
            .blocks[1]
            .text
            .clone();
        original
            .page_mut(1, 15)
            .unwrap()
            .result
            .as_mut()
            .unwrap()
            .blocks[0]
            .text = historical;
        let mut migrated = original.clone();
        let old = &original
            .page(1, 15)
            .unwrap()
            .result
            .as_ref()
            .unwrap()
            .blocks[0]
            .text;
        assert_eq!(old.matches(PRAEFATIO_PROSE_SENTINEL).count(), 1);
        let end = old.find(PRAEFATIO_PROSE_SENTINEL).unwrap() + PRAEFATIO_PROSE_SENTINEL.len();
        let record = migrated.page_mut(1, 15).unwrap();
        record.result.as_mut().unwrap().blocks[0].text.truncate(end);
        assert!(record.result.as_ref().unwrap().blocks[0].continues_previous);
        let mut expected = serde_json::to_value(&original).unwrap();
        let index = original
            .pages
            .iter()
            .position(|p| (p.source, p.source_page) == (1, 15))
            .unwrap();
        expected["pages"][index]["result"]["blocks"][0]["text"] = old[..end].into();
        assert_eq!(serde_json::to_value(&migrated).unwrap(), expected);
        assert!(old[end..].starts_with(" c. III, 5"));
        println!("Reviewed p15 prefix: {}", &old[..end]);
    }

    #[test]
    fn praefatio_cache_has_one_display_heading() {
        let sc = committed_praefatio();
        let headings: Vec<_> = PRAEFATIO_PAGES
            .iter()
            .flat_map(|&(source, page)| {
                &sc.page(source, page)
                    .unwrap()
                    .result
                    .as_ref()
                    .unwrap()
                    .blocks
            })
            .filter(|b| b.kind == crate::tags::BlockKind::Heading)
            .collect();
        assert_eq!(headings.len(), 1);
        assert_eq!(headings[0].text, "PRÆFATIO.");
    }

    #[test]
    fn praefatio_cache_marks_sentence_continuations() {
        let sc = committed_praefatio();
        for page in [15, 16] {
            let blocks = &sc.page(1, page).unwrap().result.as_ref().unwrap().blocks;
            assert_eq!(blocks[0].kind, crate::tags::BlockKind::Paragraph);
            assert!(blocks[0].continues_previous);
        }
        assert!(sc
            .page(1, 17)
            .unwrap()
            .result
            .as_ref()
            .unwrap()
            .blocks
            .iter()
            .all(|b| !b.continues_previous));
    }

    #[test]
    fn praefatio_cache_ends_page_sixteen_at_source_period() {
        let sc = committed_praefatio();
        let record = sc.page(1, 16).unwrap();
        let result = &record.result.as_ref().unwrap().blocks[0].text;
        let original = &record.pass_a.as_ref().unwrap().blocks[0].text;
        assert!(result.ends_with("cap. LXVIII, 4 e cod. 1676 dedi καθίσταντο."));
        assert!(original.starts_with(result));
        assert!(original[result.len()..].contains("***END OF JSON***"));
    }

    use super::*;
    use crate::llm::batch::{BatchOutcome, BatchRequest, BatchStatus};
    use crate::llm::instrument::Usage;
    use crate::llm::Completion;
    use crate::tags::{PageRecord, PageStatus, SourcePdf};
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// A batch backend that transcribes each submitted request with a fixed prose
    /// reply and records which `(source, page)` ids it was asked for.
    struct StubBackend {
        chunks: RefCell<HashMap<String, Vec<String>>>,
        submitted: RefCell<Vec<String>>,
        calls: RefCell<usize>,
        requests: RefCell<Vec<BatchRequest>>,
        replies: RefCell<HashMap<String, String>>,
    }

    impl StubBackend {
        fn new() -> Self {
            StubBackend {
                chunks: RefCell::new(HashMap::new()),
                submitted: RefCell::new(Vec::new()),
                calls: RefCell::new(0),
                requests: RefCell::new(Vec::new()),
                replies: RefCell::new(HashMap::new()),
            }
        }
    }

    impl BatchBackend for StubBackend {
        fn submit(&self, requests: &[BatchRequest]) -> Result<String> {
            self.requests.borrow_mut().extend_from_slice(requests);
            *self.calls.borrow_mut() += 1;
            self.submitted
                .borrow_mut()
                .extend(requests.iter().map(|r| r.custom_id.clone()));
            let id = format!("b{}", self.calls.borrow());
            self.chunks.borrow_mut().insert(
                id.clone(),
                requests.iter().map(|r| r.custom_id.clone()).collect(),
            );
            Ok(id)
        }

        fn poll(&self, _batch_id: &str) -> Result<BatchStatus> {
            Ok(BatchStatus::Ended)
        }

        fn results(&self, batch_id: &str) -> Result<HashMap<String, BatchOutcome>> {
            let ids = self
                .chunks
                .borrow()
                .get(batch_id)
                .cloned()
                .unwrap_or_default();
            Ok(ids
                .into_iter()
                .map(|id| {
                    let reply=self.replies.borrow().get(&id).cloned().unwrap_or_else(|| r#"{"blocks":[{"kind":"paragraph","text":"monitum prose transcribed faithfully"}]}"#.into());
                    (
                        id,
                        BatchOutcome::Ok(Completion {
                            text: reply,
                            usage: Usage {
                                input_tokens: 10,
                                output_tokens: 10,
                            },
                        }),
                    )
                })
                .collect())
        }
    }

    /// A vision classifier that records how many times it was called.
    struct CountingVision {
        calls: RefCell<usize>,
    }

    impl Vision for CountingVision {
        fn classify(&self, _png_base64: &str) -> Result<(PageRole, Usage)> {
            *self.calls.borrow_mut() += 1;
            Ok((PageRole::FrontProse, Usage::default()))
        }
        fn review_line(&self, _png_base64: &str) -> Result<(crate::llm::LineVerdict, Usage)> {
            Ok((crate::llm::LineVerdict::Body, Usage::default()))
        }
    }

    fn record(source: usize, page: u32, role: PageRole, done: bool) -> PageRecord {
        let result = done.then(|| crate::tags::PageDoc {
            blocks: vec![crate::tags::Block {
                kind: crate::tags::BlockKind::Paragraph,
                roman: None,
                section_no: None,
                continues_previous: false,
                text: "done".to_string(),
            }],
        });
        PageRecord {
            source,
            source_page: page,
            image_sha256: String::new(),
            role,
            pass_a: result.clone(),
            pass_b: None,
            status: if done {
                PageStatus::Agreed
            } else {
                PageStatus::NotBody
            },
            flags: Vec::new(),
            result,
            result_model: None,
        }
    }

    fn sidecar(page_count: u32, pages: Vec<PageRecord>) -> Sidecar {
        Sidecar {
            schema_version: crate::tags::SCHEMA_VERSION,
            sources: vec![SourcePdf {
                path: "v1.pdf".to_string(),
                page_count,
                file_sha256: "a".to_string(),
            }],
            front_matter: Default::default(),
            pages,
            lives: Vec::new(),
            batches: Vec::new(),
        }
    }

    #[test]
    fn crop_box_removes_fraction() {
        // A 1000×2000 image with 10% off left/right and 25%/15% off top/bottom.
        let (x, y, w, h) = crop_rect(1000, 2000, (0.10, 0.25, 0.10, 0.15));
        assert_eq!((x, y), (100, 500));
        assert_eq!(w, 800); // 1000 - 100 (left) - 100 (right)
        assert_eq!(h, 1200); // 2000 - 500 (top) - 300 (bottom)
                             // The sub-rectangle stays inside the image.
        assert!(x + w <= 1000 && y + h <= 2000);
    }

    #[test]
    fn work_remains_ignores_monitum_image_page() {
        let mut pages: Vec<_> = (1..=13)
            .map(|page| record(0, page, PageRole::Skip, false))
            .collect();
        pages[11] = record(0, 12, PageRole::FrontProse, true);
        pages[12] = record(0, 13, PageRole::FrontProse, false);
        let mut sc = sidecar(13, pages);
        assert!(!work_remains(&sc));
        sc.page_mut(0, 12).unwrap().result = None;
        assert!(work_remains(&sc));
    }

    #[test]
    fn transcribe_excludes_monitum_image_page() {
        // A cached first pass on an image page must be excluded from reconciliation too.
        for cached_pass in [false, true] {
            let mut pages: Vec<_> = (1..=13)
                .map(|page| record(0, page, PageRole::Skip, false))
                .collect();
            pages[11] = record(0, 12, PageRole::FrontProse, false);
            pages[12] = record(0, 13, PageRole::FrontProse, cached_pass);
            pages[12].result = None;
            let mut sc = sidecar(13, pages);
            let original = sc.page(0, 13).unwrap().clone();
            let dir = tempfile::tempdir().unwrap();
            let backend = StubBackend::new();
            let vision = CountingVision {
                calls: RefCell::new(0),
            };
            transcribe_front_matter(
                &mut sc,
                &dir.path().join(SIDECAR),
                &UsageTracker::new(),
                &vision,
                &backend,
                |_, _| panic!("classified a cached page"),
                |source, page| {
                    assert_eq!((source, page), (0, 12));
                    Ok(vec![4, 5, 6])
                },
            )
            .unwrap();
            let ids = backend.submitted.borrow();
            assert!(ids.iter().any(|id| id == "s_0_12"));
            assert!(!ids.iter().any(|id| id.contains("_13")));
            assert!(sc.page(0, 12).unwrap().result.is_some());
            assert_eq!(
                serde_json::to_value(sc.page(0, 13).unwrap()).unwrap(),
                serde_json::to_value(original).unwrap()
            );
            assert!(!work_remains(&sc));
        }
    }

    #[test]
    fn frontmatter_only_transcribes_frontprose() {
        // A classified source with one FrontProse and one Body page: only the
        // FrontProse page may be transcribed.
        let pages = vec![
            record(0, 2, PageRole::FrontProse, false),
            record(0, 3, PageRole::Body, false),
        ];
        let mut sc = sidecar(3, pages);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SIDECAR);
        let tracker = UsageTracker::new();
        let backend = StubBackend::new();
        let vision = CountingVision {
            calls: RefCell::new(0),
        };
        transcribe_front_matter(
            &mut sc,
            &path,
            &tracker,
            &vision,
            &backend,
            |_, _| Ok(vec![1, 2, 3]),
            |_, _| Ok(vec![4, 5, 6]),
        )
        .unwrap();
        // No submitted request targets the body page (source 0, page 3).
        assert!(
            !backend
                .submitted
                .borrow()
                .iter()
                .any(|id| id.contains("_3")),
            "a body page was submitted: {:?}",
            backend.submitted.borrow()
        );
        // The FrontProse page is finalized; the Body page is untouched.
        assert!(sc.page(0, 2).unwrap().result.is_some());
        assert!(sc.page(0, 3).unwrap().result.is_none());
        assert!(sc.page(0, 3).unwrap().pass_a.is_none());
    }

    #[test]
    fn frontmatter_offline_when_sidecar_fresh() {
        // Every page classified and the FrontProse page finalized: no classify call
        // and no batch submission.
        let pages = vec![
            record(0, 1, PageRole::Skip, false),
            record(0, 2, PageRole::FrontProse, true),
            record(0, 3, PageRole::Body, false),
        ];
        assert!(!work_remains(&sidecar(3, pages.clone())));
        let mut sc = sidecar(3, pages);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SIDECAR);
        let tracker = UsageTracker::new();
        let backend = StubBackend::new();
        let vision = CountingVision {
            calls: RefCell::new(0),
        };
        transcribe_front_matter(
            &mut sc,
            &path,
            &tracker,
            &vision,
            &backend,
            |_, _| Ok(vec![1, 2, 3]),
            |_, _| Ok(vec![4, 5, 6]),
        )
        .unwrap();
        assert_eq!(*vision.calls.borrow(), 0, "classified a fresh sidecar");
        assert_eq!(*backend.calls.borrow(), 0, "submitted a fresh sidecar");
    }

    #[test]
    fn offline_cache_preserves_incomplete_records_without_writing() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("v1.pdf");
        std::fs::write(&source, b"source bytes").unwrap();
        let path = dir.path().join(SIDECAR);
        let mut sc = sidecar(
            3,
            vec![
                record(0, 1, PageRole::FrontProse, true),
                record(0, 2, PageRole::FrontProse, false),
            ],
        );
        sc.sources[0].file_sha256 = crate::render::file_hash(&source).unwrap();
        sc.save(&path).unwrap();
        let before = std::fs::read(&path).unwrap();
        let cached = load_cached_front_matter(&path, std::slice::from_ref(&source)).unwrap();
        assert!(cached.page(0, 1).unwrap().result.is_some());
        assert!(cached.page(0, 2).unwrap().result.is_none());
        assert_eq!(std::fs::read(&path).unwrap(), before);
        std::fs::write(&source, b"different source bytes").unwrap();
        assert!(load_cached_front_matter(&path, &[source]).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}
