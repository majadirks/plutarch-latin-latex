//! Emits the per-volume inner `.tex` (trim-size, single-column Old Standard Latin
//! with re-typeset front matter) and the US Letter imposition wrapper that lays
//! those pages two-up for 32-page-signature binding. Each volume's imposed sheets
//! are bracketed by a blank front and back endpaper-wrapper sheet that carries the
//! fold line, sew marks, and cut border but no book content, so a binder can wrap
//! an oversized endpaper inside it and guillotine the block to trim size.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::tags::{Block, BlockKind, FrontMatter, LifeBoundary, PageDoc, PageRole, Sidecar};

/// Body font family and size for the inner pages. Exposed so both can be tuned
/// after the first real compile. Old Standard covers Latin and polytonic Greek
/// under lualatex, so one main font renders both scripts.
#[derive(Debug, Clone)]
pub struct Fonts {
    pub main: String,
    pub size_pt: f32,
}

impl Default for Fonts {
    fn default() -> Self {
        Fonts {
            // Loaded by file-name stem (see `inner_preamble`): the Old Standard OTFs
            // are `OldStandard-Regular.otf` and friends, which fontspec resolves via
            // kpathsea without the font being registered under a display name.
            main: "OldStandard".to_string(),
            size_pt: 10.0,
        }
    }
}

/// Sew-mark geometry, in centimetres. A kettle stitch this far in from each end
/// of the fold, `SEW_TAPES` tape supports evenly spaced between them, each tape
/// two holes `2 × SEW_HALFTAPE_CM` apart.
const SEW_MARGIN_CM: &str = "2.5400"; // 72 pt
const SEW_TAPES: u32 = 2;
const SEW_HALFTAPE_CM: &str = "0.7408"; // 21 pt (half of a 42 pt tape width)

/// Book pages per signature (8 sheets × 4 book pages per sheet).
const PAGES_PER_SIGNATURE: u32 = 32;

/// Blank flyleaves at each end of every volume, counted in whole leaves (two PDF
/// pages each), keeping the verso/recto parity the imposition mapping depends on.
pub const FLYLEAVES_PER_END: u32 = 2;

/// Toggles for the inner preamble. The transcription path passes
/// `running_head: false` (reproducing the original output byte-for-byte); the
/// facsimile path passes `true` to reserve head space and add a centered running
/// head driven by `\markboth`.
#[derive(Debug, Clone, Copy)]
pub struct InnerOpts {
    pub running_head: bool,
}

/// The plan for one output volume: its number, which source's front matter it
/// carries (when first of that source), and the Life/Comparatio ids it contains.
#[derive(Debug, Clone)]
pub struct VolumePlan {
    pub tomus: usize,
    pub source: usize,
    pub is_first_of_source: bool,
    pub life_ids: Vec<String>,
}

// ---- Inner pages ---------------------------------------------------------

/// Write one inner `.tex` per volume into `out_dir`, rendering the front-matter
/// scan images each volume references via `render_front_image(source, page)`
/// (which yields the written PNG's file name in `out_dir`).
pub fn write_inner_volumes(
    sidecar: &Sidecar,
    plans: &[VolumePlan],
    out_dir: &Path,
    fonts: &Fonts,
    mut render_front_image: impl FnMut(usize, u32) -> Result<String>,
) -> Result<Vec<PathBuf>> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("creating output dir {}", out_dir.display()))?;
    let mut paths = Vec::with_capacity(plans.len());
    for plan in plans {
        // Render the scan images this volume embeds before emitting the .tex that
        // references them.
        if plan.is_first_of_source {
            for record in &sidecar.pages {
                if record.source == plan.source && record.role == PageRole::FrontImage {
                    render_front_image(record.source, record.source_page)?;
                }
            }
        }
        let doc = inner_document(sidecar, plan, fonts);
        let path = out_dir.join(format!("tomus_{}_inner.tex", plan.tomus));
        fs::write(&path, doc).with_context(|| format!("writing {}", path.display()))?;
        paths.push(path);
    }
    Ok(paths)
}

/// The scan-image file name for a source page embedded in the inner document.
pub fn front_image_name(source: usize, page: u32) -> String {
    format!("front_s{source}_p{page}.png")
}

/// Generate the full inner `.tex` for one volume.
pub fn inner_document(sidecar: &Sidecar, plan: &VolumePlan, fonts: &Fonts) -> String {
    let mut doc = inner_preamble(
        fonts,
        InnerOpts {
            running_head: false,
        },
    );
    doc.push_str("\\begin{document}\n\n");
    doc.push_str("% Front flyleaves (blank endpapers), two PDF pages per leaf\n");
    doc.push_str(&blank_pages(2 * FLYLEAVES_PER_END));
    doc.push('\n');

    if plan.is_first_of_source {
        doc.push_str(&front_matter_block(sidecar, plan.source, plan.tomus));
    }

    doc.push_str(&tomus_label(plan.tomus));
    doc.push_str("\\beginbodynumbering\n\n");

    for id in &plan.life_ids {
        if let Some(life) = sidecar.lives.iter().find(|l| &l.id == id) {
            doc.push_str(&life_block(sidecar, life));
        }
    }

    doc.push_str("\\clearpage\n");
    doc.push_str("% Back flyleaves (blank endpapers), two PDF pages per leaf\n");
    doc.push_str(&blank_pages(2 * FLYLEAVES_PER_END));
    doc.push_str(SIGNATURE_PADDING);
    doc.push_str("\\end{document}\n");
    doc
}

/// `pages` blank, unnumbered PDF pages.
pub(crate) fn blank_pages(pages: u32) -> String {
    "\\thispagestyle{empty}\\null\\clearpage\n".repeat(pages as usize)
}

/// Blank-page padding issued at document end so the inner page count reaches a
/// whole 32-page signature, after the back flyleaves, leaving the final verso
/// blank.
pub(crate) const SIGNATURE_PADDING: &str = "\
% Pad to a whole 32-page signature after the back flyleaves; the final even\n\
% (verso) page is left blank.\n\
\\loop\\unless\\ifnum\\numexpr(\\value{page}-1)-((\\value{page}-1)/32)*32\\relax=0\n\
\x20 \\thispagestyle{empty}\\null\\clearpage\n\
\\repeat\n\n";

/// Geometry options for the transcription inner: no reserved head band.
const GEOMETRY_OPTS_PLAIN: &str = "top=1cm,bottom=1.5cm,inner=1.2cm,outer=0.9cm,footskip=0.8cm";

/// Geometry options for the facsimile inner: wider, binding-safe margins (inner /
/// outer 1 cm), 1.5 cm above the running head and 1.5 cm below the text, and a
/// reserved head band (`headheight` + `headsep`) for the centered Life running
/// head. `top` is the margin *above* the header; the header geometry sits below it.
const GEOMETRY_OPTS_FACSIMILE: &str = "top=1.5cm,bottom=1.5cm,inner=1cm,outer=1cm,footskip=1cm,%\n\
     \x20           includehead,headheight=0.5cm,headsep=0.4cm";

pub(crate) fn inner_preamble(fonts: &Fonts, opts: InnerOpts) -> String {
    let leading = fonts.size_pt * 1.3;
    // The facsimile path reserves a head band and wider margins; the transcription
    // path keeps the original geometry and `folio` style byte-for-byte (golden test).
    let geometry_opts = if opts.running_head {
        GEOMETRY_OPTS_FACSIMILE
    } else {
        GEOMETRY_OPTS_PLAIN
    };
    let fancyhead = if opts.running_head {
        "\x20 \\fancyhead[C]{\\small\\leftmark}%\n"
    } else {
        ""
    };
    format!(
        "% Inner book pages, trim size 10.5 cm x 16 cm. Compile once with lualatex.\n\
         \\documentclass{{book}}\n\n\
         \\usepackage[paperwidth=10.5cm,paperheight=16cm,%\n\
         \x20           {geometry_opts}]{{geometry}}\n\n\
         \\usepackage{{fontspec}}\n\
         \\setmainfont{{{main}}}[\n\
         \x20 Extension=.otf,\n\
         \x20 UprightFont=*-Regular,\n\
         \x20 BoldFont=*-Bold,\n\
         \x20 ItalicFont=*-Italic,\n\
         \x20 BoldItalicFont=*-BoldItalic]\n\
         \\AtBeginDocument{{\\fontsize{{{size}}}{{{leading}}}\\selectfont}}\n\n\
         \\usepackage{{graphicx}}\n\
         \\usepackage{{fancyhdr}}\n\n\
         \\fancypagestyle{{folio}}{{%\n\
         \x20 \\fancyhf{{}}%\n\
         {fancyhead}\
         \x20 \\fancyfoot[LE]{{\\thepage}}%\n\
         \x20 \\fancyfoot[RO]{{\\thepage}}%\n\
         \x20 \\renewcommand{{\\headrulewidth}}{{0pt}}%\n\
         \x20 \\renewcommand{{\\footrulewidth}}{{0pt}}%\n\
         }}\n\
         \\pagestyle{{folio}}\n\
         \\renewcommand{{\\thepage}}{{\\romannumeral\\numexpr\\value{{page}}-{front}\\relax}}\n\n\
         \\newcommand{{\\heading}}[1]{{{{\\centering\\bfseries\\large #1\\par\\bigskip}}}}\n\
         \\newcommand{{\\freshpage}}{{\\clearpage\\ifodd\\value{{page}}\\thispagestyle{{empty}}\\hbox{{}}\\clearpage\\fi}}\n\
         \\newcommand{{\\beginbodynumbering}}{{%\n\
         \x20 \\xdef\\bodyfirstpage{{\\number\\value{{page}}}}%\n\
         \x20 \\gdef\\thepage{{\\the\\numexpr\\value{{page}}-\\bodyfirstpage+1\\relax}}%\n\
         }}\n\n",
        main = fonts.main,
        size = fonts.size_pt,
        leading = leading,
        front = 2 * FLYLEAVES_PER_END,
    )
}

