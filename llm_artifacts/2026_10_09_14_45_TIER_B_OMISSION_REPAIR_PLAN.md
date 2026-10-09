# Context

The inner-tomi audit confirmed 59 unique-loss omissions; 39 (Tier A) were repaired in
the [2026-10-08 repair pass](2026_10_08_16_00_INNER_TOMI_AUDIT_REPAIRS_PLAN.md). The
remaining 20 (Tier B, 44 source Latin lines) were real confirmed losses held back
because each needed a human to read the exact source line and sign off before a fix was
enabled. That review is now complete: **all 20 events are promoted** (see
[`2026_10_09_10_45_TIER_B_PROMOTION_REVIEW_WORKSHEET.md`](2026_10_09_10_45_TIER_B_PROMOTION_REVIEW_WORKSHEET.md)),
and the exact band coordinates for each dropped line have been extracted (see
[`2026_10_09_14_30_TIER_B_BAND_EXTRACTION.md`](2026_10_09_14_30_TIER_B_BAND_EXTRACTION.md)).

This plan restores those 44 lines using the **same ad hoc per-band mechanism the Tier-A
pass used** — `SOURCE_BODY_BANDS` protection entries — with no heuristic changes. Every
dropped line is the first or last body line of a source page, removed by the furniture
or boundary trims; a protection entry re-inserts the exact detected band after those
trims. The work is deliberately mechanical and low-risk: each entry touches only its one
source page.

All Latin is source material. A protection entry restores **original source pixels** from
the owned column raster; it never retypes, normalizes, or OCR-replaces prose. Work
continues on branch `plan/inner-tomi-audit-repairs`.

# Current state of the world

- `src/facsimile/reflow.rs`
  - `SOURCE_BODY_BANDS: &[SourceBodyBand]` (reflow.rs:882) — 48 installed entries from
    Tier A. Each is `{ source, page, unit_id, dimensions, observed, protected }`.
    `dimensions` is the cropped Latin-column `(width, height)` at `BODY_DPI = 300`;
    `observed`/`protected` are `Band { top, bottom }` in that column's pixel-row space.
  - `retain_known_source_body_bands(owner, source, page, dimensions, observed, kept)`
    (reflow.rs:1545) — for each matching record whose `observed` **or** `protected` band
    is present in the `observed` input, pushes `protected` into `kept`, then sorts and
    merges overlaps. It runs twice per page: inside `render_and_band` with `owner: None`
    (after `drop_furniture`), and inside `unit_body_strips` Pass B with
    `owner: Some(unit_id)` (after `drop_opening_bands` / `drop_interior_running_head` /
    `drop_shared_tail`). So a line removed at **either** stage is restored.
  - The trims that drop these lines — `drop_furniture` (top/bottom zone),
    `retain_early_aligned_body_bands` (rescues only the `0.07..0.08` centre fraction),
    `drop_interior_running_head` → `drop_trailing_furniture` — are **not changed** by
    this plan.
  - `coalesce_known_body_fragments` (5 entries) and `preserve_known_body_crop_edge`
    (5 entries) are the sibling ad hoc tools for merged / edge-clipped bands; this plan
    does not need them (all 20 lines are recoverable as `SOURCE_BODY_BANDS` entries).
- Tests (reflow.rs test module and `audit_repair_tests.rs`)
  - `known_source_body_bands_survive_furniture_filters` (audit_repair_tests.rs:1677)
    iterates **every** `SOURCE_BODY_BANDS` record on a synthetic column and asserts
    furniture filtering drops `observed` while `retain_known_source_body_bands` rescues
    exactly one `protected`. New entries are covered automatically.
  - `traced_real_unit(range)` (reflow.rs:2604) runs the real production pipeline on a
    unit from `build_facsimile/render_cache` and checks the reviewed-band ledger
    `fixtures/closing_body_bands.json` (`ReviewedBands { source, page, dpi, column,
    body, excluded, noise }`): each `body` interval must appear **exactly once** in the
    retained bands and its retained pixels must equal the original source crop; each
    `excluded` interval must contribute no strip.
  - `extract_tier_b_source_bands` (reflow.rs test module, `#[ignore]`) — the diagnostic
    that produced the extraction table; it bands each dropped-line page and saves crops
    under `tmp/pdfs/inner_tomi_audit_repairs/band_extract/`.
- Build: `cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2
  Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline`.

# Desired end state

1. `SOURCE_BODY_BANDS` contains 22 new entries (the 20 events; Lysander and Pericles
   contribute two lines each, Antonius one 23-line block) exactly as tabulated in the
   extraction note.
2. `fixtures/closing_body_bands.json` records each restored line as a `body` interval on
   its page (and, for the three merged-head bands, the Greek/head fragment as an
   `excluded` interval), so `traced_real_unit` proves each line is retained once with
   original pixels and each head fragment stays dropped.
