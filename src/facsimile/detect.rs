//! Auto-detects the Life/Comparatio openings from the source PDFs' OCR text layer
//! and the canonical [`corpus`]. Each body page's running head names its current
//! unit, so tracing the running head down a page range segments the body into
//! contiguous unit runs; a unit's exact `start_page` is pinned by the short display
//! title line at its opening, which the running head (which may lag by a page or two
//! when early pages OCR poorly) cannot set on its own.

use std::ops::RangeInclusive;
use std::path::Path;

use anyhow::{bail, Context, Result};

use super::corpus;
use super::units::{self, Unit};
use crate::render::{running_head, title_token, Pdf, Word};
use crate::verify::fold;

/// Tokens that carry no identifying weight: folded `cum`/`seu` and any token of two
/// characters or fewer (which also drops `et` and bare initials like `C`).
const STOP: [&str; 2] = ["cvm", "sev"];

/// How many pages before a run's first running-head page to scan for the display
/// title. The running head can lag the true opening by a page or two when the first
/// pages OCR poorly (e.g. the first Life, whose opening page carries the work
/// banner instead of a clean running head).
const LOOKBACK: u32 = 3;

/// A unit's minimum run length in pages. One, because a Comparatio can occupy a
/// single trim page; isolated single-page misreads are already absorbed by
/// [`smooth`], so runs are never dropped for brevity.
const MIN_RUN_PAGES: u32 = 1;

/// Smoothing window: bridge gaps of up to this many pages that disagree with an
/// otherwise uniform run. These scans carry a clean running head only on one leaf of
/// each opening, so a unit's pages alternate with blank verso heads; a single
/// misread page sitting among those blanks widens a gap to three, which this spans.
/// A corpus unit never legitimately recurs, so a gap flanked by the same unit is
/// always noise and safe to fill.
const SMOOTH_WINDOW: u32 = 3;

/// Fold a title string into identifying tokens: whitespace-split, orthography-folded
/// ([`fold`]), with empties, stop words, and ≤2-character tokens dropped.
pub fn tokens(s: &str) -> Vec<String> {
    s.split_whitespace()
        .map(fold)
        .filter(|t| t.chars().count() > 2 && !STOP.contains(&t.as_str()))
        .collect()
}

const TITLE_TOP_ZONE_FRAC: f32 = 0.10;
const TITLE_LINE_EPSILON: f32 = 0.012;

/// Locate the known unit's printed title and first body line below top furniture.
pub fn title_box(words: &[Word], unit: usize) -> Option<(f32, f32)> {
    let wanted = tokens(corpus::corpus().get(unit)?.latin_title);
    let mut kept: Vec<&Word> = words
        .iter()
        .filter(|w| {
            w.x_max.is_finite()
                && w.x_max > 0.5
                && w.y_min > TITLE_TOP_ZONE_FRAC
                && w.y_min.is_finite()
        })
        .collect();
    kept.sort_by(|a, b| a.y_min.total_cmp(&b.y_min));
    let mut lines: Vec<(f32, Vec<&Word>)> = Vec::new();
    for word in kept {
        if let Some((y, line)) = lines.last_mut() {
            if word.y_min - *y < TITLE_LINE_EPSILON {
                line.push(word);
                continue;
            }
        }
        lines.push((word.y_min, vec![word]));
    }
    let matches = |line: &(f32, Vec<&Word>)| {
        line.1
            .iter()
            .filter_map(|w| title_token(&w.text))
            .flat_map(|t| tokens(&t))
            .any(|t| wanted.iter().any(|w| crate::assemble::token_matches(&t, w)))
    };
    let first = lines.iter().position(matches)?;
    let mut body = first + 1;
    while body < lines.len() && matches(&lines[body]) {
        body += 1;
    }
    Some((lines[first].0, lines.get(body)?.0))
}

/// The corpus unit a page belongs to, from its running head. `None` when the head
/// resolves to no unit (front/back matter, or a page whose head OCR'd too poorly).
pub fn page_unit(page_text: &str) -> Option<usize> {
    corpus::match_page(&tokens(&running_head(page_text)))
}

