# Inner-tomi held-out validation and post-validation calibration

The audit preserves its first held-out predictions and evaluations. General
boundary, source-logo scale and clipping-context revisions are evaluated as
post-validation regression work. They do not create a second independent
validation sample. Latin source text, reader PDFs/TEX, repair policy and
captured inputs remain unchanged. This note publishes the completed step 10.1
revised validation; it is a disclosed post-validation regression result, not a
new independent validation seal.

## Preserved independent evidence

The first (independent) development producer is
`d9b1d05b6ad7a45777dd67f38cda892949e1a2b614a503e784970457dfbbac6a`;
the original development freeze is
`c99fd09855268cbddc197513051e1bc1fe37f6c5d073f1e4a778028dc4581650`.

`runs/step101-validation-first/` completes all 34 validation units and 1,340
body pages with no processing errors. Its runtime is 14,540.419356 seconds.
Original specs and producing script are retained. Candidate SHA:
`9a7c9e5662f54f9bb637c9c3d55b829d5eba8aaf5c75d087c71f720f7a259645`.
The original pending-label evaluation has 1 hit, 3 misses, 131 pending and 190
not requested; it is preserved unchanged. A separate retrospective score of
the actual same candidates against independently sealed reviewed gold has 7
hits, 46 misses, 82 clean negatives, zero negative violations and 190 not
requested. The sealed independent annotation hash is
`664c8ec4303adf05e095b5236570ae0cf9afae738adbf73e1931279e022b94a2`.

D9 and C5 pass. Mandatory first-run failures are D10/D11 endpoint localization,
D12 incomplete-opening recognition and W5/W6 source-footer scale handling.
The supplemental S2 split-line nomination is not fully localized in the first
run. Most endpoint failures are localization/OCR limitations; independent
source/reader inspection finds all 34 validation terminal passages complete.
Original 36 validation core events plus 56 frozen-development events produce
92/2,678 = 3.4354 per 100 pages.

All 36 first validation core events were independently adjudicated against
original Latin columns, adjacent full reader pages and native nominated crops.
The 31 omission warnings contain 21 confirmed and 10 false positives; four
duplicate passages are confirmed semantically, without converting their
failed localized endpoints into benchmark hits. The remaining head warning is
a high-confidence false positive: ordinary body `aiunt.` ends in the nominated
`t` on VIII/131. This page is outside the fixed panel. All 107 validation panel
pages and their touching core events were reviewed, with zero high-confidence
false-positive or uncertain pages. This first-run panel result does not replace
revised-run evidence binding.

The separate previously unlabeled sample has 31 available events after excluding
D9-D12 and C5 known regression targets, rather than an invented 50. It contains
20 confirmed and 11 false positives, with zero uncertain. The overlapping
low-confidence omission sample contains nine available events. Per-class
precision describes a candidate-selected inventory and does not establish a
representative page rate or corpus-wide recall.

## General revisions and annotation provenance

The source-footer matcher searches original source pixels across configured
scan scales in .025 increments, retaining the .80 high correlation threshold and
actual reader glyph support requirements. It caches positive and negative
matches with raster/template/settings/helper identities. W5/W6 original
footers match at scale .925 with correlations .9521487/.9519556; exact scale 1.0
produces no supported peak. Development exemplars retain scale 1.0 matches.
An image-function replay on actual cached reader OCR and preserved first-run
occurrence context nominates both actual faint upper gray logo fragments at
low confidence. A source match alone cannot establish retained reader ink.

Duplicate refinement records opening-line coverage even for a genuinely
incomplete later opening below .60 overall edge coverage. It retains printed
one-letter Latin prepositions such as `a`, and extends an end line only when
its expected terminal source word was not covered. Actual source/reader replay
localizes D4-D12 and preserves the incomplete D5/D6/D12 openings. General
refinement does not convert redundant-copy prefix loss into unique source loss.

After first validation, the independently prepared D11 Aratus opening locator
was corrected from raw OCR `ase` (printed `a se`) to the printed first word
`a`, using native ink and the visible word space. The raw OCR and original
sealed gold remain intact. Both original model boundaries fail this corrected
gold, so the boundary fix remains necessary. This is a disclosed
post-validation annotation correction, not a new independent gold seal.