/// The whole re-typeset front matter for a source's first volume: the title page,
/// the embedded front-matter scans, then the re-typeset prose (Monitum / Praefatio).
/// Both build paths compose it here so the bytes stay identical; the scan PNGs are
/// rendered into the output directory separately (by the caller), as
/// [`write_inner_volumes`] already does before emitting the `.tex`.
pub(crate) fn front_matter_block(sidecar: &Sidecar, source: usize, tomus: usize) -> String {
    let mut out = title_page(&sidecar.front_matter, tomus);
    out.push_str(&front_images(sidecar, source));
    out.push_str(&front_prose(sidecar, source));
    out
}

/// The re-typeset title page, generated from the derived bibliographic metadata.
/// Empty fields are omitted so the page stays well-formed when metadata has not
/// been filled in (e.g. a dry-run), rather than emitting a bare `\\`.
fn title_page(front_matter: &FrontMatter, tomus: usize) -> String {
    let roman = to_roman(tomus as u32);
    let mut lines: Vec<String> = Vec::new();
    if !front_matter.title.is_empty() {
        lines.push(format!(
            "{{\\LARGE\\bfseries {}}}",
            escape_latex(&front_matter.title)
        ));
    }
    if !front_matter.editor.is_empty() {
        lines.push(format!(
            "{{\\large {}}}",
            escape_latex(&front_matter.editor)
        ));
    }
    if !front_matter.date.is_empty() {
        lines.push(escape_latex(&front_matter.date));
    }
    lines.push(format!("TOMUS {roman}"));
    let body = lines.join("\\\\[1em]\n\x20 ");
    format!(
        "% Re-typeset title page\n\
         \\freshpage\n\
         \\thispagestyle{{empty}}\n\
         \\null\\vfill\n\
         \\begin{{center}}\n\
         \x20 {body}\n\
         \\end{{center}}\n\
         \\vfill\n\
         \\clearpage\n\n",
    )
}

/// Embedded front-matter scans (Google/QR page, engraved vignette) for a source,
/// each cropped at include time. Trim values default to zero and are tunable after
/// a real-eyes check of the scans.
fn front_images(sidecar: &Sidecar, source: usize) -> String {
    let mut out = String::new();
    for record in &sidecar.pages {
        if record.source == source && record.role == PageRole::FrontImage {
            let name = front_image_name(record.source, record.source_page);
            out.push_str(&format!(
                "% Embedded scan of source {source} page {}\n\
                 \\freshpage\n\
                 \\thispagestyle{{empty}}\n\
                 \\begin{{center}}\n\
                 \x20 \\includegraphics[trim=0 0 0 0,clip,width=\\textwidth,height=\\textheight,keepaspectratio]{{{name}}}\n\
                 \\end{{center}}\n\
                 \\clearpage\n\n",
                record.source_page,
            ));
        }
    }
    out
}

/// The re-typeset front-matter prose (Monitum for v1, Praefatio for v2) for a
/// source, each piece starting recto. Role-driven: emits every `FrontProse` page's
/// result for the source. Used by the transcription path only.
fn front_prose(sidecar: &Sidecar, source: usize) -> String {
    let mut out = String::new();
    for record in &sidecar.pages {
        if record.source == source && record.role == PageRole::FrontProse {
            let Some(doc) = &record.result else {
                continue;
            };
            out.push_str(&front_prose_page(doc, false));
        }
    }
    out
}

/// The re-typeset front-matter prose for exactly the listed `(source, page)`
/// records, in the given order. Page-list-driven (not role-driven), so the facsimile
/// Monitum/Praefatio is composed from a precise page span and never sweeps in the
/// half-title or Volumen-title pages that share the `FrontProse` role. The section
/// flows across source pages and resets the running head once. Missing or empty
/// documents prevent paragraph continuation across the omitted text.
pub(crate) fn front_prose_pages(sidecar: &Sidecar, pages: &[(usize, u32)]) -> String {
    let blocks = collect_front_prose_blocks(sidecar, pages);
    if blocks.is_empty() {
        String::new()
    } else {
        front_prose_page_blocks(&blocks, true, true, true)
    }
}

fn collect_front_prose_blocks(sidecar: &Sidecar, pages: &[(usize, u32)]) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut previous_present = false;
    for &(source, page) in pages {
        let Some(doc) = sidecar
            .page(source, page)
            .and_then(|record| record.result.as_ref())
        else {
            previous_present = false;
            continue;
        };
        if doc.blocks.is_empty() {
            previous_present = false;
            continue;
        }
        if previous_present {
            crate::assemble::append_blocks(&mut blocks, doc);
        } else {
            let mut after_gap = doc.clone();
            after_gap.blocks[0].continues_previous = false;
            crate::assemble::append_blocks(&mut blocks, &after_gap);
        }
        previous_present = true;
    }
    blocks
}

/// One continuous section with faithful prose around dedicated corrigenda pages.
pub(crate) fn front_praefatio(sidecar: &Sidecar, image_names: &[String]) -> Result<String> {
    use crate::facsimile::frontmatter::{self, PRAEFATIO_CLOSING_PAGES, PRAEFATIO_OPENING_PAGES};
    if image_names.is_empty() {
        anyhow::bail!("praefatio: missing corrigenda image assets");
    }
    frontmatter::validate_praefatio_results(sidecar)?;
    for (source, page) in PRAEFATIO_OPENING_PAGES
        .into_iter()
        .chain(PRAEFATIO_CLOSING_PAGES)
    {
        if sidecar
            .page(source, page)
            .and_then(|r| r.result.as_ref())
            .is_none_or(|d| d.blocks.is_empty())
        {
            anyhow::bail!("praefatio source {source}, source page {page}: missing finalized prose; restore the reviewed cache or transcribe the Latin region");
        }
    }
    let mut out = front_prose_pages(sidecar, &PRAEFATIO_OPENING_PAGES);
    for name in image_names {
        out.push_str(&format!("\\noindent\\includegraphics[width=\\textwidth,keepaspectratio]{{{name}}}\\par\n\\clearpage\n"));
    }
    out.push_str(&front_prose_blocks(
        &collect_front_prose_blocks(sidecar, &PRAEFATIO_CLOSING_PAGES),
        true,
    ));
    out.push_str("\\clearpage\n");
    Ok(out)
}

/// Render one front-prose page: a fresh recto, an optional running-head reset, the
/// block stream, and a trailing page break. `reset_running_head` emits
/// `\markboth{}{}` so the facsimile path's prose carries no stale Life running head;
/// the transcription path passes `false`, keeping its golden bytes unchanged.
fn front_prose_page(doc: &PageDoc, reset_running_head: bool) -> String {
    front_prose_page_blocks(&doc.blocks, reset_running_head, false, false)
}

/// Render listed pages through the first block containing `sentinel`, preserving
/// the cached text. Warn and emit all available blocks if the sentinel is absent.
pub(crate) fn front_prose_pages_until(
    sidecar: &Sidecar,
    pages: &[(usize, u32)],
    sentinel: &str,
) -> String {
    let mut out = String::new();
    for &(source, page) in pages {
        let Some(doc) = sidecar
            .page(source, page)
            .and_then(|record| record.result.as_ref())
        else {
            continue;
        };
        if let Some(idx) = doc
            .blocks
            .iter()
            .position(|block| block.text.contains(sentinel))
        {
            out.push_str(&front_prose_page_blocks(
                &doc.blocks[..=idx],
                true,
                false,
                true,
            ));
            return out;
        }
        out.push_str(&front_prose_page_blocks(&doc.blocks, true, false, true));
    }
    eprintln!("WARNING: front-prose sentinel not found: {sentinel:?}; emitting all available prose, which may duplicate embedded errata");
    out
}

/// `recto` selects the opening page break: the facsimile path passes `true` so
/// the prose opens on a recto (`\unitrecto`); the transcription path passes
/// `false`, keeping its golden `\freshpage` bytes unchanged.
fn front_prose_page_blocks(
    blocks: &[Block],
    reset_running_head: bool,
    flexible_spacing: bool,
    recto: bool,
) -> String {
    let page_break = if recto { "\\unitrecto" } else { "\\freshpage" };
    let mut out = format!("% Re-typeset front-matter prose\n{page_break}\n");
    if reset_running_head {
        out.push_str("\\markboth{}{}\n");
    }
    out.push_str(&front_prose_blocks(blocks, flexible_spacing));
    out.push_str("\\clearpage\n\n");
    out
}

fn front_prose_blocks(blocks: &[Block], flexible_spacing: bool) -> String {
    let mut out = String::new();
    if flexible_spacing {
        // Additional stretch lets long Latin and Greek runs fit the narrow measure.
        out.push_str("\\begingroup\n\\emergencystretch=3em\n");
    }
    out.push_str(&render_blocks(blocks));
    if flexible_spacing {
        out.push_str("\\par\\endgroup\n");
    }
    out
}

