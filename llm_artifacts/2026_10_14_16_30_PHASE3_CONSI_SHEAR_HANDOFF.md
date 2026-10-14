# Handoff — Phase 3 (defect D, `consi-` gap, v2 p398) in progress

**Date:** 2026-10-14 · **Branch:** `fix/tomus-1-gaps-split-shortending`

Implementing **Phase 3** of
`llm_artifacts/2026_10_13_22_47_SKEW_TOMUS_VI_REGRESSION_REPAIRS_PLAN.md`.

> **Redirect from the plan prose (confirmed by the user and by Phase 0.2):**
> defect D is a **skew**, not a removable bridge. The fix is a `SOURCE_SHEARS`
> entry on `(1, 398)` **plus removal** of the existing `[1022,1837]` whole-strip —
> *not* the `SOURCE_FURNITURE_MASKS` bridge mask the plan's Phase 3 text assumes.
> Everything below follows the shear path. Phases 0, 1, 2 are already done,
> committed, and pushed (see the `tomi-gap-split-repairs` memory).

## Where things stand right now

### Committed + pushed
- `49b5d63` **Add failing composer-level consi-gap regression** —
  `tomus6_consi_page_reads_continuously` in `src/facsimile/audit_repair_tests.rs`
  (placed immediately before `tomus6_consi_continuation_gap_closed`). It is
  `#[ignore]`, composer-level (via `compose_unit_at_frozen_scale("cleomenes")`).
  **Confirmed FAILS on the pre-fix tree** (the merged `[1022,1837]` whole strip is
  still one strip, so `agerentur`/`consi`/`lioque` are not resolved).

### Uncommitted working tree (the FIX — correct, not yet committed)
These three changes are the Phase 3 fix. They have NOT been committed yet and the
fix has NOT been re-verified green since re-applying them.

1. `src/facsimile/reflow.rs`
   - **Added** `SOURCE_SHEARS` entry for `(source: 1, page: 398)`:
     `raw_dims: (1840, 2835)`, `fingerprint_region: [1050, 600, 1650, 750]`,
     `fingerprint: "680397c87b37fb70c0a520f82755d86b0a186a93b1d201a303c7bbc3281cd660"`,
     `x0_px: 1350`, `baseline_slope: 0.03175895765472313`.
   - **Removed** the first `WHOLE_STRIPS` entry
     `((818, 2835), Band{1022,1837}, [0,1022,818,1837], "1d7abf77…")`.
2. `src/facsimile/fixtures/tomi_gap_split_repairs_source_trace.json`
   - **Removed** the cleomenes `whole_strip` repairs record.
   - **Added** a cleomenes `shear` repairs record (compact one-record style,
     modeled on the caesar p266 shear record). Diff is minimal (11 ins / 8 del);
     JSON validated.

> ⚠️ **Formatting gotcha (do not repeat my mistake):** this fixture packs each
> `repairs` record onto a few hand-formatted lines (several fields per line), NOT
> one-field-per-line. Do **not** rewrite it with `json.dump(..., indent=2)` — that
> reformats all ~60 records (900-line diff). Edit it surgically with the `Edit`
> tool in the existing compact style. I already hit this and fixed it.

## Remaining steps (do these in order, then one commit + push)

### Step A — update `tomus6_consi_continuation_gap_closed` (STILL TODO)
`src/facsimile/audit_repair_tests.rs` ~line 4455+ (search the fn name). Its current
body describes the **whole-strip** mechanism and will now fail:
- asserts `column.dimensions() == (818, 2835)` → post-shear dims are **(818, 2894)**.
- asserts `is_known_whole_source_strip(&column, &band)` for `Band{1022,1837}` → now
  **false** (entry removed).
- uses `crop: [880, 1698]` in a `RetainedSourcePageTrace` — crop edges are still
  `[880,1698]`, but `map_row`/pixel coords are now in **sheared** space.

Replace its body with a post-shear band-domain assertion (the plan's Phase 3 step 4:
"replace its body with the continuous-read assertion" / keep only if still true).
Recommended: assert the three reviewed post-shear bands separate individually, i.e.
feed the sheared page through `render_and_band(1, 398, …)`, assert
`column.dimensions() == (818, 2894)`, assert each of `expected_bands`
`[1013,1042] [1362,1392] [1405,1436]` appears in `cut_points(&kept)` (consecutive
pairs), assert every kept band height `< 100` (no merged block survives), and verify
the reviewed Latin pixels survive via `assert_sheared_source_region_pixels_survive`
with `x0=1350, slope=0.03175895765472313, top_pad=16, crop_left=880` for the three
`preserved_latin_regions`. Do NOT keep any `is_known_whole_source_strip` assertion.
(There is no `assert_reviewed_sheared_page` option here — that helper requires a
gutter furniture mask whose ink it whitens; p398 is shear-only, no mask.)