/// The unit a page *opens*, from its display-title line: the first short, title-like
/// line (1–3 folded tokens, ALL-CAPS in the source) that the corpus resolves. This,
/// not the running head, pins the exact `start_page`. `None` when the page carries no
/// such line.
pub fn title_line_unit(page_text: &str) -> Option<usize> {
    for line in page_text.lines() {
        let caps: Vec<String> = line.split_whitespace().filter_map(title_token).collect();
        if caps.is_empty() {
            continue;
        }
        let toks: Vec<String> = caps.iter().flat_map(|t| tokens(t)).collect();
        if toks.is_empty() || toks.len() > 3 {
            continue;
        }
        if let Some(unit) = corpus::match_page(&toks) {
            return Some(unit);
        }
    }
    None
}

/// Map each page's text (indexed by source page − 1) to its running-head unit.
pub fn page_units(texts: &[Option<String>]) -> Vec<Option<usize>> {
    texts
        .iter()
        .map(|t| t.as_deref().and_then(page_unit))
        .collect()
}

/// Fill a short gap (up to `window` pages) that disagrees with an otherwise uniform
/// run from its flanking neighbours, so one bad OCR page neither splits a run nor
/// spawns a phantom unit.
pub fn smooth(seq: &[Option<usize>], window: u32) -> Vec<Option<usize>> {
    let mut out = seq.to_vec();
    let n = out.len();
    let w = window.max(1) as usize;
    let mut i = 0;
    while i < n {
        let left = out[i];
        if left.is_none() {
            i += 1;
            continue;
        }
        // Scan the gap `(i, j)` that disagrees with `left`, bounded to `window`
        // pages. Fill it only when the page just past it carries `left` again, so a
        // real boundary to a different unit is never bridged.
        let mut j = i + 1;
        while j < n && out[j] != left && j - (i + 1) < w {
            j += 1;
        }
        if j < n && j > i + 1 && out[j] == left {
            for slot in out.iter_mut().take(j).skip(i + 1) {
                *slot = left;
            }
            i = j;
        } else {
            i += 1;
        }
    }
    out
}

/// A maximal contiguous span of one corpus unit, in 1-based page numbers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Run {
    pub unit: usize,
    pub start_page: u32,
    pub end_page: u32,
    pub title_top_frac: Option<f32>,
    pub body_top_frac: Option<f32>,
}

/// Maximal contiguous spans of one corpus id, as 1-based page ranges (page = index
/// + 1). Runs shorter than [`MIN_RUN_PAGES`] are dropped.
pub fn runs(smoothed: &[Option<usize>]) -> Vec<Run> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < smoothed.len() {
        match smoothed[i] {
            Some(unit) => {
                let start = i;
                while i < smoothed.len() && smoothed[i] == Some(unit) {
                    i += 1;
                }
                let run = Run {
                    unit,
                    start_page: (start + 1) as u32,
                    end_page: i as u32,
                    title_top_frac: None,
                    body_top_frac: None,
                };
                if run.end_page + 1 - run.start_page >= MIN_RUN_PAGES {
                    out.push(run);
                }
            }
            None => i += 1,
        }
    }
    out
}

/// Detect each unit's opening over `body_range`, pinning `start_page` to the first
/// page in/just before the unit's run whose display-title line names it, and falling
/// back to the run's first page when no title line is found. End pages chain to the
/// next unit's `start_page − 1`; the last unit runs to its own run end (the source's
/// last body page). `page_text_fn(page)` is injected so tests supply canned OCR
/// without a PDF.
pub fn detect_openings<F, G>(
    body_range: RangeInclusive<u32>,
    page_text_fn: F,
    page_words_fn: G,
) -> Result<Vec<Run>>
where
    F: Fn(u32) -> Result<String>,
    G: Fn(u32) -> Result<Vec<Word>>,
{
    let start = *body_range.start();
    let end = *body_range.end();
    let mut texts: Vec<Option<String>> = Vec::new();
    for page in start..=end {
        texts.push(Some(page_text_fn(page)?));
    }
    let smoothed = smooth(&page_units(&texts), SMOOTH_WINDOW);
    let base = runs(&smoothed);
    let offset = start - 1;
    let text_for = |page: u32| texts[(page - start) as usize].as_deref().unwrap_or("");

    // Pin each run's start to its display-title line, clamped above the previous
    // unit's start so starts strictly increase.
    let mut pinned: Vec<(usize, u32, u32)> = Vec::new();
    let mut prev_start: Option<u32> = None;
    for run in &base {
        let run_start = run.start_page + offset;
        let run_end = run.end_page + offset;
        let window_floor = run_start.saturating_sub(LOOKBACK).max(start);
        let lower = match prev_start {
            Some(ps) => window_floor.max(ps + 1),
            None => window_floor,
        };
        let mut chosen = run_start;
        for page in lower..=run_start {
            if title_line_unit(text_for(page)) == Some(run.unit) {
                chosen = page;
                break;
            }
        }
        pinned.push((run.unit, chosen, run_end));
        prev_start = Some(chosen);
    }

    let mut out = Vec::with_capacity(pinned.len());
    for (idx, &(unit, s, run_end)) in pinned.iter().enumerate() {
        let e = match pinned.get(idx + 1) {
            Some(&(_, next_start, _)) => next_start.saturating_sub(1),
            None => run_end,
        };
        let boundary = title_box(&page_words_fn(s)?, unit);
        out.push(Run {
            unit,
            start_page: s,
            end_page: e.max(s),
            title_top_frac: boundary.map(|b| b.0),
            body_top_frac: boundary.map(|b| b.1),
        });
    }
    Ok(out)
}

