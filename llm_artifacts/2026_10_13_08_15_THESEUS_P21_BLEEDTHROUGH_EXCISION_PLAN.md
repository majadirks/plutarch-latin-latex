# Context

The facsimile body of Tomus I reproduces the 1857 Didot Latin column as cropped
source-image pixels, reflowed line by line into the trim text block. On source
page 21 of `Ploutarchou_Bioi_v1.pdf` (v1, zero-based `source == 0`, one-based
source page 21), which is in the **Theseus** unit, ink bled through from the
reverse leaf during scanning. Between the end of Theseus §17 (the Latin line
ending `… tradit agi.`) and the start of §18 (`XVIII. Sorte ducta quum
accepisset ex Prytaneo Theseus …`) there is a block of five faint, broken,
mirror-image strokes that are **the only ink in that vertical slab** — no real
Latin glyphs sit there. This bled slab is picked up by line-band detection and
reproduced in the reader; it currently appears on physical page 30 of
`build_facsimile/tomus_1_inner.pdf`.

The owner wants: where the bleed overlaps real Latin it stays (faithfulness —
we never erase or alter source glyphs); but where the bleed stands alone it
should be removed **and the vertical space it occupied closed up**, so §17 and
§18 are separated by an ordinary inter-line gap rather than five blank lines.

This plan makes one finite, edition-specific correction through the existing
facsimile path. It does not change transcription caches, global layout
thresholds, the unit table, the audit system, or the source PDFs. It mirrors the
existing `SOURCE_FURNITURE_MASKS` convention but adds a second, distinct
mechanism — a coordinate-pinned **row excision** — because masking alone cannot
satisfy the second half of the request (see "Why a mask is not enough" below).

# Current state of the world

- `src/facsimile/reflow.rs` renders each source page at 300 DPI
  (`BODY_DPI = 300`), then in `render_and_band` (≈ lines 899–912) does, in order:
  `mask_known_source_furniture(source, page, &mut page_img)` →
  `production_source_column(...)` (column detect) → `crop_column(...)` →
  `band_source_column(...)` (line banding + furniture drop). Pass A of
  `unit_body_strips` calls `render_and_band` once per source page and stores the
  cropped column plus its kept bands; Pass B applies boundary/furniture trims.
- `SOURCE_FURNITURE_MASKS` (`src/facsimile/reflow.rs`, around lines 36–606) is a
  `const` slice of `SourceFurnitureMask { source, page, left, top, right, bottom }`
  records, with `left/top/right/bottom` in **normalized, half-open full-page
  raster coordinates** (fractions of the full page width/height, written as
  `pixel / raster_dim` so the intent is legible). `apply_source_furniture_mask`
  paints that rectangle white (`Luma([255])`); `mask_known_source_furniture`
  applies every record matching `(source, page)`. Masks run on the **full-page
  raster before column detection**.
- `src/facsimile/lines.rs::line_bands(col, LINE_MIN_INK)` (`LINE_MIN_INK = 3`)
  splits the cropped column into maximal runs of rows whose dark-pixel count
  exceeds 3. `drop_furniture` removes bands centered in the top ~8 % / bottom
  5 % zones and drops a leading/trailing band separated by an outlier gap. It
  does **not** touch interior gaps. `cut_points(bands)` returns the first band's
  top, each interior **gap midpoint**, and the last band's bottom — so each strip
  carries its line plus the full half-gap above and below it.
- `src/facsimile/reflow.rs::strips_in_reading_order` turns each `[cut[j], cut[j+1])`
  interval into a `Strip`; `scale_strip` crops exactly that interval and scales
  it; `place_line_images` stacks the scaled strips top-to-bottom at their **full
  height**. There is no cap on an interior gap: whatever blank rows sit between
  two bands are reproduced verbatim, split as half-gaps between the two adjacent
  strips.