/// The facsimile Tomus title page includes fixed Didot bibliographic lines,
/// this Tomus's Lives in small caps, and an optional bordered ownership plate.
/// It is the first page with content in every emitting Tomus.
///
/// `life_latin_titles` are the corpus `latin_title`s (already uppercase, e.g.
/// `C. MARIUS`). Each is emitted as `\textsc{\MakeLowercase{…}}`: lower-casing feeds
/// true small caps while the font restores cap height, so an abbreviated praenomen
/// like `C. MARIUS` renders correctly without manual case surgery.
pub(crate) fn tomus_title_page(
    tomus: usize,
    life_latin_titles: &[String],
    exlibris: Option<&str>,
) -> String {
    let plate = exlibris.map(exlibris_plate).unwrap_or_default();
    let roman = to_roman(tomus as u32);
    let title =
        ["{\\LARGE\\bfseries PLUTARCHI}", "{\\LARGE\\bfseries VITÆ}"].join("\\\\[0.55em]\n\x20 ");
    let attribution = [
        "{\\large A GUILIELMO XYLANDRO}",
        "{\\large LATINE REDDITÆ,}",
        "{\\large ATQUE}",
        "{\\large A THEODORO DŒHNERO}",
        "{\\large RECOGNITÆ, EMENDATÆ}",
        "{\\large ET ORDINATÆ.}",
    ]
    .join("\\\\[0.55em]\n\x20 ");
    // A wider gap sets the attribution apart from the VITÆ title above it.
    let display = [title, attribution].join("\\\\[1.4em]\n\x20 ");
    let mut lines = vec![display, format!("{{\\large TOMUS {roman}}}")];
    if !life_latin_titles.is_empty() {
        // Each name is an unbreakable box (no mid-name hyphenation); the bullet
        // separators carry ordinary interword spaces so the list wraps at a bullet,
        // and the whole list sits in a slightly narrowed, centered parbox so a long
        // tomus never runs its Lives off the measure.
        let lives = life_latin_titles
            .iter()
            .map(|t| {
                format!(
                    "\\mbox{{\\textsc{{\\MakeLowercase{{{}}}}}}}",
                    escape_latex(t)
                )
            })
            .collect::<Vec<_>>()
            .join(" \\textbullet\\ ");
        lines.push(format!(
            "\\parbox{{0.86\\textwidth}}{{\\centering {lives}\\par}}"
        ));
    }
    // A generous gap isolates the volume number, setting it apart from the title
    // block above and the list of Lives below.
    let body = lines.join("\\\\[2.6em]\n\x20 ");
    // The title block sits in the upper third; the owner's `EX LIBRIS` plate is
    // pushed to the bottom, separated from the title by the larger flexible gap.
    format!(
        "% Tomus title page\n\
         \\unitrecto\n\
         \\thispagestyle{{empty}}\n\
         \\null\\vfill\n\
         \\begin{{center}}\n\
         \x20 {body}\n\
         \\end{{center}}\n\
         \\vfill\\vfill\n\
         {plate}\
         \\vspace{{1cm}}\n\
         \\clearpage\n\n",
    )
}

fn exlibris_plate(name: &str) -> String {
    let owner = escape_latex(&name.to_uppercase());
    format!(
        "\\begin{{center}}\n{{\\setlength{{\\fboxrule}}{{0.4pt}}\\setlength{{\\fboxsep}}{{8pt}}\n\\fbox{{\\parbox{{0.72\\textwidth}}{{\\centering EX LIBRIS\\\\[0.6em]{{\\large {owner}}}\\par}}}}}}\n\\end{{center}}\n"
    )
}

/// Embed a front-matter scan (QR page, Volumen title, colophon), scaled to fill the
/// page while keeping its aspect ratio. The PNG is already cropped in Rust (see
/// `frontmatter::render_front_scans`), so no `trim` is needed.
pub(crate) fn embed_scan(name: &str) -> String {
    format!(
        "% Embedded front-matter scan\n\
         \\unitrecto\n\
         \\thispagestyle{{empty}}\n\
         \\begin{{center}}\n\
         \x20 \\includegraphics[width=\\textwidth,height=\\textheight,keepaspectratio]{{{name}}}\n\
         \\end{{center}}\n\
         \\clearpage\n\n",
    )
}

/// Two consecutive front-matter crops in one measured, centered page box.
/// Zero-depth images keep the stack depth zero, so resizebox scales its full height.
pub(crate) fn embed_scan_pair(first: &str, second: &str) -> String {
    format!(
        "% Embedded front-matter scan pair\n\
         \\clearpage\n\
         \\thispagestyle{{empty}}\n\
         \\begingroup\n\
         \\setbox0=\\vbox{{%\n\
         \\offinterlineskip\n\
         \\hbox{{\\includegraphics[width=\\textwidth]{{{first}}}}}%\n\
         \\vskip 2mm\n\
         \\hbox{{\\includegraphics[width=\\textwidth]{{{second}}}}}%\n\
         }}%\n\
         \\dimen0=\\dimexpr\\textheight-\\baselineskip\\relax\n\
         \\noindent\\makebox[\\textwidth][c]{{%\n\
         \\ifdim\\dimexpr\\ht0+\\dp0\\relax>\\dimen0%\n\
         \\resizebox{{!}}{{\\dimen0}}{{\\copy0}}%\n\
         \\else%\n\
         \\copy0%\n\
         \\fi%\n\
         }}\\par\n\
         \\endgroup\n\
         \\clearpage\n\n",
    )
}

/// The `TOMUS N` label page.
pub(crate) fn tomus_label(tomus: usize) -> String {
    let roman = to_roman(tomus as u32);
    format!(
        "% Volume label page\n\
         \\freshpage\n\
         \\thispagestyle{{empty}}\n\
         \\null\\vfill\n\
         \\begin{{center}}{{\\Huge\\bfseries TOMUS {roman}}}\\end{{center}}\n\
         \\vfill\\clearpage\n\n",
    )
}

/// One Life/Comparatio: a fresh page, then its assembled text.
fn life_block(sidecar: &Sidecar, life: &LifeBoundary) -> String {
    let blocks = crate::assemble::assembled_text(sidecar, life);
    format!("\\freshpage\n{}\n", render_blocks(&blocks))
}

/// Render a logical block stream to LaTeX body source, keeping inline Roman
/// chapter numbers and parenthetical section numbers, and verse offset.
fn render_blocks(blocks: &[Block]) -> String {
    let mut out = String::new();
    let mut last: Option<BlockKind> = None;
    for block in blocks {
        match block.kind {
            BlockKind::Heading => {
                out.push_str(&format!("\n\\heading{{{}}}\n\n", escape_latex(&block.text)));
            }
            BlockKind::Chapter => {
                // A chapter number opens a new paragraph; its prose, when the
                // transcriber attached it to this block, follows inline.
                let roman = block.roman.as_deref().unwrap_or("");
                out.push_str(&format!("\n\n{{\\bfseries {}}}~", escape_latex(roman)));
                out.push_str(&escape_latex(&block.text));
            }
            BlockKind::Section => {
                // A section number sits inline in the flowing prose; its text, when
                // attached to this block, follows it.
                let n = block.section_no.map(|n| n.to_string()).unwrap_or_default();
                let lead = match last {
                    None | Some(BlockKind::Heading) => "\n\n",
                    _ => " ",
                };
                out.push_str(&format!("{lead}({n})~"));
                out.push_str(&escape_latex(&block.text));
            }
            BlockKind::Paragraph => {
                let lead = match last {
                    Some(BlockKind::Chapter) | Some(BlockKind::Section) => " ",
                    _ => "\n\n",
                };
                out.push_str(&format!("{lead}{}", escape_latex(&block.text)));
            }
            BlockKind::Verse => {
                out.push_str("\n\n\\begin{verse}\n");
                let lines: Vec<&str> = block.text.lines().collect();
                for (i, line) in lines.iter().enumerate() {
                    out.push_str(&escape_latex(line));
                    if i + 1 < lines.len() {
                        out.push_str(" \\\\");
                    }
                    out.push('\n');
                }
                out.push_str("\\end{verse}\n");
            }
        }
        last = Some(block.kind);
    }
    out.push('\n');
    out
}

// ---- Imposition wrapper --------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaperSize {
    Letter,
    A4,
}

struct WrapperGeometry {
    paper_width: &'static str,
    paper_height: &'static str,
    bounding_width: &'static str,
    bounding_height: &'static str,
    lx: &'static str,
    mx: &'static str,
    rx: &'static str,
    by: &'static str,
    ty: &'static str,
}

impl PaperSize {
    fn geometry(self) -> WrapperGeometry {
        match self {
            Self::Letter => WrapperGeometry {
                paper_width: "11in",
                paper_height: "8.5in",
                bounding_width: "27.94",
                bounding_height: "21.59",
                lx: "3.47",
                mx: "13.97",
                rx: "24.47",
                by: "2.795",
                ty: "18.795",
            },
            Self::A4 => WrapperGeometry {
                paper_width: "29.7cm",
                paper_height: "21cm",
                bounding_width: "29.7",
                bounding_height: "21",
                lx: "4.35",
                mx: "14.85",
                rx: "25.35",
                by: "2.5",
                ty: "18.5",
            },
        }
    }
}

/// Write one imposition wrapper `.tex` per volume into `out_dir`.
/// `inner_page_counts[i]` is volume *i*'s inner PDF page count.
pub fn write_imposition_wrappers(
    n_volumes: usize,
    inner_page_counts: &[u32],
    out_dir: &Path,
) -> Result<Vec<PathBuf>> {
    write_imposition_wrappers_for_paper(n_volumes, inner_page_counts, out_dir, PaperSize::Letter)
}

pub fn write_imposition_wrappers_for_paper(
    n_volumes: usize,
    inner_page_counts: &[u32],
    out_dir: &Path,
    paper_size: PaperSize,
) -> Result<Vec<PathBuf>> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("creating output dir {}", out_dir.display()))?;
    let mut paths = Vec::with_capacity(n_volumes);
    for i in 0..n_volumes {
        let tomus = i + 1;
        let inner_count = inner_page_counts.get(i).copied().unwrap_or(0);
        let doc = wrapper_document_for_paper(tomus, inner_count, paper_size);
        let path = out_dir.join(format!("tomus_{tomus}.tex"));
        fs::write(&path, doc).with_context(|| format!("writing {}", path.display()))?;
        paths.push(path);
    }
    Ok(paths)
}

/// One imposed sheet side: which book pages sit left and right, whether this
/// side carries the cut border, and two independent spine flags — `fold` draws
/// the dotted fold line, `sew` draws the sew marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Side {
    left: u32,
    right: u32,
    cut: bool,
    fold: bool,
    sew: bool,
}

/// The ordered sheet sides for `num_signatures` nested 32-page signatures: for
/// sheet *i* (1 = outermost) of signature *s* (base `32s`), the outer/recto side
/// carries `34−2i | 2i−1`, the inner/verso side `2i | 33−2i`. The fold line falls
/// on every outer side; the sew marks fall on every side, both faces; the cut
/// border only on the first recto of each signature.
fn sheet_sides(num_signatures: u32) -> Vec<Side> {
    let mut sides = Vec::with_capacity((num_signatures * 16) as usize);
    for s in 0..num_signatures {
        let base = s * PAGES_PER_SIGNATURE;
        for i in 1..=8u32 {
            sides.push(Side {
                left: base + 34 - 2 * i,
                right: base + 2 * i - 1,
                cut: i == 1,
                fold: true,
                sew: true,
            });
            sides.push(Side {
                left: base + 2 * i,
                right: base + 33 - 2 * i,
                cut: false,
                fold: false,
                sew: true,
            });
        }
    }
    sides
}

