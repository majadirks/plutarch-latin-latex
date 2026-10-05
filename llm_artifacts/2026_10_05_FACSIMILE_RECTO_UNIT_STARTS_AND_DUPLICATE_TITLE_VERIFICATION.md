# Verification record

Implementation follows the recto and duplicate-title plan on branch `plan/facsimile-body-blank-pages`. Each completed step is committed and pushed independently. Source text is reproduced from its existing pixels; transcription text is unchanged.

## Baseline

Evidence is saved under `tmp/pdfs/recto_units_verification/baseline/`: existing inner TeX and logs, compiled page counts and image listings, decoded PDF images, source and boundary renders, OCR boxes, and SHA-256 manifests. `source_bands_and_widths.json` records production column widths and ordered retained source-band hashes from unchanged filtering. The baseline global 95th-percentile column width is 837 pixels at 300 DPI.

## Completed checks

- Step 1.1: formatting, Clippy with warnings denied, and 23 assembler tests passed; one source compile test remains ignored until the compiled-regression step. Commit `8994687`.
- Step 2.1: the page-434 regression failed with the old midpoint predicate, then all six title-box tests passed with the corrected predicate. Formatting and Clippy passed. Commit `9eef738`.
- Step 2.2: eight range/cache tests and 42 reflow tests passed. The explicitly requested real-source Cato boundary test passed at 300 DPI. Retained crops were visually inspected: Cato includes its full final sentence; the Comparatio starts at its first body line, with neither copied title line. Formatting and Clippy passed. Commit `37dffe1`.

## Lysander diagnostic, step 2B.1A

The ignored regression fails on unchanged filtering with source ink present. The complete trace is `tmp/pdfs/recto_units_verification/lysander_trace_before.json`, and test output is `lysander_before_test.txt` beside it. Source 0/page 551 at 300 DPI has column bounds `[1063,1859)`. The closing band's rows are `[2566,2595)`, its horizontal ink span is 359 pixels, and unit metrics are height median 33, height MAD 2, width median 779. The median inter-band gap is 10 pixels.

Zone and gap filtering retain the closing line. `drop_interior_running_head` loses it through its call to `drop_trailing_furniture`; direct tracing of that trailing function confirms the same loss. This selects the planned trailing-furniture repair, without changing zone/gap filtering or metric sampling. No filtering predicate was changed for the diagnostic step.
