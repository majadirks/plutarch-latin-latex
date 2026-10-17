//! Estimates the trim pages each Life/Comparatio needs and splits the work into
//! roughly balanced output volumes on pair-group boundaries, with a hard volume
//! break forced at the v1→v2 source boundary.

use anyhow::{bail, Result};

use crate::signatures::pad_to_multiple_of_32;
use crate::tags::{Block, BlockKind, LifeBoundary, LifeKind, Sidecar};

/// Characters that fit on one typeset line of the single-column 10.5 cm page in
/// Old Standard. Tunable after the first real compile.
pub const CHARS_PER_LINE: u32 = 50;
/// Typeset lines per body page on the 16 cm page. Tunable after the first compile.
pub const LINES_PER_PAGE: u32 = 28;
/// Vertical allowance, in lines, for a unit's heading (each unit starts fresh).
pub const HEADING_LINES: u32 = 6;

/// Target physical pages per volume: ten whole 32-page signatures.
pub const TARGET_PAGES: u32 = 320;
/// Hard cap on physical pages per volume: twelve whole 32-page signatures.
pub const MAX_PAGES: u32 = 384;

/// Front-matter pages on **every** tomus: the Tomus title page + the QR page.
pub const FRONT_PER_TOMUS_PAGES: u32 = 2;
/// The engraved Volumen title, on the **first tomus of each source**.
pub const VOLUMEN_TITLE_PAGES: u32 = 1;
/// The printer's colophon, on **Tomus I only**.
pub const COLOPHON_PAGES: u32 = 1;
/// One re-typeset prose page plus one combined errata image page on Tomus I.
pub const MONITUM_PAGES: u32 = 2;
/// Hybrid Praefatio on v2's first tomus: three opening prose pages, two
/// balanced corrigenda image pages, and one closing prose page at current trim.
/// Imposition uses actual compiled counts rather than this planning estimate.
pub const PRAEFATIO_PAGES_EST: u32 = 6;
/// Blank-leaf pages in every volume: two flyleaves at each end, two PDF pages per
/// leaf.
pub const BLANK_LEAF_PAGES: u32 = 2 * 2 * 2;

/// The re-typeset front-matter page overhead a volume carries, by its place in the
/// work: every tomus gets the Tomus title page + QR page; the first tomus of each
/// source adds the Volumen title; Tomus I adds the colophon + Monitum; v2's first
/// tomus adds the Praefatio. Deliberately generous (`\freshpage` may round each up
/// to a recto); tuned in Phase 7.
pub fn front_overhead(source: usize, is_first_of_source: bool, is_tomus_one: bool) -> u32 {
    let mut pages = FRONT_PER_TOMUS_PAGES;
    if is_first_of_source {
        pages += VOLUMEN_TITLE_PAGES;
    }
    if is_tomus_one {
        pages += COLOPHON_PAGES + MONITUM_PAGES;
    }
    if is_first_of_source && source == 1 {
        pages += PRAEFATIO_PAGES_EST;
    }
    pages
}

/// One unit's page estimate, with the grouping and source needed to split.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitPages {
    pub id: String,
    pub group: u32,
    pub kind: LifeKind,
    pub source: usize,
    pub pages: u32,
}

/// Estimate the trim pages a unit's assembled text needs. Each unit starts a
/// fresh page, so the result is at least 1.
pub fn estimate_pages(blocks: &[Block]) -> u32 {
    let mut lines = HEADING_LINES;
    for block in blocks {
        match block.kind {
            BlockKind::Verse => {
                // Verse keeps its own line breaks; count them.
                lines += block.text.lines().count().max(1) as u32;
            }
            _ => {
                let chars = block.text.chars().count() as u32;
                lines += chars.div_ceil(CHARS_PER_LINE).max(1);
            }
        }
    }
    lines.div_ceil(LINES_PER_PAGE).max(1)
}

/// Build the per-unit page estimates from the detected Lives, reading each unit's
/// source from its first page and its text via [`crate::assemble::assembled_text`].
pub fn unit_pages(sidecar: &Sidecar, lives: &[LifeBoundary]) -> Vec<UnitPages> {
    lives
        .iter()
        .map(|life| {
            let source = sidecar
                .pages
                .get(life.first_page)
                .map(|p| p.source)
                .unwrap_or(0);
            let blocks = crate::assemble::assembled_text(sidecar, life);
            UnitPages {
                id: life.id.clone(),
                group: life.group,
                kind: life.kind,
                source,
                pages: estimate_pages(&blocks),
            }
        })
        .collect()
}

