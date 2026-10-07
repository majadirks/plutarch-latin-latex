# Horizontal slicing and lateral clipping verification

Implemented step 8.2 in `tools/audit_inner_tomi.py`. The clipping detector
registers retained prose independently against retrieved source alternatives,
then inspects original Latin glyph ink crossing the mapped body-image edges.
Full 300-DPI comparisons require retained ink correspondence and white reader
pixels where original glyph ink lies outside the boundary. Horizontal
components must cross the actual top/bottom edge; adjacent complementary
fragments are grouped within the same unit. Lateral cuts remain a separate
type. Exact missing glyph counts and wider extent remain unmeasured.

The shared `registered_pixels` helper supplies the full-resolution transform
and images to both region comparisons and clipping. Projected words outside
the physical reader page are rejected before raster bounds are calculated.
Pages without assigned source passages and registration failures remain
explicit in clipping diagnostics. Competing physical scans remain separate
image-supported observations, not established occurrence-coverage aliases.

## Actual verification

`tmp/pdfs/inner_tomi_audit/runs/step82-localized-final` processed all 158 body
pages of Alexander and Romulus. The manifest records complete requested
coverage, no errors, and the current script identity. Captured inputs were
rehashed before and after the run. Python compilation and `git diff --check`
passed. No unit tests or Rust checks were required for this script change.

- S1: `slicing_alexander_52_213_bottom` is high confidence, spans VI/52–53 and
  corresponds to v2/213. Its minimum retained-word correlation is 0.946286.
  Original whole words and the complementary reader fragments were visually
  inspected. The existing evaluator reports a localized S1 hit.
- C4: `lateral_clipping_alexander_53_213_left` is high, with 17 affected
  source-line observations and minimum correlation 0.861555. The source
  `rebat` and clipped reader beginning were visually compared; full independent
  source/reader overviews also show the documented line-beginning losses.
- C6: I/68 against v1/39 is high, with six affected-line observations and
  minimum correlation 0.940873. I/69 remains low: eleven observed lines have
  weaker correlations, with minimum 0.652951. Those counts describe detector
  observations, not a complete measured extent or exact lost-glyph total.

The evaluator reports one hit, no misses, two pending clipping annotations and
322 cases outside this requested scope. C4/C6 are pending benchmark box review;
their correspondence checks are not claimed as evaluated benchmark hits.
S2/C7 remain deferred validation examples.

## Revisions and limits

The initial run stopped on a projected source word beyond the physical reader
page. Guarding empty intersections resolved that failure. A subsequent run
used the selected occurrence chain alone and retained weaker source-page 211
evidence at the Alexander repeat. Searching every retrieved occurrence
alternative, with independent image registration for each, recovered the
stronger page 213 correspondence without finding-specific conditions.

The first complete alternative-source run saved boxes around two confirmed
word fragments and missed the predeclared S1 localization criterion at the
lower edge. The final evidence regions span the affected source line's
intersection with each actual body image. This displays the full implicated
line while retaining the individual confirmed glyph measurements; neither
gold coordinates nor the localization rule changed.

The final run contains 85 exploratory layout candidates: 82 lateral (22 high,
60 low) and three horizontal (one high, two low). Most remain unreviewed; this
is not a precision or corpus-recall claim. Alternative physical scans can
produce overlapping observations, and broad lateral extent still needs
adjudication. The earlier runs and manual source-assignment probe remain on
disk. The latter is direct image verification, not autonomous performance.

Production Latin, caches, source PDFs, reader PDFs/TeX and repair policy remain
unchanged.
