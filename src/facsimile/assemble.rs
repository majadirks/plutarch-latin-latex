//! Emits the facsimile inner `.tex`: a typeset running head and display title per
//! unit, one composed body image per trim page, and the reused flyleaf and
//! signature-padding structure the imposition depends on.
//!
//! The page frame (geometry, `folio` style, `\freshpage`, `\beginbodynumbering`)
//! comes from [`crate::latex::inner_preamble`] with the running head on; the
//! flyleaves and end padding reuse the same pieces [`crate::latex::inner_document`]
//! uses, so a facsimile inner honors the structural contract the imposition assumes.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::facsimile::frontmatter;
use crate::latex::{self, Fonts, InnerOpts};
use crate::tags::LifeKind;

const RECTO_START_DEFINITION: &str = "\\newcommand{\\unitrecto}{\\clearpage\\ifodd\\value{page}\\else\\thispagestyle{empty}\\hbox{}\\clearpage\\fi}\n";

/// One unit placed in a facsimile volume: its Latin title (for the inline chapter
/// header and running head), its kind (so the Tomus title page lists Lives only),
/// and the composed body-image file names, in reading order.
#[derive(Debug, Clone)]
pub struct PlacedUnit {
    pub id: String,
    pub latin_title: String,
    pub kind: LifeKind,
    pub image_names: Vec<String>,
}

/// One facsimile output volume.
#[derive(Debug, Clone)]
pub struct FacsimileVolume {
    pub tomus: usize,
    pub source: usize,
    pub is_first_of_source: bool,
    pub units: Vec<PlacedUnit>,
    /// Re-typeset Monitum (Tomus I) or Praefatio (v2's first tomus) LaTeX, prepared
    /// upstream in `fill_front_matter` (which holds the sidecar). Empty on every
    /// other emitting volume, and on restricted runs.
    pub prose: String,
    /// Whether this volume emits the new front matter (Tomus title page, embedded
    /// scans, prose). True for a normal build; false for a restricted single-volume
    /// `--unit`/`--pages` run, whose scans are never rendered.
    pub emit_front_matter: bool,
}

/// Write one facsimile inner `.tex` into `out_dir`, returning its path.
pub fn write_inner(vol: &FacsimileVolume, out_dir: &Path, fonts: &Fonts) -> Result<PathBuf> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("creating output dir {}", out_dir.display()))?;
    let doc = inner_document(vol, fonts);
    let path = out_dir.join(format!("tomus_{}_inner.tex", vol.tomus));
    fs::write(&path, doc).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

/// Generate the full facsimile inner `.tex` for one volume.
pub fn inner_document(vol: &FacsimileVolume, fonts: &Fonts) -> String {
    let mut doc = latex::inner_preamble(fonts, InnerOpts { running_head: true });
    doc.push_str(RECTO_START_DEFINITION);
    doc.push_str("\\begin{document}\n\n");
    doc.push_str("% Front flyleaves (blank endpapers), two PDF pages per leaf\n");
    doc.push_str(&latex::blank_pages(2 * latex::FLYLEAVES_PER_END));
    doc.push('\n');

    if vol.emit_front_matter {
        // Tomus title page (every emitting tomus): lists this tomus's Lives only.
        let lives: Vec<String> = vol
            .units
            .iter()
            .filter(|u| u.kind == LifeKind::Life)
            .map(|u| u.latin_title.clone())
            .collect();
        doc.push_str(&latex::tomus_title_page(vol.tomus, &lives));
        // The reproduction-notice (QR) scan is on every tomus.
        doc.push_str(&latex::embed_scan(&frontmatter::front_qr_name(vol.source)));
        // The engraved Volumen title opens each volumen (first tomus of a source).
        if vol.is_first_of_source {
            doc.push_str(&latex::embed_scan(&frontmatter::front_volumen_name(
                vol.source,
            )));
        }
        // The printer's colophon follows the Volumen title on Tomus I only.
        if vol.tomus == 1 {
            doc.push_str(&latex::embed_scan(frontmatter::front_colophon_name()));
        }
        // The re-typeset Monitum / Praefatio prose (empty on other tomi).
        doc.push_str(&vol.prose);
        if vol.tomus == 1 {
            doc.push_str(&latex::embed_scan_pair(
                frontmatter::front_monitum_a_name(),
                frontmatter::front_monitum_b_name(),
            ));
        }
    }

    if vol.units.iter().any(|unit| !unit.image_names.is_empty()) {
        doc.push_str("\\unitrecto\n");
    }
    doc.push_str("\\beginbodynumbering\n\n");

    for unit in &vol.units {
        doc.push_str(&unit_block(unit));
    }

    doc.push_str("\\clearpage\n");
    doc.push_str("% Back flyleaves (blank endpapers), two PDF pages per leaf\n");
    doc.push_str(&latex::blank_pages(2 * latex::FLYLEAVES_PER_END));
    doc.push_str(latex::SIGNATURE_PADDING);
    doc.push_str("\\end{document}\n");
    doc
}

