//! The Message Batches API backend: submit a set of requests as one asynchronous
//! batch, poll it to completion, and read results keyed by `custom_id`. Submitting
//! and polling are split so the pipeline can persist a batch id between them and
//! re-attach after a crash instead of resubmitting and re-paying. Requests are
//! chunked to stay under the API's per-batch size and count caps.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};

use crate::llm::instrument::progress_bar;
use crate::llm::{build_body, extract_text, extract_usage, Client, Completion, ContentBlock};
use crate::tags::{BatchState, Sidecar};

/// The Message Batches endpoint base.
const BATCHES_URL: &str = "https://api.anthropic.com/v1/messages/batches";

/// Safety margin below the API's 256 MB per-batch cap: a chunk's cumulative
/// encoded size stays under this.
pub const BATCH_MAX_BYTES: usize = 200 * 1024 * 1024;

/// Safety margin below the API's 100,000-request per-batch cap.
pub const BATCH_MAX_REQUESTS: usize = 90_000;

/// Initial poll interval; backs off up to [`POLL_MAX`] between polls.
const POLL_INITIAL: Duration = Duration::from_secs(10);
/// Longest interval between polls.
const POLL_MAX: Duration = Duration::from_secs(60);

/// One request in a batch: everything needed to build a Messages-API body plus the
/// `custom_id` that keys its result. Opus escalation requests set `effort` and
/// leave `thinking` false (effort subsumes thinking); Sonnet/Haiku requests set
/// `effort: None`, `thinking: false`.
#[derive(Debug, Clone)]
pub struct BatchRequest {
    pub custom_id: String,
    pub model: String,
    pub system: Option<String>,
    pub content: Vec<ContentBlock>,
    pub max_tokens: u32,
    pub schema: Option<Value>,
    pub effort: Option<&'static str>,
    pub thinking: bool,
}

/// The outcome of one request in a finished batch.
#[derive(Debug, Clone)]
pub enum BatchOutcome {
    Ok(Completion),
    Err(String),
}

/// Whether a submitted batch is still processing or has ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchStatus {
    Processing,
    Ended,
}

/// The submit/poll/results split that lets the pipeline persist a batch id between
/// submission and completion so an interrupted run re-attaches.
pub trait BatchBackend {
    /// Submit a chunk of requests as one batch, returning its id.
    fn submit(&self, requests: &[BatchRequest]) -> Result<String>;
    /// Whether the batch has ended.
    fn poll(&self, batch_id: &str) -> Result<BatchStatus>;
    /// Collect a finished batch's results, keyed by `custom_id`.
    fn results(&self, batch_id: &str) -> Result<HashMap<String, BatchOutcome>>;
}

/// The production [`BatchBackend`] over the Anthropic Message Batches endpoints,
/// reusing the shared [`Client`] for transport and [`build_body`] for each
/// request's params.
pub struct ApiBatchBackend {
    client: Client,
}

impl ApiBatchBackend {
    pub fn new(client: Client) -> Self {
        ApiBatchBackend { client }
    }

    /// GET the batch object as JSON.
    fn fetch_batch(&self, batch_id: &str) -> Result<Value> {
        let url = format!("{BATCHES_URL}/{batch_id}");
        let response = self
            .client
            .http()
            .get(&url)
            .header("x-api-key", self.client.api_key())
            .header("anthropic-version", Client::anthropic_version())
            .send()
            .context("fetching batch")?;
        let status = response.status();
        let text = response.text().context("reading batch response")?;
        if !status.is_success() {
            bail!("batch fetch failed: {status} — {text}");
        }
        serde_json::from_str(&text).context("parsing batch response")
    }
}

impl BatchBackend for ApiBatchBackend {
    fn submit(&self, requests: &[BatchRequest]) -> Result<String> {
        let body = build_batch_body(requests);
        let response = self
            .client
            .http()
            .post(BATCHES_URL)
            .header("x-api-key", self.client.api_key())
            .header("anthropic-version", Client::anthropic_version())
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .context("submitting batch")?;
        let status = response.status();
        let text = response.text().context("reading batch submit response")?;
        if !status.is_success() {
            bail!("batch submit failed: {status} — {text}");
        }
        let value: Value = serde_json::from_str(&text).context("parsing batch submit response")?;
        value
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| anyhow!("batch submit response has no id"))
    }

    fn poll(&self, batch_id: &str) -> Result<BatchStatus> {
        let value = self.fetch_batch(batch_id)?;
        match value.get("processing_status").and_then(Value::as_str) {
            Some("ended") => Ok(BatchStatus::Ended),
            _ => Ok(BatchStatus::Processing),
        }
    }

    fn results(&self, batch_id: &str) -> Result<HashMap<String, BatchOutcome>> {
        let batch = self.fetch_batch(batch_id)?;
        let results_url = batch
            .get("results_url")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("batch {batch_id} has no results_url"))?;
        let response = self
            .client
            .http()
            .get(results_url)
            .header("x-api-key", self.client.api_key())
            .header("anthropic-version", Client::anthropic_version())
            .send()
            .context("fetching batch results")?;
        let status = response.status();
        let body = response.text().context("reading batch results")?;
        if !status.is_success() {
            bail!("batch results fetch failed: {status} — {body}");
        }
        Ok(parse_batch_results(&body))
    }
}

