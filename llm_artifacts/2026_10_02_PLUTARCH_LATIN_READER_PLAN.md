# Plutarch Latin Reader Generator — Implementation Plan

**Authoring date:** 2026-10-02

## Context

This project builds a Rust command-line tool that reads the two scanned Google
Books PDFs of **Plutarch's *Lives*** in the Didot bilingual recension (Dœhner,
Paris 1857) — `Ploutarchou_Bioi_v1.pdf` (643 pp) and `Ploutarchou_Bioi_v2.pdf`
(722 pp) — extracts **only the Latin translation**, and emits LaTeX source plus
compiled PDFs for a **Loeb-size (10.5 × 16 cm), hand-bound, Latin-only reader**.
The finished pages are imposed two-up onto US Letter (8.5 × 11 in) sheets, printed
double-sided, folded into 8-sheet (32-page) signatures, sewn through printed
stations, and trimmed to the pocket size.

Each source page carries **two side-by-side columns: Greek on the left, Latin on
the right.** We keep the Latin and discard the Greek — so, unlike the bilingual
French reader this project borrows its binding machinery from, the output is
**monolingual**: there is no `reledpar`, no facing-page balancing, and no
cross-language alignment tagging. The hard problem moves entirely to **faithful
vision transcription of Latin from untrusted scans**. The signature imposition,
fold/sew/cut guides, volume splitting, signature padding, and two-stage compile
all carry over, structurally, from the prior art.

The work matters now because we have a specific printed edition to reproduce
faithfully for binding, and the OCR layer in these Google scans is not
trustworthy. We transcribe with AI vision, using the facing Greek as an accuracy
check, and gate the result behind an automated multi-model verification scheme so
the Latin we print matches the page.

**Prior art (style and structure to follow, not behavior to copy wholesale):**
`C:\Users\majad\coding\bilingual_french_readers\bilingual_french_readers`. Reuse
its crate conventions, its Anthropic-over-HTTP client shape, its sidecar/caching
discipline, and its imposition/signature/compile code. Do **not** import its
`reledpar` parallel-typesetting layer — this reader is single-column.

**Project conventions (from `CLAUDE.md`), binding on every step:**

- **No git, no CI.** OneDrive provides version history. Do not run `git init`,
  propose branches/PRs, or add `.github/` workflows. The prior art's CI phases do
  **not** carry over. Quality gates run locally: `cargo fmt --all --check`,
  `cargo clippy --all-targets -- -D warnings`, and `cargo test` must all pass
  before any step is "done."
- **Transcribed Latin is source text, never silently "improved."** Do not
  normalize u/v, i/j, ligatures, the long s, spelling, punctuation, or
  capitalization in the generator. The transcription cache is the single source
  of truth; corrections are deliberate, auditable edits to the cache with a note,
  never on-the-fly cleanups downstream. This is the project's correctness
  invariant — the analog of the French reader's faithfulness gate.
- Rewrite prose in place (no "update note" layers); comments describe present
  state (no history, no plan references); place limiting modifiers next to what
  they modify; do not renumber existing numbered items (suffix/fraction instead).

## Current state of the world

- **Repository:** the project root `C:\Users\majad\OneDrive\Documents\Latin\Plutarch`.
  There is **no Rust code yet** — no `Cargo.toml`, no `src/`. The only code-adjacent
  artifacts are this plan and the design-decision notes.
- **Inputs:** two Google Books scans in the project root:
  - `Ploutarchou_Bioi_v1.pdf` — 643 pages, 479 × 713 pt. `VOLUMEN PRIMUM`.
  - `Ploutarchou_Bioi_v2.pdf` — 722 pages, 445 × 678 pt. `VOLUMEN SECUNDUM`.
  Together they are the two physical halves of one continuous work; we concatenate
  and re-paginate freely into N output volumes.
- **Page anatomy (body):** two columns, **Greek left / Latin right**, consistent
  on recto and verso. A centered running head carries the Life name and the
  chapter numbers on the page (e.g. `ROMULUS. 11, 12.`); the printed page number
  sits in the top outer corner. Within the Latin column, **chapter numbers are
  Roman** (`XI.`, `XII.`) and **section numbers are parenthetical arabic**
  (`(2) (3) (4)`). The far-left marginal numbers (5, 10, 15 …) are **Greek line
  numbers** and are discarded.
- **Structure:** the work is a sequence of **pair-groups** — two parallel Lives
  followed by their *Comparatio* (synkrisis), e.g. Theseus + Romulus +
  *Comparatio Thesei et Romuli*. Life/Comparatio boundaries are visible from
  display headings and running heads.
- **Front matter (v1):** p1 Google/QR notice; p2–4 cover/marbling; p5–7, 9, 11
  blank; **p8** bilingual half-title (`ΠΛΟΥΤΑΡΧΟΥ ΣΥΓΓΡΑΜΜΑΤΩΝ … PLUTARCHI OPERUM
  VOLUMEN PRIMUM`); **p10** title page (`PLUTARCHI VITÆ … THEOD. DŒHNER … GRÆCE
  ET LATINE … PARISIIS, FIRMIN DIDOT … M DCCC LVII`) with a decorative engraved
  vignette; **p12–13** Monitum (Latin prose interleaved with Greek fragments);
  body begins ~p14–15. v2 mirrors this (its own half-title/title for
  `VOLUMEN SECUNDUM`, plus a **Praefatio** — Latin prose; no Monitum).
- **Back matter:** library date-due slips, barcodes, marbling, dark cover scans —
  all ignored.
- **Local tooling (verified present):** poppler `pdftoppm`/`pdfinfo`
  (`C:\Users\majad\Downloads\poppler\…\bin`); MiKTeX `lualatex`/`xelatex`
  (needed for Old Standard via `fontspec`); Tesseract with the `lat` language
  model installed (`C:\Program Files\Tesseract-OCR`); `cargo`/`rustc`. No
  ghostscript/mutool/imagemagick.
- **Settled design decisions** (recorded in agent memory
  `plutarch-design-decisions.md`), summarized in "Desired end state" below.

## Desired end state

A single Rust crate, **`plutarch-reader`**, builds a binary invoked as:

```
plutarch-reader \
  --v1 Ploutarchou_Bioi_v1.pdf \
  --v2 Ploutarchou_Bioi_v2.pdf \
  --out build/ \
  [--retag] [--dry-run N] [--pages v1:A-B | v2:A-B]
```

Behavior:

