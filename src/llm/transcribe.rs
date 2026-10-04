//! Per-page structured transcription: the faithfulness-first prompts, the forced
//! `PageDoc` output schema, the reply parser, and the single-page API call.

use std::path::Path;

use anyhow::{Context, Result};
use serde_json::{json, Value};

use crate::llm::batch::{self, BatchBackend, BatchOutcome, BatchRequest};
use crate::llm::instrument::UsageTracker;
use crate::llm::{ContentBlock, HAIKU, SONNET};
use crate::render::hash_bytes;
use crate::tags::{PageDoc, PageRole, Sidecar};

/// Default for the Haiku divergence signal (idea F): on, Sonnet and Haiku both
/// transcribe each page and their disagreement drives escalation. The owner flips
/// this to `false` at the cost-validation gate if the run would exceed budget;
/// it is threaded as an explicit parameter so tests exercise both states.
pub const DIVERGENCE_SIGNAL: bool = false;

/// What a page is transcribed as.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// A body page: transcribe only the right/Latin column, using the Greek as a
    /// disambiguation aid.
    BodyLatinOnly,
    /// A front-matter prose page (Monitum, Praefatio, title): transcribe the whole
    /// page faithfully, Greek fragments included.
    FrontProseFull,
}

/// Output-token ceiling for a page transcription. A dense full page of Latin set as
/// structured JSON can be large, so this is well above a typical page.
const MAX_TOKENS: u32 = 16000;

const SYSTEM_PROMPT: &str = "You are a meticulous transcriber of a printed \
nineteenth-century edition of Plutarch's Lives (Didot, Paris 1857). You reproduce \
the printed text exactly as it appears, never modernizing or correcting it. You \
return only the requested JSON object.";

/// The user prompt for a mode. Exposed via [`body_user_prompt`] /
/// [`front_user_prompt`] so tests can assert its content.
pub fn user_prompt(mode: Mode) -> String {
    match mode {
        Mode::BodyLatinOnly => BODY_PROMPT.to_string(),
        Mode::FrontProseFull => FRONT_PROMPT.to_string(),
    }
}

const BODY_PROMPT: &str = "This page has two columns: Greek on the left and Latin \
on the right. Transcribe ONLY the Latin (right) column, exactly as printed. Do \
NOT normalize or correct anything: preserve the original spelling, u/v and i/j \
usage, ligatures, the long s, accents, and punctuation exactly as they appear. \
Use the Greek column only to disambiguate an unclear Latin word; never translate \
or transcribe the Greek. Reproduce the inline Roman chapter numbers (e.g. XI.) as \
Chapter blocks and the parenthetical section numbers (e.g. (2)) as Section blocks. \
Include the roman field only on a Chapter block and the section_no field only on a \
Section block; include continues_previous only when it is true. Mark any embedded \
verse quotation (line-broken poetry) as a Verse block, preserving its line breaks \
exactly with newline characters. Ignore the running head, the printed page number, \
and the marginal Greek line numbers.";

const FRONT_PROMPT: &str = "Transcribe this entire page faithfully, exactly as \
printed, INCLUDING any Greek fragments (put the Greek in the block text). Do NOT \
normalize, modernize, or correct spelling, orthography, or punctuation. Preserve \
the order of the text and any numbering. Use Heading blocks for display headings \
and Paragraph blocks for running prose; include continues_previous only when a \
paragraph continues from the previous page. EXCLUDE page furniture that is not part \
of the editorial prose: running heads, printed page numbers and folios, printer's \
signatures and catchwords (for example a line at the foot of the page reading \
'PLUTARCHUS. I' or a bare signature mark), and engraver's or printer's imprints. \
When the page shows only the tail of the previous page's section and carries NO \
heading of its own, do not invent one: continue the running prose with \
continues_previous set on the first paragraph. A heading the page genuinely prints \
is prose and must be kept; drop only a heading the page does not actually show.";

/// The JSON schema the reply is forced to, mirroring `PageDoc`.
pub fn output_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "blocks": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "kind": {
                            "type": "string",
                            "enum": ["heading", "chapter", "section", "paragraph", "verse"]
                        },
                        "roman": { "type": ["string", "null"] },
                        "section_no": { "type": ["integer", "null"] },
                        "continues_previous": { "type": "boolean" },
                        "text": { "type": "string" }
                    },
                    "required": ["kind", "text"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["blocks"],
        "additionalProperties": false
    })
}

