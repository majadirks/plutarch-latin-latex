//! The Anthropic Messages-API client with image support. Page-role classification
//! (Haiku) runs synchronously behind the [`Vision`] trait so tests supply a
//! deterministic stub; transcription (Sonnet + optional Haiku) and Opus escalation
//! run through the [`batch`] backend.

use std::time::Duration;

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

pub mod arbitrate;
pub mod batch;
pub mod instrument;
pub mod transcribe;

use crate::tags::PageRole;
use instrument::Usage;

/// Cheap model: page-role classification (pass B of transcription).
pub const HAIKU: &str = "claude-haiku-4-5";
/// Mid model: pass A of transcription.
pub const SONNET: &str = "claude-sonnet-5";
/// Capable model: arbitration, with adaptive thinking.
pub const OPUS: &str = "claude-opus-4-8";

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// One block of a user-turn message: either text or a base64 PNG image.
#[derive(Debug, Clone)]
pub enum ContentBlock {
    Text(String),
    Image { png_base64: String },
}

/// A reply and the tokens it cost.
#[derive(Debug, Clone)]
pub struct Completion {
    pub text: String,
    pub usage: Usage,
}

/// A blocking client for the Anthropic Messages API. Cheap to clone (the HTTP
/// client is internally reference-counted), so one client can back both the
/// synchronous `ApiVision` and the `ApiBatchBackend`.
#[derive(Clone)]
pub struct Client {
    http: reqwest::blocking::Client,
    api_key: String,
}

/// Transport/connect timeout for establishing the connection.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
/// Overall per-request timeout. Generous because an Opus arbitration with adaptive
/// thinking on a dense page can run for minutes.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(600);
/// How many times a transient failure (timeout, dropped connection, 429/5xx/529)
/// is retried before giving up, so one network blip does not abort a long batch.
const MAX_ATTEMPTS: u32 = 5;

/// The outcome of one HTTP attempt: a retryable transient failure or a fatal one.
enum Attempt {
    Retryable(String),
    Fatal(anyhow::Error),
}

impl Client {
    /// Build a client from an API key, with connect/request timeouts so a stalled
    /// upload surfaces promptly instead of hanging indefinitely.
    pub fn new(api_key: String) -> Self {
        let http = reqwest::blocking::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .expect("building the HTTP client");
        Client { http, api_key }
    }

    /// The underlying HTTP client, for the batch backend's raw requests to the
    /// Message Batches endpoints.
    pub(crate) fn http(&self) -> &reqwest::blocking::Client {
        &self.http
    }

    /// The API key, for the batch backend's request headers.
    pub(crate) fn api_key(&self) -> &str {
        &self.api_key
    }