- **Transcription (the core).** Every body page is transcribed **twice and
  independently** — pass A on **Sonnet**, pass B on **Haiku** — each given the
  full page image and instructed to transcribe **only the right/Latin column**,
  using the Greek **only** as a disambiguation aid. Output is a **structured**
  per-page document (headings, chapter markers, section markers, paragraph
  continuation flags, running text), not a blob. The two passes are diffed after
  whitespace/line-wrap normalization; on any disagreement — or when a cheap
  deterministic check fires on a page where the two passes *agreed* —
  **Opus arbitrates** from the image + Greek + both candidates and produces the
  authoritative text. There is **no human in the loop**. Latin-prose front matter
  (the Monitum) is transcribed **faithfully in full, Greek fragments included**,
  in a single "full transcription" mode.
- **Cheap verification signals (every page):** (a) **section-number continuity** —
  `(2)(3)(4)…` and Roman chapter numbers must stay contiguous/monotonic across
  pages; a break escalates the page to Opus; (b) **Tesseract-`lat` cross-check** —
  OCR the Latin column and compare char/line counts against the accepted
  transcription; a large divergence escalates to Opus. The pipeline emits a
  lightweight log (`agreed / arbitrated / escalated` counts with page refs) the
  user may optionally spot-check; nothing blocks on review. At the end of any run —
  dry-run or full — it also prints a **usage report**: per model (Haiku, Sonnet,
  Opus) the number of API calls made this run and the tokens spent — total, median
  per page, max over all pages, and min over all pages. A fully-cached offline run
  reports zero calls.
- **Caching / purity.** A committed sidecar `plutarch.tags.json` holds per-page
  records keyed by `(source_sha256, source_page, dpi)` — so a resumed run
  recognizes an already-tagged page **without re-rasterizing** it — with the
  page-image SHA-256 stored only as an integrity field. Each record carries both
  candidate transcriptions, the agreement status, the arbitrated result, and the
  per-page verification flags; alongside them the sidecar holds the page-role map,
  the Life/pair-group boundary table, derived front-matter metadata, and source
  hashes. While LLM calls are in flight (classification, transcription,
  arbitration), a live `indicatif` progress bar shows how far through the batch the
  run is. ("Committed" means kept in the OneDrive project tree — there is no git.)
  It is a **resumable accumulator**: a crash or `Ctrl-C` resumes, re-running only
  missing/forced pages. A **restricted run** (`--dry-run N` or `--pages`) writes a
  separate `plutarch.dryrun.tags.json` so it never pollutes the committed cache.
  Normal builds read the sidecar and make **zero** network calls; `--retag` (or a
  changed source hash) re-calls the API using the key in `secrets.json`.
- **Front matter** renders with **roman** page numbers and is **per source
  volume**, opening that source's first output TOMUS: for **v1**, the Google/QR
  page as an **embedded cropped scan image**, the title page **re-typeset** in Old
  Standard (engraved vignette embedded as a cropped image in place), and the
  **Monitum** re-typeset (Latin + Greek); for **v2**, v2's Google/QR image, title
  page, and the **Praefatio** re-typeset. The original `VOLUMEN PRIMUM/SECUNDUM`
  half-titles are dropped as artifacts of the original split (flagged for
  confirmation). Blank pages are inserted to keep parity (e.g. each prose piece
  starts recto/odd). The body renders with **arabic** page numbers.
- **Body** is single-column Latin in **Old Standard**, Loeb 10.5 × 16 cm trim,
  reproducing Roman chapter numbers and parenthetical section numbers **inline,
  exactly as printed**. Embedded **verse quotations** (e.g. the Pindar lines
  opening *Nicias*) are set as offset, line-broken verse, visually distinct from
  the surrounding prose. Each **Life** and each **Comparatio** starts on a fresh
  page. A **pair-group** (two Lives + their Comparatio) is never split across a
  volume boundary. Each volume opens with a generated label page reading
  `TOMUS I`, `TOMUS II`, …
- **Volumes.** A **hard volume break is forced at the v1→v2 source boundary**:
  v1's pair-groups and v2's pair-groups are split into output volumes
  independently and concatenated, so v2 always begins a fresh output TOMUS
  carrying v2's own front matter. Target **~400 physical pages** each, **≤ 500**,
  roughly balanced, split only on pair-group boundaries. Two blank leaves at the
  front and two at the back of every volume (inside the sewn block). Each volume's
  inner page count is padded with blank pages to a **multiple of 32** for
  signature binding.
- **Output per volume:** an inner `.tex` (`tomus_1_inner.tex`, …) at trim size and
  an imposition wrapper (`tomus_1.tex`, …). The build is two-stage: `lualatex` the
  inner file, **read its real page count from the lualatex log**, pad to a
  multiple of 32, then emit and `lualatex` the wrapper, which pulls inner pages
  two-up via `\includegraphics[page=N]{tomus_1_inner.pdf}` in 32-page-signature
  folding order. Imposition: sheet *i* outer side `34−2i | 2i−1`, inner side
  `2i | 33−2i`; even book-pages on the left so gutters face the fold. A dotted
  **fold line** plus **sew marks** are drawn on every **outer/recto** side only
  (between 1|32, not 2|31); a dotted **cut border** on the first recto of each
  signature. Sew marks: solid dots (radius `\sewdot = 0.06 cm`), two kettle
  stitches 72 pt (`SEW_MARGIN_CM = 2.54`) from each fold end, plus 2 tape supports
  evenly spaced between them, each tape two holes 42 pt apart (`SEW_HALFTAPE_CM =
  0.7408`) — 6 dots per fold, symmetric about center. These are tunable constants.
- **`--dry-run N`** runs the full pipeline on only the first N body pages
  (producing a short inner + wrapper PDF) for cheap testing; `--pages v1:A-B`
  restricts to a source page range (qualified by which input PDF). Both are
  restricted runs and use the separate `plutarch.dryrun.tags.json`. A **README**
  explains build, dependencies (poppler, MiKTeX + Old Standard, Tesseract-`lat`, a
  `secrets.json`), and how to run.

A junior developer recognizes "done" when: `cargo test` is green; a first run
with a valid `secrets.json` builds `plutarch.tags.json`, and subsequent runs are
offline and instant; and the two-stage compile yields per-volume US Letter PDFs
whose folded-and-trimmed leaves read in order, whose fold/sew/cut guides land
correctly, and whose Latin — on a real-eyes check against the scans — matches the
page, numbering included.

## Tech Learnings

