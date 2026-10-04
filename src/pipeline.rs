//! Top-level orchestration: pick the sidecar, classify, transcribe, reconcile
//! (all sharing one usage tracker), then assemble Lives, split volumes, emit the
//! inner `.tex` and imposition wrappers, and run the two-stage compile. A complete
//! fresh sidecar makes zero network calls.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

use crate::assemble::detect_lives;
use crate::compile;
use crate::latex::{self, Fonts, VolumePlan};
use crate::llm::batch::BatchBackend;
use crate::llm::instrument::UsageTracker;
use crate::llm::transcribe::{transcribe_all, DIVERGENCE_SIGNAL};
use crate::llm::Vision;
use crate::pages::{classify, is_boundary, BOUNDARY_SCAN};
use crate::render::{
    file_hash, render_latin_column, render_page, Pdf, THUMBNAIL_DPI, TRANSCRIBE_DPI,
};
use crate::tags::{PageRecord, PageRole, PageStatus, Sidecar, SourcePdf};
use crate::verify::{self, reconcile_all};
use crate::volumes::{
    self, split_into_volumes, unit_pages, volume_physical_pages, MAX_PAGES, TARGET_PAGES,
};

/// Parsed command-line inputs for a run.
#[derive(Debug, Clone)]
pub struct RunArgs {
    pub v1: PathBuf,
    pub v2: PathBuf,
    pub out: PathBuf,
    pub retag: bool,
    pub dry_run: Option<u32>,
    pub pages: Option<String>,
    pub no_compile: bool,
}

/// Which pages a run covers.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Scope {
    /// The whole work: classify every source page, then transcribe all body and
    /// front-matter prose.
    Full,
    /// The first N body pages of v1, for a cheap test.
    DryRun(u32),
    /// A source-qualified page range, e.g. `v1:40-60`.
    Pages { source: usize, start: u32, end: u32 },
}

impl RunArgs {
    fn restricted(&self) -> bool {
        self.dry_run.is_some() || self.pages.is_some()
    }

    fn scope(&self) -> Result<Scope> {
        if let Some(n) = self.dry_run {
            return Ok(Scope::DryRun(n));
        }
        if let Some(spec) = &self.pages {
            return parse_pages(spec);
        }
        Ok(Scope::Full)
    }

    fn sidecar_path(&self) -> PathBuf {
        if self.restricted() {
            PathBuf::from("plutarch.dryrun.tags.json")
        } else {
            PathBuf::from("plutarch.tags.json")
        }
    }
}

/// Parse a `--pages v1:A-B` / `v2:A-B` spec.
fn parse_pages(spec: &str) -> Result<Scope> {
    let (src, range) = spec
        .split_once(':')
        .ok_or_else(|| anyhow!("--pages must look like v1:40-60"))?;
    let source = match src {
        "v1" => 0,
        "v2" => 1,
        other => bail!("--pages source must be v1 or v2, got {other:?}"),
    };
    let (a, b) = range
        .split_once('-')
        .ok_or_else(|| anyhow!("--pages range must look like 40-60"))?;
    let start: u32 = a.trim().parse().context("parsing --pages start")?;
    let end: u32 = b.trim().parse().context("parsing --pages end")?;
    if start == 0 || end < start {
        bail!("--pages range {start}-{end} is empty or 1-based-invalid");
    }
    Ok(Scope::Pages { source, start, end })
}

