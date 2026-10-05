//! Free local verification — section-number continuity, orthography-folded
//! Sonnet/Haiku similarity, and a coarse Tesseract-`lat` page-integrity check — and
//! the reconcile step that accepts an unflagged page's Sonnet pass as-is or
//! escalates a flagged page to Opus (batched, at reduced effort) and finalizes each
//! page's authoritative transcription.

use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::llm::arbitrate;
use crate::llm::batch::{self, BatchBackend, BatchOutcome, BatchRequest};
use crate::llm::instrument::{PageId, UsageTracker};
use crate::llm::transcribe::{output_schema, parse_pagedoc, user_prompt, Mode};
use crate::llm::{ContentBlock, OPUS, SONNET};
use crate::tags::{
    BlockKind, FlagKind, PageDoc, PageRecord, PageRole, PageStatus, Sidecar, VerifyFlag,
};

// ---- Deterministic checks -------------------------------------------------

/// Running numbering state carried across page boundaries.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Continuity {
    /// The last chapter numeral seen, as an integer.
    pub last_chapter: Option<u32>,
    /// The last parenthetical section number seen in the current chapter.
    pub last_section: Option<u32>,
}

/// Flag section-number gaps and chapter-number resets across the boundary into
/// `page`. A reset is allowed when the page opens a new Life/Comparatio (it
/// carries a `Heading` block) or when a new `Chapter` block legitimately restarts
/// the section count.
pub fn section_continuity(prev: &Continuity, page: &PageDoc) -> Vec<VerifyFlag> {
    let life_start = page.blocks.iter().any(|b| b.kind == BlockKind::Heading);
    let mut flags = Vec::new();
    let mut last_chapter = prev.last_chapter;
    let mut last_section = prev.last_section;

    for block in &page.blocks {
        match block.kind {
            BlockKind::Chapter => {
                if let Some(n) = block.roman.as_deref().and_then(roman_to_u32) {
                    if let Some(prev_ch) = last_chapter {
                        if n < prev_ch && !life_start {
                            flags.push(VerifyFlag {
                                kind: FlagKind::SectionContinuity,
                                detail: format!("chapter number went {prev_ch} -> {n}"),
                            });
                        }
                    }
                    last_chapter = Some(n);
                    last_section = None;
                }
            }
            BlockKind::Section => {
                if let Some(s) = block.section_no {
                    if let Some(prev_s) = last_section {
                        if s != prev_s + 1 {
                            flags.push(VerifyFlag {
                                kind: FlagKind::SectionContinuity,
                                detail: format!("section number went {prev_s} -> {s}"),
                            });
                        }
                    }
                    last_section = Some(s);
                }
            }
            _ => {}
        }
    }
    flags
}

/// Advance the running numbering state through `page`.
pub fn advance(prev: &Continuity, page: &PageDoc) -> Continuity {
    let mut state = prev.clone();
    for block in &page.blocks {
        match block.kind {
            BlockKind::Chapter => {
                if let Some(n) = block.roman.as_deref().and_then(roman_to_u32) {
                    state.last_chapter = Some(n);
                    state.last_section = None;
                }
            }
            BlockKind::Section => {
                if let Some(s) = block.section_no {
                    state.last_section = Some(s);
                }
            }
            _ => {}
        }
    }
    state
}

/// Parse a Roman numeral (as printed, with an optional trailing `.`) to an
/// integer. Returns `None` for anything that is not a plain Roman numeral.
pub fn roman_to_u32(s: &str) -> Option<u32> {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_alphabetic())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    let value = |c: char| match c {
        'I' => Some(1),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        'D' => Some(500),
        'M' => Some(1000),
        _ => None,
    };
    let digits: Option<Vec<u32>> = cleaned.chars().map(value).collect();
    let digits = digits?;
    let mut total = 0i64;
    for i in 0..digits.len() {
        let cur = digits[i] as i64;
        let next = digits.get(i + 1).copied().unwrap_or(0) as i64;
        if cur < next {
            total -= cur;
        } else {
            total += cur;
        }
    }
    u32::try_from(total).ok()
}

/// The `tesseract` binary, overridable with `PLUTARCH_TESSERACT`.
fn tesseract_bin() -> String {
    std::env::var("PLUTARCH_TESSERACT").unwrap_or_else(|_| "tesseract".to_string())
}

