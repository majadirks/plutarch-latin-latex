# Tomus I Romulus III right-edge crop — implementation plan

## Context

The Tomus I facsimile drops right-edge Latin letters in Romulus III, sections
(2)–(4). In the supplied `build_facsimile/tomus_1_inner.pdf`, the relevant
passage is on PDF pp. **57–58**, which carry printed folios **43–44**. The
source is `Ploutarchou_Bioi_v1.pdf`, 1-based PDF p. **34**, at the bottom
of its right-hand Latin column. Locate the passage by its text and printed folio
as well as the PDF index when regenerating, since composition can change.

Earlier skew work on this source page straightened the scan, removed divider
fragments, and closed the oversized II/III chapter gap. Its regression checks
the heading and a narrow part of the right column, but it does not cover all
right-edge endings in III (2)–(4). The printed Latin is source material: preserve
the source pixels and spelling exactly. No transcription-cache edit is called
for by the observed crop defect.

## Current state of the world

- `src/facsimile/reflow.rs::render_and_band` renders source `(0, 34)`, applies
  `apply_known_source_shear`, masks furniture, excises reviewed blank rows,
  calls `production_source_column`, crops, and bands. The installed shear uses
  raw dimensions `1982×3008`, `x0_px=1400`, slope
  `-0.016079632465543646`, and `top_pad=23`. The post-shear row excision is
  `[1892, 2135)`, leaving a transformed image `1982×2797`.
- `production_source_column` calls `preserve_known_body_crop_edge`. Its guarded
  `(0, 34)` arm accepts a detector result `[950, 1774)` on the transformed
  raster and returns `[945, 1773)`. The right bound is exclusive, so any glyph
  ink at `x ≥ 1773` is discarded. The arm already has a post-transform pixel
  fingerprint. `band_source_column` has a second signature on the resulting
  `828×2797` crop and uses its interior text window to ignore gutter ink while
  finding line bands.
- `src/facsimile/audit_repair_tests.rs::romulus_chapter_break_deskewed` and
  `assert_reviewed_sheared_page` require the existing `[945, 1773)` crop and
  preserved heading bands. The shear/row-excision records in
  `src/facsimile/fixtures/tomi_gap_split_repairs_source_trace.json` also pin
  those edges and dimensions. They must be updated together with any measured
  crop change, while preserving the independently reviewed shear and gap facts.
- A close visual comparison of source p. 34 and reader folios 43–44 shows
  missing or truncated right endings on folio 44, including the lines ending
  `patrimonio`, `re-`, and `ex` in sections (3)–(4). These readings identify
  locations to inspect, not replacement text to insert. Compare their printed
  glyph shapes directly to establish the exact affected source-pixel boxes.
- The v2 p. 266 repair provides the closest code pattern: another guarded
  `preserve_known_body_crop_edge` arm extends the detected right edge, and
  `tomus6_p266_right_glyphs_restored` checks the restored source pixels and
  `source_strip_scale` page-width limit. Reuse that path and the existing
  `assert_sheared_source_region_pixels_survive` helper. The scale cap is
  signature-scoped: `source_strip_scale` currently recognizes only three
  reviewed columns, none of them Romulus p. 34. A new Romulus-specific guard
  is required if the widened column exceeds the current text measure.

## Desired end state

The complete rightmost glyphs of every affected Romulus III (2)–(4) line
survive once, in source order, on the Tomus I reader. The crop remains scoped
to the reviewed source raster and detector signature, keeps the original left
edge, excludes adjacent furniture, and fits the inner page's text width. The
chapter-break shear, mask, and gap repair remain
effective. The repair ledger, regressions, full Rust checks, and an offline
rebuild agree with the visually inspected source and reader pages.

## Tech Learnings

Crop bounds are half-open: `[left, right)` keeps the last column `right - 1`.
The shear changes each pixel's **row** by an integer depending on `x`, without
changing its `x` coordinate or grayscale value. The row excision then changes
rows below its slab. A source-pixel comparison must apply both row mappings;
the right crop bound itself is measured in the transformed raster's unchanged
`x` coordinates. Widening the crop may change its image width, the banding
signature, and the compositor's scale.

## Phase 1 — diagnose the exact right-edge loss (Suggested implementer: Opus)

### 1.1 Establish a reproducible baseline and measured glyph boxes