    /// The `anthropic-version` header value, shared with the batch backend.
    pub(crate) fn anthropic_version() -> &'static str {
        ANTHROPIC_VERSION
    }

    /// Send one message and return the reply text and its token usage.
    ///
    /// When `output_schema` is given, the reply is forced to a JSON object
    /// matching that schema. The returned text is the first content block whose
    /// type is `"text"`, so a leading `thinking` block (present when adaptive
    /// thinking is enabled) is skipped rather than mistaken for the answer.
    pub fn message(
        &self,
        model: &str,
        system: Option<&str>,
        content: &[ContentBlock],
        max_tokens: u32,
        output_schema: Option<Value>,
    ) -> Result<Completion> {
        self.send(
            model,
            system,
            content,
            max_tokens,
            output_schema,
            None,
            false,
        )
    }

    /// Like [`Client::message`] but with adaptive thinking enabled. The reply's
    /// leading `thinking` block is skipped by [`extract_text`].
    pub fn message_thinking(
        &self,
        model: &str,
        system: Option<&str>,
        content: &[ContentBlock],
        max_tokens: u32,
        output_schema: Option<Value>,
    ) -> Result<Completion> {
        self.send(
            model,
            system,
            content,
            max_tokens,
            output_schema,
            None,
            true,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn send(
        &self,
        model: &str,
        system: Option<&str>,
        content: &[ContentBlock],
        max_tokens: u32,
        output_schema: Option<Value>,
        effort: Option<&str>,
        thinking: bool,
    ) -> Result<Completion> {
        let body = build_body(
            model,
            system,
            content,
            max_tokens,
            output_schema,
            effort,
            thinking,
        );
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.try_send(&body) {
                Ok(completion) => return Ok(completion),
                Err(Attempt::Fatal(err)) => return Err(err),
                Err(Attempt::Retryable(reason)) => {
                    if attempt >= MAX_ATTEMPTS {
                        return Err(anyhow!(
                            "Anthropic request failed after {attempt} attempts: {reason}"
                        ));
                    }
                    // Exponential backoff: 2, 4, 8, 16 seconds.
                    let wait = Duration::from_secs(1u64 << attempt);
                    eprintln!(
                        "  Anthropic request failed ({reason}); retry {attempt}/{} in {}s",
                        MAX_ATTEMPTS - 1,
                        wait.as_secs()
                    );
                    std::thread::sleep(wait);
                }
            }
        }
    }

    /// One HTTP attempt. Transport errors (timeout, dropped connection) and
    /// server-side transient statuses (408/409/425/429/5xx/529) are reported as
    /// retryable; auth errors, refusals, and malformed replies are fatal.
    fn try_send(&self, body: &Value) -> std::result::Result<Completion, Attempt> {
        let response = match self
            .http
            .post(API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(body)
            .send()
        {
            Ok(response) => response,
            // A transport-level failure (timeout, connect, dropped connection) is
            // worth retrying on a long batch.
            Err(err) => return Err(Attempt::Retryable(format!("transport error: {err}"))),
        };

        let status = response.status();
        let request_id = response
            .headers()
            .get("request-id")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let text = match response.text() {
            Ok(text) => text,
            Err(err) => return Err(Attempt::Retryable(format!("reading response body: {err}"))),
        };

        if !status.is_success() {
            let message = format!(
                "Anthropic request failed: {status}{} — {text}",
                request_id_suffix(&request_id),
            );
            return Err(if is_transient_status(status.as_u16()) {
                Attempt::Retryable(message)
            } else {
                Attempt::Fatal(anyhow!(message))
            });
        }

        let json: Value = match serde_json::from_str(&text) {
            Ok(json) => json,
            Err(err) => return Err(Attempt::Fatal(anyhow!("parsing Anthropic response: {err}"))),
        };
        if json.get("stop_reason").and_then(Value::as_str) == Some("refusal") {
            return Err(Attempt::Fatal(anyhow!(
                "Anthropic refused the request{}",
                request_id_suffix(&request_id),
            )));
        }
        match extract_text(&json) {
            Ok(text) => Ok(Completion {
                text,
                usage: extract_usage(&json),
            }),
            Err(err) => Err(Attempt::Fatal(err)),
        }
    }
}

/// Whether an HTTP status warrants a retry: request timeout, conflict, too-early,
/// rate limit, the overloaded 529, and the 5xx server errors.
fn is_transient_status(code: u16) -> bool {
    matches!(code, 408 | 409 | 425 | 429 | 500 | 502 | 503 | 504 | 529)
}

fn request_id_suffix(request_id: &Option<String>) -> String {
    match request_id {
        Some(id) => format!(" (request-id: {id})"),
        None => String::new(),
    }
}

/// Build the JSON request body. Factored out so tests and the batch backend can
/// assert its shape or reuse it without the network. The user turn's content is
/// the mixed text/image array. When `effort` is `Some`, Opus 4.8's thinking depth
/// is set through `output_config.effort`; a request that uses `effort` leaves
/// `thinking` off, since the two are redundant.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_body(
    model: &str,
    system: Option<&str>,
    content: &[ContentBlock],
    max_tokens: u32,
    output_schema: Option<Value>,
    effort: Option<&str>,
    thinking: bool,
) -> Value {
    let blocks: Vec<Value> = content
        .iter()
        .map(|block| match block {
            ContentBlock::Text(text) => json!({ "type": "text", "text": text }),
            ContentBlock::Image { png_base64 } => json!({
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": "image/png",
                    "data": png_base64,
                }
            }),
        })
        .collect();

    let mut body = json!({
        "model": model,
        "max_tokens": max_tokens,
        "messages": [{ "role": "user", "content": blocks }],
    });
    if let Some(system) = system {
        body["system"] = json!(system);
    }
    let mut output_config = serde_json::Map::new();
    if let Some(schema) = output_schema {
        output_config.insert(
            "format".to_string(),
            json!({ "type": "json_schema", "schema": schema }),
        );
    }
    if let Some(effort) = effort {
        output_config.insert("effort".to_string(), json!(effort));
    }
    if !output_config.is_empty() {
        body["output_config"] = Value::Object(output_config);
    }
    if thinking {
        body["thinking"] = json!({ "type": "adaptive" });
    }
    body
}

