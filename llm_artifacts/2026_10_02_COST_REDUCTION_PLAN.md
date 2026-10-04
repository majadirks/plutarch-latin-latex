# Transcription Cost-Reduction Plan

**Authoring date:** 2026-10-02

## Context

The transcription pipeline works, but a full run is too expensive to be
feasible. A dry-run in which Opus arbitrated **every** page costs ≈ $0.23/page,
which extrapolates to **≈ $310** for the whole ≈ 1,328-page work — the realistic
worst case, because the current verification scheme sends essentially every dense
Latin page to Opus. The owner's ceiling for this one-time transcription is **≤ $35
total**.

This plan restructures the network-calling stages (classification, transcription,
verification, arbitration) to cut cost by roughly an order of magnitude while
preserving the project's correctness invariant: the stored Latin stays a verbatim
reproduction of the page, and cross-model reliability is kept rather than traded
away.

Six levers are incorporated:

1. **Batch API** — the run is asynchronous, so submit it through the Message
   Batches API for a flat 50% discount with no quality change. Because each request
   carries an inline image, the batch is **chunked** to stay under the 256 MB
   per-batch limit.
2. **Lean arbitration** — stop paying Opus on every page. The two passes are
   compared under **orthography-folding** (u/v, i/j, long-s, ligatures, case,
   spacing are ignored), so only **substantive** disagreement — not spelling noise —
   triggers Opus, and the remaining Opus calls run at **reduced effort**.
3. **Local cross-checks (free)** — section/chapter-number continuity and a coarse
   Tesseract-`lat` char-count check (page dropped/garbled) supplement the
   cross-model signal.
4. **Crop to the Latin column** — transcribe only the right column of **body**
   pages, halving image input tokens and usually improving accuracy. Front-matter
   prose is full-width and is **not** cropped.
5. **Model roles** — **Sonnet is the sole authoritative transcriber.** Haiku's
   Latin quality on this nineteenth-century Didot text is inadequate, so it is
   **never used as a source of stored text**; it runs as a cheap **divergence
   signal** only (its reading is compared against Sonnet's to decide whether a page
   needs Opus). Opus produces the authoritative text only on escalated pages.
6. **Compact output schema** — make `roman`/`section_no`/`continues_previous`
   optional so the model stops emitting null boilerplate on every block, trimming
   Sonnet's output tokens (which dominate its cost).

**Budget reality.** The Sonnet + Haiku two-pass base alone is ≈ $0.027/page
(≈ $35 over 1,328 pp) *before any Opus*, so realistic totals with the Haiku signal
land **≈ $40–45**. The plan therefore (a) gates the Haiku pass behind a toggle (a
`bool` threaded as a parameter, defaulting from a constant) so it is removable
without code surgery, and (b) ends with a **cost-validation gate**: the owner runs a
contiguous body-page sample, reads the per-model token totals from the end-of-run
usage report, applies the reference prices (batch-halved) by hand, and extrapolates
to the full page count. If that exceeds $35 the first remedy is to disable the Haiku
signal (largest single saving), then lower Opus effort, then raise the similarity
threshold. The owner decides at the gate.

**Project conventions (from `CLAUDE.md`), binding on every step:** no git (OneDrive
holds history; the Rust `target/` dir is redirected out of the synced tree via
`.cargo/config.toml`); transcribed Latin is verbatim source text, never normalized
in the generator; quality gates `cargo fmt --all --check`, `cargo clippy
--all-targets -- -D warnings`, and `cargo test` must all pass locally before a step
is done.

## Current state of the world

The crate `plutarch-reader` compiles with a green test suite. Relevant modules and
the cost drivers they create:

- **`src/llm/mod.rs`** — the blocking `Client` (`message` / `message_thinking` →
  `Completion { text, usage }`, timeouts + bounded retry), `build_body` /
  `extract_text` / `extract_usage`, `ContentBlock` (`Text` / `Image`), the `Vision`
  trait (`classify` / `transcribe` / `arbitrate`) and its production `ApiVision`,
  and the constants `HAIKU = "claude-haiku-4-5"`, `SONNET = "claude-sonnet-5"`,
  `OPUS = "claude-opus-4-8"`. Every call is **synchronous, full-price** — driver (1).