/// Parse a `custom_id` of the form `{role}_{src}_{page}` back into its parts,
/// where `role` is `s` (Sonnet), `h` (Haiku), or `o` (Opus), `src` the source
/// index, and `page` the 1-based source page.
pub fn parse_custom_id(id: &str) -> Option<(char, usize, u32)> {
    let mut parts = id.split('_');
    let role = parts.next()?.chars().next()?;
    let source: usize = parts.next()?.parse().ok()?;
    let page: u32 = parts.next()?.parse().ok()?;
    Some((role, source, page))
}

/// Build the JSON body for `POST /v1/messages/batches` from a set of requests,
/// wrapping each request's Messages-API body as its `params` under its `custom_id`.
pub fn build_batch_body(requests: &[BatchRequest]) -> Value {
    let reqs: Vec<Value> = requests
        .iter()
        .map(|r| {
            json!({
                "custom_id": r.custom_id,
                "params": build_body(
                    &r.model,
                    r.system.as_deref(),
                    &r.content,
                    r.max_tokens,
                    r.schema.clone(),
                    r.effort,
                    r.thinking,
                ),
            })
        })
        .collect();
    json!({ "requests": reqs })
}

/// Parse the newline-delimited JSON results stream into outcomes keyed by
/// `custom_id`. A `succeeded` result's message is read with the shared
/// [`extract_text`] / [`extract_usage`]; any other result type
/// (`errored | canceled | expired`) becomes a [`BatchOutcome::Err`].
pub fn parse_batch_results(text: &str) -> HashMap<String, BatchOutcome> {
    let mut out = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(custom_id) = value.get("custom_id").and_then(Value::as_str) else {
            continue;
        };
        let result = value.get("result");
        let rtype = result
            .and_then(|r| r.get("type"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let outcome = if rtype == "succeeded" {
            match result.and_then(|r| r.get("message")) {
                Some(message) => match extract_text(message) {
                    Ok(text) => BatchOutcome::Ok(Completion {
                        text,
                        usage: extract_usage(message),
                    }),
                    Err(err) => BatchOutcome::Err(format!("no text in succeeded result: {err}")),
                },
                None => BatchOutcome::Err("succeeded result has no message".to_string()),
            }
        } else {
            BatchOutcome::Err(format!("request result type {rtype}"))
        };
        out.insert(custom_id.to_string(), outcome);
    }
    out
}

/// Estimate one request's encoded size: the base64 image plus prompt text and the
/// small fixed fields. The image dominates, so this tracks the per-batch byte
/// budget closely enough to chunk safely below the 256 MB cap.
fn estimated_size(req: &BatchRequest) -> usize {
    let mut n = req.custom_id.len() + req.model.len() + req.system.as_ref().map_or(0, String::len);
    for block in &req.content {
        n += match block {
            ContentBlock::Text(t) => t.len(),
            ContentBlock::Image { png_base64 } => png_base64.len(),
        };
    }
    n
}

/// Split requests into chunks whose cumulative estimated size stays under
/// `max_bytes` and whose count stays under `max_requests`. A single request larger
/// than `max_bytes` forms its own chunk rather than being dropped.
fn chunk_requests(
    requests: Vec<BatchRequest>,
    max_bytes: usize,
    max_requests: usize,
) -> Vec<Vec<BatchRequest>> {
    let mut chunks = Vec::new();
    let mut current: Vec<BatchRequest> = Vec::new();
    let mut current_bytes = 0usize;
    for req in requests {
        let size = estimated_size(&req);
        let would_exceed = !current.is_empty()
            && (current_bytes + size > max_bytes || current.len() >= max_requests);
        if would_exceed {
            chunks.push(std::mem::take(&mut current));
            current_bytes = 0;
        }
        current_bytes += size;
        current.push(req);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// Submit `requests` to `backend` in size-bounded chunks, persisting each chunk's
/// batch id in `sidecar.batches` so an interrupted run re-attaches rather than
/// resubmitting, and return every result keyed by `custom_id`. `stage` labels the
/// batches (`"transcribe"`, `"escalate"`) so distinct stages' chunk ids never
/// collide.
pub fn run_chunked(
    backend: &dyn BatchBackend,
    requests: Vec<BatchRequest>,
    stage: &str,
    sidecar: &mut Sidecar,
    sidecar_path: &Path,
    label: &str,
) -> Result<HashMap<String, BatchOutcome>> {
    run_chunked_with(
        backend,
        requests,
        stage,
        sidecar,
        sidecar_path,
        label,
        BATCH_MAX_BYTES,
        BATCH_MAX_REQUESTS,
    )
}

/// [`run_chunked`] with explicit chunk limits, so tests can exercise the byte
/// budget without supplying hundreds of megabytes.
#[allow(clippy::too_many_arguments)]
fn run_chunked_with(
    backend: &dyn BatchBackend,
    requests: Vec<BatchRequest>,
    stage: &str,
    sidecar: &mut Sidecar,
    sidecar_path: &Path,
    label: &str,
    max_bytes: usize,
    max_requests: usize,
) -> Result<HashMap<String, BatchOutcome>> {
    let chunks = chunk_requests(requests, max_bytes, max_requests);
    let mut all = HashMap::new();
    let pb = progress_bar(chunks.len() as u64, label);
    for (i, chunk) in chunks.iter().enumerate() {
        if chunk.is_empty() {
            pb.inc(1);
            continue;
        }
        // Resume a persisted batch for this stage/chunk if present, else submit and
        // persist its id before waiting so a crash re-attaches.
        let batch_id = match sidecar
            .batches
            .iter()
            .find(|b| b.stage == stage && b.chunk == i)
        {
            Some(state) => state.batch_id.clone(),
            None => {
                let id = backend.submit(chunk)?;
                sidecar.batches.push(BatchState {
                    stage: stage.to_string(),
                    chunk: i,
                    batch_id: id.clone(),
                });
                sidecar.save(sidecar_path)?;
                id
            }
        };

        let mut wait = POLL_INITIAL;
        while backend.poll(&batch_id)? == BatchStatus::Processing {
            std::thread::sleep(wait);
            wait = (wait * 2).min(POLL_MAX);
        }

        all.extend(backend.results(&batch_id)?);
        sidecar
            .batches
            .retain(|b| !(b.stage == stage && b.chunk == i));
        sidecar.save(sidecar_path)?;
        pb.inc(1);
    }
    pb.finish_and_clear();
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::instrument::Usage;
    use std::cell::RefCell;

    fn text_request(id: &str, text: &str) -> BatchRequest {
        BatchRequest {
            custom_id: id.to_string(),
            model: "claude-sonnet-5".to_string(),
            system: None,
            content: vec![ContentBlock::Text(text.to_string())],
            max_tokens: 16,
            schema: None,
            effort: None,
            thinking: false,
        }
    }

    fn ok_outcome() -> BatchOutcome {
        BatchOutcome::Ok(Completion {
            text: "{}".to_string(),
            usage: Usage::default(),
        })
    }

    /// A backend that records submissions and serves canned successes. `submit`
    /// assigns a fresh id and remembers the chunk's custom_ids; `results` returns a
    /// success per custom_id of the batch that id refers to.
    #[derive(Default)]
    struct StubBackend {
        chunks: RefCell<HashMap<String, Vec<String>>>,
        submit_count: RefCell<usize>,
    }

    impl StubBackend {
        fn register(&self, batch_id: &str, ids: Vec<String>) {
            self.chunks.borrow_mut().insert(batch_id.to_string(), ids);
        }
    }

    impl BatchBackend for StubBackend {
        fn submit(&self, requests: &[BatchRequest]) -> Result<String> {
            let mut count = self.submit_count.borrow_mut();
            let id = format!("batch{}", *count);
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
            Ok(ids.into_iter().map(|id| (id, ok_outcome())).collect())
        }
    }

    fn empty_sidecar() -> Sidecar {
        Sidecar {
            schema_version: crate::tags::SCHEMA_VERSION,
            sources: Vec::new(),
            front_matter: Default::default(),
            pages: Vec::new(),
            lives: Vec::new(),
            batches: Vec::new(),
        }
    }

    #[test]
    fn build_batch_body_wraps_requests_with_custom_ids() {
        let reqs = vec![text_request("s_0_40", "a"), text_request("h_0_40", "b")];
        let body = build_batch_body(&reqs);
        let arr = body["requests"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["custom_id"], "s_0_40");
        assert_eq!(arr[0]["params"]["model"], "claude-sonnet-5");
        assert_eq!(arr[1]["custom_id"], "h_0_40");
    }

    #[test]
    fn parse_batch_results_keys_by_custom_id() {
        let stream = concat!(
            r#"{"custom_id":"s_0_40","result":{"type":"succeeded","message":{"content":[{"type":"text","text":"hello"}],"usage":{"input_tokens":5,"output_tokens":2}}}}"#,
            "\n",
            r#"{"custom_id":"s_0_41","result":{"type":"succeeded","message":{"content":[{"type":"text","text":"world"}],"usage":{"input_tokens":7,"output_tokens":3}}}}"#,
        );
        let out = parse_batch_results(stream);
        match out.get("s_0_40").unwrap() {
            BatchOutcome::Ok(c) => {
                assert_eq!(c.text, "hello");
                assert_eq!(c.usage.input_tokens, 5);
            }
            BatchOutcome::Err(e) => panic!("expected ok: {e}"),
        }
        assert!(matches!(out.get("s_0_41"), Some(BatchOutcome::Ok(_))));
    }

    #[test]
    fn parse_batch_results_marks_errored_and_expired() {
        let stream = concat!(
            r#"{"custom_id":"e1","result":{"type":"errored","error":{"type":"invalid_request_error"}}}"#,
            "\n",
            r#"{"custom_id":"e2","result":{"type":"expired"}}"#,
        );
        let out = parse_batch_results(stream);
        assert!(matches!(out.get("e1"), Some(BatchOutcome::Err(_))));
        assert!(matches!(out.get("e2"), Some(BatchOutcome::Err(_))));
    }

    #[test]
    fn run_chunked_splits_on_byte_budget() {
        // Four ~10-byte requests with a 25-byte budget split into chunks of at most
        // two, so the backend sees multiple submissions and every result returns.
        let reqs = vec![
            text_request("a", "0123456789"),
            text_request("b", "0123456789"),
            text_request("c", "0123456789"),
            text_request("d", "0123456789"),
        ];
        let backend = StubBackend::default();
        let mut sidecar = empty_sidecar();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dryrun.tags.json");
        let out = run_chunked_with(
            &backend,
            reqs,
            "transcribe",
            &mut sidecar,
            &path,
            "batching",
            25,
            90_000,
        )
        .unwrap();
        assert_eq!(out.len(), 4);
        assert!(
            *backend.submit_count.borrow() > 1,
            "should split into chunks"
        );
        assert!(sidecar.batches.is_empty(), "state cleared after collection");
    }

    #[test]
    fn run_chunked_resumes_from_persisted_state() {
        let reqs = vec![text_request("a", "x"), text_request("b", "y")];
        let backend = StubBackend::default();
        // Pre-seed the persisted batch id and register its outcomes, as if a prior
        // run had submitted chunk 0 and crashed before collecting.
        backend.register("preexisting", vec!["a".to_string(), "b".to_string()]);
        let mut sidecar = empty_sidecar();
        sidecar.batches.push(BatchState {
            stage: "transcribe".to_string(),
            chunk: 0,
            batch_id: "preexisting".to_string(),
        });
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dryrun.tags.json");
        let out = run_chunked(
            &backend,
            reqs,
            "transcribe",
            &mut sidecar,
            &path,
            "batching",
        )
        .unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(*backend.submit_count.borrow(), 0, "must not resubmit");
        assert!(sidecar.batches.is_empty());
    }

    /// Live smoke test: a one-request batch end to end. Needs a real `secrets.json`;
    /// ignored by default. Run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn live_one_request_batch() {
        let secrets = crate::secrets::load(Path::new("secrets.json")).unwrap();
        let backend = ApiBatchBackend::new(Client::new(secrets.anthropic_api_key));
        let reqs = vec![BatchRequest {
            custom_id: "probe".to_string(),
            model: crate::llm::HAIKU.to_string(),
            system: None,
            content: vec![ContentBlock::Text(
                "Reply with the single word: ok".to_string(),
            )],
            max_tokens: 16,
            schema: None,
            effort: None,
            thinking: false,
        }];
        let mut sidecar = empty_sidecar();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dryrun.tags.json");
        let out = run_chunked(&backend, reqs, "probe", &mut sidecar, &path, "probing").unwrap();
        match out.get("probe").unwrap() {
            BatchOutcome::Ok(c) => println!("reply: {} usage: {:?}", c.text, c.usage),
            BatchOutcome::Err(e) => panic!("probe errored: {e}"),
        }
    }
}