/// Return the text of the first content block whose type is `"text"`. When
/// adaptive thinking is enabled the first block is a `thinking` block, so this
/// must not index `content[0]` blindly.
pub(crate) fn extract_text(response: &Value) -> Result<String> {
    let blocks = response
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("response has no content array"))?;
    for block in blocks {
        if block.get("type").and_then(Value::as_str) == Some("text") {
            if let Some(text) = block.get("text").and_then(Value::as_str) {
                return Ok(text.to_string());
            }
        }
    }
    Err(anyhow!("response contains no text block"))
}

/// Read the reply's top-level `usage.input_tokens` / `usage.output_tokens`,
/// defaulting a missing field to 0.
pub(crate) fn extract_usage(response: &Value) -> Usage {
    let usage = response.get("usage");
    let field = |name: &str| {
        usage
            .and_then(|u| u.get(name))
            .and_then(Value::as_u64)
            .unwrap_or(0)
    };
    Usage {
        input_tokens: field("input_tokens"),
        output_tokens: field("output_tokens"),
    }
}

/// What an outlier line band is, as judged by the optional Claude Vision line
/// review. Scoped to the boundary decisions a consumer acts on: a leading band may
/// be the unit's printed display `Title` or page `Furniture` (both pinned into
/// `skip_top_bands`), or ordinary `Body` (the heuristic's decision stands).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineVerdict {
    Title,
    Furniture,
    Body,
}

/// Page-role classification behind a trait so tests can substitute a stub.
/// Transcription and escalation no longer go through here — they run as batches.
pub trait Vision {
    /// Classify a page from a low-DPI thumbnail.
    fn classify(&self, png_base64: &str) -> Result<(PageRole, Usage)>;

    /// Judge a single line-band crop (used only by the opt-in `--review-lines`
    /// authoring pass) as a display title, page furniture, or ordinary body line.
    fn review_line(&self, png_base64: &str) -> Result<(LineVerdict, Usage)>;
}

/// The production [`Vision`] backed by the Anthropic client.
pub struct ApiVision {
    client: Client,
}

impl ApiVision {
    pub fn new(client: Client) -> Self {
        ApiVision { client }
    }
}

impl Vision for ApiVision {
    fn classify(&self, png_base64: &str) -> Result<(PageRole, Usage)> {
        crate::pages::classify_page(&self.client, png_base64)
    }

    fn review_line(&self, png_base64: &str) -> Result<(LineVerdict, Usage)> {
        crate::pages::review_line_page(&self.client, png_base64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_body_places_image_and_text_blocks() {
        let content = vec![
            ContentBlock::Text("describe this".to_string()),
            ContentBlock::Image {
                png_base64: "QUJD".to_string(),
            },
        ];
        let body = build_body(SONNET, Some("sys"), &content, 1024, None, None, false);
        assert_eq!(body["model"], SONNET);
        assert_eq!(body["max_tokens"], 1024);
        assert_eq!(body["system"], "sys");
        let blocks = body["messages"][0]["content"].as_array().unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0]["type"], "text");
        assert_eq!(blocks[0]["text"], "describe this");
        assert_eq!(blocks[1]["type"], "image");
        assert_eq!(blocks[1]["source"]["type"], "base64");
        assert_eq!(blocks[1]["source"]["media_type"], "image/png");
        assert_eq!(blocks[1]["source"]["data"], "QUJD");
    }

