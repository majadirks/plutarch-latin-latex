# Skew and Tomus VI repairs: Phase 1 handoff

Resume at **Phase 2.1** of
`llm_artifacts/2026_10_13_19_00_SKEW_AND_TOMUS_VI_REPAIRS_PLAN.md`.
The plan's **Diagnosis results** section and the `diagnoses` array in
`src/facsimile/fixtures/tomi_gap_split_repairs_source_trace.json` hold the
reviewed, measured values. Copy slopes, `x0_px`, fingerprints, fingerprint
regions, raw dimensions, matched gaps, and crop edges from there — do **not**
re-measure them. The Latin transcription cache must stay untouched.

## Completed work

- **Phase 0** (diagnosis): commits `8573fed` (0.1 locator trace) and `e5f104b`
  (0.2 measurement/classification). The five machine-readable diagnoses live in
  the fixture's top-level `diagnoses` array; it is evidence only.
- **Phase 1.1** (commit `4c223ee`): the guarded lossless shear in
  `src/facsimile/reflow.rs`.
  - `struct SourceShear { source, page, raw_dims, fingerprint_region,
    fingerprint, x0_px, baseline_slope }` and `const SOURCE_SHEARS` (currently
    **empty** — Phase 2 fills it).
  - `source_shear_shift(x, x0_px, slope)` =
    `(-((x - x0_px) * slope)).round() as i64` (identical arithmetic to the Phase
    0.2 trace), `source_shear_shifts` (returns the shift vector and `top_pad =
    -min_shift`), `shear_image` (integer per-column vertical shift into a
    white-padded destination `raw_height + max_shift - min_shift` tall; every raw
    pixel `(x,y)` lands once at `(x, y + shift(x) + top_pad)`), and
    `apply_known_source_shear`, which is the **first** transform in
    `render_and_band` and returns the input untouched unless source, page,
    `raw_dims`, and the raw-pixel fingerprint all match.
  - Synthetic tests `source_shear_levels_both_slope_directions`,
    `source_shear_preserves_every_column_with_padding`,
    `source_shear_rejects_changed_source_identity`. In
    `src/facsimile/audit_repair_tests.rs`: the x-aware helper
    `assert_sheared_source_region_pixels_survive(original, retained, x0_px,
    slope, top_pad, crop_left, deleted_rows, region)` (maps source `(x,y)` →
    sheared row `y + shift(x) + top_pad` → `map_row` over sheared-coordinate
    excisions → `x - crop_left`) and the synthetic
    `source_shear_pixels_survive_after_pipeline`.
- **Phase 1.2** (commit `bff041e`): the shared ledger validated by disposition.
  - The fixture `description` was rewritten in place to cover all five
    dispositions and Tomus VI.
  - `tomi_gap_split_ledger_has_expected_shape` dispatches on `disposition`
    (`mask` / `shear` / `whole_strip` / `word_patch` /
    `source_spacing_retained`); every record is checked for source identity, raw
    dimensions (`raster`), and non-empty `preserved_latin_regions`.
  - `WholeStripSignature` and `WHOLE_STRIPS` were hoisted to module scope
    (entries and matching helper unchanged) so the ledger test can read them.
  - `inner_tomi_shears_match_reviewed_ledger` cross-checks `SOURCE_SHEARS`
    against the `repairs` shear records; `inner_tomi_whole_strips_match_reviewed_ledger`
    cross-checks `WHOLE_STRIPS` against the `repairs` whole-strip records.
  - The raw-fraction mask collectors (`reviewed_page_masks`,
    `all_reviewed_page_masks`) skip records with `coordinate_space == "sheared"`.
  - `inner_tomi_known_masks_preserve_latin` now feeds the **raw** page to
    `render_and_band` (so a sheared page is not transformed twice).
  - Ignored scaffolds `inner_tomi_shears_preserve_latin` and
    `inner_tomi_whole_strips_preserve_latin` iterate the installed records
    through the real pipeline (vacuous until Phase 2 adds records).

No `repairs` records were added in Phase 1: `SOURCE_SHEARS` is empty and the
`repairs` array still contains the five installed masks only.

## Decisions to carry forward