/// The two sides of one blank endpaper-wrapper sheet. This single folded sheet
/// carries the fold line, sew marks, and cut border on the face toward the cover
/// board and is blank on the face toward the endpaper; the `sew` flag tracks the
/// marked face. `marks_first` puts the marked side on the sheet's recto (the front
/// wrapper) or its verso (the back wrapper). Both sides reference book page
/// `inner_count + 1`, one past the last real page, so the imposition macro's
/// `\ifnum#1>\innercount` guard draws no inner content on them and the wrappers
/// never affect the book's page numbering.
fn endpaper_wrapper_sides(inner_count: u32, marks_first: bool) -> [Side; 2] {
    let blank_page = inner_count + 1;
    let marked = Side {
        left: blank_page,
        right: blank_page,
        cut: true,
        fold: true,
        sew: true,
    };
    let blank = Side {
        left: blank_page,
        right: blank_page,
        cut: false,
        fold: false,
        sew: false,
    };
    if marks_first {
        [marked, blank]
    } else {
        [blank, marked]
    }
}

/// Generate the full imposition wrapper `.tex` for one volume.
pub fn wrapper_document(tomus: usize, inner_count: u32) -> String {
    wrapper_document_for_paper(tomus, inner_count, PaperSize::Letter)
}

/// Generate the full imposition wrapper `.tex` for one volume on `paper_size`.
/// The emitted sheet sides are, in order: the front endpaper-wrapper sheet, the
/// body signatures from [`sheet_sides`], then the back endpaper-wrapper sheet.
pub fn wrapper_document_for_paper(tomus: usize, inner_count: u32, paper_size: PaperSize) -> String {
    let signatures = inner_count.div_ceil(PAGES_PER_SIGNATURE).max(1);
    let mut sides = Vec::with_capacity((signatures * 16 + 4) as usize);
    sides.extend(endpaper_wrapper_sides(inner_count, true));
    sides.extend(sheet_sides(signatures));
    sides.extend(endpaper_wrapper_sides(inner_count, false));
    let pdf = format!("tomus_{tomus}_inner.pdf");
    let geometry = paper_size.geometry();

    let mut doc = String::new();
    doc.push_str(&format!(
        "% Imposition wrapper: landscape, two book pages per sheet side in\n\
         % 32-page-signature folding order. Compile once with lualatex after the inner\n\
         % PDF exists.\n\
         \\documentclass{{article}}\n\
         \\usepackage[paperwidth={},paperheight={},margin=0pt]{{geometry}}\n\
         \\usepackage{{graphicx}}\n\
         \\usepackage{{tikz}}\n\
         \\pagestyle{{empty}}\n\
         \\setlength{{\\parindent}}{{0pt}}\n",
        geometry.paper_width, geometry.paper_height
    ));
    doc.push_str(&format!("\\newcommand{{\\innercount}}{{{inner_count}}}\n"));
    doc.push_str(&format!(
        "\\def\\Lx{{{}}}\n\\def\\Mx{{{}}}\n\\def\\Rx{{{}}}\n\\def\\By{{{}}}\n\\def\\Ty{{{}}}\n\\def\\sewdot{{0.06}}\n",
        geometry.lx, geometry.mx, geometry.rx, geometry.by, geometry.ty
    ));
    doc.push_str(&format!(
        "\\def\\sewmargin{{{SEW_MARGIN_CM}}}\n\
         \\def\\sewtapes{{{SEW_TAPES}}}\n\
         \\def\\sewhalftape{{{SEW_HALFTAPE_CM}}}\n"
    ));
    doc.push_str(WRAPPER_SEWMACRO);
    doc.push_str(&wrapper_sheet_macro(&pdf, &geometry));
    doc.push_str("\\begin{document}\n\n");
    for side in &sides {
        doc.push_str(&format!(
            "\\imposedsheetside{{{}}}{{{}}}{{{}}}{{{}}}{{{}}}\n",
            side.left,
            side.right,
            u8::from(side.cut),
            u8::from(side.fold),
            u8::from(side.sew),
        ));
    }
    doc.push_str("\n\\end{document}\n");
    doc
}

fn wrapper_sheet_macro(pdf: &str, geometry: &WrapperGeometry) -> String {
    format!(
        "% #1 left book page, #2 right book page, #3 cut flag, #4 fold flag, #5 sew flag\n\
         \\newcommand{{\\imposedsheetside}}[5]{{%\n\
         \x20 \\begin{{tikzpicture}}[x=1cm,y=1cm]\n\
         \x20   \\useasboundingbox (0,0) rectangle ({},{});\n\
         \x20   \\ifnum#1>\\innercount\\else\n\
         \x20     \\node[anchor=south west,inner sep=0] at (\\Lx,\\By)\n\
         \x20       {{\\includegraphics[page=#1]{{{pdf}}}}};\n\
         \x20   \\fi\n\
         \x20   \\ifnum#2>\\innercount\\else\n\
         \x20     \\node[anchor=south west,inner sep=0] at (\\Mx,\\By)\n\
         \x20       {{\\includegraphics[page=#2]{{{pdf}}}}};\n\
         \x20   \\fi\n\
         \x20   \\ifnum#4=1\n\
         \x20     \\draw[dotted,thick] (\\Mx,\\By) -- (\\Mx,\\Ty);\n\
         \x20   \\fi\n\
         \x20   \\ifnum#5=1\n\
         \x20     \\drawsewmarks\n\
         \x20   \\fi\n\
         \x20   \\ifnum#3=1\n\
         \x20     \\draw[dotted,thick] (\\Lx,\\By) rectangle (\\Rx,\\Ty);\n\
         \x20   \\fi\n\
         \x20 \\end{{tikzpicture}}%\n\
         \x20 \\clearpage\n\
         }}\n\n",
        geometry.bounding_width, geometry.bounding_height
    )
}

const WRAPPER_SEWMACRO: &str = "\
\\newcommand{\\drawsewmarks}{%\n\
\x20 \\pgfmathsetmacro{\\yb}{\\By+\\sewmargin}%\n\
\x20 \\pgfmathsetmacro{\\yt}{\\Ty-\\sewmargin}%\n\
\x20 \\pgfmathsetmacro{\\sewspan}{\\yt-\\yb}%\n\
\x20 \\fill (\\Mx,\\yb) circle (\\sewdot);\n\
\x20 \\fill (\\Mx,\\yt) circle (\\sewdot);\n\
\x20 \\foreach \\k in {1,...,\\sewtapes}{%\n\
\x20   \\pgfmathsetmacro{\\cy}{\\yb+\\sewspan*\\k/(\\sewtapes+1)}%\n\
\x20   \\fill (\\Mx,{\\cy-\\sewhalftape}) circle (\\sewdot);\n\
\x20   \\fill (\\Mx,{\\cy+\\sewhalftape}) circle (\\sewdot);\n\
\x20 }%\n\
}\n";

// ---- Helpers -------------------------------------------------------------

/// Escape LaTeX special characters, leaving UTF-8 (accents, the long s,
/// ligatures, Greek) untouched for `lualatex`.
pub(crate) fn escape_latex(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\textbackslash{}"),
            '&' => out.push_str("\\&"),
            '%' => out.push_str("\\%"),
            '#' => out.push_str("\\#"),
            '_' => out.push_str("\\_"),
            '$' => out.push_str("\\$"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '~' => out.push_str("\\textasciitilde{}"),
            '^' => out.push_str("\\textasciicircum{}"),
            _ => out.push(c),
        }
    }
    out
}