/// OCR a body page's PNG with Tesseract-`lat` and compare its char count to the
/// Sonnet pass, flagging a coarse page-integrity divergence. Front-matter prose is
/// not checked here — the Latin OCR model cannot read its Greek.
pub fn ocr_crosscheck(png: &std::path::Path, page: &PageDoc) -> Result<Vec<VerifyFlag>> {
    let output = Command::new(tesseract_bin())
        .arg(png)
        .arg("stdout")
        .arg("-l")
        .arg("lat")
        .output()
        .with_context(|| format!("running tesseract on {}", png.display()))?;
    if !output.status.success() {
        bail!(
            "tesseract failed on {}: {}",
            png.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let ocr = String::from_utf8_lossy(&output.stdout);
    Ok(ocr_divergence_flags(&ocr, page))
}

/// Lower / upper bounds for the coarse OCR page-integrity check, as fractions of
/// the Sonnet pass's non-space char count. This is a secondary signal — it only
/// catches a dropped or badly garbled page, not fine wording — so the band is
/// wide; the Sonnet↔Haiku similarity is the primary cross-check.
const OCR_LOW_FRACTION: f64 = 0.5;
const OCR_HIGH_FRACTION: f64 = 1.6;

/// Coarsely compare OCR output to the Sonnet pass by non-space char count, flagging
/// only when the OCR count falls far below or far above the pass. Pure, so it is
/// tested without invoking Tesseract.
pub fn ocr_divergence_flags(ocr_text: &str, page: &PageDoc) -> Vec<VerifyFlag> {
    let doc_text: String = page
        .blocks
        .iter()
        .map(|b| b.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let doc_chars = doc_text.chars().filter(|c| !c.is_whitespace()).count();
    let ocr_chars = ocr_text.chars().filter(|c| !c.is_whitespace()).count();

    let mut flags = Vec::new();
    if doc_chars == 0 {
        return flags;
    }
    let low = (doc_chars as f64) * OCR_LOW_FRACTION;
    let high = (doc_chars as f64) * OCR_HIGH_FRACTION;
    if (ocr_chars as f64) < low || (ocr_chars as f64) > high {
        flags.push(VerifyFlag {
            kind: FlagKind::OcrDivergence,
            detail: format!("OCR {ocr_chars} vs Sonnet {doc_chars} non-space chars"),
        });
    }
    flags
}

// ---- Orthography folding and similarity -----------------------------------

/// Fold a string so spelling noise drops out before comparison: lowercase, map
/// u→v and i→j, the long s to s, æ→ae, œ→oe, and strip everything that is not a
/// letter (whitespace, punctuation, digits). Only real wording differences survive.
pub fn fold(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        for c in ch.to_lowercase() {
            match c {
                'ſ' => out.push('s'),
                'æ' => out.push_str("ae"),
                'œ' => out.push_str("oe"),
                'u' => out.push('v'),
                'i' => out.push('j'),
                c if c.is_alphabetic() => out.push(c),
                _ => {}
            }
        }
    }
    out
}

/// A page's block text as a sequence of folded, non-empty word tokens.
fn folded_tokens(doc: &PageDoc) -> Vec<String> {
    let mut joined = String::new();
    for block in &doc.blocks {
        joined.push_str(&block.text);
        joined.push(' ');
    }
    joined
        .split_whitespace()
        .map(fold)
        .filter(|t| !t.is_empty())
        .collect()
}

/// Below this folded-token similarity a page's two passes count as a substantive
/// disagreement and the page is escalated to Opus.
pub const SIMILARITY_THRESHOLD: f32 = 0.90;

/// Token-level similarity of two pages' folded block text, in `[0, 1]` where `1.0`
/// is identical: `1 − token_levenshtein / max_token_len`. A page is flagged when
/// this falls below [`SIMILARITY_THRESHOLD`], so spelling-only differences (folded
/// away) do not escalate but real wording differences do.
pub fn similarity(a: &PageDoc, b: &PageDoc) -> f32 {
    let ta = folded_tokens(a);
    let tb = folded_tokens(b);
    let max_len = ta.len().max(tb.len());
    if max_len == 0 {
        return 1.0;
    }
    let dist = token_levenshtein(&ta, &tb);
    1.0 - dist as f32 / max_len as f32
}

/// Levenshtein edit distance over two token sequences.
fn token_levenshtein(a: &[String], b: &[String]) -> usize {
    let (n, m) = (a.len(), b.len());
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut cur = vec![0usize; m + 1];
    for i in 1..=n {
        cur[0] = i;
        for j in 1..=m {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[m]
}

/// Compute a page's verification flags from its **Sonnet** pass (`pass_a`): a
/// numbering-continuity break against `continuity`, the coarse OCR integrity check
/// (Body pages only, when `ocr` is `Some`), and — only when `divergence_signal` and
/// a Haiku `pass_b` exists — a Sonnet↔Haiku similarity below
/// [`SIMILARITY_THRESHOLD`]. Taking the signal as a parameter keeps both states
/// testable.
pub fn page_flags(
    continuity: &Continuity,
    record: &PageRecord,
    ocr: Option<&str>,
    divergence_signal: bool,
) -> Vec<VerifyFlag> {
    let mut flags = Vec::new();
    let Some(pass_a) = &record.pass_a else {
        return flags;
    };
    flags.extend(section_continuity(continuity, pass_a));
    if record.role == PageRole::Body {
        if let Some(text) = ocr {
            flags.extend(ocr_divergence_flags(text, pass_a));
        }
    }
    if divergence_signal {
        if let Some(pass_b) = &record.pass_b {
            let sim = similarity(pass_a, pass_b);
            if sim < SIMILARITY_THRESHOLD {
                flags.push(VerifyFlag {
                    kind: FlagKind::ModelDivergence,
                    detail: format!("Sonnet/Haiku similarity {sim:.2} below threshold"),
                });
            }
        }
    }
    flags
}

// ---- Reconcile ------------------------------------------------------------

/// Thinking depth for the Opus escalation. On Opus 4.8 `effort` governs thinking,
/// so escalation sets `effort` and sends no separate adaptive-`thinking` block.
pub const ARBITRATION_EFFORT: &str = "medium";

/// Total characters of a document's block text, for the Opus-length guard.
fn doc_chars(doc: &PageDoc) -> usize {
    doc.blocks.iter().map(|b| b.text.chars().count()).sum()
}

/// The transcription mode for a page role (Body = cropped Latin column, FrontProse
/// = full page), used to pick the render closure and the escalation prompt.
fn mode_for(role: PageRole) -> Mode {
    match role {
        PageRole::FrontProse => Mode::FrontProseFull,
        _ => Mode::BodyLatinOnly,
    }
}

/// Build one Opus escalation request: the authoritative-arbiter system prompt, a
/// user prompt presenting the candidate pass(es), the page image, the compact
/// schema, and `effort` (no separate thinking block).
fn escalation_request(
    custom_id: String,
    pass_a: &PageDoc,
    pass_b: &PageDoc,
    png_base64: String,
    mode: Mode,
) -> Result<BatchRequest> {
    // Body escalations present both candidates for the arbiter to resolve against
    // the image; front prose re-transcribes with the full-page prompt as context.
    let text = match mode {
        Mode::FrontProseFull => user_prompt(mode),
        Mode::BodyLatinOnly => arbitrate::user_prompt(pass_a, pass_b)?,
    };
    Ok(BatchRequest {
        custom_id,
        model: OPUS.to_string(),
        system: Some(arbitrate::SYSTEM_PROMPT.to_string()),
        content: vec![ContentBlock::Text(text), ContentBlock::Image { png_base64 }],
        max_tokens: arbitrate::MAX_TOKENS,
        schema: Some(output_schema()),
        effort: Some(ARBITRATION_EFFORT),
        thinking: false,
    })
}

/// Reconcile every transcribed page. A free first pass advances numbering
/// continuity off each page's Sonnet pass (`pass_a`) and stores its
/// [`page_flags`]; an unflagged page is accepted as-is (`status = Agreed`,
/// `result =` the Sonnet pass). The flagged pages are collected into one Opus
/// chunked batch that re-decides each from its own image (Body = `render_body`
/// cropped column, FrontProse = `render_full`) at reduced effort; on success
/// `result =` the Opus pass (`status = Escalated`), guarded so an implausibly short
/// Opus reply keeps the Sonnet pass. A failed Opus request also keeps the Sonnet
/// pass and is counted and warned, so no page is dropped. Targets require only
/// `pass_a` (not `pass_b`), so the signal-off path and a page whose Haiku request
/// errored still reconcile.
#[allow(clippy::too_many_arguments)]
pub fn reconcile_all(
    backend: &dyn BatchBackend,
    sidecar: &mut Sidecar,
    tracker: &UsageTracker,
    sidecar_path: &Path,
    divergence_signal: bool,
    render_body: impl Fn(usize, u32) -> Result<Vec<u8>>,
    render_full: impl Fn(usize, u32) -> Result<Vec<u8>>,
    ocr: impl Fn(usize, u32) -> Result<Option<String>>,
) -> Result<()> {
    reconcile_all_in_scope(
        backend,
        sidecar,
        tracker,
        sidecar_path,
        divergence_signal,
        render_body,
        render_full,
        ocr,
        |_, _| true,
    )
}

/// Reconcile unfinished transcriptions within the caller's page scope.
#[allow(clippy::too_many_arguments)]
pub fn reconcile_all_in_scope(
    backend: &dyn BatchBackend,
    sidecar: &mut Sidecar,
    tracker: &UsageTracker,
    sidecar_path: &Path,
    divergence_signal: bool,
    render_body: impl Fn(usize, u32) -> Result<Vec<u8>>,
    render_full: impl Fn(usize, u32) -> Result<Vec<u8>>,
    ocr: impl Fn(usize, u32) -> Result<Option<String>>,
    in_scope: impl Fn(usize, u32) -> bool,
) -> Result<()> {
    let targets: Vec<(usize, u32)> = sidecar
        .pages
        .iter()
        .filter(|p| {
            matches!(p.role, PageRole::Body | PageRole::FrontProse)
                && in_scope(p.source, p.source_page)
                && p.pass_a.is_some()
                && p.result.is_none()
        })
        .map(|p| (p.source, p.source_page))
        .collect();

    // Free first pass: compute and store flags off the Sonnet pass, accepting every
    // unflagged page immediately and collecting the flagged ones for Opus.
    let mut continuity = Continuity::default();
    let mut flagged: Vec<(usize, u32)> = Vec::new();
    for &(source, page) in &targets {
        let role = sidecar.page(source, page).expect("target exists").role;
        let ocr_text = if role == PageRole::Body {
            ocr(source, page)?
        } else {
            None
        };
        let record = sidecar.page(source, page).expect("target exists");
        let flags = page_flags(&continuity, record, ocr_text.as_deref(), divergence_signal);
        if let Some(pass_a) = &record.pass_a {
            continuity = advance(&continuity, pass_a);
        }
        let is_flagged = !flags.is_empty();
        let record = sidecar.page_mut(source, page).expect("target exists");
        record.flags = flags;
        if is_flagged {
            flagged.push((source, page));
        } else {
            record.result = record.pass_a.clone();
            record.result_model = Some(SONNET.to_string());
            record.status = PageStatus::Agreed;
        }
    }
    sidecar.save(sidecar_path)?;

    if flagged.is_empty() {
        return Ok(());
    }

    // Render each flagged page and build its Opus escalation request.
    let mut requests: Vec<BatchRequest> = Vec::new();
    let empty = PageDoc { blocks: Vec::new() };
    for &(source, page) in &flagged {
        let record = sidecar.page(source, page).expect("flagged exists");
        let role = record.role;
        let mode = mode_for(role);
        let pass_a = record.pass_a.clone().expect("flagged page has Sonnet pass");
        let pass_b = record.pass_b.clone().unwrap_or_else(|| empty.clone());
        let bytes = match mode {
            Mode::BodyLatinOnly => render_body(source, page)?,
            Mode::FrontProseFull => render_full(source, page)?,
        };
        let base64 = crate::b64(&bytes);
        requests.push(escalation_request(
            format!("o_{source}_{page}"),
            &pass_a,
            &pass_b,
            base64,
            mode,
        )?);
    }

    let outcomes = batch::run_chunked(
        backend,
        requests,
        "escalate",
        sidecar,
        sidecar_path,
        "escalating",
    )?;

    let mut failures = 0usize;
    for &(source, page) in &flagged {
        let custom_id = format!("o_{source}_{page}");
        let pass_a = sidecar
            .page(source, page)
            .and_then(|p| p.pass_a.clone())
            .expect("flagged page has Sonnet pass");
        let record = sidecar.page_mut(source, page).expect("flagged exists");
        record.status = PageStatus::Escalated;
        match outcomes.get(&custom_id) {
            Some(BatchOutcome::Ok(completion)) => match parse_pagedoc(&completion.text) {
                Ok(doc) => {
                    tracker.record(OPUS, (source, page), completion.usage);
                    // Guard a starved/truncated Opus reply: if far shorter than the
                    // Sonnet pass, keep Sonnet rather than shipping a blank page.
                    if doc_chars(&doc) * 2 < doc_chars(&pass_a) {
                        record.flags.push(VerifyFlag {
                            kind: FlagKind::ArbiterFallback,
                            detail: format!(
                                "Opus reply {} chars vs Sonnet {}; kept Sonnet",
                                doc_chars(&doc),
                                doc_chars(&pass_a),
                            ),
                        });
                        record.result = Some(pass_a);
                        record.result_model = Some(SONNET.to_string());
                    } else {
                        record.result = Some(doc);
                        record.result_model = Some(OPUS.to_string());
                    }
                }
                Err(err) => {
                    failures += 1;
                    record.flags.push(VerifyFlag {
                        kind: FlagKind::ArbiterFallback,
                        detail: format!("unparseable Opus reply ({err}); kept Sonnet"),
                    });
                    record.result = Some(pass_a);
                    record.result_model = Some(SONNET.to_string());
                }
            },
            Some(BatchOutcome::Err(reason)) => {
                failures += 1;
                record.flags.push(VerifyFlag {
                    kind: FlagKind::ArbiterFallback,
                    detail: format!("Opus request failed ({reason}); kept Sonnet"),
                });
                record.result = Some(pass_a);
                record.result_model = Some(SONNET.to_string());
            }
            None => {
                failures += 1;
                record.flags.push(VerifyFlag {
                    kind: FlagKind::ArbiterFallback,
                    detail: "Opus result missing; kept Sonnet".to_string(),
                });
                record.result = Some(pass_a);
                record.result_model = Some(SONNET.to_string());
            }
        }
    }
    if failures > 0 {
        eprintln!("  {failures} Opus escalation(s) failed; kept the Sonnet pass on those pages");
    }
    sidecar.save(sidecar_path)?;
    Ok(())
}

/// Counts of page statuses after reconciliation, with page references, for the
/// run log. Pages are sorted into accepted (Sonnet) and escalated (Opus); the
/// legacy `Arbitrated` status counts as escalated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileSummary {
    pub agreed: Vec<PageId>,
    pub escalated: Vec<PageId>,
}

impl std::fmt::Display for ReconcileSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "reconcile: {} accepted (Sonnet), {} escalated (Opus)",
            self.agreed.len(),
            self.escalated.len()
        )
    }
}