- **`src/llm/transcribe.rs`** — `transcribe_all` runs **two synchronous vision
  passes per page** (Sonnet then Haiku) via `transcribe_page`; `output_schema()`
  **requires** `kind, roman, section_no, continues_previous, text` on every block,
  so paragraphs carry `"roman":null,"section_no":null,"continues_previous":false`
  boilerplate — drivers (2) and (6). Also: `Mode`, `user_prompt`, `parse_pagedoc`.
- **`src/llm/arbitrate.rs`** — `arbitrate_page` calls Opus with `message_thinking`
  (adaptive) at `MAX_TOKENS = 24000`; thinking bills as output at $25/MTok. Also:
  `user_prompt`.
- **`src/verify.rs`** — `reconcile_page` takes pass A + pass B;
  `normalize_for_diff` keeps orthography, so Sonnet/Haiku disagree on essentially
  every page and Opus arbitrates ≈ 100% — driver (3). Also: `section_continuity`,
  `advance`, `roman_to_u32`, `ocr_crosscheck` / `ocr_divergence_flags`
  (`OCR_CHAR_TOLERANCE = 0.5`), `reconcile_all`, `summary`, `ArbiterFallback`
  guard (`doc_chars`).
- **`src/render.rs`** — `render_page` rasterizes the **whole page** at
  `TRANSCRIBE_DPI = 300`; `THUMBNAIL_DPI = 50`; `OPEN_RETRIES` for OneDrive
  hydration. The `image` crate is already a dependency (used by
  `pages::looks_like_skip`). Full-page image on every call is driver (4).
- **`src/pages.rs`** — `classify` (boundary-region Haiku vision via the `Vision`
  trait + a `render_thumb` closure; deterministic `looks_like_skip`),
  `classify_page`. **This stays synchronous and largely unchanged.**
- **`src/tags.rs`** — `Sidecar` (`schema_version`, `sources`, `front_matter`,
  `pages`, `lives`), `PageRecord` (`pass_a`, `pass_b`, `status`, `flags`,
  `result`), `PageDoc` / `Block`, `cache_key`.
- **`src/pipeline.rs`** — `run()` threads classify → `transcribe_all` →
  `reconcile_all` → assemble → split → latex → compile, sharing one
  `UsageTracker`; `RunArgs`, `Scope`. `run` takes a lazy `vision_factory`, invoked
  only under `args.retag || has_llm_work(...)`.

## Desired end state

A restructured network path whose full run targets **≤ $35 one-time** (re-runs stay
free and offline). The pipeline becomes:

1. **Classification** — Haiku over boundary thumbnails, **synchronous and
   unchanged** (small page count; batching it was judged not worth the churn).
2. **Transcription** — per page, a **Sonnet** pass (authoritative) and, when the
   divergence signal is enabled, a **Haiku** pass (signal only). Body pages use the
   **cropped Latin column**; front-matter prose uses the **full page**. Both passes
   use the **compact schema** and are submitted through the **Batch API in chunks**.
3. **Local verification (free)** — orthography-folded **similarity** of the Sonnet
   and Haiku passes (substantive-disagreement flag), **plus** numbering continuity,
   **plus** a coarse Tesseract char-count check (page dropped/garbled; secondary,
   skipped if Tesseract is absent or for front prose). A page with no flags is
   accepted as-is (`status = Agreed`, `result =` the **Sonnet** pass).
4. **Escalation (batched, rare)** — flagged pages are re-transcribed by **Opus at
   reduced effort**; `result =` the Opus pass (`status = Escalated`), with the
   `ArbiterFallback` guard keeping the Sonnet pass if Opus returns something
   implausibly short.

**Model decision:** Sonnet is the only source of stored transcription text on
unescalated pages; Opus on escalated pages. Haiku is a disagreement detector only —
its text is never stored as `result`.

**Toggle:** `const DIVERGENCE_SIGNAL: bool` (in `transcribe.rs`) is the default for
the Haiku pass, **threaded as a `bool` parameter** through `transcribe_all` and the
flag computation so a `#[test]` can exercise both states without touching the
constant. On → cross-model consensus (idea 5/F), higher cost. Off → Sonnet-only,
relying on numbering + Tesseract (cheaper). The owner flips the constant at the
validation gate (Phase 5) if the run would exceed budget.

**Cost model** (per-page, rounded; sanity-checked against a sample run's usage report
before the full run):