Clipping comparisons include quarter-glyph-height raster context around raw
embedded word boxes, retaining the original boxes separately. Two-letter
words and Roman chapter markers are eligible for pixel correspondence.
Correlation, crossing-component, outside-ink and white-space gates remain
unchanged. Actual S2 replay matches `IV` at .723 and localizes the complementary
VII/244-245 fragments at low confidence; Exsilii remains below the .65 gate.
C7 total lateral extent and precise missing glyph count remain unconfirmed.

## Revised development run

The complete revised development run is
`runs/step101-development-revised-1/`. It covers 34 units and 1,338 body pages
with all detectors and workers 4, no processing errors, 38 hits, 25 misses,
118 clean negatives and zero pending/negative violations. All required core
positives retain their localized hits and strong confidence. The 56 core events
have identical independent source/output regions and crop hashes to the 9.2
review; all 93 panel pages have zero high-confidence false-positive/uncertain
pages. Full candidate count is 698, including 642 supplemental/endpoint records.
Development candidate burden is 56 grouped core events, 4.185 per 100 body pages.
Runtime is 3572.794345 seconds; run storage before the final manifest is
165,457,899 bytes and shared cache storage is 6,528,280,896 bytes.

Its official freeze SHA is
`77e6d6be418b169b56ff5b52432decd70d6450971cfeb06bf7bc472c1d166fda`;
manifest SHA
`39ce6ce8e0b79a2fc2c2706ddaa36dbbf96c4240e80a9abf347272095e3a3b91`;
candidate SHA
`6262949e6e8d8cc5aff496c29bdf82afd5eb6513c76046b07807eaada6ab1eb3`;
evaluation SHA
`dc2feef71f58247ea24760ae23330170f06339c0cf81cba87b9d10eca3036333`;
review SHA
`dac15fd449d258ec3f72ec1d8046a8378488edff9a1c377a59abf715fef73f89`.
The producer SHA-256 is
`748b8684b23d74c09da84f3c7a9170eaf185ca008f3b7522519ee40b13eb6227`.
The report was regenerated after evaluation. All 8,457 local references resolve
and 3,772 distinct actual candidate crop files match their recorded hashes.
Browser-rendered layout was not inspected.

## Revised validation run

The complete revised validation run is
`runs/step101-validation-revised-1/`. It covers all 34 validation units and
1,340 body pages with all detectors and workers 4, using the revised
development freeze and the exact retained producer
`748b8684b23d74c09da84f3c7a9170eaf185ca008f3b7522519ee40b13eb6227`. It reports
`complete:true`, no processing errors, no pending detectors, and matching
frozen source-index, settings and version identities. Runtime is 3279.862859
seconds; run storage before the final manifest is 144,911,460 bytes; shared
cache storage is 6,528,828,013 bytes. OCR language is `eng`; Tesseract
v5.5.0.20241111; Poppler pdftoppm 25.12.0. Full candidate count is 681.

Run identities:
manifest SHA
`aea092780dabddb3201c2c76349233a8cd5c752862c518b695f78bcfeea592c1`;
candidate SHA
`e65424565facace0292ddc17b4d77450bed5f4aeec48176759779509dfb2799b`;
evaluation SHA
`60d54d185712cd223f2a73dcae067781ed1621f0862fe67a41a2091c4d34b441`;
review SHA
`393486518811d88ccdf798a1169f4f0e294803aff182f372a060342ab1c41c7c`.

Every mandatory validation core case is now a correct-type localized hit:
duplicates D9 (`dup_cicero_463_466`), D10 (`dup_antonius_521_524`),
D11 (`dup_aratus_671_674`) and D12 (`dup_otho_699_702`); unique loss C5
(`miss_brutus_624_12394`); watermarks W5 (`watermark_brutus_0cdb555538eb`,
reader VIII/87) and W6 (`watermark_brutus_5c9b85db4567`, reader VIII/128); and
supplemental slicing S2
(`slicing_comparatio_demosthenis_et_ciceronis_244_480_bottom`). D10/D11/D12
revised contribution boundaries are byte-identical to the independently
measured gold in
`runs/step101-validation-gold-preparation/duplicate-boundary-replay.json`
(bound to the frozen labels SHA
`a96a58c6bd924704b7b4e1686cad91851a6f8503880ba182fa836e7354c747c9`), whose
localized-hit record is `true` for all nine duplicate cases. D5/D6/D12 retain
their incomplete second openings; approved D1-D3 exclusions create no unique
loss. W5/W6 reproduce identical source/output regions and every crop hash to the
reader-logo replay `independent-reviews.json`; both remain low confidence
because their strong reader predicates do not all pass. The head warning on
VIII/131 (`aiunt.` final `t`) remains a confirmed high-confidence false positive
outside the fixed panel.