- **Rasterizing PDF pages (poppler `pdftoppm`).** We shell out to `pdftoppm`
  (already installed) to turn a source page into a PNG:
  `pdftoppm -png -f N -l N -r <dpi> input.pdf outprefix` writes
  `outprefix-00N.png`. DPI trades legibility against vision token cost; this plan
  fixes a constant (tunable) around 300 for transcription. We hash the rendered
  PNG bytes (SHA-256) to key the cache, so re-rendering identical pixels is a
  cache hit.
- **The Anthropic Messages API with images.** No official Rust SDK, so we call
  `POST https://api.anthropic.com/v1/messages` with `reqwest` (headers
  `x-api-key`, `anthropic-version: 2023-06-01`, `content-type: application/json`).
  An **image** is a content block
  `{"type":"image","source":{"type":"base64","media_type":"image/png","data":…}}`
  placed in the user turn alongside a `{"type":"text",…}` instruction block. The
  reply text is the **first content block whose `type` is `"text"`** (skip any
  `thinking` block that precedes it when extended thinking is on). Model IDs:
  `claude-sonnet-5` (pass A), `claude-haiku-4-5` (pass B), `claude-opus-4-8`
  (arbiter, with adaptive thinking).
- **Token usage from the Messages API.** Every reply carries a top-level
  `usage` object — `{"input_tokens": N, "output_tokens": M, …}`. We read those two
  fields off each response to tally what the run spent. Input tokens dominate here
  because each transcription call ships a ~300-DPI page image; a tqdm-style bar and
  an end-of-run report let the user see the cost without external tooling.
- **`indicatif` progress bars.** `indicatif::ProgressBar::new(len)` with a
  `ProgressStyle` template renders a tqdm-style bar (`[####>   ] 42/128 ETA 00:30`)
  on stderr, advanced one tick per page. One shared style/helper keeps the bars in
  classification, transcription, and arbitration visually consistent.
- **Structured outputs.** We force each transcription reply to a JSON schema via
  `output_config.format` so the per-page document is guaranteed parseable (blocks
  with `kind`, optional `roman`/`section_no`, `continues_previous`, `text`)
  instead of free text we must scrape.