/// Build the unit table from detected runs: each run becomes a [`Unit`] taking its
/// titles, kind, group, and source from the corpus and its `start_page` from the
/// run. Errors when a corpus unit is detected twice, when a run's source disagrees
/// with the corpus, or when a corpus unit interior to a source's detected span was
/// never covered (a detection gap reported for a manual pin rather than silently
/// dropped).
pub fn units_from_runs(runs_v1: &[Run], runs_v2: &[Run]) -> Result<Vec<Unit>> {
    let canon = corpus::corpus();
    let mut seen = vec![false; canon.len()];
    let mut units: Vec<Unit> = Vec::new();
    for (expected_source, runs) in [(0usize, runs_v1), (1usize, runs_v2)] {
        for run in runs {
            let cu = &canon[run.unit];
            if cu.source != expected_source {
                bail!(
                    "corpus unit {:?} (source {}) detected in source {}",
                    cu.id,
                    cu.source,
                    expected_source
                );
            }
            if seen[run.unit] {
                bail!("corpus unit {:?} appears in more than one run", cu.id);
            }
            seen[run.unit] = true;
            units.push(Unit {
                id: cu.id.to_string(),
                latin_title: cu.latin_title.to_string(),
                greek_title: cu.greek_title.to_string(),
                kind: cu.kind,
                group: cu.group,
                source: cu.source,
                start_page: run.start_page,
                skip_top_bands: None,
                title_top_frac: run.title_top_frac,
                body_top_frac: run.body_top_frac,
                last_page: None,
                pinned: false,
                review_notes: None,
            });
        }
    }
    for source in 0..2 {
        let covered: Vec<usize> = canon
            .iter()
            .enumerate()
            .filter(|(i, u)| u.source == source && seen[*i])
            .map(|(i, _)| i)
            .collect();
        if let (Some(&lo), Some(&hi)) = (covered.first(), covered.last()) {
            let missing: Vec<&str> = canon
                .iter()
                .enumerate()
                .filter(|(i, u)| u.source == source && *i >= lo && *i <= hi && !seen[*i])
                .map(|(_, u)| u.id)
                .collect();
            if !missing.is_empty() {
                bail!("detection missing interior unit(s): {}", missing.join(", "));
            }
        }
    }
    units.sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then(a.start_page.cmp(&b.start_page))
    });

    // Record each source's last body page (its final run's end) on that source's
    // last unit, so a cached full build can bound it without re-detecting.
    for (expected_source, runs) in [(0usize, runs_v1), (1usize, runs_v2)] {
        let last_body = runs.iter().map(|r| r.end_page).max();
        if let (Some(end), Some(unit)) = (
            last_body,
            units.iter_mut().rfind(|u| u.source == expected_source),
        ) {
            unit.last_page = Some(end);
        }
    }

    units::validate(&units)?;
    Ok(units)
}