- No existing mechanism compacts an interior blank between two legitimate body
  lines. The prior repair plan
  `llm_artifacts/2026_10_11_20_01_INFELICITIOUS_ARTIFACTS_MINIMAL_REPAIR_PLAN.md`
  states this explicitly ("removing ink does not compact blank rows by itself";
  "Do not improvise a generic row-compaction algorithm"). Its gap repairs all
  work by masking a **bridge** (a vertical scan rule / scan edge) that had merged
  several rows — plus annexed furniture/margin — into one tall band; once the
  bridge is masked, normal banding re-splits the rows and the annexed
  furniture/margin falls outside the body bands. Our case has **no bridge**: the
  bleed forms its own genuine bands between two real paragraphs.
- Measured evidence for page 21 (300-DPI render, raster **2000 × 2975**; Latin
  column is the right column):
  - Real Latin above the slab (`… tradit agi.`) ends by raster row ≈ **2018**.
  - The bleed slab is faint: in a column-ink row profile it reads ≈ 34–36 dark
    px/row, versus ≈ 270 dark px/row for real Latin lines — about 8× fainter, but
    still far above `LINE_MIN_INK = 3`, which is why it currently bands and
    renders. The slab spans raster rows ≈ **2021 → ~2200** (five broken lines).
  - Real Latin of §18 (`XVIII. Sorte ducta …`) resumes by raster row ≈ **2204**.
  - So there is clean whitespace margin on both sides of the slab; no real glyph
    row lies inside `[~2020, ~2200)`.
  - These coordinates are **starting estimates for one 300-DPI render**; Phase 1
    re-measures them at production scale and pins the final interval.
- The ordinary `cargo test` suite runs from tracked fixtures and synthetic
  inputs; source-PDF-dependent checks are `#[ignore]`d and named explicitly.
  `src/facsimile/audit_repair_tests.rs` (included inside `reflow.rs`'s test
  module) provides the production-trace helpers `audit_traced_unit(id)`,
  `production_body_scale()`, `assert_reviewed_source_lines_survive(...)`, and
  `audit_save_unit_trace(...)`, used by the existing `backlog_*` ignored tests.
