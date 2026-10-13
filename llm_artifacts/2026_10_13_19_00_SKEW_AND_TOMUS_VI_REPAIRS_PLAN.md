# Skewed-scan & Tomus VI facsimile repairs — implementation plan (2026-10-13)

## Context

This plan covers **five** reviewed facsimile defects that the gap/split/short-ending
branch (`fix/tomus-1-gaps-split-shortending`) deliberately left for a follow-up:
**two skewed pages in Tomi I and III** and **three defects in Tomus VI**. They are
grouped together because the dominant root cause — a scan photographed at a slight
angle — is shared, and one new mechanism (a *scoped lossless shear*) addresses most
of them.

The reader must reproduce the Latin from the printed source faithfully, once and in
order. Source transcription is outside this work and must not be normalized or edited.
Each defect below identifies what a reader sees wrong in the generated facsimile.

> **Edition index.** `source 0` = `Ploutarchou_Bioi_v1.pdf` (volume 1 of the two
> source scans); `source 1` = `Ploutarchou_Bioi_v2.pdf` (volume 2). "Source page N"
> is the 1-based page of that scan PDF. The Greek text is the **left** column of each
> source page and the Latin is the **right** column; a thin divider rule sits between
> them. The generated readers are the eight `tomus_*_inner.pdf` files.

## Current state of the world

`src/facsimile/reflow.rs::render_and_band` renders, masks, excises rows, detects and
crops the Latin column, then bands it. `src/facsimile/lines.rs::line_bands` uses a
horizontal ink projection. `drop_furniture` can discard a detached first or last
band; `compose_body_canvases` can fill-split a `Tall` strip. The reviewed
`WHOLE_STRIPS` signatures suppress that split for specific strips. Existing masks,
row excisions, and short-ending rules are page-scoped.

The fixture `src/facsimile/fixtures/tomi_gap_split_repairs_source_trace.json` and
its shape and pixel tests presently handle masks only. In particular,
`tomi_gap_split_ledger_has_expected_shape` requires disposition `mask` and
`mask_rectangles`. `assert_source_region_pixels_survive` maps through a rectangular
crop and row excisions but has no x-dependent row mapping for shear.

## Desired end state

Each of the five reader symptoms is repaired, except that the `consi-` gap may be
retained if source-image evidence proves it is authentic. The Latin on every
affected page is present once, in source order, with reviewed source pixel values
preserved in the unscaled reader stream. Every page-scoped repair is source-guarded
and ledger-recorded. Ordinary tests, ignored repair regressions, formatting,
clippy, and the offline facsimile rebuild pass locally; the rebuilt PDFs are
visually compared with the source scans.

## Tech Learnings

An integer vertical shift of each image column moves pixels without interpolating
their values. Preserving *all* source pixels requires a destination tall enough
for the maximum upward and downward shifts. Exact dimensions alone do not identify
a scan: a distinctive raw-pixel fingerprint guards a reviewed transformation.

## The five defects

| # | Short name | Where (source) | Where (reader) | What's wrong |
|---|------------|----------------|----------------|--------------|
| 1 | **Romulus chapter-break skew** | v1 (source 0) page **34** | Tomus I inner, the page beginning `III. Ceterum receptissimae …` | A vertical stack of spurious tiny marks (`; ) . - ç ç 'ι η …`) runs down the left gutter above/beside the heading, and `III. Ceterum` does not start cleanly at the top of its page. |
| 2 | **Philopoemen victory-line skew** | v1 (source 0) page **448** | Tomus III inner, after the line `… sed propter victoriam` | An inflated gap after `… sed propter victoriam`, with stray Greek glyph fragments bleeding into the left margin of the Latin. |
| 3 | **"quidem" page-straddle split** | v2 (source 1); source page **TBD in Phase 0** (reader side is inner pdf 86–87, folio 78–79) | Tomus VI inner, the page break between pdf 86 and 87 | The final line is split across the page break so that its first word, **"quidem"**, is stranded at the foot of one page while the rest of the line continues on the next. |
| 4 | **v2 p266 angled scan** | v2 (source 1) page **266** (generated inner pdf page 158) | Tomus VI inner, pdf page 158 | The scan is tilted, so the right-hand crop clips the last glyph of several lines — the **"e" in "sive"**, the terminal **"r" in "praeter"**, the **closing paren in "(3)"**, the **"o" in "auferro,"**, and the **"m" in "idem"** — and Greek bleeds into the left margin. |
| 5 | **"consi-" continuation gap** | v2 (source 1); source page **TBD in Phase 0** (reader side is inner pdf 394, folio 386) | Tomus VI inner, pdf page 394 | An inflated gap after the hyphenated line `non praebebat, Sosibus autem, qui summae rerum consi-`, before its continuation. |

