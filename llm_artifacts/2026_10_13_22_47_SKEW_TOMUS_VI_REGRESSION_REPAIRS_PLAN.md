# Skew / Tomus VI regression repairs — implementation plan (2026-10-13)

## Context

The branch `fix/tomus-1-gaps-split-shortending` installed five reviewed facsimile
repairs for skewed scans and Tomus VI (commits `4c223ee`…`508ee34`; see
`llm_artifacts/2026_10_13_19_00_SKEW_AND_TOMUS_VI_REPAIRS_PLAN.md` and its
verification note `…23_30_SKEW_AND_TOMUS_VI_REPAIRS_VERIFICATION.md`). A human
visual review of the rebuilt inner PDFs now shows that **four** of the reviewed
symptoms are either still present or were made worse by those repairs:

1. **Tomus III (v1 p448, reader p121) — URGENT regression.** The Latin line
   `subito illis nihil tale exspectantibus, sed propter victoriam` has
   **disappeared entirely** from the reader. The text jumps from
   `… superbe Gythium oppugnabat` straight to `secure agentibus`, skipping the
   intervening line. The failure occurs in the sheared-page pipeline, but the
   exact stage that loses it is not yet established. **Restoring this line is
   the most urgent item in this plan.**
2. **Tomus I (v1 p34, reader ≈ pdf 57 / folio 43) — not fixed.** The gap
   **before** `III. Ceterum receptissimae` is still present. The installed shear +
   gutter mask addressed the left-gutter ink fragments but not the gap the reader
   actually sees.
3. **Tomus VI (v2 p231, reader pp86–87) — not fixed.** The horizontal split on
   the final line still leaves `quidem` (the first word) straddling the two
   pages. Phase 0 of the prior plan classified this `no_repair_required`; that
   classification was wrong for the composed reader.
4. **Tomus VI (v2 p398, reader p394) — new gap introduced.** The original gap
   after `… qui summae rerum consi-` is fixed, but the whole-strip repair that
   fixed it introduced a **new** gap on the same reader page, following
   `agerentur pensitantis`.

The one prior-plan defect that is confirmed fixed and needs no further work is
**v2 p266 (reader p158)** right-edge clipping.

The reader must reproduce the Latin from the printed source **faithfully, once,
and in source order**. The transcription cache is out of scope and must not be
touched. Every repair here stays page-scoped and ledger-recorded, consistent with
the rest of this branch; no global threshold, body-scale, column-detector, or
composer-policy change is permitted.

> **Edition index.** `source 0` = `Ploutarchou_Bioi_v1.pdf`; `source 1` =
> `Ploutarchou_Bioi_v2.pdf`. "Source page N" is the 1-based page of that scan.
> The Greek is the **left** column of each source page, the Latin the **right**;
> a thin divider rule sits between. The generated readers are the eight
> `tomus_*_inner.pdf` files.

## Current state of the world

The source-repair pipeline lives in `src/facsimile/reflow.rs`.
`render_and_band` (`reflow.rs:1266`) runs, in order:

```
shear  →  mask furniture  →  excise rows  →  crop Latin column  →  band  →  retain reviewed body bands
```

concretely:

```rust
let mut page_img = apply_known_source_shear(source, page, page_img);   // SOURCE_SHEARS
mask_known_source_furniture(source, page, &mut page_img);              // SOURCE_FURNITURE_MASKS
let page_img = excise_known_source_rows(source, page, page_img);       // SOURCE_ROW_EXCISIONS
let column = production_source_column(source, page, &page_img)?;        // + preserve_known_body_crop_edge
let cropped = crop_column(&page_img, &column);
let (raw, bands) = band_source_column(source, page, &cropped);         // line_bands → drop_furniture
let bands = retain_known_source_body_bands(None, source, page, cropped.dimensions(), &raw, &bands); // SOURCE_BODY_BANDS
```

Key facts that this plan depends on:

