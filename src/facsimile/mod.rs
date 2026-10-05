//! The facsimile build path: instead of transcribing the Latin column, it crops
//! that column out of the source scans, slices it at inter-line whitespace, and
//! reflows the image strips into the trim text block. The body reproduction is the
//! original 1857 Didot type pixel-for-pixel; only the front matter is re-typeset
//! from vision transcription.
//!
//! The submodules split the work: [`units`] loads the hand-authored unit table,
//! [`columns`] and [`lines`] do the deterministic image analysis, [`reflow`]
//! composes body-page images, and [`assemble`] emits the inner `.tex`.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use image::GrayImage;

pub mod assemble;
pub mod columns;
pub mod corpus;
pub mod detect;
pub mod frontmatter;
pub mod lines;
pub mod metrics;
pub mod praefatio;
pub mod reflow;
pub mod review;
pub mod units;

use crate::compile;
use crate::latex::{self, Fonts};
use crate::llm::instrument::UsageTracker;
use crate::render::{self, Pdf};
use crate::tags::LifeKind;
use crate::volumes::{
    split_into_volumes, volume_physical_pages, UnitPages, MAX_PAGES, TARGET_PAGES,
};
use assemble::{write_inner, FacsimileVolume, PlacedUnit};
use reflow::TrimGeom;
use units::{Unit, UnitRange};

/// Parsed command-line inputs for a facsimile build.
#[derive(Debug, Clone)]
pub struct FacsimileArgs {
    pub v1: PathBuf,
    pub v2: PathBuf,
    pub out: PathBuf,
    /// Optional owner name for the bordered Tomus title-page plate.
    pub exlibris: Option<String>,
    /// Path to the auto-detected, hand-editable `facsimile_units.json` cache.
    pub units: PathBuf,
    pub no_compile: bool,
    /// Re-transcribe the front matter even when its sidecar is fresh.
    pub retag: bool,
    /// Read existing front-matter prose without invoking vision or updating its cache.
    pub offline: bool,
    /// Detect the unit table, write/refresh `facsimile_units.json`, and exit.
    pub detect_units: bool,
    /// Re-detect the unit table from the sources even when the cache is present,
    /// preserving `pinned` hand edits.
    pub redetect: bool,
    /// Restrict a build to a source-qualified page range, e.g. `v1:40-60`.
    pub pages: Option<String>,
    /// Build only the named unit, for a cheap single-unit test run.
    pub unit: Option<String>,
    /// Run the opt-in Claude Vision line review, writing pinned `skip_top_bands` /
    /// `review_notes` back into the unit table, then exit without building.
    pub review_lines: bool,
    /// Delete `<out>/render_cache/` at the start of the run, forcing needed pages
    /// to be re-rasterized. Off by default; the raster cache is otherwise preserved.
    pub clean_cache: bool,
}

impl FacsimileArgs {
    /// The two source PDFs, in order (index 0 = v1, index 1 = v2).
    pub fn source_paths(&self) -> [PathBuf; 2] {
        [self.v1.clone(), self.v2.clone()]
    }

    /// Whether this is a restricted (single-unit or page-range) run, which emits
    /// one volume rather than splitting.
    fn restricted(&self) -> bool {
        self.unit.is_some() || self.pages.is_some()
    }
}

/// Validate an owner name without changing its spaces or punctuation.
pub fn validate_exlibris_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        bail!("exlibris name must not be blank");
    }
    Ok(())
}

