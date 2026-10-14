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
   intervening line. The installed shear is the cause. **Restoring this line is
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
- **The regression blind spot.** Every skew/Tomus VI regression
  (`romulus_chapter_break_deskewed`, `philopoemen_victory_line_deskewed`,
  `tomus6_quidem_line_complete`, `tomus6_consi_continuation_gap_closed`,
  `tomus6_p266_right_glyphs_restored`, in `audit_repair_tests.rs`) asserts only in
  the **band / crop / pixel domain** (the output of `render_and_band` and
  `assert_reviewed_sheared_page` at `audit_repair_tests.rs:3873`). **None of them
  runs `compose_body_canvases`.** That is why all five passed while the composed
  reader is wrong: a line dropped by `drop_furniture`, a `Tall` fill-split that
  orphans a word, and a whole-strip bump that leaves a page-bottom gap are all
  invisible to band-domain assertions. The two existing composer-level tests
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

## Root-cause diagnosis

### A — Philopoemen victory line dropped (v1 p448) — URGENT

The p448 shear (`SOURCE_SHEARS`, slope `+0.03048780487804878`, `x0=1400`,
`top_pad=18`) straightens the baselines so `line_bands` now resolves the formerly
merged column into individual lines. The verification note for the prior plan
records that the first post-shear band `[330,366]` is "dropped by
`drop_furniture`", leaving `[380,415]` as the first retained body band — and the
stale-fixture fix changed `inner_tomi_known_masks_preserve_latin`'s
`expected_first_band` from `[300,1501]` to `[380,415]` to match. That dropped band
`[330,366]` is the victory line `subito illis … victoriam`: it is the first body
line on the page, it sits above a larger-than-median gap after the shear, so
`drop_gap_outliers` prunes it as a detached leading band.

`philopoemen_victory_line_deskewed` did not catch this because it asserts only
that **some** retained band overlaps the reviewed region `[909,315,929,1050]` — a
tall strip spanning the whole left column, which many bands overlap even with the
victory-line band gone. It never pins the victory line specifically and never
composes.

**Fix direction:** keep the shear (it correctly straightens the page) and add a
reviewed `SOURCE_BODY_BANDS` entry for `(0, 448)` that rescues the victory-line
band, exactly as that mechanism is designed to do. Then strengthen the regression
to pin the specific band and to assert the line survives **composition**.

### B — gap before `III. Ceterum receptissimae` (v1 p34)

The p34 shear + sheared-coordinate gutter mask removed the left-gutter ink
fragments (the symptom the prior plan scoped) but not the gap the reader sees. The
gap is the chapter-break whitespace: in the source a large blank band (the prior
work measured a ≈252 px chapter gap) precedes the `III.` heading. After banding it
survives as an oversized inter-line gap and the composer reproduces it at scale,
reading as a gap before the heading. The shear does not change the size of a blank
band.

**Fix direction:** confirm in Phase 0 whether the gap is (i) an oversized blank
band between the preceding body line and the heading band, or (ii) a composer
page-fill artifact. If (i), add a page-scoped `SOURCE_ROW_EXCISIONS` entry on p34
that deletes the excess blank rows down to an ordinary inter-line gap (the same
mechanism already used for blank slabs), measured in **post-shear** coordinates.
Do not shrink the gap below a normal inter-line gap and do not touch any Latin row.

### C — `quidem` straddle (v2 p231)

Phase 0 of the prior plan found in the band domain that `quidem passus
Callisthenem` bands as a complete line (strip `[2224,2317]`, ≈112 scaled px) and
concluded `no_repair_required`. But that line is the **last** line of its page in
the composer, it is classified `Tall`, and when it does not fit the remaining
budget `compose_body_canvases` **fill-splits** it at a low-ink row — orphaning
`quidem` at the bottom of p86 while the rest flows to p87. The band-domain
completeness check (`tomus6_quidem_line_complete`) is true and irrelevant to the
composer's cut.