| Configuration | ≈ $/page | ≈ full run |
|---|---|---|
| Current dual-pass + always-Opus | 0.23 | 310 |
| Sonnet single pass, compact, cropped, batched | 0.020 | 27 |
| + rare Opus escalation (numbering/OCR only) | +0.003–0.006 | 31–35 |
| **+ Haiku divergence signal** (idea F, `DIVERGENCE_SIGNAL = true`) | +0.0065 base, +Opus on real disagreements | **40–45** |

So `DIVERGENCE_SIGNAL = true` is expected to exceed $35; the human cost gate (Phase 5)
confirms the real figure from a sample run and, if over, the documented remedy is:
disable the Haiku signal → lower `ARBITRATION_EFFORT` to `"low"` → raise
`SIMILARITY_THRESHOLD`.

**Preserved invariants:** verbatim faithfulness (Body crop must not clip Latin —
human-verified; front prose never cropped; verification only chooses which model's
verbatim text to keep, never rewrites it); resumability (in-flight batch ids
persisted per chunk so a crash re-attaches instead of resubmitting and re-paying);
fully offline rebuilds from a complete sidecar (zero network calls); the end-of-run
usage report.

**Sidecar additions:** `schema_version` bumps; `batches: Vec<BatchState>` recording
in-flight batch ids **per stage and per chunk**; `PageRecord.result_model:
Option<String>` recording whether `result` came from Sonnet or Opus.

## Tech Learnings

- **Message Batches API.** `POST /v1/messages/batches` with
  `{ requests: [{ custom_id, params }, …] }`; poll the batch until
  `processing_status == "ended"`; stream results, each carrying its `custom_id` and
  a `result.type` of `succeeded | errored | canceled | expired` (on success read
  `result.message.content` / `.usage`). **Flat 50% discount.** Limits: 100,000
  requests **or 256 MB** per batch — the latter binds here because images are inline
  and each page's PNG is sent **twice** when the Haiku signal is on (one Sonnet, one
  Haiku request), so **chunk by cumulative encoded request size** against a byte
  budget below 256 MB, not by a fixed page count. Most batches finish ≤ 1 h (max
  24 h); results retained 29 days; **unordered — key by `custom_id`.** First-party
  only; the endpoint rejects `fallbacks`/fast mode (neither used). Called directly
  over `reqwest`, like the existing `Client`.
- **`custom_id` scheme** (unique, ≤ 64 chars): `s_{src}_{page}` (Sonnet),
  `h_{src}_{page}` (Haiku signal), `o_{src}_{page}` (Opus escalation), where `{src}`
  is the **source index** (`0` = v1, `1` = v2) and `{page}` the 1-based source page.
  Results are parsed back to `(role, source, page)` by splitting on `_`.
- **In-memory column crop.** Render the full page with the existing `render_page`,
  then decode and crop the right fraction with the **`image` crate** (already a
  dependency): keep `x ∈ [round(w·CROP_LEFT_FRAC), w)`, full height, re-encode PNG.
  This avoids `pdftoppm` device-pixel math and per-source page-size plumbing (v1 is
  479×713 pt, v2 445×678 pt) and adapts to each page's actual rendered width.
- **Compact structured output.** Keep only `kind` and `text` in the schema's
  `required`; `roman`/`section_no`/`continues_previous` stay as nullable properties
  but are omitted unless they apply. `parse_pagedoc` already fills missing fields via
  serde defaults — no parser change.
- **`effort`.** `output_config: { effort: "low" | "medium" | "high" | "max" }` on
  Opus 4.8 controls thinking depth/output. Because it already governs thinking, a
  request that sets `effort` does **not** also send an adaptive-`thinking` block (the
  two are redundant); Opus escalation uses `effort` alone. `build_body` gains an
  `effort: Option<&str>` parameter (rippling to its callers, which pass `None`).
- **Orthography folding + similarity.** `fold(text)` lowercases, maps u→v and i→j,
  ſ→s, æ→ae and œ→oe, and strips whitespace/punctuation; `similarity(sonnet, haiku)`
  is `1 − token_levenshtein(fold(sonnet), fold(haiku)) / max_token_len`, a value in
  `[0, 1]` where `1.0` is identical, and a page is flagged when it falls **below**
  `SIMILARITY_THRESHOLD`. Folding spelling noise away means only real wording
  differences escalate. (The metric is deliberately a similarity, so its name and the
  "flag when below threshold" test read the same direction.)
