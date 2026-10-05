//! Stitches the reconciled per-page transcriptions into logical units: detects
//! Life and Comparatio boundaries from display headings, binds each Comparatio to
//! the two Lives it names into a pair-group, and concatenates a unit's pages into
//! a page-independent block stream with cross-page paragraphs rejoined.

use crate::tags::{Block, BlockKind, LifeBoundary, LifeKind, PageDoc, PageRole, Sidecar};

/// Walk the body pages in order and detect Life/Comparatio boundaries from their
/// display headings, assigning each unit a pair-group: a Comparatio shares its
/// group with the Lives it names, and an unpaired Life gets its own group.
pub fn detect_lives(sidecar: &Sidecar) -> Vec<LifeBoundary> {
    let mut units: Vec<LifeBoundary> = Vec::new();

    for (index, record) in sidecar.pages.iter().enumerate() {
        if record.role != PageRole::Body {
            continue;
        }
        let Some(doc) = &record.result else {
            continue;
        };
        for block in &doc.blocks {
            if block.kind == BlockKind::Heading {
                let title = block.text.trim().to_string();
                units.push(LifeBoundary {
                    id: slug(&title),
                    title: title.clone(),
                    kind: classify(&title),
                    group: 0,
                    first_page: index,
                    last_page: index,
                });
            }
        }
        if let Some(last) = units.last_mut() {
            last.last_page = index;
        }
    }

    assign_groups(&mut units);
    units
}

/// Whether a heading names a Comparatio (synkrisis) or a Life.
fn classify(title: &str) -> LifeKind {
    if title.to_uppercase().contains("COMPARATIO") {
        LifeKind::Comparatio
    } else {
        LifeKind::Life
    }
}

/// A URL/id-safe slug of a title.
fn slug(title: &str) -> String {
    title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// Assign pair-group numbers: a run of Lives followed by a Comparatio forms one
/// group with the Lives the Comparatio names; Lives with no Comparatio each get
/// their own group.
fn assign_groups(units: &mut [LifeBoundary]) {
    let mut group = 0u32;
    let mut i = 0usize;
    while i < units.len() {
        if units[i].kind == LifeKind::Comparatio {
            group += 1;
            units[i].group = group;
            i += 1;
            continue;
        }
        // A run of consecutive Lives.
        let start = i;
        while i < units.len() && units[i].kind == LifeKind::Life {
            i += 1;
        }
        let run = start..i;
        if i < units.len() && units[i].kind == LifeKind::Comparatio {
            // The Comparatio and the Lives it names share a fresh group; any other
            // Lives in the run get their own.
            let names = life_names_in(&units[i].title);
            group += 1;
            let comp_group = group;
            for k in run {
                if names_any(&units[k].title, &names) {
                    units[k].group = comp_group;
                } else {
                    group += 1;
                    units[k].group = group;
                }
            }
            units[i].group = comp_group;
            i += 1;
        } else {
            for k in run {
                group += 1;
                units[k].group = group;
            }
        }
    }
}

/// The life-name tokens a Comparatio title names, e.g. `COMPARATIO THESEI ET
/// ROMULI` yields `["THESEI", "ROMULI"]`.
fn life_names_in(comparatio_title: &str) -> Vec<String> {
    comparatio_title
        .split_whitespace()
        .map(|t| {
            t.trim_matches(|c: char| !c.is_alphanumeric())
                .to_uppercase()
        })
        .filter(|t| t != "COMPARATIO" && t != "ET" && !t.is_empty())
        .collect()
}

/// Whether a Life title matches any of a Comparatio's named tokens, by a
/// case-insensitive longest-common-prefix of at least four letters (so the
/// genitive `THESEI` matches the nominative `THESEUS`).
fn names_any(life_title: &str, names: &[String]) -> bool {
    let life = life_title.to_uppercase();
    names.iter().any(|name| token_matches(&life, name))
}

/// The project's title-token matcher: two tokens match when they share a
/// common prefix of at least four characters (so the genitive `THESEI` matches the
/// nominative `THESEUS`) or are identical (so a short name like `DIO`, too short to
/// reach a four-character prefix, still matches itself). Callers fold orthography
/// first ([`crate::verify::fold`]) so OCR and 1857 spelling noise are already
/// erased; this compares the folded forms.
pub(crate) fn token_matches(a: &str, b: &str) -> bool {
    a == b || common_prefix_len(a, b) >= 4
}

pub(crate) fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

/// Concatenate a unit's pages into one logical block stream, rejoining a block
/// flagged `continues_previous` onto the preceding paragraph so a sentence split
/// across a page break is made whole.
pub fn assembled_text(sidecar: &Sidecar, life: &LifeBoundary) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    for record in &sidecar.pages[life.first_page..=life.last_page] {
        let Some(doc) = &record.result else {
            continue;
        };
        append_blocks(&mut out, doc);
    }
    out
}