| Defect | Source | Reader | Disposition | Key measured values (in `diagnoses`) |
|---|---|---|---|---|
| Romulus heading | v1 p34 | Tomus I p58 | `shear` | `1982×3008`; slope `-0.016079632465543646`, `x0=1400`, shifts `[-23,9]`, `top_pad=23`; fp region `[1100,600,1700,750]`; crop `[945,1773)` merges `[907,2633]` |
| Philopoemen victory | v1 p448 | Tomus III p121 | `shear` | `1988×3007`; slope `-0.03048780487804878`, `x0=1400`, shifts `[-43,18]`, `top_pad=43`; fp region `[1100,600,1700,750]`; crop `[909,1988)` merges `[300,1501]`+`[1527,2663]` |
| `quidem` straddle | v2 p231 | Tomus VI pp86–87 | **no repair** | Complete `quidem passus Callisthenem` on p86; a 112-px scaled `Tall` strip exists but the break follows the complete line |
| Caesar right glyphs | v2 p266 | Tomus VI p158 | `shear`, then recheck crop | `1878×2825`; slope `0.03745928338762215`, `x0=1350`, shifts `[-20,51]`, `top_pad=20`; fp region `[1050,600,1650,750]`; baseline crop `[880,1708)` cuts `sive` at x1720 |
| `consi-` continuation | v2 p398 | Tomus VI pp394–395 | `whole_strip` | crop `818×2835`; strip `[1022,1837)` 980 scaled px (<1394 budget); strip fp region `[0,1022,818,1837]`, digest `1d7abf778c…c5bef5df`; source gaps 20/18 px |

The exact SHA-256 fingerprints, matched gap rows, and raw dimensions for each
defect are in the `diagnoses` array and the plan's "Diagnosis results" section.

## Phase 2 order of work

Do the steps in plan order, each as its own commit+push, keeping every suite
green between commits.

### Before the first shear installs — extend `audit_traced_unit`

`audit_traced_unit` in `src/facsimile/audit_repair_tests.rs` currently mirrors
`render_and_band` as **mask → excise → crop** and captures `deleted_rows`
against the raw height. It does **not** apply the shear. Before (or as part of)
Phase 2.1, extend it to **shear → mask → excise → crop**, and compute
`deleted_rows` in post-shear coordinates (`excision_rows` against the sheared
height), so the trace's `crop` and row mapping match the real sheared retained
page. The three reviewed shear pages (v1 p34, v1 p448, v2 p266) carry **no**
`SOURCE_ROW_EXCISIONS`, so `deleted_rows` is empty for them, but the crop must
be detected on the sheared raster. Without this change the shear
survival/trace tests would measure the wrong page.

### 2.1 Romulus v1 p34, 2.2 Philopoemen v1 p448, 2.4 Caesar v2 p266 (`shear`)

For each page:
1. Write the named baseline-failing regression first and confirm it fails on the
   current build: `romulus_chapter_break_deskewed`,
   `philopoemen_victory_line_deskewed`, `tomus6_p266_right_glyphs_restored`
   (all `#[ignore]`, release, require source PDFs).
2. Add the `SOURCE_SHEARS` entry, copying `raw_dims`, `fingerprint_region`,
   `fingerprint`, `x0_px`, and `baseline_slope` verbatim from the matching
   `diagnoses` record.
3. Add the matching `repairs` **shear** record in the same commit. Its shape
   (enforced by `tomi_gap_split_ledger_has_expected_shape` and checked field-for
   -field against `SOURCE_SHEARS` by `inner_tomi_shears_match_reviewed_ledger`):
   `status:"repaired"`, `unit_id`, `source`, `source_page`, `raster:[w,h]`,
   `disposition:"shear"`, `fingerprint`, `fingerprint_region:[4]`,
   `matched_gaps:{left_x,right_x,left_rows:[2],right_rows:[2]}`, `x0_px`,
   `baseline_slope`, `shift_range:[2]`, `top_pad`, `crop_edges:[2]`,
   `expected_bands:[…]`, and non-empty `preserved_latin_regions`. `x0_px`,
   `baseline_slope`, `fingerprint`, `fingerprint_region`, and `raster` must equal
   the `SOURCE_SHEARS` entry exactly.
