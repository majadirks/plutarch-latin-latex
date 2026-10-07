# Rules, blank-body hypotheses and source spacing

Implemented step 8.1 of the inner-tomi audit plan in the single diagnostic
script. `find_rules_and_gaps` uses mapped body placements, measured glyph
heights, collinear vertical ink and source-corresponding prose around large
gaps. Blank-body hypotheses require a regional OCR retry; unexplained text-like
ink remains inconclusive. Adjacent-page whitespace is examined within a unit,
so sparse conclusions are not treated as copied margins.

## Actual verification

The development run `tmp/pdfs/inner_tomi_audit/runs/step81-refined` processed all
199 body pages in Romulus, Coriolanus, Timoleon and Aemilius Paullus. No tool or
OCR errors were reported. Captured inputs were rehashed before and after the
run. The script compiled and `git diff --check` passed.

The run produced eight high-confidence geometric rule candidates and four
copied-gap candidates: three high, one low. Full-resolution line correlations
for the high gaps are 0.936389 at II/273–274, 0.883276 at II/207 and 0.876385 at
II/223. The II/290 gap remains low with correlation 0.630698. These values
measure image similarity rather than certainty.

All seven large interior Romulus gaps in the run were retained as negative
source-spacing comparisons, including genuine chapter spacing on I/61 and
I/62. Their neighboring line correlations range from 0.914173 to 0.970966.
An earlier run, `step81-development`, retained three incorrect low gap
nominations because coarse anchors reached the next source page. Comparing
both neighboring lines against the plausible original pages resolved those
assignments without page-specific detector conditions. Both runs remain on
disk.

The II/203 rule and II/223 whitespace evidence crops were visually inspected.
No blank-body candidates were produced, including at the sparse unit endings
in this scope. Genuine I/36 and I/46 spacing, and Brutus examples, belong to
validation units and remain deferred until development freeze.

The existing evaluator reports 37 pending annotations and 288 cases outside
the requested detector/unit scope. No benchmark recall or panel false-positive
rate is claimed. Some documented copied-margin gaps fall below the initial
three-line-pitch nomination threshold; broken rule fragments can fall below
the minimum connected extent. These supplemental misses require calibration
and remain explicit rather than being counted as clean pages. Source-gap
measurements under one affine transform cannot establish arbitrary added
spacing without independent piecewise correspondence.

No production text, reader, source PDF or repair policy was modified.