/// The physical page count of a volume: its content pages plus the blank leaves and
/// the per-tomus front-matter overhead ([`front_overhead`]), padded to a whole
/// 32-page signature.
pub fn volume_physical_pages(
    content_pages: u32,
    source: usize,
    is_first_of_source: bool,
    is_tomus_one: bool,
) -> u32 {
    let front = front_overhead(source, is_first_of_source, is_tomus_one);
    pad_to_multiple_of_32(content_pages + BLANK_LEAF_PAGES + front)
}

/// Split the units into output volumes, returning each volume's unit ids. Whole
/// pair-groups are never split; a hard volume break is forced at the v1→v2 source
/// boundary (each source's groups are split independently and concatenated). The
/// forced break is only sound when the source boundary coincides with a
/// pair-group boundary, which is asserted.
pub fn split_into_volumes(units: &[UnitPages], target: u32, max: u32) -> Result<Vec<Vec<String>>> {
    let groups = pair_groups(units)?;
    check_source_boundary(units, &groups)?;

    let mut out: Vec<Vec<String>> = Vec::new();
    let sources = distinct_sources(units);
    for src in sources {
        let src_groups: Vec<&Vec<usize>> = groups
            .iter()
            .filter(|g| units[g[0]].source == src)
            .collect();
        let group_pages: Vec<u32> = src_groups
            .iter()
            .map(|g| g.iter().map(|&i| units[i].pages).sum())
            .collect();
        let split = split_source(&group_pages, target, max, src);
        for volume_groups in split {
            let ids: Vec<String> = volume_groups
                .into_iter()
                .flat_map(|gi| src_groups[gi].iter().map(|&i| units[i].id.clone()))
                .collect();
            out.push(ids);
        }
    }
    Ok(out)
}

/// Consecutive units sharing a group number, each a maximal pair-group.
fn pair_groups(units: &[UnitPages]) -> Result<Vec<Vec<usize>>> {
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for (i, unit) in units.iter().enumerate() {
        match groups.last_mut() {
            Some(g) if units[g[0]].group == unit.group => g.push(i),
            _ => groups.push(vec![i]),
        }
    }
    // A pair-group may not straddle the source boundary.
    for g in &groups {
        let src = units[g[0]].source;
        if g.iter().any(|&i| units[i].source != src) {
            bail!(
                "pair-group {} straddles the v1→v2 source boundary",
                units[g[0]].group
            );
        }
    }
    Ok(groups)
}

/// Verify the forced v1→v2 break is sound: v1's last unit closes a pair-group as a
/// Comparatio and v2's first unit opens a new one as a Life.
fn check_source_boundary(units: &[UnitPages], _groups: &[Vec<usize>]) -> Result<()> {
    let last_v1 = units.iter().rev().find(|u| u.source == 0);
    let first_v2 = units.iter().find(|u| u.source == 1);
    if let (Some(last), Some(first)) = (last_v1, first_v2) {
        if last.kind != LifeKind::Comparatio {
            bail!(
                "v1 does not end on a Comparatio (last unit {:?} is {:?}); the forced \
                 v1→v2 volume break would bisect a pair-group",
                last.id,
                last.kind
            );
        }
        if first.kind != LifeKind::Life {
            bail!(
                "v2 does not open with a Life (first unit {:?} is {:?})",
                first.id,
                first.kind
            );
        }
        if last.group == first.group {
            bail!("the v1→v2 boundary falls inside pair-group {}", last.group);
        }
    }
    Ok(())
}

fn distinct_sources(units: &[UnitPages]) -> Vec<usize> {
    let mut sources: Vec<usize> = units.iter().map(|u| u.source).collect();
    sources.sort_unstable();
    sources.dedup();
    sources
}