/// One unit: a recto start with the running head set to the Latin name, then one body
/// image per page. There is no standalone title page; the unit's first body page
/// opens with a large centered inline chapter header (the Latin title only) and
/// carries no running head, while later pages keep the centered Latin running head.
fn unit_block(unit: &PlacedUnit) -> String {
    if unit.image_names.is_empty() {
        return String::new();
    }
    let latin = latex::escape_latex(&unit.latin_title);
    let mut out = String::new();
    out.push_str("\\unitrecto\n");
    // The running head on every body page of this unit is its Latin name.
    out.push_str(&format!("\\markboth{{{latin}}}{{{latin}}}\n"));
    let mut images = unit.image_names.iter();
    if let Some(first) = images.next() {
        // First body page: no running head, inline Latin header above the image.
        out.push_str("\\thispagestyle{empty}\n");
        out.push_str(&format!("\\heading{{{latin}}}\n"));
        out.push_str(&format!(
            "\\noindent\\includegraphics[width=\\textwidth]{{{first}}}\\par\\clearpage\n"
        ));
    }
    // Later pages keep the Latin running head from \markboth.
    for name in images {
        out.push_str(&format!(
            "\\noindent\\includegraphics[width=\\textwidth]{{{name}}}\\par\\clearpage\n"
        ));
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facsimile_units_use_recto_command() {
        let vol = volume(vec![
            unit("theseus", &["life.png"]),
            unit_kind("comparison", LifeKind::Comparatio, &["comparison.png"]),
        ]);
        let doc = inner_document(&vol, &Fonts::default());
        assert_eq!(doc.matches(RECTO_START_DEFINITION).count(), 1);
        assert_eq!(doc.matches("\\unitrecto\n\\markboth").count(), 2);
    }

    #[test]
    fn first_body_alignment_precedes_numbering() {
        for tomus in [1, 2] {
            for front in [false, true] {
                let vol = volume_full(
                    tomus,
                    0,
                    tomus == 1,
                    front,
                    "",
                    vec![unit("theseus", &["body.png"])],
                );
                let doc = inner_document(&vol, &Fonts::default());
                assert!(
                    doc.contains("\\unitrecto\n\\beginbodynumbering\n\n\\unitrecto\n\\markboth")
                );
            }
        }
    }

    #[test]
    fn empty_unit_emits_no_heading_or_page_break() {
        assert_eq!(unit_block(&unit("empty", &[])), "");
        let mut vol = volume(vec![unit("empty", &[]), unit("theseus", &["body.png"])]);
        vol.emit_front_matter = false;
        let doc = inner_document(&vol, &Fonts::default());
        assert!(!doc.contains("\\heading{EMPTY}"));
        assert_eq!(doc.matches("\\unitrecto\n\\markboth").count(), 1);
        let empty = volume(vec![unit("empty", &[])]);
        assert!(!inner_document(&empty, &Fonts::default()).contains("\\unitrecto\n"));
    }

    fn unit(id: &str, images: &[&str]) -> PlacedUnit {
        unit_kind(id, LifeKind::Life, images)
    }

    fn unit_kind(id: &str, kind: LifeKind, images: &[&str]) -> PlacedUnit {
        PlacedUnit {
            id: id.to_string(),
            latin_title: id.to_uppercase(),
            kind,
            image_names: images.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn volume(units: Vec<PlacedUnit>) -> FacsimileVolume {
        FacsimileVolume {
            tomus: 1,
            source: 0,
            is_first_of_source: true,
            units,
            prose: String::new(),
            emit_front_matter: true,
        }
    }

    #[test]
    fn inner_tex_sets_running_head_per_unit() {
        let vol = volume(vec![
            unit("theseus", &["facsimile_theseus_1.png"]),
            unit("romulus", &["facsimile_romulus_1.png"]),
        ]);
        let doc = inner_document(&vol, &Fonts::default());
        assert!(doc.contains("\\markboth{THESEUS}{THESEUS}"));
        assert!(doc.contains("\\markboth{ROMULUS}{ROMULUS}"));
    }

    #[test]
    fn inner_tex_includes_one_image_per_page() {
        let mut vol = volume(vec![unit(
            "theseus",
            &[
                "facsimile_theseus_1.png",
                "facsimile_theseus_2.png",
                "facsimile_theseus_3.png",
            ],
        )]);
        vol.emit_front_matter = false;
        let doc = inner_document(&vol, &Fonts::default());
        let images = doc.matches("\\includegraphics[width=\\textwidth]").count();
        assert_eq!(images, 3);
        // Each image is followed by a page break.
        let breaks = doc.matches("\\par\\clearpage").count();
        assert_eq!(breaks, 3);
        assert!(doc.contains("{facsimile_theseus_2.png}"));
    }

    #[test]
    fn inner_tex_starts_each_unit_on_recto() {
        let mut vol = volume(vec![
            unit("theseus", &["facsimile_theseus_1.png"]),
            unit("romulus", &["facsimile_romulus_1.png"]),
        ]);
        vol.emit_front_matter = false;
        let doc = inner_document(&vol, &Fonts::default());
        // Count body openings independently of the command definition.
        assert_eq!(doc.matches("\\unitrecto\n\\markboth").count(), 2);
        // The inline chapter header carries the Latin title only.
        assert!(doc.contains("\\heading{THESEUS}"));
        assert!(doc.contains("\\heading{ROMULUS}"));
        // Theseus opens before Romulus.
        assert!(doc.find("THESEUS").unwrap() < doc.find("ROMULUS").unwrap());
    }

    #[test]
    fn facsimile_inner_emits_flyleaves_and_signature_padding() {
        let vol = volume(vec![unit("theseus", &["facsimile_theseus_1.png"])]);
        let doc = inner_document(&vol, &Fonts::default());
        // Front and back flyleaves: 2 * FLYLEAVES_PER_END blank pages at each end.
        let blanks = doc
            .matches("\\thispagestyle{empty}\\null\\clearpage")
            .count();
        assert!(
            blanks >= (4 * latex::FLYLEAVES_PER_END) as usize,
            "expected front+back flyleaves, found {blanks}"
        );
        // The same end-of-document signature padding as the transcription inner.
        assert!(doc.contains("Pad to a whole 32-page signature"));
        assert!(doc.contains("\\beginbodynumbering"));
        assert!(doc.trim_end().ends_with("\\end{document}"));
    }

    #[test]
    fn running_head_preamble_used() {
        let vol = volume(vec![unit("theseus", &["facsimile_theseus_1.png"])]);
        let doc = inner_document(&vol, &Fonts::default());
        assert!(doc.contains("includehead,headheight=0.5cm,headsep=0.4cm"));
        assert!(doc.contains("\\fancyhead[C]{\\small\\leftmark}"));
    }

    /// A volume with explicit placement flags and prepared prose, for front-matter
    /// ordering and guard tests.
    #[allow(clippy::too_many_arguments)]
    fn volume_full(
        tomus: usize,
        source: usize,
        is_first_of_source: bool,
        emit_front_matter: bool,
        prose: &str,
        units: Vec<PlacedUnit>,
    ) -> FacsimileVolume {
        FacsimileVolume {
            tomus,
            source,
            is_first_of_source,
            units,
            prose: prose.to_string(),
            emit_front_matter,
        }
    }

    #[test]
    fn tomus_title_page_lists_only_lives() {
        let vol = volume_full(
            1,
            0,
            true,
            true,
            "",
            vec![
                unit_kind("theseus", LifeKind::Life, &["facsimile_theseus_1.png"]),
                unit_kind("romulus", LifeKind::Life, &["facsimile_romulus_1.png"]),
                unit_kind(
                    "comp_tr",
                    LifeKind::Comparatio,
                    &["facsimile_comp_tr_1.png"],
                ),
            ],
        );
        let doc = inner_document(&vol, &Fonts::default());
        // The title page lists the Lives (small caps) and the owner's plate.
        assert!(doc.contains("\\MakeLowercase{THESEUS}"));
        assert!(doc.contains("\\MakeLowercase{ROMULUS}"));
        assert!(doc.contains("MARTINEZ-DIRKS"));
        // The Comparatio is not listed on the title page.
        assert!(!doc.contains("\\MakeLowercase{COMP\\_TR}"));
        assert!(!doc.contains("\\MakeLowercase{COMP_TR}"));
    }

    #[test]
    fn tomus_title_page_has_single_tomus_line() {
        let vol = volume_full(
            2,
            0,
            false,
            true,
            "",
            vec![unit("theseus", &["facsimile_theseus_1.png"])],
        );
        let doc = inner_document(&vol, &Fonts::default());
        // Exactly one TOMUS page: the old standalone tomus_label is gone.
        assert_eq!(doc.matches("TOMUS ").count(), 1);
        assert!(doc.contains("TOMUS II"));
        assert!(!doc.contains("\\Huge\\bfseries TOMUS"));
    }

    #[test]
    fn tomus_one_embeds_monitum_errata_images() {
        let first = volume_full(
            1,
            0,
            true,
            true,
            "% PROSE MARKER\n",
            vec![unit("a", &["x.png"])],
        );
        let doc = inner_document(&first, &Fonts::default());
        assert_eq!(doc.matches("% Embedded front-matter scan pair").count(), 1);
        let pair_start = doc.find("% Embedded front-matter scan pair").unwrap();
        let pair_end = doc[pair_start..].find("\\beginbodynumbering").unwrap() + pair_start;
        assert_eq!(doc[pair_start..pair_end].matches("\\clearpage").count(), 2);
        let prose = doc.find("PROSE MARKER").unwrap();
        let a = doc.find("front_monitum_a.png").unwrap();
        let b = doc.find("front_monitum_b.png").unwrap();
        let body = doc.find("{x.png}").unwrap();
        assert!(prose < a && a < b && b < body);
        for name in ["front_monitum_a.png", "front_monitum_b.png"] {
            assert_eq!(doc.matches(name).count(), 1);
        }
        for vol in [
            volume_full(2, 0, false, true, "", vec![]),
            volume_full(3, 1, true, true, "% PRAEFATIO\n", vec![]),
            volume_full(1, 0, true, false, "", vec![]),
        ] {
            let doc = inner_document(&vol, &Fonts::default());
            assert!(!doc.contains("front_monitum_a.png"));
            assert!(!doc.contains("front_monitum_b.png"));
        }
    }

    #[test]
    fn monitum_errata_follow_prose_before_recto_body() {
        let vol = volume_full(
            1,
            0,
            true,
            true,
            "% PROSE MARKER\n",
            vec![
                unit("theseus", &["first.png"]),
                unit("romulus", &["second.png"]),
            ],
        );
        let doc = inner_document(&vol, &Fonts::default());
        let prose = doc.find("% PROSE MARKER").unwrap();
        let first = doc.find("{first.png}").unwrap();
        assert!(!doc[prose..first].contains("\\freshpage"));
        assert!(doc[prose..first].contains("\\unitrecto\n\\markboth{THESEUS}"));
        assert!(doc[first..].contains("\\unitrecto\n\\markboth{ROMULUS}"));
    }

    #[test]
    fn first_body_uses_recto_start_without_monitum() {
        for vol in [
            volume_full(2, 0, false, true, "", vec![unit("theseus", &["x.png"])]),
            volume_full(1, 0, true, false, "", vec![unit("theseus", &["x.png"])]),
        ] {
            let doc = inner_document(&vol, &Fonts::default());
            assert!(doc.contains("\\unitrecto\n\\markboth{THESEUS}"));
        }
    }

    #[test]
    fn monitum_without_cached_prose_still_aligns_body_on_recto() {
        let vol = volume_full(1, 0, true, true, "", vec![unit("theseus", &["x.png"])]);
        let doc = inner_document(&vol, &Fonts::default());
        let pair = doc.find("% Embedded front-matter scan pair").unwrap();
        let body = doc.find("{x.png}").unwrap();
        assert!(!doc[pair..body].contains("\\freshpage"));
        assert!(doc[pair..body].contains("\\unitrecto\n\\markboth{THESEUS}"));
    }

    #[test]
    fn facsimile_frontmatter_order() {
        let vol = volume_full(
            1,
            0,
            true,
            true,
            "% MONITUM PROSE MARKER\n",
            vec![unit("theseus", &["facsimile_theseus_1.png"])],
        );
        let doc = inner_document(&vol, &Fonts::default());
        let title = doc.find("PLUTARCHI VITÆ").unwrap();
        let qr = doc.find("front_qr_s0.png").unwrap();
        let volumen = doc.find("front_volumen_s0.png").unwrap();
        let colophon = doc.find("front_colophon.png").unwrap();
        let prose = doc.find("MONITUM PROSE MARKER").unwrap();
        assert!(
            title < qr && qr < volumen && volumen < colophon && colophon < prose,
            "front-matter order wrong: title={title} qr={qr} volumen={volumen} colophon={colophon} prose={prose}"
        );
    }

    #[test]
    fn qr_scan_on_every_tomus() {
        // A non-first-of-source emitting volume still embeds its source's QR scan,
        // but not the Volumen title, colophon, or prose.
        let vol = volume_full(
            3,
            1,
            false,
            true,
            "",
            vec![unit("nicias", &["facsimile_nicias_1.png"])],
        );
        let doc = inner_document(&vol, &Fonts::default());
        assert!(doc.contains("front_qr_s1.png"));
        assert!(!doc.contains("front_volumen_s1.png"));
        assert!(!doc.contains("front_colophon.png"));
    }

    #[test]
    fn restricted_volume_emits_no_front_matter() {
        let vol = volume_full(
            1,
            0,
            true,
            false,
            "",
            vec![unit("theseus", &["facsimile_theseus_1.png"])],
        );
        let doc = inner_document(&vol, &Fonts::default());
        assert!(!doc.contains("PLUTARCHI VITÆ"));
        assert!(!doc.contains("front_qr_s0.png"));
        assert!(!doc.contains("front_volumen_s0.png"));
        assert!(!doc.contains("front_colophon.png"));
        // No second TOMUS page and no tomus_label either.
        assert!(!doc.contains("TOMUS"));
    }

    #[test]
    fn volumen_title_only_first_of_source() {
        let first = volume_full(1, 0, true, true, "", vec![unit("a", &["x.png"])]);
        let later = volume_full(2, 0, false, true, "", vec![unit("b", &["y.png"])]);
        assert!(inner_document(&first, &Fonts::default()).contains("front_volumen_s0.png"));
        assert!(!inner_document(&later, &Fonts::default()).contains("front_volumen_s0.png"));
    }

    #[test]
    fn colophon_only_tomus_one() {
        let t1 = volume_full(1, 0, true, true, "", vec![unit("a", &["x.png"])]);
        let t2 = volume_full(2, 0, false, true, "", vec![unit("b", &["y.png"])]);
        assert!(inner_document(&t1, &Fonts::default()).contains("front_colophon.png"));
        assert!(!inner_document(&t2, &Fonts::default()).contains("front_colophon.png"));
    }

    #[test]
    fn unit_block_has_no_standalone_title_page() {
        let out = unit_block(&unit("theseus", &["f1.png", "f2.png"]));
        // The old standalone title page was `\heading{…}\clearpage`; now the heading
        // sits directly above the first image, with no intervening page break.
        assert!(!out.contains("\\heading{THESEUS}\n\\clearpage"));
        assert!(out.contains(
            "\\heading{THESEUS}\n\\noindent\\includegraphics[width=\\textwidth]{f1.png}"
        ));
    }

    #[test]
    fn unit_first_page_suppresses_running_head() {
        let out = unit_block(&unit("theseus", &["f1.png", "f2.png", "f3.png"]));
        // Only the first body page suppresses the running head.
        assert_eq!(out.matches("\\thispagestyle{empty}").count(), 1);
        // The suppression precedes the first image, not the later ones.
        let empty = out.find("\\thispagestyle{empty}").unwrap();
        let first_img = out.find("{f1.png}").unwrap();
        let second_img = out.find("{f2.png}").unwrap();
        assert!(empty < first_img && first_img < second_img);
    }

    #[test]
    fn unit_header_is_latin_only() {
        let out = unit_block(&unit("theseus", &["f1.png"]));
        assert!(out.contains("\\heading{THESEUS}"));
        // No Greek title is emitted.
        assert!(!out.contains("ΘΗΣΕΥΣ"));
        assert!(!out.contains("\\\\"));
    }

    #[test]
    #[ignore = "requires source PDFs, cached front matter, Poppler, and MiKTeX"]
    fn monitum_combined_compile_has_adjacent_content_pages() {
        use crate::{compile, render};
        use std::process::Command;

        assert!(render::pdftotext_available(), "pdftotext is required");
        let dir = tempfile::tempdir().unwrap();
        let sources = [
            PathBuf::from("Ploutarchou_Bioi_v1.pdf"),
            PathBuf::from("Ploutarchou_Bioi_v2.pdf"),
        ];
        let cache =
            frontmatter::load_cached_front_matter(Path::new(frontmatter::SIDECAR), &sources)
                .expect("load real front-matter cache read-only");
        let prose = latex::front_prose_pages_until(
            &cache,
            &frontmatter::MONITUM_PROSE_PAGES,
            frontmatter::MONITUM_PROSE_SENTINEL,
        );
        let pdf = render::Pdf::open(&sources[0]).expect("open v1 source PDF");
        frontmatter::render_front_scans(0, true, true, &pdf, dir.path())
            .expect("render real front scans");
        let body = image::GrayImage::from_fn(300, 100, |x, y| {
            image::Luma([if (10..290).contains(&x) && (10..90).contains(&y) {
                0
            } else {
                255
            }])
        });
        for name in ["theseus_first.png", "theseus_second.png"] {
            body.save(dir.path().join(name)).unwrap();
        }
        let vol = volume_full(
            1,
            0,
            true,
            true,
            &prose,
            vec![unit(
                "theseus",
                &["theseus_first.png", "theseus_second.png"],
            )],
        );
        write_inner(&vol, dir.path(), &Fonts::default()).unwrap();
        compile::compile(dir.path(), "tomus_1_inner.tex", compile::INNER_PASSES)
            .unwrap_or_else(|e| panic!("compile in {}: {e:#}", dir.path().display()));
        let page_count = compile::inner_page_count(dir.path(), "tomus_1_inner").unwrap();
        let inner_pdf = dir.path().join("tomus_1_inner.pdf");
        let pages: Vec<String> = (1..=page_count)
            .map(|page| {
                render::page_text(&inner_pdf, page).unwrap_or_else(|e| {
                    panic!("extract page {page} of {}: {e:#}", inner_pdf.display())
                })
            })
            .collect();
        let prose_pages: Vec<usize> = pages
            .iter()
            .enumerate()
            .filter_map(|(i, text)| {
                let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
                normalized
                    .contains(frontmatter::MONITUM_PROSE_SENTINEL)
                    .then_some(i + 1)
            })
            .collect();
        assert_eq!(
            prose_pages.len(),
            1,
            "sentinel pages {prose_pages:?} in {}",
            inner_pdf.display()
        );
        let prose_page = prose_pages[0];
        let body_page = pages
            .iter()
            .enumerate()
            .skip(prose_page)
            .find_map(|(i, text)| text.contains("THESEUS").then_some(i + 1))
            .unwrap_or_else(|| {
                panic!(
                    "no THESEUS after page {prose_page} in {}",
                    inner_pdf.display()
                )
            });
        assert_eq!(
            body_page,
            prose_page + 2,
            "body adjacency in {}",
            inner_pdf.display()
        );
        assert!(
            !pages[body_page - 1].lines().any(|line| {
                let line = line.trim();
                !line.is_empty() && line.chars().all(|c| c.is_ascii_digit())
            }),
            "opening page {body_page} has a folio in {}",
            inner_pdf.display()
        );
        assert!(
            pages[body_page].contains("THESEUS")
                && pages[body_page].lines().any(|l| l.trim() == "2"),
            "second body page {} lacks running head/folio 2 in {}: {:?}",
            body_page + 1,
            inner_pdf.display(),
            pages[body_page]
        );

        let dimensions: Vec<_> = [
            frontmatter::front_monitum_a_name(),
            frontmatter::front_monitum_b_name(),
        ]
        .iter()
        .map(|name| image::image_dimensions(dir.path().join(name)).unwrap())
        .collect();
        assert_ne!(dimensions[0], dimensions[1]);
        let output = Command::new(
            std::env::var("PLUTARCH_PDFIMAGES").unwrap_or_else(|_| "pdfimages".into()),
        )
        .arg("-list")
        .arg(&inner_pdf)
        .output()
        .unwrap_or_else(|e| panic!("pdfimages on {}: {e}", inner_pdf.display()));
        let listing = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "pdfimages on {}: {listing}\n{}",
            inner_pdf.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        for (width, height) in dimensions {
            let matches: Vec<u32> = listing
                .lines()
                .filter_map(|line| {
                    let fields: Vec<_> = line.split_whitespace().collect();
                    if fields.len() < 5 || fields[2] != "image" {
                        return None;
                    }
                    let page = fields[0].parse::<u32>().ok()?;
                    fields[1].parse::<u32>().ok()?;
                    (fields[3].parse::<u32>().ok()? == width
                        && fields[4].parse::<u32>().ok()? == height)
                        .then_some(page)
                })
                .collect();
            assert_eq!(
                matches,
                vec![(prose_page + 1) as u32],
                "crop {width}x{height} in {}:\n{listing}",
                inner_pdf.display()
            );
        }
        let rendered = render::render_page(
            &inner_pdf,
            (prose_page + 1) as u32,
            100,
            &dir.path().join("renders"),
        )
        .unwrap_or_else(|e| {
            panic!(
                "render errata page {} of {}: {e:#}",
                prose_page + 1,
                inner_pdf.display()
            )
        });
        let image = image::open(&rendered).unwrap().to_luma8();
        for (start, end) in [
            (0, image.height() / 2),
            (image.height() / 2, image.height()),
        ] {
            let dark = image
                .enumerate_pixels()
                .filter(|(_, y, pixel)| *y >= start && *y < end && pixel[0] < 128)
                .count();
            let total = u64::from(image.width()) * u64::from(end - start);
            assert!(
                dark as u64 * 100 >= total,
                "insufficient ink in rows {start}..{end} of {}",
                rendered.display()
            );
        }
        println!(
            "Verified {} pages: prose {prose_page}, errata {}, body {body_page}, next body folio 2",
            page_count,
            prose_page + 1
        );
    }

    #[test]
    fn write_inner_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let vol = volume(vec![unit("theseus", &["facsimile_theseus_1.png"])]);
        let path = write_inner(&vol, dir.path(), &Fonts::default()).unwrap();
        assert!(path.exists());
        assert_eq!(path.file_name().unwrap(), "tomus_1_inner.tex");
    }
}