- **Old Standard via `fontspec` (lualatex).** Old Standard TT covers Latin **and
  polytonic Greek**, so one `\setmainfont{Old Standard}` under `lualatex` renders
  both scripts (needed for the Monitum's Greek fragments). This is why the engine
  is `lualatex`, not `pdflatex`.
- **Imposition with `graphicx` + `tikz`.** The trim-size inner PDF is not printed
  directly: a US Letter landscape wrapper pulls single inner pages with
  `\includegraphics[page=N]{inner.pdf}` and positions two per sheet side inside a
  `tikzpicture` whose coordinates are the sheet in centimetres; the same picture
  draws the fold line, sew dots, and cut border. Placing pages in the signature
  folding order makes the nested/folded/trimmed stack read 1, 2, 3, ….
- **Reading the real page count.** `reledpar` is gone, so the inner compile is a
  single `lualatex` pass, but we still must learn the **actual** inner page count
  to drive the wrapper and signature padding. We read it from the lualatex
  `.log`/`.aux` (or `pdfinfo` on the produced inner PDF) rather than estimating.
- **`serde`/`serde_json`, `clap`, `sha2`** as in the prior art: sidecar
  (de)serialization, CLI parsing (`--v1/--v2/--out/--retag/--dry-run/--pages`),
  and content hashing for cache keys.

---

## Phase 1: Crate skeleton, secrets, CLI

**Suggested implementer: Sonnet.** Mechanical, well-specified.

### Step 1.1 — Create the Cargo crate

Create `Cargo.toml` and a minimal binary so the repo compiles.

- `Cargo.toml`: package `plutarch-reader`, edition `2021`. Dependencies: `clap`
  (features `derive`), `serde` (features `derive`), `serde_json`, `sha2`, `anyhow`,
  `thiserror`, `reqwest` (features `blocking`, `json`, `rustls-tls`; default
  features off), `regex`, `base64`, `indicatif`. Dev-dependency: `tempfile`.
- `src/main.rs`: parse `--v1`, `--v2`, `--out` (`PathBuf`), `--retag` (bool),
  `--dry-run` (`Option<u32>`, max body pages), `--pages` (`Option<String>`, a
  source-qualified range like `v1:40-60`) with `clap`; call into the library. A
  restricted run (`--dry-run` or `--pages` set) targets the separate
  `plutarch.dryrun.tags.json`, mirroring the prior art's `sidecar_path_for`; a full
  run targets `plutarch.tags.json`. All real logic lives in `src/lib.rs`.
- `src/lib.rs`: declare modules filled by later phases, each an empty file with a
  short present-tense module doc: `pub mod secrets; pub mod render; pub mod pages;
  pub mod tags; pub mod llm; pub mod verify; pub mod assemble; pub mod volumes;
  pub mod latex; pub mod signatures; pub mod compile;`.

**Tests:** none beyond compilation.
**LLM verification:** `cargo build`; `cargo run -- --v1 Ploutarchou_Bioi_v1.pdf
--v2 Ploutarchou_Bioi_v2.pdf --out build/` prints the parsed args; `cargo fmt
--all --check` and `cargo clippy --all-targets -- -D warnings` clean.
**Human verification:** None (skeleton only).

### Step 1.2 — Secrets loading and ignore rules

- `src/secrets.rs`: `Secrets { anthropic_api_key: String }` (`Deserialize`) and
  `fn load(path: &Path) -> anyhow::Result<Secrets>`; clear error if missing.
- Create a `.gitignore`-style ignore note is unnecessary (no git); instead ensure
  `secrets.json` and `build/`, `target/` are documented in the README as local and
  not shared. Commit `secrets.example.json` with `{"anthropic_api_key":"sk-ant-…"}`.

**Tests:** `secrets_load_reads_key`; `secrets_load_missing_file_errors`.
**LLM verification:** `cargo test secrets`.
**Human verification:** None.

---

## Phase 2: PDF rasterization and page hashing

**Suggested implementer: Sonnet.** Thin wrapper over `pdftoppm`, well-specified.

### Step 2.1 — Render a source page to PNG and hash it

In `src/render.rs`:

- A `Pdf` handle wrapping a source path and its page count (from `pdfinfo` or a
  one-time `pdftoppm` probe).
- `fn render_page(pdf: &Path, page: u32, dpi: u32, out_dir: &Path) -> Result<PathBuf>`
  shelling out to `pdftoppm -png -f page -l page -r dpi …`, returning the produced
  PNG path. Keep the poppler binary path configurable (env var
  `PLUTARCH_PDFTOPPM`, default to `pdftoppm` on `PATH`).
- `fn page_image_hash(png: &Path) -> Result<String>`: SHA-256 of the PNG bytes,
  hex — stored as a per-page **integrity** field, not the cache key. The cache key
  is `(source_sha256, source_page, dpi)`, so the pipeline skips an already-tagged
  page **without** rendering it; rendering happens only for pages that still need
  tagging.
- `const TRANSCRIBE_DPI: u32 = 300;` (tunable).

**Tests:** `render_page_produces_png` (render page 1 of a tiny fixture or the real
v1 p1; assert the file exists and is non-empty — gated behind the binary being
present); `page_image_hash_is_stable` (hash the same bytes twice → equal).
**LLM verification:** `cargo test render`; manually render v1 p40 and confirm a
PNG appears.
**Human verification:** Open one rendered PNG and confirm the Latin column is
legible at the chosen DPI.

---

## Phase 3: Anthropic client with image support

**Suggested implementer: Opus.** External API shape, image blocks, error handling.

### Step 3.1 — Messages-API client

In `src/llm/mod.rs`:

- Reuse the prior art's `Client` (`new(api_key)`) and the free functions
  `build_body` / `extract_text` **as-is** — `extract_text` already returns the
  first `type=="text"` block past a leading `thinking` block and needs no change.
- The only extension for images: change the message content from `user: &str` to a
  `&[ContentBlock]`, where `ContentBlock` is `Text(String)` or
  `Image { png_base64: String }`, and have `build_body` emit the mixed content
  array. Keep the existing **`message` / `message_thinking` split** (do not add a
  `thinking: bool` parameter); the Opus arbiter calls `message_thinking`.
- **Capture token usage.** Have `message` / `message_thinking` return a
  `Completion { text: String, usage: Usage }` instead of a bare `String`, where
  `Usage { input_tokens: u64, output_tokens: u64 }` (defined in Step 3.2). Add a
  pure `fn extract_usage(reply_json) -> Usage` that reads the reply's top-level
  `usage.input_tokens` / `usage.output_tokens` (defaulting a missing field to 0),
  alongside the unchanged `extract_text`. Callers do
  `let c = client.message(…)?; let text = c.text; tracker.record(model, page, c.usage);`.
- Add the `SONNET = "claude-sonnet-5"` constant alongside the existing `HAIKU` /
  `OPUS`.

**Tests:** `build_body_places_image_and_text_blocks` (the new mixed content array);
`extract_usage_reads_token_counts`; `extract_usage_defaults_missing_to_zero`; the
existing `build_body_places_fields_correctly` / `build_body_enables_thinking` stay
green; `extracts_first_text_block_past_thinking` already covers `extract_text`
unchanged. No network in tests.
**LLM verification:** `cargo test llm`; `cargo build`.
**Human verification:** With a real `secrets.json`, a `#[ignore]`d test makes one
tiny Haiku call on a rendered page and prints the reply **and its token usage**, to
confirm auth, image upload, and usage parsing work. Keep it ignored afterward.

### Step 3.2 — Usage tracking and progress-bar instrumentation

In `src/llm/instrument.rs` (a submodule of `llm`; add `pub mod instrument;` to
`src/llm/mod.rs` — the top-level module list in `src/lib.rs` is untouched):

- `struct Usage { input_tokens: u64, output_tokens: u64 }` (derives `Debug, Clone,
  Copy, Default, PartialEq, Eq`) with `fn total(&self) -> u64` and an `AddAssign`
  impl so per-page sums accumulate.
- `type PageId = (usize, u32)` — source index + 1-based source page, matching the
  `PageRecord` key — identifies which page a call was spent on.
- `struct UsageTracker` wrapping a `RefCell<Vec<CallRecord>>` (`CallRecord { model:
  String, page: PageId, usage: Usage }`), so it threads through the orchestration as
  `&UsageTracker` without borrow conflicts against the `&mut Sidecar`. Method
  `fn record(&self, model: &str, page: PageId, usage: Usage)` appends one record per
  API call.
- `fn report(&self) -> UsageReport`: group calls by model; for each model compute
  (a) call count, (b) total tokens, and (c) **per-page** token totals — summing the
  usage of every call to that model on the same page — then from that vector of
  per-page totals compute median, max, and min. `UsageReport` holds one
  `ModelUsage { model, calls, total_tokens, pages, median_per_page, max_per_page,
  min_per_page }` per model that was called (models never called are omitted).
- `impl Display for UsageReport`: print one aligned row per model — calls, total
  tokens, and median / max / min tokens per page — under a header, so Phase 13 can
  `println!("{report}")`.
- `fn progress_bar(len: u64, label: &str) -> indicatif::ProgressBar`: build a bar
  with one shared `ProgressStyle` (a `{prefix} [{bar}] {pos}/{len} ETA {eta}`
  template, `label` as the prefix) so classification, transcription, and
  arbitration all render the same tqdm-style bar. Callers `pb.inc(1)` per page and
  `pb.finish_and_clear()` at the end.

**Tests:** `usage_report_counts_calls_per_model`;
`usage_report_computes_per_page_median_max_min`;
`usage_report_sums_multiple_calls_on_one_page` (two calls to one model on one page
collapse to a single per-page total); `usage_report_omits_uncalled_models`;
`progress_bar_has_expected_length`. No network.
**LLM verification:** `cargo test instrument`; `cargo build`.
**Human verification:** None (exercised live in later phases).

---

## Phase 4: Sidecar data model

**Suggested implementer: Sonnet.** serde structs and I/O, well-specified.

### Step 4.1 — Define and (de)serialize the sidecar

In `src/tags.rs`, define (each struct derives `Debug, Clone, PartialEq, Eq,
Serialize, Deserialize`; each enum additionally uses `#[serde(rename_all =
"snake_case")]` for stable JSON — matching the prior art, and required for the
roundtrip-equality test to compile):

```rust
pub struct Sidecar {
    pub schema_version: u32,              // start at 1
    pub sources: Vec<SourcePdf>,          // path, page_count, file_sha256
    pub front_matter: FrontMatter,        // derived bibliographic metadata only (titles, editor, date)
    pub pages: Vec<PageRecord>,           // every processed source page, in order
    pub lives: Vec<LifeBoundary>,         // id, title, kind, first/last page index
}
pub struct PageRecord {
    pub source: usize,                    // index into `sources`
    pub source_page: u32,                 // 1-based page in that PDF
    pub image_sha256: String,             // cache key
    pub role: PageRole,                   // Skip | FrontImage | FrontProse | Body
    pub pass_a: Option<PageDoc>,          // Sonnet transcription (None if Skip)
    pub pass_b: Option<PageDoc>,          // Haiku transcription
    pub status: PageStatus,               // Agreed | Arbitrated | Escalated | NotBody
    pub flags: Vec<VerifyFlag>,           // continuity/ocr findings
    pub result: Option<PageDoc>,          // authoritative transcription
}
pub struct PageDoc { pub blocks: Vec<Block> }
pub struct Block {
    pub kind: BlockKind,                  // Heading | Chapter | Section | Paragraph
    pub roman: Option<String>,            // chapter numeral, as printed
    pub section_no: Option<u32>,          // parenthetical section number
    pub continues_previous: bool,         // body text continues prior page/para
    pub text: String,                     // verbatim Latin (+ Greek for FrontProse); a Verse block keeps line breaks as \n
}
pub enum PageRole { Skip, FrontImage, FrontProse, Body }
pub enum PageStatus { NotBody, Agreed, Arbitrated, Escalated }
pub enum BlockKind { Heading, Chapter, Section, Paragraph, Verse }
pub struct LifeBoundary { pub id: String, pub title: String, pub kind: LifeKind,
                          pub group: u32, pub first_page: usize, pub last_page: usize }
pub enum LifeKind { Life, Comparatio }
pub struct VerifyFlag { pub kind: FlagKind, pub detail: String }
```

The **front-matter prose** (Monitum, Praefatio) is not duplicated in `FrontMatter`:
its transcription is the `result` of the corresponding `FrontProse` `PageRecord`s —
the single source of truth. `FrontMatter` holds only the derived bibliographic
strings used on the re-typeset title page. The per-page **cache key** is
`(sources[record.source].file_sha256, record.source_page, TRANSCRIBE_DPI)`;
`image_sha256` is kept only as an integrity check on the rendered PNG.

Add `fn load(path)`, `fn save(&self, path)` (pretty JSON), and
`fn page(&self, source: usize, source_page: u32) -> Option<&PageRecord>` for
resumable lookups (no rendering needed), plus `fn is_source_stale(&self, sources)
-> bool` comparing stored file hashes.

**Tests:** `sidecar_roundtrips_through_json`; `page_lookup_by_source_and_page`;
`is_source_stale_detects_file_change`.
**LLM verification:** `cargo test tags`.
**Human verification:** None.

---

## Phase 5: Layout spikes (throwaway) — de-risk LaTeX before generating

**Suggested implementer: Opus.** Requires judgment on whether the layout behaves.

### Step 5.1 — Hand-build the trim-size inner sample

By hand, write `spikes/inner_sample.tex`: a 10.5 × 16 cm single-column document in
**Old Standard via fontspec** (lualatex), with roman-numbered front matter (a
re-typeset title page with a placeholder embedded image box for the vignette, and
a short Monitum-style page mixing Latin and **polytonic Greek**), then
arabic-numbered body with two short "Lives" and a "Comparatio", each starting a
fresh page, reproducing inline Roman chapter numbers and parenthetical section
numbers, and including a short offset **verse** quotation to prove the verse
environment. Compile with `lualatex`.

**LLM verification:** PDF builds without errors; `pdfinfo` shows the expected page
count; a later Life starts on a fresh page.
**Human verification (critical):** Real-eyes check — Old Standard renders both
Latin and the Greek fragments; trim geometry is 10.5 × 16 cm; front matter roman,
body arabic; numbering inline as printed. **If Old Standard cannot render the
polytonic Greek cleanly under lualatex, stop and pick a fallback (e.g. a dedicated
Greek font via `\newfontfamily`) before proceeding.**

### Step 5.2 — Verify the existing imposition wrapper on a monolingual inner

The imposition layer already exists and is language-agnostic — it only places
inner-PDF pages two-up. **Reuse the prior art's `latex::write_imposition_wrappers`
and its emitted `\imposedsheetside` / `\drawsewmarks` macros and sew constants
(`SEW_MARGIN_CM`, `SEW_TAPES`, `SEW_HALFTAPE_CM`, `\sewdot`) verbatim.** This step is
a *verification*, not a hand-build: run `write_imposition_wrappers` against the Step
5.1 `inner_sample.pdf`'s real page count and compile the wrapper to confirm it drives
a single-column inner correctly (imposition formula: sheet *i* outer `34−2i | 2i−1`
fold on, inner `2i | 33−2i` fold off; cut border only on sheet 1 outer).

**LLM verification:** wrapper builds; 16 US Letter landscape pages; page 1 carries
the cut border + fold line + sew marks.
**Human verification (critical):** Real-eyes check — two pages per side; fold line
and 6 symmetric sew dots only on outer/recto sides (between 1|32, absent 2|31);
cut border only on the first recto; folded order reads in sequence; even/odd →
left/right puts spine margins toward the fold. Optionally print, fold, and sew one
signature.

---

## Phase 6: Page-role classification

**Suggested implementer: Opus.** Vision-assisted judgment with a cached, editable map.

### Step 6.1 — Classify every source page

In `src/pages.rs`:

- `fn classify(client, pdf_handles, sidecar, dpi, tracker: &UsageTracker) ->
  Result<()>`: vision-classify only the **front- and back-matter regions** (the
  first and last ~25 pages of each PDF). The contiguous middle is marked `Body`
  **without any API call** — those pages are unambiguously body — which avoids
  ~1300 needless Haiku calls. For a page in a boundary region not already
  classified, render a **low-DPI thumbnail** and ask Haiku (cheap) to label it
  `Skip | FrontImage | FrontProse | Body` with a one-line reason, forced via schema;
  store `role` on the `PageRecord` and `tracker.record(HAIKU, page_id, usage)`.
  Deterministic shortcuts first: near-uniform (all-dark/all-light) thumbnails and
  detected marbling are `Skip` without an API call, using a simple
  luminance-histogram heuristic. The body region is bounded by the first/last `Body`
  page the boundary scans find. Wrap the boundary-region scan in an
  `instrument::progress_bar` (one tick per boundary page, whether classified by API
  or deterministic shortcut).
- The map is cached and **hand-editable** in the sidecar; a `--reclassify` flag (or
  `--retag`) forces a re-run. Document that the human may correct a role directly in
  the JSON.

**Tests:** `uniform_thumbnail_classified_skip_without_api` (stubbed client panics
if called); `middle_pages_marked_body_without_api`; `classify_is_cached` (a populated
role is not re-requested); `classify_round_trips_roles_in_sidecar`;
`classify_records_usage_per_classified_page` (stubbed client returns a canned
`Usage`; assert the tracker logged one Haiku call per boundary page that hit the
API and none for deterministic skips).
**LLM verification:** `cargo test pages`.
**Human verification:** Run classification on v1 pp1–20 and v2 pp1–20; confirm the
inventory matches the known front matter (v1: p1 FrontImage, p8/p10/p12–13
FrontProse, p14+ Body; marbling/blanks Skip).

---

## Phase 7: Dual-pass transcription

**Suggested implementer: Opus.** Prompt design, schema, orchestration — the core.

### Step 7.1 — Per-page structured transcription prompt

In `src/llm/transcribe.rs`:

- `fn transcribe_page(client, model, png_base64, mode) -> Result<PageDoc>` where
  `mode` is `BodyLatinOnly` or `FrontProseFull`.
  - **BodyLatinOnly** system/user prompt: "This page has Greek in the left column
    and Latin in the right. Transcribe **only the Latin (right) column**, exactly
    as printed — do **not** normalize spelling, u/v, i/j, ligatures, the long s, or
    punctuation. Use the Greek only to disambiguate unclear Latin. Reproduce Roman
    chapter numbers and parenthetical section numbers. Mark embedded **verse**
    (line-broken quotations) as `Verse` blocks, preserving their line breaks exactly.
    Mark a block that continues text from the previous page." Forced to the `PageDoc`
    schema.
  - **FrontProseFull**: "Transcribe the entire page faithfully, **including Greek
    fragments**, preserving order and numbering." Forced to the same schema (Greek
    lives in `text`).