/// Run the whole pipeline. `vision_factory` is called lazily, only when some LLM
/// work actually remains, so a fully-cached offline run never constructs it; it
/// yields both the synchronous classifier (`Vision`) and the batch backend used by
/// transcription and escalation, built from one shared client.
pub fn run(
    args: &RunArgs,
    vision_factory: impl FnOnce() -> Result<(Box<dyn Vision>, Box<dyn BatchBackend>)>,
) -> Result<()> {
    let scope = args.scope()?;
    let sidecar_path = args.sidecar_path();
    let source_paths: Vec<PathBuf> = vec![args.v1.clone(), args.v2.clone()];

    let mut sidecar = load_or_init(&sidecar_path, &source_paths, args.retag)?;
    seed_restricted_scope(&mut sidecar, &scope);

    let tracker = UsageTracker::new();

    if args.retag || has_llm_work(&sidecar, &scope) {
        let (vision, backend) = vision_factory()?;
        run_llm_phases(
            vision.as_ref(),
            backend.as_ref(),
            &mut sidecar,
            &tracker,
            &sidecar_path,
            &source_paths,
            &scope,
        )?;
    }

    // Deterministic, offline assembly and layout.
    sidecar.lives = detect_lives(&sidecar);
    if sidecar.lives.is_empty() && scope != Scope::Full {
        // A restricted run may cover body pages that contain no Life heading, so no
        // unit is detected. Synthesize one so the sample still renders a PDF.
        if let Some(sample) = sample_life(&sidecar) {
            sidecar.lives.push(sample);
        }
    }
    sidecar.save(&sidecar_path)?;

    let units = unit_pages(&sidecar, &sidecar.lives);
    let volumes = split_into_volumes(&units, TARGET_PAGES, MAX_PAGES)?;
    let plans = build_plans(&units, &volumes);

    let fonts = Fonts::default();
    let inner_paths =
        latex::write_inner_volumes(&sidecar, &plans, &args.out, &fonts, |source, page| {
            let name = latex::front_image_name(source, page);
            let png = render_page(&source_paths[source], page, PRINT_DPI, &args.out)?;
            std::fs::rename(&png, args.out.join(&name))
                .with_context(|| format!("naming front image {name}"))?;
            Ok(name)
        })?;

    let compiling = !args.no_compile && compile::lualatex_available();
    if compiling {
        compile_volumes(&args.out, &inner_paths)?;
    } else {
        write_wrappers_from_estimate(&args.out, &units, &volumes, &plans, &inner_paths)?;
    }

    println!("Sidecar: {}", sidecar_path.display());
    println!("Volumes: {}", plans.len());
    println!("{}", verify::summary(&sidecar));
    println!("{}", tracker.report());
    Ok(())
}

/// DPI for embedded front-matter scans.
const PRINT_DPI: u32 = 300;

/// A single synthetic unit spanning every transcribed body page, so a restricted
/// run with no detected Life heading still renders its transcription into a PDF.
fn sample_life(sidecar: &Sidecar) -> Option<crate::tags::LifeBoundary> {
    let indices: Vec<usize> = sidecar
        .pages
        .iter()
        .enumerate()
        .filter(|(_, p)| p.role == PageRole::Body && p.result.is_some())
        .map(|(i, _)| i)
        .collect();
    let first = *indices.first()?;
    let last = *indices.last()?;
    Some(crate::tags::LifeBoundary {
        id: "sample".to_string(),
        title: "SAMPLE".to_string(),
        kind: crate::tags::LifeKind::Life,
        group: 1,
        first_page: first,
        last_page: last,
    })
}

/// The LLM phases, sharing one usage tracker: classify (synchronous Haiku),
/// transcribe (batched Sonnet + optional Haiku), reconcile (free checks + batched
/// Opus on flagged pages). Body pages render the cropped Latin column; front-matter
/// prose renders the full page.
fn run_llm_phases(
    vision: &dyn Vision,
    backend: &dyn BatchBackend,
    sidecar: &mut Sidecar,
    tracker: &UsageTracker,
    sidecar_path: &Path,
    source_paths: &[PathBuf],
    scope: &Scope,
) -> Result<()> {
    if *scope == Scope::Full {
        classify(vision, sidecar, tracker, |source, page| {
            let png = render_page(&source_paths[source], page, THUMBNAIL_DPI, &scratch_dir())?;
            std::fs::read(&png).with_context(|| format!("reading thumbnail {}", png.display()))
        })?;
        sidecar.save(sidecar_path)?;
    }

    let limit = match scope {
        Scope::DryRun(n) => Some(*n),
        _ => None,
    };
    let in_scope = |source: usize, page: u32| page_in_scope(scope, source, page);
    let render_body = |source: usize, page: u32| -> Result<Vec<u8>> {
        let png = render_latin_column(&source_paths[source], page, TRANSCRIBE_DPI, &scratch_dir())?;
        std::fs::read(&png).with_context(|| format!("reading Latin column {}", png.display()))
    };
    let render_full = |source: usize, page: u32| -> Result<Vec<u8>> {
        let png = render_page(&source_paths[source], page, TRANSCRIBE_DPI, &scratch_dir())?;
        std::fs::read(&png).with_context(|| format!("reading page image {}", png.display()))
    };
    transcribe_all(
        backend,
        sidecar,
        tracker,
        sidecar_path,
        limit,
        in_scope,
        DIVERGENCE_SIGNAL,
        render_body,
        render_full,
    )?;

    reconcile_all(
        backend,
        sidecar,
        tracker,
        sidecar_path,
        DIVERGENCE_SIGNAL,
        render_body,
        render_full,
        |source, page| ocr_text(&source_paths[source], page),
    )?;
    Ok(())
}