**Fix direction:** add the `[2224,2317]` strip to `WHOLE_STRIPS` so it is wrapped
whole instead of fill-split (its ≈112 scaled px is far below the page budget, so it
fits a fresh page cleanly and the resulting p86 bottom loses only one line — not a
visible gap). Guard it with the strip fingerprint like the other `WHOLE_STRIPS`
entries, and add a composer-level regression that asserts `quidem` is not cut.

### D — `consi-` new gap (v2 p398)

The installed `WHOLE_STRIPS` entry wraps the strip `(818,2835) [1022,1837]`. That
band is **815 raw px tall** — at ≈33–40 raw px per line that is roughly **20
merged lines**, not a single continuation line. The region merged into one `Tall`
band because a bridge (divider / scan-edge / bled-Greek streak) fills the inter-line
gaps across part of the width (the prior plan itself noted blank gaps of only
18–20 px at specific x-ranges, i.e. gaps that exist at some x but not across the
full width — the classic bridge-merge signature from this branch's other gap
defects). Wrapping that ≈20-line block whole forces the entire block to the next
canvas; the previous page (p394) closes after `agerentur pensitantis`, leaving its
bottom blank — the new gap.

**Fix direction:** replace the whole-strip with a `SOURCE_FURNITURE_MASKS` entry
that removes the bridge, so the block bands into ≈20 individual lines that the
composer packs naturally across p394/p395. This is the same mask mechanism that
fixed the other gap defects on this branch (G1/G2/S1), and it removes both the
original split and the injected gap. The whole-strip entry for `(818,2835)` is then
**removed** from `WHOLE_STRIPS`. If Phase 0 finds no removable bridge and the block
is genuinely one or two short lines (it is not, per the height), fall back to a
narrower whole-strip; otherwise the mask is the fix.

> **Cross-cutting note (C vs. D).** Wrapping a strip whole trades a mid-unit split
> for a page-bottom gap. That trade is acceptable for C (one orphaned line, ≈1-line
> gap) but not for D (≈20-line block, large gap). This is why C stays a whole-strip
> and D becomes a mask. Record the reasoning in each ledger entry.

## Phase 0 — confirm root causes and build the composer-level harness (Suggested implementer: Opus)

### 0.1 Add a fast composer-level assertion helper

Add a shared test helper in `src/facsimile/audit_repair_tests.rs` that composes a
single unit at the frozen scale and lets a test assert at the reader-page level
without `production_body_scale()`. It must:

1. take a unit id, render + band it with `audit_traced_unit(<unit>)` (≈50 s, no
   whole-corpus scale),
2. build `LineMetrics`/`TrimGeom` for that unit the same way the existing
   composer tests do,
3. call `compose_body_canvases(&pages, 1.2023952, &geom, geom.first_body_budget(),
   geom.body_budget(), &metrics)`,
4. return the packed canvases plus the per-canvas band/strip provenance
   (`ranges_for_composed_canvases` already exists and is used at
   `audit_repair_tests.rs:6121`).

Name the helper so its intent is clear (e.g. `compose_unit_at_frozen_scale`).
Do not call `assert_reviewed_source_lines_survive` (that is the slow Tier-B path).

**LLM verification:** add a throwaway `#[ignore]` smoke test that composes one
small unit through the helper and prints canvas count; run it with
`cargo test --release <smoke_name> -- --ignored` and confirm it completes in ≈1 min.
Run `cargo fmt`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Human verification:** none (infrastructure only).

### 0.2 Confirm each defect's mechanism and pin coordinates

Extend the ignored diagnostics in `audit_repair_tests.rs` (reuse
`skew_and_tomus6_repair_trace`). For each defect record the evidence in the shared
fixture `src/facsimile/fixtures/inner_tomi_repair_source_trace.json` `diagnoses`
array (do not add to `repairs` until the matching table entry is installed):

- **A (v1 p448):** print the full post-shear `raw` band list and the `kept` list
  from `band_source_column`; confirm the victory-line band is present in `raw` and
  absent from `kept`, and whether it is dropped by the top-zone filter or by
  `drop_gap_outliers`. Record the exact `observed`/`protected` band in
  `(1079,3068)` coordinates and the OCR'd text of that band to prove it is
  `subito illis … victoriam`.