- A pure `fn parse_pagedoc(reply_json) -> Result<PageDoc>`.

**Tests:** `parse_pagedoc_reads_blocks_and_numbering`;
`parse_pagedoc_rejects_malformed`; `parse_pagedoc_reads_verse_line_breaks`;
`body_prompt_mentions_latin_only_and_no_normalize`; `body_prompt_requests_verse_marking`
(assert the constructed prompt string contains the faithfulness and verse
instructions). No network.
**LLM verification:** `cargo test transcribe`.
**Human verification:** Transcribe v1 p40 with Sonnet and with Haiku; print both
`PageDoc`s beside the scan and confirm Latin-only, numbering preserved, no
normalization.

### Step 7.2 — Run both passes, resumably, over the body

- `fn transcribe_all(client, pdf_handles, sidecar, limit, page_range, tracker:
  &UsageTracker) -> Result<()>`: for each `Body` **and** `FrontProse` page without a
  cached `result`, render at `TRANSCRIBE_DPI`, run pass A (Sonnet) and pass B
  (Haiku), store both `PageDoc`s, and `tracker.record` the Sonnet and Haiku usage
  against that page. Body pages use the `BodyLatinOnly` prompt; `FrontProse` pages
  use `FrontProseFull` — but both go through the same dual-pass + arbitration path
  (Phase 8), so the Monitum/Praefatio Greek gets the same cross-model verification
  as the body (the Latin-only Tesseract check is simply skipped for `FrontProse`).
  Honor `--dry-run N` (first N body pages) and `--pages v1:A-B`; restricted runs
  target the dryrun sidecar. Drive an `instrument::progress_bar` over the pages
  needing transcription (one tick per page). Save the sidecar incrementally (after
  each page) so the run is resumable.