### Step B — verify green
Run (all from repo root; shear/compose tests need the source PDFs + render cache):
```
cargo fmt && cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test                                   # full non-ignored suite
cargo test --release tomus6_consi_page_reads_continuously -- --ignored   # now PASSES
cargo test --release tomus6_consi_continuation_gap_closed -- --ignored   # updated body passes
cargo test inner_tomi_shears_match_reviewed_ledger inner_tomi_whole_strips_match_reviewed_ledger tomi_gap_split_ledger_has_expected_shape
cargo test --release inner_tomi_shears_preserve_latin inner_tomi_known_masks_preserve_latin -- --ignored
```
Ledger cross-checks that MUST stay consistent after the record swap:
- `inner_tomi_shears_match_reviewed_ledger` iterates `SOURCE_SHEARS` **and** shear
  records (both directions) → the new `(1,398)` entry needs its shear record (done).
- `inner_tomi_whole_strips_match_reviewed_ledger` iterates `whole_strip` records →
  removing the cleomenes record keeps it consistent (done).
- `inner_tomi_whole_strips_preserve_latin` iterates `whole_strip` records → cleomenes
  no longer in it (expected).

### Step C — commit + push the fix (one commit)
Stage `src/facsimile/reflow.rs` + the fixture + the updated
`tomus6_consi_continuation_gap_closed`. Message e.g. "Deskew Cleomenes p398 and drop
its whole-strip to close the consi gap". End with the
`Co-Authored-By: Claude Opus 4.8` trailer. Push.

Then update the `tomi-gap-split-repairs` memory to mark Phase 3 done and point to
Phase 4 (romulus `III. Ceterum` row-excision) + Phase 5 (rebuild + verification + PR).

## Key measured facts (authoritative — do not re-measure)

- **Source render:** `build_facsimile/render_cache/Ploutarchou_Bioi_v2-p0398-300dpi.png`,
  RGB `(1840, 2835)`. `real_source_image` converts via `to_luma8()`; verified that
  **Rec.709 coefficients + round** (`0.2126 R + 0.7152 G + 0.0722 B`, then
  `round`, clip 0–255) reproduces Rust's `to_luma8` byte-for-byte on these scans
  (matched 3 known fingerprints). So Python PIL→`convert('RGB')`→that formula is
  safe for fingerprinting.
- **Shear fingerprint** of raw region `[1050,600,1650,750]` =
  `680397c87b37fb70c0a520f82755d86b0a186a93b1d201a303c7bbc3281cd660`.
- **Slope choice:** diagnosis recorded `skew_slope 0.02414`, but that was an
  underestimate. Matched-gap and cross-correlation measurement give the true skew
  ≈ **0.031–0.033** (adjacent page p266 is 0.0375). A slope sweep showed **every**
  value in `[0.0241, 0.0375]` separates the merged block into ~20 clean bands
  (max band height ≤ 36 px), so correctness is insensitive to the exact value.
  Chosen `0.03175895765472313` = `(1306−1286.5)/(1640−1026)` from the clean matched
  gap recorded in `matched_gaps` (left_x 1026 rows [1273,1300], right_x 1640 rows
  [1293,1319]) — internally consistent, reproducible, mid-range.
- **Post-shear geometry** (confirmed by running the trace at this slope): crop
  `[880,1698]`, cropped dims **(818, 2894)**, `top_pad 16`, `shift_range [-16, 43]`,
  bottom_pad 43.
- **Three reviewed lines (RAW word boxes → post-shear band → strip, canvas 40):**
  - `agerentur pensitantis` — raw `[910,987,1198,1017]` → band `[1013,1042]` →
    strip `(1005,1049)` (Narrow)
  - `qui summæ rerum consi-` — raw `[1390,1354,1676,1380]` → band `[1362,1392]` →
    strip `(1355,1398)` (Normal)
  - `lioque praeerat` (continuation) — raw `[900,1376,1102,1411]` → band
    `[1405,1436]` → strip `(1398,1442)` (Normal)
  All three are **distinct consecutive Normal/Narrow strips on canvas 40**, in
  source order; the whole p398 block flows continuously canvas 40→41 with no
  injected gap (pre-fix the `[1022,1837]` whole strip dumped the block onto canvas
  41 leaving a 741 px gap on canvas 40). `cleomenes` retained index for p398 = 19;
  `audit_traced_unit`/`compose_unit_at_frozen_scale` already apply the shear
  (reflow.rs:3027).

## Diagnostics / how to re-measure if needed
- Trace test: `cargo test --release skew_and_tomus6_repair_trace -- --ignored
  --nocapture` → prints `BREAK cleomenes v2 p398 …` (crop, dims, raw/kept bands,
  strips) and `PLACE consi_block cleomenes …` (per-strip canvas placement). It picks
  up the installed shear automatically.
- OCR of the Latin column used `C:/Program Files/Tesseract-OCR/tesseract.exe` on a
  crop of the p398 render (see `inner-tomi-audit-env` memory). Set
  `PYTHONIOENCODING=utf-8`.
