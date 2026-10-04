//! Emits the per-volume inner `.tex` (trim-size, single-column Old Standard Latin
//! with re-typeset front matter) and the US Letter imposition wrapper that lays
//! those pages two-up for 32-page-signature binding.

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
/// half-title or Volumen-title pages that share the `FrontProse` role. Each page
/// resets the running head with `\markboth{}{}` so the prose carries no stale Life
/// running head under the facsimile preamble.
pub(crate) fn front_prose_pages(sidecar: &Sidecar, pages: &[(usize, u32)]) -> String {
    let mut out = String::new();
    for &(source, page) in pages {
        let Some(record) = sidecar.page(source, page) else {
            continue;
        };
        let Some(doc) = &record.result else {
            continue;
        };
        out.push_str(&front_prose_page(doc, true));
    }
    out
}

/// Render one front-prose page: a fresh recto, an optional running-head reset, the
/// block stream, and a trailing page break. `reset_running_head` emits
/// `\markboth{}{}` so the facsimile path's prose carries no stale Life running head;
/// the transcription path passes `false`, keeping its golden bytes unchanged.
fn front_prose_page(doc: &PageDoc, reset_running_head: bool) -> String {
    let mut out = String::from("% Re-typeset front-matter prose\n\\freshpage\n");
    if reset_running_head {
        out.push_str("\\markboth{}{}\n");
    }
    out.push_str(&render_blocks(&doc.blocks));
    out.push_str("\\clearpage\n\n");
    out
}

/// The facsimile Tomus title page: the fixed Didot bibliographic lines, the
/// `TOMUS <roman>` line, this tomus's Lives (small caps, bullet-separated), and the
/// owner's `EX LIBRIS / MARTINEZ-DIRKS` plate. Replaces both the derived
/// `title_page` and the `tomus_label` on the facsimile path; it is the first page
/// with content in every emitting tomus.
///
/// `life_latin_titles` are the corpus `latin_title`s (already uppercase, e.g.
/// `C. MARIUS`). Each is emitted as `\textsc{\MakeLowercase{…}}`: lower-casing feeds
/// true small caps while the font restores cap height, so an abbreviated praenomen
/// like `C. MARIUS` renders correctly without manual case surgery.
pub(crate) fn tomus_title_page(tomus: usize, life_latin_titles: &[String]) -> String {
    let roman = to_roman(tomus as u32);
    let mut lines: Vec<String> = vec![
        "{\\LARGE\\bfseries PLUTARCHI VITÆ}".to_string(),
        "SECUNDUM CODICES PARISINOS RECOGNOVIT".to_string(),
        "{\\large THEODOR DŒHNER}".to_string(),
        format!("{{\\large TOMUS {roman}}}"),
    ];
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
    let body = lines.join("\\\\[1em]\n\x20 ");
    // The title block sits in the upper third; the owner's `EX LIBRIS` plate is
    // pushed to the bottom, separated from the title by the larger flexible gap.
    format!(
        "% Tomus title page\n\
         \\freshpage\n\
         \\thispagestyle{{empty}}\n\
         \\null\\vfill\n\
         \\begin{{center}}\n\
         \x20 {body}\n\
         \\end{{center}}\n\
         \\vfill\\vfill\n\
         \\begin{{center}}\n\
         \x20 EX LIBRIS\\\\[0.6em]\n\
         \x20 {{\\large MARTINEZ-DIRKS}}\n\
         \\end{{center}}\n\
         \\vspace{{1cm}}\n\
         \\clearpage\n\n",
    )
}