/// Run the facsimile build: compose each unit's body images at one global scale,
/// emit the inner `.tex` per volume, and (unless `--no-compile`) compile and
/// impose. Body rendering and image analysis are offline and deterministic.
pub fn run_facsimile(args: &FacsimileArgs) -> Result<()> {
    if let Some(name) = args.exlibris.as_deref() {
        validate_exlibris_name(name)?;
    }
    if args.offline && (args.retag || args.review_lines) {
        bail!("--offline cannot be combined with --retag or --review-lines");
    }
    let source_paths = args.source_paths();
    let pdfs = [Pdf::open(&source_paths[0])?, Pdf::open(&source_paths[1])?];
    let page_words = |source: usize, page: u32| render::page_words(&source_paths[source], page);
    let page_text = |source: usize, page: u32| render::page_text(&source_paths[source], page);

    // `--detect-units` writes/refreshes the cache from the sources and exits.
    if args.detect_units {
        let units = detect::load_or_detect(&args.units, &pdfs, true, page_text, page_words)?;
        println!("Detected {} units -> {}", units.len(), args.units.display());
        for u in &units {
            println!(
                "  [{}] {} p{:>3} {}",
                u.source,
                if u.kind == LifeKind::Comparatio {
                    "comp"
                } else {
                    "life"
                },
                u.start_page,
                u.latin_title
            );
        }
        return Ok(());
    }

    // The final unit of each source is bounded by its detected `last_page`; the page
    // count is the fallback when a unit carries none. Restricted runs never reach it.
    let last_body = [pdfs[0].page_count, pdfs[1].page_count];

    let mut units = if args.pages.is_some() {
        Vec::new()
    } else {
        detect::load_or_detect(&args.units, &pdfs, args.redetect, page_text, page_words)?
    };
    let ranges = select_ranges(args, &last_body, &units)?;

    std::fs::create_dir_all(&args.out)
        .with_context(|| format!("creating output dir {}", args.out.display()))?;
    // Clear the previous run's generated artifacts (not the raster cache) so an
    // orphan PNG or stale wrapper from a prior boundary cannot linger. Skipped for
    // restricted runs so a single-unit test does not wipe a full build.
    if !args.restricted() {
        clean_generated(&args.out)?;
    }
    if args.clean_cache {
        clean_render_cache(&args.out)?;
    }
    let cache = args.out.join("render_cache");
    std::fs::create_dir_all(&cache).with_context(|| format!("creating {}", cache.display()))?;

    let geom = TrimGeom::default();
    let dpi = geom.dpi;
    let render = |source: usize, page: u32| render_gray(&source_paths[source], page, dpi, &cache);

    // `--review-lines`: run the opt-in Claude Vision boundary review, write the pinned
    // `skip_top_bands` / `review_notes` back into the unit table, and exit. The normal
    // (no-flag) build below never constructs a Vision client.
    if args.review_lines {
        let tracker = UsageTracker::new();
        let ran = review::review_lines_pass(true, &ranges, &render, &tracker, &mut units, || {
            let secrets = crate::secrets::load(Path::new("secrets.json"))?;
            let client = crate::llm::Client::new(secrets.anthropic_api_key);
            Ok(Box::new(crate::llm::ApiVision::new(client)) as Box<dyn crate::llm::Vision>)
        })?;
        if ran {
            let json = serde_json::to_string_pretty(&units).context("serializing unit table")?;
            std::fs::write(&args.units, json)
                .with_context(|| format!("writing {}", args.units.display()))?;
            println!("{}", tracker.report());
            println!("Reviewed lines -> {}", args.units.display());
        }
        return Ok(());
    }

    // Pass 1: one global scale from every selected page's column width.
    let scale = global_scale_for(&ranges, geom.content_width(), &render);

    // Pass 2: compose each unit, write its body images, record its page count.
    let mut placed = Vec::new();
    let mut unit_pages = Vec::new();
    for range in &ranges {
        let (pages, metrics) = reflow::unit_body_strips(range, render)?;
        // Every unit opens with an inline chapter header, so its first page gets a
        // reduced budget to fit the header above the first image.
        let composed = reflow::compose_pages(
            &pages,
            scale,
            &geom,
            geom.first_body_budget(),
            geom.body_budget(),
            &metrics,
        );
        let mut image_names = Vec::with_capacity(composed.len());
        for (i, img) in composed.iter().enumerate() {
            let name = reflow::body_image_name(&range.unit.id, i + 1);
            img.save(args.out.join(&name))
                .with_context(|| format!("writing body image {name}"))?;
            image_names.push(name);
        }
        println!(
            "  {}: {} source page(s) -> {} trim page(s)",
            range.unit.id,
            range.end_page + 1 - range.unit.start_page,
            composed.len()
        );
        unit_pages.push(UnitPages {
            id: range.unit.id.clone(),
            group: range.unit.group,
            kind: range.unit.kind,
            source: range.unit.source,
            pages: composed.len() as u32,
        });
        placed.push(PlacedUnit {
            id: range.unit.id.clone(),
            latin_title: range.unit.latin_title.clone(),
            kind: range.unit.kind,
            image_names,
        });
    }

    let mut volumes = plan_volumes(&unit_pages, &placed, args.restricted())?;

    // Fill the re-typeset front matter (Monitum/Praefatio/title page) on each
    // source's first volume. Restricted runs carry no front matter.
    if !args.restricted() {
        fill_front_matter(&mut volumes, &pdfs, &source_paths, args)?;
    }

    let fonts = Fonts::default();
    let mut inner_paths = Vec::with_capacity(volumes.len());
    for vol in &volumes {
        inner_paths.push(write_inner(
            vol,
            &args.out,
            &fonts,
            args.exlibris.as_deref(),
        )?);
    }

    if !args.no_compile && compile::lualatex_available() {
        compile_and_impose(&args.out, &inner_paths)?;
    } else {
        write_wrappers_from_estimate(&args.out, &volumes)?;
    }

    println!("Volumes: {}", volumes.len());
    println!("Output: {}", args.out.display());
    Ok(())
}