fn to_roman(mut n: u32) -> String {
    const TABLE: &[(u32, &str)] = &[
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut out = String::new();
    for &(value, sym) in TABLE {
        while n >= value {
            out.push_str(sym);
            n -= value;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    fn hybrid_fixture() -> (Sidecar, Vec<String>) {
        let sc = Sidecar::load(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(crate::facsimile::frontmatter::SIDECAR),
        )
        .unwrap();
        (
            sc,
            vec![
                "front_praefatio_errata_001.png".into(),
                "front_praefatio_errata_002.png".into(),
            ],
        )
    }

    #[test]
    fn tomus_title_page_omits_visible_exlibris_by_default() {
        let page = tomus_title_page(1, &[], None);
        for absent in ["EX LIBRIS", "\\fbox", "phantom"] {
            assert!(!page.contains(absent));
        }
        assert!(page.contains("PLUTARCHI"));
        assert!(page.contains("VIT\u{c6}"));
        assert!(page.ends_with("\\clearpage\n\n"));
        // The title page opens on a recto, not a verso.
        assert!(page.contains("\\unitrecto"));
        assert!(!page.contains("\\freshpage"));
    }
    #[test]
    fn tomus_title_page_renders_bordered_uppercase_exlibris() {
        let page = tomus_title_page(1, &[], Some("Alex Smith"));
        for expected in [
            "\\fbox{",
            "EX LIBRIS",
            "ALEX SMITH",
            "\\fboxrule}{0.4pt}",
            "\\fboxsep}{8pt}",
            "\\parbox{0.72\\textwidth}",
        ] {
            assert!(page.contains(expected), "{expected}");
        }
        assert!(!page.contains("Alex Smith"));
    }
    #[test]
    fn exlibris_plate_uppercases_unicode_and_escapes_latex() {
        let plate = exlibris_plate("\u{c9}lodie & Andr\u{e9} \\{}%_#$~^");
        assert!(plate.contains("\u{c9}LODIE \\& ANDR\u{c9}"));
        for expected in [
            "\\textbackslash{}",
            "\\{",
            "\\}",
            "\\%",
            "\\_",
            "\\#",
            "\\$",
            "\\textasciitilde{}",
            "\\textasciicircum{}",
        ] {
            assert!(plate.contains(expected), "{expected}");
        }
        assert!(exlibris_plate(" Alex Smith ").contains(" ALEX SMITH "));
    }
    #[test]
    fn tomus_title_page_preserves_requested_legacy_owner() {
        let page = tomus_title_page(1, &[], Some("Martinez-Dirks"));
        assert!(page.contains("\\fbox{"));
        assert!(page.contains("MARTINEZ-DIRKS"));
    }

    #[test]
    fn praefatio_hybrid_emits_prose_images_prose_once() {
        let (sc, names) = hybrid_fixture();
        let out = front_praefatio(&sc, &names).unwrap();
        assert_eq!(
            out.matches(crate::facsimile::frontmatter::PRAEFATIO_PROSE_SENTINEL)
                .count(),
            1
        );
        assert_eq!(out.matches("\\heading{PRÆFATIO.}").count(), 1);
        assert_eq!(out.matches("\\markboth{}{}").count(), 1);
        assert!(!out.contains("consentiente ipso Plutarcho"));
        let a = out.find(&names[0]).unwrap();
        let b = out.find(&names[1]).unwrap();
        let c = out.find("Interpretationem latinam").unwrap();
        assert!(a < b && b < c);
        assert!(names.iter().all(|n| out.matches(n.as_str()).count() == 1));
    }

    #[test]
    fn praefatio_hybrid_has_no_internal_parity_breaks() {
        let (sc, names) = hybrid_fixture();
        let out = front_praefatio(&sc, &names).unwrap();
        // One recto break opens the section; no verso break and no internal parity breaks.
        assert_eq!(out.matches("\\unitrecto").count(), 1);
        assert!(!out.contains("\\freshpage"));
        assert_eq!(out.matches("\\clearpage").count(), 4);
        for token in ["\\hbox{}", "\\begin{center}", "\\vfill"] {
            assert!(!out.contains(token));
        }
    }

    #[test]
    fn praefatio_hybrid_keeps_page_seventeen_paragraphs_separate() {
        let (sc, names) = hybrid_fixture();
        let out = front_praefatio(&sc, &names).unwrap();
        let blocks = &sc.page(1, 17).unwrap().result.as_ref().unwrap().blocks;
        assert_eq!(blocks.len(), 2);
        assert!(out.contains(&format!(
            "{}\n\n{}",
            escape_latex(&blocks[0].text),
            escape_latex(&blocks[1].text)
        )));
    }

    #[test]
    fn praefatio_hybrid_rejects_missing_prose_or_images() {
        let (mut sc, names) = hybrid_fixture();
        assert!(front_praefatio(&sc, &[]).is_err());
        for page in [14, 15, 17] {
            let original = sc.page_mut(1, page).unwrap().result.take();
            let err = front_praefatio(&sc, &names).unwrap_err();
            assert!(err.to_string().contains(&format!("source page {page}")));
            sc.page_mut(1, page).unwrap().result = original;
        }
    }
    use super::*;
    use crate::tags::{LifeKind, PageDoc, PageRecord, PageStatus, SourcePdf};

    fn block(kind: BlockKind, text: &str) -> Block {
        Block {
            kind,
            roman: None,
            section_no: None,
            continues_previous: false,
            text: text.to_string(),
        }
    }

    fn body_page(source: usize, page: u32, blocks: Vec<Block>) -> PageRecord {
        PageRecord {
            source,
            source_page: page,
            image_sha256: String::new(),
            role: PageRole::Body,
            pass_a: None,
            pass_b: None,
            status: PageStatus::Agreed,
            flags: Vec::new(),
            result: Some(PageDoc { blocks }),
            result_model: Some(crate::llm::SONNET.to_string()),
        }
    }

    fn front_prose_page(source: usize, page: u32, blocks: Vec<Block>) -> PageRecord {
        let mut r = body_page(source, page, blocks);
        r.role = PageRole::FrontProse;
        r
    }

    fn front_image_page(source: usize, page: u32) -> PageRecord {
        PageRecord {
            source,
            source_page: page,
            image_sha256: String::new(),
            role: PageRole::FrontImage,
            pass_a: None,
            pass_b: None,
            status: PageStatus::NotBody,
            flags: Vec::new(),
            result: None,
            result_model: None,
        }
    }

    fn sidecar(pages: Vec<PageRecord>, lives: Vec<LifeBoundary>) -> Sidecar {
        Sidecar {
            schema_version: crate::tags::SCHEMA_VERSION,
            sources: vec![
                SourcePdf {
                    path: "v1.pdf".to_string(),
                    page_count: 100,
                    file_sha256: "a".to_string(),
                },
                SourcePdf {
                    path: "v2.pdf".to_string(),
                    page_count: 100,
                    file_sha256: "b".to_string(),
                },
            ],
            front_matter: FrontMatter {
                title: "PLUTARCHI VITAE".to_string(),
                editor: "THEOD. DŒHNER".to_string(),
                date: "M DCCC LVII".to_string(),
            },
            pages,
            lives,
            batches: Vec::new(),
        }
    }

    fn plan(tomus: usize, source: usize, first: bool, ids: &[&str]) -> VolumePlan {
        VolumePlan {
            tomus,
            source,
            is_first_of_source: first,
            life_ids: ids.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn life(id: &str, kind: LifeKind, first: usize, last: usize) -> LifeBoundary {
        LifeBoundary {
            id: id.to_string(),
            title: id.to_uppercase(),
            kind,
            group: 1,
            first_page: first,
            last_page: last,
        }
    }

    #[test]
    fn latex_specials_escaped_text_preserved() {
        let esc = escape_latex("a & b % c # d _ e $ f { } ~ ^ ſubláta uirtus ἀρετή œ");
        assert!(esc.contains("\\&"));
        assert!(esc.contains("\\%"));
        assert!(esc.contains("\\#"));
        assert!(esc.contains("\\_"));
        assert!(esc.contains("\\$"));
        assert!(esc.contains("\\{"));
        assert!(esc.contains("\\}"));
        // Orthography, the long s, and Greek pass through untouched.
        assert!(esc.contains("ſubláta uirtus"));
        assert!(esc.contains("ἀρετή"));
        assert!(esc.contains("œ"));
    }

    #[test]
    fn each_unit_starts_fresh_page_command_present() {
        let pages = vec![
            body_page(
                0,
                14,
                vec![
                    block(BlockKind::Heading, "THESEUS"),
                    block(BlockKind::Paragraph, "a"),
                ],
            ),
            body_page(
                0,
                15,
                vec![
                    block(BlockKind::Heading, "ROMULUS"),
                    block(BlockKind::Paragraph, "b"),
                ],
            ),
        ];
        let lives = vec![
            life("theseus", LifeKind::Life, 0, 0),
            life("romulus", LifeKind::Life, 1, 1),
        ];
        let sc = sidecar(pages, lives);
        let doc = inner_document(
            &sc,
            &plan(1, 0, true, &["theseus", "romulus"]),
            &Fonts::default(),
        );
        // Each Life block opens with a fresh page.
        assert!(doc.contains("\\freshpage\n\n\\heading{THESEUS}"));
        assert!(doc.contains("\\freshpage\n\n\\heading{ROMULUS}"));
        let theseus = doc.find("THESEUS").unwrap();
        assert!(doc[..theseus].contains("\\freshpage"));
    }

    #[test]
    fn inner_preamble_running_head_off_unchanged() {
        // The transcription path's preamble is byte-for-byte the original: the
        // geometry carries no head space and the folio style has no running head.
        let p = inner_preamble(
            &Fonts::default(),
            InnerOpts {
                running_head: false,
            },
        );
        assert!(
            p.contains("top=1cm,bottom=1.5cm,inner=1.2cm,outer=0.9cm,footskip=0.8cm]{geometry}")
        );
        assert!(!p.contains("includehead"));
        assert!(!p.contains("headheight"));
        assert!(!p.contains("headsep"));
        assert!(!p.contains("\\fancyhead"));
        assert!(p.contains(
            "\\fancypagestyle{folio}{%\n  \\fancyhf{}%\n  \\fancyfoot[LE]{\\thepage}%\n  \\fancyfoot[RO]{\\thepage}%\n  \\renewcommand{\\headrulewidth}{0pt}%\n  \\renewcommand{\\footrulewidth}{0pt}%\n}\n"
        ));
    }

    #[test]
    fn inner_preamble_running_head_on_reserves_head_space() {
        let p = inner_preamble(&Fonts::default(), InnerOpts { running_head: true });
        // Wider binding-safe margins, 1.5 cm above the header and below the text,
        // and a reserved head band for the centered running head.
        assert!(p.contains("top=1.5cm,bottom=1.5cm,inner=1cm,outer=1cm,footskip=1cm"));
        assert!(p.contains("includehead,headheight=0.5cm,headsep=0.4cm]{geometry}"));
        assert!(p.contains("\\fancyhead[C]{\\small\\leftmark}"));
    }

    #[test]
    fn head_geometry_matches_reflow_constants() {
        // The preamble's margin / head strings must match the centimetre constants
        // the reflow geometry derives the body text block from.
        use crate::facsimile::reflow;
        assert_eq!(format!("{}", reflow::HEADHEIGHT_CM), "0.5");
        assert_eq!(format!("{}", reflow::HEADSEP_CM), "0.4");
        assert_eq!(format!("{}", reflow::INNER_CM), "1");
        assert_eq!(format!("{}", reflow::OUTER_CM), "1");
        assert_eq!(format!("{}", reflow::TOP_CM), "1.5");
        assert_eq!(format!("{}", reflow::BOTTOM_CM), "1.5");
    }

    #[test]
    fn frontmatter_roman_body_arabic_numbering() {
        let sc = sidecar(vec![], vec![]);
        let doc = inner_document(&sc, &plan(1, 0, true, &[]), &Fonts::default());
        let expected = format!(
            "\\renewcommand{{\\thepage}}{{\\romannumeral\\numexpr\\value{{page}}-{}\\relax}}",
            2 * FLYLEAVES_PER_END
        );
        assert!(doc.contains(&expected));
        assert!(doc.contains("\\beginbodynumbering"));
        assert!(doc.contains("\\numexpr\\value{page}-\\bodyfirstpage+1"));
    }

    #[test]
    fn monitum_renders_latin_and_greek() {
        let pages = vec![front_prose_page(
            0,
            12,
            vec![
                block(BlockKind::Heading, "MONITUM"),
                block(BlockKind::Paragraph, "Latina prosa cum ἀρετή fragmento."),
            ],
        )];
        let sc = sidecar(pages, vec![]);
        let doc = inner_document(&sc, &plan(1, 0, true, &[]), &Fonts::default());
        assert!(doc.contains("MONITUM"));
        assert!(doc.contains("Latina prosa"));
        assert!(doc.contains("ἀρετή"));
    }

    #[test]
    fn praefatio_renders_on_v2_first_output_tomus() {
        let pages = vec![front_prose_page(
            1,
            5,
            vec![
                block(BlockKind::Heading, "PRAEFATIO"),
                block(BlockKind::Paragraph, "v2 prose"),
            ],
        )];
        let sc = sidecar(pages, vec![]);
        let v2 = inner_document(&sc, &plan(3, 1, true, &[]), &Fonts::default());
        assert!(v2.contains("PRAEFATIO"));
        // A non-first-of-source volume does not re-emit the front matter.
        let later = inner_document(&sc, &plan(4, 1, false, &[]), &Fonts::default());
        assert!(!later.contains("PRAEFATIO"));
    }

    #[test]
    fn front_matter_block_unchanged_for_transcription() {
        // The extracted composer emits exactly the title page + scans + prose that
        // `inner_document` embeds for a source's first volume.
        let pages = vec![
            front_image_page(0, 2),
            front_prose_page(
                0,
                12,
                vec![
                    block(BlockKind::Heading, "MONITUM"),
                    block(BlockKind::Paragraph, "Latina prosa cum ἀρετή fragmento."),
                ],
            ),
        ];
        let sc = sidecar(pages, vec![]);
        let block = front_matter_block(&sc, 0, 1);
        let doc = inner_document(&sc, &plan(1, 0, true, &[]), &Fonts::default());
        assert!(
            doc.contains(&block),
            "inner_document must embed front_matter_block verbatim"
        );
        // It is the concatenation of the three pieces, in order.
        let expected = {
            let mut s = title_page(&sc.front_matter, 1);
            s.push_str(&front_images(&sc, 0));
            s.push_str(&front_prose(&sc, 0));
            s
        };
        assert_eq!(block, expected);
    }

    #[test]
    fn verse_blocks_render_offset_preserving_line_breaks() {
        let mut verse = block(BlockKind::Verse, "line one\nline two\nline three");
        verse.continues_previous = false;
        let pages = vec![body_page(
            0,
            14,
            vec![block(BlockKind::Heading, "NICIAS"), verse],
        )];
        let lives = vec![life("nicias", LifeKind::Life, 0, 0)];
        let sc = sidecar(pages, lives);
        let doc = inner_document(&sc, &plan(1, 0, true, &["nicias"]), &Fonts::default());
        assert!(doc.contains("\\begin{verse}"));
        assert!(doc.contains("\\end{verse}"));
        assert!(doc.contains("line one \\\\"));
        assert!(doc.contains("line two \\\\"));
        assert!(doc.contains("line three"));
    }

    #[test]
    fn qr_page_embedded_as_image() {
        let pages = vec![front_image_page(0, 1)];
        let sc = sidecar(pages, vec![]);
        let doc = inner_document(&sc, &plan(1, 0, true, &[]), &Fonts::default());
        assert!(doc.contains("\\includegraphics"));
        assert!(doc.contains(&front_image_name(0, 1)));
    }

    #[test]
    fn tomus_label_page_present() {
        let sc = sidecar(vec![], vec![]);
        let doc = inner_document(&sc, &plan(2, 0, false, &[]), &Fonts::default());
        assert!(doc.contains("TOMUS II"));
    }

    #[test]
    fn tomus_title_page_renders_fixed_lines_and_small_caps() {
        let page = tomus_title_page(1, &["THESEUS".to_string(), "C. MARIUS".to_string()], None);
        let mut previous = 0;
        for line in [
            "PLUTARCHI",
            "VITÆ",
            "A GUILIELMO XYLANDRO",
            "LATINE REDDITÆ,",
            "ATQUE",
            "A THEODORO DŒHNERO",
            "RECOGNITÆ, EMENDATÆ",
            "ET ORDINATÆ.",
        ] {
            assert_eq!(page.matches(line).count(), 1, "{line}");
            let position = page.find(line).unwrap();
            assert!(position >= previous, "{line} is out of order");
            previous = position + line.len();
        }
        assert_eq!(page.matches("\\\\[0.55em]").count(), 6);
        assert_eq!(page.matches("\\\\[1.4em]").count(), 1);
        assert!(!page.contains("SECUNDUM CODICES PARISINOS RECOGNOVIT"));
        assert!(page.contains("TOMUS I"));
        // Small caps via lower-casing; abbreviated praenomen kept verbatim.
        assert!(page.contains("\\textsc{\\MakeLowercase{THESEUS}}"));
        assert!(page.contains("\\textsc{\\MakeLowercase{C. MARIUS}}"));
        // Names are unbreakable boxes joined by bullets, inside a wrapping parbox.
        assert!(page.contains("\\mbox{\\textsc{\\MakeLowercase{THESEUS}}}"));
        assert!(page.contains("\\textbullet"));
        assert!(page.contains("\\parbox{0.86\\textwidth}"));
        assert_eq!(page.matches("TOMUS ").count(), 1);
    }

    #[test]
    fn tomus_title_page_preserves_tomus_lives_and_exlibris() {
        let page = tomus_title_page(
            2,
            &["THESEUS".to_string(), "C. MARIUS".to_string()],
            Some("Alex Smith"),
        );
        assert_eq!(page.matches("TOMUS II").count(), 1);
        assert_eq!(page.matches("EX LIBRIS").count(), 1);
        assert_eq!(page.matches("ALEX SMITH").count(), 1);
        assert_eq!(page.matches("\\textbullet").count(), 1);
        assert!(page.contains("\\textsc{\\MakeLowercase{THESEUS}}"));
        assert!(page.contains("\\textsc{\\MakeLowercase{C. MARIUS}}"));
    }

    #[test]
    fn embed_scan_pair_preserves_order_and_single_page() {
        let s = embed_scan_pair("first.png", "second.png");
        assert_eq!(s.matches("first.png").count(), 1);
        assert_eq!(s.matches("second.png").count(), 1);
        assert!(s.find("first.png").unwrap() < s.find("second.png").unwrap());
        assert_eq!(s.matches("\\clearpage").count(), 2);
        assert!(s.contains("\\thispagestyle{empty}"));
        assert!(!s.contains("\\freshpage"));
        assert!(!s.contains("{center}"));
    }

    #[test]
    fn embed_scan_pair_measures_and_shrinks_the_shared_box() {
        let s = embed_scan_pair("first.png", "second.png");
        for expected in [
            "\\setbox0=\\vbox{%",
            "\\offinterlineskip",
            "\\hbox{\\includegraphics[width=\\textwidth]{first.png}}%",
            "\\hbox{\\includegraphics[width=\\textwidth]{second.png}}%",
            "\\vskip 2mm",
            "\\dimen0=\\dimexpr\\textheight-\\baselineskip\\relax",
            "\\ifdim\\dimexpr\\ht0+\\dp0\\relax>\\dimen0%",
            "\\resizebox{!}{\\dimen0}{\\copy0}%",
            "\\else%\n\\copy0%",
            "\\noindent\\makebox[\\textwidth][c]{%",
            "}\\par\n\\endgroup",
        ] {
            assert!(s.contains(expected), "missing {expected}: {s}");
        }
    }

    #[test]
    fn embed_scan_includes_named_png_scaled() {
        let s = embed_scan("front_qr_s0.png");
        assert!(s.contains(
            "\\includegraphics[width=\\textwidth,height=\\textheight,keepaspectratio]{front_qr_s0.png}"
        ));
        assert!(s.contains("\\thispagestyle{empty}"));
        // The PNG is cropped in Rust, so no LaTeX trim is emitted.
        assert!(!s.contains("trim="));
        // The embedded scan opens on a recto, not a verso.
        assert!(s.contains("\\unitrecto"));
        assert!(!s.contains("\\freshpage"));
    }

    #[test]
    fn front_prose_pages_until_stops_at_sentinel() {
        let sc = sidecar(
            vec![
                front_prose_page(
                    0,
                    12,
                    vec![
                        block(BlockKind::Paragraph, "Earlier significavit remains."),
                        block(
                            BlockKind::Paragraph,
                            "Dœhnerus hæc mutanda esse significavit.",
                        ),
                        block(BlockKind::Paragraph, "ERRATA FIRST"),
                        block(BlockKind::Paragraph, "ERRATA SECOND"),
                    ],
                ),
                front_prose_page(0, 13, vec![block(BlockKind::Paragraph, "LATER PAGE")]),
            ],
            vec![],
        );
        let out = front_prose_pages_until(&sc, &[(0, 12), (0, 13)], "mutanda esse significavit.");
        assert!(out.contains("Earlier significavit remains."));
        assert!(out.contains("Dœhnerus hæc mutanda esse significavit."));
        assert!(!out.contains("ERRATA"));
        assert!(!out.contains("LATER PAGE"));
        assert_eq!(out.matches("\\unitrecto").count(), 1);
        assert!(!out.contains("\\freshpage"));
        assert!(out.contains("\\markboth{}{}"));
        assert!(out.ends_with("\\clearpage\n\n"));
    }

    #[test]
    fn front_prose_pages_until_emits_all_when_sentinel_absent() {
        let sc = sidecar(
            vec![
                front_prose_page(
                    0,
                    12,
                    vec![
                        block(BlockKind::Paragraph, "FIRST BLOCK"),
                        block(BlockKind::Paragraph, "SECOND BLOCK"),
                    ],
                ),
                front_prose_page(0, 13, vec![block(BlockKind::Paragraph, "LATER PAGE")]),
            ],
            vec![],
        );
        let pages = [(0, 11), (0, 12), (0, 13)];
        let out = front_prose_pages_until(&sc, &pages, "absent");
        assert!(out.contains("FIRST BLOCK\n\nSECOND BLOCK"));
        assert!(out.contains("LATER PAGE"));
        assert_eq!(out.matches("\\unitrecto").count(), 2);
        assert!(!out.contains("\\freshpage"));
        assert_eq!(out.matches("\\markboth{}{}").count(), 2);
        assert_eq!(out.matches("\\clearpage").count(), 2);
    }

    #[test]
    fn front_prose_pages_flows_as_one_section() {
        let sc = sidecar(
            vec![
                front_prose_page(
                    1,
                    14,
                    vec![
                        block(BlockKind::Heading, "PRÆFATIO."),
                        block(BlockKind::Paragraph, "First"),
                    ],
                ),
                front_prose_page(1, 15, vec![block(BlockKind::Paragraph, "Second")]),
            ],
            vec![],
        );
        let out = front_prose_pages(&sc, &[(1, 14), (1, 15)]);
        assert_eq!(out.matches("\\unitrecto").count(), 1);
        assert!(!out.contains("\\freshpage"));
        assert_eq!(out.matches("\\clearpage").count(), 1);
        assert_eq!(out.matches("\\markboth{}{}").count(), 1);
        assert_eq!(out.matches("\\heading{PRÆFATIO.}").count(), 1);
        assert!(out.contains("\\begingroup\n\\emergencystretch=3em\n"));
        assert!(out.contains("\\par\\endgroup\n\\clearpage"));
        assert!(out.ends_with("\\clearpage\n\n"));
    }

    #[test]
    fn front_prose_pages_rejoins_continued_paragraphs() {
        let mut continuation = block(BlockKind::Paragraph, "ordinem recepi æ; ipso");
        continuation.continues_previous = true;
        let mut tail = block(BlockKind::Paragraph, "Plutarcho καθίσταντο.");
        tail.continues_previous = true;
        let sc = sidecar(
            vec![
                front_prose_page(1, 14, vec![block(BlockKind::Paragraph, "confirmatas in")]),
                front_prose_page(1, 15, vec![continuation]),
                front_prose_page(1, 16, vec![tail]),
            ],
            vec![],
        );
        let before = sc.clone();
        let out = front_prose_pages(&sc, &[(1, 14), (1, 15), (1, 16)]);
        assert!(out.contains("confirmatas in ordinem recepi æ; ipso Plutarcho καθίσταντο."));
        assert_eq!(sc, before);
    }

    #[test]
    fn front_prose_pages_preserves_new_paragraph_and_dateline() {
        let sc = sidecar(
            vec![
                front_prose_page(1, 16, vec![block(BlockKind::Paragraph, "καθίσταντο.")]),
                front_prose_page(
                    1,
                    17,
                    vec![
                        block(BlockKind::Paragraph, "Interpretationem"),
                        block(BlockKind::Paragraph, "Scribebam Schneebergæ."),
                    ],
                ),
            ],
            vec![],
        );
        assert!(front_prose_pages(&sc, &[(1, 16), (1, 17)])
            .contains("καθίσταντο.\n\nInterpretationem\n\nScribebam Schneebergæ."));
    }

    #[test]
    fn front_prose_pages_empty_emits_nothing() {
        let sc = sidecar(vec![front_prose_page(1, 14, vec![])], vec![]);
        assert!(front_prose_pages(&sc, &[]).is_empty());
        assert!(front_prose_pages(&sc, &[(1, 14), (1, 15)]).is_empty());
    }

    #[test]
    fn front_prose_pages_does_not_join_across_missing_page() {
        for gap in 0..3 {
            let mut tail = block(BlockKind::Paragraph, "after gap");
            tail.continues_previous = true;
            let mut records = vec![front_prose_page(
                1,
                14,
                vec![block(BlockKind::Paragraph, "before gap")],
            )];
            if gap != 0 {
                let mut record = front_prose_page(1, 15, vec![]);
                if gap == 1 {
                    record.result = None;
                }
                records.push(record);
            }
            records.push(front_prose_page(1, 16, vec![tail]));
            let sc = sidecar(records, vec![]);
            assert!(front_prose_pages(&sc, &[(1, 14), (1, 15), (1, 16)])
                .contains("before gap\n\nafter gap"));
        }
    }

    #[test]
    fn front_prose_pages_emits_only_listed() {
        let pages = vec![
            front_prose_page(0, 8, vec![block(BlockKind::Heading, "HALF TITLE P8")]),
            front_prose_page(0, 10, vec![block(BlockKind::Heading, "VOLUMEN P10")]),
            front_prose_page(
                0,
                12,
                vec![
                    block(BlockKind::Heading, "MONITUM"),
                    block(BlockKind::Paragraph, "monitum body"),
                ],
            ),
            front_prose_page(0, 13, vec![block(BlockKind::Paragraph, "monitum page two")]),
        ];
        let sc = sidecar(pages, vec![]);
        let out = front_prose_pages(&sc, &[(0, 12), (0, 13)]);
        assert!(out.contains("monitum body"));
        assert!(out.contains("monitum page two"));
        // Neither the half-title (p8) nor the Volumen title (p10) is emitted.
        assert!(!out.contains("HALF TITLE P8"));
        assert!(!out.contains("VOLUMEN P10"));
        // The section resets the running head once.
        assert_eq!(out.matches("\\markboth{}{}").count(), 1);
    }

    #[test]
    fn chapter_and_section_numbers_inline_verbatim() {
        let mut chapter = block(BlockKind::Chapter, "");
        chapter.roman = Some("XI.".to_string());
        let mut section = block(BlockKind::Section, "");
        section.section_no = Some(2);
        let pages = vec![body_page(
            0,
            14,
            vec![
                block(BlockKind::Heading, "ROMULUS"),
                chapter,
                section,
                block(BlockKind::Paragraph, "Romulus"),
            ],
        )];
        let lives = vec![life("romulus", LifeKind::Life, 0, 0)];
        let sc = sidecar(pages, lives);
        let doc = inner_document(&sc, &plan(1, 0, true, &["romulus"]), &Fonts::default());
        assert!(doc.contains("{\\bfseries XI.}"));
        assert!(doc.contains("(2)~"));
    }

    #[test]
    fn chapter_and_section_block_text_is_rendered() {
        // Transcribers attach a chapter's/section's prose to the Chapter/Section
        // block's `text`; that prose must appear in the output, not just the number.
        let mut chapter = block(BlockKind::Chapter, "principio igitur regnauit");
        chapter.roman = Some("XXV".to_string());
        let mut section = block(BlockKind::Section, "deinde profectus est");
        section.section_no = Some(5);
        let out = render_blocks(&[chapter, section]);
        assert!(out.contains("{\\bfseries XXV}"));
        assert!(out.contains("principio igitur regnauit"));
        assert!(out.contains("(5)~"));
        assert!(out.contains("deinde profectus est"));
    }

    // ---- Imposition (carried from prior art) ----

    #[test]
    fn imposition_orders_pages_for_signature() {
        let sides = sheet_sides(1);
        assert_eq!(sides.len(), 16);
        assert_eq!(
            sides[0],
            Side {
                left: 32,
                right: 1,
                cut: true,
                fold: true,
                sew: true
            }
        );
        assert_eq!(
            sides[1],
            Side {
                left: 2,
                right: 31,
                cut: false,
                fold: false,
                sew: true
            }
        );
        assert_eq!(
            sides[2],
            Side {
                left: 30,
                right: 3,
                cut: false,
                fold: true,
                sew: true
            }
        );
        assert_eq!(
            sides[3],
            Side {
                left: 4,
                right: 29,
                cut: false,
                fold: false,
                sew: true
            }
        );
        let two = sheet_sides(2);
        assert_eq!(
            two[16],
            Side {
                left: 64,
                right: 33,
                cut: true,
                fold: true,
                sew: true
            }
        );
    }

    #[test]
    fn imposition_places_odd_book_pages_on_right() {
        for signatures in [1, 2, 4, 11] {
            let sides = sheet_sides(signatures);
            let mut pages = Vec::new();
            for side in sides {
                assert_eq!(side.right % 2, 1);
                assert_eq!(side.left % 2, 0);
                pages.extend([side.left, side.right]);
            }
            pages.sort_unstable();
            assert_eq!(pages, (1..=32 * signatures).collect::<Vec<_>>());
        }
    }

    #[test]
    fn fold_line_outer_only_sew_on_every_side() {
        for (i, side) in sheet_sides(2).iter().enumerate() {
            assert_eq!(side.fold, i % 2 == 0);
            assert!(side.sew, "side {i} should carry sew marks");
        }
        let doc = wrapper_document(1, 32);
        assert!(doc.contains("\\drawsewmarks"));
        assert!(doc.contains("\\draw[dotted,thick] (\\Mx,\\By) -- (\\Mx,\\Ty);"));
    }

    #[test]
    fn sew_marks_emitted_on_inner_sheet_sides() {
        let doc = wrapper_document(1, 32);
        // An inner body side: no cut, no fold line, but the sew flag is set.
        assert!(doc.contains("\\imposedsheetside{2}{31}{0}{0}{1}"));
        // No four-group body call survives the arity bump.
        assert!(!doc.contains("\\imposedsheetside{2}{31}{0}{0}\n"));
    }

    #[test]
    fn cut_border_only_on_first_recto_of_each_signature() {
        let sides = sheet_sides(2);
        for (i, side) in sides.iter().enumerate() {
            assert_eq!(side.cut, i == 0 || i == 16);
        }
    }

    #[test]
    fn endpaper_wrapper_sides_carry_no_book_pages() {
        // The sentinel is `inner_count + 1` = 33 for a one-signature volume.
        assert_eq!(
            endpaper_wrapper_sides(32, true),
            [
                Side {
                    left: 33,
                    right: 33,
                    cut: true,
                    fold: true,
                    sew: true
                },
                Side {
                    left: 33,
                    right: 33,
                    cut: false,
                    fold: false,
                    sew: false
                },
            ]
        );
        assert_eq!(
            endpaper_wrapper_sides(32, false),
            [
                Side {
                    left: 33,
                    right: 33,
                    cut: false,
                    fold: false,
                    sew: false
                },
                Side {
                    left: 33,
                    right: 33,
                    cut: true,
                    fold: true,
                    sew: true
                },
            ]
        );
    }

    #[test]
    fn endpaper_wrapper_pages_exceed_inner_count_so_they_draw_blank() {
        for inner in [0u32, 5, 32, 33, 128] {
            for side in endpaper_wrapper_sides(inner, true)
                .iter()
                .chain(endpaper_wrapper_sides(inner, false).iter())
            {
                assert!(side.left > inner, "left {} !> {inner}", side.left);
                assert!(side.right > inner, "right {} !> {inner}", side.right);
            }
        }
    }

    #[test]
    fn endpaper_wrappers_bracket_the_body() {
        let doc = wrapper_document(1, 32);
        // 16 body sides for one 32-page signature + 4 wrapper sides.
        assert_eq!(doc.matches("\\imposedsheetside{").count(), 20);
    }

    #[test]
    fn endpaper_wrappers_emit_marked_blank_sides() {
        let doc = wrapper_document(1, 32);
        assert!(doc.contains("\\imposedsheetside{33}{33}{1}{1}{1}"));
        assert!(doc.contains("\\imposedsheetside{33}{33}{0}{0}{0}"));
    }

    #[test]
    fn endpaper_wrappers_do_not_renumber_inner_pages() {
        let doc = wrapper_document(1, 32);
        assert!(doc.contains("\\imposedsheetside{32}{1}{1}{1}{1}"));
        assert!(doc.contains("\\imposedsheetside{2}{31}{0}{0}{1}"));
    }

    #[test]
    fn front_wrapper_is_first_sheet_back_wrapper_is_last() {
        let doc = wrapper_document(1, 32);
        let calls: Vec<&str> = doc
            .lines()
            .filter(|l| l.starts_with("\\imposedsheetside{"))
            .collect();
        assert_eq!(calls[0], "\\imposedsheetside{33}{33}{1}{1}{1}");
        assert_eq!(calls[1], "\\imposedsheetside{33}{33}{0}{0}{0}");
        let n = calls.len();
        assert_eq!(calls[n - 2], "\\imposedsheetside{33}{33}{0}{0}{0}");
        assert_eq!(calls[n - 1], "\\imposedsheetside{33}{33}{1}{1}{1}");
    }

    #[test]
    fn pages_past_inner_count_blank() {
        let doc = wrapper_document(1, 5);
        assert!(doc.contains("\\newcommand{\\innercount}{5}"));
        assert!(doc.contains("\\ifnum#1>\\innercount\\else"));
        assert!(doc.contains("\\imposedsheetside{32}{1}{1}{1}{1}"));
    }

    #[test]
    fn wrapper_references_volume_inner_pdf() {
        let doc = wrapper_document(3, 32);
        assert!(doc.contains("tomus_3_inner.pdf"));
    }

    #[test]
    fn wrapper_letter_retains_existing_sheet_geometry() {
        let doc = wrapper_document_for_paper(1, 32, PaperSize::Letter);
        for expected in [
            "paperwidth=11in,paperheight=8.5in",
            "\\useasboundingbox (0,0) rectangle (27.94,21.59)",
            "\\def\\Lx{3.47}",
            "\\def\\Mx{13.97}",
            "\\def\\Rx{24.47}",
            "\\def\\By{2.795}",
            "\\def\\Ty{18.795}",
        ] {
            assert!(doc.contains(expected), "missing {expected}");
        }
    }

    #[test]
    fn wrapper_a4_centers_unchanged_trim_rectangles() {
        let doc = wrapper_document_for_paper(1, 32, PaperSize::A4);
        for expected in [
            "paperwidth=29.7cm,paperheight=21cm",
            "\\useasboundingbox (0,0) rectangle (29.7,21)",
            "\\def\\Lx{4.35}",
            "\\def\\Mx{14.85}",
            "\\def\\Rx{25.35}",
            "\\def\\By{2.5}",
            "\\def\\Ty{18.5}",
        ] {
            assert!(doc.contains(expected), "missing {expected}");
        }
        let g = PaperSize::A4.geometry();
        let cm = |value: &str| (value.parse::<f64>().unwrap() * 1000.0).round() as i32;
        assert_eq!(cm(g.mx) - cm(g.lx), 10_500);
        assert_eq!(cm(g.rx) - cm(g.mx), 10_500);
        assert_eq!(cm(g.ty) - cm(g.by), 16_000);
    }

    #[test]
    fn wrapper_paper_size_keeps_signature_order_and_marks() {
        let letter = wrapper_document_for_paper(2, 33, PaperSize::Letter);
        let a4 = wrapper_document_for_paper(2, 33, PaperSize::A4);
        for expected in [
            "\\imposedsheetside{32}{1}{1}{1}{1}",
            "\\imposedsheetside{2}{31}{0}{0}{1}",
            "\\imposedsheetside{64}{33}{1}{1}{1}",
            "\\drawsewmarks",
            "\\draw[dotted,thick] (\\Mx,\\By) -- (\\Mx,\\Ty)",
            "\\draw[dotted,thick] (\\Lx,\\By) rectangle (\\Rx,\\Ty)",
            "\\includegraphics[page=#1]",
        ] {
            assert!(letter.contains(expected), "Letter missing {expected}");
            assert!(a4.contains(expected), "A4 missing {expected}");
        }
    }

    #[test]
    fn legacy_wrapper_api_remains_letter() {
        assert_eq!(
            wrapper_document(1, 32),
            wrapper_document_for_paper(1, 32, PaperSize::Letter)
        );
        let dir = tempfile::tempdir().unwrap();
        let paths = write_imposition_wrappers(1, &[32], dir.path()).unwrap();
        assert_eq!(
            std::fs::read_to_string(&paths[0]).unwrap(),
            wrapper_document_for_paper(1, 32, PaperSize::Letter)
        );
    }

    #[test]
    fn write_files_create_inner_and_wrappers() {
        let dir = tempfile::tempdir().unwrap();
        let sc = sidecar(vec![], vec![]);
        let plans = vec![plan(1, 0, true, &[]), plan(2, 0, false, &[])];
        let inner = write_inner_volumes(&sc, &plans, dir.path(), &Fonts::default(), |_, _| {
            Ok("x.png".to_string())
        })
        .unwrap();
        assert_eq!(inner.len(), 2);
        assert!(dir.path().join("tomus_1_inner.tex").exists());
        let wrappers = write_imposition_wrappers(2, &[32, 64], dir.path()).unwrap();
        assert_eq!(wrappers.len(), 2);
        assert!(dir.path().join("tomus_1.tex").exists());
    }

    #[test]
    fn roman_numerals() {
        assert_eq!(to_roman(1), "I");
        assert_eq!(to_roman(4), "IV");
        assert_eq!(to_roman(42), "XLII");
    }

    /// End-to-end: the generated inner `.tex` compiles under lualatex at the trim
    /// size, with Latin and Greek in Old Standard. Ignored by default because it
    /// shells out to lualatex; run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn generated_inner_compiles() {
        let mut chapter = block(BlockKind::Chapter, "");
        chapter.roman = Some("I.".to_string());
        let mut section = block(BlockKind::Section, "");
        section.section_no = Some(1);
        let verse = block(
            BlockKind::Verse,
            "Roma caput mundi \\\\ ignored\nsecunda linea",
        );
        let pages = vec![
            front_prose_page(
                0,
                12,
                vec![
                    block(BlockKind::Heading, "MONITUM"),
                    block(
                        BlockKind::Paragraph,
                        "Latina prosa cum ἀρετὴ καὶ δόξα fragmento.",
                    ),
                ],
            ),
            body_page(
                0,
                14,
                vec![
                    block(BlockKind::Heading, "THESEUS"),
                    chapter,
                    section,
                    block(
                        BlockKind::Paragraph,
                        "ſubláta uirtus & pax; iidem Iuppiter.",
                    ),
                    verse,
                ],
            ),
        ];
        let lives = vec![life("theseus", LifeKind::Life, 1, 1)];
        let sc = sidecar(pages, lives);
        let dir = tempfile::tempdir().unwrap();
        let doc = inner_document(&sc, &plan(1, 0, true, &["theseus"]), &Fonts::default());
        let tex = dir.path().join("tomus_1_inner.tex");
        std::fs::write(&tex, doc).unwrap();
        let status = std::process::Command::new("lualatex")
            .current_dir(dir.path())
            .arg("-interaction=nonstopmode")
            .arg("-halt-on-error")
            .arg("tomus_1_inner.tex")
            .status()
            .expect("running lualatex");
        assert!(status.success(), "lualatex failed on generated inner");
        assert!(dir.path().join("tomus_1_inner.pdf").exists());
    }

    /// End-to-end two-stage compile: inner, then the imposition wrapper two-up on
    /// US Letter. Ignored by default (shells out to lualatex).
    #[test]
    #[ignore]
    fn generated_wrapper_imposes_inner() {
        let pages = vec![body_page(
            0,
            14,
            vec![
                block(BlockKind::Heading, "THESEUS"),
                block(BlockKind::Paragraph, "breuis uita."),
            ],
        )];
        let sc = sidecar(pages, vec![life("theseus", LifeKind::Life, 0, 0)]);
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("tomus_1_inner.tex"),
            inner_document(&sc, &plan(1, 0, true, &["theseus"]), &Fonts::default()),
        )
        .unwrap();
        crate::compile::compile(dir.path(), "tomus_1_inner.tex", 1).unwrap();
        let count = crate::compile::inner_page_count(dir.path(), "tomus_1_inner").unwrap();
        assert_eq!(
            count % 32,
            0,
            "inner self-pads to a whole signature: {count}"
        );

        write_imposition_wrappers(1, &[count], dir.path()).unwrap();
        crate::compile::compile(dir.path(), "tomus_1.tex", 1).unwrap();
        let wrapper_pdf = dir.path().join("tomus_1.pdf");
        assert!(wrapper_pdf.exists());
    }
}