- **Reference model prices** (per MTok, input/output; Sonnet's intro pricing expired
  2026-08-31, so standard): Haiku $1/$5, Sonnet $3/$15, Opus $5/$25; the Batch API
  halves both. These are **reference figures for the human cost gate** (Phase 5), not
  stored in code — verify them against current pricing before trusting the gate.
  Classification is synchronous and billed at full price; the Sonnet / Haiku / Opus
  batch stages are halved.

---

## Phase 1: Crop the Latin column (body pages only)

**Suggested implementer: Sonnet.** Mechanical render change reusing an existing
dependency; the crop fraction is a human-verified tunable.

### Step 1.1 — In-memory right-column crop

In `src/render.rs`:

- `pub const CROP_LEFT_FRAC: f32 = 0.48;` — fraction of page width left of the Latin
  column; tuned by eye.
- `pub fn crop_right_column(png_bytes: &[u8], left_frac: f32) -> Result<Vec<u8>>`:
  decode with `image`, crop `x ∈ [round(w·left_frac), w)` full height, re-encode PNG.
- `pub fn render_latin_column(pdf, page, dpi, out_dir) -> Result<PathBuf>`: call the
  existing `render_page`, read the bytes, `crop_right_column`, write the cropped PNG.
- `render_page` is unchanged and still serves classification thumbnails and
  front-matter scans; front-matter **prose** transcription (Phase 3) also uses the
  full page, never the crop.

**Tests:** `crop_right_column_keeps_right_fraction` (synthetic wide image →
output width ≈ `(1−frac)·w`, full height); `crop_preserves_png` (decodes back);
`render_latin_column_produces_png` (gated on `pdftoppm` + a real source PDF, like
the existing render test).
**LLM verification:** `cargo test render`; crop v1 p40 and confirm a PNG appears.
**Human verification (critical):** Open the cropped PNG for several body pages
(recto and verso, both volumes) and confirm it contains the **entire** Latin column
plus a small margin and **never clips** Latin text or the inline chapter/section
numbers; adjust `CROP_LEFT_FRAC` if the gutter sits differently between v1 and v2. If
a single fraction cannot fit both volumes, promote `CROP_LEFT_FRAC` to a per-source
pair and thread the source index into `render_latin_column`.

---

## Phase 2: Batch API backend (chunked, resumable)

**Suggested implementer: Opus.** External async API, chunking, result routing, and
resumable state.

### Step 2.1 — `build_body` gains `effort`; batch client

In `src/llm/mod.rs`, add `effort: Option<&str>` to `build_body` (emit
`output_config.effort` when `Some`) and thread it through `send` / `message` /
`message_thinking` (existing callers pass `None`). Update the affected `build_body`
tests.

In a new `src/llm/batch.rs` (`pub mod batch;`):

- `pub struct BatchRequest { custom_id, model, system: Option<String>, content:
  Vec<ContentBlock>, max_tokens, schema: Option<Value>, effort: Option<&'static
  str>, thinking: bool }`. For the Opus escalation requests `effort` is `Some` and
  `thinking` is `false` (effort subsumes thinking); Sonnet/Haiku requests set
  `effort: None`, `thinking: false`.
- `pub enum BatchOutcome { Ok(Completion), Err(String) }`; `pub enum BatchStatus {
  Processing, Ended }`.
- `pub trait BatchBackend { fn submit(&self, requests: &[BatchRequest]) ->
  Result<String>; fn poll(&self, batch_id: &str) -> Result<BatchStatus>; fn
  results(&self, batch_id: &str) -> Result<HashMap<String, BatchOutcome>>; }` — the
  submit/poll/results split lets the pipeline persist the id between submit and
  completion.
- `pub struct ApiBatchBackend { client: Client }` implementing it against the batch
  endpoints, reusing `build_body` for each request's `params` and `extract_text` /
  `extract_usage` for each result.
