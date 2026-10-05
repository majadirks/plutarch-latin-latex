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

## Lysander repair, step 2B.1

The proposed alignment predicate passes the real 300-DPI regression without widening its thresholds. The closing gap is 17 pixels, below the 22-pixel ceiling. `lysander_trace_after.json` records no loss; the retained closing crop was visually checked through `commemoranda habuimus.`. Four metric tests, five line tests, and 45 nonignored reflow tests pass, as do formatting and Clippy. Centered signatures and thin rules remain excluded in synthetic regressions. The exhaustive scaled-composition checks are added in the following shared-verification step.

## Shared source/composition checks, step 2B.2

The raw 300-DPI Latin columns for 434, 551, 637, and 705–708 were inspected in full before freezing `src/facsimile/fixtures/closing_body_bands.json`. The ledger enumerates all substantive body bands, source furniture, and tiny noise intervals separately. The trace requires inclusive render and successful-detection ledgers before indexing pages. Every reviewed body interval must survive once with byte-identical unscaled pixels. Furniture must contribute no packing strip.

The independent stream assertion resizes source cut-point intervals once, places each on white rows with the real horizontal clipping, and compares dimensions and bytes against decoded canvas rows after removing their leading inset. It covers noninteger scaling, ordinary wrapping, tall splitting, and a rejected control signature. Cato and Lysander both passed these real-source assertions at the measured production global scale; neither test silently skipped prerequisites. Synthetic stream checks and the 47 nonignored reflow tests passed, alongside formatting and Clippy.

The plan records two measured refinements: rejected noise can influence surrounding whitespace cut points without contributing text, and Otho's first body bands on pages 706/707 need a narrow geometry-based rescue from the top zone. The shared 8% furniture API remains unchanged. These refinements follow source-column inspection rather than weakening the exhaustive body ledger.

## Terminal endpoints and restored pages

Step 2C.1 (`7ed6d76`) passed the explicit endpoint/range/request and pin-precedence fixtures, all 26 nonignored detector tests, and nine unit-table tests. Real-source detection with an absent temporary cache and no pins passed, ending at 637/708. The operational cache was never redetected by a test. Invalid endpoints fail before saving, and OCR detection is bounded before duplicate terminal scans.

Step 2C.2 (`c35f6bc`) passed the complete page-637 body ledger, first/middle/last OCR-location checks, and exact scaled stream. Its old endpoint 636 fails the same endpoint assertion on an isolated cloned row. The retained page was visually inspected through `fuisse.`, with no detached ornament.

Step 2D.1 passed the ignored real-source Otho test after demonstrating failure against the actual old operational endpoint 704. Every source page in 696–708 renders and detects successfully once; 709/710 are outside the range. Every reviewed body band on 705–708 survives with source pixels intact, and the entire resized row stream matches composition. The concluding crop includes chapter XVIII and the last hyphenated sentence. Pages 706/707 also retain their first body lines through the geometry-based top-zone rescue.

The page-708 ledger initially failed on the library stamp at rows 2080–2117. The plan now specifies removal of a distant narrow furniture cluster after aligned body text. Its synthetic stamp/rule test passes, as do all 50 nonignored reflow tests, formatting, and Clippy. The remedy shares the ink-alignment helper with short closing-line protection; neither page numbers nor Latin strings drive production filtering.