- **Banding and furniture dropping.** `band_source_column` (`reflow.rs:1300`)
  calls `line_bands` (`lines.rs:50`), then `coalesce_known_body_fragments`, then
  `drop_furniture` (`lines.rs:75`), then `retain_early_aligned_body_bands`.
  `drop_furniture` first discards bands whose centre is in the top-furniture zone
  (`TOP_FURNITURE_FRAC` of page height) or bottom zone, then calls
  `drop_gap_outliers` (`lines.rs:91`), which removes a **detached first or last
  band** whose adjoining gap exceeds `GAP_OUTLIER_FACTOR ×` the median inter-line
  gap. On a straightened (sheared) page this can prune a real first body line.
- **Reviewed body-band rescue.** `retain_known_source_body_bands`
  (`reflow.rs:2441`) runs *after* `drop_furniture` and re-adds a reviewed
  `SourceBodyBand.protected` band whenever the raw banding contained its
  `observed` (or `protected`) band. `SOURCE_BODY_BANDS` (`reflow.rs:1470`) is the
  existing, page-scoped mechanism for exactly the "drop_furniture dropped a real
  body line" case.
- **Composition.** `compose_body_canvases` (`reflow.rs:3276`) packs scaled strips
  greedily onto fixed-height canvases. A strip classified `BandClass::Tall` is
  **fill-split** at a low-ink row to finish a page (`reflow.rs:3331`), *unless*
  `is_known_whole_source_strip` (`reflow.rs:3153`, backed by `WHOLE_STRIPS` at
  `reflow.rs:2985`) marks it whole. A whole (non-`Tall`) strip that does not fit
  the remaining space is **not** split: the current canvas is closed and the strip
  is retried on a fresh canvas (`reflow.rs:3348`), leaving the previous page's
  bottom blank.
- **The regression blind spot.** The skew/Tomus VI regressions
  (`romulus_chapter_break_deskewed`, `philopoemen_victory_line_deskewed`,
  `tomus6_quidem_line_complete`, `tomus6_consi_continuation_gap_closed`,
  `tomus6_p266_right_glyphs_restored`, in `audit_repair_tests.rs`) asserts only in
  the **band / crop / pixel domain** (the output of `render_and_band` and
  `assert_reviewed_sheared_page` at `audit_repair_tests.rs:3873`). **None of them
  composes its affected unit.** The current `philopoemen_victory_line_deskewed`
  test already requires the exact `[330,366]` band; the prior verification note
  says that same band was dropped. Phase 0 must resolve this contradiction before
  naming the loss stage. A `Tall` fill-split that orphans a word and a whole-strip
  bump that leaves a page-bottom gap are invisible to band-domain assertions.
  The two existing composer-level tests
  (`audit_repair_tests.rs:4553`, `:6110`) run the slow Tier-B
  `assert_reviewed_source_lines_survive` path, which pulls `production_body_scale()`
  (whole-corpus sampling, >10 min), so the skew/Tomus VI repairs deliberately
  avoided them.
- **Frozen production scale.** `production_body_scale()` is frozen at `1.2023952`
  for the current PDFs. A composer-level check can pass this literal `f32` directly
  and compose only the affected unit's pages (via `audit_traced_unit`), keeping the
  test near ~1 min instead of >10 min.

## Desired end state

- The Philopoemen victory line `subito illis nihil tale exspectantibus, sed
  propter victoriam` is present **once**, in source order, in Tomus III, with its
  reviewed source pixels intact.
- The gap before `III. Ceterum receptissimae` in Tomus I is reduced to an ordinary
  inter-line gap (or the heading moves to the top of a page), with no left-gutter
  fragments.
- `quidem passus Callisthenem` renders as a complete line on one reader page in
  Tomus VI, with no part of `quidem` orphaned across the pp86–87 break.
- Tomus VI reader p394 reads continuously from `agerentur pensitantis` through
  `… qui summae rerum consi-` to its continuation, with no injected page-bottom gap
  and no mid-word page split.