/// Embed a front-matter scan (QR page, Volumen title, colophon), scaled to fill the
/// page while keeping its aspect ratio. The PNG is already cropped in Rust (see
/// `frontmatter::render_front_scans`), so no `trim` is needed.
pub(crate) fn embed_scan(name: &str) -> String {
    format!(
        "% Embedded front-matter scan\n\
         \\freshpage\n\
         \\thispagestyle{{empty}}\n\
         \\begin{{center}}\n\
         \x20 \\includegraphics[width=\\textwidth,height=\\textheight,keepaspectratio]{{{name}}}\n\
         \\end{{center}}\n\
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

/// Write one imposition wrapper `.tex` per volume into `out_dir`.
/// `inner_page_counts[i]` is volume *i*'s inner PDF page count.
pub fn write_imposition_wrappers(
    n_volumes: usize,
    inner_page_counts: &[u32],
    out_dir: &Path,
) -> Result<Vec<PathBuf>> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("creating output dir {}", out_dir.display()))?;
    let mut paths = Vec::with_capacity(n_volumes);
    for i in 0..n_volumes {
        let tomus = i + 1;
        let inner_count = inner_page_counts.get(i).copied().unwrap_or(0);
        let doc = wrapper_document(tomus, inner_count);
        let path = out_dir.join(format!("tomus_{tomus}.tex"));
        fs::write(&path, doc).with_context(|| format!("writing {}", path.display()))?;
        paths.push(path);
    }
    Ok(paths)
}

/// One imposed sheet side: which book pages sit left and right, and whether this
/// side carries the cut border and the fold line + sew marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Side {
    left: u32,
    right: u32,
    cut: bool,
    fold: bool,
}

/// The ordered sheet sides for `num_signatures` nested 32-page signatures: for
/// sheet *i* (1 = outermost) of signature *s* (base `32s`), the outer/recto side
/// carries `34−2i | 2i−1`, the inner/verso side `2i | 33−2i`. The fold line and
/// sew marks fall on every outer side; the cut border only on the first recto of
/// each signature.
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
            });
            sides.push(Side {
                left: base + 2 * i,
                right: base + 33 - 2 * i,
                cut: false,
                fold: false,
            });
        }
    }
    sides
}

/// Generate the full imposition wrapper `.tex` for one volume.
pub fn wrapper_document(tomus: usize, inner_count: u32) -> String {
    let signatures = inner_count.div_ceil(PAGES_PER_SIGNATURE).max(1);
    let sides = sheet_sides(signatures);
    let pdf = format!("tomus_{tomus}_inner.pdf");

    let mut doc = String::new();
    doc.push_str(WRAPPER_HEAD);
    doc.push_str(&format!("\\newcommand{{\\innercount}}{{{inner_count}}}\n"));
    doc.push_str(WRAPPER_GEOMETRY);
    doc.push_str(&format!(
        "\\def\\sewmargin{{{SEW_MARGIN_CM}}}\n\
         \\def\\sewtapes{{{SEW_TAPES}}}\n\
         \\def\\sewhalftape{{{SEW_HALFTAPE_CM}}}\n"
    ));
    doc.push_str(WRAPPER_SEWMACRO);
    doc.push_str(&wrapper_sheet_macro(&pdf));
    doc.push_str("\\begin{document}\n\n");
    for side in &sides {
        doc.push_str(&format!(
            "\\imposedsheetside{{{}}}{{{}}}{{{}}}{{{}}}\n",
            side.left,
            side.right,
            u8::from(side.cut),
            u8::from(side.fold),
        ));
    }
    doc.push_str("\n\\end{document}\n");
    doc
}

fn wrapper_sheet_macro(pdf: &str) -> String {
    format!(
        "% #1 left book page, #2 right book page, #3 cut flag, #4 fold flag\n\
         \\newcommand{{\\imposedsheetside}}[4]{{%\n\
         \x20 \\begin{{tikzpicture}}[x=1cm,y=1cm]\n\
         \x20   \\useasboundingbox (0,0) rectangle (27.94,21.59);\n\
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
         \x20     \\drawsewmarks\n\
         \x20   \\fi\n\
         \x20   \\ifnum#3=1\n\
         \x20     \\draw[dotted,thick] (\\Lx,\\By) rectangle (\\Rx,\\Ty);\n\
         \x20   \\fi\n\
         \x20 \\end{{tikzpicture}}%\n\
         \x20 \\clearpage\n\
         }}\n\n",
    )
}