- `pub const BATCH_MAX_BYTES: usize = 200 * 1024 * 1024;` (a safety margin below the
  256 MB cap) and `pub const BATCH_MAX_REQUESTS: usize = 90_000;` (margin below the
  100,000 cap). A helper `pub fn run_chunked(backend, requests, stage, sidecar,
  sidecar_path, label) -> Result<HashMap<String, BatchOutcome>>` splits `requests`
  into chunks whose **cumulative encoded size stays under `BATCH_MAX_BYTES`** (and
  count under `BATCH_MAX_REQUESTS`), estimating each request's size from its base64
  image length plus prompt; a single request larger than the budget forms its own
  chunk. For each chunk: reuse a persisted `BatchState { stage, chunk, batch_id }`
  if present (resume), else `submit` and persist it; `poll` with backoff to `Ended`;
  collect `results`; clear that `BatchState`; drive an `instrument::progress_bar`
  across chunks.
- Pure, network-free helpers: `build_batch_body(requests) -> Value` and
  `parse_batch_results(text) -> HashMap<String, BatchOutcome>`.

This step adds the backend **standalone**: nothing calls `run_chunked` or
`ApiBatchBackend` yet, so the crate still compiles with the existing synchronous
stages untouched. Wiring happens in Steps 3.2 and 4.2.

**Tests:** `build_body_emits_effort_when_present`;
`build_batch_body_wraps_requests_with_custom_ids`;
`parse_batch_results_keys_by_custom_id`;
`parse_batch_results_marks_errored_and_expired`;
`run_chunked_splits_on_byte_budget` and
`run_chunked_resumes_from_persisted_state` (stub backend); a `#[ignore]`d live
one-request batch test.
**LLM verification:** `cargo test batch llm`; `cargo build`.
**Human verification:** None (exercised live in Phase 5).

### Step 2.2 — Sidecar batch state

In `src/tags.rs`: bump `SCHEMA_VERSION`; add `pub batches: Vec<BatchState>` to
`Sidecar` (`BatchState { stage: String, chunk: usize, batch_id: String }`) and
`pub result_model: Option<String>` to `PageRecord`.

**Tests:** `sidecar_roundtrips_batches_and_result_model`; `schema_version_bumped`.
**LLM verification:** `cargo test tags`.
**Human verification:** None.

---

## Phase 3: Compact schema + batched transcription

**Suggested implementer: Opus.** Restructures the transcription loop and rewires the
pipeline's transcription call.

### Step 3.1 — Compact output schema

In `src/llm/transcribe.rs`: in `output_schema()` keep only `kind` and `text` in
`required` (others stay nullable); in `BODY_PROMPT` / `FRONT_PROMPT` instruct the
model to include `roman` only on chapter blocks, `section_no` only on section
blocks, and `continues_previous` only when true. `parse_pagedoc` is unchanged.

**Tests:** `output_schema_requires_only_kind_and_text`;
`parse_pagedoc_fills_defaults_for_omitted_fields`; existing `body_prompt_*` /
`parse_pagedoc_*` stay green (add an "include only when applicable" assertion).
**LLM verification:** `cargo test transcribe`.
**Human verification:** None.

### Step 3.2 — Batched Sonnet + optional Haiku passes

- Add `pub const DIVERGENCE_SIGNAL: bool = true;` as the **default** for the Haiku
  pass. Thread the choice as an explicit `divergence_signal: bool` parameter through
  `transcribe_all` (and, in Phase 4, the flag computation) rather than reading the
  constant inside those functions, so a `#[test]` can exercise both states; the
  pipeline passes `DIVERGENCE_SIGNAL`.
- Rewrite `transcribe_all(backend, sidecar, tracker, sidecar_path, limit, in_scope,
  divergence_signal, render_body, render_full) -> Result<()>`: gather every in-scope
  `Body`/`FrontProse` page into a `Vec<BatchRequest>` — a **Sonnet** request
  (`s_{src}_{page}`) for each page whose `pass_a` is `None`, and, when
  `divergence_signal`, a **Haiku** request (`h_{src}_{page}`) for each page whose
  `pass_b` is `None`. Gating on the **missing pass** (not on a missing `result`)
  means a re-run re-batches only the passes actually absent, so an errored/expired
  request is retried on the next run. **Body** pages use `render_body` (cropped
  column); **FrontProse** pages use `render_full`. `--dry-run N` still caps body
  pages via `limit`; `in_scope` restricts a `--pages` range. Submit via
  `batch::run_chunked`.