/// Split one source's pair-group page counts into volumes on a nearest-boundary
/// rule: a group is never divided; before adding the next group to a non-empty
/// volume, close that volume if adding would land farther from `target` than
/// closing now, or would push it past `max`. This balances the split instead of
/// floor-to-exceed, so a volume sitting just under target does not grab a whole
/// oversized group. The first volume carries the richer front-matter overhead
/// (and, for source 0, the Tomus I colophon/Monitum); every volume carries the
/// blank leaves and per-tomus overhead.
fn split_source(group_pages: &[u32], target: u32, max: u32, source: usize) -> Vec<Vec<usize>> {
    let overhead_of = |is_first: bool| {
        BLANK_LEAF_PAGES + front_overhead(source, is_first, is_first && source == 0)
    };
    let mut volumes: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    let mut sum = overhead_of(true);

    for (i, &pages) in group_pages.iter().enumerate() {
        if !current.is_empty() {
            let over_max = sum + pages > max;
            let pre_dev = (sum as i64 - target as i64).abs();
            let post_dev = (sum as i64 + pages as i64 - target as i64).abs();
            if over_max || post_dev > pre_dev {
                volumes.push(std::mem::take(&mut current));
                sum = overhead_of(false);
            }
        }
        current.push(i);
        sum += pages;
    }
    if !current.is_empty() {
        volumes.push(current);
    }
    volumes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(id: &str, group: u32, kind: LifeKind, source: usize, pages: u32) -> UnitPages {
        UnitPages {
            id: id.to_string(),
            group,
            kind,
            source,
            pages,
        }
    }

    /// A regular v1 (two pair-groups) followed by a regular v2 (one pair-group).
    fn regular_units() -> Vec<UnitPages> {
        vec![
            unit("theseus", 1, LifeKind::Life, 0, 100),
            unit("romulus", 1, LifeKind::Life, 0, 100),
            unit("comp_thes_rom", 1, LifeKind::Comparatio, 0, 20),
            unit("lycurgus", 2, LifeKind::Life, 0, 100),
            unit("numa", 2, LifeKind::Life, 0, 100),
            unit("comp_lyc_num", 2, LifeKind::Comparatio, 0, 20),
            unit("nicias", 3, LifeKind::Life, 1, 100),
            unit("crassus", 3, LifeKind::Life, 1, 100),
            unit("comp_nic_crass", 3, LifeKind::Comparatio, 1, 20),
        ]
    }

    #[test]
    fn estimate_pages_scales_with_length() {
        let short = vec![Block {
            kind: BlockKind::Paragraph,
            roman: None,
            section_no: None,
            continues_previous: false,
            text: "x".repeat(100),
        }];
        let long = vec![Block {
            kind: BlockKind::Paragraph,
            roman: None,
            section_no: None,
            continues_previous: false,
            text: "x".repeat(20_000),
        }];
        assert!(estimate_pages(&long) > estimate_pages(&short));
        assert!(estimate_pages(&short) >= 1);
    }

    #[test]
    fn split_never_breaks_a_pair_group() {
        let units = regular_units();
        let volumes = split_into_volumes(&units, 250, 400).unwrap();
        // Every group's ids must all land in the same volume.
        for group in [
            ["theseus", "romulus", "comp_thes_rom"],
            ["lycurgus", "numa", "comp_lyc_num"],
            ["nicias", "crassus", "comp_nic_crass"],
        ] {
            let vol = volumes
                .iter()
                .position(|v| v.contains(&group[0].to_string()))
                .unwrap();
            for id in group {
                assert!(volumes[vol].contains(&id.to_string()), "{id} split off");
            }
        }
    }

    #[test]
    fn each_volume_within_max_pages() {
        let units = regular_units();
        let volumes = split_into_volumes(&units, 250, 400).unwrap();
        for volume in &volumes {
            let content: u32 = volume
                .iter()
                .map(|id| units.iter().find(|u| &u.id == id).unwrap().pages)
                .sum();
            assert!(
                volume_physical_pages(content, 0, true, true) <= pad_to_multiple_of_32(MAX_PAGES)
            );
        }
    }

    #[test]
    fn split_closes_at_nearest_boundary() {
        // The measured v1 pair-group sizes: with target 400 / max 500, closing after
        // group 4 (sum ~389 incl. overhead) beats grabbing group 5 (~487), so group 5
        // opens the second volume.
        let units = vec![
            unit("g1", 1, LifeKind::Life, 0, 98),
            unit("g2", 2, LifeKind::Life, 0, 96),
            unit("g3", 3, LifeKind::Life, 0, 80),
            unit("g4", 4, LifeKind::Life, 0, 101),
            unit("g5", 5, LifeKind::Life, 0, 98),
        ];
        let volumes = split_into_volumes(&units, 400, 500).unwrap();
        assert_eq!(volumes.len(), 2, "{volumes:?}");
        assert!(volumes[0].contains(&"g4".to_string()));
        assert!(!volumes[0].contains(&"g5".to_string()));
        assert!(volumes[1].contains(&"g5".to_string()));
    }

    #[test]
    fn split_respects_max_when_nearest_would_exceed() {
        // Target far above any single volume, so the nearest-boundary rule alone
        // would never close; the MAX guard forces the split.
        let units = vec![
            unit("g1", 1, LifeKind::Life, 0, 100),
            unit("g2", 2, LifeKind::Life, 0, 100),
            unit("g3", 3, LifeKind::Life, 0, 100),
            unit("g4", 4, LifeKind::Life, 0, 100),
        ];
        let volumes = split_into_volumes(&units, 10_000, 300).unwrap();
        assert!(volumes.len() >= 2, "MAX must force a split: {volumes:?}");
        for v in &volumes {
            let content: u32 = v
                .iter()
                .map(|id| units.iter().find(|u| &u.id == id).unwrap().pages)
                .sum();
            assert!(content <= 300, "volume content {content} exceeds max");
        }
    }

    #[test]
    fn volumes_are_roughly_balanced() {
        // Twelve equal pair-groups in one source.
        let mut units = Vec::new();
        for g in 1..=12u32 {
            units.push(unit(&format!("a{g}"), g, LifeKind::Life, 0, 30));
            units.push(unit(&format!("b{g}"), g, LifeKind::Life, 0, 30));
            units.push(unit(&format!("c{g}"), g, LifeKind::Comparatio, 0, 10));
        }
        let volumes = split_into_volumes(&units, 250, 400).unwrap();
        let sums: Vec<u32> = volumes
            .iter()
            .map(|v| {
                v.iter()
                    .map(|id| units.iter().find(|u| &u.id == id).unwrap().pages)
                    .sum()
            })
            .collect();
        let max = *sums.iter().max().unwrap();
        let min = *sums.iter().min().unwrap();
        assert!(max - min <= 140, "volumes imbalanced: {sums:?}");
    }

    #[test]
    fn monitum_front_overhead_counts_two_content_pages() {
        assert_eq!(MONITUM_PAGES, 2);
        assert_eq!(front_overhead(0, true, true), 6);
    }

    #[test]
    fn overhead_charged_per_tomus_and_front_matter() {
        // Every volume carries the blank leaves + the per-tomus front pages (Tomus
        // title + QR).
        assert_eq!(
            volume_physical_pages(0, 0, false, false),
            pad_to_multiple_of_32(BLANK_LEAF_PAGES + FRONT_PER_TOMUS_PAGES)
        );
        // Tomus I (source 0, first, tomus one) adds Volumen title + colophon + Monitum.
        let t1 = BLANK_LEAF_PAGES
            + FRONT_PER_TOMUS_PAGES
            + VOLUMEN_TITLE_PAGES
            + COLOPHON_PAGES
            + MONITUM_PAGES;
        assert_eq!(
            volume_physical_pages(0, 0, true, true),
            pad_to_multiple_of_32(t1)
        );
        // v2's first tomus adds Volumen title + Praefatio (no colophon/Monitum).
        let v2_first =
            BLANK_LEAF_PAGES + FRONT_PER_TOMUS_PAGES + VOLUMEN_TITLE_PAGES + PRAEFATIO_PAGES_EST;
        assert_eq!(
            volume_physical_pages(0, 1, true, false),
            pad_to_multiple_of_32(v2_first)
        );
        // front_overhead is monotone: a plain later tomus < Tomus I.
        assert!(front_overhead(0, false, false) < front_overhead(0, true, true));
    }

    #[test]
    fn split_forces_break_at_v1_v2_boundary() {
        let units = regular_units();
        let volumes = split_into_volumes(&units, 10_000, 10_000).unwrap();
        // Even with a budget large enough to hold everything, v2 starts a new
        // volume: no volume mixes a v1 unit with a v2 unit.
        for volume in &volumes {
            let sources: Vec<usize> = volume
                .iter()
                .map(|id| units.iter().find(|u| &u.id == id).unwrap().source)
                .collect();
            assert!(
                sources.windows(2).all(|w| w[0] == w[1]),
                "volume mixes sources: {volume:?}"
            );
        }
    }

    #[test]
    fn split_errors_if_source_boundary_bisects_pair_group() {
        // v1 ends on a Life (no closing Comparatio) — the forced break would
        // bisect a pair-group.
        let units = vec![
            unit("theseus", 1, LifeKind::Life, 0, 100),
            unit("romulus", 1, LifeKind::Life, 0, 100),
            unit("nicias", 1, LifeKind::Life, 1, 100),
        ];
        assert!(split_into_volumes(&units, 250, 400).is_err());
    }
}
