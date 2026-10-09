# Antonius 23-line block — test-install findings

Written 2026-10-09. This records the test-install proposed in the
[Tier-B repair plan](2026_10_09_14_45_TIER_B_OMISSION_REPAIR_PLAN.md) and
[band extraction](2026_10_09_14_30_TIER_B_BAND_EXTRACTION.md).
**The plain single-band trial failed the reader check.** Its protection fired, but
the trial inspection reported that the 23 lines were still absent from their expected
reading-order position. The first downstream loss or unreadability stage is not yet
established. The 19 other events / 21 protection records can proceed independently.

## What was tried

Installed exactly the tabulated entry: one SourceBodyBand for antonius v2 p562,
dimensions (815, 2835), observed 249..1356, protected 249..1356. The detector emits
the 23-line conclusion as one band at LINE_MIN_INK = 3. A full offline rebuild of
all eight inner readers was run with a warm render cache, taking about five minutes.

## What happened

The recorded trial established successful protection followed by a failed reader check:

- extract_tier_b_source_bands reported survives_render_and_band=true for 249..1356.
  The scratch debug_antonius_p562 trace reported that the band was retained in the
  unit's final bands, classified Tall, and associated with a strip 249..1364.
- The rebuilt tomus_7_inner.pdf grew from 416 to 448 pages. The recorded inspection
  still found the jump from "… Alii in seria inclusam aspidem asser-" to
  "LXXXVII. Antonius ex tribus uxoribus …". It reported no appearance of the distinctive
  words Archibius, talentis, venenum, fistula, undequadraginta, Enimvero, or aureo fuso
  in the Antonius tail. These are the trial's observations, not a substitute for
  source-to-canvas-to-PDF pixel coverage.
- The inner reader uses 32-page signature padding, so total page-count growth cannot
  establish how many body pages changed or whether the block reached a canvas.
  Record unpadded composition counts and embedded image identities in the next trace.

## Causal explanation still to establish

The measured absence of broad blank runs makes automatic horizontal cuts uncertain,
but does not by itself explain disappearance of the complete block.

compose_pages preserves the existing raster layout; it does not reconstruct individual
lines. Its lowest_ink_row_in selects the least-ink row in any nonempty window even if
every row contains ink. The split retains both its piece and remainder. Thus unsafe
cuts can bisect glyphs, but a claim of complete loss requires a trace beyond band
retention. Strip ink filtering, packed canvases, final canvas filtering, source
configuration, generated image identities, and PDF embedding must all be checked.

The original probes reported no run of at least six near-blank rows and approximately
four groups with dense-coverage separation at 0.30. Those probes do not establish
safe line cuts or rule out every alternative. line_bands splits when a row's ink
count is at most 3; it does not require a six-row blank run. The saved source crop
also shows sloping lines, so nominal 48-pixel pitch cannot supply glyph-safe horizontal
boundaries. Marginal letters and tight leading may contribute to merged detection;
the first downstream failure remains an open diagnosis.

A read-only check of the cached p562 raster, matching the saved 815-pixel-wide block
crop to full-source bounds [906,1721], found the original 249..1356 parent removed
by the unchanged furniture gap filter. This supports the possibility of grouped
protection records on that raster; it does not prove their glyph-safe composition.
The Rust production trace must confirm the crop, filters, and configuration.

## Required next evidence

The plan's Step 3.1 requires a reproducible full-unit trace at the full-work production
scale, with separate baseline, plain-entry, and candidate outputs. It must follow
all 23 reviewed glyph rectangles and their neighboring anchors through:

- source raster identity, raw and production column bounds, and detector bands;
- furniture-kept parent presence, protection after Pass A, and final Pass-B bands;
- actual strips, classifications, ink, scaled dimensions, budgets, and cut positions;
- packed canvases before filtering and returned canvases after filtering;
- generated image hashes, fresh tomus 7 PDF embedding, and physical output pages.

Reuse audit_traced_unit, audit_save_unit_trace, assert_scaled_stream, and
assert_reviewed_source_lines_survive. Record the first failed boundary. A failed
diagnostic is retained as evidence; it must not become a passing restoration claim.
OCR absence alone does not establish missing source pixels.

## Candidate approaches

Step 3.1 in the repair plan evaluates whole-block preservation first, then a faithful
crop adjustment, then explicit bands. These possibilities remain unverified:

1. **Hand-measured line bands.** Derive 23 complete glyph rectangles, protected
   intervals, and actual cut_points midpoints from source overlays. Preserve strict
   gaps so the helper does not merge touching bands. Shared observed 249..1356 is
   usable only when it remains the actual detector signature and its parent is absent
   from kept. Require exactly 23 distinct bands after both protection passes,
   ownership/signature mismatch checks, and each whole reviewed line on one returned
   canvas. If adjacent skewed glyph extents overlap vertically, reject disjoint
   horizontal splitting; metadata gaps do not create source whitespace.

2. **Crop-edge adjustment.** Test measured production bounds over the entire p562
   body, including the following chapter and leftmost glyphs. The original confirming
   read approved the in-block section letters as harmless source content to retain.
   Do not silently discard them to manufacture detector gaps. Record marginal
   artifacts and reject a crop that removes approved content. All pixel assertions
   must use the production crop after the edge correction, not raw detector bounds.

2A. **Preserve the block whole.** Measure the complete strip, including its surrounding
half-gap, at the production scale. If it fits a fresh body canvas, evaluate the existing
is_known_whole_source_strip exact-signature/fingerprint pattern to wrap the block whole.
Verify all 23 lines and neighbors in the returned canvases and freshly compiled PDF.
This candidate avoids a forced per-line split; it has not yet passed a trial.

For any selected method, archive exact measurements, source overlays, configuration
identities, and passing evidence. Add the method's named regression and the 23-line
whole-glyph composition test before calling the repair installed or restored.

## Delivery scope

Antonius remains historically initial_tier: "B" with a confirming-read promotion.
Its current repair outcome is unresolved until source, composition, and real-eyes PDF
checks pass. Preserve historical classifications and the failed-trial evidence.

A partial delivery restores the other 19 events / 21 lines and explicitly reports
Antonius unresolved (one event / 23 lines). Full completion requires all 20 events /
44 lines restored. The machine-readable ledger, portable evidence, and verification
note must agree with the selected delivery scope.

## State left behind by the trial

The trial entry and scratch debug_antonius_p562 trace were reverted from
src/facsimile/reflow.rs. extract_tier_b_source_bands remains as a diagnostic.
The recorded trial left build_facsimile holding test-install output. Rebuild from
the intended configuration and confirm fresh compilation before using those PDFs
as deliverables; a source revert alone does not regenerate them.