Render source v1 p. 34 at the same resolution as
`src/facsimile/reflow.rs`'s `real_source_image` test helper. Trace the existing
shear → masks → row excision → detected crop → guarded crop → banding path for
`(0, 34)` and record the actual dimensions, detector edges, final edges, and
band rows. On the raw source, mark every affected line's complete terminal
glyph rectangle in III (2)–(4), including at least the endings identified
above. For each rectangle, count dark source pixels at and beyond `x=1773`,
then map its rows through the shear and excision. Measure the smallest **safe**
new exclusive right bound that includes complete glyphs and a little whitespace;
inspect that boundary across the entire source page for Greek, divider, scan
edge, or other furniture. Keep the new bound as a measured value, not a guess.
Check whether folio 43 also has affected right-end glyphs.

Record the measurement and selected bound in the existing fixture's p. 34
diagnosis. The existing p. 34 `shear` repair record already carries
`crop_edges` and `preserved_latin_regions`; update the first in Phase 2.1 and
the second in Phase 2.2, following the v2 p. 266 pattern. Keep the separate mask and row-excision
records. A wider crop is the intended disposition if the missing glyphs are
present in the raw raster beyond the existing bound. If those pixels are
absent in the raw source, stop this crop path and diagnose the actual loss
stage before changing code. Measure the current composed strip width and the
new predicted width against `TrimGeom::default().content_width()` at the
production Romulus scale. Record whether the new width needs a scale cap.

**LLM verification:** Inspect a baseline raster/trace for `(0, 34)` and confirm
`[945, 1773)` before editing production code; save the measured source boxes,
ink counts, mapped rows, chosen bound, width comparison, and furniture check
in the fixture's diagnosis or a dated measurement note in `llm_artifacts/`. Run
`cargo test romulus_chapter_break_deskewed -- --ignored` to confirm the
existing regression's baseline.

**Human verification:** Compare the marked right-end boxes against source v1
p. 34 and reader folios 43–44. Confirm the exact terminal glyphs and that the
proposed right edge contains Latin without neighboring furniture.

## Phase 2 — apply the scoped crop repair (Suggested implementer: Sonnet)

### 2.1 Extend the existing guarded crop arm and update its dependent checks

In `src/facsimile/reflow.rs::preserve_known_body_crop_edge`, change the
`(source=0, page=34)` result's exclusive right edge from `1773` to the
measured Phase 1 bound. Retain its source/page, transformed dimensions,
detector-edge, and pixel-fingerprint guard, and its left edge `945`. Do not
add another shear, paste glyphs, or change the transcription cache. Run the
real-page crop and banding path. The existing `band_source_column` p. 34 arm
matches `(828, 2797)`, so update that dimension to the measured widened crop
size. Its fingerprint rectangle `[100, 600, 600, 750]` is away from the
changed right edge and its value should stay the same; recompute and change
the digest only if measured pixels in that rectangle actually changed. Check
the existing `Some((60, 798))` interior text window against the widened crop;
extend its right endpoint only as far as needed to band the restored terminal
letters while continuing to exclude gutter/furniture ink. Update p. 34
`crop_edges` in the shear repair record and `column_dims` in the row-excision
record to describe the **final** crop. Preserve any field describing the
earlier detected/raw crop. Update the existing
`romulus_chapter_break_deskewed` expected crop to the newly measured result,
keeping its heading, gutter, and gap assertions.

If the Phase 1 width measurement exceeds the text measure, add a guarded p. 34
column signature to the existing `source_strip_scale` predicate, using the
widened image dimensions and a reviewed fingerprint rectangle within the
retained column. Reuse its current `requested.min(content_width / width)`
calculation; do not introduce a second scaling path. Test the unchanged
requested scale for an altered same-size image as well as the capped scale
for the reviewed p. 34 image in an ignored real-PDF test named
`romulus_p34_wide_crop_fits_content_width`. If the measured width already
fits, that test should assert it fits at the ordinary production scale and
should not add a new scale signature. Re-measure band and canvas placement
after any scale change.