- A **fast composer-level regression** exists for each repaired defect, so a
  future band-domain-only change cannot silently reintroduce a dropped line, an
  orphaned word, or a page-bottom gap.
- `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`, every ignored repair regression, and the offline facsimile rebuild
  pass locally; the rebuilt Tomi I, III, and VI pages are visually compared with
  the source scans.

## Tech Learnings

- **Band domain vs. composer domain.** `render_and_band` decides *which source
  rows are Latin lines*; `compose_body_canvases` decides *how those lines are
  packed onto fixed-height reader pages*. A line can survive the first and still be
  dropped, split, or stranded by the second. Faithfulness therefore has to be
  checked at both layers.
- **`drop_gap_outliers` after a shear.** A shear straightens baselines but also
  changes which band is "first". If the straightened first body line sits above a
  larger-than-median gap (e.g. a chapter break or a running-head remnant just below
  it), `drop_gap_outliers` treats it as detached furniture and removes it. The
  reviewed-body-band rescue (`SOURCE_BODY_BANDS`) is the page-scoped antidote.
- **Whole-strip wrapping is atomic.** Marking a merged `Tall` strip whole forces
  *all* of its lines to move together. For a strip that is really one or two lines
  this is harmless; for a strip that is a multi-line bridge-merge it bumps a whole
  block to the next page and leaves a large page-bottom gap. The height of the
  strip tells you which case you are in: one body line is ≈ 33–40 raw px.

## The four live defects

| # | Short name | Source | Reader | Symptom | Prior disposition |
|---|-----------|--------|--------|---------|-------------------|
| A | Philopoemen victory line dropped | v1 p448 | Tomus III p121 | `subito illis … victoriam` missing entirely | `shear` (+ masks) |
| B | `III. Ceterum` gap | v1 p34 | Tomus I ≈ pdf 57 / folio 43 | gap before the heading persists | `shear` + gutter mask |
| C | `quidem` straddle | v2 p231 | Tomus VI pp86–87 | `quidem` orphaned across the page break | `no_repair_required` |
| D | `consi-` new gap | v2 p398 | Tomus VI p394 | new page-bottom gap after `agerentur pensitantis` | `whole_strip` |

## Working diagnoses to verify in Phase 0

### A — Philopoemen victory line dropped (v1 p448) — URGENT

The p448 shear (`SOURCE_SHEARS`, slope `+0.03048780487804878`, `x0=1400`,
`top_pad=18`) straightens the baselines so `line_bands` resolves the formerly
merged column into individual lines. The prior verification note says the
post-shear band `[330,366]` is dropped and `[380,415]` becomes the first retained
body band. But `philopoemen_victory_line_deskewed` already requires both exact
bands through `assert_reviewed_sheared_page`. These claims cannot both describe
the same test run. The reviewed region `[909,315,929,1050]` also spans many lines;
its overlap alone does not identify the missing victory line. The first task is
to run that ignored test, identify the victory line by its source pixels, and
trace the band through raw banding, Pass A, Pass B, strip selection, and composition.

**Fix direction:** retain the shear if the trace confirms it preserves the
source pixels. If the victory line exists in raw banding but is dropped in Pass A
or Pass B, add a reviewed `SOURCE_BODY_BANDS` entry for `(0, 448)` with the exact
observed and protected bands. If it is absent from raw banding or is present in
the composed canvas but lost later, repair the measured page-scoped stage instead.
The regression must prove the specific line survives once in source order.

### B — gap before `III. Ceterum receptissimae` (v1 p34)

The p34 shear + sheared-coordinate gutter mask removed the left-gutter ink
fragments (the symptom the prior plan scoped) but not the gap the reader sees. The
gap is the chapter-break whitespace: in the source a large blank band (the prior
work measured a ≈252 px chapter gap) precedes the `III.` heading. After banding it
survives as an oversized inter-line gap and the composer reproduces it at scale,
reading as a gap before the heading. The shear does not change the size of a blank
band.