- **B (v1 p34):** print the post-shear bands around the `III.` heading, the gap
  (in rows) between the preceding body band and the heading band, and the typical
  inter-line gap on the page. Decide blank-band excision vs. composer artifact and
  record the row range to excise if (i).
- **C (v2 p231):** compose the Alexander unit with the 0.1 helper; confirm the
  `[2224,2317]` line is fill-split across two canvases and that `quidem` is the
  orphaned piece. Record the strip fingerprint region + hash for a `WHOLE_STRIPS`
  entry.
- **D (v2 p398):** print the raw band list for the `[1022,1837]` region to show it
  is many lines merged into one `Tall` band; locate the bridge (its x-range and ink
  profile) so a furniture mask can target it; confirm after a trial mask the region
  bands into individual lines. Record the bridge rectangle as source-fraction
  coordinates.

**LLM verification:** `cargo test --release skew_and_tomus6_repair_trace -- --ignored`;
inspect the printed bands, OCR, and saved PNGs under `tmp/pdfs/skew/`. Run
`cargo fmt`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Human verification:** read the OCR'd victory-line band and the four recorded
mechanism decisions; confirm each against the source scan.

## Phase 1 — URGENT: restore the Philopoemen victory line (Suggested implementer: Opus)

### 1.1 Rescue the dropped band and prove it survives composition

Write the failing regression first, then fix:

1. `philopoemen_victory_line_present` (composer-level, using the 0.1 helper):
   assert the victory line appears on exactly one reader canvas, in source order
   between its neighbours, and that no canvas omits it. Confirm it **fails** on the
   current tree.
2. Add a `SOURCE_BODY_BANDS` entry for `(0, 448, (1079,3068))` with
   `observed`/`protected` set to the victory-line band measured in Phase 0.2 and
   `unit_id` set to the Philopoemen unit. This re-adds the band after
   `drop_furniture`.
3. Strengthen `philopoemen_victory_line_deskewed` so its band assertion pins the
   **specific** victory-line band as retained (not merely overlapped).
4. Record the repair in the fixture `repairs` with disposition
   `source_body_band` (or the existing disposition this mechanism already uses in
   the ledger) and cross-check it in the ledger-shape test.

**LLM verification:** `cargo test --release philopoemen_victory_line_present -- --ignored`
(now passes), `philopoemen_victory_line_deskewed`,
`inner_tomi_known_masks_preserve_latin`, the ledger-shape test, `cargo fmt`,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Human verification:** in the rebuilt Tomus III, confirm
`… superbe Gythium oppugnabat / subito illis nihil tale exspectantibus, sed propter
victoriam / secure agentibus …` reads in order with the line present once.

## Phase 2 — `quidem` straddle (Suggested implementer: Opus)

### 2.1 Wrap the `quidem` line whole

1. Write `tomus6_quidem_not_split` (composer-level): assert the `quidem passus
   Callisthenem` strip lands on a single canvas and `quidem` is not the sole
   content of a canvas boundary. Confirm it **fails** on the current tree.
2. Add the `[2224,2317]` strip to `WHOLE_STRIPS` with the fingerprint from
   Phase 0.2; assert its scaled height fits the page budget (it does, ≈112 px).
3. Add the matching `repairs` record (disposition `whole_strip`) and cross-check
   with `inner_tomi_whole_strips_match_reviewed_ledger`.
4. Keep `tomus6_quidem_line_complete` (band-domain completeness) as-is.

**LLM verification:** `cargo test --release tomus6_quidem_not_split -- --ignored`
(passes), `tomus6_quidem_line_complete`, the whole-strip ledger test, `cargo fmt`,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Human verification:** confirm `quidem passus Callisthenem` is one complete line
on Tomus VI p86 (or p87), with `quidem` not orphaned.

## Phase 3 — `consi-` gap: mask the bridge, drop the whole-strip (Suggested implementer: Opus)

### 3.1 Replace the whole-strip with a furniture mask