/// Overlay a human's pinned rows onto freshly detected units: a detected row whose
/// id matches a `pinned` existing row is replaced by that row verbatim, and a pinned
/// row with no detected counterpart is added, so hand corrections survive a
/// regeneration. The result is re-sorted into source/`start_page` order.
fn merge_pins(detected: Vec<Unit>, existing: &[Unit]) -> Vec<Unit> {
    let pins: Vec<&Unit> = existing.iter().filter(|u| u.pinned).collect();
    let mut out: Vec<Unit> = detected
        .into_iter()
        .map(|d| {
            pins.iter()
                .find(|p| p.id == d.id)
                .map(|p| (*p).clone())
                .unwrap_or(d)
        })
        .collect();
    for p in &pins {
        if !out.iter().any(|u| u.id == p.id) {
            out.push((*p).clone());
        }
    }
    out.sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then(a.start_page.cmp(&b.start_page))
    });
    out
}

/// Load the committed unit table, or regenerate it by detection. With `redetect`
/// false and the file present, it is loaded and validated. Otherwise detection runs
/// over both sources, pinned hand edits from any existing file are merged over the
/// result, and the merged table is written to `path` and returned. `page_text_fn`
/// (source, page) is injected so tests can supply canned OCR without a PDF.
pub fn load_or_detect<F, G>(
    path: &Path,
    pdfs: &[Pdf],
    redetect: bool,
    page_text_fn: F,
    page_words_fn: G,
) -> Result<Vec<Unit>>
where
    F: Fn(usize, u32) -> Result<String>,
    G: Fn(usize, u32) -> Result<Vec<Word>>,
{
    if !redetect && path.exists() {
        return units::load_units(path);
    }
    let mut runs: Vec<Vec<Run>> = Vec::new();
    for (source, pdf) in pdfs.iter().enumerate() {
        runs.push(detect_openings(
            1..=pdf.page_count,
            |page| page_text_fn(source, page),
            |page| page_words_fn(source, page),
        )?);
    }
    let detected = units_from_runs(&runs[0], &runs[1])?;
    let existing = if path.exists() {
        units::load_units(path).unwrap_or_default()
    } else {
        Vec::new()
    };
    let merged = merge_pins(detected, &existing);
    units::validate(&merged)?;
    let json = serde_json::to_string_pretty(&merged).context("serializing unit table")?;
    std::fs::write(path, json).with_context(|| format!("writing {}", path.display()))?;
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(text: &str, x: f32, y: f32) -> Word {
        Word {
            text: text.into(),
            x_min: x,
            x_max: x + 0.1,
            y_min: y,
            y_max: y + 0.008,
        }
    }

    #[test]
    fn title_box_two_line_comparatio_crossing_midpoint() {
        let words = vec![
            word("CATONIS", 0.6, 0.06),
            word("clarissimus.", 0.6, 0.51),
            word("ΑΡΙΣΤΕΙΔΟΥ", 0.15, 0.5491843),
            word("ARISTIDIS", 220.562 / 460.32, 392.381_2 / 714.48),
            word("CUM", 0.66, 0.5491843),
            word("CATΟNE", 0.75, 0.5491843),
            word("COMPARATIO.", 0.6, 0.57787454),
            word("Si", 0.55, 0.6090118),
        ];
        let (title, body) = title_box(&words, id("comparatio_aristidis_et_catonis")).unwrap();
        assert!((title - 0.5491843).abs() < 0.000001);
        assert!((body - 0.6090118).abs() < 0.000001);
    }

    #[test]
    fn title_box_excludes_greek_words_ending_before_midpoint() {
        let words = vec![
            word("ARISTIDIS", 0.2, 0.3),
            word("ΑΡΙΣΤΕΙΔΟΥ", 0.479, 0.4),
            word("ARISTIDIS", 0.479, 0.55),
            word("COMPARATIO.", 0.6, 0.58),
            word("Si", 0.55, 0.61),
        ];
        assert_eq!(
            title_box(&words, id("comparatio_aristidis_et_catonis")),
            Some((0.55, 0.61))
        );
        let mut invalid = word("ARISTIDIS", 0.6, 0.3);
        invalid.x_max = f32::NAN;
        let mut words = words;
        words.push(invalid);
        assert_eq!(
            title_box(&words, id("comparatio_aristidis_et_catonis")),
            Some((0.55, 0.61))
        );
    }

    #[test]
    fn title_box_narrow_midpage() {
        let words = vec![
            word("ROMULUS", 0.6, 0.06),
            word("tail", 0.6, 0.2),
            word("ROMULUS.", 0.6, 0.39),
            word("Ingens", 0.6, 0.43),
        ];
        assert_eq!(title_box(&words, id("romulus")), Some((0.39, 0.43)));
    }

    #[test]
    fn title_box_wide_two_line() {
        let words = vec![
            word("COMPARATIO", 0.51, 0.765),
            word("THESEI", 0.7, 0.766),
            word("ROMULO.", 0.6, 0.787),
            word("Habes", 0.6, 0.827),
        ];
        assert_eq!(
            title_box(&words, id("comparatio_thesei_et_romuli")),
            Some((0.765, 0.827))
        );
    }

    #[test]
    fn title_box_skips_left_greek_column() {
        let words = vec![
            word("ROMULUS", 0.1, 0.2),
            word("body", 0.1, 0.24),
            word("ROMULUS", 0.6, 0.39),
            word("Ingens", 0.6, 0.43),
        ];
        assert_eq!(title_box(&words, id("romulus")), Some((0.39, 0.43)));
    }

    #[test]
    fn title_box_none_when_absent() {
        assert_eq!(
            title_box(
                &[word("Romulus", 0.6, 0.3), word("body", 0.6, 0.34)],
                id("romulus")
            ),
            None
        );
        assert_eq!(title_box(&[word("ROMULUS", 0.6, 0.3)], id("romulus")), None);
    }

    fn id(id: &str) -> usize {
        corpus::corpus().iter().position(|u| u.id == id).unwrap()
    }

    /// One run per corpus unit, pages spaced so `start_page` strictly increases
    /// within each source, split into (v1, v2).
    fn full_runs() -> (Vec<Run>, Vec<Run>) {
        let mut v1 = Vec::new();
        let mut v2 = Vec::new();
        let (mut p1, mut p2) = (10u32, 10u32);
        for (i, u) in corpus::corpus().iter().enumerate() {
            if u.source == 0 {
                v1.push(Run {
                    unit: i,
                    start_page: p1,
                    end_page: p1 + 1,
                    title_top_frac: None,
                    body_top_frac: None,
                });
                p1 += 2;
            } else {
                v2.push(Run {
                    unit: i,
                    start_page: p2,
                    end_page: p2 + 1,
                    title_top_frac: None,
                    body_top_frac: None,
                });
                p2 += 2;
            }
        }
        (v1, v2)
    }

    #[test]
    fn tokens_fold_orthography() {
        assert_eq!(tokens("VITÆ"), tokens("VITAE"));
        assert_eq!(tokens("ROMVLVS"), tokens("ROMULUS"));
        // `cum`/`et` and bare initials drop out.
        assert_eq!(tokens("COMPARATIO THESEI CUM ROMULO").len(), 3);
        assert_eq!(tokens("C MARIUS"), tokens("MARIUS"));
    }

    #[test]
    fn page_unit_matches_comparatio_wording() {
        let a = page_unit("COMPARATIO THESEI CUM ROMULO. 56");
        let b = page_unit("THESEI ET ROMULI COMPARATIO. 56");
        assert!(a.is_some());
        assert_eq!(a, b);
        assert_eq!(a, Some(id("comparatio_thesei_et_romuli")));
    }

    #[test]
    fn page_unit_distinguishes_life_from_comparatio() {
        assert_eq!(page_unit("ROMULUS. 34"), Some(id("romulus")));
    }

    #[test]
    fn runs_segment_on_name_change() {
        let texts: Vec<Option<String>> = (0..10)
            .map(|i| {
                Some(if i < 5 {
                    "THESEUS. 1".to_string()
                } else {
                    "ROMULUS. 1".to_string()
                })
            })
            .collect();
        let r = runs(&smooth(&page_units(&texts), 1));
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].unit, id("theseus"));
        assert_eq!((r[0].start_page, r[0].end_page), (1, 5));
        assert_eq!(r[1].unit, id("romulus"));
        assert_eq!((r[1].start_page, r[1].end_page), (6, 10));
    }

    #[test]
    fn smooth_fills_single_bad_page() {
        let texts = vec![
            Some("THESEUS. 1".to_string()),
            Some("~~garbled~~".to_string()),
            Some("THESEUS. 3".to_string()),
        ];
        let r = runs(&smooth(&page_units(&texts), 1));
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].unit, id("theseus"));
        assert_eq!((r[0].start_page, r[0].end_page), (1, 3));
    }

    #[test]
    fn start_page_pinned_by_display_title() {
        // The running head only reads cleanly from page 4 on, but page 1 carries the
        // display-title line — detection must start the unit on page 1.
        let pages = |p: u32| -> Result<String> {
            Ok(match p {
                1 => "123\n...\n(2)\n\nTHESEUS.\n1. Quod faciunt".to_string(),
                2 | 3 => "~~garbled~~".to_string(),
                _ => "THESEUS. 10".to_string(),
            })
        };
        let detected = detect_openings(1..=6, pages, |_| Ok(vec![])).unwrap();
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].unit, id("theseus"));
        assert_eq!(detected[0].start_page, 1);
    }

    #[test]
    fn start_page_falls_back_to_run_when_no_title() {
        // Every page resolves to the unit via its running head, but no page has a
        // short (≤3-token) title line, so the start falls back to the run's start.
        let pages = |_p: u32| -> Result<String> {
            Ok("ALPHA BETA GAMMA DELTA THESEUS\n1. body text".to_string())
        };
        let detected = detect_openings(1..=3, pages, |_| Ok(vec![])).unwrap();
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].unit, id("theseus"));
        assert_eq!(detected[0].start_page, 1);
        assert_eq!(detected[0].end_page, 3);
    }

    #[test]
    fn units_from_runs_builds_valid_table() {
        let (v1, v2) = full_runs();
        let units = units_from_runs(&v1, &v2).unwrap();
        assert_eq!(units.len(), corpus::corpus().len());
        // A Comparatio shares its group with the Lives it pairs.
        let theseus = units.iter().find(|u| u.id == "theseus").unwrap();
        let romulus = units.iter().find(|u| u.id == "romulus").unwrap();
        let comp = units
            .iter()
            .find(|u| u.id == "comparatio_thesei_et_romuli")
            .unwrap();
        assert_eq!(theseus.group, comp.group);
        assert_eq!(romulus.group, comp.group);
        // All v1 units precede all v2 units.
        let last_v1 = units.iter().rposition(|u| u.source == 0).unwrap();
        let first_v2 = units.iter().position(|u| u.source == 1).unwrap();
        assert!(last_v1 < first_v2);
    }

    #[test]
    fn units_from_runs_carries_fracs() {
        let (mut v1, v2) = full_runs();
        v1[1].title_top_frac = Some(0.39);
        v1[1].body_top_frac = Some(0.43);
        let units = units_from_runs(&v1, &v2).unwrap();
        assert_eq!(units[1].title_top_frac, Some(0.39));
        assert_eq!(units[1].body_top_frac, Some(0.43));
    }

    #[test]
    fn units_from_runs_reports_missing_unit() {
        let (mut v1, v2) = full_runs();
        // Drop Romulus, an interior unit of v1's detected span.
        v1.retain(|r| r.unit != id("romulus"));
        assert!(units_from_runs(&v1, &v2).is_err());
    }

    #[test]
    fn load_or_detect_writes_when_absent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("facsimile_units.json");
        // Two v1 Lives plus their Comparatio, two pages each; v2 empty.
        let pages = |source: usize, page: u32| -> Result<String> {
            if source != 0 {
                return Ok(String::new());
            }
            Ok(match page {
                1..=2 => "THESEUS. 1".to_string(),
                3..=4 => "ROMULUS. 1".to_string(),
                5..=6 => "COMPARATIO THESEI CUM ROMULO. 1".to_string(),
                _ => String::new(),
            })
        };
        let pdfs = [stub_pdf("v1", 6), stub_pdf("v2", 0)];
        let boxes = |_: usize, p: u32| {
            Ok(match p {
                3 => vec![word("ROMULUS", 0.6, 0.39), word("Ingens", 0.6, 0.43)],
                5 => vec![
                    word("COMPARATIO", 0.6, 0.765),
                    word("ROMULO", 0.6, 0.787),
                    word("Habes", 0.6, 0.827),
                ],
                _ => vec![],
            })
        };
        let units = load_or_detect(&path, &pdfs, false, pages, boxes).unwrap();
        assert!(path.exists());
        let ids: Vec<&str> = units.iter().map(|u| u.id.as_str()).collect();
        assert_eq!(ids, ["theseus", "romulus", "comparatio_thesei_et_romuli"]);
        assert_eq!(units[0].start_page, 1);
        assert_eq!(units[1].start_page, 3);
        assert_eq!(units[1].title_top_frac, Some(0.39));
        assert_eq!(units[2].body_top_frac, Some(0.827));
        assert_eq!(units::load_units(&path).unwrap(), units);
    }

    #[test]
    fn load_or_detect_preserves_pinned() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("facsimile_units.json");
        let pages = |source: usize, page: u32| -> Result<String> {
            if source != 0 {
                return Ok(String::new());
            }
            Ok(match page {
                1..=2 => "THESEUS. 1".to_string(),
                3..=4 => "ROMULUS. 1".to_string(),
                _ => String::new(),
            })
        };
        let pdfs = [stub_pdf("v1", 4), stub_pdf("v2", 0)];
        // First detection writes the file; then a human pins Theseus with a corrected
        // start_page and a skip_top_bands tweak.
        let mut units = load_or_detect(&path, &pdfs, false, pages, |_, _| Ok(vec![])).unwrap();
        units[0].start_page = 2;
        units[0].skip_top_bands = Some(3);
        units[0].title_top_frac = Some(0.3);
        units[0].body_top_frac = Some(0.35);
        units[0].pinned = true;
        std::fs::write(&path, serde_json::to_string_pretty(&units).unwrap()).unwrap();
        // A redetect that would set Theseus back to page 1 must keep the pin.
        let redetected = load_or_detect(&path, &pdfs, true, pages, |_, _| Ok(vec![])).unwrap();
        let theseus = redetected.iter().find(|u| u.id == "theseus").unwrap();
        assert_eq!(theseus.start_page, 2);
        assert_eq!(theseus.skip_top_bands, Some(3));
        assert_eq!(theseus.title_top_frac, Some(0.3));
        assert_eq!(theseus.body_top_frac, Some(0.35));
        assert!(theseus.pinned);
        // The unpinned Romulus row is still freshly detected.
        let romulus = redetected.iter().find(|u| u.id == "romulus").unwrap();
        assert_eq!(romulus.start_page, 3);
        assert!(!romulus.pinned);
    }

    fn stub_pdf(stem: &str, page_count: u32) -> Pdf {
        Pdf {
            path: std::path::PathBuf::from(format!("{stem}.pdf")),
            page_count,
        }
    }
}