/// Parse a transcription reply into a `PageDoc`, tolerating a reply envelope: a
/// Markdown code fence (```` ```json … ``` ````) or commentary around the JSON. The
/// first balanced top-level `{ … }` is extracted and parsed; a reply with no object
/// at all is parsed verbatim so genuinely malformed input still errors. This only
/// relaxes the *envelope*, never the schema or the accepted text.
pub fn parse_pagedoc(reply: &str) -> Result<PageDoc> {
    let json = extract_json_object(reply).unwrap_or(reply);
    let doc: PageDoc =
        serde_json::from_str(json).context("parsing transcription reply as a PageDoc")?;
    Ok(doc)
}

/// The first balanced top-level JSON object `{ … }` in `reply`, or `None` if there is
/// none. String contents (including escaped quotes) are skipped so a brace inside a
/// string literal does not throw off the depth count.
fn extract_json_object(reply: &str) -> Option<&str> {
    let start = reply.find('{')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (i, &b) in reply.as_bytes().iter().enumerate().skip(start) {
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&reply[start..=i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// The transcription mode for a page role: body pages take the Latin-only prompt,
/// front-matter prose the full-page prompt.
fn mode_for(role: PageRole) -> Option<Mode> {
    match role {
        PageRole::Body => Some(Mode::BodyLatinOnly),
        PageRole::FrontProse => Some(Mode::FrontProseFull),
        PageRole::Skip | PageRole::FrontImage => None,
    }
}

/// Submit, through the Batch API, a Sonnet pass (authoritative) for every in-scope
/// `Body`/`FrontProse` page whose `pass_a` is absent and, when `divergence_signal`,
/// a Haiku pass (signal only) for every such page whose `pass_b` is absent. Gating
/// on the missing pass — not on a missing `result` — means a re-run re-batches only
/// the passes actually absent, so an errored/expired request is retried next run.
/// `Body` pages render through `render_body` (the cropped Latin column);
/// `FrontProse` pages through `render_full` (the whole page). `--dry-run N` caps
/// body pages via `limit`; `in_scope` restricts a `--pages` range. A failed request
/// leaves its pass unset and is counted and warned, never fabricated.
#[allow(clippy::too_many_arguments)]
pub fn transcribe_all(
    backend: &dyn BatchBackend,
    sidecar: &mut Sidecar,
    tracker: &UsageTracker,
    sidecar_path: &Path,
    limit: Option<u32>,
    in_scope: impl Fn(usize, u32) -> bool,
    divergence_signal: bool,
    render_body: impl Fn(usize, u32) -> Result<Vec<u8>>,
    render_full: impl Fn(usize, u32) -> Result<Vec<u8>>,
) -> Result<()> {
    // Decide which pages and which passes still need a request.
    let mut targets: Vec<(usize, u32, Mode, bool, bool)> = Vec::new();
    let mut body_seen = 0u32;
    for record in &sidecar.pages {
        let Some(mode) = mode_for(record.role) else {
            continue;
        };
        if !in_scope(record.source, record.source_page) {
            continue;
        }
        if mode == Mode::BodyLatinOnly {
            if let Some(n) = limit {
                if body_seen >= n {
                    continue;
                }
                body_seen += 1;
            }
        }
        let need_sonnet = record.pass_a.is_none();
        let need_haiku = divergence_signal && record.pass_b.is_none();
        if need_sonnet || need_haiku {
            targets.push((
                record.source,
                record.source_page,
                mode,
                need_sonnet,
                need_haiku,
            ));
        }
    }

    // Render each target once (body cropped, front full), set its image hash, and
    // build the Sonnet/Haiku requests that share that one image.
    let mut requests: Vec<BatchRequest> = Vec::new();
    for &(source, page, mode, need_sonnet, need_haiku) in &targets {
        let bytes = match mode {
            Mode::BodyLatinOnly => render_body(source, page)?,
            Mode::FrontProseFull => render_full(source, page)?,
        };
        let base64 = crate::b64(&bytes);
        let sha = hash_bytes(&bytes);
        if let Some(record) = sidecar.page_mut(source, page) {
            record.image_sha256 = sha;
        }
        let content = vec![
            ContentBlock::Text(user_prompt(mode)),
            ContentBlock::Image { png_base64: base64 },
        ];
        if need_sonnet {
            requests.push(transcribe_request(
                format!("s_{source}_{page}"),
                SONNET,
                &content,
            ));
        }
        if need_haiku {
            requests.push(transcribe_request(
                format!("h_{source}_{page}"),
                HAIKU,
                &content,
            ));
        }
    }

    if requests.is_empty() {
        return Ok(());
    }

    let outcomes = batch::run_chunked(
        backend,
        requests,
        "transcribe",
        sidecar,
        sidecar_path,
        "transcribing",
    )?;

    let mut failures = 0usize;
    for (custom_id, outcome) in &outcomes {
        let Some((role, source, page)) = batch::parse_custom_id(custom_id) else {
            continue;
        };
        let model = match role {
            's' => SONNET,
            'h' => HAIKU,
            _ => continue,
        };
        match outcome {
            BatchOutcome::Ok(completion) => match parse_pagedoc(&completion.text) {
                Ok(doc) => {
                    tracker.record(model, (source, page), completion.usage);
                    if let Some(record) = sidecar.page_mut(source, page) {
                        if role == 's' {
                            record.pass_a = Some(doc);
                            record.result_model = None;
                        } else {
                            record.pass_b = Some(doc);
                        }
                    }
                }
                Err(err) => {
                    failures += 1;
                    eprintln!("  warning: {custom_id} returned unparseable transcription: {err}");
                }
            },
            BatchOutcome::Err(reason) => {
                failures += 1;
                eprintln!("  warning: {custom_id} transcription request failed: {reason}");
            }
        }
    }
    if failures > 0 {
        eprintln!(
            "  {failures} transcription request(s) failed; affected passes left unset for a later run"
        );
    }
    sidecar.save(sidecar_path)?;
    Ok(())
}

/// Build one transcription batch request: the faithfulness system prompt, the
/// page image, the compact output schema, and no thinking/effort (Sonnet and
/// Haiku transcribe at base settings).
fn transcribe_request(custom_id: String, model: &str, content: &[ContentBlock]) -> BatchRequest {
    BatchRequest {
        custom_id,
        model: model.to_string(),
        system: Some(SYSTEM_PROMPT.to_string()),
        content: content.to_vec(),
        max_tokens: MAX_TOKENS,
        schema: Some(output_schema()),
        effort: None,
        thinking: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::instrument::Usage;
    use crate::tags::BlockKind;

    #[test]
    fn parse_pagedoc_reads_blocks_and_numbering() {
        let reply = r#"{"blocks":[
            {"kind":"chapter","roman":"XI.","section_no":null,"continues_previous":false,"text":""},
            {"kind":"section","roman":null,"section_no":2,"continues_previous":false,"text":""},
            {"kind":"paragraph","roman":null,"section_no":null,"continues_previous":true,"text":"Romulus igitur"}
        ]}"#;
        let doc = parse_pagedoc(reply).unwrap();
        assert_eq!(doc.blocks.len(), 3);
        assert_eq!(doc.blocks[0].kind, BlockKind::Chapter);
        assert_eq!(doc.blocks[0].roman.as_deref(), Some("XI."));
        assert_eq!(doc.blocks[1].kind, BlockKind::Section);
        assert_eq!(doc.blocks[1].section_no, Some(2));
        assert!(doc.blocks[2].continues_previous);
        assert_eq!(doc.blocks[2].text, "Romulus igitur");
    }

    #[test]
    fn parse_pagedoc_rejects_malformed() {
        assert!(parse_pagedoc("not json").is_err());
        assert!(parse_pagedoc(r#"{"blocks":[{"kind":"paragraph"}]}"#).is_err());
    }

    #[test]
    fn parse_pagedoc_strips_code_fence() {
        let reply =
            "```json\n{\"blocks\":[{\"kind\":\"paragraph\",\"text\":\"Romulus igitur\"}]}\n```";
        let doc = parse_pagedoc(reply).unwrap();
        assert_eq!(doc.blocks.len(), 1);
        assert_eq!(doc.blocks[0].text, "Romulus igitur");
    }

    #[test]
    fn parse_pagedoc_extracts_object_amid_prose() {
        let reply = "Here is the faithful transcription:\n\
            {\"blocks\":[{\"kind\":\"paragraph\",\"text\":\"a { brace } in text\"}]}\n\
            Let me know if you need anything else.";
        let doc = parse_pagedoc(reply).unwrap();
        assert_eq!(doc.blocks.len(), 1);
        // A brace inside the string literal must not truncate the extracted object.
        assert_eq!(doc.blocks[0].text, "a { brace } in text");
    }

    #[test]
    fn output_schema_requires_only_kind_and_text() {
        let schema = output_schema();
        let required = schema["properties"]["blocks"]["items"]["required"]
            .as_array()
            .unwrap();
        let names: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
        assert_eq!(names, vec!["kind", "text"]);
        // The optional fields remain declared as nullable properties.
        let props = &schema["properties"]["blocks"]["items"]["properties"];
        assert!(props.get("roman").is_some());
        assert!(props.get("section_no").is_some());
        assert!(props.get("continues_previous").is_some());
    }

    #[test]
    fn parse_pagedoc_fills_defaults_for_omitted_fields() {
        // A compact reply omitting roman/section_no/continues_previous parses, with
        // serde defaults filling the absent fields.
        let reply = r#"{"blocks":[{"kind":"paragraph","text":"Romulus igitur"}]}"#;
        let doc = parse_pagedoc(reply).unwrap();
        assert_eq!(doc.blocks[0].roman, None);
        assert_eq!(doc.blocks[0].section_no, None);
        assert!(!doc.blocks[0].continues_previous);
        assert_eq!(doc.blocks[0].text, "Romulus igitur");
    }

    #[test]
    fn parse_pagedoc_reads_verse_line_breaks() {
        let reply =
            "{\"blocks\":[{\"kind\":\"verse\",\"roman\":null,\"section_no\":null,\"continues_previous\":false,\"text\":\"line one\\nline two\\nline three\"}]}";
        let doc = parse_pagedoc(reply).unwrap();
        assert_eq!(doc.blocks[0].kind, BlockKind::Verse);
        assert_eq!(doc.blocks[0].text, "line one\nline two\nline three");
    }

    #[test]
    fn body_prompt_mentions_latin_only_and_no_normalize() {
        let p = user_prompt(Mode::BodyLatinOnly);
        assert!(p.contains("ONLY the Latin"));
        assert!(p.contains("Do NOT normalize"));
        assert!(p.contains("u/v"));
        assert!(p.contains("long s"));
        // The compact schema is driven by instructing the model to include the
        // optional fields only when they apply.
        assert!(p.contains("only on a Chapter block"));
        assert!(p.contains("continues_previous only when"));
    }

    #[test]
    fn body_prompt_requests_verse_marking() {
        let p = user_prompt(Mode::BodyLatinOnly);
        assert!(p.contains("Verse block"));
        assert!(p.contains("line breaks"));
    }

    #[test]
    fn front_prompt_requests_greek_included() {
        let p = user_prompt(Mode::FrontProseFull);
        assert!(p.contains("Greek"));
        assert!(p.contains("faithfully"));
    }

    #[test]
    fn front_prompt_drops_furniture_and_continues_across_pages() {
        let p = user_prompt(Mode::FrontProseFull);
        // Printer furniture (signatures/catchwords, running heads, folios) is excluded.
        assert!(p.contains("PLUTARCHUS. I"));
        assert!(p.contains("signatures and catchwords"));
        assert!(p.contains("running heads"));
        // A continuation page with no heading of its own joins across the break; a
        // heading the source prints is kept.
        assert!(p.contains("do not invent one"));
        assert!(p.contains("must be kept"));
    }

    use crate::llm::batch::{BatchBackend, BatchOutcome, BatchRequest, BatchStatus};
    use crate::llm::Completion;
    use crate::tags::{BatchState, PageRecord, PageRole, PageStatus, SourcePdf};
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// A batch backend that records submissions and serves canned transcriptions.
    /// Each succeeded result is a one-block `PageDoc` whose text is the request's
    /// role char (`s`/`h`), so a test can tell which model produced a stored pass.
    /// Custom ids listed in `fail` come back as errors.
    #[derive(Default)]
    struct StubBatch {
        chunks: RefCell<HashMap<String, Vec<String>>>,
        submit_calls: RefCell<usize>,
        fail: Vec<String>,
    }

    impl StubBatch {
        fn failing(fail: &[&str]) -> Self {
            StubBatch {
                fail: fail.iter().map(|s| s.to_string()).collect(),
                ..Default::default()
            }
        }

        fn register(&self, batch_id: &str, ids: Vec<String>) {
            self.chunks.borrow_mut().insert(batch_id.to_string(), ids);
        }

        /// Every custom id across batches this stub itself submitted (ids keyed
        /// `b{n}`), ignoring any pre-registered resume batches.
        fn submitted_ids(&self) -> Vec<String> {
            let mut ids = Vec::new();
            for (key, values) in self.chunks.borrow().iter() {
                if key.starts_with('b') {
                    ids.extend(values.iter().cloned());
                }
            }
            ids
        }
    }

    impl BatchBackend for StubBatch {
        fn submit(&self, requests: &[BatchRequest]) -> Result<String> {
            let mut count = self.submit_calls.borrow_mut();
            let id = format!("b{}", *count);
            *count += 1;
            let ids = requests.iter().map(|r| r.custom_id.clone()).collect();
            self.chunks.borrow_mut().insert(id.clone(), ids);
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
                let outcome = if self.fail.contains(&id) {
                    BatchOutcome::Err("errored".to_string())
                } else {
                    let role = id.chars().next().unwrap();
                    let text = format!(r#"{{"blocks":[{{"kind":"paragraph","text":"{role}"}}]}}"#);
                    BatchOutcome::Ok(Completion {
                        text,
                        usage: Usage {
                            input_tokens: 10,
                            output_tokens: 1,
                        },
                    })
                };
                out.insert(id, outcome);
            }
            Ok(out)
        }
    }

    fn record(source: usize, page: u32, role: PageRole) -> PageRecord {
        PageRecord {
            source,
            source_page: page,
            image_sha256: String::new(),
            role,
            pass_a: None,
            pass_b: None,
            status: PageStatus::NotBody,
            flags: Vec::new(),
            result: None,
            result_model: None,
        }
    }

    fn fixture(pages: Vec<PageRecord>) -> Sidecar {
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

    /// Drive `transcribe_all` through `backend` with trivial render closures.
    fn run_stub(
        sidecar: &mut Sidecar,
        tracker: &UsageTracker,
        backend: &StubBatch,
        divergence_signal: bool,
        limit: Option<u32>,
        in_scope: impl Fn(usize, u32) -> bool,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dryrun.tags.json");
        transcribe_all(
            backend,
            sidecar,
            tracker,
            &path,
            limit,
            in_scope,
            divergence_signal,
            |_, _| Ok(vec![1, 2, 3]),
            |_, _| Ok(vec![4, 5, 6]),
        )
        .unwrap();
    }

    #[test]
    fn transcribe_all_submits_sonnet_and_haiku_per_page() {
        let mut sidecar = fixture(vec![record(0, 20, PageRole::Body)]);
        let tracker = UsageTracker::new();
        let backend = StubBatch::default();
        run_stub(&mut sidecar, &tracker, &backend, true, None, |_, _| true);
        let ids = backend.submitted_ids();
        assert!(ids.contains(&"s_0_20".to_string()));
        assert!(ids.contains(&"h_0_20".to_string()));
        let p = sidecar.page(0, 20).unwrap();
        assert_eq!(p.pass_a.as_ref().unwrap().blocks[0].text, "s");
        assert_eq!(p.pass_b.as_ref().unwrap().blocks[0].text, "h");
        // result_model stays unset here; it is decided during reconciliation.
        assert_eq!(p.result_model, None);
    }

    #[test]
    fn transcribe_all_omits_haiku_when_signal_off() {
        let mut sidecar = fixture(vec![record(0, 20, PageRole::Body)]);
        let tracker = UsageTracker::new();
        let backend = StubBatch::default();
        run_stub(&mut sidecar, &tracker, &backend, false, None, |_, _| true);
        let ids = backend.submitted_ids();
        assert!(ids.contains(&"s_0_20".to_string()));
        assert!(!ids.iter().any(|i| i.starts_with('h')));
        assert!(sidecar.page(0, 20).unwrap().pass_b.is_none());
    }

    #[test]
    fn frontprose_uses_full_page_body_uses_crop() {
        let mut sidecar = fixture(vec![
            record(0, 12, PageRole::FrontProse),
            record(0, 20, PageRole::Body),
        ]);
        let tracker = UsageTracker::new();
        let backend = StubBatch::default();
        let body_calls = RefCell::new(Vec::new());
        let full_calls = RefCell::new(Vec::new());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dryrun.tags.json");
        transcribe_all(
            &backend,
            &mut sidecar,
            &tracker,
            &path,
            None,
            |_, _| true,
            true,
            |s, p| {
                body_calls.borrow_mut().push((s, p));
                Ok(vec![1, 2, 3])
            },
            |s, p| {
                full_calls.borrow_mut().push((s, p));
                Ok(vec![4, 5, 6])
            },
        )
        .unwrap();
        assert_eq!(*body_calls.borrow(), vec![(0, 20)]);
        assert_eq!(*full_calls.borrow(), vec![(0, 12)]);
    }

    #[test]
    fn transcribe_all_records_usage_per_model() {
        let mut sidecar = fixture(vec![record(0, 30, PageRole::Body)]);
        let tracker = UsageTracker::new();
        let backend = StubBatch::default();
        run_stub(&mut sidecar, &tracker, &backend, true, None, |_, _| true);
        let report = tracker.report();
        let sonnet = report.models.iter().find(|m| m.model == SONNET).unwrap();
        let haiku = report.models.iter().find(|m| m.model == HAIKU).unwrap();
        assert_eq!(sonnet.calls, 1);
        assert_eq!(haiku.calls, 1);
    }

    #[test]
    fn transcribe_all_skips_pages_with_passes() {
        let mut sidecar = fixture(vec![record(0, 20, PageRole::Body)]);
        sidecar.pages[0].pass_a = Some(PageDoc { blocks: Vec::new() });
        sidecar.pages[0].pass_b = Some(PageDoc { blocks: Vec::new() });
        let tracker = UsageTracker::new();
        let backend = StubBatch::default();
        run_stub(&mut sidecar, &tracker, &backend, true, None, |_, _| true);
        assert_eq!(*backend.submit_calls.borrow(), 0);
    }

    #[test]
    fn transcribe_all_warns_and_leaves_pass_unset_on_errored_request() {
        let mut sidecar = fixture(vec![record(0, 20, PageRole::Body)]);
        let tracker = UsageTracker::new();
        let backend = StubBatch::failing(&["s_0_20"]);
        run_stub(&mut sidecar, &tracker, &backend, true, None, |_, _| true);
        let p = sidecar.page(0, 20).unwrap();
        // The errored Sonnet pass stays absent; the succeeded Haiku pass is stored.
        assert!(p.pass_a.is_none());
        assert!(p.pass_b.is_some());
    }

    #[test]
    fn dry_run_limits_body_pages() {
        let mut sidecar = fixture(vec![
            record(0, 20, PageRole::Body),
            record(0, 21, PageRole::Body),
            record(0, 22, PageRole::Body),
        ]);
        let tracker = UsageTracker::new();
        let backend = StubBatch::default();
        run_stub(&mut sidecar, &tracker, &backend, true, Some(1), |_, _| true);
        assert!(sidecar.page(0, 20).unwrap().pass_a.is_some());
        assert!(sidecar.page(0, 21).unwrap().pass_a.is_none());
        assert!(sidecar.page(0, 22).unwrap().pass_a.is_none());
    }

    #[test]
    fn transcribe_all_resumes_from_persisted_batch_id() {
        let mut sidecar = fixture(vec![record(0, 20, PageRole::Body)]);
        let backend = StubBatch::default();
        backend.register("pre", vec!["s_0_20".to_string(), "h_0_20".to_string()]);
        sidecar.batches.push(BatchState {
            stage: "transcribe".to_string(),
            chunk: 0,
            batch_id: "pre".to_string(),
        });
        let tracker = UsageTracker::new();
        run_stub(&mut sidecar, &tracker, &backend, true, None, |_, _| true);
        assert_eq!(*backend.submit_calls.borrow(), 0, "must not resubmit");
        assert!(sidecar.page(0, 20).unwrap().pass_a.is_some());
    }

    #[test]
    fn restricted_scope_skips_out_of_range_pages() {
        let mut sidecar = fixture(vec![
            record(0, 40, PageRole::Body),
            record(0, 80, PageRole::Body),
        ]);
        let tracker = UsageTracker::new();
        let backend = StubBatch::default();
        run_stub(&mut sidecar, &tracker, &backend, true, None, |_, page| {
            (40..=60).contains(&page)
        });
        assert!(sidecar.page(0, 40).unwrap().pass_a.is_some());
        assert!(sidecar.page(0, 80).unwrap().pass_a.is_none());
    }
}
