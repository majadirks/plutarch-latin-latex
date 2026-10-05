//! Front-matter transcription for the facsimile path. The body is reproduced as
//! pixels, while the Monitum prose (v1), Praefatio (v2), and derived title pages are
//! re-typeset to the trim measure. The Monitum errata are embedded source scans.
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
            && !MONITUM_IMAGE_PAGES.contains(&(p.source, p.source_page))
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
    classify(vision, sidecar, tracker, &render_thumb)?;
    sidecar.save(sidecar_path)?;

    let front: HashSet<(usize, u32)> = sidecar
        .pages
        .iter()
        .filter(|p| {
            p.role == PageRole::FrontProse
                && !MONITUM_IMAGE_PAGES.contains(&(p.source, p.source_page))
        })
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
        &render_full,
    )?;
    reconcile_all_in_scope(
        backend,
        sidecar,
        tracker,
        sidecar_path,
        DIVERGENCE_SIGNAL,
        render_body,
        &render_full,
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
/// Source pages re-typeset as the Praefatio, on v2's first tomus.
pub const PRAEFATIO_PAGES: [(usize, u32); 4] = [(1, 14), (1, 15), (1, 16), (1, 17)];

/// Validate finalized praefatio prose, retaining candidate passes as evidence.
pub(crate) fn validate_praefatio_results(sidecar: &Sidecar) -> Result<()> {
    for (source, page) in PRAEFATIO_PAGES {
        if let Some(doc) = sidecar
            .page(source, page)
            .and_then(|record| record.result.as_ref())
        {
            crate::llm::transcribe::validate_pagedoc_text(doc)
                .with_context(|| format!("praefatio source {source}, source page {page}"))?;
        }
    }
    Ok(())
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
    fn praefatio_validation_rejects_contaminated_final_result() {
        let mut rec = record(1, 16, PageRole::FrontProse, true);
        rec.result.as_mut().unwrap().blocks[0].text = "***END OF JSON***".into();
        let err = validate_praefatio_results(&sidecar(17, vec![rec])).unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("source 1, source page 16"));
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
    }

    impl StubBackend {
        fn new() -> Self {
            StubBackend {
                chunks: RefCell::new(HashMap::new()),
                submitted: RefCell::new(Vec::new()),
                calls: RefCell::new(0),
            }
        }
    }

    impl BatchBackend for StubBackend {
        fn submit(&self, requests: &[BatchRequest]) -> Result<String> {
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
                    (
                        id,
                        BatchOutcome::Ok(Completion {
                            text: r#"{"blocks":[{"kind":"paragraph","text":"monitum prose transcribed faithfully"}]}"#.to_string(),
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
