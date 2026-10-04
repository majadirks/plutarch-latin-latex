//! The hand-authored unit table that drives the facsimile body. Because the
//! facsimile path never transcribes the body, it cannot detect Life/Comparatio
//! boundaries from headings the way [`crate::assemble::detect_lives`] does; the
//! boundaries come instead from a committed, auditable JSON table of every Life
//! and Comparatio in source order.

use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::tags::LifeKind;

/// One Life or Comparatio: its titles, pairing group, source, and the source page
/// its text opens on. End pages are derived in [`ranges`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub id: String,
    pub latin_title: String,
    pub greek_title: String,
    pub kind: LifeKind,
    /// Pair-group number binding two Lives and their Comparatio, matching the
    /// grouping [`crate::volumes::split_into_volumes`] enforces.
    pub group: u32,
    /// Index into the source PDFs (0 = v1, 1 = v2).
    pub source: usize,
    /// 1-based source page the unit's text opens on.
    pub start_page: u32,
    /// How many leading line-bands to drop on the unit's first source page (the
    /// work title / Life title / edition cross-reference). `None` falls back to the
    /// reflow heuristic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_top_bands: Option<u32>,
    /// Top of the printed title on the opening page, as a page-height fraction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_top_frac: Option<f32>,
    /// Top of the first body line on the opening page, as a page-height fraction.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_top_frac: Option<f32>,
    /// The inclusive last source body page of this unit, recorded by detection only
    /// on each source's final unit (whose end cannot be derived from a following
    /// unit's `start_page`). It excludes the back-matter/library pages the source
    /// page count would otherwise include. `None` falls back to the source's last
    /// body page passed to [`ranges`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_page: Option<u32>,
    /// A human correction the detector must not overwrite: when `true`, a
    /// regeneration ([`crate::facsimile::detect::load_or_detect`]) keeps this row's
    /// fields verbatim instead of replacing them with freshly detected values.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub pinned: bool,
    /// Free-text provenance from the optional Claude Vision line review
    /// (`--review-lines`), e.g. `"vision: title at band 4"`. Advisory only; the
    /// value the review acts on is recorded in `skip_top_bands`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_notes: Option<String>,
}

/// A unit paired with the inclusive last source page of its body range.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitRange {
    pub unit: Unit,
    pub end_page: u32,
    /// Whether `end_page` is a page **shared** with the next unit in the same source
    /// (the next unit's text opens partway down it). When set, the reflow keeps only
    /// the bands above the next unit's printed title on that page; otherwise the
    /// whole `end_page` belongs to this unit.
    pub shared_tail: bool,
}

/// Load and validate the hand-authored `facsimile_units.json`.
pub fn load_units(path: &Path) -> Result<Vec<Unit>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading unit table {}", path.display()))?;
    let units: Vec<Unit> =
        serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    validate(&units)?;
    Ok(units)
}

/// Enforce the table's invariants: non-empty titles, strictly increasing
/// `start_page` within a source, and all v1 units before all v2 units.
pub(crate) fn validate(units: &[Unit]) -> Result<()> {
    if units.is_empty() {
        bail!("unit table is empty");
    }
    for unit in units {
        if unit.latin_title.trim().is_empty() {
            bail!("unit {:?} has an empty latin_title", unit.id);
        }
        if unit.greek_title.trim().is_empty() {
            bail!("unit {:?} has an empty greek_title", unit.id);
        }
    }
    for pair in units.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        if b.source < a.source {
            bail!(
                "unit {:?} (source {}) appears after source {}; v1 units must precede v2",
                b.id,
                b.source,
                a.source
            );
        }
        if a.source == b.source && b.start_page <= a.start_page {
            bail!(
                "unit {:?} start_page {} does not increase within source {} (after {:?} @ {})",
                b.id,
                b.start_page,
                b.source,
                a.id,
                a.start_page
            );
        }
    }
    Ok(())
}