**Fix direction:** confirm in Phase 0 whether the gap is (i) oversized blank rows
between the preceding body line and the heading band, or (ii) a composer
page-fill artifact. If (i), add a page-scoped `SOURCE_ROW_EXCISIONS` entry on p34
that deletes excess blank rows down to an ordinary inter-line gap, measured in
**post-shear** coordinates. Row excision changes the cropped height. The existing
`band_source_column` special case for `(0, 34, (828, 3040))` will therefore stop
matching; update that guard to the measured post-excision dimensions and a
fingerprint of unchanged source pixels, retaining its horizontal text window
`(60, 798)`. Remeasure every affected p34 band and fixture coordinate. Do not
shrink the gap below a normal inter-line gap or delete any Latin row.

### C — `quidem` straddle (v2 p231)

Phase 0 of the prior plan found in the band domain that `quidem passus
Callisthenem` bands as a complete line (strip `[2224,2317]`, ≈112 scaled px) and
concluded `no_repair_required`. The reported reader break is consistent with a
`Tall` fill-split, but its exact cut point and canvas placement have not been
recorded. The band-domain completeness check (`tomus6_quidem_line_complete`)
cannot establish whether the composer cuts the line.

**Fix direction:** if the composed trace confirms that exact strip is fill-split,
add it to `WHOLE_STRIPS` with a source fingerprint so it wraps whole. It fits a
fresh body budget, but measure the resulting p86 bottom space rather than assuming
it is unobtrusive. Add a composer-level regression that identifies the `quidem`
pixels and proves the whole line occupies one canvas.

### D — `consi-` new gap (v2 p398)

The installed `WHOLE_STRIPS` entry wraps the strip `(818,2835) [1022,1837]`. Its
**815 raw px** height suggests many merged lines rather than one continuation
line. Wrapping a large block can leave substantial unused space on p394. Height
alone does not prove a removable divider, scan-edge, or bleed bridge caused the
merge. Phase 0 must measure the bridge and the composed gap before choosing a mask.

**Fix direction:** if a non-Latin bridge is found and a trial mask separates the
lines without erasing Latin pixels, install a guarded `SOURCE_FURNITURE_MASKS`
entry and remove the `(818,2835) [1022,1837]` whole-strip entry. Verify that the
new strips pack across p394/p395 without the original mid-word cut or a new large
gap. If no safe bridge mask exists, use the smallest page-scoped alternative that
the trace supports; do not assume a narrower whole-strip can be matched until
the relevant cut-point interval has actually been separated.

> **Cross-cutting note (C vs. D).** Wrapping a strip whole can trade a mid-unit
> split for page-bottom whitespace. Measure both outcomes in the composed reader:
> C concerns a short strip, while D concerns a much taller one. Record the
> chosen remedy and its measured spacing in each ledger entry.

## Phase 0 — confirm root causes and build the composer-level harness (Suggested implementer: Opus)

### 0.1 Add a fast composer-level assertion helper

Add a shared test helper in `src/facsimile/audit_repair_tests.rs` that composes a
single unit at the frozen scale and lets a test assert at the canvas level
without `production_body_scale()`. It must:

1. take a unit id, render + band it with `audit_traced_unit(<unit>)` (≈50 s, no
   whole-corpus scale),
2. use the `LineMetrics` returned by `audit_traced_unit` and `TrimGeom::default()`,
3. call `compose_pages(&pages, 1.2023952, &geom, geom.first_body_budget(),
   geom.body_budget(), &metrics)` so the production canvas ink floor is applied,
4. return the canvases and their strip/canvas ranges through the existing
   `ranges_for_composed_canvases` helper (already in the same `reflow::tests`
   module), together with the retained-source-page traces. Reuse
   `strip_and_canvas_ranges` where practical rather than duplicating its mapping.
   Because `compose_pages` may filter a low-ink canvas, compare its output with
   `compose_body_canvases` and require identical canvas counts for the four units
   before using the existing cumulative-range mapping. If a canvas is filtered,
   map surviving canvases by their image hashes instead of shifting provenance.