/// Resolve the units to build into source page ranges: a single synthetic range for
/// a `--pages` run, the one named `--unit`, or the whole detected table.
fn select_ranges(
    args: &FacsimileArgs,
    last_body: &[u32; 2],
    units: &[Unit],
) -> Result<Vec<UnitRange>> {
    if let Some(spec) = &args.pages {
        let (source, start, end) = parse_pages(spec)?;
        return Ok(vec![UnitRange {
            unit: Unit {
                id: "pages".to_string(),
                latin_title: "SAMPLE".to_string(),
                greek_title: "—".to_string(),
                kind: LifeKind::Life,
                group: 1,
                source,
                start_page: start,
                skip_top_bands: Some(0),
                title_top_frac: None,
                body_top_frac: None,
                last_page: None,
                pinned: false,
                review_notes: None,
            },
            end_page: end,
            shared_tail: false,
            open_body_frac: None,
            tail_cut_frac: None,
        }]);
    }

    let all = units::ranges(units, last_body);
    match &args.unit {
        Some(id) => {
            let selected: Vec<UnitRange> = all.into_iter().filter(|r| &r.unit.id == id).collect();
            if selected.is_empty() {
                bail!("no unit in the table matches --unit {id:?}");
            }
            Ok(selected)
        }
        None => Ok(all),
    }
}

/// Parse a `--pages v1:A-B` / `v2:A-B` spec into `(source, start, end)`.
fn parse_pages(spec: &str) -> Result<(usize, u32, u32)> {
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
    Ok((source, start, end))
}

/// One global scale from the 95th-percentile column width across every selected
/// page, mapped to the usable measure. Pages that fail detection are skipped here
/// (and reported in the compose pass).
fn global_scale_for(
    ranges: &[UnitRange],
    content_w: u32,
    render: &impl Fn(usize, u32) -> Result<GrayImage>,
) -> f32 {
    let mut widths = Vec::new();
    for range in ranges {
        for page in range.unit.start_page..=range.end_page {
            if let Ok(img) = render(range.unit.source, page) {
                if let Ok(col) = columns::detect_latin_column(&img) {
                    widths.push(col.right - col.left);
                }
            }
        }
    }
    reflow::global_scale(&widths, content_w)
}

/// Render one page to grayscale, caching the PNG on disk so the width pass and the
/// compose pass each rasterize it at most once.
fn render_gray(pdf: &Path, page: u32, dpi: u32, cache: &Path) -> Result<GrayImage> {
    let stem = pdf
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "page".to_string());
    let png = cache.join(format!("{stem}-p{page:04}-{dpi}dpi.png"));
    if !png.exists() {
        render::render_page(pdf, page, dpi, cache)?;
    }
    let img = image::open(&png).with_context(|| format!("decoding {}", png.display()))?;
    Ok(img.to_luma8())
}

/// Group the composed units into volumes: one volume for a restricted run; a
/// balanced split (reusing the transcription path's splitter) otherwise.
fn plan_volumes(
    unit_pages: &[UnitPages],
    placed: &[PlacedUnit],
    restricted: bool,
) -> Result<Vec<FacsimileVolume>> {
    if restricted {
        let source = placed
            .first()
            .and_then(|p| unit_pages.iter().find(|u| u.id == p.id))
            .map(|u| u.source)
            .unwrap_or(0);
        return Ok(vec![FacsimileVolume {
            tomus: 1,
            source,
            is_first_of_source: true,
            units: placed.to_vec(),
            prose: String::new(),
            praefatio_errata_images: Vec::new(),
            emit_front_matter: false,
        }]);
    }

    let volumes = split_into_volumes(unit_pages, TARGET_PAGES, MAX_PAGES)?;
    let mut seen_sources: Vec<usize> = Vec::new();
    let mut out = Vec::with_capacity(volumes.len());
    for (i, ids) in volumes.iter().enumerate() {
        let source = ids
            .first()
            .and_then(|id| unit_pages.iter().find(|u| &u.id == id))
            .map(|u| u.source)
            .unwrap_or(0);
        let is_first_of_source = !seen_sources.contains(&source);
        if is_first_of_source {
            seen_sources.push(source);
        }
        let units: Vec<PlacedUnit> = ids
            .iter()
            .filter_map(|id| placed.iter().find(|p| &p.id == id).cloned())
            .collect();
        out.push(FacsimileVolume {
            tomus: i + 1,
            source,
            is_first_of_source,
            units,
            prose: String::new(),
            praefatio_errata_images: Vec::new(),
            emit_front_matter: true,
        });
    }
    Ok(out)
}

