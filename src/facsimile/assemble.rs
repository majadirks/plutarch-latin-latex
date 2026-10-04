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
            doc.push_str(&latex::embed_scan(frontmatter::front_monitum_a_name()));
            doc.push_str(&latex::embed_scan(frontmatter::front_monitum_b_name()));
        }
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

/// One unit: a fresh page with the running head set to the Latin name, then one body
/// image per page. There is no standalone title page; the unit's first body page
/// opens with a large centered inline chapter header (the Latin title only) and
/// carries no running head, while later pages keep the centered Latin running head.
fn unit_block(unit: &PlacedUnit) -> String {
    let latin = latex::escape_latex(&unit.latin_title);
    let mut out = String::new();
    out.push_str("\\freshpage\n");
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
        let vol = volume(vec![unit(
            "theseus",
            &[
                "facsimile_theseus_1.png",
                "facsimile_theseus_2.png",
                "facsimile_theseus_3.png",
            ],
        )]);
        let doc = inner_document(&vol, &Fonts::default());
        let images = doc.matches("\\includegraphics[width=\\textwidth]").count();
        assert_eq!(images, 3);
        // Each image is followed by a page break.
        let breaks = doc.matches("\\par\\clearpage").count();
        assert_eq!(breaks, 3);
        assert!(doc.contains("{facsimile_theseus_2.png}"));
    }

    #[test]
    fn inner_tex_starts_each_unit_fresh_page() {
        let vol = volume(vec![
            unit("theseus", &["facsimile_theseus_1.png"]),
            unit("romulus", &["facsimile_romulus_1.png"]),
        ]);
        let doc = inner_document(&vol, &Fonts::default());
        // A fresh page opens each unit (the preamble also *defines* \freshpage, so
        // count the unit openings via the \freshpage + \markboth pairing).
        assert_eq!(doc.matches("\\freshpage\n\\markboth").count(), 2);
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
    fn write_inner_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let vol = volume(vec![unit("theseus", &["facsimile_theseus_1.png"])]);
        let path = write_inner(&vol, dir.path(), &Fonts::default()).unwrap();
        assert!(path.exists());
        assert_eq!(path.file_name().unwrap(), "tomus_1_inner.tex");
    }
}