/// A working directory for rendered page images that do not belong in the output.
pub(crate) fn scratch_dir() -> PathBuf {
    std::env::temp_dir().join("plutarch-render")
}

/// OCR a body page's **Latin column** with Tesseract-`lat`, returning `None` when
/// Tesseract is not available so the cross-check is simply skipped. The crop must
/// match what Sonnet transcribed: OCRing the full two-column page would read the
/// Greek too and roughly double the char count, flagging nearly every page.
fn ocr_text(pdf: &Path, page: u32) -> Result<Option<String>> {
    let png = render_latin_column(pdf, page, TRANSCRIBE_DPI, &scratch_dir())?;
    let bin = std::env::var("PLUTARCH_TESSERACT").unwrap_or_else(|_| "tesseract".to_string());
    let output = std::process::Command::new(bin)
        .arg(&png)
        .arg("stdout")
        .arg("-l")
        .arg("lat")
        .output();
    match output {
        Ok(o) if o.status.success() => Ok(Some(String::from_utf8_lossy(&o.stdout).into_owned())),
        _ => Ok(None),
    }
}

/// Whether a page is covered by the scope's transcription range.
fn page_in_scope(scope: &Scope, source: usize, page: u32) -> bool {
    match scope {
        Scope::Full => true,
        Scope::DryRun(_) => source == 0,
        Scope::Pages {
            source: s,
            start,
            end,
        } => source == *s && (*start..=*end).contains(&page),
    }
}

/// Load the sidecar if it is present, fresh, and not forced; otherwise initialize
/// a new one with the source page counts and hashes.
pub(crate) fn load_or_init(
    sidecar_path: &Path,
    source_paths: &[PathBuf],
    retag: bool,
) -> Result<Sidecar> {
    if !retag && sidecar_path.exists() {
        let sidecar = Sidecar::load(sidecar_path)?;
        let refs: Vec<&Path> = source_paths.iter().map(|p| p.as_path()).collect();
        if !sidecar.is_source_stale(&refs) {
            return Ok(sidecar);
        }
    }
    init_sidecar(source_paths)
}

/// Build a fresh sidecar: open each source for its page count and hash. The hash
/// is taken first, which reads the whole file and so forces a OneDrive
/// Files-On-Demand placeholder to hydrate before poppler opens it.
fn init_sidecar(source_paths: &[PathBuf]) -> Result<Sidecar> {
    let mut sources = Vec::with_capacity(source_paths.len());
    for path in source_paths {
        let file_sha256 = file_hash(path)?;
        let pdf = Pdf::open(path)?;
        sources.push(SourcePdf {
            path: path.to_string_lossy().into_owned(),
            page_count: pdf.page_count,
            file_sha256,
        });
    }
    Ok(Sidecar {
        schema_version: crate::tags::SCHEMA_VERSION,
        sources,
        front_matter: Default::default(),
        pages: Vec::new(),
        lives: Vec::new(),
        batches: Vec::new(),
    })
}

/// For a restricted run, seed the in-scope pages as body so classification can be
/// skipped: a dry-run targets the first N body (middle) pages of v1, a `--pages`
/// range the given span.
fn seed_restricted_scope(sidecar: &mut Sidecar, scope: &Scope) {
    let seed = |sidecar: &mut Sidecar, source: usize, page: u32| {
        if sidecar.page(source, page).is_none() {
            sidecar.pages.push(PageRecord {
                source,
                source_page: page,
                image_sha256: String::new(),
                role: PageRole::Body,
                pass_a: None,
                pass_b: None,
                status: PageStatus::NotBody,
                flags: Vec::new(),
                result: None,
                result_model: None,
            });
        }
    };
    match scope {
        Scope::Full => {}
        Scope::DryRun(n) => {
            let first_body = BOUNDARY_SCAN + 1;
            for page in first_body..first_body + n {
                seed(sidecar, 0, page);
            }
        }
        Scope::Pages { source, start, end } => {
            for page in *start..=*end {
                seed(sidecar, *source, page);
            }
        }
    }
    sidecar.pages.sort_by_key(|p| (p.source, p.source_page));
}

/// Whether any classify/transcribe/reconcile work remains for the scope.
fn has_llm_work(sidecar: &Sidecar, scope: &Scope) -> bool {
    if *scope == Scope::Full {
        for (si, src) in sidecar.sources.iter().enumerate() {
            for page in 1..=src.page_count {
                if is_boundary(page, src.page_count) && sidecar.page(si, page).is_none() {
                    return true;
                }
            }
        }
    }
    sidecar.pages.iter().any(|p| {
        matches!(p.role, PageRole::Body | PageRole::FrontProse)
            && page_in_scope(scope, p.source, p.source_page)
            && (p.result.is_none()
                || (p.pass_a.is_some() && p.pass_b.is_some() && p.result.is_none()))
    })
}