const WRAPPER_HEAD: &str = "\
% Imposition wrapper: US Letter, landscape, two book pages per sheet side in\n\
% 32-page-signature folding order. Compile once with lualatex after the inner\n\
% PDF exists.\n\
\\documentclass{article}\n\
\\usepackage[paperwidth=11in,paperheight=8.5in,margin=0pt]{geometry}\n\
\\usepackage{graphicx}\n\
\\usepackage{tikz}\n\
\\pagestyle{empty}\n\
\\setlength{\\parindent}{0pt}\n";

const WRAPPER_GEOMETRY: &str = "\
\\def\\Lx{3.47}\n\
\\def\\Mx{13.97}\n\
\\def\\Rx{24.47}\n\
\\def\\By{2.795}\n\
\\def\\Ty{18.795}\n\
\\def\\sewdot{0.06}\n";

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
        let page = tomus_title_page(1, &["THESEUS".to_string(), "C. MARIUS".to_string()]);
        assert!(page.contains("{\\LARGE\\bfseries PLUTARCHI VITÆ}"));
        assert!(page.contains("SECUNDUM CODICES PARISINOS RECOGNOVIT"));
        assert!(page.contains("THEODOR DŒHNER"));
        assert!(page.contains("TOMUS I"));
        // Small caps via lower-casing; abbreviated praenomen kept verbatim.
        assert!(page.contains("\\textsc{\\MakeLowercase{THESEUS}}"));
        assert!(page.contains("\\textsc{\\MakeLowercase{C. MARIUS}}"));
        // Names are unbreakable boxes joined by bullets, inside a wrapping parbox.
        assert!(page.contains("\\mbox{\\textsc{\\MakeLowercase{THESEUS}}}"));
        assert!(page.contains("\\textbullet"));
        assert!(page.contains("\\parbox{0.86\\textwidth}"));
        assert!(page.contains("EX LIBRIS"));
        assert!(page.contains("MARTINEZ-DIRKS"));
        assert_eq!(page.matches("TOMUS ").count(), 1);
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
        // Each emitted prose page resets the running head.
        assert_eq!(out.matches("\\markboth{}{}").count(), 2);
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
                fold: true
            }
        );
        assert_eq!(
            sides[1],
            Side {
                left: 2,
                right: 31,
                cut: false,
                fold: false
            }
        );
        assert_eq!(
            sides[2],
            Side {
                left: 30,
                right: 3,
                cut: false,
                fold: true
            }
        );
        assert_eq!(
            sides[3],
            Side {
                left: 4,
                right: 29,
                cut: false,
                fold: false
            }
        );
        let two = sheet_sides(2);
        assert_eq!(
            two[16],
            Side {
                left: 64,
                right: 33,
                cut: true,
                fold: true
            }
        );
    }

    #[test]
    fn fold_and_sew_only_on_outer_sides() {
        for (i, side) in sheet_sides(2).iter().enumerate() {
            assert_eq!(side.fold, i % 2 == 0);
        }
        let doc = wrapper_document(1, 32);
        assert!(doc.contains("\\drawsewmarks"));
        assert!(doc.contains("\\draw[dotted,thick] (\\Mx,\\By) -- (\\Mx,\\Ty);"));
    }

    #[test]
    fn cut_border_only_on_first_recto_of_each_signature() {
        let sides = sheet_sides(2);
        for (i, side) in sides.iter().enumerate() {
            assert_eq!(side.cut, i == 0 || i == 16);
        }
    }

    #[test]
    fn pages_past_inner_count_blank() {
        let doc = wrapper_document(1, 5);
        assert!(doc.contains("\\newcommand{\\innercount}{5}"));
        assert!(doc.contains("\\ifnum#1>\\innercount\\else"));
        assert!(doc.contains("\\imposedsheetside{32}{1}{1}{1}"));
    }

    #[test]
    fn wrapper_references_volume_inner_pdf() {
        let doc = wrapper_document(3, 32);
        assert!(doc.contains("tomus_3_inner.pdf"));
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