Reviewed adjudication binds the revised candidate SHA. Thirty-three
first-validation verdicts transferred with identical source/output regions and
crop hashes; the three changed duplicate boundaries and the two newly nominated
W5/W6 watermarks received explicit verdicts against replay gold and reader-logo
evidence, with all revised crops present and hash-verified. Validation reviewed
precision: duplicate 4/4 = 1.0; watermark 2/2 = 1.0; missing_occurrence
21/31 = 0.677; head 0/1 = 0.0; order has no adjudicated events. All 107
validation fixed-panel pages are fully reviewed (`complete_panel_review:true`)
with zero high-confidence false-positive pages and zero high-uncertain pages.
Validation candidate burden is 38 grouped core events, 2.836 per 100 body pages.

The revised previously unlabeled candidate-selected sample
(`previously-unlabeled-sample-revised.json`) has 31 eligible core events after
excluding the D9-D12/C5/W5/W6 known regression targets: 20 confirmed, 11 false
positives, zero uncertain (precision 0.645). The overlapping low-confidence
omission sample contains nine events: seven confirmed, two false positives.
This candidate-selected precision is separate from the 200-page fixed-panel
false-positive rate and does not establish corpus-wide recall.

Revised HTML verification: all 3,867 distinct crop references in the validation
`index.html` resolve on disk; all 3,648 recorded candidate crop files are
present and match their stored SHA-256; the additional 219 referenced images are
retained rejected watermark probes. Browser-rendered layout was not inspected.

## Combined final gates and input verification

`runs/step101-validation-gold-preparation/verify-final-regression.py` verifies
both revised halves together and writes
`final-regression-verification.json`. All assertions pass:

- Mandatory core hits across the two runs: D4-D12, C1/C2/C3/C5, W4/W5/W6, H1/H2,
  each with the labelled correct type. Strong development positives remain high
  for D4-D8, C3, W4, H1, H2.
- Supplemental S1/S2 and C4/C6 are localized hits.
- Combined coverage is exactly 2,678 body pages and 68 units (1,338 development
  + 1,340 validation, disjoint), no processing errors, matching settings,
  producer SHA, source-index identity and tool versions across both runs.
- The eight unique missing source lines across C1/C2/C3/C5 remain distinct.
- Grouped core events total 94, i.e. 3.510 per 100 body pages (<= 5).
- All 200 fixed-panel pages are fully reviewed with zero high-confidence
  false-positive pages and zero high-uncertain pages.
- All 68 terminal passages were independently viewed complete. 42 endpoint cases
  are algorithmic localization/OCR misses, not confirmed truncation.
- 23 supplemental M/R/F/C layout cases remain explicitly exploratory misses,
  including M10/M11/R13 and unresolved C7 extent; these do not erase core
  success.
- Every live and captured input hash matches
  `snapshots/baseline_2026_10_06/manifest.json` (both source PDFs, 8 reader
  PDFs, 8 TEX files, unit map, repair trace and findings report).

Dependency profile (from both run manifests): Python 3.13.14; NumPy 2.4.3;
Pillow 12.1.1; opencv-python-headless 4.13.0.92; scikit-learn 1.9.0;
RapidFuzz 3.14.3; PyMuPDF 1.27.2; Tesseract 5.5.0.20241111; Poppler 25.12.0.

## Limitations

Known core regression targets are mandatory checks, not proof of corpus-wide
recall. Candidate-selected precision describes a chosen inventory, not a
representative page rate. Low-confidence nominations (W5/W6, S2) are localized
findings, not high-confidence assurances. Independent ending imagery establishes
terminal completeness, not interior completeness; matched endpoints do not prove
interior prose is intact. Browser-rendered HTML layout was not inspected. This
revised run is disclosed post-validation regression work and does not claim a
fresh independent validation seal. Source Latin, reader PDFs/TEX, repair policy
and captured inputs are unchanged.

## Delivery and remaining work

Step 10.1 is complete and committed on branch `inner-tomi-audit-phase-1`. The
user requested a pause after 10.1. Step 10.2 remains responsible for the
explicit all-unit final run, resume stability, final-producer workers 1/4
comparison and completion evidence. The main README section about detected
issues, invocation, reports and methodologies follows 10.2. A detailed 10.2
implementor handoff accompanies this delivery.