**Tests:** `transcribe_all_skips_cached_pages`; `dry_run_limits_body_pages`;
`frontprose_runs_dual_pass_like_body`; `restricted_run_uses_dryrun_sidecar`;
`transcribe_all_records_sonnet_and_haiku_usage_per_page` (stubbed client returns
canned `Usage`; assert two calls — one Sonnet, one Haiku — logged per transcribed
page). Use a stubbed client.
**LLM verification:** `cargo test transcribe`.
**Human verification:** `--dry-run 3` against real PDFs produces three body pages
of dual-pass transcription in the sidecar; inspect them.

---

## Phase 8: Verification and arbitration

**Suggested implementer: Opus.** Diffing, deterministic checks, Opus escalation.

### Step 8.1 — Cheap deterministic checks

In `src/verify.rs`:

- `fn section_continuity(prev_tail, page: &PageDoc) -> Vec<VerifyFlag>`: confirm
  parenthetical section numbers are contiguous and Roman chapter numbers
  non-decreasing across the page boundary; flag gaps/resets that are not a Life
  start.
- `fn ocr_crosscheck(png, page: &PageDoc) -> Result<Vec<VerifyFlag>>`: run
  Tesseract `-l lat` on the page (or the right-column crop), compare char and line
  counts to the accepted text; flag a divergence beyond a tunable threshold. This is
  **body-only** — it is skipped for `FrontProse` pages, whose Greek the Latin OCR
  model cannot read. Tesseract path configurable (`PLUTARCH_TESSERACT`).

**Tests:** `continuity_flags_missing_section`; `continuity_allows_life_reset`;
`ocr_crosscheck_flags_large_divergence` (with a synthetic/stubbed OCR count).
**LLM verification:** `cargo test verify`.
**Human verification:** Run the checks over the dry-run pages; confirm a
hand-corrupted `PageDoc` (deleted section) is flagged.

### Step 8.2 — Diff, arbitrate, finalize

In `src/verify.rs` + `src/llm/arbitrate.rs`:

- `fn normalize_for_diff(doc) -> String`: collapse whitespace/line-wrap; keep
  spelling/punctuation/orthography (so u/v, long-s, ligature differences remain
  real disagreements).
- `fn reconcile_page(client, record, png, tracker: &UsageTracker) -> Result<()>`:
  if pass A and pass B normalize-equal **and** no cheap-check flag fires →
  `status = Agreed`, `result = pass_a`, and **no** API call (nothing recorded).
  Otherwise call **Opus** (`arbitrate.rs`) with the image, a note that the Greek is
  available context, and both candidate `PageDoc`s, asking for the correct faithful
  transcription; `tracker.record(OPUS, page_id, usage)`; set `result` and
  `status = Arbitrated` (disagreement) or `Escalated` (agreed-but-flagged). Opus
  uses adaptive thinking. The reconcile loop that calls `reconcile_page` over the
  pages drives an `instrument::progress_bar` (one tick per page).
- `fn summary(sidecar) -> ReconcileSummary`: counts of agreed/arbitrated/escalated
  with page refs, for the log.

**Tests:** `agreement_without_flags_takes_pass_a_without_opus` (stub panics if Opus
called; asserts the tracker logged zero Opus calls for an agreed page);
`disagreement_invokes_arbiter`; `agreed_but_flagged_escalates_to_arbiter`;
`arbitration_records_opus_usage` (an arbitrated page logs exactly one Opus call);
`summary_counts_statuses`.
**LLM verification:** `cargo test verify`; `cargo test arbitrate`.
**Human verification:** On the dry-run pages, confirm agreed pages skip Opus and a
seeded disagreement routes to Opus; read the printed summary.