3. All eight inner readers rebuild offline; every one of the 20 reader seams catalogued
   in the worksheet now reads continuously (the dropped line present, in order, with its
   split-word continuation rejoined, neighbours intact), confirmed by real-eyes review.
4. Antonius's 23-line block composes into readable, correctly separated lines in the
   rebuilt reader (verified specifically, because it restores as one tall band).
5. No heuristic, CLI, dependency, or global-scale change; raw render caches, source
   PDFs, and transcription caches untouched. `cargo fmt --all --check`,
   `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass.
6. A durable verification note records before/after prose landmarks for each line and
   every retained-artifact decision (the three excluded head fragments, the Antonius
   composition check, the dio p580 skew check).

# Tech Learnings

- **`observed` vs `protected`.** `observed` must equal the band `line_bands` actually
  emits (matched by exact integer equality), so it is copied verbatim from the
  extraction. `protected` is what gets re-inserted: it equals `observed` for a clean
  line, but for a band where `line_bands` merged a running-head fragment above the Latin
  (brutus, cato_minor p329, demosthenes, phocion) `protected` is the lower Latin
  sub-range only, so the Greek/head scrap stays dropped.
- **Why one entry can restore 23 lines (Antonius).** On v2 p562 the inter-line gaps
  never fall below `LINE_MIN_INK`, so `line_bands` returns the whole conclusion as one
  1107-px band. Protecting that band restores all 23 lines, but `compose_pages` must
  then fill-split the tall strip back into lines — hence the dedicated composition check.
- **`closing_body_bands.json` is the real-source proof.** Adding a `body` interval there
  makes `traced_real_unit` assert the line is retained with byte-identical source pixels;
  this is the regression that would catch a future change silently dropping the line again.

# Phase 1 — Install the clean single-line entries (Suggested implementer: Sonnet)

The 17 clean bands (`protected = observed`) from the extraction note. No sub-range
reasoning; copy the tabulated tuples verbatim. Group the steps so each leaves the suite
green.

## 1.1 Head-region clean entries

Append these `SOURCE_BODY_BANDS` entries (from the extraction note's "Ready-to-install"
block): aemilius_paulus p326, caius_marius p519, crassus p44, lucullus p631, marcellus
p377, nicias p25, timoleon p318, lysander p535 (both lines). For each, add the
`protected` interval to that page's `body` list in `closing_body_bands.json` (create the
page record if absent, with `dpi: 300` and `column: [left, right]` from the detector —
obtain `column` by reading it off `traced_real_unit`'s own detection or the extraction
dimensions).

**LLM verification:** `cargo test --release -p plutarch-reader
known_source_body_bands_survive_furniture_filters` passes; `cargo test --release -p
plutarch-reader traced_real_unit` (or the per-unit ignored real-source tests for
aemilius_paulus, caius_marius, crassus, lucullus, marcellus, nicias, timoleon, lysander)
pass; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`.

**Human verification:** None beyond Phase 4's reader review (these are data additions;
the pixel equality is machine-checked).

## 1.2 Foot-region clean entries

Append: agesilaus p110 (the foot band — distinct from the existing head entry on p110),
artoxerxes p633, cato_minor p352, dio p566, dio p580. Add each `protected` to
`closing_body_bands.json`. For **dio p580**, after rebuild (Phase 4) confirm the restored
line shows no clipped descender; if it does, extend `protected.bottom` a few pixels and
re-verify (the human directive is to prefer extra space over a mangled line).

**LLM verification:** the `known_source_body_bands...` test and the per-unit real-source
tests for agesilaus, artoxerxes, cato_minor, dio pass; fmt and clippy clean.

**Human verification:** None here; dio p580 descender is eyeballed in Phase 4.

## 1.3 Verse-line clean entries

Append: alcibiades p242, pericles p214 (verse 1) and p215 (verse 2). These are indented
verse lines; the protection restores their source pixels (indentation preserved). Add
each `protected` to `closing_body_bands.json`.

**LLM verification:** the `known_source_body_bands...` test and the alcibiades / pericles
real-source tests pass; fmt and clippy clean.

**Human verification:** None here; verse layout is eyeballed in Phase 4.

# Phase 2 — Install the merged-head entries (Suggested implementer: Opus)

The four bands where `line_bands` merged a running-head fragment above the Latin line, so
`protected ⊂ observed` and the fragment must stay dropped: **brutus p619** (Greek `ΟΣ.`),
**cato_minor p329** (head `…N.`), **demosthenes p445** (Greek `ΕΝΗΣ.`), **phocion p291**
(whitespace-padded tall band).

## 2.1 Append the four entries with sub-range `protected`