/// Build one [`VolumePlan`] per volume, charging each source's front matter to its
/// first output volume.
fn build_plans(units: &[volumes::UnitPages], volumes: &[Vec<String>]) -> Vec<VolumePlan> {
    let mut seen_sources: Vec<usize> = Vec::new();
    let mut plans = Vec::with_capacity(volumes.len());
    for (i, ids) in volumes.iter().enumerate() {
        let source = ids
            .first()
            .and_then(|id| units.iter().find(|u| &u.id == id))
            .map(|u| u.source)
            .unwrap_or(0);
        let is_first_of_source = !seen_sources.contains(&source);
        if is_first_of_source {
            seen_sources.push(source);
        }
        plans.push(VolumePlan {
            tomus: i + 1,
            source,
            is_first_of_source,
            life_ids: ids.clone(),
        });
    }
    plans
}

/// Compile each volume: one inner pass, read the real page count, emit the
/// wrapper, impose it.
fn compile_volumes(out: &Path, inner_paths: &[PathBuf]) -> Result<()> {
    let mut inner_page_counts = Vec::with_capacity(inner_paths.len());
    for (i, inner) in inner_paths.iter().enumerate() {
        let inner_name = file_name(inner);
        let inner_stem = file_stem(inner);
        println!(
            "  Tomus {:>2}: compiling inner ({} pass)…",
            i + 1,
            compile::INNER_PASSES
        );
        compile::compile(out, &inner_name, compile::INNER_PASSES)?;
        let pages = compile::inner_page_count(out, &inner_stem)?;
        inner_page_counts.push(pages);
        println!("           {inner_stem}.pdf: {pages} pages");
    }
    let wrappers = latex::write_imposition_wrappers(inner_paths.len(), &inner_page_counts, out)?;
    for wrapper in &wrappers {
        let name = file_name(wrapper);
        println!("           imposing {name}…");
        compile::compile(out, &name, 1)?;
    }
    Ok(())
}