---

## Phase 9: Assembly into Lives and documents

**Suggested implementer: Opus.** Cross-page stitching and boundary detection.

### Step 9.1 — Stitch pages and detect Life boundaries

In `src/assemble.rs`:

- `fn detect_lives(sidecar) -> Vec<LifeBoundary>`: walk body pages in order; a
  `Heading` block whose text names a new Life/Comparatio opens a new unit (running
  heads cross-check the name). Assign `group` by binding each `Comparatio` to the two
  preceding Lives it names in its title (e.g. *Comparatio Thesei et Romuli*) rather
  than assuming a rigid (Life, Life, Comparatio) triple — and handle a Life that has
  no following Comparatio (some Lives are unpaired). Cache into `Sidecar::lives`;
  allow hand override.
- `fn assembled_text(sidecar, life) -> Vec<Block>`: concatenate the `result` blocks
  of a Life's pages in order, **rejoining** a block with `continues_previous = true`
  onto the previous block's paragraph (so a sentence split across a page break is
  made whole). Produces the logical, page-independent block stream the LaTeX layer
  renders.

**Tests:** `detect_lives_groups_pair_and_comparatio`;
`detect_lives_binds_comparatio_to_named_lives`;
`detect_lives_handles_life_without_comparatio`;
`detect_lives_finds_boundaries_from_headings`;
`assembled_text_rejoins_continued_paragraph`;
`assembled_text_preserves_numbering_order`.
**LLM verification:** `cargo test assemble`.
**Human verification:** Print the detected Life list (ids, titles, groups, page
spans) for the dry-run range; confirm pairs + Comparationes group correctly.

---

## Phase 10: Volume splitting

**Suggested implementer: Opus.** Balancing heuristic with a hard constraint.

### Step 10.1 — Estimate pages and split on pair-group boundaries

In `src/volumes.rs`:

- `fn estimate_pages(life, fonts) -> u32`: estimate trim pages for a Life/Comparatio
  from character counts, **reusing** the prior art's tunable constants
  `CHARS_PER_LINE`, `LINES_PER_PAGE`, and `HEADING_LINES` (re-tuned for the
  single-column Old Standard page); each unit starts a fresh page. Follow
  `main.rs::estimate_inner_pages` for folding in per-volume front matter and the
  `FLYLEAVES_PER_END` blank leaves, then `signatures::pad_to_multiple_of_32`.
- `fn split_into_volumes(lives, target_pages, max_pages) -> Vec<Vec<String>>`:
  greedily accumulate **whole pair-groups** (never splitting a group) until the next
  group would exceed `max_pages`; prefer cutting near `target_pages` for balance.
  **Force a volume break at the v1→v2 source boundary** — split each source volume's
  pair-groups independently and concatenate, so v2 always starts a new output TOMUS.
  Account for per-volume front matter (each source volume's front matter is charged
  to its first output TOMUS) and the two front + two back blank leaves in the
  budget. `TARGET_PAGES = 400`, `MAX_PAGES = 500` (physical pages).
- **Boundary invariant.** The forced v1→v2 break is only safe because the physical
  boundary coincides with a pair-group boundary: v1 ends with *Comparatio Cimonis
  et Luculli* (closing the complete Cimon–Lucullus group) and v2 opens the
  Nicias–Crassus group. Assert this at runtime — v1's last detected unit is a
  `Comparatio` closing a group, and v2's first unit opens a new group — and **error
  loudly** if a future re-scan or edition violates it, rather than silently
  splitting a pair across the forced break.

**Tests:** `split_never_breaks_a_pair_group`; `each_volume_within_max_pages`;
`volumes_are_roughly_balanced`; `front_and_back_blank_leaves_charged_to_each_volume`;
`split_forces_break_at_v1_v2_boundary`;
`split_errors_if_source_boundary_bisects_pair_group`.
**LLM verification:** `cargo test volumes`.
**Human verification:** Print computed volume boundaries (which pair-groups land in
each TOMUS) and estimated page counts; sanity-check volumes near 400 pp.

---

## Phase 11: LaTeX generation — inner

**Suggested implementer: Opus.** Ties assembly to the spiked trim-size layout.

### Step 11.1 — Emit the per-volume inner `.tex`

In `src/latex.rs`, generate `tomus_N_inner.tex` per volume using the Phase 5.1
layout:

- **Preamble:** 10.5 × 16 cm geometry, Old Standard via fontspec, roman front-matter
  / arabic body page numbering, tunable font-size constant.
- **Front leaves:** two blank leaves.
- **Front matter (roman), per source volume on that source's first output TOMUS:**
  the Google/QR page and the title-page vignette as embedded scans. Since no raster
  crop tool is installed, render the **full** source page to PNG (poppler, print DPI)
  into `out_dir` and crop at include time with graphicx
  `\includegraphics[trim=L B R T,clip]` (or `viewport`) — no external image edit. The
  title page and the source's prose piece — the **Monitum** for v1, the **Praefatio**
  for v2 — are re-typeset (Latin + Greek). Insert blank pages so each prose piece
  starts recto/odd. Later output volumes of the same source open directly with their
  `TOMUS N` label page.
- **Volume label page:** `TOMUS N` (roman numeral in the label text).
- **Body (arabic):** for each Life/Comparatio in the volume, force a fresh page,
  print the heading, then render `assembled_text` blocks with inline Roman chapter
  numbers and parenthetical section numbers, **verbatim** (no normalization). Render
  `Verse` blocks in an offset, line-broken verse environment (LaTeX `verse`, or an
  indented block), preserving the transcribed line breaks.