- On results, key by `custom_id`: store each succeeded Sonnet `PageDoc` in `pass_a`
  and each succeeded Haiku one in `pass_b`, set `result_model = None` (decided in
  Phase 4), set `image_sha256` from the page's rendered bytes, record each model's
  usage against `(src, page)`, and save the sidecar per chunk. A `BatchOutcome::Err`
  (errored/expired request) leaves that pass `None` and is **counted and printed as a
  warning**, never fabricated — the page simply lacks that pass until a later run
  re-batches it.
- Keep the `Vision` trait and `ApiVision` **intact** in this step (classification
  still uses `Vision::classify`; `Vision::transcribe` / `Vision::arbitrate` and
  `transcribe_page` / `arbitrate_page` are now unused by the batched path but still
  compile, and are removed in Step 4.2 once `reconcile_all` no longer uses them).
  Keep `user_prompt`, `output_schema`, `parse_pagedoc`, which the batch request
  builder reuses. Rewrite only this module's transcription-loop tests as
  `BatchBackend`-stub tests.
- **Wire the signature change end to end in this same step** so the crate keeps
  compiling: change `run`'s factory from `impl FnOnce() -> Result<Box<dyn Vision>>`
  to one yielding **both** a `Box<dyn Vision>` (for classification) and a `Box<dyn
  BatchBackend>`, built lazily from a single `Client`; update `main.rs` to build
  both. `run` already invokes the factory only under `args.retag ||
  has_llm_work(...)`, so a fully-cached offline run builds neither and makes zero
  network calls. Thread the backend through `run_llm_phases` alongside the existing
  `vision`, and update the `transcribe_all` call in `pipeline.rs` to pass
  `DIVERGENCE_SIGNAL`, a `render_body` closure backed by `render::render_latin_column`
  bytes, and a `render_full` closure backed by `render::render_page` bytes. Leave the
  `reconcile_all` call (still Vision-based) unchanged; it keeps compiling.

**Tests (stubbed `BatchBackend`):** `transcribe_all_submits_sonnet_and_haiku_per_page`
(`divergence_signal = true`); `transcribe_all_omits_haiku_when_signal_off`
(`divergence_signal = false`, zero Haiku requests);
`frontprose_uses_full_page_body_uses_crop` (assert which render fn is called);
`transcribe_all_records_usage_per_model`;
`transcribe_all_skips_pages_with_passes` (a page already holding `pass_a`/`pass_b` is
not re-requested); `transcribe_all_warns_and_leaves_pass_unset_on_errored_request`;
`dry_run_limits_body_pages`; `transcribe_all_resumes_from_persisted_batch_id`.
**LLM verification:** `cargo test transcribe`; `cargo build` (crate still compiles
end to end).
**Human verification:** `--dry-run 3` against the real PDFs produces three pages of
Sonnet (and Haiku) transcription in the dryrun sidecar; inspect the Latin.

---

## Phase 4: Divergence-driven, lean arbitration

**Suggested implementer: Opus.** Changes what triggers Opus and how much it costs,
and finishes rewiring the pipeline.

### Step 4.1 — Fold, compare, and flag

In `src/verify.rs`:

- `pub fn fold(text: &str) -> String` (orthography folding per "Tech Learnings").
- `pub fn similarity(a: &PageDoc, b: &PageDoc) -> f32` — token-level **similarity** in
  `[0, 1]` of the folded block-text concatenations (`1.0` = identical):
  `1 − token_levenshtein / max_token_len`. `pub const SIMILARITY_THRESHOLD: f32 =
  0.90;`. The metric is a similarity so the naming and the comparison agree: a page is
  flagged when similarity is **below** the threshold.
- Retune `ocr_divergence_flags` to a **coarse** page-integrity check: flag only when
  the OCR non-space char count is `< 0.5×` or `> 1.6×` the Sonnet pass's; document
  that it is secondary (skipped when Tesseract is absent or for front prose) and that
  the Sonnet↔Haiku similarity is the primary cross-check.
- `pub fn page_flags(continuity: &Continuity, record: &PageRecord, ocr: Option<&str>,
  divergence_signal: bool) -> Vec<VerifyFlag>`: compute a page's flags from its
  **Sonnet** pass (`pass_a`) — a `section_continuity` break, the coarse OCR check
  (Body only, when `ocr` is `Some`), and, **only when `divergence_signal` and
  `pass_b.is_some()`**, `similarity(pass_a, pass_b) < SIMILARITY_THRESHOLD`. Taking
  the signal as a parameter keeps both states testable without the constant.