    #[test]
    fn build_body_places_fields_correctly() {
        let schema = json!({ "type": "object" });
        let content = vec![ContentBlock::Text("user prompt".to_string())];
        let body = build_body(
            HAIKU,
            Some("system prompt"),
            &content,
            256,
            Some(schema.clone()),
            None,
            false,
        );
        assert_eq!(body["model"], HAIKU);
        assert_eq!(body["max_tokens"], 256);
        assert_eq!(body["system"], "system prompt");
        assert_eq!(body["messages"][0]["role"], "user");
        assert_eq!(body["output_config"]["format"]["type"], "json_schema");
        assert_eq!(body["output_config"]["format"]["schema"], schema);
        assert!(body.get("thinking").is_none());
    }

    #[test]
    fn build_body_enables_thinking() {
        let content = vec![ContentBlock::Text("hi".to_string())];
        let body = build_body(OPUS, None, &content, 16, None, None, true);
        assert_eq!(body["thinking"]["type"], "adaptive");
        assert!(body.get("system").is_none());
        assert!(body.get("output_config").is_none());
    }

    #[test]
    fn build_body_emits_effort_when_present() {
        let content = vec![ContentBlock::Text("hi".to_string())];
        let body = build_body(OPUS, None, &content, 16, None, Some("medium"), false);
        assert_eq!(body["output_config"]["effort"], "medium");
        // effort subsumes thinking, so a request using effort sends no thinking block.
        assert!(body.get("thinking").is_none());
        // With effort but no schema, output_config carries effort and no format.
        assert!(body["output_config"].get("format").is_none());
    }

    #[test]
    fn extracts_first_text_block_past_thinking() {
        let response = json!({
            "content": [
                { "type": "thinking", "thinking": "hmm" },
                { "type": "text", "text": "the answer" }
            ],
            "stop_reason": "end_turn"
        });
        assert_eq!(extract_text(&response).unwrap(), "the answer");
    }

    #[test]
    fn transient_statuses_are_retried_not_fatal() {
        for code in [408, 429, 500, 502, 503, 504, 529] {
            assert!(is_transient_status(code), "{code} should retry");
        }
        for code in [400, 401, 403, 404, 422] {
            assert!(!is_transient_status(code), "{code} should be fatal");
        }
    }

    #[test]
    fn extract_usage_reads_token_counts() {
        let response = json!({
            "content": [],
            "usage": { "input_tokens": 1234, "output_tokens": 56 }
        });
        assert_eq!(
            extract_usage(&response),
            Usage {
                input_tokens: 1234,
                output_tokens: 56
            }
        );
    }

    #[test]
    fn extract_usage_defaults_missing_to_zero() {
        let response = json!({ "content": [], "usage": { "input_tokens": 10 } });
        assert_eq!(
            extract_usage(&response),
            Usage {
                input_tokens: 10,
                output_tokens: 0
            }
        );
        let no_usage = json!({ "content": [] });
        assert_eq!(extract_usage(&no_usage), Usage::default());
    }

    /// Live smoke test: one tiny Haiku call on a rendered page, printing the reply
    /// and its token usage, to confirm auth, image upload, and usage parsing. Needs
    /// a real `secrets.json` and the source PDF; ignored by default (hits the
    /// network and spends tokens). Run with `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn live_haiku_call_round_trips() {
        use std::path::Path;
        let secrets = crate::secrets::load(Path::new("secrets.json")).unwrap();
        let client = Client::new(secrets.anthropic_api_key);
        let dir = tempfile::tempdir().unwrap();
        let png = crate::render::render_page(
            Path::new("Ploutarchou_Bioi_v1.pdf"),
            40,
            crate::render::THUMBNAIL_DPI,
            dir.path(),
        )
        .unwrap();
        let bytes = std::fs::read(&png).unwrap();
        let content = vec![
            ContentBlock::Text("In one word, what script is the text on this page?".to_string()),
            ContentBlock::Image {
                png_base64: crate::b64(&bytes),
            },
        ];
        let completion = client.message(HAIKU, None, &content, 64, None).unwrap();
        println!("reply: {}", completion.text);
        println!("usage: {:?}", completion.usage);
        assert!(completion.usage.input_tokens > 0);
    }
}
