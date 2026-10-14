# Implementation status (updated 2026-10-13, second session)

**Ten of the twelve defects are implemented**, verified by ignored production
regressions, and committed on branch `fix/tomus-1-gaps-split-shortending`. The two
remaining (G3, T3a) are deferred to a follow-up plan — see below.

- **Done (first session):** G1 (p23 mask), G2 (p30 mask), S1 (p41 mask), T2b
  (p315 right-edge mask) — `backlog_tomi_divider_masks_restore_banding` + the
  faithfulness gate `inner_tomi_known_masks_preserve_latin`; G4 (p36 excision),
  G5 (p37 excision) — `backlog_romulus_chapter_gaps_excised`; M1 (short-ending
  merge) — `backlog_solon_poplicola_short_ending_merges`.
- **Done (second session):**
  - **T3b (p451)**, **T3c (p462)**: page-foot merged strips bisected mid-glyph.
    Fixed by registering the exact production cut-point strip in `WHOLE_STRIPS`
    (`is_known_whole_source_strip`) so each wraps whole: T3b `(751,2953)` band
    `[2344,2433)` fp `71401035…`; T3c `(801,2988)` band `[412,502)` fp
    `f31a3736…`. Measured scale-independently by the new diagnostic
    `backlog_tomus3_split_strip_trace`; regression
    `backlog_tomus3_foot_splits_wrap_whole`.
  - **T2a (p289→p290 boundary)**: the "O ma-/ter" gap is **not** a p290 problem —
    it is a right scan-edge streak at abs x `[1904,1911)` on **p289** that extends
    the last Latin band ("… exclamans : O ma-", rows `[2559,2586]`) down to row
    2728, so the "O ma-" strip carried ~140 px of trailing whitespace. Fixed by a
    right-edge `SOURCE_FURNITURE_MASKS` rectangle (`1904/1913 → 1.0`, full height),
    recorded in `fixtures/tomi_gap_split_repairs_source_trace.json`; regression
    `backlog_coriolanus_boundary_gap_masked`.
- **Deferred to a follow-up plan — G3 (p34), T3a (p448):** both are **true skew**,
  not edge streaks. Measurement (this session):
  - **T3a p448**: even cropped tight to the real Latin column `[975,1720]` there are
    only ~6 blank gap-rows for ~40 lines — tilted baselines fill the inter-line gaps
    across the full width. A furniture mask cannot de-bridge it; a whole-strip guard
    cannot apply (the merged blocks are `[300,1514]` h=1214 and `[1514,2668]` h=1154,
    scaled ~1460/1388, over the 1394-px page budget).
  - **G3 p34**: the bridge is a vertical rule that **overlays the text body**
    (abs x `[965,974]`, inside the glyphs), not a clean gutter streak. Masking it only
    in the chapter-break whitespace `[1862,2120)` does split the band and remove the
    fragments — but the 252-px break gap then trips `drop_gap_outliers`
    (median gap = 3 because the rest of the column stays merged), which prunes the
    "III. Ceterum" block entirely; collapsing the gap to survive that heuristic needs
    an unnaturally small (~6-px) excision. (Masking also shifts the detected left
    crop 945→970, requiring a `preserve_known_body_crop_edge` pin.) Not a clean fix.
  - **Recommended follow-up mechanism — a scoped *lossless shear* (not a deskew).**
    The baselines are straight lines tilted ~1.2°, so a per-column integer vertical
    shift `round(x·slope)` straightens them into rectangles **without interpolation**
    — every glyph pixel is translated to an integer location, so byte-identity holds
    (unlike rotation). After shearing, the existing rectangular banding/crop/compose
    path works unchanged, and the gap-outlier/furniture heuristics see normal gaps.
    This is page-scoped like the masks/excisions and is simpler and safer than a
    rotation deskew; it also covers the Tomus-VI skew page (v2 p266). It needs its own
    regression proving all glyphs survive once and remain legible. Freeform
    polygon-cut-and-paste was considered but rejected: keeping the slant leaves lines
    visibly tilted and needs new slanted-strip packing; the shear is "polygon extract
    + straighten" done as an integer remap that drops back into the rectangular code.