- Escape LaTeX specials (`& % # _ $ ~ ^ \`) in body text while preserving accented
  characters, the long s, ligatures, and Greek (lualatex passes UTF-8 through).
- `fn write_inner_volumes(sidecar, volumes, out_dir, fonts) -> Result<Vec<PathBuf>>`;
  it also renders the needed front-matter page images (QR, vignette) into `out_dir`
  via `render` and references them with trimmed `\includegraphics`.

**Tests:** `latex_specials_escaped_text_preserved` (orthography/long-s/ligatures
survive; `& % # _ $` escaped); `each_unit_starts_fresh_page_command_present`;
`frontmatter_roman_body_arabic_numbering`; `monitum_renders_latin_and_greek`;
`praefatio_renders_on_v2_first_output_tomus`; `verse_blocks_render_offset_preserving_line_breaks`;
`qr_page_embedded_as_image`; `tomus_label_page_present`.
**LLM verification:** `cargo test latex`; run end-to-end offline (cached sidecar)
and confirm `tomus_N_inner.tex` files appear.
**Human verification (critical):** Compile `tomus_1_inner.tex` with `lualatex`;
real-eyes check against the scans — front matter faithful, Monitum's Greek intact,
Lives on fresh pages, numbering inline-as-printed, no normalization, trim size
correct.

---

## Phase 12: Imposition wrapper, signatures, and two-stage compile

**Suggested implementer: Opus.** Reuses prior-art imposition; wires the page-count readback.

### Step 12.1 — Signature padding arithmetic

The inner document self-pads to a whole signature at its end (a LaTeX helper reads
`\value{page}` and emits the right number of blank pages), guaranteeing a blank final
verso with the two back blank leaves (`FLYLEAVES_PER_END`) ahead of the padding. For
page-count estimates the Rust side **reuses** the prior art's `src/signatures.rs`
verbatim — `pad_to_multiple_of_32` and `PAGES_PER_SIGNATURE` already exist.

**Tests:** the existing `pad_to_multiple_of_32_from_various_counts` (1→32, 32→32,
33→64, 500→512) carries over unchanged.
**LLM verification:** `cargo test signatures`.
**Human verification:** None (arithmetic).

### Step 12.2 — Emit the imposition wrapper

**Reuse** the prior art's `latex::write_imposition_wrappers(volumes,
inner_page_counts, out_dir)` and its emitted `\imposedsheetside` / `\drawsewmarks`
macros and sew constants **verbatim** — the wrapper is monolingual-agnostic (it only
places inner pages two-up). No new imposition code is written here; this step just
calls it with the padded inner page counts from Step 12.3.

**Tests:** the prior art's imposition tests (`imposition_orders_pages_for_signature`,
`fold_and_sew_only_on_outer_sides`, `cut_border_only_on_first_recto_of_each_signature`,
`pages_past_inner_count_blank`) carry over unchanged.
**LLM verification:** `cargo test latex`.
**Human verification (critical):** Two-stage compile of volume 1 (inner, then
wrapper) and real-eyes check the imposed PDF as in Step 5.2.

### Step 12.3 — Two-stage compile driver

**Reuse** the prior art's `src/compile.rs`: `run_lualatex`, `compile(dir, file,
passes)`, `parse_page_count`, `inner_page_count`, and `lualatex_available`. Because
`reledpar` is gone the inner compiles in **one** pass — call `compile(out, inner, 1)`
(drop `RELEDPAR_PASSES`). The inner document self-pads to a whole signature at its end
(Step 12.1), so `inner_page_count` already returns a multiple of 32. The driver:
assert `lualatex_available()` and **error** if not (we assume lualatex is present);
compile inner (1 pass) → `inner_page_count` → emit the wrapper (Step 12.2) with that
count → `compile(out, wrapper, 1)`. lualatex path configurable (`PLUTARCH_LUALATEX`).

**Tests:** the existing `parse_page_count_*` tests carry over; add
`inner_compiles_in_one_pass` (asserts the driver requests a single inner pass). The
compile calls themselves are gated behind `lualatex_available()`.
**LLM verification:** `cargo test compile`.
**Human verification:** Full build of volume 1 produces `tomus_1.pdf` with the
expected page count (inner padded /2 per sheet side).

---

## Phase 13: Pipeline wiring, dry-run, and README

**Suggested implementer: Opus.** Top-level orchestration and docs.

### Step 13.1 — Top-level pipeline

In `src/lib.rs`: `fn run(args) -> Result<()>` threading: pick the sidecar path
(`plutarch.dryrun.tags.json` for a restricted run, else `plutarch.tags.json`) →
load/resume → create a single `instrument::UsageTracker` → classify pages (Phase 6)
→ transcribe body + front prose (Phase 7) → verify + arbitrate (Phase 8) — all three
sharing that tracker → assemble + detect lives (Phase 9) → split volumes (Phase 10)
→ emit inner (Phase 11) → two-stage compile + impose (Phase 12, which errors if
`lualatex` is absent) → write the reconcile summary log → `println!("{}",
tracker.report())`. `--retag` forces re-tagging; `--dry-run N` and `--pages v1:A-B`
restrict scope and target the dryrun sidecar; with a complete fresh sidecar, the run
makes zero network calls (and the usage report shows zero calls for every model).

**Tests:** `run_offline_with_fresh_sidecar_makes_no_network_calls` (stub client
panics on use); `run_offline_usage_report_is_empty` (the report over a zero-call run
lists no models / zero calls); `dry_run_end_to_end_produces_short_pdfs` (gated on
tooling).
**LLM verification:** `cargo test`; a `--dry-run 4` full build.
**Human verification (critical):** `--dry-run` build end-to-end produces a small
inner + imposed PDF; sanity-read it. Watch the progress bars advance during the
LLM passes, and confirm the end-of-run usage report shows plausible per-model call
counts and token totals for the pages transcribed.

### Step 13.2 — README

Write `README.md`: what the tool does; dependencies (poppler, MiKTeX + the Old
Standard font, Tesseract with `lat`, Rust) and where each is expected / how to
point the env-var overrides at them; the `secrets.json` format; example invocations
(full build, `--dry-run N`, `--retag`); the two-stage compile and output layout; the
progress bars shown during LLM passes and the end-of-run usage report (per-model
call counts and token totals/median/max/min per page); and the note that the sidecar
is committed while `build/`, `target/`, and `secrets.json` are local.

**LLM verification:** none (docs).
**Human verification:** Follow the README from scratch and confirm a `--dry-run`
build succeeds.

---

## Cross-cutting requirements

- Every Rust-touching step ends green on `cargo fmt --all --check`,
  `cargo clippy --all-targets -- -D warnings`, and the relevant `cargo test`; the
  full `cargo test` is green before a phase is declared done. There is no CI — these
  run locally.
- **No git operations** (the project uses OneDrive history). No `.github/`.
- The scanned PDFs are never modified; all derived data lives in the sidecar or
  under `build/`.
- The transcribed Latin (and the Monitum's Greek) is reproduced **verbatim** — no
  normalization anywhere in the generator; corrections are deliberate, noted edits
  to the sidecar.
- LLM calls happen only during classification/transcription/arbitration and only
  when the sidecar is missing/stale or `--retag` is passed; every other run is
  offline.
- No code comment references this plan, its phases, or `llm_artifacts/`.
```
