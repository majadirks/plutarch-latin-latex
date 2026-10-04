# Auto-populating the facsimile unit table

**Authoring date:** 2026-10-03

## Context

The facsimile build path (`2026_10_03_FACSIMILE_READER_PLAN.md`) drives body
pagination from a unit table — `facsimile_units.json`, one row per Life /
Comparatio: `(id, latin_title, greek_title, kind, group, source, start_page)`. That
plan made the full table a **hand-authored human deliverable** (its Phase 5.2). In
practice that will not happen: hand-transcribing ~46 Lives + ~19 Comparationes
across two volumes, with exact source start pages, is precisely the tedious,
error-prone work the project exists to avoid.

This plan replaces the manual step with **deterministic auto-detection** from the
source PDFs. It hinges on two facts established by the spike
(`spikes/facsimile_unit_detect_spike.py`) run against the real scans:

1. **The source PDFs carry an embedded OCR text layer.** `pdftotext -f N -l N`
   returns usable Latin text for every page, *including the running head*, with no
   vision/network call. On an opening page the display title is present too
   (`THESEUS.`, `ROMULUS.`, `SOLON.`); on a Comparatio the running head reads
   `COMPARATIO THESEI CUM ROMULO` or `LYCURGI ET NUMÆ COMPARATIO` (word order and
   line-wrapping vary).
2. **The running head names every body page's current unit.** Tracing the running
   head down a page range yields the unit sequence directly; a unit's pages form a
   contiguous run, so unit boundaries are where the running-head name changes.

The spike also established what does **not** work: the large display-title band is
**not** a reliable discriminator. Band height over the page's median body-line
height was ~2.0 for Theseus/Romulus but only ~1.0–1.1 for the Comparationes, while
an interior body page (v1 p20, a verse block) scored 5.5 — a false positive. So the
opening cannot be found by type size alone; the **OCR text layer is the primary
signal**, with the display-title band only a secondary tie-breaker.

Polytonic Greek OCR from an 1857 scan is unreliable, so the **Greek titles,
grouping, kind, and canonical order are not read from the page at all**: they come
from a small built-in **canonical corpus** of Plutarch's Parallel Lives in the
Didot edition's order. The detector's only job is to recognize *which* known unit a
page belongs to (robust even with noisy Latin OCR — `ROMVLVS`/`ROMULUS` both match
canonical `ROMULUS`) and *where* it starts.

This plan also **folds in the still-unimplemented tail of
`2026_10_03_FACSIMILE_READER_PLAN.md`**: Phases 1–4 there (scaffolding through page
frame / volume split / imposition) are implemented and green; its Phase 5.2 (the
manual unit table) is retired by this plan's auto-detection, and its remaining Steps
5.1 (front-matter hybrid) and 5.3 (README) are carried here as Phase 5, so one plan
takes the facsimile reader to a complete end-to-end build.

## Current state of the world

- **`src/facsimile/units.rs`** — `Unit { id, latin_title, greek_title, kind, group,
  source, start_page, skip_top_bands }`, `UnitRange { unit, end_page }`,
  `load_units(path)` (reads + validates `facsimile_units.json`), `ranges(units,
  source_last_body_page)`.