/// Summarize the sidecar's page statuses.
pub fn summary(sidecar: &Sidecar) -> ReconcileSummary {
    let mut summary = ReconcileSummary {
        agreed: Vec::new(),
        escalated: Vec::new(),
    };
    for p in &sidecar.pages {
        let id = (p.source, p.source_page);
        match p.status {
            PageStatus::Agreed => summary.agreed.push(id),
            PageStatus::Arbitrated | PageStatus::Escalated => summary.escalated.push(id),
            PageStatus::NotBody => {}
        }
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::instrument::Usage;
    use crate::tags::Block;

    fn para(text: &str) -> Block {
        Block {
            kind: BlockKind::Paragraph,
            roman: None,
            section_no: None,
            continues_previous: false,
            text: text.to_string(),
        }
    }

    fn section(n: u32) -> Block {
        Block {
            kind: BlockKind::Section,
            roman: None,
            section_no: Some(n),
            continues_previous: false,
            text: String::new(),
        }
    }

    fn heading(text: &str) -> Block {
        Block {
            kind: BlockKind::Heading,
            roman: None,
            section_no: None,
            continues_previous: false,
            text: text.to_string(),
        }
    }

    fn doc(blocks: Vec<Block>) -> PageDoc {
        PageDoc { blocks }
    }

    #[test]
    fn roman_parsing() {
        assert_eq!(roman_to_u32("XI."), Some(11));
        assert_eq!(roman_to_u32("IX"), Some(9));
        assert_eq!(roman_to_u32("XLII."), Some(42));
        assert_eq!(roman_to_u32("(2)"), None);
    }

    #[test]
    fn continuity_flags_missing_section() {
        let prev = Continuity {
            last_chapter: Some(11),
            last_section: Some(2),
        };
        let page = doc(vec![section(4), para("…")]);
        let flags = section_continuity(&prev, &page);
        assert_eq!(flags.len(), 1);
        assert_eq!(flags[0].kind, FlagKind::SectionContinuity);
    }

    #[test]
    fn continuity_allows_life_reset() {
        let prev = Continuity {
            last_chapter: Some(30),
            last_section: Some(7),
        };
        // A new Life opens with a heading and chapter I, section counting restarts.
        let page = doc(vec![
            heading("THESEUS"),
            Block {
                kind: BlockKind::Chapter,
                roman: Some("I.".to_string()),
                section_no: None,
                continues_previous: false,
                text: String::new(),
            },
            section(2),
            para("…"),
        ]);
        let flags = section_continuity(&prev, &page);
        assert!(flags.is_empty(), "life reset should not flag: {flags:?}");
    }

    #[test]
    fn ocr_crosscheck_flags_large_divergence() {
        let page = doc(vec![para(&"verbum ".repeat(100))]);
        // OCR found almost nothing — a coarse page-integrity divergence.
        let flags = ocr_divergence_flags("x", &page);
        assert_eq!(flags.len(), 1);
        assert_eq!(flags[0].kind, FlagKind::OcrDivergence);
        // A close match does not flag.
        let close = "verbum ".repeat(95);
        assert!(ocr_divergence_flags(&close, &page).is_empty());
    }

    #[test]
    fn fold_maps_orthography() {
        assert_eq!(fold("Uirtus"), fold("Virtus"));
        assert_eq!(fold("ſublata"), fold("sublata"));
        assert_eq!(fold("iam"), fold("jam"));
        assert_eq!(fold("Cæsar"), fold("Caesar"));
        // Whitespace and punctuation drop out entirely.
        assert_eq!(fold("a, b. c!"), "abc");
    }

    #[test]
    fn similarity_ignores_orthography_but_catches_wording() {
        let a = doc(vec![para("sublata est virtus")]);
        let b = doc(vec![para("ſublata eſt uirtus")]);
        assert!(similarity(&a, &b) >= SIMILARITY_THRESHOLD);
        let c = doc(vec![para("omega psi chi")]);
        assert!(similarity(&a, &c) < SIMILARITY_THRESHOLD);
    }

    // ---- reconcile_all over a stubbed batch backend ----

    use crate::llm::batch::{BatchBackend, BatchOutcome, BatchRequest, BatchStatus};
    use crate::llm::Completion;
    use crate::tags::SourcePdf;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Clone, Copy)]
    enum OpusReply {
        Long,
        Short,
        Error,
    }

    /// A batch backend standing in for Opus escalation. It records the requests it
    /// was given (so a test can assert `effort`/`thinking`) and replies per its
    /// configured mode.
    struct StubOpus {
        chunks: RefCell<HashMap<String, Vec<String>>>,
        submit_calls: RefCell<usize>,
        submitted: RefCell<Vec<BatchRequest>>,
        reply: OpusReply,
    }

    impl StubOpus {
        fn new(reply: OpusReply) -> Self {
            StubOpus {
                chunks: RefCell::new(HashMap::new()),
                submit_calls: RefCell::new(0),
                submitted: RefCell::new(Vec::new()),
                reply,
            }
        }
    }

    impl BatchBackend for StubOpus {
        fn submit(&self, requests: &[BatchRequest]) -> Result<String> {
            *self.submit_calls.borrow_mut() += 1;
            self.submitted.borrow_mut().extend(requests.iter().cloned());
            let id = format!("b{}", self.submit_calls.borrow());
            self.chunks.borrow_mut().insert(
                id.clone(),
                requests.iter().map(|r| r.custom_id.clone()).collect(),
            );
            Ok(id)
        }

        fn poll(&self, _batch_id: &str) -> Result<BatchStatus> {
            Ok(BatchStatus::Ended)
        }

        fn results(&self, batch_id: &str) -> Result<HashMap<String, BatchOutcome>> {
            let ids = self
                .chunks
                .borrow()
                .get(batch_id)
                .cloned()
                .unwrap_or_default();
            let mut out = HashMap::new();
            for id in ids {
                let outcome = match self.reply {
                    OpusReply::Long => BatchOutcome::Ok(Completion {
                        text: r#"{"blocks":[{"kind":"paragraph","text":"opus decided this faithful latin text"}]}"#.to_string(),
                        usage: Usage {
                            input_tokens: 200,
                            output_tokens: 40,
                        },
                    }),
                    OpusReply::Short => BatchOutcome::Ok(Completion {
                        text: r#"{"blocks":[{"kind":"paragraph","text":"x"}]}"#.to_string(),
                        usage: Usage::default(),
                    }),
                    OpusReply::Error => BatchOutcome::Err("errored".to_string()),
                };
                out.insert(id, outcome);
            }
            Ok(out)
        }
    }

    fn body_record(page: u32, pass_a: PageDoc, pass_b: Option<PageDoc>) -> PageRecord {
        PageRecord {
            source: 0,
            source_page: page,
            image_sha256: String::new(),
            role: PageRole::Body,
            pass_a: Some(pass_a),
            pass_b,
            status: PageStatus::NotBody,
            flags: Vec::new(),
            result: None,
            result_model: None,
        }
    }

    fn sidecar_with(pages: Vec<PageRecord>) -> Sidecar {
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

    fn run_reconcile(
        sidecar: &mut Sidecar,
        tracker: &UsageTracker,
        backend: &StubOpus,
        divergence_signal: bool,
        ocr: impl Fn(usize, u32) -> Result<Option<String>>,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dryrun.tags.json");
        reconcile_all(
            backend,
            sidecar,
            tracker,
            &path,
            divergence_signal,
            |_, _| Ok(vec![1, 2, 3]),
            |_, _| Ok(vec![4, 5, 6]),
            ocr,
        )
        .unwrap();
    }

    #[test]
    fn agreement_folds_orthography_and_skips_opus() {
        // Sonnet and Haiku differ only by long-s and u/v — folded away, so the page
        // is accepted as Sonnet with no Opus request.
        let pass_a = doc(vec![para("sublata est virtus")]);
        let pass_b = doc(vec![para("ſublata eſt uirtus")]);
        let mut sidecar = sidecar_with(vec![body_record(10, pass_a.clone(), Some(pass_b))]);
        let tracker = UsageTracker::new();
        let backend = StubOpus::new(OpusReply::Long);
        run_reconcile(&mut sidecar, &tracker, &backend, true, |_, _| Ok(None));
        assert_eq!(*backend.submit_calls.borrow(), 0);
        let p = sidecar.page(0, 10).unwrap();
        assert_eq!(p.status, PageStatus::Agreed);
        assert_eq!(p.result.as_ref().unwrap(), &pass_a);
        assert_eq!(p.result_model.as_deref(), Some(SONNET));
    }

    #[test]
    fn substantive_disagreement_escalates() {
        let pass_a = doc(vec![para("alpha beta gamma delta")]);
        let pass_b = doc(vec![para("omega psi chi phi")]);
        let mut sidecar = sidecar_with(vec![body_record(10, pass_a, Some(pass_b))]);
        let tracker = UsageTracker::new();
        let backend = StubOpus::new(OpusReply::Long);
        run_reconcile(&mut sidecar, &tracker, &backend, true, |_, _| Ok(None));
        assert_eq!(*backend.submit_calls.borrow(), 1);
        let p = sidecar.page(0, 10).unwrap();
        assert_eq!(p.status, PageStatus::Escalated);
        assert_eq!(p.result_model.as_deref(), Some(OPUS));
        assert!(p.result.as_ref().unwrap().blocks[0].text.contains("opus"));
    }

    #[test]
    fn numbering_break_escalates() {
        // A within-page section jump 2 -> 4; both passes identical so only the
        // numbering check fires.
        let pass = doc(vec![section(2), para("x"), section(4), para("y")]);
        let mut sidecar = sidecar_with(vec![body_record(10, pass.clone(), Some(pass))]);
        let tracker = UsageTracker::new();
        let backend = StubOpus::new(OpusReply::Long);
        run_reconcile(&mut sidecar, &tracker, &backend, true, |_, _| Ok(None));
        assert_eq!(*backend.submit_calls.borrow(), 1);
        assert_eq!(sidecar.page(0, 10).unwrap().status, PageStatus::Escalated);
    }

    #[test]
    fn coarse_ocr_flag_escalates() {
        let pass = doc(vec![para(&"verbum ".repeat(100))]);
        let mut sidecar = sidecar_with(vec![body_record(10, pass.clone(), Some(pass))]);
        let tracker = UsageTracker::new();
        let backend = StubOpus::new(OpusReply::Long);
        // OCR returns almost nothing — a coarse integrity divergence.
        run_reconcile(&mut sidecar, &tracker, &backend, true, |_, _| {
            Ok(Some("x".to_string()))
        });
        assert_eq!(*backend.submit_calls.borrow(), 1);
        assert_eq!(sidecar.page(0, 10).unwrap().status, PageStatus::Escalated);
    }

    #[test]
    fn escalation_sends_effort_medium_and_no_thinking() {
        let pass_a = doc(vec![para("alpha")]);
        let pass_b = doc(vec![para("beta")]);
        let mut sidecar = sidecar_with(vec![body_record(10, pass_a, Some(pass_b))]);
        let tracker = UsageTracker::new();
        let backend = StubOpus::new(OpusReply::Long);
        run_reconcile(&mut sidecar, &tracker, &backend, true, |_, _| Ok(None));
        let reqs = backend.submitted.borrow();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].model, OPUS);
        assert_eq!(reqs[0].effort, Some(ARBITRATION_EFFORT));
        assert!(!reqs[0].thinking);
    }

    #[test]
    fn short_opus_result_falls_back_to_sonnet() {
        let long_a = doc(vec![para(&"alpha ".repeat(60))]);
        let pass_b = doc(vec![para(&"beta ".repeat(60))]);
        let mut sidecar = sidecar_with(vec![body_record(10, long_a.clone(), Some(pass_b))]);
        let tracker = UsageTracker::new();
        let backend = StubOpus::new(OpusReply::Short);
        run_reconcile(&mut sidecar, &tracker, &backend, true, |_, _| Ok(None));
        let p = sidecar.page(0, 10).unwrap();
        assert_eq!(p.result.as_ref().unwrap(), &long_a);
        assert_eq!(p.result_model.as_deref(), Some(SONNET));
        assert!(p.flags.iter().any(|f| f.kind == FlagKind::ArbiterFallback));
    }

    #[test]
    fn errored_opus_request_keeps_sonnet() {
        let pass_a = doc(vec![para("alpha beta gamma")]);
        let pass_b = doc(vec![para("omega psi chi")]);
        let mut sidecar = sidecar_with(vec![body_record(10, pass_a.clone(), Some(pass_b))]);
        let tracker = UsageTracker::new();
        let backend = StubOpus::new(OpusReply::Error);
        run_reconcile(&mut sidecar, &tracker, &backend, true, |_, _| Ok(None));
        let p = sidecar.page(0, 10).unwrap();
        assert_eq!(p.result.as_ref().unwrap(), &pass_a);
        assert_eq!(p.result_model.as_deref(), Some(SONNET));
        assert!(p.flags.iter().any(|f| f.kind == FlagKind::ArbiterFallback));
    }

    #[test]
    fn signal_off_relies_on_numbering_and_ocr() {
        // With the signal off, a clean page carrying no Haiku pass skips Opus.
        let pass_a = doc(vec![para("some clean faithful latin text")]);
        let mut sidecar = sidecar_with(vec![body_record(10, pass_a.clone(), None)]);
        let tracker = UsageTracker::new();
        let backend = StubOpus::new(OpusReply::Long);
        run_reconcile(&mut sidecar, &tracker, &backend, false, |_, _| Ok(None));
        assert_eq!(*backend.submit_calls.borrow(), 0);
        let p = sidecar.page(0, 10).unwrap();
        assert_eq!(p.status, PageStatus::Agreed);
        assert_eq!(p.result_model.as_deref(), Some(SONNET));
    }

    #[test]
    fn summary_counts_statuses() {
        let mut sidecar = sidecar_with(Vec::new());
        for (i, status) in [
            PageStatus::Agreed,
            PageStatus::Arbitrated,
            PageStatus::Escalated,
            PageStatus::Agreed,
            PageStatus::NotBody,
        ]
        .into_iter()
        .enumerate()
        {
            sidecar.pages.push(PageRecord {
                source: 0,
                source_page: i as u32,
                image_sha256: String::new(),
                role: PageRole::Body,
                pass_a: None,
                pass_b: None,
                status,
                flags: Vec::new(),
                result: None,
                result_model: None,
            });
        }
        let s = summary(&sidecar);
        assert_eq!(s.agreed.len(), 2);
        // The legacy Arbitrated status counts as escalated alongside Escalated.
        assert_eq!(s.escalated.len(), 2);
    }
}
