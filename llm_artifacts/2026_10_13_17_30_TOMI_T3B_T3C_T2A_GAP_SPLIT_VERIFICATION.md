# Tomi I–III gap/split repairs — second-session verification (2026-10-13)

Branch `fix/tomus-1-gaps-split-shortending`. This session implemented **three** of
the five remaining "harder-class" defects — **T3b, T3c, T2a** — and, after
measurement, **deferred G3 and T3a** to a follow-up plan (they are true skew, not
edge streaks). Tally for the twelve reviewed Tomi I–III defects: **10 implemented,
2 deferred.** No global threshold, body scale, CLI, dependency, or audit-algorithm
change; no transcription cache edited. All repairs stay inside the four existing
mechanisms (`SOURCE_FURNITURE_MASKS`, `is_known_whole_source_strip`/`WHOLE_STRIPS`,
`SOURCE_ROW_EXCISIONS`, `REVIEWED_SHORT_ENDINGS`).

## Diagnostics re-run

- `backlog_tomus1_gap_split_trace` (committed) — regenerated the production crop
  edges + `line_bands` for the five source pages. Confirmed the prior measurements.
- `backlog_tomus3_split_strip_trace` (**new**) — dumps the cut-point **strips** the
  composer actually packs (band, class, raw/scaled height, budget-fit, and the raw
  grayscale SHA-256 of each strip's full-width crop region) for the page-foot split
  pages. Uses the frozen `production_body_scale()` literal (1.2023952) so it stays in
  the ~1-min band domain. This supplied the exact `WHOLE_STRIPS` signatures below.

## T3b — philopoemen v1 p451 (generated III · 129→130)

Reported split row: `oppidanorum inflaverat, et ad portas exciverat. Ut vero`.

- Production cropped column `(751, 2953)`, crop edges `[1075,1826]`.
- The reported row and the following line fuse (tight foot spacing) into one **Tall**
  cut-point strip `Band { top: 2344, bottom: 2433 }` (h=89), which the composer was
  bisecting mid-glyph to fill the page.
- Fix: registered in `WHOLE_STRIPS`
  `((751, 2953), Band{2344,2433}, [0,2344,751,2433],
  "71401035f8ab7fd9f305a8c4aa47f02dbe32d72c64f8db212dc6ef96e0b7b1db")`.
- Scaled height 89·1.2023952 ≈ **107 px ≤ 1394-px body budget**, so it wraps whole.

## T3c — flamininus v1 p462 (generated III · 151→152)

Reported split row: `…del-phis enim clypeos argenteos affigens, suumque scutum ita`.

- Production cropped column `(801, 2988)`, crop edges `[917,1718]`.
- The `…phis …` continuation and its `inscripsit :` colophon fuse into one **Tall**
  strip `Band { top: 412, bottom: 502 }` (h=90), bisected mid-glyph across the page
  break.
- Fix: registered in `WHOLE_STRIPS`
  `((801, 2988), Band{412,502}, [0,412,801,502],
  "f31a373625ac795614f07c68557c2668890af5776d7ffa456e981994c61b7068")`.
- Scaled height 90·1.2023952 ≈ **108 px ≤ 1394**, wraps whole.

Regression for T3b+T3c: **`backlog_tomus3_foot_splits_wrap_whole`** (band/pixel
domain, ~3 s after build). For each page it proves the registered cut-point strip is
Tall and now recognized (fails on the baseline — no `WHOLE_STRIPS` row matched), fits
a fresh budget, that sibling strips on the page are **not** recognized (signature is
specific), that a one-pixel tamper is rejected (fingerprint guard), and that every
glyph in the merged strip is byte-identical to the source.

## T2a — marcius_coriolanus v1 p289→p290 boundary (generated II · 202)

Reported: `… exclamans : O ma-` [inflated gap] `ter, ut mecum egisti!`.

- **Root cause pinned (differs from the first-session hypothesis).** The gap is a
  **p289** problem, not p290. The last Latin line `… exclamans : O ma-` sits at raster
  rows `[2559,2586]`; a right scan-edge streak at abs x `[1904,1911)` (full-page
  height, measured on the 300-DPI render; the Latin ends near x=1826, so the streak is
  clear of all glyphs) extends that band down to row **2728**. The "O ma-" strip
  therefore carried ~140 px of trailing whitespace that rendered as the inflated gap
  before "ter, …" (the first line of p290).
- Fix: `SOURCE_FURNITURE_MASKS` right-edge rectangle on p289
  `left: 1904.0/1913.0, top: 0.0, right: 1.0, bottom: 1.0` (raster `(1913,2963)`),
  recorded in `fixtures/tomi_gap_split_repairs_source_trace.json` (`T2a`,
  `mask_rectangles [[0.9952953476215368, 0.0, 1.0, 1.0]]`,
  `preserved_latin_regions [[1085,2504,1860,2544],[1085,2559,1860,2586]]`).
- Post-mask the foot band collapses to `[2559,2586]` (the real "O ma-" row); the
  streak region `[2587,2728]` carries no retained band.
- Regression: **`backlog_coriolanus_boundary_gap_masked`** (band/pixel domain) —
  proves no p289 band reaches the streak region (fails on the baseline, foot band
  `[2559,2728]`), the "O ma-" row is one band, and both "O ma-" (p289) and the
  continuing "ter …" (p290) survive once byte-identically.

## Deferred — G3 (p34) and T3a (p448)

Both are **true skew**, not maskable edge streaks (full measurement and the
recommended follow-up mechanism — a *scoped lossless shear* — are documented in the
plan's "Implementation status" section). Summary:

- **T3a p448**: even the tight Latin crop `[975,1720]` has only ~6 blank gap-rows for
  ~40 lines; merged blocks `[300,1514]`/`[1514,2668]` scale to ~1460/1388 px, over the
  1394-px budget, so neither a mask nor a whole-strip guard applies.
- **G3 p34**: the bridge rule overlays the glyph body (abs x `[965,974]`). Masking it
  only in the chapter-break whitespace removes the fragments and splits the band, but
  the 252-px break gap trips `drop_gap_outliers` (median gap 3) and prunes the
  "III. Ceterum" block; a clean fix needs the shear, not a mask. The G3 mask attempt
  was reverted after this was observed.

## Test / build results

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean.
- `cargo test` (ordinary suite) — **434 passed, 0 failed, 104 ignored**.
- Ignored regressions (release, with Poppler env):
  - `backlog_tomus3_foot_splits_wrap_whole` — PASS.
  - `backlog_coriolanus_boundary_gap_masked` — PASS.
  - `inner_tomi_masks_match_reviewed_ledger` — PASS.
  - `inner_tomi_known_masks_preserve_latin` (all masked pages, incl. new p289) — PASS.
  - `tomi_gap_split_ledger_has_expected_shape` — PASS.
- Full offline rebuild (`cargo run --release --bin facsimile … --offline`,
  `PLUTARCH_LUALATEX` = MiKTeX x64): _see "Rebuild" below (filled on completion)._

## Rebuild

Full offline rebuild completed cleanly (`cargo run --release --bin facsimile --
--v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/
--offline`, Poppler env + `PLUTARCH_LUALATEX` = MiKTeX x64; `REBUILD_EXIT=0`). All
**16 PDFs** produced (8 inner + 8 imposed). Inner page counts:

| Tomus | pages |
|-------|-------|
| 1 | 416 |
| 2 | 448 |
| 3 | 416 |
| 4 | 128 |
| 5 | 384 |
| 6 | 480 |
| 7 | 448 |
| 8 | 192 |

Tomus 1 is unchanged at 416 (its lone defect M1 was a short-ending merge already
landed in the first session; no new Tomus 1 change this session). The three fixes
this session touch Tomi 2 (T2a, p289→p290) and 3 (T3b p451, T3c p462); both page
counts are stable (448 / 416), confirming the repairs reflow the affected seams
without adding or dropping body pages. All eight imposition passes succeeded.