- **Continuity runs off the Sonnet pass.** When flags are computed for the whole run
  up front (Step 4.2) the running `Continuity` is advanced with `advance(&state,
  pass_a)` — the Opus results do not exist yet — so numbering continuity is judged
  from Sonnet throughout.

### Step 4.2 — Opus re-transcribes flagged pages at reduced effort

- `pub const ARBITRATION_EFFORT: &str = "medium";`. On Opus 4.8 `effort` controls
  thinking depth, so escalation sets `effort = Some(ARBITRATION_EFFORT)` and sends
  **no separate adaptive-`thinking` block** (`thinking: false`); the two are
  redundant.
- Rewrite `reconcile_all(backend, sidecar, tracker, sidecar_path, divergence_signal,
  render_body, render_full, ocr) -> Result<()>`:
  - **Targets** are in-order `Body`/`FrontProse` pages with `pass_a.is_some()` and
    `result.is_none()` (`pass_b` is **not** required, so the signal-off path and a
    page whose Haiku request errored still reconcile).
  - First pass (free): walk the targets in order, advancing `Continuity` off each
    page's `pass_a`, and store each page's `page_flags(...)` on its record (Body
    pages consult `ocr(source, page)`).
  - Collect the flagged pages into **one** Opus chunked batch (`o_{src}_{page}`), each
    re-transcribing from the page's **own image mode** (Body = `render_body` cropped,
    FrontProse = `render_full`), compact schema, `effort = ARBITRATION_EFFORT`,
    `max_tokens` headroom (keep the existing `24000`). A page with no flags is
    accepted as-is: `status = Agreed`, `result = pass_a`, `result_model = Some(SONNET
    .into())`, **no Opus call**.
  - On Opus results: `result =` the Opus `PageDoc`, `status = Escalated`,
    `result_model = Some(OPUS.into())`, record Opus usage. Keep the `ArbiterFallback`
    guard (`doc_chars`): if the Opus result is implausibly short vs `pass_a`, keep
    `pass_a` and push an `ArbiterFallback` flag. A `BatchOutcome::Err` for a flagged
    page keeps `pass_a` as `result` with an `ArbiterFallback` flag and is
    counted/warned, so no page is dropped.
  - `summary` reports agreed / escalated counts with page refs. The `Arbitrated`
    status (and `ReconcileSummary.arbitrated`) is no longer produced; leave the enum
    variant in place so old sidecars still deserialize, but `summary` only sorts
    pages into agreed / escalated.
- **Now** trim the `Vision` trait to **`classify` only**: remove `Vision::transcribe`
  / `Vision::arbitrate`, `ApiVision`'s impls of them, the now-unused `transcribe_page`
  (keep `user_prompt` / `output_schema` / `parse_pagedoc`) and `arbitrate_page` (keep
  `arbitrate::user_prompt`, reused by the Opus request builder), and drop those
  methods from the remaining test stubs. Rewrite the reconcile tests as
  `BatchBackend`-stub tests.
- **Wire the call site in this same step:** update the `reconcile_all` call in
  `pipeline.rs` to pass the `BatchBackend`, `DIVERGENCE_SIGNAL`, the `render_body` /
  `render_full` closures, and the existing `ocr` closure. After this step the crate
  compiles with classification synchronous and both batched stages wired.

**Tests (stubbed backend):** `agreement_folds_orthography_and_skips_opus` (Sonnet vs
Haiku differing only by u/v, long-s → similarity ≥ threshold → not flagged, zero Opus
requests); `substantive_disagreement_escalates`; `numbering_break_escalates`;
`coarse_ocr_flag_escalates`; `escalation_sends_effort_medium_and_no_thinking`;
`short_opus_result_falls_back_to_sonnet`; `errored_opus_request_keeps_sonnet`;
`signal_off_relies_on_numbering_and_ocr` (`divergence_signal = false`, a clean page
with no `pass_b` skips Opus); `summary_counts_statuses`. Existing `continuity_*`,
`roman_parsing`, `ocr_*`, `doc_chars` tests stay green.
**LLM verification:** `cargo test verify`; `cargo build`.
**Human verification:** On the dry-run pages, confirm orthography-only differences do
not escalate, a seeded substantive disagreement and a seeded numbering break each
route exactly one page to Opus, and read the summary.