4. **Remeasure** the crop edges and `band_source_column` output on the sheared
   raster and record them as `crop_edges` / `expected_bands`; the pre-shear crop
   in `diagnoses` will change once the baselines are level.
5. For v2 p266 only: after the shear, re-check all five right-edge glyphs
   (`sive`, `praeter`, `(3)`, `auferro,`, `idem`). If any glyph is still clipped
   by the crop, implement the word-patch mechanism (New mechanism B) under its
   contract and add `source_word_patch_replaces_overlap_once` +
   `source_word_patch_rejects_changed_source_identity`. Do not add the patch if
   the shear restores every glyph.
6. Add a sheared-coordinate left-gutter / Greek-bleed mask only if fragments
   remain outside reviewed Latin pixels; mark such a mask record
   `coordinate_space:"sheared"` and measure its fractions against the sheared
   dimensions.
7. Run the regression (now passing), `inner_tomi_shears_match_reviewed_ledger`,
   `inner_tomi_shears_preserve_latin`, `tomi_gap_split_ledger_has_expected_shape`,
   then `cargo fmt`, `cargo fmt --all --check`,
   `cargo clippy --all-targets -- -D warnings`, and `cargo test`. Commit and push.

### 2.3 `quidem` v2 p231 (no repair unless reproduced)

Phase 0 found no mid-line split in the current reader: `quidem passus
Callisthenem` is complete on p86. Write `tomus6_quidem_wraps_whole` (or a
completeness check) that asserts the source line is complete and unique across
pp86–87, and confirm it **passes** on the current baseline. Install **no**
guard. Only if a later baseline reproduces a genuine mid-line split with a
`Tall` fill-split do you add a `WHOLE_STRIPS` signature + whole-strip `repairs`
record. Do not infer a word patch from the page location.

### 2.5 `consi-` v2 p398 (`whole_strip`)

Write the baseline-failing regression (`tomus6_consi_continuation_gap_closed`
if closing the fill-split; the source gap is genuine so the `source_spacing`
path does not apply here — the strip simply must wrap whole). Add a
`WHOLE_STRIPS` signature matching the diagnosed `column_dims (818,2835)`, band
`[1022,1837)`, fingerprint region `[0,1022,818,1837]`, and digest
`1d7abf778c…c5bef5df`, plus a matching `repairs` **whole_strip** record
(`column_dims:[2]`, `composer_strip:[2]`, `strip_fingerprint_region:[4]`,
`strip_fingerprint`, non-empty `preserved_latin_regions`). Assert the scaled
strip height (980 px) fits the 1394-px page budget. Run
`inner_tomi_whole_strips_match_reviewed_ledger`,
`inner_tomi_whole_strips_preserve_latin`, the ledger shape test, fmt, clippy,
and the suite. Commit and push.

### Phase 3 (Sonnet)

Full offline rebuild + verification note + branch PR. Use the frozen production
body scale **1.2023952** for budget checks in band/trace/pixel tests; set the
Poppler env vars and `PLUTARCH_LUALATEX` to the MiKTeX x64 binary; run the
`cargo run --release --bin facsimile … --offline` rebuild; record before/after
Tomi I/III/VI inner page counts; save
`llm_artifacts/<ts>_SKEW_AND_TOMUS_VI_REPAIRS_VERIFICATION.md`.

## Verification and workspace state

Phase 1 passed `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test`: **440 library
tests passed, 108 ignored; 12 binary tests passed** (the four new synthetic
shear tests plus the two new ledger match tests; the two new ignored survival
scaffolds raise the ignored count). The four Phase 1.1 synthetic tests and the
ledger tests run in the ordinary (non-ignored) suite.

The working tree carries pre-existing, unrelated modifications to `BACKLOG.md`
and `prompts.txt` plus many untracked `tmp/` files. They were left out of every
Phase 0 and Phase 1 commit; preserve them and stage only files belonging to
your step.

Slow-path reminders (see the project memory): `production_body_scale()` samples
the whole corpus (>15 min) and is frozen at 1.2023952; keep the inner loop in
band/trace/pixel tests. Run hour-plus commands inside a Monitor, not a
turn-yielding background job.