Append the four `SOURCE_BODY_BANDS` entries from the extraction note (with their measured
`protected` sub-ranges). In `closing_body_bands.json`, for brutus/cato_minor p329/
demosthenes add the `protected` interval to `body` **and** add the merged head fragment's
rows to `excluded` (brutus `[129,158]`, cato_minor p329 `[114,142]`, demosthenes
`[127,156]` — from the extraction note's measured groups), so the regression proves the
Latin is restored and the Greek/head scrap contributes no strip. For phocion add the
`protected` interval to `body` (its band held only whitespace above the Latin).

**LLM verification:** `known_source_body_bands_survive_furniture_filters` passes
(it exercises the `protected.top < observed.top` branch); the real-source tests for
brutus, cato_minor, demosthenes, phocion pass, including the `excluded`-fragment
assertions; fmt and clippy clean.

**Human verification:** In the Phase 4 rebuild, confirm for brutus/demosthenes that **no
Greek fragment** appears above the restored Latin line, and for cato_minor p329 that no
head fragment or large blank gap appears above `rum. Interim Rhodi …`.

# Phase 3 — Restore and verify the Antonius 23-line block (Suggested implementer: Opus)

A test-install of the plain single-band entry was run and **failed** — see
[`2026_10_09_15_30_ANTONIUS_BLOCK_COMPOSITION_FINDINGS.md`](2026_10_09_15_30_ANTONIUS_BLOCK_COMPOSITION_FINDINGS.md).
The protection fired (the band was retained), but `compose_pages` cannot split a single
1107-px `Tall` strip that has **no internal low-ink rows** into readable lines, so the
block did not appear at its reading-order position in the rebuilt reader. The block also
cannot be line-split automatically (no ≥6-row blanks; dense-coverage separation yields
only ~4 groups — the left-margin section letters and tight leading keep ink in every
inter-line row). Phase 3 therefore installs **explicit, hand-measured line bands**, not
one tall band.

## 3.1 Decide the split method

Choose, and record the choice with evidence:

- **(a) Per-page crop-edge adjustment.** Test whether excluding the left-margin section
  letters from the p562 column crop (a `preserve_known_body_crop_edge`-style entry) lets
  `line_bands` clear the inter-line rows and emit 23 separate bands. If so, Antonius
  reduces to ordinary single-line `SOURCE_BODY_BANDS` entries (or needs none). Verify it
  changes nothing else on p562.
- **(b) Explicit line bands (fallback).** If (a) does not cleanly separate the lines,
  hand-measure the 23 line bands at the printed pitch (~48 px across `249..1356`) and
  install 23 `SOURCE_BODY_BANDS` records that all share `observed: 249..1356` with one
  non-overlapping line each in `protected` (small gaps between them so the retain merge
  keeps them separate). `retain_known_source_body_bands` pushes every matching record's
  `protected`, so one merged `observed` re-inserts 23 lines — no new mechanism.

## 3.2 Install and verify

Install the chosen method. Add each restored line interval to `closing_body_bands.json`
`body` for page 562. Rebuild `tomus_7_inner.pdf`.

**LLM verification:** the antonius real-source test and `known_source_body_bands...`
pass; fmt and clippy clean.

**Human verification (gating):** Real-eyes read of the rebuilt tomus 7 Antonius ending —
all 23 lines (`vatam dicunt … a Cæsare redemit.`) present, **individually legible and
correctly separated**, in order, between `… aspidem asser-` and `LXXXVII. Antonius ex
tribus uxoribus …`, neighbours intact. If this cannot be achieved, Antonius stays an
unresolved Tier-A event; it does not block the other 21 repairs, which ship independently.

# Phase 4 — Rebuild, verify all seams, and record evidence (Suggested implementer: Opus)

## 4.1 Full rebuild and whole-suite green

Rebuild all eight inner readers offline. Run the complete suite.

**LLM verification:** the build command succeeds; `cargo test` (full) passes;
`cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings` clean.

**Human verification:** None (this step is the mechanical gate).

## 4.2 Verify every restored seam and write the note

For each of the 20 events, open the rebuilt `tomus_N_inner.pdf` at the worksheet's
recorded page and confirm the seam now reads continuously with the restored line in
place (the worksheet lists the exact before/after anchors and tomus/page for each).
Write a durable verification note under `llm_artifacts/` recording, per event: the
tomus/page, the before anchors (the broken jump) and after text (continuous), and the
retained-artifact decisions (the three excluded head fragments; the Antonius composition
result; the dio p580 descender check).

**LLM verification:** none (documentation step).

**Human verification:** The 20 seams read correctly; the note matches what the PDFs show.
Spot-check that no unrelated page changed (the entries are page-scoped, so only the 21
source pages are affected).

# Durable record

The band coordinates and their derivation are frozen in
[`2026_10_09_14_30_TIER_B_BAND_EXTRACTION.md`](2026_10_09_14_30_TIER_B_BAND_EXTRACTION.md);
the per-event confirmed Latin and seam locators are in the
[promotion worksheet](2026_10_09_10_45_TIER_B_PROMOTION_REVIEW_WORKSHEET.md). Together
with the Phase 4 verification note, these document why each of the 22 `SOURCE_BODY_BANDS`
entries exists.