/// Render emitting volumes' scans and load or transcribe their cached prose.
/// An emitting Praefatio requires the complete prose cache and corrigenda assets;
/// unrelated volumes can continue with scans when prose loading fails.
fn fill_front_matter(
    volumes: &mut [FacsimileVolume],
    pdfs: &[Pdf; 2],
    source_paths: &[PathBuf; 2],
    args: &FacsimileArgs,
) -> Result<()> {
    // 1. Render each volume's scans (offline, no network).
    for vol in volumes.iter_mut().filter(|v| v.emit_front_matter) {
        frontmatter::render_front_scans(
            vol.source,
            vol.is_first_of_source,
            vol.tomus == 1,
            &pdfs[vol.source],
            &args.out,
        )?;
        if vol.is_first_of_source && vol.source == 1 {
            vol.praefatio_errata_images = praefatio::render_praefatio_errata(&pdfs[1], &args.out)?;
        }
    }

    // Load the cached prose or transcribe missing results through the shared pipeline.
    let tracker = UsageTracker::new();
    let factory = || {
        let secrets = crate::secrets::load(Path::new("secrets.json"))?;
        let client = crate::llm::Client::new(secrets.anthropic_api_key);
        let vision: Box<dyn crate::llm::Vision> =
            Box::new(crate::llm::ApiVision::new(client.clone()));
        let backend: Box<dyn crate::llm::batch::BatchBackend> =
            Box::new(crate::llm::batch::ApiBatchBackend::new(client));
        Ok((vision, backend))
    };
    let prose = if args.offline {
        frontmatter::load_cached_front_matter(Path::new(frontmatter::SIDECAR), source_paths)
    } else {
        frontmatter::run_front_matter(source_paths, args.retag, &tracker, factory)
    };
    let sidecar = match prose {
        Ok(sidecar) => sidecar,
        Err(err) => {
            if volumes
                .iter()
                .any(|v| v.emit_front_matter && v.is_first_of_source && v.source == 1)
            {
                return Err(err)
                    .context("emitting Praefatio requires its finalized Latin prose cache");
            }
            eprintln!("  front-matter prose skipped ({err}); volumes carry the scans but no re-typeset prose");
            return Ok(());
        }
    };
    assign_front_prose(volumes, &sidecar)
}

/// Validate and compose cached prose before assigning it to emitting volumes.
fn assign_front_prose(
    volumes: &mut [FacsimileVolume],
    sidecar: &crate::tags::Sidecar,
) -> Result<()> {
    if volumes
        .iter()
        .any(|vol| vol.emit_front_matter && vol.is_first_of_source && vol.source == 1)
    {
        frontmatter::validate_praefatio_results(sidecar)?;
    }
    let mut prepared = Vec::new();
    for (index, vol) in volumes
        .iter()
        .enumerate()
        .filter(|(_, vol)| vol.emit_front_matter)
    {
        let prose = if vol.tomus == 1 {
            latex::front_prose_pages_until(
                sidecar,
                &frontmatter::MONITUM_PROSE_PAGES,
                frontmatter::MONITUM_PROSE_SENTINEL,
            )
        } else if vol.is_first_of_source && vol.source == 1 {
            latex::front_praefatio(sidecar, &vol.praefatio_errata_images)?
        } else {
            String::new()
        };
        prepared.push((index, prose));
    }
    for (index, prose) in prepared {
        volumes[index].prose = prose;
    }
    Ok(())
}

/// Compile each inner, read its real page count, then write and compile the
/// imposition wrappers — reusing the transcription path's compile and imposition.
fn compile_and_impose(out: &Path, inner_paths: &[PathBuf]) -> Result<()> {
    let mut counts = Vec::with_capacity(inner_paths.len());
    for (i, inner) in inner_paths.iter().enumerate() {
        let name = file_name(inner);
        let stem = file_stem(inner);
        println!("  Tomus {:>2}: compiling inner…", i + 1);
        compile::compile(out, &name, compile::INNER_PASSES)?;
        let pages = compile::inner_page_count(out, &stem)?;
        println!("           {stem}.pdf: {pages} pages");
        counts.push(pages);
    }
    let wrappers = latex::write_imposition_wrappers(inner_paths.len(), &counts, out)?;
    for wrapper in &wrappers {
        let name = file_name(wrapper);
        println!("           imposing {name}…");
        compile::compile(out, &name, 1)?;
    }
    // The composed body and front-scan PNGs are now embedded in the inner PDFs and
    // nothing downstream reads them; remove them so the build dir holds only the
    // .tex/.pdf deliverables and render_cache/. This runs only on the compile branch;
    // the estimate branch leaves every PNG so a manual compile still works.
    remove_composed_pngs(out);
    Ok(())
}

