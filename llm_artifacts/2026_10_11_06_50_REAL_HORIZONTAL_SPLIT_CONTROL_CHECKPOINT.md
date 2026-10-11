# Real horizontal-split control checkpoint

## Fixed acceptance controls

Commit `e078a2d` records the two user-reported defects in
`tools/facsimile-real-acceptance-controls.json` and inserts step 7.1a without
renumbering existing plan steps. Both original source rows were visually
inspected independently of the reader outcomes. Source Latin, the transcription
cache and production repair decisions remain unchanged.

| Control | Intact source | Defective reader join |
| --- | --- | --- |
| Lycurgus–Numa comparison | Source v1/105, box `[1083,2184,1880,2235]` at 300 DPI | Tomus I physical 212–213, folios 196–197 |
| Pericles | Source v1/200, box `[928,2537,1720,2578]` at 300 DPI | Tomus II physical 20–21, folios 12–13 |

The Pericles quotation and folios identify physical pages 20–21 in the current
authenticated PDF. The originally reported 21–22 is retained as provenance.
The user excerpts are locators, not replacement transcriptions. These two
controls remain separate from the 200-page random panel and 54 earlier targeted
controls. Header leakage is outside scope.

## Targeted detector and measured results

`facsimile_line_delivery.py controls` delegates to the shared glyph-cut predicate
in `facsimile_glyph_splits.py`. The control mode uses fixed reviewed source boxes
and reader page pairs. Intact neighboring OCR words provide independently
screened paired-glyph scale and translation bounds. The existing strict shared
SIFT/RANSAC registration primitive supplies four valid page-edge transforms;
no registration or ink tolerances were loosened.

The predicate requires actual visible support for upper and lower portions of
the same required source components on consecutive reader pages. Both captured
threshold observations must support the finding. Combining ink across pages
does not establish readable glyphs. Two or more persistent components are
required, using the captured clipping setting.

Both real controls pass: 22 persistent crossing components in the comparison
line and 24 in the Pericles line. These are component counts within two affected
lines, not 46 missing lines. All four fits pass scale, word-box corroboration,
inlier, width, residual, translation and placement-ambiguity checks.

This is targeted source-backed detection, not proof of blind full-corpus
localization. It preserves the machine ledger and records comparison completion
separately from control measurement. General seam integration remains unfinished;
step 7.1a is not declared fully complete. The original calibration freeze and
full-run acceptance work also remain outstanding.

## Verification and durable evidence

Two intact neighboring source rows, measured against actual reader pixels with
the validated transforms, produce zero glyph-cut findings. Altered source hashes
and wrong reader-page/folio pairs are rejected with exit 2. All ten source,
reader-edge and projected-reader images retain their recorded hashes, and all
ten HTML image links resolve. The report presents images before expandable
measurement records. Component coordinates are relative to the required source
crop; the enclosing source box uses full-source coordinates.

Python compilation, `cargo fmt --all --check`, strict all-target Clippy and
`cargo test` pass. The existing suite reports 419 library and nine CLI tests
passed, with 76 existing tests ignored. No unit-test framework was added. All
21 protected source/reader/unit-table/repair-ledger files retain captured hashes.
The preexisting `BACKLOG.md` edit remains outside these commits.

| Evidence | Location or identity |
| --- | --- |
| Fixed controls SHA-256 | `b16a5c094626ffa3d5fbadc41613a152c43e4af2a082d038d37bf51d24b943bc` |
| Glyph-cut helper SHA-256 | `a251fd3a83cbdae2d1926dadc5e25180549051fcdde9b050329198567e217c35` |
| Delivery CLI SHA-256 | `a2f6885415227fb58f520d56b1ce868ffd089b0ffba84c1aa8192e01d05637bd` |
| Final report JSON | `tmp/pdfs/facsimile_line_audit/runs/verification/known_horizontal_splits_2026_10_11/controls_delivery_3.json` |
| Final report HTML | `tmp/pdfs/facsimile_line_audit/runs/verification/known_horizontal_splits_2026_10_11/controls_delivery_3.html` |
| Final evidence directory | `tmp/pdfs/facsimile_line_audit/runs/verification/known_horizontal_splits_2026_10_11/controls_delivery_3_evidence/` |

Earlier registration probes and control predictions remain on disk. The first
probe's whole-body scale estimates disagreed on two edges; using paired intact
anchor words, as in the existing comparison method, resolves those differences
without relaxing validity gates. No paid model request was made.

The ongoing full candidate run retains all five captured comparison-code hashes
and continues processing local strips after completing its 1,315-page source
inventory. The independent control helper does not invalidate that run or alter
its cached verdicts. Resume work from this checkpoint by integrating the shared
predicate into general seam evidence, then verifying the completed full run,
its exact resume, panel/calibration results and final archive.