Name the helper so its intent is clear (e.g. `compose_unit_at_frozen_scale`).
`src/facsimile/mod.rs` subsequently calls `merge_reviewed_short_ending`; assert
that none of the four affected units is in `REVIEWED_SHORT_ENDINGS`, or apply
that step before interpreting canvas indices. Use the retained-source-page trace
to associate a strip with a source page. A canvas index within one unit is not a
physical reader PDF page number; resolve the latter during the rebuilt-PDF check.
Do not call `assert_reviewed_source_lines_survive` (the slow Tier-B path).

**LLM verification:** add a temporary `#[ignore]` smoke test that composes one
small unit through the helper and prints canvas count; run it with
`cargo test --release <smoke_name> -- --ignored`, then remove the smoke test.
Compare canvas hashes with the slow Tier-B path on one already verified unit once
to confirm the frozen scale and helper path agree. Run `cargo fmt`,
`cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Human verification:** none (infrastructure only).

### 0.2 Confirm each defect's mechanism and pin coordinates

Extend the ignored diagnostics in `audit_repair_tests.rs` (reuse
`skew_and_tomus6_repair_trace`). For each defect record the evidence in
`src/facsimile/fixtures/tomi_gap_split_repairs_source_trace.json`'s `diagnoses`
array (do not add to `repairs` until the matching table entry is installed):

- **A (v1 p448):** run `philopoemen_victory_line_deskewed` first and save its
  result. Print the post-shear `raw` and Pass-A `kept` lists from
  `band_source_column`, the Pass-B bands from `audit_traced_unit`, the selected
  strips, and the composed canvases. Resolve why the existing exact-band test
  requires `[330,366]` while the prior note reports it dropped. Identify the
  victory line by a tight source-pixel rectangle and a visual comparison with
  the scan; OCR may assist but is not the authority. Record the exact
  `observed`/`protected` bands in `(1079,3068)` coordinates only if a band is
  actually dropped, and identify the stage that drops it.
- **B (v1 p34):** print the post-shear bands around the `III.` heading, the gap
  (in rows) between the preceding body band and heading band, typical inter-line
  gaps, and the composed canvas boundary and whitespace. Decide blank-row
  excision vs. composer artifact. If excision applies, record the post-shear
  `[top,bottom)` deletion, resulting raster/crop dimensions, the unchanged-pixel
  fingerprint for the `band_source_column` p34 guard, remapped heading and
  neighboring bands, and every affected fixture expectation.
- **C (v2 p231):** compose the Alexander unit with the 0.1 helper; locate the
  `[2224,2317]` strip on the two canvases, record its actual cut row, and use
  reviewed tight source-pixel regions for `quidem` and its neighboring words to
  establish which pieces land on each canvas. Record its fingerprint region/hash,
  the body budget, and before/after bottom whitespace in scaled pixels.
- **D (v2 p398):** record the actual bands within `[1022,1837]`, the before
  canvas boundary and blank-space height, and tight source-pixel regions around
  `agerentur pensitantis`, `consi-`, and its continuation. Locate any connecting
  non-Latin pixels by x-range and row profile. Trial-mask only those pixels;
  compare all affected Latin pixels against the unmasked source, then record the
  new bands, composed boundary, and gap height. Record a source-fraction mask
  rectangle only if the trial proves the mask safe and effective.

**LLM verification:** `cargo test --release philopoemen_victory_line_deskewed -- --ignored`
and `cargo test --release skew_and_tomus6_repair_trace -- --ignored`; inspect the
printed bands, canvas measurements, and saved PNGs under `tmp/pdfs/skew/`. Run
`cargo fmt`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Human verification:** compare each tight source-pixel region and its claimed
word with the source scan, and review the four measured mechanism decisions.

## Phase 1 — URGENT: restore the Philopoemen victory line (Suggested implementer: Opus)

### 1.1 Restore the line at its measured loss stage and prove composition

Write the failing regression first, then fix:

1. `philopoemen_victory_line_present` (composer-level, using the 0.1 helper):
   take the tight, reviewed source-pixel rectangle identified in Phase 0.2;
   compare its scaled pixels with the appropriate composed canvas crop, require
   exactly one occurrence, and locate the neighboring source strips before and
   after it in that order. Assert the corresponding source-page provenance and
   require that no part of the line is on a second canvas. Confirm the test
   **fails** on the current tree for the reported missing line.
2. If Phase 0.2 finds a raw band dropped in Pass A or Pass B, add a
   `SOURCE_BODY_BANDS` entry for `(0, 448, (1079,3068))` with the measured
   `observed`/`protected` bands and `unit_id: "philopoemen"`. Otherwise implement
   the page-scoped repair for the measured loss stage and document its test and
   ledger signature before changing production code.
3. Keep the existing exact-band assertions in
   `philopoemen_victory_line_deskewed`; correct any stale expected band only
   after reconciling the prior verification note with the current ignored test.
   Add a tight victory-line pixel assertion, so an unrelated retained band cannot
   satisfy the test.
4. If the repair uses `SOURCE_BODY_BANDS`, add a `source_body_band` record to
   `tomi_gap_split_repairs_source_trace.json` with `unit_id`, source/page, raw
   `raster`, `preserved_latin_regions`, `column_dims`, and exact `observed` and
   `protected` `[top,bottom)` bands. Extend
   `tomi_gap_split_ledger_has_expected_shape` to validate these fields and add
   `inner_tomi_source_body_bands_match_reviewed_ledger` to compare this record
   with `SOURCE_BODY_BANDS` in both directions for the new p448 entry.

**LLM verification:** `cargo test --release philopoemen_victory_line_present -- --ignored`
(now passes), `philopoemen_victory_line_deskewed`,
`inner_tomi_known_masks_preserve_latin`, the new table/ledger check, the
ledger-shape test, `cargo fmt`,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Human verification:** in the rebuilt Tomus III, confirm
`… superbe Gythium oppugnabat / subito illis nihil tale exspectantibus, sed propter
victoriam / secure agentibus …` reads in order with the line present once.

## Phase 2 — `quidem` straddle (Suggested implementer: Opus)

### 2.1 Wrap the `quidem` line whole

1. Write `tomus6_quidem_not_split` (composer-level): match the scaled pixels
   of the reviewed `quidem` and following-word source regions to the composed
   canvases. Require the complete `[2224,2317]` strip on exactly one canvas,
   with no part on the preceding canvas; assert its source-page provenance and
   neighbors' order. Confirm it **fails** on the current tree.
2. If Phase 0.2 confirms a fill-split of that strip, add `[2224,2317]` to
   `WHOLE_STRIPS` with the fingerprint measured there; assert its scaled height
   fits a fresh page budget. If the split is elsewhere, repair that measured
   page-scoped cause and update the test and ledger disposition accordingly.
3. For the whole-strip remedy, add the matching `repairs` record (disposition
   `whole_strip`) and cross-check with
   `inner_tomi_whole_strips_match_reviewed_ledger`.
4. Keep `tomus6_quidem_line_complete` (band-domain completeness) as-is. Measure
   the resulting bottom whitespace on the preceding canvas and require it to
   be no greater than two median complete-strip heights for that unit; record
   the measurement in the ledger and verification note.

**LLM verification:** `cargo test --release tomus6_quidem_not_split -- --ignored`
(passes), `tomus6_quidem_line_complete`, the whole-strip ledger test, `cargo fmt`,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Human verification:** confirm `quidem passus Callisthenem` is one complete line
on Tomus VI p86 (or p87), with `quidem` not orphaned.

## Phase 3 — `consi-` gap: mask the bridge, drop the whole-strip (Suggested implementer: Opus)

### 3.1 Replace the whole-strip with a furniture mask

1. Write `tomus6_consi_page_reads_continuously` (composer-level): match tight
   scaled source-pixel regions for `agerentur pensitantis`, `consi-`, and its
   continuation against canvases exactly once and in source order. Locate their
   strips and require that no word is divided across canvases. For the canvas
   ending after `agerentur pensitantis` on the baseline, measure unused body
   height; after repair require no more than two median complete-strip heights
   unless that text has moved to the next canvas without an abnormal gap.
   Confirm the test **fails** on the current tree.
2. Only if Phase 0.2 proved a removable non-Latin bridge, add a page-scoped
   `SOURCE_FURNITURE_MASKS` entry for `(1, 398)` using its measured source-fraction
   rectangle. Check every Latin source pixel in the affected `[1022,1837]` region
   against the post-mask retained image, and verify the new individual bands.
3. Once the safe mask and composed-page regression pass, **remove** the
   `(818,2835) [1022,1837]` entry from `WHOLE_STRIPS` and its current `repairs`
   record; add a `mask` record with `mask_rectangles`, preserved Latin regions,
   and before/after gap measurements. Assert the removed signature is absent
   from `WHOLE_STRIPS`, since the existing ledger test checks installed records
   in only one direction. If no safe mask exists, document the
   measured alternative and its exact ledger/test change before implementing it.
4. Keep `tomus6_consi_continuation_gap_closed` only if it still describes a true
   post-mask expectation; otherwise replace its body with the continuous-read
   assertion. Do not leave a whole-strip assertion that no longer matches the
   installed mechanism.

**LLM verification:** `cargo test --release tomus6_consi_page_reads_continuously -- --ignored`
(passes), `inner_tomi_masks_match_reviewed_ledger`,
`inner_tomi_known_masks_preserve_latin`,
`inner_tomi_whole_strips_match_reviewed_ledger`, the ledger-shape test, `cargo fmt`,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Human verification:** confirm Tomus VI p394 reads continuously from
`agerentur pensitantis` through `… qui summae rerum consi-` to its continuation
with no injected gap and no mid-word page split.

## Phase 4 — gap before `III. Ceterum` (Suggested implementer: Opus)

### 4.1 Trim the chapter-break whitespace

Implement the mechanism Phase 0.2 chose:

- **If oversized blank rows (case i):** write `romulus_chapter_break_gap_normal`
  (composer-level). Match tight scaled source-pixel regions for the preceding
  Latin line and `III. Ceterum` exactly once and in order. Require the heading
  either at the top of the next canvas or separated from the preceding line by
  no more than twice the median ordinary inter-line gap in the same unit; verify
  the left gutter remains blank. Confirm it fails, then add a
  `SOURCE_ROW_EXCISIONS` entry on `(0, 34)` deleting the Phase 0.2 post-shear
  `[top,bottom)` blank interval. Update the `(0, 34, (828, 3040))` guarded
  `band_source_column` case to the measured new crop dimensions and an
  unchanged-pixel fingerprint, preserving the `(60, 798)` horizontal window.
  Update the p34 shear fixture's `expected_bands` and every older mask-fixture
  band expectation affected by the shifted rows. Keep raw-source coordinates
  for `preserved_latin_regions` and map them through shear plus excision when
  asserting the retained image. Add a `row_excision` `repairs` record in
  `tomi_gap_split_repairs_source_trace.json` with source/page, raw `raster`,
  `sheared_raster`, normalized post-shear `top`/`bottom` fractions, exact deleted
  rows, resulting crop dimensions, before/after gap rows, and preserved Latin
  regions. Extend `tomi_gap_split_ledger_has_expected_shape` for these fields
  and add `inner_tomi_row_excisions_match_reviewed_ledger` to compare the exact
  new record with `SOURCE_ROW_EXCISIONS` in both directions.
- **If a composer page-fill artifact (case ii):** add a suffixed Phase 4.1a
  specifying the exact composer interaction observed and the smallest page-scoped
  fix, with its own failing composer-level regression. Do not change global
  composer policy.

Do not reduce the gap below an ordinary inter-line gap or excise any Latin row.
`romulus_chapter_break_deskewed` must still pass with updated post-excision
coordinates (heading and preceding line survive once, no gutter fragments).

**LLM verification:** the applicable new regression, `romulus_chapter_break_deskewed`,
`inner_tomi_known_masks_preserve_latin`, the new table/ledger check, the
ledger-shape test, `cargo fmt`,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Human verification:** in rebuilt Tomus I, confirm `III. Ceterum receptissimae`
follows the previous line with an ordinary gap (or sits at a page top) and no
left-gutter fragments.

## Phase 5 — corpus rebuild, verification, delivery (Suggested implementer: Sonnet)

### 5.1 Rebuild and verify

Run `cargo fmt`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test`. Run each new and
pre-existing ignored repair regression separately (A/B/C/D regressions plus the
pixel-survival checks `inner_tomi_shears_preserve_latin`,
`inner_tomi_whole_strips_preserve_latin`, `inner_tomi_known_masks_preserve_latin`).
Set the Poppler env vars (`PLUTARCH_PDFTOPPM`/`PDFINFO`/`PDFTOTEXT`/`PDFIMAGES`)
and `PLUTARCH_LUALATEX` to the MiKTeX x64 `lualatex.exe`, then run the offline
rebuild inside a `Monitor` (long run):