1. Write `tomus6_consi_page_reads_continuously` (composer-level): assert reader
   p394 contains `agerentur pensitantis` immediately followed by its successor with
   no page-bottom gap, and that `… consi-` and its continuation are present in
   order. Confirm it **fails** on the current tree.
2. Add a `SOURCE_FURNITURE_MASKS` entry for `(1, 398)` targeting the bridge
   rectangle measured in Phase 0.2 (source fractions, matching the
   `SourceFurnitureMask` format bit-for-bit). Verify the region now bands into
   individual lines.
3. **Remove** the `(818,2835) [1022,1837]` entry from `WHOLE_STRIPS` and its
   `repairs` whole-strip record; replace the `repairs` record with a `mask` record
   and its `mask_rectangles`.
4. Keep `tomus6_consi_continuation_gap_closed` only if it still describes a true
   post-mask expectation; otherwise replace its body with the continuous-read
   assertion. Do not leave a whole-strip assertion that no longer matches the
   installed mechanism.

**LLM verification:** `cargo test --release tomus6_consi_page_reads_continuously -- --ignored`
(passes), `inner_tomi_masks_match_reviewed_ledger`,
`inner_tomi_known_masks_preserve_latin`, the ledger-shape test, `cargo fmt`,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Human verification:** confirm Tomus VI p394 reads continuously from
`agerentur pensitantis` through `… qui summae rerum consi-` to its continuation
with no injected gap and no mid-word page split.

## Phase 4 — gap before `III. Ceterum` (Suggested implementer: Opus)

### 4.1 Trim the chapter-break whitespace

Implement the mechanism Phase 0.2 chose:

- **If an oversized blank band (case i):** write `romulus_chapter_break_gap_normal`
  (composer-level): assert the gap before the `III. Ceterum` heading is comparable
  to an ordinary inter-line gap and the heading survives once with no gutter marks.
  Confirm it fails, then add a `SOURCE_ROW_EXCISIONS` entry on `(0, 34)` deleting
  the excess blank rows (post-shear coordinates) down to a normal gap. Add the
  matching `repairs` record (disposition `row_excision`).
- **If a composer page-fill artifact (case ii):** add a suffixed Phase 4.1a
  specifying the exact composer interaction observed and the smallest page-scoped
  fix, with its own failing composer-level regression. Do not change global
  composer policy.

Do not reduce the gap below an ordinary inter-line gap and do not excise any Latin
row; `romulus_chapter_break_deskewed` must still pass (heading + preceding line
survive once, no gutter fragments).

**LLM verification:** the applicable new regression, `romulus_chapter_break_deskewed`,
`inner_tomi_known_masks_preserve_latin`, the ledger-shape test, `cargo fmt`,
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
change (the D mask will reflow p394/p395; a one-page count change there is
expected and acceptable if the text reads continuously). Save a dated
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

- **Which filter drops the victory line (A).** Phase 0.2 must confirm whether
  `drop_furniture`'s zone filter or `drop_gap_outliers` removes `[330,366]`. The
  `SOURCE_BODY_BANDS` rescue fixes either, but the regression must assert the
  specific band so the cause is pinned.
- **Whole-strip ↔ gap trade (C).** Wrapping the `quidem` line whole shortens p86 by
  one line. Phase 0.2's compose trace must confirm this is a ≈1-line bottom, not a
  visible gap; if it is visible, treat C like D (diagnose a bridge-merge / mask)
  rather than accepting the gap.
- **No removable bridge on p398 (D).** If Phase 0.2 finds the merge is not a
  maskable bridge, the mask fix does not apply; fall back to a narrower whole-strip
  covering only the `consi-`/continuation pair and document why the larger block
  cannot be separated.
- **Composer-helper fidelity (0.1).** The fast helper must compose exactly as
  production does (same scale, geom, budgets, strip ordering). Cross-check its
  canvas hashes against the slow Tier-B path on one already-verified unit before
  trusting its reader-level assertions.
- Do **not** add a global deskew, change global thresholds, the body scale, the
  Latin-column detector, the composer's packing policy, or the transcription cache.
  Every fix stays page-scoped and ledger-recorded.