- The test module already carries the machinery for **deleted rows**. The
  test-only `RetainedSourcePageTrace` (in `reflow.rs`'s `mod tests`, ≈ line 4376)
  has a `deleted_rows: Vec<[u32; 2]>` field documented as "half-open `[top,
  bottom)` row intervals deleted by masking/**compaction**, in original-source
  coordinates" and a `map_row(row) -> Option<u32>` that returns `None` for a
  deleted row and otherwise subtracts the height of every deletion above it — i.e.
  it maps an original-source row to its row in the retained (compacted) column.
  `assert_source_region_pixels_survive` compares `original.get_pixel(x, y)` with
  `retained.get_pixel(x - crop[0], trace.map_row(y))`, and
  `assert_reviewed_source_lines_survive` uses `map_row(top)` / `map_row(bottom-1)`
  plus an `assert_eq!(mapped_bottom - mapped_top, bottom - top, "no deleted Latin
  rows")` guard. The reviewed `region` is given in **original-source** coordinates
  (its `x` is checked against `trace.crop`, its `y` is an original-source row).
  **Today `audit_traced_unit` always sets `deleted_rows: vec![]`** — production has
  no row-deletion/compaction step yet — so `map_row` is currently the identity
  (minus the horizontal crop). `crop_column` crops horizontally only and keeps full
  height, which is why retained bands are in original-source row coordinates.

## Why a mask is not enough

Whiting out the slab with a `SOURCE_FURNITURE_MASKS` entry removes the bleed
**ink**: the faint bands disappear. But it leaves the **space**. After masking,
`line_bands` sees one large blank interval between the `… tradit agi.` band and
the `XVIII …` band. `cut_points` puts a single cut at that interval's midpoint,
so the `… tradit agi.` strip carries ≈ half of the ~180 px blank as its trailing
half-gap and the `XVIII …` strip carries the other half as its leading half-gap.
`place_line_images` stacks both at full height, so ≈ four blank lines of space
survive between the two paragraphs. `drop_furniture`/`drop_gap_outliers` only act
on leading/trailing bands, never interior gaps, so nothing collapses it. Removing
the space therefore requires removing the **rows**, not just the ink.

# Desired end state

1. A new, distinct, coordinate-pinned mechanism — call it **source row
   excision** — exists alongside `SOURCE_FURNITURE_MASKS` in
   `src/facsimile/reflow.rs`: a `const` table of
   `SourceRowExcision { source, page, top, bottom }` records in normalized
   half-open full-page raster coordinates, an `apply_source_row_excision` that
   returns a new `GrayImage` with rows `[top, bottom)` deleted (the rows below
   shifted up, width unchanged), and an `excise_known_source_rows(source, page,
   img)` dispatcher. It runs in `render_and_band` **immediately after**
   `mask_known_source_furniture` and **before** `production_source_column`.
2. Exactly one excision record is installed: `source == 0`, `page == 21`, with a
   `[top, bottom)` interval measured in Phase 1 that lies strictly inside the
   bleed-only slab and leaves exactly one ordinary inter-line gap between §17 and
   §18. No real Latin glyph row is deleted.
3. In the rebuilt `build_facsimile/tomus_1_inner.pdf`, the Theseus §17/§18
   transition shows `… tradit agi.` immediately followed, after one normal line
   gap, by `XVIII. Sorte ducta quum accepisset ex Prytaneo Theseus …`, with **no
   bled strokes and no multi-line blank** between them. Every real Latin glyph of
   §17 and §18 is present exactly once, byte-identical to the source crop.
4. The test-only trace builder `audit_traced_unit` records each excised interval
   as the page's `deleted_rows` (original-source coordinates), so the existing
   `map_row` / `assert_source_region_pixels_survive` / `assert_reviewed_source_lines_survive`
   helpers validate the compacted output without new coordinate math.
5. Regression coverage: an ordinary synthetic unit test proving the excision
   primitive deletes the right rows and leaves an unconfigured page untouched;
   an `#[ignore]`d production-trace test proving, on real page 21, that the bleed
   bands are gone, the two real lines survive once (byte-identical pixels), and the
   collapsed gap meets a numeric bound; and confirmation that the three existing
   page-17 Theseus ignored tests and the mask-ledger tests still pass unchanged. No
   global threshold, body scale, CLI, dependency, or audit-algorithm change is
   introduced.
6. Local formatting, strict Clippy, and the full ordinary `cargo test` pass. The
   eight inner and eight imposed PDFs are rebuilt and the §17/§18 transition is
   eyeballed. A dated verification note under `llm_artifacts/` records the pinned
   interval, before/after locators, and test results.

# Tech Learnings

- **Normalized half-open furniture coordinates.** The existing masks store
  `pixel / raster_dim` fractions and convert back with
  `(frac.clamp(0,1) * dim).round()`. Writing the literal division (e.g.
  `2035.0 / 2975.0`) keeps the source pixel legible in the code. Row excision
  reuses this convention for `top`/`bottom` only (full width is implied).
- **The `image` crate.** A `GrayImage` is `ImageBuffer<Luma<u8>, Vec<u8>>`.
  `image::imageops::crop_imm(img, x, y, w, h).to_image()` copies a sub-rectangle
  into a new owned image; `image::imageops::overlay(&mut dst, &src, x, y)` pastes
  one image onto another. To delete rows `[t, b)` from an `h`-row image: make a
  new `GrayImage` of height `h - (b - t)`, overlay `crop_imm(img, 0, 0, w, t)` at
  `(0,0)` and `crop_imm(img, 0, b, w, h-b)` at `(0, t)`. This "vertical splice"
  is the whole primitive.
- **Why excision and not a gap cap.** `line_bands` → `cut_points` →
  `scale_strip` → `place_line_images` is a shared pipeline used by every page;
  capping interior gaps there would be a global behavior change (the reader
  deliberately preserves authentic source spacing elsewhere). Deleting rows from
  one reviewed `(source, page)` raster is a local, auditable exception that reuses
  the entire downstream pipeline unchanged — the column simply no longer contains
  those rows.
- **This is the "compaction" the trace layer already anticipates.** The test-only
  `RetainedSourcePageTrace.deleted_rows` / `map_row` and the
  `assert_source_region_pixels_survive` / `assert_reviewed_source_lines_survive`
  helpers were written to validate exactly a masking/**compaction** row deletion.
  The plan therefore **reuses** that machinery (by populating `deleted_rows` in the
  trace builder) rather than inventing a parallel coordinate mapping in the new
  test — do not hand-roll row-shift arithmetic that `map_row` already performs.

# Phase 1 — Measure and pin the excision interval (Suggested implementer: Opus)

## 1.1 Trace page 21 at production scale and fix `[top, bottom)`

Goal: replace the Phase-0 single-render estimates with production-scale
measurements and a pinned normalized interval, before writing any production
code.

Steps:

1. In `src/facsimile/audit_repair_tests.rs` (included in `reflow.rs`'s test
   module), add an `#[ignore = "requires source PDFs and production-DPI renders"]`
   diagnostic test `theseus_p21_bleedthrough_trace`. Reuse `audit_traced_unit("theseus")`
   and `production_body_scale()`; do not re-implement rendering, banding, or
   composition. From the returned `PageBands` for source-0 page 21, dump, for the
   region `[1900, 2300)` of the cropped Latin column: the per-row ink profile
   (`ink_profile_rows` over `INK_THRESHOLD`), the raw and kept `line_bands`, and
   each band's ink/row. Save the trace via `audit_save_unit_trace` under the
   existing `tmp/pdfs/inner_tomi_audit*/` convention (group
   `theseus_p21_bleedthrough`). Also save a 300-DPI PNG crop of the full-page
   raster rows `[1850, 2300)` for human inspection.
2. From that trace, record precisely, in full-page raster coordinates:
   - `tradit_agi_bottom`: the last real-glyph row of `… tradit agi.` (bottom of
     the band just above the slab, including descenders).
   - `xviii_top`: the first real-glyph row of `XVIII. Sorte ducta …` (top of the
     band just below the slab, including the `X` ascender).
   - the slab's faint-band extent between them.
3. Choose the excision interval `[top, bottom)` so that: `top > tradit_agi_bottom`
   with a small whitespace margin; `bottom < xviii_top` with a small whitespace
   margin; and the **residual** whitespace left after deletion (i.e.
   `(top - tradit_agi_bottom) + (xviii_top - bottom)`, after excising `bottom-top`
   rows) equals **one ordinary inter-line pitch** for this unit (median line
   height + median gap, available from the `LineMetrics` returned by
   `audit_traced_unit`). Convert `top`/`bottom` to normalized fractions of the
   page height (denominator = the actual page-21 raster height, expected 2975),
   written as `pixel / raster_h` literals.
4. Record the pinned values in this plan (replace the estimates in "Current state
   of the world" in place) and in the Phase-1 trace output: the raster
   dimensions, `tradit_agi_bottom`, `xviii_top`, the chosen `[top, bottom)` in
   both pixels and normalized form, and the predicted residual gap in pixels and
   in line-pitches.

Acceptance: the pinned interval contains **no** real-glyph row (verified against
the measured band boundaries with ascender/descender margins) and the predicted
residual gap is within ±0.25 line-pitch of one pitch.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`;
`cargo test theseus_p21_bleedthrough_trace -- --ignored --nocapture`; `cargo build`.
Confirm the printed band boundaries match the recorded `tradit_agi_bottom` /
`xviii_top` and that `[top, bottom)` falls strictly between them. Run
`git diff --check` and read the new test directly (an untracked file is absent
from an ordinary diff).

**Human verification:** Open the saved `[1850, 2300)` crop and confirm by eye
that the chosen `[top, bottom)` covers only bled strokes — no part of `tradit
agi.` or `XVIII …` glyphs — and that the slab really is bleed-only (compare with
`Ploutarchou_Bioi_v1.pdf` page 21 and the reverse leaf, page 22).

# Phase 2 — Install the row-excision mechanism (Suggested implementer: Opus)

## 2.1 Add the excision primitive and table, wire it into `render_and_band`

In `src/facsimile/reflow.rs`, next to `SOURCE_FURNITURE_MASKS` and its helpers,
add:

1. A struct `SourceRowExcision { source: usize, page: u32, top: f64, bottom: f64 }`
   (doc comment: normalized half-open full-page-raster row interval deleted from
   the page before column detection; used to remove a reviewed bleed-only slab and
   close the space it occupied).
2. A `const SOURCE_ROW_EXCISIONS: &[SourceRowExcision]` with the single Phase-1
   record for `source: 0, page: 21` and the pinned `top`/`bottom` literals (as
   `pixel / 2975.0`). Precede it with a comment naming the Theseus §17/§18
   bleed-through.
3. `fn apply_source_row_excision(excision: &SourceRowExcision, img: &GrayImage) -> GrayImage`.
   Guard exactly like `apply_source_furniture_mask`: if any coordinate is
   non-finite or `top >= bottom`, return `img.clone()` unchanged. Convert `top`/
   `bottom` to rows with `(frac.clamp(0.0, 1.0) * h).round() as u32`, clamp to
   `[0, h]`, and if the resulting `t >= b` return `img.clone()`. Otherwise build a
   new `GrayImage` of the same width and height `h - (b - t)` via the vertical
   splice described in Tech Learnings (overlay the `[0, t)` rows at `y = 0` and the
   `[b, h)` rows at `y = t`).
4. `fn excise_known_source_rows(source: usize, page: u32, img: GrayImage) -> GrayImage`
   that deletes every matching record's rows from `img`. Require the records for a
   given `(source, page)` to be **sorted ascending and non-overlapping** — the same
   invariant the trace's `deleted_rows` already documents — and build the result in
   a single pass by copying the surviving spans (between/around the deletions) into
   one new image, rather than deleting intervals one at a time (avoids repeated
   allocation and off-by-one row-shift bugs). With no matching record, return `img`
   unchanged.
5. In `render_and_band`, immediately after
   `mask_known_source_furniture(source, page, &mut page_img);`, replace the local
   with the excised image:
   `let page_img = excise_known_source_rows(source, page, page_img);`
   (then continue with `production_source_column(source, page, &page_img)?` etc.).
   Nothing downstream changes.

Add an ordinary (non-ignored) unit test module entry
`source_row_excision_splices_configured_rows`: build a synthetic `GrayImage`
(e.g. 20×30) with row `y` filled to value `y`, apply `apply_source_row_excision`
with an interval covering a known normalized row span, and assert: the output
height is `h - (b - t)`; the rows above `t` are byte-identical; the rows at/after
`t` equal the original rows at/after `b` (the splice shifted them up); width is
unchanged. Add `source_row_excision_absent_leaves_image_unchanged`: assert
`excise_known_source_rows(0, 999, img)` (no record) returns an image equal to the
input, and that a degenerate interval (`top == bottom`, or `top > bottom`) is a
no-op clone.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`;
`cargo test source_row_excision_splices_configured_rows`;
`cargo test source_row_excision_absent_leaves_image_unchanged`;
`cargo test` (full ordinary suite — confirm no synthetic banding/compose test
regressed); `cargo build`.

**Human verification:** None (pure mechanism; behavior is verified in 2.2 and
Phase 3).

## 2.2 Teach the trace builder about excised rows

The survival helpers map original-source rows through `trace.map_row`, so they
only work once `audit_traced_unit` records the excised interval as `deleted_rows`.
Update `audit_traced_unit` (in `audit_repair_tests.rs`) so that, for each page
`p`, it sets `deleted_rows` to the `(source, p)` excision intervals expressed in
**original-source row pixels** — `[round(top * h), round(bottom * h))` using `h` =
the **pre-excision** page-raster height (the height of the freshly rendered
`real_source_image(source, p)`, before any excision), sorted ascending. The `crop`
edges come from `production_source_column` on the masked raster and are
**unchanged** by excision (it removes rows only, never columns), so leave that line
as-is. Pages with no excision record keep `deleted_rows: vec![]`. Factor the
normalized-interval → row-pixel conversion into one small helper the trace builder
and the Phase-2.3 test both call (do not duplicate the arithmetic).

This makes `map_row` subtract the excised height for every original row below the
slab, so the `XVIII …` region maps to its compacted row and the
`assert_eq!(mapped_bottom - mapped_top, bottom - top, "no deleted Latin rows")`
guard confirms no glyph row was excised.

**LLM verification:** `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`; `cargo build`;
`cargo test repair_trace_maps_skipped_pages_and_deleted_rows` (the existing
`map_row` unit test must still pass). No behavior change to ordinary tests.

**Human verification:** None.

## 2.3 Prove the real-page effect with a production-trace test

In `src/facsimile/audit_repair_tests.rs`, add an
`#[ignore = "requires source PDFs and production-DPI renders"]` test
`backlog_theseus_p21_bleedthrough_excised`. Using `audit_traced_unit("theseus")`
(now carrying the page-21 `deleted_rows`) and `production_body_scale()`:

1. **Ink gone.** Via `retained_page(&traces, 0, 21)`, confirm the trace's
   `deleted_rows` contains the pinned interval, and assert there is no retained
   band between the `… tradit agi.` band and the `XVIII …` band — the bleed bands
   are gone and the two paragraphs are now adjacent bands with a single gap.
2. **Glyphs preserved (faithfulness gate).** Build two `ReviewedSourceLine` values
   inline (no new JSON fixture for one page) for the `… tradit agi.` line and the
   `XVIII. Sorte ducta …` line, with `region` boxes in **original-source**
   coordinates from Phase 1 (full ascenders/descenders; `x` within the column
   crop). Call `assert_reviewed_source_lines_survive(..., false)` with a
   `ReviewedLineContext` built from the traced unit and `production_body_scale()`.
   The helper asserts each line survives exactly once, maps through `map_row`, and
   that `assert_source_region_pixels_survive` finds byte-identical source pixels in
   the compacted column; the `XVIII` box exercises the 2.2 `deleted_rows` shift.
3. **Gap collapsed, not fused.** Compute the residual gap in **compacted column
   rows** (not composed output, to avoid page-break ambiguity): `map_row` the
   `XVIII` band top and the `… tradit agi.` band bottom and assert their difference
   is `> 0` and `≤` the bound frozen in Phase 1 (one line-pitch + a stated
   tolerance from the unit's `LineMetrics`).

Record the pre-excision baseline for comparison: with the excision record
temporarily removed, assertion 3's bound fails (~4 line-pitches) and assertion 1
finds bleed bands; capture that for the Phase-3 note but commit only the passing
form.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`;
`cargo test backlog_theseus_p21_bleedthrough_excised -- --ignored --nocapture`;
`cargo build`. Confirm the test executes and passes (not merely selected).

**Human verification:** Read the trace output: confirm the two real lines are the
ones surviving and the removed region contained only bled strokes.

## 2.4 Confirm the excision does not disturb the rest of Theseus

Excising ~180 px from page 21 shifts every Theseus body canvas composed from page
21 onward; earlier canvases (including the I1 orphan-line repair on page 17) are
unaffected. The new excision is **not** a mask, so it stays outside the
`SOURCE_FURNITURE_MASKS` ledger and needs no page-21 entry in
`inner_tomi_masks_match_reviewed_ledger` / `inner_tomi_known_masks_preserve_latin`
— run those only to confirm they still pass unchanged (page 21 carries no mask).

Run the three existing Theseus-touching ignored tests —
`backlog_theseus_orphan_line_rejoins_body` (I1, page 17),
`theseus_p17_analysis_keeps_original_latin_pixels`, and
`theseus_page_17_followup_trace` — and confirm they still pass; all three are
page-17-scoped and must be unaffected. If any other ignored test asserts a Theseus
**unit-total** canvas/page count or canvas content at or after page 21, update only
the genuinely shifted numeric expectation (with the Phase-3 human review), never a
glyph-preservation assertion.

**LLM verification:** `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`;
`cargo test backlog_theseus_orphan_line_rejoins_body -- --ignored`;
`cargo test theseus_p17_analysis_keeps_original_latin_pixels -- --ignored`;
`cargo test inner_tomi_masks_match_reviewed_ledger`;
`cargo test inner_tomi_known_masks_preserve_latin -- --ignored`;
`cargo test` (full ordinary suite). Require executed, passing tests.

**Human verification:** None beyond Phase 3's full-build eyeball.

# Phase 3 — Rebuild and record (Suggested implementer: Sonnet)

## 3.1 Full build and verification note

Preserve the current `build_facsimile/tomus_1_inner.pdf` §17/§18 region as a
before-crop, then run the full offline build (README Windows command):

```powershell
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
& $env:PLUTARCH_LUALATEX --version
if ($LASTEXITCODE -ne 0) { throw "LuaLaTeX is unavailable; fix the MiKTeX path or installation before building." }
cargo run --release --bin facsimile --target-dir target -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
if ($LASTEXITCODE -ne 0) { throw "The facsimile build failed; inspect the error above." }
Get-ChildItem build_facsimile/tomus*.pdf
```

Reject `lualatex not run` as an incomplete PDF build even on a zero exit code;
confirm all sixteen PDFs exist and are freshly written. Locate the Theseus
§17/§18 transition in the rebuilt `tomus_1_inner.pdf` (it was physical page 30 in
the pre-change build; it may shift) and save an after-crop.

Because deleting ~180 px from one Theseus page slightly shortens that unit's body,
the Theseus page count can change by at most one canvas, which can ripple into
recto alignment, the signature-padded total, and `split_into_volumes` membership.
Record — do not force — these: Theseus body-canvas count and first-page/folio,
the inner-PDF total and trailing signature-blank count, and whether volume
membership changed. A padded total may stay the same or cross a 32-page boundary;
do not require a specific delta.

Write a dated note `llm_artifacts/YYYY_MM_DD_HH_MM_THESEUS_P21_BLEEDTHROUGH_VERIFICATION.md`
recording: the pinned `[top, bottom)` (pixels + normalized), raster dims,
`tradit_agi_bottom`/`xviii_top`, before/after reader locators and crops, the
baseline-failure vs. post-fix pass evidence for `backlog_theseus_p21_bleedthrough_excised`,
the full test/build results, and the (unforced) page-count/membership observations.
Keep this plan and the note as historical records.

**LLM verification:** `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`; `cargo test`;
`cargo test backlog_theseus_p21_bleedthrough_excised -- --ignored`;
`cargo build --release --bin facsimile`; the full offline build above; confirm
sixteen fresh PDFs and successful lualatex logs. `git diff --check`; read the new
note directly.

**Human verification:** Compare the before/after crops of the Theseus §17/§18
transition at full resolution. Confirm the bled strokes are gone, `… tradit agi.`
is followed by one normal line gap and then `XVIII. Sorte ducta quum accepisset
ex Prytaneo Theseus …`, every §17 and §18 word is intact and unduplicated, and no
new clipping or fusion was introduced. Skim a few other Theseus pages to confirm
nothing else shifted unexpectedly.