#[cfg(test)]
mod full_source {
    use super::*;
    use crate::render::{page_count, page_text, pdftotext_available};
    use std::path::Path;

    /// Both sources present and `pdftotext` callable, else the real-PDF tests skip.
    fn sources() -> Option<[std::path::PathBuf; 2]> {
        if !pdftotext_available() {
            return None;
        }
        let v1 = Path::new("Ploutarchou_Bioi_v1.pdf");
        let v2 = Path::new("Ploutarchou_Bioi_v2.pdf");
        (v1.exists() && v2.exists()).then(|| [v1.to_path_buf(), v2.to_path_buf()])
    }

    fn detect_source(pdf: &Path) -> Vec<Run> {
        let last = page_count(pdf).unwrap();
        detect_openings(
            1..=last,
            |p| page_text(pdf, p),
            |p| crate::render::page_words(pdf, p),
        )
        .unwrap()
    }

    #[test]
    fn full_detection_covers_corpus() {
        let Some(paths) = sources() else {
            eprintln!("skipping: sources or pdftotext not available");
            return;
        };
        let runs_v1 = detect_source(&paths[0]);
        let runs_v2 = detect_source(&paths[1]);
        let units = units_from_runs(&runs_v1, &runs_v2).unwrap();
        // Every corpus unit appears exactly once, in corpus order.
        let detected: Vec<&str> = units.iter().map(|u| u.id.as_str()).collect();
        let expected: Vec<&str> = corpus::corpus().iter().map(|u| u.id).collect();
        assert_eq!(detected, expected);
    }

    #[test]
    fn last_body_from_runs_excludes_backmatter() {
        let Some(paths) = sources() else {
            eprintln!("skipping: sources or pdftotext not available");
            return;
        };
        let runs_v1 = detect_source(&paths[0]);
        let runs_v2 = detect_source(&paths[1]);
        let units = units_from_runs(&runs_v1, &runs_v2).unwrap();
        for (source, pdf) in paths.iter().enumerate() {
            let last = units
                .iter()
                .rfind(|u| u.source == source)
                .unwrap()
                .last_page
                .unwrap();
            assert!(
                last < page_count(pdf).unwrap(),
                "source {source} last body page {last} not below the page count"
            );
        }
    }
}