/// Whether a file name in the output dir is a generated build artifact: a composed
/// body / front-scan PNG, or an inner / imposition `.tex`/`.pdf`. Excludes
/// `render_cache/` (a directory) and unrelated files.
fn is_generated_artifact(name: &str) -> bool {
    (name.ends_with(".png") && (name.starts_with("facsimile_") || name.starts_with("front_")))
        || (name.starts_with("tomus_") && (name.ends_with(".tex") || name.ends_with(".pdf")))
}

/// Remove the previous run's generated artifacts from `out` (composed / front PNGs,
/// inner `.tex`/`.pdf`, imposition wrappers), leaving `render_cache/` and any
/// unrelated files. Non-recursive, so files inside `render_cache/` are untouched.
fn clean_generated(out: &Path) -> Result<()> {
    if let Ok(entries) = std::fs::read_dir(out) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && is_generated_artifact(&entry.file_name().to_string_lossy()) {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
    Ok(())
}

/// Delete `<out>/render_cache/` entirely (the poppler raster cache). A no-op when
/// it does not exist, so `--clean-cache` is safe on a fresh output directory.
fn clean_render_cache(out: &Path) -> Result<()> {
    let cache = out.join("render_cache");
    if cache.exists() {
        std::fs::remove_dir_all(&cache).with_context(|| format!("removing {}", cache.display()))?;
    }
    Ok(())
}

/// Remove only the composed body / front-scan PNGs from `out`, keeping the
/// `.tex`/`.pdf` deliverables and `render_cache/`.
fn remove_composed_pngs(out: &Path) {
    if let Ok(entries) = std::fs::read_dir(out) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_file()
                && name.ends_with(".png")
                && (name.starts_with("facsimile_") || name.starts_with("front_"))
            {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

/// When not compiling, write the imposition wrappers from an estimated inner page
/// count (each unit's body images — the standalone title page is gone — charged with
/// the blank leaves and per-tomus front-matter overhead) and print the manual
/// compile instructions.
fn write_wrappers_from_estimate(out: &Path, volumes: &[FacsimileVolume]) -> Result<()> {
    let counts: Vec<u32> = volumes
        .iter()
        .map(|v| {
            let content: u32 = v.units.iter().map(|u| u.image_names.len() as u32).sum();
            volume_physical_pages(content, v.source, v.is_first_of_source, v.tomus == 1)
        })
        .collect();
    latex::write_imposition_wrappers(volumes.len(), &counts, out)?;
    for (i, count) in counts.iter().enumerate() {
        println!("  Tomus {:>2}: ~{count} inner pages", i + 1);
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
    use crate::render::pdftoppm_available;

    #[test]
    fn validate_exlibris_name_rejects_blank() {
        for name in ["", "   ", "\t"] {
            assert_eq!(
                validate_exlibris_name(name).unwrap_err().to_string(),
                "exlibris name must not be blank"
            );
        }
        assert!(validate_exlibris_name("Alex Smith").is_ok());
    }
    #[test]
    fn run_facsimile_rejects_blank_exlibris_before_pdf_access() {
        let dir = tempfile::tempdir().unwrap();
        let args = FacsimileArgs {
            v1: dir.path().join("missing1.pdf"),
            v2: dir.path().join("missing2.pdf"),
            out: dir.path().join("output"),
            units: dir.path().join("units.json"),
            exlibris: Some("\t".into()),
            no_compile: true,
            retag: false,
            offline: true,
            detect_units: false,
            redetect: false,
            pages: None,
            unit: None,
            review_lines: false,
            clean_cache: false,
        };
        assert_eq!(
            run_facsimile(&args).unwrap_err().to_string(),
            "exlibris name must not be blank"
        );
        assert!(!args.out.exists());
    }

    #[test]
    fn assign_front_prose_hybrid_failure_does_not_mutate_volumes() {
        let sc = crate::tags::Sidecar::load(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(frontmatter::SIDECAR),
        )
        .unwrap();
        let mut volumes = vec![
            FacsimileVolume {
                tomus: 1,
                source: 0,
                is_first_of_source: true,
                units: vec![],
                prose: "monitum unchanged".into(),
                praefatio_errata_images: vec![],
                emit_front_matter: true,
            },
            FacsimileVolume {
                tomus: 5,
                source: 1,
                is_first_of_source: true,
                units: vec![],
                prose: "praefatio unchanged".into(),
                praefatio_errata_images: vec![],
                emit_front_matter: true,
            },
        ];
        assert!(assign_front_prose(&mut volumes, &sc).is_err());
        assert_eq!(volumes[0].prose, "monitum unchanged");
        assert_eq!(volumes[1].prose, "praefatio unchanged");
    }

    #[test]
    fn praefatio_images_are_excluded_from_other_and_restricted_volumes() {
        let mut volumes = vec![
            FacsimileVolume {
                tomus: 6,
                source: 1,
                is_first_of_source: false,
                units: vec![],
                prose: String::new(),
                praefatio_errata_images: vec![],
                emit_front_matter: true,
            },
            FacsimileVolume {
                tomus: 5,
                source: 1,
                is_first_of_source: true,
                units: vec![],
                prose: String::new(),
                praefatio_errata_images: vec![],
                emit_front_matter: false,
            },
        ];
        let sc = crate::tags::Sidecar::load(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(frontmatter::SIDECAR),
        )
        .unwrap();
        assign_front_prose(&mut volumes, &sc).unwrap();
        for vol in &volumes {
            assert!(vol.praefatio_errata_images.is_empty());
            assert!(
                !assemble::inner_document(vol, &latex::Fonts::default(), None)
                    .contains("praefatio_errata")
            );
        }
        let restricted = plan_volumes(
            &[unit_pages("nicias", 1, LifeKind::Life, 1, 2)],
            &[placed("nicias", 1)],
            true,
        )
        .unwrap();
        assert!(!restricted[0].emit_front_matter);
        assert!(restricted[0].praefatio_errata_images.is_empty());
    }

    #[test]
    #[ignore = "requires complete offline QA output and its UTF-8 build log"]
    fn praefatio_full_build_matches_current_volume_estimate() {
        let log = std::fs::read_to_string("tmp/praefatio/full-build.log").unwrap();
        let pattern =
            regex::Regex::new(r"(?m)^  ([a-z_]+): \d+ source page\(s\) -> (\d+) trim page\(s\)")
                .unwrap();
        let counts: std::collections::HashMap<_, _> = pattern
            .captures_iter(&log)
            .map(|c| (c[1].to_string(), c[2].parse::<u32>().unwrap()))
            .collect();
        let metadata = units::load_units(Path::new("facsimile_units.json")).unwrap();
        assert_eq!(counts.len(), metadata.len());
        let unit_pages: Vec<_> = metadata
            .iter()
            .map(|u| UnitPages {
                id: u.id.clone(),
                group: u.group,
                kind: u.kind,
                source: u.source,
                pages: counts[&u.id],
            })
            .collect();
        let volumes = split_into_volumes(&unit_pages, TARGET_PAGES, MAX_PAGES).unwrap();
        for (index, ids) in volumes.iter().enumerate() {
            let doc = std::fs::read_to_string(format!(
                "build_facsimile/praefatio_hybrid_check/tomus_{}_inner.tex",
                index + 1
            ))
            .unwrap();
            for u in &metadata {
                assert_eq!(
                    doc.contains(&format!("facsimile_{}_1.png", u.id)),
                    ids.contains(&u.id),
                    "tomus {} unit {}",
                    index + 1,
                    u.id
                );
            }
        }
    }

    #[test]
    fn assign_front_prose_rejects_poisoned_praefatio_without_mutating_volumes() {
        let mut sc = crate::tags::Sidecar::load(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(frontmatter::SIDECAR),
        )
        .unwrap();
        sc.page_mut(1, 17).unwrap().result.as_mut().unwrap().blocks[0]
            .text
            .push_str("***END OF JSON***");
        let mut volumes = vec![FacsimileVolume {
            tomus: 5,
            source: 1,
            is_first_of_source: true,
            units: vec![],
            prose: "unchanged".into(),
            praefatio_errata_images: vec!["errata.png".into()],
            emit_front_matter: true,
        }];
        assert!(assign_front_prose(&mut volumes, &sc).is_err());
        assert_eq!(volumes[0].prose, "unchanged");
        volumes[0].emit_front_matter = false;
        assign_front_prose(&mut volumes, &sc).unwrap();
        assert_eq!(volumes[0].prose, "unchanged");
    }

    #[test]
    fn assign_front_prose_composes_finalized_praefatio() {
        let sc = crate::tags::Sidecar::load(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(frontmatter::SIDECAR),
        )
        .unwrap();
        let mut volumes = vec![FacsimileVolume {
            tomus: 5,
            source: 1,
            is_first_of_source: true,
            units: vec![],
            prose: String::new(),
            praefatio_errata_images: vec!["errata.png".into()],
            emit_front_matter: true,
        }];
        assign_front_prose(&mut volumes, &sc).unwrap();
        let prose = &volumes[0].prose;
        assert_eq!(prose.matches("\\heading{PRÆFATIO.}").count(), 1);
        assert!(prose.contains("confirmatas in ordinem recepi"));
        assert!(!prose.contains("consentiente ipso Plutarcho"));
        assert!(prose.contains("errata.png"));
        assert!(!prose.contains("END OF JSON"));
        let expected = prose.clone();
        let mut historical = sc.clone();
        historical
            .page_mut(1, 16)
            .unwrap()
            .result
            .as_mut()
            .unwrap()
            .blocks[0]
            .text = "***END OF JSON***".into();
        assign_front_prose(&mut volumes, &historical).unwrap();
        assert_eq!(volumes[0].prose, expected);
    }

    fn placed(id: &str, images: usize) -> PlacedUnit {
        PlacedUnit {
            id: id.to_string(),
            latin_title: id.to_uppercase(),
            kind: LifeKind::Life,
            image_names: (1..=images)
                .map(|n| reflow::body_image_name(id, n))
                .collect(),
        }
    }

    fn unit_pages(id: &str, group: u32, kind: LifeKind, source: usize, pages: u32) -> UnitPages {
        UnitPages {
            id: id.to_string(),
            group,
            kind,
            source,
            pages,
        }
    }

    /// v1 (life/life/comp) then v2 (life/life/comp), so the forced v1→v2 break is
    /// valid and the two sources never share a volume.
    fn two_source_units() -> (Vec<UnitPages>, Vec<PlacedUnit>) {
        let up = vec![
            unit_pages("theseus", 1, LifeKind::Life, 0, 200),
            unit_pages("romulus", 1, LifeKind::Life, 0, 200),
            unit_pages("comp_tr", 1, LifeKind::Comparatio, 0, 40),
            unit_pages("nicias", 2, LifeKind::Life, 1, 200),
            unit_pages("crassus", 2, LifeKind::Life, 1, 200),
            unit_pages("comp_nc", 2, LifeKind::Comparatio, 1, 40),
        ];
        let pl = up.iter().map(|u| placed(&u.id, 3)).collect();
        (up, pl)
    }

    #[test]
    fn facsimile_units_split_into_volumes() {
        let (up, pl) = two_source_units();
        let volumes = plan_volumes(&up, &pl, false).unwrap();
        assert!(volumes.len() >= 2, "v1 and v2 must not share a volume");
        // No volume mixes a v1 unit with a v2 unit.
        for vol in &volumes {
            let sources: Vec<usize> = vol
                .units
                .iter()
                .map(|u| up.iter().find(|x| x.id == u.id).unwrap().source)
                .collect();
            assert!(
                sources.windows(2).all(|w| w[0] == w[1]),
                "volume mixes sources"
            );
        }
        // Tomus numbers are 1-based and sequential.
        for (i, v) in volumes.iter().enumerate() {
            assert_eq!(v.tomus, i + 1);
        }
        // Each source's first volume carries the front-matter charge flag.
        assert!(volumes
            .iter()
            .any(|v| v.source == 0 && v.is_first_of_source));
        assert!(volumes
            .iter()
            .any(|v| v.source == 1 && v.is_first_of_source));
    }

    #[test]
    fn plan_volumes_restricted_is_single_volume() {
        let (up, pl) = two_source_units();
        let volumes = plan_volumes(&up[..1], &pl[..1], true).unwrap();
        assert_eq!(volumes.len(), 1);
        assert_eq!(volumes[0].tomus, 1);
        assert!(volumes[0].is_first_of_source);
        assert_eq!(volumes[0].units.len(), 1);
    }

    #[test]
    fn facsimile_imposition_wrappers_written() {
        let dir = tempfile::tempdir().unwrap();
        let (up, pl) = two_source_units();
        let volumes = plan_volumes(&up, &pl, false).unwrap();
        write_wrappers_from_estimate(dir.path(), &volumes).unwrap();
        for i in 1..=volumes.len() {
            assert!(
                dir.path().join(format!("tomus_{i}.tex")).exists(),
                "wrapper for tomus {i} missing"
            );
        }
    }

    #[test]
    fn start_cleanup_removes_only_generated() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path();
        std::fs::write(out.join("facsimile_theseus_1.png"), b"x").unwrap();
        std::fs::write(out.join("front_qr_s0.png"), b"x").unwrap();
        std::fs::write(out.join("tomus_1.pdf"), b"x").unwrap();
        std::fs::write(out.join("tomus_1_inner.tex"), b"x").unwrap();
        std::fs::write(out.join("notes.txt"), b"keep me").unwrap();
        std::fs::create_dir_all(out.join("render_cache")).unwrap();
        std::fs::write(
            out.join("render_cache").join("v1-p0001-300dpi.png"),
            b"cache",
        )
        .unwrap();

        clean_generated(out).unwrap();

        assert!(!out.join("facsimile_theseus_1.png").exists());
        assert!(!out.join("front_qr_s0.png").exists());
        assert!(!out.join("tomus_1.pdf").exists());
        assert!(!out.join("tomus_1_inner.tex").exists());
        // Unrelated files and the raster cache survive.
        assert!(out.join("notes.txt").exists());
        assert!(out
            .join("render_cache")
            .join("v1-p0001-300dpi.png")
            .exists());
    }

    #[test]
    fn clean_render_cache_removes_the_dir() {
        let dir = tempfile::tempdir().unwrap();
        let cache = dir.path().join("render_cache");
        std::fs::create_dir_all(cache.join("nested")).unwrap();
        std::fs::write(cache.join("nested").join("page.png"), b"cache").unwrap();
        std::fs::write(dir.path().join("notes.txt"), b"keep me").unwrap();

        clean_render_cache(dir.path()).unwrap();
        assert!(!cache.exists());
        assert!(dir.path().join("notes.txt").exists());
        clean_render_cache(dir.path()).unwrap();
    }

    #[test]
    fn start_cleanup_skipped_for_restricted() {
        // A restricted run must not wipe a sibling unit's PNGs. The start-of-run
        // cleanup is guarded by !restricted(), so a --unit run skips it.
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("facsimile_theseus_1.png"), b"x").unwrap();
        let args = FacsimileArgs {
            v1: PathBuf::from("v1.pdf"),
            v2: PathBuf::from("v2.pdf"),
            out: dir.path().to_path_buf(),
            units: PathBuf::from("facsimile_units.json"),
            no_compile: true,
            retag: false,
            offline: false,
            detect_units: false,
            redetect: false,
            pages: None,
            unit: Some("romulus".to_string()),
            review_lines: false,
            clean_cache: false,
            exlibris: None,
        };
        assert!(args.restricted());
        if !args.restricted() {
            clean_generated(dir.path()).unwrap();
        }
        assert!(dir.path().join("facsimile_theseus_1.png").exists());
    }

    #[test]
    fn post_compile_cleanup_removes_pngs() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path();
        std::fs::write(out.join("facsimile_theseus_1.png"), b"x").unwrap();
        std::fs::write(out.join("front_colophon.png"), b"x").unwrap();
        std::fs::write(out.join("tomus_1_inner.pdf"), b"x").unwrap();
        std::fs::write(out.join("tomus_1.pdf"), b"x").unwrap();
        std::fs::create_dir_all(out.join("render_cache")).unwrap();
        std::fs::write(out.join("render_cache").join("keep.png"), b"cache").unwrap();

        remove_composed_pngs(out);

        // The composed / front PNGs are gone; the deliverable PDFs and cache remain.
        assert!(!out.join("facsimile_theseus_1.png").exists());
        assert!(!out.join("front_colophon.png").exists());
        assert!(out.join("tomus_1_inner.pdf").exists());
        assert!(out.join("tomus_1.pdf").exists());
        assert!(out.join("render_cache").join("keep.png").exists());
    }

    #[test]
    fn parse_pages_reads_source_and_range() {
        assert_eq!(parse_pages("v1:14-45").unwrap(), (0, 14, 45));
        assert_eq!(parse_pages("v2:1-5").unwrap(), (1, 1, 5));
        assert!(parse_pages("v3:1-5").is_err());
        assert!(parse_pages("v1:5-1").is_err());
    }

    #[test]
    fn run_facsimile_smoke() {
        // Rendering is a local poppler call (no network). Gate on poppler and the
        // source PDF being present, as the render tests do.
        if !pdftoppm_available() {
            eprintln!("skipping: pdftoppm not available");
            return;
        }
        let v1 = Path::new("Ploutarchou_Bioi_v1.pdf");
        if !v1.exists() {
            eprintln!("skipping: source PDF not present");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let args = FacsimileArgs {
            v1: v1.to_path_buf(),
            v2: Path::new("Ploutarchou_Bioi_v2.pdf").to_path_buf(),
            out: dir.path().to_path_buf(),
            units: PathBuf::from("facsimile_units.json"),
            no_compile: true,
            retag: false,
            offline: false,
            detect_units: false,
            redetect: false,
            pages: Some("v1:14-16".to_string()),
            unit: None,
            review_lines: false,
            clean_cache: false,
            exlibris: None,
        };
        run_facsimile(&args).unwrap();
        // A body image and an imposition wrapper were produced offline.
        assert!(dir.path().join("facsimile_pages_1.png").exists());
        assert!(dir.path().join("tomus_1.tex").exists());
    }
}