---

## Phase 5: Offline-path guard, cost gate, and README

**Suggested implementer: Opus.** Final integration check plus the go/no-go budget
decision.

### Step 5.1 — Verify the offline path and end-to-end dry-run

The stage wiring landed with the stages themselves (Steps 3.2 and 4.2):
classification stays synchronous (`Vision::classify`, `ApiVision` built lazily), and
`transcribe_all` / `reconcile_all` run through the `ApiBatchBackend`, all sharing one
`UsageTracker`, with `render_body = render::render_latin_column` and `render_full =
render::render_page`. This step confirms the integration holds together and preserves
the offline guarantees:

- Confirm the factory (both `Vision` and `BatchBackend`) is invoked only under
  `args.retag || has_llm_work(...)`, so a complete sidecar makes **zero** network
  calls; `--retag`, `--dry-run`, `--pages`, and the sample-life fallback still work.
- Confirm per-chunk sidecar saves and persisted `BatchState` make an interrupted run
  re-attach rather than resubmit.

**Tests:** `run_offline_with_fresh_sidecar_makes_no_network_calls` and
`run_offline_usage_report_is_empty` stay green.
**LLM verification:** `cargo test`; `cargo build`.
**Human verification (critical):** A `--dry-run 4` end-to-end build produces a small
inner + imposed PDF and a non-trivial usage report.

**Cost gate (critical, human):** Run a contiguous body-page sample, e.g.
`--pages v1:200-260` (~60 pages), on a **fresh** dryrun sidecar (delete it or pass
`--retag` so usage is actually measured rather than served from cache). Read the
per-model token totals from the printed usage report, apply the reference prices
(batch-halved for the Sonnet / Haiku / Opus stages; classification, if exercised, is
full price) by hand, and extrapolate to the full page count. **If over $35**, apply
the remedies in order and re-measure: (1) set `DIVERGENCE_SIGNAL = false` (removes the
Haiku pass — the largest saving); (2) set `ARBITRATION_EFFORT = "low"`; (3) raise
`SIMILARITY_THRESHOLD` toward 0.97 so only large disagreements escalate. Do **not**
launch the full run until the gate passes or the owner accepts the figure.

### Step 5.3 — README

Update `README.md`: the run is submitted via the Batch API in chunks (expect up to
~1 h; batch ids persist so an interrupted run re-attaches); transcription is a
Sonnet pass on the cropped Latin column with an optional Haiku divergence signal,
numbering + coarse OCR as local checks, and Opus only on flagged pages at reduced
effort; the compact schema; the one-time cost and how to sanity-check it from a
sample run's usage report against the reference prices; and that reliability rests on
Sonnet-quality text plus cross-model divergence detection plus the human spot-check,
not on Haiku's own Latin readings.

**LLM verification:** none (docs).
**Human verification:** Follow the README; confirm a `--dry-run` build works and the
usage-report cost check is reproducible.

---

## Cross-cutting requirements

- Every Rust-touching step ends green on `cargo fmt --all --check`, `cargo clippy
  --all-targets -- -D warnings`, and the relevant `cargo test`; the full `cargo
  test` is green before a phase is declared done. No CI — run locally.
- Each step leaves the crate compiling: a step that changes a public function's
  signature (`transcribe_all`, `reconcile_all`, `run`'s factory) updates that
  function's call sites and `main.rs` in the **same** step, and the `Vision` trait is
  trimmed only in Step 4.2, after `reconcile_all` stops calling `arbitrate`.
- No git operations; no `.github/`. The build dir stays out of OneDrive via the
  existing `.cargo/config.toml`.
- Transcribed Latin is reproduced **verbatim**: Body crop must not clip it, front
  prose is never cropped, and verification/escalation only chooses which model's
  verbatim text to keep (Sonnet or Opus) — never normalizes or rewrites. Haiku text
  is a signal only and is never stored as `result`.
- No page is silently dropped: a failed/expired batch request leaves the affected
  pass unset (so a later run re-batches it) or keeps the Sonnet pass as `result`, and
  the failure is counted and printed.
- Network calls happen only in classification / transcription / escalation, only
  when the sidecar is missing/stale/`--retag`, and only through the Batch API
  (except synchronous classification); every other run is offline with zero calls.
- No code comment references this plan, its phases, or `llm_artifacts/`.