Defects 1, 2, and 4 are **true skew** (the whole scan is rotated ~1–1.5°). Defects 3
and 5 look like the bridge/split/short-ending family already repaired on this branch,
but their source pages have not yet been pinned — Phase 0 below classifies them before
any mechanism is chosen.

## Diagnosis results

Phase 0 fills this section. It is the plan-side home for the one-line mechanism
decision and its evidence that Phase 0.2 records for each of the five defects (the
machine-readable copy lives in the fixture's `diagnoses` array). Each entry records
the pinned source page, raw dimensions, raw fingerprint, measured baseline slope and
`x0_px` (for skew defects), the chosen disposition (`shear`, `mask`, `whole_strip`,
`word_patch`, or `source_spacing_retained`), and the evidence behind it. Phase 2 reads
the slope, `x0_px`, fingerprint, and dimensions recorded here when it installs each
`SOURCE_SHEARS` (or other table) entry, so these measurements are taken once and not
re-derived downstream.

- **Defect 1 — Romulus chapter-break skew (v1 p34):** _pending Phase 0._
- **Defect 2 — Philopoemen victory-line skew (v1 p448):** _pending Phase 0._
- **Defect 3 — "quidem" page-straddle split (v2, page TBD):** _pending Phase 0._
- **Defect 4 — angled v2 p266:** _pending Phase 0._
- **Defect 5 — "consi-" continuation gap (v2, page TBD):** _pending Phase 0._

---

## Why the existing four mechanisms are not enough

The branch already carries four page-scoped repair mechanisms in
`src/facsimile/reflow.rs`:

- `SOURCE_FURNITURE_MASKS` — white-out a furniture rectangle (divider/scan-edge streak).
- `SOURCE_ROW_EXCISIONS` — delete a blank row slab.
- `is_known_whole_source_strip` / `WHOLE_STRIPS` — wrap a reviewed cut-point strip whole
  instead of letting the composer bisect it to fill a page.
- `REVIEWED_SHORT_ENDINGS` — stack a lone final line onto its predecessor.

All four assume **horizontal baselines**: `line_bands` finds blank rows *across the
full column width* to separate lines. On a skewed page no row is blank across the
width (the tilt fills every inter-line gap somewhere), so the whole column merges into
one giant band that the composer then slices mid-glyph. Measurements taken while
deferring defects 1 and 2 confirmed this:

- **Philopoemen victory-line skew (v1 p448):** even cropped tight to the real Latin
  column `[975,1720]`, only ~6 rows out of ~40 lines are blank across the width. The
  merged blocks are `[300,1514]` (h≈1214) and `[1514,2668]` (h≈1154); scaled by the
  production body scale they come to ~1460 / ~1388 px, over the 1394-px page budget, so
  a whole-strip guard cannot apply either.
- **Romulus chapter-break skew (v1 p34):** the "bridge" is a vertical divider rule that
  overlays the glyph body (abs x `[965,974]`, inside the letters), not a clean gutter
  streak. Masking it only in the chapter-break whitespace does split the band, but the
  252-px chapter gap then trips `drop_gap_outliers` (median gap ≈ 3 because the rest of
  the column stays merged) and prunes the `III. Ceterum` block entirely. Masking also
  shifts the detected left crop 945→970, which would clip the leading glyph stems.

Skew needs a transform that **straightens the baselines first**. Crop, band, and
compose must then be remeasured; their coordinates can change after the transform.

---

## New mechanism A — scoped lossless shear

**Idea.** The baselines are straight lines tilted by a small angle, so a per-column
**integer vertical shift** straightens them into horizontal rows. Because every
pixel moves by a whole number of rows *within its own
column*, no interpolation happens: each glyph's pixels are translated to integer
locations, never blended or resampled. The glyph pixel **values** are therefore
preserved exactly — only their row coordinates change. This is the key difference from
a rotation/deskew (which resamples and would violate byte-identity).

The shear is applied across the **full page width** (Greek column, divider, and Latin
column together) using the slope measured from the Latin baselines. This is correct
only under the load-bearing premise that the defect is a single **uniform rotation**
of the whole scan, so one slope straightens every baseline on the page. That premise
is what lets Phase 2.2/2.4 measure Greek-bleed masks in post-shear coordinates. A
warped (non-uniform) scan would break it — see Risks.

**Where it slots in.** A new first transform in `render_and_band`
(`src/facsimile/reflow.rs:1073`), *before* masking/excision, so that all downstream
measurements (masks, excisions, crop, bands) are taken on the straightened raster:

```rust
fn render_and_band(
    source: usize,
    page: u32,
    render_fn: &impl Fn(usize, u32) -> Result<GrayImage>,
) -> Result<(GrayImage, Vec<Band>)> {
    let page_img = render_fn(source, page)?;
    let mut page_img = apply_known_source_shear(source, page, page_img); // first
    mask_known_source_furniture(source, page, &mut page_img);
    let page_img = excise_known_source_rows(source, page, page_img);
    let column = production_source_column(source, page, &page_img)?;
    let cropped = crop_column(&page_img, &column);
    let (raw, bands) = band_source_column(source, page, &cropped);
    let bands =
        retain_known_source_body_bands(None, source, page, cropped.dimensions(), &raw, &bands);
    Ok((cropped, bands))
}
```

The only change to the existing body is the inserted `apply_known_source_shear`
line; the `render_fn(source, page)?` binding is already `mut` today, and the
`retain_known_source_body_bands(None, …)` tail is unchanged (Phase 2.1 relies on it).

**The table.** Mirror the existing mask/excision tables:

```rust
struct SourceShear {
    source: usize,
    page: u32,
    raw_dims: (u32, u32),
    fingerprint_region: [u32; 4],
    fingerprint: &'static str, // SHA-256 of distinctive raw source pixels
    x0_px: u32,
    baseline_slope: f64, // measured change in baseline y per x pixel
}
const SOURCE_SHEARS: &[SourceShear] = &[ … ];
```

`apply_known_source_shear` returns the input untouched unless source, page, raw
dimensions, and `matches_source_region_fingerprint` all match. Check the fingerprint
on the **raw image before transformation**, following the existing `WHOLE_STRIPS`
identity pattern. A same-sized but changed render must not trigger the shear.

**Measuring the slope.** From the Phase-0 trace (below): detect two well-separated
inter-line gap rows — one near the left edge of the Latin column, one near the right —
and set `baseline_slope = (y_right − y_left) / (x_right − x_left)`. The applied
shift has the **opposite sign**:
`shift(x) = round(−(x − x0_px) · baseline_slope)`. Use the same `f64` arithmetic and
rounding in production and tests. `x0_px` is a **full-page raw x coordinate** (the
same space as `x` in the shift formula, which is computed over the whole raw width),
not a column-local offset; choose it near the Latin-column center. Because the shift
is relative, the exact `x0_px` only shifts the whole straightened raster up or down
by a constant — that constant is absorbed by `top_pad` — so it does not affect
straightening and its precise value is non-critical. Confirm by re-banding the
sheared page and measuring the actual clean gaps. A
synthetic test with both descending-right and ascending-right baselines must pin
the sign; the two measured gaps must belong to the same inter-line interval.

Compute all signed shifts over the raw width. Include zero in `min_shift` and
`max_shift`, then set `top_pad = −min_shift`. Allocate a white image of width
`raw_width` and height `raw_height + max_shift − min_shift`; copy every raw pixel
`(x,y)` once to `(x, y + shift(x) + top_pad)`. Use checked conversions and
dimensions. The padding prevents top/bottom clipping. The transformed raster's
height and `top_pad` become part of every downstream coordinate mapping. Any new
mask on this page is measured **after shear**, with normalized fractions computed
against the transformed dimensions. Existing masks on other pages retain their
coordinates.

**Faithfulness under shear.** Test the primitive's full per-column pixel multiset,
including synthetic ink at the top and bottom edges. Separately compare every pixel
of each reviewed Latin region with its retained pixel at the mapped coordinate:
source `(x,y)` maps to `(x, y + shift(x) + top_pad)`, then through any row excision
and the measured crop. A rectangular raw region becomes x-dependent after shear;
extend the existing source-region mapping helper or add a shared x-aware variant
instead of copying the full survival test. The multiset test alone cannot prove
that the crop or mask kept Latin. Keep `assert_source_region_pixels_survive` for
unsheared pages.

---

## New mechanism B — reviewed word-crop paste

**Idea.** Use this mechanism only if diagnosis shows that a complete glyph exists
in the raw raster but the detected crop drops it. Lift **real source pixels**
containing the full glyph and restore them before line banding. No typesetting or
synthetic glyphs are allowed. A `quidem` split caused by a `Tall` composer cut
belongs in `WHOLE_STRIPS`, not in this patch path.

**Hard rule: replace, never duplicate.** If part of the glyph already survives the crop,
excise that remnant before pasting the full glyph, so the letter is never printed twice.
The regression must assert the restored word appears **exactly once**.

**The table.**

```rust
struct SourceWordPatch {
    source: usize,
    page: u32,
    raw_dims: (u32, u32),
    src_rect: [u32; 4],     // glyph region in the raw raster (incl. the clipped edge)
    dest_anchor: [u32; 2],  // in the widened cropped image
    excise_rect: Option<[u32; 4]>, // same coordinate space; surviving remnant
    fingerprint: &'static str, // SHA-256 of raw src_rect
}
```

Call a patch helper in `render_and_band` **after** selecting the Latin crop and
**before** `band_source_column`. Preserve the original raw render for the source
rectangle. If the page is also sheared, map its raw source pixels through the
shear before pasting. Widen the returned cropped image with white pixels enough
to contain the entire destination rectangle; an overlay clipped at the original
crop boundary is insufficient. Clear only the measured surviving remnant, then
paste the complete source region once. Define source, destination, and excision
rectangles as half-open and record the crop edges and coordinate transform in the
fixture. Check that the widened strip fits the composed page's safe text width
at the production scale. If it cannot, diagnose a wider crop or another scoped
placement instead of accepting an unmeasured overflow. Test byte-identical source
pixels, the cleared overlap, uniqueness, banding, and rendered placement. Do not
implement the table unless a diagnosed defect needs it.

---

## Phase 0 — source diagnosis (Suggested implementer: Opus)

### 0.1 Locate source pages and baseline symptoms

Use OCR of the raw right-column renders only as a locator for Tomus VI inner pp86–87
and p394. Match the named Latin against the raw source page and adjacent page,
then verify by eye. Take render-cache filenames from the actual cache code or
existing files; do not assume they contain the source PDF basename. V2 p266 is
already located but still needs exact raw dimensions and fingerprint. Record
source page, raw dimensions, raw fingerprint, source-line boxes, reader page,
and baseline symptom for all five defects in a top-level `diagnoses` array in
the shared fixture. Do not add an uninstalled record to `repairs`: its existing
shape test must remain green through diagnosis. Keep Latin readings exactly as
scanned. This trace records all five defects — including the v1 Tomi I/III skews —
so name the ignored locator test `skew_and_tomus6_source_pages_trace`, sharing the
scope of the Phase 0.2 trace rather than implying a Tomus-VI-only reach.

**LLM verification:** run the new ignored locator trace individually with
`cargo test --release skew_and_tomus6_source_pages_trace -- --ignored` and inspect
the saved source and reader images. After changing Rust, run `cargo fmt`,
`cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test`.

**Human verification:** compare each located raw page and named passage with
the current inner PDF; confirm the source page and visible symptom.

### 0.2 Measure skew and classify all five defects

Extend ignored diagnostics in `src/facsimile/audit_repair_tests.rs`, reusing
`backlog_tomus1_gap_split_trace` and `backlog_tomus3_split_strip_trace`. For v1
pp34/448 and v2 p266, report crop edges, `line_bands`, blank-across-width rows,
matched left/right gap coordinates, measured baseline slope, and applied signed
shift range. Save raw and diagnostic PNGs under `tmp/pdfs/skew/`. For `quidem`
and `consi-`, report raw bands, cut points, source spacing, and whether the
composer classifies the relevant strip as `Tall`. Record a one-line mechanism
decision and its evidence for each defect in a diagnosis section of this plan
and in the fixture's `diagnoses` array: shear, mask, whole strip, patch, or
retained source spacing. Name the ignored trace `skew_and_tomus6_repair_trace`.
Do not choose a word patch solely because the symptom occurs at a page break.

**LLM verification:** run
`cargo test --release skew_and_tomus6_repair_trace -- --ignored`; inspect numeric
output, saved PNGs, and all five fixture decisions. Run `cargo fmt`,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test` after changing Rust.

**Human verification:** inspect the measured source gaps and both sides of the
`quidem` and `consi-` passages; correct any unsupported mechanism decision.

---

## Phase 1 — shared mechanisms and ledger (Suggested implementer: Opus)

### 1.1 Implement the guarded lossless shear

Add `SOURCE_SHEARS` and its helper to `src/facsimile/reflow.rs` under the contract
above; call it once at the start of `render_and_band`. Add ordinary synthetic
tests `source_shear_levels_both_slope_directions`,
`source_shear_preserves_every_column_with_padding` (including edge ink), and
`source_shear_rejects_changed_source_identity`. Add an x-aware source-to-retained
mapping helper in `src/facsimile/audit_repair_tests.rs`. Name its test
`source_shear_pixels_survive_after_pipeline`; check source pixels after shear,
mask, excision, and crop.

**LLM verification:** run the four named filtered tests, `cargo fmt`,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test`; inspect both signed synthetic coordinate traces.

**Human verification:** inspect a synthetic before/after image and the documented
coordinate mapping; no source-page judgment is required yet.

### 1.2 Extend the shared repair ledger checks

Keep the existing mask records and checks intact. Rewrite the fixture's top-level
`description` in place so it no longer reads as masks-only for Tomi I–III: it now
covers the skew/whole-strip/word-patch/retained-spacing dispositions and the Tomus
VI defects as well. (Rewrite the text rather than appending an update note — older
wording lives in git history.) Update the fixture schema and test helpers to accept
dispositions `shear`, `whole_strip`, `word_patch` (only if used), and
`source_spacing_retained` (only if justified). Add an entry to `repairs` only in the
same step that installs its matching table entry or reviewed disposition, so every
intermediate step stays green. Update `tomi_gap_split_ledger_has_expected_shape` to
validate fields **by disposition**. Every record has source identity, raw
dimensions, and reviewed Latin regions. Shear records include the raw fingerprint,
matched gap positions, `x0_px`, slope, shift range/padding, expected crop and bands.
Whole-strip records contain the existing column dimensions, exact band, fingerprint
region, and fingerprint. Mask records state whether their coordinates refer to raw
or sheared dimensions; the existing mask records are in raw full-source fractions,
so that is the default when the field is absent. Patch records contain all raw and
destination rectangles and fingerprints. Retained-spacing records contain measured
source/reader gaps and the reason.

Add `inner_tomi_shears_match_reviewed_ledger` and
`inner_tomi_whole_strips_match_reviewed_ledger`; add a patch ledger test only if
the patch mechanism is used. Share identity, region, and pixel-mapping helpers
instead of copying the existing mask loops. Extend
`inner_tomi_known_masks_preserve_latin` to feed the **original raw** page to
`render_and_band` on a page with shear: it currently feeds an already masked
clone, which would otherwise be sheared and masked again. Add separate ignored
pixel-survival tests for new dispositions. Preserve the existing
`is_known_whole_source_strip` table and matching helper; do not create a parallel
composer rule.

**LLM verification:** run `cargo test tomi_gap_split_ledger_has_expected_shape`,
`cargo test inner_tomi_masks_match_reviewed_ledger`, the new filtered ledger
tests, `cargo fmt`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test`.

**Human verification:** compare one fixture record of each installed disposition
with its source image, coordinate system, and actual Rust table entry.

## Phase 2 — individual repairs (Suggested implementer: Opus)

For each repair, first write its named regression and confirm that it fails on
the baseline; then install the smallest diagnosed page-scoped entry and confirm
it passes. Remeasure crop edges and bands after any shear or patch. The one
exception is the `source_spacing_retained` disposition (the retained-spacing
branch of 2.5): it is **not** a repair, its regression may pass on the baseline,
and the "confirm it fails first" rule does not apply to it.

Each shear entry's numeric fields — `baseline_slope`, `x0_px`, `fingerprint`,
`fingerprint_region`, and `raw_dims` — are the values measured in Phase 0.2 and
recorded in the "Diagnosis results" section and the fixture's `diagnoses` array.
Copy them from there rather than re-measuring; `inner_tomi_shears_match_reviewed_ledger`
cross-checks the installed Rust table against the fixture.

### 2.1 Romulus chapter break, v1 p34

Add a reviewed shear entry. Add a narrow **sheared-coordinate** left-gutter mask
only if divider fragments remain outside reviewed Latin pixels. Trace the actual
post-shear sequence through `drop_furniture`. The 252-pixel chapter gap is not
made small by restoring ordinary three-pixel inter-line gaps; if the heading
still drops, use a fingerprint-guarded page-scoped body-band retention entry
following `retain_known_source_body_bands`, with the exact band and Latin pixels
recorded. Adjust `preserve_known_body_crop_edge` only if the measured crop loses
leading stems. `romulus_chapter_break_deskewed` asserts the heading and preceding
line survive once, with no gutter marks.

**LLM verification:** run `cargo test --release romulus_chapter_break_deskewed -- --ignored`,
the shear and ledger tests, `cargo fmt`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test`.

**Human verification:** compare the heading, preceding line, and leading stems
in the source and rebuilt Tomus I pages.

### 2.2 Philopoemen victory line, v1 p448

Add a reviewed shear entry and confirm the giant bands resolve into complete
source lines. Measure the gap after `… sed propter victoriam`. Add a sheared-coordinate
Greek-bleed mask only if it avoids Latin. `philopoemen_victory_line_deskewed`
asserts the victory line and continuation survive once and the reader gap is no
longer inflated relative to adjacent source inter-line gaps.

**LLM verification:** run `cargo test --release philopoemen_victory_line_deskewed -- --ignored`,
the shear and ledger tests, `cargo fmt`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test`.

**Human verification:** inspect the before/after Tomus III page beside v1 p448,
including the left margin and the continuation.

### 2.3 `quidem` straddle, located v2 page

If Phase 0 shows a `Tall` fill-split, add a reviewed signature to the existing
`WHOLE_STRIPS` table and ledger. Assert that the strip's scaled height fits a full
page budget; a guard cannot fix a strip taller than that budget. The regression
`tomus6_quidem_wraps_whole` asserts source order, one occurrence of `quidem`, a
complete line on one reader page, and no mid-glyph cut. If the strip is not
`Tall`, trace the actual split operation. Use a word patch only if a complete raw
word was lost at crop. Otherwise add a suffixed Phase 2.3a before implementation,
specifying the diagnosed operation, scoped fix, failing regression, ledger
disposition, and pixel mapping. Do not paste a word from one genuine source line
into another.

**LLM verification:** run the applicable filtered ignored regression, ledger
tests, `cargo fmt`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test`.

**Human verification:** compare both sides of the rebuilt Tomus VI break with
the complete source line; check `quidem` occurs once in its proper place.

### 2.4 Angled v2 p266

Use shear if it restores all five right-edge glyphs without losing other Latin.
Mask Greek bleed only outside reviewed Latin. If shear does not restore a glyph,
use the word-patch contract for the specific remaining source pixels. Add tests
`source_word_patch_replaces_overlap_once` and
`source_word_patch_rejects_changed_source_identity` if the patch is used.
`tomus6_p266_right_glyphs_restored` checks exact source-pixel presence once for
`sive`, `praeter`, `(3)`, `auferro,`, and `idem`, plus line order and crop width.

**LLM verification:** run `cargo test --release tomus6_p266_right_glyphs_restored -- --ignored`,
any patch primitive tests, shear and ledger tests, `cargo fmt`,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test`.

**Human verification:** inspect all five word endings and the left margin in
v2 p266 and rebuilt Tomus VI p158; check the safe text area if widened.

### 2.5 `consi-` continuation, located v2 page

Use an existing mask for a proven removable streak, or the existing whole-strip
path for a proven `Tall` cut point whose scaled height fits one page. If the gap
is genuine, add only a `source_spacing_retained` evidence record. For a repair,
the baseline-failing `tomus6_consi_continuation_gap_closed` checks the gap against
neighboring source inter-line gaps and both lines once. For retained spacing,
instead use `tomus6_consi_source_spacing_retained`: check measured source-to-reader
spacing and intact Latin. That test may pass on the baseline; do not describe
retained spacing as a repair.

**LLM verification:** run the applicable filtered ignored regression and ledger
tests, `cargo fmt`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test`.

**Human verification:** inspect the source and rebuilt continuation together;
confirm either the closed artificial gap or the justified retained spacing.

---

## Phase 3 — corpus verification and delivery (Suggested implementer: Sonnet)

### 3.1 Verify the complete output

The faithfulness gate applies to every reviewed Latin line: present once, in
order, with source pixel values unchanged in the unscaled strips. The shear
multiset test is necessary but does not replace source-to-retained region checks.
Patch tests check both replacement and the absence of a duplicate remnant.

Run `cargo fmt`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test`. Run each applicable
new ignored release regression separately, plus ignored Latin pixel-survival
checks. Keep the inner loop in band/trace/pixel tests using the frozen current
production scale **1.2023952** for strip-budget checks; verify that scale again
in the final build. Set the established Poppler environment variables and
`PLUTARCH_LUALATEX` to the available MiKTeX x64 binary, then run:

```text
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
```

Record before/after Tomi I, III, and VI inner page counts and explain every
change. Inspect the five target locations and adjacent pages in the rebuilt
PDFs. Save a dated
`llm_artifacts/<ts>_SKEW_AND_TOMUS_VI_REPAIRS_VERIFICATION.md` with diagnosis,
per-defect before/after evidence, commands/results, the page-count table, and
any retained-spacing decision. Keep both plan and note as historical records.
Commit the work and open the branch PR after the five dispositions are verified.

**LLM verification:** confirm each command exits successfully; compare the
fixture with every installed table entry and inspect the rebuilt pages and
source-to-retained pixel traces.

**Human verification:** compare the five rebuilt reader passages and nearby
page breaks with their source scans; read the verification note and PR diff for
faithful pixels, one occurrence per Latin line, and expected spacing.

---

## Risks & open questions

- **Slope precision.** A single slope may not straighten a warped scan. A word
  patch can fix isolated crop loss but cannot repair a column whose lines still merge.
- **Shear vs. crop order.** Shearing changes row positions and canvas height.
  Remeasure the Latin crop and body-zone filtering on each transformed page.
- **Defects 3 and 5 may not be skew at all** — Phase 0 might reclassify them as ordinary
  mask/whole-strip cases, in which case no new mechanism is needed for them.
- **Patch width.** A widened crop must fit the safe composed text area; verify
  the scaled strip and PDF rather than assuming right-margin overflow is harmless.
- Do **not** add a global deskew or any change to global thresholds, body scale, the
  Latin-column detector algorithm, or the transcription cache. Every fix stays
  page-scoped and ledger-recorded, consistent with the rest of this branch.