/// Derive each unit's inclusive page range. When the next unit is in the same
/// source its text opens partway down the next unit's `start_page`, which is shared:
/// the current unit's `end_page` is that page (inclusive) with `shared_tail` set, so
/// the reflow keeps only the bands above the next unit's printed title there. The
/// final unit of a source runs to its `last_page` (or `source_last_body_page[s]`)
/// with `shared_tail` false. `source_last_body_page[s]` is the inclusive last body
/// page of source `s`.
pub fn ranges(units: &[Unit], source_last_body_page: &[u32]) -> Vec<UnitRange> {
    let mut out = Vec::with_capacity(units.len());
    for (i, unit) in units.iter().enumerate() {
        let (end_page, shared_tail) = match units.get(i + 1) {
            Some(next) if next.source == unit.source => (next.start_page, true),
            _ => {
                let end = unit.last_page.unwrap_or_else(|| {
                    source_last_body_page
                        .get(unit.source)
                        .copied()
                        .unwrap_or(unit.start_page)
                });
                (end, false)
            }
        };
        out.push(UnitRange {
            unit: unit.clone(),
            end_page,
            shared_tail,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_fraction_round_trip_and_legacy_defaults() {
        let mut row = unit("romulus", 0, 33);
        let legacy = serde_json::to_string(&row).unwrap();
        assert!(!legacy.contains("top_frac"));
        assert_eq!(serde_json::from_str::<Unit>(&legacy).unwrap(), row);
        row.title_top_frac = Some(0.39);
        row.body_top_frac = Some(0.43);
        assert_eq!(
            serde_json::from_str::<Unit>(&serde_json::to_string(&row).unwrap()).unwrap(),
            row
        );
    }

    fn unit(id: &str, source: usize, start: u32) -> Unit {
        Unit {
            id: id.to_string(),
            latin_title: id.to_uppercase(),
            greek_title: "Τ".to_string(),
            kind: LifeKind::Life,
            group: 1,
            source,
            start_page: start,
            skip_top_bands: None,
            title_top_frac: None,
            body_top_frac: None,
            last_page: None,
            pinned: false,
            review_notes: None,
        }
    }

    fn write_units(units: &[Unit]) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("facsimile_units.json");
        std::fs::write(&path, serde_json::to_string_pretty(units).unwrap()).unwrap();
        (dir, path)
    }

    #[test]
    fn load_units_parses_and_validates() {
        let units = vec![
            unit("theseus", 0, 14),
            unit("romulus", 0, 46),
            Unit {
                kind: LifeKind::Comparatio,
                ..unit("comp_thesei_romuli", 0, 79)
            },
            unit("nicias", 1, 15),
        ];
        let (_dir, path) = write_units(&units);
        let loaded = load_units(&path).unwrap();
        assert_eq!(loaded.len(), 4);
        assert_eq!(loaded[0].id, "theseus");
        assert_eq!(loaded[2].kind, LifeKind::Comparatio);
        assert_eq!(loaded[3].source, 1);
    }

    #[test]
    fn unit_table_roundtrips_review_fields() {
        // A unit carrying review output round-trips through JSON…
        let mut u = unit("theseus", 0, 14);
        u.skip_top_bands = Some(4);
        u.review_notes = Some("vision: title at band 3".to_string());
        let json = serde_json::to_string_pretty(&[u.clone()]).unwrap();
        let back: Vec<Unit> = serde_json::from_str(&json).unwrap();
        assert_eq!(back[0].skip_top_bands, Some(4));
        assert_eq!(
            back[0].review_notes.as_deref(),
            Some("vision: title at band 3")
        );
        // …and a table without the fields still parses (they default to None).
        let minimal = r#"[{"id":"x","latin_title":"X","greek_title":"Τ","kind":"life","group":1,"source":0,"start_page":5}]"#;
        let parsed: Vec<Unit> = serde_json::from_str(minimal).unwrap();
        assert_eq!(parsed[0].review_notes, None);
        assert_eq!(parsed[0].skip_top_bands, None);
    }

    #[test]
    fn load_units_rejects_unsorted() {
        // start_page does not increase within source 0.
        let units = vec![unit("a", 0, 46), unit("b", 0, 14)];
        let (_dir, path) = write_units(&units);
        assert!(load_units(&path).is_err());

        // v2 unit appears before a v1 unit.
        let units = vec![unit("a", 1, 10), unit("b", 0, 14)];
        let (_dir, path) = write_units(&units);
        assert!(load_units(&path).is_err());

        // empty title.
        let mut bad = unit("a", 0, 14);
        bad.latin_title = "  ".to_string();
        let (_dir, path) = write_units(&[bad]);
        assert!(load_units(&path).is_err());
    }

    #[test]
    fn ranges_derives_end_pages() {
        let units = vec![
            unit("theseus", 0, 14),
            unit("romulus", 0, 46),
            unit("nicias", 1, 15),
        ];
        let ranges = ranges(&units, &[637, 713]);
        // Theseus shares its boundary page with Romulus: end_page is Romulus's
        // start (inclusive), with shared_tail set.
        assert_eq!(ranges[0].end_page, 46);
        assert!(ranges[0].shared_tail);
        // Romulus is the last v1 unit → runs to v1's last body page, not shared.
        assert_eq!(ranges[1].end_page, 637);
        assert!(!ranges[1].shared_tail);
        // The last v2 unit → v2 last body page, not shared.
        assert_eq!(ranges[2].end_page, 713);
        assert!(!ranges[2].shared_tail);
    }

    #[test]
    fn ranges_shares_boundary_page_in_source() {
        let units = vec![
            unit("theseus", 0, 14),
            unit("romulus", 0, 46),
            unit("nicias", 1, 15),
        ];
        let ranges = ranges(&units, &[637, 713]);
        // Consecutive same-source units share the boundary page.
        assert_eq!(ranges[0].end_page, ranges[1].unit.start_page);
        assert!(ranges[0].shared_tail);
        // Across the v1→v2 boundary the page is not shared.
        assert!(!ranges[1].shared_tail);
    }

    #[test]
    fn ranges_last_unit_runs_to_source_end() {
        let units = vec![unit("only", 0, 14)];
        let ranges = ranges(&units, &[637, 713]);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].end_page, 637);
        assert!(!ranges[0].shared_tail);
    }
}