```text
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf \
  --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
```

Record before/after inner-PDF page counts for Tomi I, III, and VI and explain any
change (a D mask, if used, may reflow p394/p395; a page-count change is acceptable
only when all affected Latin survives once and the spacing is resolved). Save a dated
`llm_artifacts/<ts>_SKEW_TOMUS_VI_REGRESSION_REPAIRS_VERIFICATION.md` with
per-defect before/after evidence, commands/results, and the page-count table. Keep
both this plan and the note as historical records. Commit each defect's fix in its
own commit (regression-first), then open / update the branch PR once all four are
verified.

**LLM verification:** confirm every command exits 0; confirm the four composer-level
regressions and all pixel-survival checks pass; compare the fixture against every
installed table entry.

**Human verification:** compare the four rebuilt reader passages and their adjacent
page breaks with the source scans; confirm each Latin line present once, in order,
with the gaps resolved. This human sign-off is the faithfulness gate for the phase.

## Risks & open questions

- **Which stage loses the victory line (A).** First reconcile the current exact
  `[330,366]` assertion with the prior note that reports it dropped. Trace Pass A,
  Pass B, strip selection, composition, and final PDF placement. Use
  `SOURCE_BODY_BANDS` only if a measured raw/retained band is dropped.
- **Whole-strip ↔ gap trade (C).** If the short strip wraps whole, measure the
  preceding canvas's unused height and apply the two-median-strip bound. If it
  exceeds the bound, diagnose the source strip before accepting the wrap.
- **No removable bridge on p398 (D).** If Phase 0.2 cannot prove a mask removes
  only non-Latin pixels and separates the merged lines, do not install it. A
  narrower whole-strip is possible only after a measured page-scoped change
  creates a matching shorter cut-point interval; specify and test that change.
- **Composer-helper fidelity (0.1).** The fast helper must use `compose_pages`
  with production scale, geom, budgets, strip ordering, and canvas retention.
  Cross-check its canvas hashes against the slow Tier-B path on one already
  verified unit before trusting its canvas-level assertions.
- Do **not** add a global deskew, change global thresholds, the body scale, the
  Latin-column detector, the composer's packing policy, or the transcription cache.
  Every fix stays page-scoped and ledger-recorded.