The three **Tomus VI** items the owner added (pdf 86-87 "quidem" split; v2 p266
skew losing right-edge glyphs + left Greek bleed; gap on pdf 394 after "…summae
rerum consi-") remain scheduled for a **separate** plan after this branch lands;
G3/T3a should join that plan since the scoped-shear mechanism serves all three.

# Context

`BACKLOG.md`'s "Tomus I" section lists seven visible defects in
`build_facsimile/tomus_1_inner.pdf`. They fall into three mechanical classes the
facsimile path already has vocabulary for: (a) excessive vertical **gaps** where
a chapter/section break or a scan artifact inflates the space between two real
Latin lines; (b) one **horizontally split** glyph row cut across a page break so
both halves are illegible; and (c) one near-empty final body page holding a
single complete line. The reader reproduces the Latin column as source-image
pixels, so every repair must keep every distinct Latin glyph present exactly once
and in order — OCR only locates a defect, it never supplies replacement text.

This plan makes finite, edition-specific corrections through the existing
facsimile path (`src/facsimile/reflow.rs`). It does not change transcription
caches, global layout thresholds, the unit table, the audit system, or the source
PDFs. It reuses the four mechanisms the prior Tomus-I repairs established:
`SOURCE_FURNITURE_MASKS` (white out a measured furniture/bleed rectangle before
column detection), `SOURCE_ROW_EXCISIONS` (delete a reviewed bleed-only or
blank-only row slab and close the space it occupied — see
`2026_10_13_08_15_THESEUS_P21_BLEEDTHROUGH_EXCISION_PLAN.md`),
`is_known_whole_source_strip` (recognize an exact strip by dimensions/band/
fingerprint so it wraps whole instead of being split), and
`merge_reviewed_short_ending` / `REVIEWED_SHORT_ENDINGS` (stack a reviewed lone
final line onto its predecessor canvas).

The baseline is the current `build_facsimile/tomus_1_inner.pdf` (416 physical
pages). Page references below are one-based **physical pages of that inner PDF**
("pdf page") paired with the number printed at the foot of the generated page
("folio page"); the two differ by a constant 14 pages of front matter
(pdf = folio + 14). These are **not** the source edition's own printed page
numbers. Source pages are one-based pages of `Ploutarchou_Bioi_v1.pdf` (source 0).

# Current state of the world

This plan covers **twelve** reviewed defects across the first three inner tomi
(seven added first for Tomus I, then two for Tomus II and three for Tomus III).

- **Folio ↔ source mapping.** The backlog "folio page" is the *generated* page's
  own footer number, not the source's printed number, so anchors are mapped to
  source pages by their Latin text. Using the cached 300-DPI renders in
  `build_facsimile/render_cache/` (every source page, `…-pNNNN-300dpi.png`) and
  OCR of the right-hand Latin column, the twelve defects resolve to (all v1 /
  `source == 0`):

  | ID | tomus · pdf/folio | Anchor (Latin) | Src page | Unit | Class |
  | --- | --- | --- | --- | --- | --- |
  | G1 | I · 33/19 | after `… excepisse Ariadnam,` | 23 | theseus | gap / left-streak bridge |
  | G2 | I · 48/34 | after `… Theseum in vinculis servavit.` | 30 | theseus | gap / left-streak bridge |
  | G3 | I · 58/44 | top gap before `III. Ceterum receptissimae …` + left fragments | 34 | romulus | **skew** bridge + left divider |
  | G4 | I · 62/48 | after `… Hinc adolescentibus magnum nomen.` (→ `VII.`) | 36 | romulus | authentic chapter gap |
  | G5 | I · 64/50 | after `… remotis arbitris de his ageret.` (→ `VIII.`) | 37 | romulus | authentic chapter gap |
  | S1 | I · 70-71/56-57 | split row `… genethliacus : cui scientiæ ita se spe-` | 41 | romulus | left-streak bridge → split |
  | M1 | I · 296/282 | lone line `jungeret, et omnem belli apparatum tolleret.` | 146 | comparatio_solonis_et_poplicolae | short ending |
  | T2a | II · 202/194 | `… exclamans : O ma-` [gap] `ter, ut mecum egisti!` | 290 | marcius_coriolanus | gap / right streak + speck |
  | T2b | II · 258/250 | `… operam suam nava-` [gap] `vit, ut eum perinde …` | 315→316 | timoleon | gap / right-streak bridge (page foot) |
  | T3a | III · 121/113 | after `… sed propter victoriam` + left Greek fragments | 448 | philopoemen | gap / wide crop + left Greek bleed |
  | T3b | III · 129-130/121-122 | split row `oppidanorum inflaverat, et ad portas exciverat. Ut vero` | 451 | philopoemen | page-foot bridge → split |
  | T3c | III · 151-152/143-144 | split row `… -phis enim clypeos argenteos affigens, suumque scutum ita` | 462 | flamininus | bridge → split |

- **Measured findings (ignored diagnostic `backlog_tomus1_gap_split_trace`,
  production crop + `line_bands`).** The INK_THRESHOLD is 170 and `line_bands`
  splits where a row's ink ≤ `LINE_MIN_INK` (3). Most defects are a thin vertical
  **ink streak** (the Greek/Latin divider rule, a right scan edge, or bled Greek)
  caught at the column-crop edge: it adds >3 ink to nearly every row, so
  `line_bands` merges the whole column into giant bands (e.g. p23 `[804,2028)`
  h=1224; p41 `[1371,2530)` h=1159), which `compose_body_canvases` then splits
  badly (early page breaks, horizontal cuts, rendered gutter fragments). Removing
  the streak with a `SOURCE_FURNITURE_MASKS` rectangle restores normal ~33-px
  banding. Simulated masks that cleanly restore banding without touching glyphs:
  - **G1 p23** (raster 2007×2993, crop [1047,1870]): left streak local 5-18;
    glyphs start local 29 → mask abs x **[1047,1065)**, full height. Verified: after
    masking, 51 bands, max h 39.
  - **G2 p30** (1984×2998, crop [936,1765]): mask abs x **[936,955)** → 54 bands,
    max h 40. (Confirm no left-sticking capital in the strip.)
  - **S1 p41** (1990×2974, crop [1037,1883]): mask abs x **[1037,1072)** → 48
    bands, max h 44.
  - **T2b p315** (1883×2961, crop [1044,1883]): **right** scan streak local 834-835;
    text ends local 765 → mask abs x **[1880,1883)**, full height → 54 bands.
  - **T3a p448** (1988×3007, crop [909,1988], abnormally wide 1079 px): a left
    column of bled Greek glyph fragments (abs x ≈ 909-965) with Latin starting
    ≈ abs 975; the crop also over-reaches on the right. Needs a left Greek mask and
    possibly a right-edge mask; a single left mask ≤ 120 px did **not** fully
    de-bridge — requires the Phase-1 trace to pin both edges.
  - **G3 p34** (1982×3008, crop [945,1773]): the page is **skewed ≈ 1.2°** (left-half
    inter-line gaps sit ≈ 8 px above right-half gaps), so no row is blank across the
    width and edge-masking cannot fully de-bridge. The *reported* defect is the left
    divider streak rendering as gutter fragments above `III. Ceterum`; a narrow left
    divider mask (abs x ≈ [943,958)) removes the fragments. The residual
    skew-merge must be eyeballed on rebuild; do **not** add a global deskew.
  - **T3b p451** (1881×2953, crop [1075,1826]) and **T3c p462** (1974×2988, crop
    [917,1718]): the bridge is a localized page-foot/mid band (p451 `[2479,2606)`
    h=127; p462 `[516,616)` h=100) that edge-masking ≤ 120 px did not clear →
    treat as a **whole-strip** (`is_known_whole_source_strip`) so the merged band
    wraps whole instead of splitting mid-glyph, if its scaled height fits a budget;
    otherwise pin the exact streak rectangle from the trace.
  - **T2a p290** (1974×2986, crop [933,1719]): banding is otherwise normal; the
    `O ma-/ter` gap comes from a right scan artifact plus a **speck** between the
    lines (a tiny false band). Pin the right streak and the speck row from the
    trace; mask the streak and, if the speck sits in clean whitespace, excise its
    row slab.
  - **G4 p36** (1975×2978, crop [979,1774]) and **G5 p37** (2004×2990, crop
    [1060,1854]): **not** bridged — banding is clean and the gaps are authentic
    chapter-break whitespace (p36 kept-gaps of 163 and 193 px around the `VII.`
    break; p37 a 198-px gap between `[2266,2298)` and `[2496,2530)` at the `VIII.`
    break), versus a ~12-13-px median inter-line gap. Repair by `SOURCE_ROW_EXCISIONS`
    deleting the clean blank slab down to one median gap (both cut edges in clean
    whitespace, no glyph row deleted).
  - **M1**: `comparatio_solonis_et_poplicolae` lone final line → `REVIEWED_SHORT_ENDINGS`
    (measure the 1004×44 final-canvas fingerprint from `compose_pages`).

- **Mask ↔ ledger coupling.** `inner_tomi_masks_match_reviewed_ledger` requires
  every installed `SOURCE_FURNITURE_MASKS` rectangle on a ledger page to match a
  reviewed record, and `inner_tomi_known_masks_preserve_latin` checks each masked
  page's `preserved_latin_regions`. New masks therefore need matching records in
  `src/facsimile/fixtures/backlog_artifact_source_trace.json`
  (`disposition:"mask"`, `status:"repaired"`, `raster`, `mask_rectangles`
  (normalized), `preserved_latin_regions`, `source`, `source_page`,
  `preservation_boundary`). Excisions feed `deleted_rows` through `audit_traced_unit`.

- **Layout confirmed by eye on the renders.** The Didot source sets Greek in the
  left column and Latin in the right column, separated by a thin full-height
  vertical rule. Source page 23 (G1) embeds three centred verse quotations in the
  Latin column, each surrounded by extra vertical whitespace. Source pages 30
  (G2), 34 (G3), 36 (G4), 37 (G5) each carry a genuine chapter break — a sentence
  ending in a period followed by extra whitespace and the next chapter's Roman
  numeral. On generated page 58 (G3) a vertical stack of spurious tiny marks
  (`; ) . - ç ç 'ι η …`) runs down the left gutter above/beside `III. Ceterum`,
  i.e. the column divider/bleed caught at the Latin column's left crop edge.
  Generated pages 70→71 (S1) show the first body row of source page 41
  (`maticus, verum idem genethliacus : cui scientiæ ita se spe-`) cut
  horizontally: its lower part degraded at the foot of page 70, its upper part an
  illegible broken band at the head of page 71. Generated page 296 (M1) is the
  final page of *Solonis et Poplicolae Comparatio* holding only
  `jungeret, et omnem belli apparatum tolleret.` under the running head.

- **The reflow pipeline** (`src/facsimile/reflow.rs`): `render_and_band` renders
  a source page at 300 DPI, applies `mask_known_source_furniture` then
  `excise_known_source_rows` **before** `production_source_column` →
  `crop_column` → banding. `line_bands` (`src/facsimile/lines.rs`, `LINE_MIN_INK`)
  splits the cropped column into maximal ink runs; `cut_points` puts a cut at each
  interior gap midpoint, so the lines on either side of a wide gap become tall
  strips each carrying half of that gap. `compose_body_canvases` packs scaled
  strips top-to-bottom: a strip that exceeds the remaining budget and is **not**
  class `Tall` closes the current canvas early and restarts on a fresh canvas
  (reflow.rs ≈ 2972-2978) — this is the "page ends early" signature of G1/G2. A
  `Tall` strip is cut at its lowest-ink row to fill the page (≈ 2955-2967) — the
  horizontal-split mechanism, which `is_known_whole_source_strip` suppresses for a
  reviewed strip that fits a fresh budget.

- **The four reuse mechanisms** live in `reflow.rs`: `SOURCE_FURNITURE_MASKS`
  (≈ 36), `mask_known_source_furniture` (≈ 623); `SOURCE_ROW_EXCISIONS` (≈ 645),
  `apply_source_row_excision` / `excise_known_source_rows` (≈ 720);
  `is_known_whole_source_strip` + the `WHOLE_STRIPS` fingerprint table (≈ 2656);
  `REVIEWED_SHORT_ENDINGS` + `merge_reviewed_short_ending` (≈ 2838), which
  requires `geom.tb_w == 1004`, zero padding, ≥ 3 canvases, a final canvas of the
  recorded dimensions matching a raw-grayscale SHA-256 fingerprint, and a height
  sum within `geom.content_height()`.

- **Test harness.** `src/facsimile/audit_repair_tests.rs` (included in
  `reflow.rs`'s test module) provides `audit_traced_unit(id)`,
  `production_body_scale()`, `retained_page(&traces, source, page)`,
  `assert_reviewed_source_lines_survive(…)`, `assert_source_region_pixels_survive`,
  `audit_save_unit_trace`, `real_source_image`, and the ignored diagnostic
  conventions (`#[ignore = "requires source PDFs and production-DPI renders"]`).
  Band/trace/pixel-domain checks run in ~1 min; anything calling
  `production_body_scale()` or recomposing a full unit runs >10 min — keep the
  diagnostics in the band/trace/pixel domain where possible.

# Desired end state

1. Each of the seven defects is `repaired` through one of the four existing
   mechanisms, or — only if a trace proves a gap is authentic source spacing that
   the owner wants kept — `source_spacing_retained` with recorded rationale. No
   defect is left `unresolved` without an explicit note. Every §/chapter of the
   affected units keeps all its Latin, present once, in order.
2. G3's spurious left-gutter fragments are gone and `III. Ceterum receptissimae`
   begins at the top of its generated page with only an ordinary leading gap.
   G1/G2/G4/G5 show their two real lines separated by one ordinary inter-line gap
   (or the authentic break, if retained) with no under-full "early" page break
   attributable to an inflated gap. S1's split row renders whole on one page.
   M1's final line sits at the foot of the previous page and the lone final body
   page is gone.
3. Changes stay within `SOURCE_FURNITURE_MASKS`, `SOURCE_ROW_EXCISIONS`,
   `WHOLE_STRIPS` (via `is_known_whole_source_strip`), and `REVIEWED_SHORT_ENDINGS`
   plus their regression tests. No global threshold, body scale, CLI, dependency,
   or audit-algorithm change. No transcription cache edited.
4. `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and the
   full ordinary `cargo test` pass; the new ignored source regressions pass; the
   existing mask-ledger / whole-strip / short-ending / shared-page tests still
   pass (updating only genuinely shifted numeric expectations, never a
   glyph-preservation assertion). All sixteen inner+imposed PDFs rebuild. A dated
   verification note records the pinned coordinates/fingerprints, before/after
   locators, and test/build results.

# Tech Learnings

- **Classifying a gap before repairing it.** A wide inter-line gap can be (i) a
  scan artifact — a divider rule or right/left scan edge whose ink bridges or
  annexes rows, or bled ink forming faint bands; or (ii) authentic paragraph/
  chapter whitespace in the source. The trace distinguishes them: an artifact
  shows stray bands/ink (`ink/row` well above `LINE_MIN_INK`) or an annexed
  furniture band inside the gap; authentic whitespace shows clean rows
  (`ink/row ≈ 0`) between the two real line bands. Artifact → `SOURCE_FURNITURE_MASKS`
  (white the rule/bleed so banding re-splits correctly). Authentic-but-inflated
  blank slab → `SOURCE_ROW_EXCISIONS` (delete clean blank rows down to one median
  inter-line gap; both cut edges in clean whitespace; never delete a glyph row).
- **Whole-strip fingerprints.** `is_known_whole_source_strip` matches
  `(column.width, column.height)`, the `Band { top, bottom }`, a crop box, and a
  raw-grayscale SHA-256 over that box. A matching strip wraps whole only when its
  scaled height fits a fresh budget. Measure all four from the production trace of
  the exact column that is being split; do not shrink the band to the surviving
  fragment.
- **Short-ending fingerprints.** `REVIEWED_SHORT_ENDINGS` records
  `(unit_id, (w, h), sha256_of_raw)` of the *unmerged* final canvas from
  `compose_pages` at `production_body_scale()`. The lone final line must be the
  canvas's only content and `(w, h)` is typically `(1004, 44)`.

# Phase 1 — Measure and classify every defect (Suggested implementer: Opus)

## 1.1 Trace the five gaps and the split, classify each, and pin coordinates

In `src/facsimile/audit_repair_tests.rs`, add one ignored diagnostic test
`backlog_tomus1_gap_split_trace` (`#[ignore = "requires source PDFs and
production-DPI renders"]`). It reuses `audit_traced_unit("theseus")`,
`audit_traced_unit("romulus")`, and `audit_traced_unit("comparatio_solonis_et_poplicolae")`
— it does not re-implement rendering/banding/composition. For each
`(source=0, page)` in {23, 30, 34, 36, 37, 41} it locates the retained page via
`retained_page(&traces, 0, page)` and dumps, around the anchor's band: the raw
`line_bands` with per-row ink (`ink_profile_rows` over `INK_THRESHOLD`), the kept
bands, the sorted interior gaps and their median, the unit `LineMetrics`, and the
crop edges. It saves a PNG crop of each anchor region to
`tmp/pdfs/backlog_tomus1/<id>.png` for human inspection.

From the dumps, record in this plan's table (replacing the "likely mechanism"
below) for each of G1–G5, S1 the **measured** finding:

- For each gap: the two real line bands (original-source rows), the gap between
  them, and whether the intervening rows are **clean blank** (authentic → excision
  candidate) or carry **stray ink / an annexed band** (artifact → mask). Pin the
  exact rectangle: for a mask, `[left, top, right, bottom]` in normalized
  full-page raster coordinates (fractions `pixel/raster_dim`) covering only the
  rule/bleed and no glyph; for an excision, `[top, bottom)` in normalized
  full-page raster rows, both edges in clean whitespace, residual gap = one median
  inter-line gap. For G3 specifically, measure the left-gutter fragment column's
  x-extent so the mask excludes the leftmost glyph stems of the Latin body.
- For S1: the retained column `(width, height)`, the split `Band { top, bottom }`
  for source page 41's first body row, the crop box `[0, top, width, bottom]`, and
  its raw-grayscale SHA-256 (via `matches_source_region_fingerprint`-style
  hashing). Confirm its scaled height fits a fresh canvas budget.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`;
`cargo test backlog_tomus1_gap_split_trace -- --ignored --nocapture`;
`cargo build`. `git diff --check`; read the new test directly.

**Human verification:** Open the saved crops; confirm each gap's classification
(artifact vs authentic) and that each proposed mask/excision rectangle covers only
furniture/bleed/blank, never a Latin glyph. Confirm the S1 band covers the whole
`genethliacus … spe-` row with full ascenders/descenders.

## 1.2 Fingerprint the M1 short ending

Add an ignored test `backlog_solon_poplicola_short_ending_trace` that composes
`comparatio_solonis_et_poplicolae` at `production_body_scale()` via `compose_pages`
(the unmerged canvases), asserts the unit has ≥ 3 canvases, that the final canvas
is `(1004, 44)` and contains the complete `jungeret … tolleret.` line, and prints
its raw-grayscale SHA-256 and the predecessor's height so the stacked sum is
confirmed ≤ `geom.content_height()`. Record the fingerprint for Phase 2.

**LLM verification:** `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`;
`cargo test backlog_solon_poplicola_short_ending_trace -- --ignored --nocapture`.

**Human verification:** Confirm the hashed final canvas is exactly the lone
`jungeret … tolleret.` line.

# Phase 2 — Install the repairs (Suggested implementer: Opus)

Install each measured correction from 1.1/1.2 into the matching existing table in
`reflow.rs`, one source page at a time, running its regression before proceeding:

- **G3** (and any other gap 1.1 classifies as an artifact): add the pinned
  rectangle(s) to `SOURCE_FURNITURE_MASKS` with a comment naming the unit and
  defect. Follow the shared-ledger discipline in
  `inner_tomi_masks_match_reviewed_ledger` / `inner_tomi_known_masks_preserve_latin`:
  record `preserved_latin_regions` and `expected_crop_edges`/`expected_pass_a_bands`
  for the touched page, and confirm no shared-page neighbour loses Latin.
- **G1/G2/G4/G5** (any gap 1.1 classifies as authentic-but-inflated clean blank):
  add the pinned `[top, bottom)` to `SOURCE_ROW_EXCISIONS` with a comment naming
  the unit and chapter break, leaving one median inter-line gap. If 1.1 instead
  shows an artifact, use a mask as for G3. If 1.1 shows the gap is authentic and
  modest (no inflation), mark it `source_spacing_retained` and do not alter it.
- **S1**: add the measured `((width, height), Band, crop, fingerprint)` entry to
  the `WHOLE_STRIPS` table read by `is_known_whole_source_strip`.
- **M1**: add `("comparatio_solonis_et_poplicolae", (1004, 44), "<sha256>")` to
  `REVIEWED_SHORT_ENDINGS`.

**LLM verification per item:** `cargo fmt --all`; `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`; the item's new ignored regression
(Phase 3); the touched mask-ledger/whole-strip/short-ending tests; `cargo build`.

**Human verification:** After each item, re-extract its generated page(s) from a
rebuilt (or `--unit`-scoped) sample and confirm the defect is gone and the Latin
is intact.

# Phase 3 — Regression tests (Suggested implementer: Opus)

In `audit_repair_tests.rs`, add ignored source regressions, each proving the
defect on the unmodified baseline and passing after the fix:

- `backlog_tomus1_romulus_divider_fragments_masked` (G3 and any mask-classified
  gap): the stray fragments are absent, `III. Ceterum …` / the chapter numeral
  begins the retained bands, and the surrounding Latin rows survive once
  (byte-identical via `assert_source_region_pixels_survive`).
- `backlog_tomus1_chapter_gaps_collapse` (any excision-classified G1/G2/G4/G5):
  `trace.deleted_rows` equals the pinned interval(s); no retained band lies in the
  excised slab; the two real lines survive once byte-identical; residual gap
  `> 0` and `≤ 20` px.
- `backlog_romulus_genethliacus_line_composes_whole` (S1): using
  `assert_reviewed_source_lines_survive(..., whole_line: true)` on the split row's
  source box, the complete row survives once on one canvas (this calls
  `production_body_scale()`; budget ~>10 min).
- `backlog_solon_poplicola_short_ending_merges` (M1): `merge_reviewed_short_ending`
  returns `true`, one-canvas reduction, both stacked regions byte-identical,
  preceding canvases unchanged, within the text block; add the reject-identity and
  identity-scoped coverage mirroring the existing short-ending tests.

Also run the existing shared-page / mask / whole-strip / short-ending regressions
named in the prior Tomus-I plans and update only genuinely shifted numeric
expectations.

**LLM verification:** `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`; `cargo test` (full ordinary suite);
each new ignored test `-- --ignored`; `cargo build`.

**Human verification:** Read each trace's output; confirm the surviving lines are
the real ones and the removed/masked regions held only furniture/bleed/blank.

# Phase 4 — Rebuild and record (Suggested implementer: Sonnet)

Preserve before-crops of the seven locations, then run the full offline build
(README Windows command with the LuaLaTeX guard), confirm all sixteen PDFs are
freshly written with successful lualatex logs, and re-extract the seven locations
for after-crops. Because G1/G2/G4/G5 excisions and the M1 merge each shorten a
unit's body slightly, record (do not force) the affected units' body-canvas
counts, the inner-PDF total, trailing signature-blank count, and any
`split_into_volumes` membership change. Write a dated verification note
`llm_artifacts/YYYY_MM_DD_HH_MM_TOMUS_I_GAP_SPLIT_SHORTENDING_VERIFICATION.md`
with the pinned coordinates/fingerprints, before/after locators and crops,
baseline-failure vs post-fix-pass evidence for each new regression, and the full
test/build results. Keep this plan and the note as historical records.

**LLM verification:** `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`; `cargo test`; all new ignored
regressions; `cargo build --release --bin facsimile`; the full offline build;
`git diff --check`; read the note directly.

**Human verification:** Compare before/after crops of all seven locations at full
resolution; confirm each defect is resolved, every affected Latin line is intact
and unduplicated, and no new clipping/fusion was introduced. Skim adjacent pages
of Theseus/Romulus/the comparison to confirm nothing else shifted unexpectedly.