**LLM verification:** Add a small non-ignored test named
`romulus_p34_right_crop_guard_rejects_unreviewed_rasters` that checks unchanged
behavior for another page/source, changed dimensions, changed detector edges,
and a same-sized raster with a changed fingerprint. Verify the positive guard
on the real source raster in `romulus_chapter_break_deskewed`; a synthetic
white raster cannot match the production fingerprint. Run
`cargo test romulus_p34_right_crop_guard_rejects_unreviewed_rasters`,
`cargo test romulus_chapter_break_deskewed -- --ignored`, `cargo fmt`,
`cargo test romulus_p34_wide_crop_fits_content_width -- --ignored`,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test`.

**Human verification:** Inspect the widened p. 34 Latin crop at full size.
Confirm its right edge clears complete glyphs and does not include Greek or
scan furniture; confirm the left gutter and chapter-break spacing still look
as before.

### 2.2 Prove source-pixel survival through composition

In `src/facsimile/audit_repair_tests.rs`, add the ignored production-PDF
regression `romulus_p34_right_glyphs_restored`. For every reviewed affected
terminal rectangle from Phase 1, assert that source ink exists in the strip
`[1773, new_right)` and that
`assert_sheared_source_region_pixels_survive` finds each original grayscale
pixel in the retained crop at its shear-plus-excision mapped coordinate.
Require that the corresponding retained line band and the full-unit composed
strip appear exactly once, in order, on the reader canvases spanning III
(2)–(4). Use `compose_unit_at_frozen_scale("romulus")`,
`retained_page(&traces, 0, 34)`, and `RetainedSourcePageTrace::map_row` to locate
each reviewed line within the existing strip and canvas ranges, following
`romulus_chapter_break_gap_normal`. Reuse those helpers instead of building a
parallel mapper. Assert that `source_strip_scale` keeps the
widened line inside `TrimGeom::default().content_width()`. Keep a neighboring
unaffected source line in the check so a shifted or duplicated strip cannot
pass by accident. Extend the p. 34 fixture's preserved Latin regions to cover
the new terminal rectangles; the existing `inner_tomi_shears_preserve_latin`
test must exercise them too.

**LLM verification:** Run
`cargo test romulus_p34_right_glyphs_restored -- --ignored`,
`cargo test inner_tomi_shears_preserve_latin -- --ignored`,
`cargo test romulus_chapter_break_gap_normal -- --ignored`, `cargo fmt`,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test`. Prove the regression is meaningful by running it against an
explicit old-bound candidate or by checking its failure on the parent commit
before accepting its passing result with the wider bound.

**Human verification:** Review the source-to-retained pixel-box overlays and
the composed passage. Confirm each affected ending is complete, appears once,
and remains in its printed order.

## Phase 3 — rebuild and verify the reader (Suggested implementer: Sonnet)

### 3.1 Rebuild and compare the affected pages

Run the existing offline facsimile build with both source PDFs and the local
LaTeX/Poppler binaries as documented by the previous verification note:

```powershell
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
```

Render the rebuilt Tomus I folios 43–44 and their neighboring pages. Compare
them against source v1 p. 34, paying particular attention to III (2)–(4), the
right edge of each line, the chapter III heading and left gutter, page breaks,
and any scale change. Record actual PDF page indices from the rebuilt file.
Compare page count and other known skew repairs to their pre-change outputs;
investigate any unexpected reflow rather than assuming the old page indices
remain fixed. Save a dated verification note under `llm_artifacts/` with the
measured right bound, pixel-test result, build commands, and visual findings.

**LLM verification:** Run `cargo build`, `cargo fmt`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`, and the targeted
ignored regressions from Phase 2 after the final code and fixture edits.
Confirm the offline build exits successfully and the resulting Tomus I PDF
contains the reviewed passage once.

**Human verification:** Compare the source and rebuilt PDF at readable zoom.
Sign off that every right-edge letter in III (2)–(4) is visible and faithful,
while the chapter break, gutter, margins, and adjoining folios still render
correctly.

## Risks and decision points

- The correct right bound may exceed the width used by the p. 34 banding
  signature and the page's content measure. Re-measure its text window and
  add the scoped `source_strip_scale` signature when needed, following the
  v2 p. 266 pattern. Do not disable the gutter filter or copy v2's width.
- A full-width right extension might admit furniture on another vertical part
  of source p. 34. If so, use an existing source-scoped furniture mask only
  after proving it touches no Latin pixels. Keep the crop wide enough for
  every terminal glyph.
- The older p. 34 regression pins `[945, 1773)` as a prior output fact. Its
  crop assertion must change with the repair; its independently reviewed
  heading and gap expectations should continue to pass.