pub(crate) fn append_blocks(out: &mut Vec<Block>, doc: &PageDoc) {
    for block in &doc.blocks {
        let continuable = matches!(block.kind, BlockKind::Paragraph | BlockKind::Verse);
        if block.continues_previous && continuable {
            if let Some(last) = out.last_mut() {
                if last.kind == block.kind {
                    if !last.text.is_empty() && !block.text.is_empty() {
                        last.text.push(' ');
                    }
                    last.text.push_str(&block.text);
                    continue;
                }
            }
        }
        let mut joined = block.clone();
        joined.continues_previous = false;
        out.push(joined);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tags::{PageRecord, PageStatus, SourcePdf};

    fn block(kind: BlockKind, text: &str) -> Block {
        Block {
            kind,
            roman: None,
            section_no: None,
            continues_previous: false,
            text: text.to_string(),
        }
    }

    fn body_page(page: u32, blocks: Vec<Block>) -> PageRecord {
        PageRecord {
            source: 0,
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

    fn sidecar(pages: Vec<PageRecord>) -> Sidecar {
        Sidecar {
            schema_version: crate::tags::SCHEMA_VERSION,
            sources: vec![SourcePdf {
                path: "v1.pdf".to_string(),
                page_count: 100,
                file_sha256: "a".to_string(),
            }],
            front_matter: Default::default(),
            pages,
            lives: Vec::new(),
            batches: Vec::new(),
        }
    }

    #[test]
    fn detect_lives_finds_boundaries_from_headings() {
        let sc = sidecar(vec![
            body_page(
                1,
                vec![
                    block(BlockKind::Heading, "THESEUS"),
                    block(BlockKind::Paragraph, "a"),
                ],
            ),
            body_page(2, vec![block(BlockKind::Paragraph, "b")]),
            body_page(
                3,
                vec![
                    block(BlockKind::Heading, "ROMULUS"),
                    block(BlockKind::Paragraph, "c"),
                ],
            ),
        ]);
        let lives = detect_lives(&sc);
        assert_eq!(lives.len(), 2);
        assert_eq!(lives[0].title, "THESEUS");
        assert_eq!(lives[0].first_page, 0);
        assert_eq!(lives[0].last_page, 1);
        assert_eq!(lives[1].title, "ROMULUS");
        assert_eq!(lives[1].first_page, 2);
        assert_eq!(lives[1].last_page, 2);
    }

    #[test]
    fn detect_lives_groups_pair_and_comparatio() {
        let sc = sidecar(vec![
            body_page(1, vec![block(BlockKind::Heading, "THESEUS")]),
            body_page(2, vec![block(BlockKind::Heading, "ROMULUS")]),
            body_page(
                3,
                vec![block(BlockKind::Heading, "COMPARATIO THESEI ET ROMULI")],
            ),
        ]);
        let lives = detect_lives(&sc);
        assert_eq!(lives.len(), 3);
        assert_eq!(lives[0].group, lives[1].group);
        assert_eq!(lives[1].group, lives[2].group);
        assert_eq!(lives[2].kind, LifeKind::Comparatio);
    }

    #[test]
    fn detect_lives_binds_comparatio_to_named_lives() {
        let sc = sidecar(vec![
            body_page(1, vec![block(BlockKind::Heading, "THESEUS")]),
            body_page(2, vec![block(BlockKind::Heading, "ROMULUS")]),
            body_page(
                3,
                vec![block(BlockKind::Heading, "COMPARATIO THESEI ET ROMULI")],
            ),
            body_page(4, vec![block(BlockKind::Heading, "LYCURGUS")]),
            body_page(5, vec![block(BlockKind::Heading, "NUMA")]),
            body_page(
                6,
                vec![block(BlockKind::Heading, "COMPARATIO LYCURGI ET NUMAE")],
            ),
        ]);
        let lives = detect_lives(&sc);
        let g1 = lives[2].group;
        let g2 = lives[5].group;
        assert_ne!(g1, g2);
        assert_eq!(lives[0].group, g1);
        assert_eq!(lives[1].group, g1);
        assert_eq!(lives[3].group, g2);
        assert_eq!(lives[4].group, g2);
    }

    #[test]
    fn detect_lives_handles_life_without_comparatio() {
        let sc = sidecar(vec![
            body_page(1, vec![block(BlockKind::Heading, "ARATUS")]),
            body_page(2, vec![block(BlockKind::Heading, "ARTAXERXES")]),
        ]);
        let lives = detect_lives(&sc);
        assert_eq!(lives.len(), 2);
        // Two unpaired Lives are not forced into one pair-group.
        assert_ne!(lives[0].group, lives[1].group);
    }

    #[test]
    fn assembled_text_rejoins_continued_paragraph() {
        let mut tail = block(BlockKind::Paragraph, "sentence end.");
        tail.continues_previous = true;
        let sc = sidecar(vec![
            body_page(1, vec![block(BlockKind::Paragraph, "A split")]),
            body_page(2, vec![tail]),
        ]);
        let life = LifeBoundary {
            id: "x".to_string(),
            title: "X".to_string(),
            kind: LifeKind::Life,
            group: 1,
            first_page: 0,
            last_page: 1,
        };
        let blocks = assembled_text(&sc, &life);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].text, "A split sentence end.");
    }

    #[test]
    fn assembled_text_preserves_numbering_order() {
        let mut chapter = block(BlockKind::Chapter, "");
        chapter.roman = Some("XI.".to_string());
        let mut section = block(BlockKind::Section, "");
        section.section_no = Some(2);
        let sc = sidecar(vec![body_page(
            1,
            vec![chapter, section, block(BlockKind::Paragraph, "text")],
        )]);
        let life = LifeBoundary {
            id: "x".to_string(),
            title: "X".to_string(),
            kind: LifeKind::Life,
            group: 1,
            first_page: 0,
            last_page: 0,
        };
        let blocks = assembled_text(&sc, &life);
        assert_eq!(blocks[0].kind, BlockKind::Chapter);
        assert_eq!(blocks[0].roman.as_deref(), Some("XI."));
        assert_eq!(blocks[1].kind, BlockKind::Section);
        assert_eq!(blocks[1].section_no, Some(2));
        assert_eq!(blocks[2].kind, BlockKind::Paragraph);
    }
}