- **`src/facsimile/mod.rs`** — `run_facsimile` calls `units::load_units` and
  `units::ranges`; `select_ranges` derives per-unit ranges. `last_body` per source
  is currently the PDF page count (an upper bound; the real last body page is still
  approximate — see this plan's Phase 4).
- **`facsimile_units.json`** — a committed seed with three real rows (Theseus @ v1
  p14, Romulus, Comparatio Thesei et Romuli). Hand-authored; incomplete.
- **`src/render.rs`** — `Pdf::open` (page count via `pdfinfo`), `render_page`
  (poppler `pdftoppm`). No `pdftotext` wrapper exists yet.
- **`src/verify.rs`** — `pub fn fold(text: &str) -> String` already folds
  orthography to a letters-only form (lowercase; `u→v`, `i→j`; `ſ→s`; `æ→ae`,
  `œ→oe`; strips whitespace/punctuation/digits). **Reuse it** for token folding
  rather than writing a second folder.
- **`src/assemble.rs`** — `life_names_in`, `names_any`, `common_prefix_len` (all
  private) already match a Comparatio title's genitive name tokens to the Lives it
  pairs, by a case-folded common prefix of ≥4 letters (so `THESEI` matches
  `THESEUS`). This is the project's title-matching idiom; the detector should reuse
  it (lift the shared helper to `pub(crate)`) instead of a parallel exact matcher.
- **`src/latex.rs`** — `title_page`, `front_images`, `front_prose` (all private)
  typeset the re-typeset front matter; `inner_document` composes them. The facsimile
  needs the same output as its `FacsimileVolume.front_matter` string (Phase 5).
- **`src/facsimile/columns.rs` / `lines.rs`** — Latin-column detection and line
  banding over a `GrayImage`; available as a last-resort visual tie-breaker for an
  ambiguous opening, but **not** used in the primary (text-layer) detection path.
- **`src/facsimile/assemble.rs`** — `FacsimileVolume { …, front_matter: String }`
  already carries a slot for the re-typeset front matter, prepended when
  `is_first_of_source`; it is empty today (Phase 5 fills it).
- The spike `spikes/facsimile_unit_detect_spike.py` validates OCR-title extraction
  and documents the band-height heuristic's unreliability. Retain it as the dated
  record of why this design is OCR-first.

There is no `pdftotext` wrapper, no canonical corpus, and no unit detector; the
facsimile front matter is not yet produced.

## Desired end state

- A new module **`src/facsimile/detect.rs`** auto-derives the unit table from the
  source PDFs' OCR layer plus the built-in canonical corpus, producing the same
  `Vec<Unit>` shape `units::load_units` returns.
- **`facsimile_units.json` becomes an auto-generated, committed, hand-editable
  cache** (like `plutarch.tags.json`): a build regenerates it when it is missing or
  `--redetect` is passed, and otherwise loads the committed file. Hand corrections
  edited into the file are preserved — regeneration **merges**, never clobbers, rows
  a human has marked `"pinned": true`.
- `cargo run --bin facsimile -- --v1 … --v2 … --detect-units` writes/refreshes
  `facsimile_units.json` and exits; a normal build auto-detects on first run if the
  file is absent. Detection is offline and deterministic (OCR layer + corpus).
- The detected table reproduces, by eye on a spot-check, the real Didot order and
  start pages for both volumes, with every Life and Comparatio present exactly once
  and groups binding each pair with its Comparatio.

## Tech Learnings

- **`pdftotext` and the text layer.** `pdftotext -f N -l N file.pdf -` prints page
  N's embedded text. These scans were OCR'd (by Google) and the layer is good enough
  for ALL-CAPS titles even where body text is noisy. The running head is among the
  first few non-empty lines; the folio (page number) is interleaved and must be
  stripped. Wrap it in `render.rs` beside `render_page`, overridable via
  `PLUTARCH_PDFTOTEXT`, returning `Result<String>` (empty string when the tool is
  absent or a page has no text layer, so a missing layer surfaces as a reported gap
  rather than a hard failure). The spike confirmed the layer is present throughout
  these scans, so a Tesseract OCR fallback is a documented contingency only —
  implement it in Phase 4 *iff* validation turns up pages with an empty layer.
- **Orthography folding for matching — reuse `verify::fold`.** OCR and 1857
  orthography introduce exactly the differences `verify::fold` already erases
  (`u/v`, `i/j`, `æ→ae`, long s, case, punctuation, digits). Fold each OCR'd title
  word with it, then match tokens to the corpus by the **common-prefix idiom from
  `assemble.rs`** (`common_prefix_len ≥ 4`), which tolerates OCR truncation and the
  nominative/genitive split: a Life matches on its name (`ROMULUS`); a Comparatio
  matches when both paired genitive name stems plus a `COMPARATIO` token are present
  (order-independent, so `COMPARATIO THESEI CUM ROMULO` and `LYCURGI ET NUMÆ
  COMPARATIO` both resolve). Do not write a second folder or a parallel matcher —
  lift `assemble`'s helper to `pub(crate)` and share it.
- **The display-title *line* is the primary start signal; the running head gives
  identity; the band is last resort.** An opening page carries the unit's title as a
  short, title-like OCR line (`THESEUS.`, `COMPARATIO THESEI CUM ROMULO`). That text
  line — short, ALL-CAPS, fold-matching a corpus unit — pins the exact `start_page`
  reliably. The *running head* names every page's unit and so segments the body into
  unit runs (for identity and coverage), but its first clean occurrence lags or
  leads the true opening by several pages when early pages OCR poorly, so it must
  **not** set `start_page` on its own. The display-title *band height* is only a
  final visual tie-breaker — the spike shows it mislabels verse pages (false
  positive) and underweights Comparationes (false negative), so it is never primary.
- **Run-length segmentation with smoothing.** Per-page OCR occasionally drops or
  misreads the running head (a blank, a page-number-only verso, a garbled line).
  Assign each page its best corpus match or `None`, then smooth: a lone `None` or a
  single-page disagreement inside an otherwise uniform run is filled from its
  neighbours. Unit runs are the maximal contiguous spans of one corpus id; they
  bound each unit and catch a unit whose display-title line OCR'd too poorly to find.
- **Why a canonical corpus, not page OCR, for Greek/kind/group/order.** The 65-odd
  units are a fixed, well-documented corpus in a fixed edition order. Encoding them
  once (Latin + Greek title, kind, pair-group, source) turns an unreliable
  Greek-OCR problem into a reliable Latin-recognition problem, and makes `kind`,
  `group`, and ordering exact by construction rather than inferred.

---

## Phase 1: `pdftotext` wrapper and the canonical corpus

**Suggested implementer: Opus.** Establishes the corpus data model (domain
knowledge about the Didot order) and the OCR plumbing the rest depends on.

### Step 1.1 — `pdftotext` page-text wrapper

- In `src/render.rs` add `pub fn page_text(pdf: &Path, page: u32) -> Result<String>`
  shelling out to `pdftotext -f {page} -l {page} {pdf} -`, binary overridable via
  `PLUTARCH_PDFTOTEXT` (default `pdftotext`), and `pub fn pdftotext_available() ->
  bool`. On a non-zero exit or a missing binary return `Ok(String::new())` (not an
  error) so callers can fall back to Tesseract.
- Add `pub fn running_head(page_text: &str) -> String`: from the first few non-empty
  lines, drop pure-digit and punctuation-only tokens, keep tokens that are ALL-CAPS
  (after folding), join with spaces. This is the per-page title candidate.

**Tests:** `running_head_strips_folio_and_keeps_title` (`"ROMULUS. 18"` →
`"ROMULUS"`; `"115 SOLON."` → `"SOLON"`; `"LYCURGI ET NUMÆ COMPARATIO. 89"` →
`"LYCURGI ET NUMÆ COMPARATIO"`); `running_head_empty_on_noise` (a line of stray
punctuation → `""`). `page_text` is covered by an integration test gated on
`pdftotext_available()` and the source PDF being present.
**LLM verification:** `cargo test render`.
**Human verification:** None.

### Step 1.2 — Canonical corpus of the Parallel Lives

- Add `src/facsimile/corpus.rs` with `pub struct CanonUnit { pub id: &'static str,
  pub latin_title: &'static str, pub greek_title: &'static str, pub kind: LifeKind,
  pub group: u32, pub source: usize, pub match_tokens: &'static [&'static str] }`
  and `pub fn corpus() -> &'static [CanonUnit]`.
- Populate it with every Life and Comparatio in Didot order for v1 and v2, grouping
  each Life pair with its Comparatio (same `group`), `source` 0 for v1 and 1 for v2.
  `match_tokens` are the folded tokens a page must contain to match: for a Life, its
  name (e.g. `["ROMULUS"]`); for a Comparatio, both genitive names plus
  `["COMPARATIO"]` (e.g. `["THESEI","ROMULI","COMPARATIO"]` / `["LYCURGI","NUMAE",
  "COMPARATIO"]`). The implementer fills the full list from the running-head survey
  (extend the spike's page scan across both sources to read every running head), and
  confirms v1 ends on the Cimon–Lucullus Comparatio and v2 opens with a Life, so the
  forced v1→v2 volume break stays valid.
- **Greek titles source.** The Greek title of every Life and Comparison is on the
  Greek Wikipedia article *Βίοι Παράλληλοι*
  (`https://el.wikipedia.org/wiki/Βίοι_Παράλληλοι`) — Θησεύς–Ρωμύλος,
  Λυκούργος–Νουμάς, Σόλων–Ποπλικόλας, … including the four unpaired Lives (Άρατος,
  Αρταξέρξης, Γάλβας, Όθων). That list is in the **traditional manuscript order, not
  Didot's**, so take only the Greek string per Life from it and let the Didot order
  come from detection. A Comparison's Greek title is formed `ΣΥΓΚΡΙΣΙΣ <genitive> ΚΑΙ
  <genitive>` (e.g. `ΣΥΓΚΡΙΣΙΣ ΘΗΣΕΩΣ ΚΑΙ ΡΩΜΥΛΟΥ`). Cross-check each against the
  Greek column's display title on the opening page when spot-checking in Phase 4.
- Lift `assemble::common_prefix_len` to `pub(crate)` and add
  `pub(crate) fn token_matches(a: &str, b: &str) -> bool` (= `common_prefix_len ≥ 4`)
  so `corpus`, `detect`, and `assemble` share one matcher (no parallel implementation).
- Add `pub fn match_page(page_tokens: &[String]) -> Option<usize>` returning the
  unique corpus unit all of whose `match_tokens` have a `token_matches` partner in
  `page_tokens`. When more than one unit matches
  (e.g. a Comparatio line also contains the two Life names), prefer the match
  requiring the most tokens — the Comparatio — so a Comparatio page never resolves to
  one of its component Lives. `match_tokens` hold **folded** forms (run them through
  `verify::fold` in the corpus or at match time), so the corpus literals and the
  page tokens compare in the same space.

**Tests:** `corpus_is_ordered_and_grouped` (strictly non-decreasing `source`; each
group has ≥1 Life and at most one Comparatio; ids unique; v1's last unit is a
Comparatio, v2's first a Life); `match_page_resolves_life` (`{"ROMULUS"}` → Romulus);
`match_page_resolves_comparatio` (`{"LYCURGI","NUMAE","COMPARATIO"}` → that
Comparatio, not Lycurgus or Numa); `match_page_none_on_unknown`.
**LLM verification:** `cargo test corpus`.
**Human verification:** Read `corpus()` against a Loeb/standard index of the
Parallel Lives and confirm the order, pairings, and the v1/v2 split are correct.

---

## Phase 2: Page→unit matching and run segmentation

**Suggested implementer: Opus.** The segmentation and smoothing carry the
correctness of every start page; edge cases need judgment.

### Step 2.1 — Folding, token matching, and per-page unit

- In `src/facsimile/detect.rs` add `pub fn tokens(s: &str) -> Vec<String>`: split on
  whitespace, `verify::fold` each word, drop empties and stop tokens (`et`, `cum`,
  `seu`, and ≤2-letter tokens). Reuse `verify::fold` — do not reimplement it.
- The shared matcher (`token_matches`, `verify::fold`) is already in place from Step
  1.2; `corpus::match_page` uses it.
- `pub fn page_unit(page_text: &str) -> Option<usize>`:
  `corpus::match_page(&tokens(running_head(text)))`.

**Tests:** `tokens_fold_orthography` (`"VITÆ"`/`"VITAE"` and `"ROMVLVS"`/`"ROMULUS"`
produce matching tokens via `verify::fold`); `page_unit_matches_comparatio_wording`
(both `"COMPARATIO THESEI CUM ROMULO"` and `"THESEI ET ROMULI COMPARATIO"` → the
same corpus index); `page_unit_distinguishes_life_from_comparatio` (`"ROMULUS"` →
the Life, not the Comparatio that also names Romulus).
**LLM verification:** `cargo test detect`.
**Human verification:** None.

### Step 2.2 — Display-title openings, run segmentation, and start pages

- `pub fn title_line_unit(page_text: &str) -> Option<usize>`: scan the OCR lines for
  a **short, title-like** line (≤3 tokens after folding, ALL-CAPS in the source) that
  `corpus::match_page` resolves — the display title. Returns the opening unit, or
  `None` when the page carries no display title. This, not the running head, is the
  start-page signal.
- `pub fn page_units(texts: &[Option<String>]) -> Vec<Option<usize>>` mapping each
  page's text (indexed by source page − 1) through `page_unit` (running head → unit).
- `pub fn smooth(seq: &[Option<usize>], window: u32) -> Vec<Option<usize>>`: fill a
  lone `None` or single-page disagreement inside an otherwise uniform run from its
  neighbours, so one bad OCR page neither splits a run nor spawns a phantom unit.
- `pub struct Run { pub unit: usize, pub start_page: u32, pub end_page: u32 }` and
  `pub fn runs(smoothed) -> Vec<Run>`: maximal contiguous spans of one corpus id
  (1-based pages). `MIN_RUN_PAGES` is **1** (a Comparatio can be a single trim page),
  so runs are not dropped for brevity; isolated single-page *disagreements* are
  already absorbed by `smooth`.
- `pub fn detect_openings(pdf, source, body_range, page_text_fn) -> Result<Vec<Run>>`:
  for each corpus unit the runs cover, set `start_page` to the **first page in/just
  before its run whose `title_line_unit` is that unit** (the display title); fall
  back to the run's first page when no title line is found. End page is the next
  unit's `start_page − 1` (the last unit runs to the source's last body page, Phase
  4.1). `page_text_fn(page) -> Result<String>` is injected so tests supply canned OCR
  without a PDF.

**Tests (canned per-page text, no PDF):** `runs_segment_on_name_change`
(THESEUS×5, ROMULUS×5 → two runs); `smooth_fills_single_bad_page` (THESEUS, garbled,
THESEUS → one run); `start_page_pinned_by_display_title` (a run whose running head
first reads cleanly three pages in, but whose first page has the display-title line,
starts on that first page); `start_page_falls_back_to_run_when_no_title`.
**LLM verification:** `cargo test detect`.
**Human verification:** Run detection over v1's body range and print the
`(start_page, latin_title)` list; confirm it matches the real openings
(Theseus p14, Romulus ~p34, Comparatio Thesei ~p56, Lycurgus ~p60, …).

---

## Phase 3: Emit the unit table as a cache, and wire it in

**Suggested implementer: Sonnet.** Mechanical once Phases 1–2 exist: map runs to
`Unit`s, merge with pinned hand edits, serialize, and add the CLI path.

### Step 3.1 — Runs → `Unit`s

- `pub fn units_from_runs(runs_v1, runs_v2) -> Vec<Unit>`: each run → a `Unit` taking
  `id/latin_title/greek_title/kind/group/source` from `corpus()` and `start_page`
  from the run (already pinned to the display-title line in Step 2.2). Validate the
  result with the existing `units::` invariants (strictly increasing `start_page` per
  source, v1 before v2, non-empty titles) and that every matched corpus unit appears
  exactly once.
- A unit the runs never covered (its running head OCR'd too poorly anywhere) is
  reported as a gap for a manual pin, not silently dropped — mirrors the no-silent-
  drop rule elsewhere.

**Tests:** `units_from_runs_builds_valid_table` (synthetic runs for both sources →
passes `units::` validation; groups bind pairs); `units_from_runs_reports_missing_unit`.
**LLM verification:** `cargo test detect`.
**Human verification:** None.

### Step 3.2 — Cache file with merge-preserving hand edits

- `pub fn load_or_detect(path, pdfs, redetect, page_text_fn) -> Result<Vec<Unit>>`
  mirroring the front-matter sidecar semantics: if `path` exists and `!redetect`,
  load it; else detect, **merge** over any existing rows whose JSON carries
  `"pinned": true` (a human correction stays authoritative), write the file, and
  return it. Add an optional `#[serde(default)] pinned: bool` to `Unit` so pins
  round-trip and `skip_top_bands` hand-tweaks survive regeneration.
- Add `--detect-units` and `--redetect` to `src/bin/facsimile.rs` and
  `FacsimileArgs`. `--detect-units` writes/refreshes `facsimile_units.json` and
  exits. A normal build calls `load_or_detect` (auto-detecting when the file is
  absent) instead of `units::load_units`.

**Tests:** `load_or_detect_writes_when_absent` (a temp dir with a stub
`page_text_fn` → file created, rows match detection); `load_or_detect_preserves_pinned`
(a pinned hand-edited row survives a redetect that would otherwise change it);
`detect_units_cli_flags_parse`.
**LLM verification:** `cargo test facsimile`; `cargo build --bins`.
**Human verification:** Run `--detect-units`, open `facsimile_units.json`, confirm
the full table looks right; pin and edit one row, re-run `--redetect`, confirm the
edit survives.

---

## Phase 4: Full-source validation and the real last-body page

**Suggested implementer: Opus.** Runs the detector over both complete sources,
fixes edge cases, and closes the approximate `last_body` gap the facsimile build
still carries.

### Step 4.1 — Detect over both full sources and reconcile

- Run detection over v1 and v2 in full. Verify every corpus unit is found exactly
  once, in order, and spot-check ~10 start pages by rendering the opening page and
  eyeballing the title. Handle the unpaired Lives (Aratus, Artaxerxes, Galba, Otho —
  each its own group, no Comparatio) and any edition quirk in word order or wrapping
  the running head throws. Record any page where OCR forced a manual pin.
- Derive each source's **last body page** from the detector's last run's `end_page`
  (the final Comparatio's last page), and feed it into `run_facsimile`'s `last_body`
  in place of the PDF page count, so the final unit's range no longer runs into the
  back-matter/library pages.

**Tests:** `full_detection_covers_corpus` (gated on `pdftotext` + PDFs: detection
over both sources yields exactly the corpus units, ordered, each once);
`last_body_from_runs_excludes_backmatter` (last `end_page` < source page count).
**LLM verification:** `cargo test`; `cargo build --bins`.
**Human verification (critical):** Diff the auto-detected `facsimile_units.json`
against the three seeded rows and a hand-check of ~10 openings across both volumes;
run a full `--no-compile` build and confirm unit count, order, and the v1→v2 split.

### Step 4.2 — Supersede the manual table step

- In `2026_10_03_FACSIMILE_READER_PLAN.md`, rewrite Phase 5.2 in place (per CLAUDE.md,
  no "update" block) to record that the unit table is **auto-detected**, pointing to
  this plan; it is no longer a hand-authored human deliverable. (The README is
  updated once, comprehensively, in Step 5.3 below.)

**LLM verification:** none (docs).
**Human verification:** None.

---

## Phase 5: Front-matter hybrid and the two-path README (folded from the facsimile reader plan)

**Suggested implementer: Opus.** Integrates the one remaining vision dependency
(front-matter transcription) and its faithfulness gate into the facsimile path, then
documents both build paths. This phase folds in the still-unimplemented Steps 5.1 and
5.3 of `2026_10_03_FACSIMILE_READER_PLAN.md`; its Step 5.2 (the manual unit table) is
retired by Phases 1–4 of this plan.

### Step 5.1 — Front matter via cached transcription + reused LaTeX

- The facsimile body needs no vision, but the **front matter** (v1 Monitum, v2
  Praefatio, derived title pages) is full-width prose that must be re-typeset to the
  trim measure. There is no front-matter-only `Scope` in `pipeline::run`, and `run`
  always transcribes the body, so **do not** call `pipeline::run`. Instead reuse the
  stage functions directly: load/initialize a committed sidecar
  `plutarch.frontmatter.tags.json`, run `pages::classify` over the boundary regions,
  then `transcribe::transcribe_all` / `verify::reconcile_all` with an `in_scope`
  closure that admits **only `FrontProse`/`FrontImage`** pages (body pages are never
  requested). Build the `Client` / `ApiVision` / `ApiBatchBackend` lazily, only when
  front-matter work remains (mirror `main.rs`).
- **Caching (required).** `plutarch.frontmatter.tags.json` is a committed, hand-
  editable cache with the same semantics as `plutarch.tags.json`: on load, if present,
  not stale (`Sidecar::is_source_stale`), and the `FrontProse` `result`s are filled,
  the build makes **zero** network calls. The Monitum/Praefatio are transcribed once;
  later builds read the cache. Re-transcription happens only when the sidecar is
  missing, a source hash changed, or `--retag` is passed (`FacsimileArgs.retag`
  already exists and is wired to the CLI). The cache is the single source of truth —
  human proofreading corrections are edited into it, never regenerated over (the
  CLAUDE.md faithfulness rule).
- **Reuse the front-matter LaTeX, don't duplicate it.** Extract the front-matter
  composition `inner_document` currently inlines — `title_page` + `front_images` +
  `front_prose` — into a `pub(crate) fn latex::front_matter_block(sidecar, source,
  tomus, render_front_image) -> String`, and have the transcription `inner_document`
  call it too (its emitted bytes are unchanged, since it is the same three pieces in
  the same order). The facsimile fills `FacsimileVolume.front_matter` by calling
  `front_matter_block` for the first volume of each source, rendering the embedded
  front-image scans into `--out` exactly as `write_inner_volumes` does. Bib strings
  (title/editor/date) are hand-filled in the front-matter sidecar.

**Tests:** `frontmatter_only_transcribes_frontprose` (an `in_scope` rejecting body
pages → zero body requests through a stub `BatchBackend`);
`frontmatter_offline_when_sidecar_fresh` (a complete, non-stale front-matter sidecar →
no vision/backend constructed, zero calls); `front_matter_block_unchanged_for_transcription`
(golden: `inner_document`'s front-matter bytes are identical after the extraction).
**LLM verification:** `cargo test facsimile latex`.
**Human verification:** Proofread the typeset Monitum and Praefatio against the source
pages; confirm verbatim fidelity and intact Greek fragments; run a second build and
confirm zero network calls (cached).

### Step 5.2 — End-to-end full build

- With the auto-detected unit table (Phases 1–4) and the front-matter hybrid (5.1),
  run a full `--no-compile` build and then a real compile of Tomus 1 for each source.
  Confirm atomicity (each unit opens fresh, Comparationes follow their pair), running
  heads, folios, legibility, margins, and that no source furniture or marginal
  letters leak into the body beyond the known left-edge gutter-rule artifact (an
  accepted artifact).

**Tests:** none beyond those above (this step is integration/real-eyes).
**LLM verification:** `cargo test`; `cargo build --bins`.
**Human verification (critical):** Compile Tomus 1 for v1 and for v2; open the PDFs
and verify the full chain end to end.

### Step 5.3 — README for the two build paths

Rewrite `README.md` **in place** (no "update"/"revision" blocks) to document both
build paths:

- **facsimile** (`cargo run --bin facsimile -- …`): body pages are the original 1857
  Didot Latin column, cropped and reflowed into the trim text block at one global
  scale, under a typeset centered Latin running head and the per-Tomus folio; margins
  are inner/outer 1 cm, 1.5 cm above the running head, 1.5 cm below; each
  Life/Comparatio opens fresh; the unit table is **auto-detected** into
  `facsimile_units.json` (`--detect-units` / `--redetect`, `pinned` hand edits
  preserved); front matter is transcribed **once** and cached in
  `plutarch.frontmatter.tags.json` (`--retag`/stale re-runs it); imposition,
  signatures, fold/sew/cut marks, and the two-stage compile are shared with the
  transcription path.
- **transcription** (`cargo run …`): re-typeset, selectable text, per the cost gate.
- When to use each (facsimile = verbatim pixels, ~zero body cost, non-searchable,
  image-heavy PDFs; transcription = re-typeset, selectable), the dependencies
  (poppler incl. `pdftotext` + lualatex; no API key except the one-time front-matter
  transcription), and that `facsimile_units.json` and `plutarch.frontmatter.tags.json`
  are committed, hand-editable caches. Point to this plan,
  `2026_10_03_FACSIMILE_READER_PLAN.md`, and `2026_10_03_COST_GATE_FINDINGS.md`.

**LLM verification:** none (docs).
**Human verification:** Follow the README on a fresh checkout: `--detect-units`, a
`--unit <id>` build, and a full Tomus compile all work as written.

---

## Cross-cutting requirements

- Detection is offline and deterministic: OCR text layer + the built-in corpus, no
  network. Tesseract on the title band is a *fallback* only where the text layer is
  empty, never the primary path.
- Greek titles, `kind`, `group`, and order come from the corpus, never from page
  OCR. Latin OCR is used only to *identify* and *locate* units.
- `facsimile_units.json` stays a committed, hand-editable cache; regeneration
  preserves `pinned` rows and `skip_top_bands` tweaks (merge, never clobber).
- Every Rust-touching step ends green on `cargo fmt --all --check`, `cargo clippy
  --all-targets -- -D warnings`, and the relevant `cargo test`; full `cargo test`
  green before a phase is declared done. No CI.
- No code comment references this plan, its phases, or `llm_artifacts/`.
- The spike `spikes/facsimile_unit_detect_spike.py` is retained as the dated record
  of why detection is OCR-first and band-height is only a tie-breaker.