/// When not compiling, write wrappers from an estimated inner page count and print
/// the manual two-stage compile instructions.
fn write_wrappers_from_estimate(
    out: &Path,
    units: &[volumes::UnitPages],
    volumes: &[Vec<String>],
    plans: &[VolumePlan],
    inner_paths: &[PathBuf],
) -> Result<()> {
    let counts: Vec<u32> = volumes
        .iter()
        .zip(plans)
        .map(|(ids, plan)| {
            let content: u32 = ids
                .iter()
                .filter_map(|id| units.iter().find(|u| &u.id == id))
                .map(|u| u.pages)
                .sum();
            volume_physical_pages(
                content,
                plan.source,
                plan.is_first_of_source,
                plan.tomus == 1,
            )
        })
        .collect();
    latex::write_imposition_wrappers(volumes.len(), &counts, out)?;
    for (i, inner) in inner_paths.iter().enumerate() {
        println!(
            "  Tomus {:>2}: ~{} inner pages — {}",
            i + 1,
            counts[i],
            inner.display()
        );
    }
    println!(
        "\nlualatex not run. Compile each volume with:\n  \
         lualatex tomus_N_inner.tex\n  \
         lualatex tomus_N.tex\n\
         If the compiled inner page count differs from the estimate, regenerate so \
         the wrapper matches."
    );
    Ok(())
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tags::{Block, BlockKind, LifeBoundary, LifeKind, PageDoc};

    #[test]
    fn parse_pages_reads_source_and_range() {
        assert_eq!(
            parse_pages("v1:40-60").unwrap(),
            Scope::Pages {
                source: 0,
                start: 40,
                end: 60
            }
        );
        assert_eq!(
            parse_pages("v2:1-5").unwrap(),
            Scope::Pages {
                source: 1,
                start: 1,
                end: 5
            }
        );
        assert!(parse_pages("v3:1-5").is_err());
        assert!(parse_pages("v1:5-1").is_err());
    }

    #[test]
    fn restricted_run_uses_dryrun_sidecar() {
        let mut args = sample_args();
        assert_eq!(args.sidecar_path(), PathBuf::from("plutarch.tags.json"));
        args.dry_run = Some(3);
        assert_eq!(
            args.sidecar_path(),
            PathBuf::from("plutarch.dryrun.tags.json")
        );
    }

    fn sample_args() -> RunArgs {
        RunArgs {
            v1: PathBuf::from("v1.pdf"),
            v2: PathBuf::from("v2.pdf"),
            out: PathBuf::from("build"),
            retag: false,
            dry_run: None,
            pages: None,
            no_compile: true,
        }
    }

    fn heading_page(source: usize, page: u32, title: &str) -> PageRecord {
        PageRecord {
            source,
            source_page: page,
            image_sha256: "h".to_string(),
            role: PageRole::Body,
            pass_a: Some(PageDoc {
                blocks: vec![mk(BlockKind::Heading, title)],
            }),
            pass_b: Some(PageDoc {
                blocks: vec![mk(BlockKind::Heading, title)],
            }),
            status: PageStatus::Agreed,
            flags: Vec::new(),
            result: Some(PageDoc {
                blocks: vec![
                    mk(BlockKind::Heading, title),
                    mk(BlockKind::Paragraph, "text"),
                ],
            }),
            result_model: Some(crate::llm::SONNET.to_string()),
        }
    }

    fn mk(kind: BlockKind, text: &str) -> Block {
        Block {
            kind,
            roman: None,
            section_no: None,
            continues_previous: false,
            text: text.to_string(),
        }
    }

    /// A complete sidecar for two tiny sources (every page present and reconciled),
    /// structured so the forced v1→v2 break is valid.
    fn complete_sidecar() -> Sidecar {
        let pages = vec![
            heading_page(0, 1, "THESEUS"),
            heading_page(0, 2, "ROMULUS"),
            heading_page(0, 3, "COMPARATIO THESEI ET ROMULI"),
            heading_page(1, 1, "NICIAS"),
            heading_page(1, 2, "CRASSUS"),
            heading_page(1, 3, "COMPARATIO NICIAE ET CRASSI"),
        ];
        Sidecar {
            schema_version: crate::tags::SCHEMA_VERSION,
            sources: vec![
                SourcePdf {
                    path: "v1.pdf".into(),
                    page_count: 3,
                    file_sha256: "a".into(),
                },
                SourcePdf {
                    path: "v2.pdf".into(),
                    page_count: 3,
                    file_sha256: "b".into(),
                },
            ],
            front_matter: Default::default(),
            pages,
            lives: Vec::new(),
            batches: Vec::new(),
        }
    }

    #[test]
    fn run_offline_with_fresh_sidecar_makes_no_network_calls() {
        // The vision factory is constructed only when work remains or --retag is
        // set. A complete sidecar leaves nothing to do, so no network client is
        // ever built — the guard `run` uses.
        let sidecar = complete_sidecar();
        assert!(!has_llm_work(&sidecar, &Scope::Full));
    }

    #[test]
    fn run_offline_usage_report_is_empty() {
        // With no calls recorded, the end-of-run report lists no models.
        let tracker = UsageTracker::new();
        let report = tracker.report();
        assert!(report.models.is_empty());
    }

    #[test]
    fn incomplete_sidecar_has_work() {
        let mut sidecar = complete_sidecar();
        sidecar.pages[0].result = None; // a page still needs reconciling
        assert!(has_llm_work(&sidecar, &Scope::Full));
    }

    #[test]
    fn build_plans_charges_front_matter_once_per_source() {
        let units = vec![
            crate::volumes::UnitPages {
                id: "a".into(),
                group: 1,
                kind: LifeKind::Life,
                source: 0,
                pages: 10,
            },
            crate::volumes::UnitPages {
                id: "b".into(),
                group: 2,
                kind: LifeKind::Life,
                source: 1,
                pages: 10,
            },
        ];
        let volumes = vec![vec!["a".to_string()], vec!["b".to_string()]];
        let plans = build_plans(&units, &volumes);
        assert!(plans[0].is_first_of_source);
        assert_eq!(plans[0].source, 0);
        assert!(plans[1].is_first_of_source);
        assert_eq!(plans[1].source, 1);
    }

    #[test]
    fn detect_lives_populates_after_offline_run() {
        // Sanity: detect_lives over the complete sidecar yields the six units, and
        // they split into volumes without tripping the boundary invariant.
        let sidecar = complete_sidecar();
        let lives: Vec<LifeBoundary> = detect_lives(&sidecar);
        assert_eq!(lives.len(), 6);
        let units = unit_pages(&sidecar, &lives);
        let volumes = split_into_volumes(&units, TARGET_PAGES, MAX_PAGES).unwrap();
        let plans = build_plans(&units, &volumes);
        // v1 and v2 never share a volume.
        assert!(plans.iter().any(|p| p.source == 0 && p.is_first_of_source));
        assert!(plans.iter().any(|p| p.source == 1 && p.is_first_of_source));
    }
}
