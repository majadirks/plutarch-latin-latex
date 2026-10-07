# Watermark template matching verification

Implemented audit-plan step 6.1 in `tools/audit_inner_tomi.py` using the reviewed
development source logos on v1/287, 299 and 321. `make_templates` derives full
logos, Google text, distinctive glyph groups and shallow horizontal fragments
from those original grayscale pixels. Every fragment retains source coordinates,
pixel identity and glyph bounds. Constant/near-empty templates and nonfinite
correlation scores are rejected.

`match_templates` compares grayscale and local-contrast views using normalized
correlation, the specified scale/rotation bank, and consistent raster glyph-height
measurements. Faint-ink support separates gray logo pixels from ordinary black
Latin shapes. Full-resolution retries preserve failed confidence predicates.
High confidence requires stable views, at least two observed glyphs, and either
a distinct second group or occurrence-aligned source-tail corroboration.
Source-template exemplars are separate from actual occurrence source regions;
an exemplar is not claimed as the physical source of an unrelated reader patch.

`source_context_at_region` reuses the occurrence path to locate prose before and
after an image event. `source_tail_correspondence` checks the original footer of
the preceding physical source page at a forward source-page transition. This
avoids assuming that the development template page itself supplied the output
logo. The current helpers do not read labels or finding anchors.

Up to four workers perform pure NumPy/OpenCV searches. PDF adapter calls stay in
the main process. Equivalent template pixels share comparison work; rendered
image matches are cached with their pixel/template identities, settings, helper
identity and OpenCV version. Existing source-reference vocabulary remains
transform-only when reader OCR is processed.

## Actual verification

Final run: `tmp/pdfs/inner_tomi_audit/runs/step61-final`, baseline
`baseline_2026_10_06`, development units `aemilius_paulus`,
`marcius_coriolanus` and `timoleon`, `--detectors watermark --workers 4`.

- All 150 requested body pages completed; captured input hashes were verified
  before and after processing.
- 30 source fragments were prepared from the three reviewed source logos.
- 67 component events remain visible: one high-confidence candidate, four
  low-confidence candidates and 62 low-confidence inconclusive events.
- W4 has six localized components on II/273-274. The strongest component on
  II/274 has refined correlation 0.975512, stable views, observed multi-glyph
  support and source-tail corroboration at v1/321. Its surrounding occurrence
  assignments move from source 321 to source 322.
- Repaired seams II/199 and II/227 have zero nominations. Coriolanus and
  Timoleon have no high-confidence watermark event in this run.
- The source footer and reader Google-fragment crops were visually inspected.
  Ordinary black-letter curve crops were inspected too; the broad initial pass
  had nominated 433 such shapes without finding W4's shallow slices. The
  retained calibration runs document the typography/fragment corrections.
- Every recorded evidence crop exists and passes the evaluator's hash checks.
- `python -m py_compile tools/audit_inner_tomi.py` and `git diff --check` passed.
- The existing evaluator accepted the final candidate schema and source bounds.
  It reports one miss, four pending negative annotations and 320 cases not
  requested. W4 is not yet counted as a hit because the label requires both
  output pages in one event. Cross-page event grouping is the next step.

No Brutus template or validation reader detection was used. No source Latin,
transcription cache, production mask or reader was changed. No unit tests or
Rust checks were added or run, following the diagnostic plan's instruction.

## Remaining work

The 62 inconclusive nominations include coarse antialiasing matches on ordinary
Latin strokes that fail the 300-DPI comparison. They are preserved for review;
this run does not meet the final candidate-burden target or establish panel
precision. Overlapping components and W4's two output locations need logical
event grouping. The repaired negative labels still require independent gold
measurement during the reporting/calibration work. Grayscale and contrast are
two views of the same raster, not statistically independent evidence.
