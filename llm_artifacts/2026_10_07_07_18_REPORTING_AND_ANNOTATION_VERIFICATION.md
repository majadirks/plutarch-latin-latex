# Reporting and independent development annotation verification

Step 9.1 of the inner-tomi Python audit implementation plan is complete. The script writes static local HTML, CSV, enriched raw-observation JSON, requested-page coverage, and explicit error/limitation records. `report --run-dir ...` regenerates these artifacts for an existing run without changing candidate scoring, regions, confidence, or identities. The existing evaluator supplies saved hit/miss counts; there is no second evaluator.

The HTML exposes every recorded high, low, and unknown candidate or endpoint observation. Evidence includes original source and reader crops, physical-page locators, printed source folios when independently observed, reader body folios with printed/inferred/unknown provenance, raw source/reader word observations, confidence reasons, component scores, related events, and region coordinates. Original observations remain diagnostic: scores are similarities rather than probabilities, and complete OCR word boxes can extend beyond the displayed region. HTML text and local paths are escaped; CSV strings that could execute as spreadsheet formulas are quoted with a leading apostrophe.

## Actual corpus verification

Existing development runs were regenerated and checked against their unchanged candidate JSON and run manifests:

| Run | Candidates/observations | Requested coverage rows | Local HTML links checked | Raw output word observations | Unknown source folios |
| --- | ---: | ---: | ---: | ---: | ---: |
| `step82-localized-final` | 85 | 158 | 821 | 908 | 23 |
| `phase5-romulus-g` | 2 | 49 | 15 | 26 | 0 |
| `step62-aemilius` | 11 | 52 | 85 | 22 | 0 |
| `phase4-agis-3` | 1 | 28 | 33 | 1,179 | 0 |

Every HTML article identity matched CSV, candidate JSON, and enriched report JSON. Every local link resolved to an existing file. Coverage row counts matched actual requested pages. CSV confidence and scores matched JSON. Candidate file hashes remained unchanged, and the generated HTML contains no script elements. Evidence covers an omission, repeated reader contributions, a faint split watermark, and the independently reviewed genuine-spacing negatives. The verification record is `tmp/pdfs/inner_tomi_audit/runs/step91-report-verification.json`.

A fresh development run, `step91-report-fresh`, processed the four body pages of `comparatio_thesei_et_romuli` with boundary detection and automatically wrote all report artifacts. Its existing evaluator scored the independently reviewed E03 endpoint as one hit, zero misses, zero pending annotations, and 324 cases outside requested scope. Regenerating its report after evaluation exposed those saved counts. This verifies reporting integration, not full development detector performance.

The in-app browser skill was read and its available-browser inventory queried, but no browser backend was connected. Browser rendering was therefore unexercised. HTML structure, escaping, local links, and the underlying source/reader images were checked directly; a human should still open the HTML to assess browser layout.

## Independent annotations

All 187 development cases are reviewed, including the 180 previously pending cases. The tracked `tools/inner-audit-labels.json` and diagnostic `spec/labels.json` agree. Validation annotations remain unchanged: 134 cases are still explicitly pending until independent review before their evaluation. No validation reader detector run or scoring occurred.

Annotation measurements used original source/reader rasters, original embedded word coordinates, diagnostic reader OCR, and the findings report as locators. Detector predictions were not annotation inputs. The ignored annotation runs retain original overviews, measurement drafts, localized crops, and reviewed boundary sheets. They include development endpoints, generated headings, D4–D8 duplicate contributions, redundant second-copy openings, clipping/scraps/reference letters, copied-margin seams, rules, genuine spacing, repaired furniture, intentional notices, and the white separator.

Independent raster review corrected annotation attribution where the report named a prose seam rather than the scan contributing surviving rule ink: R4 uses source 289; R5 uses 297/307/315; R7 uses 329; R9 uses the single ruled scan 375; R10 uses 297. Their provenance records the corrections and image evidence. The repaired following top rules retain separate negative regions. This prevents a surviving rule and its repaired neighbor from becoming contradictory gold on the same reader page.

Endpoint review also distinguished a representative report anchor from actual final prose. E03 includes the final split continuation; E27 includes the terminal Philopoemen reference; E19/E28/E65 exclude following comparison/unit headings; E52 includes its three actual terminal prose lines. Margins include visible glyph ink beyond embedded/OCR word boxes. Production Latin, transcription caches, reader/source PDFs and TeX, and repair policy were not edited.

## Checks and remaining work

`python -m py_compile tools/audit_inner_tomi.py`, benchmark schema validation, and `git diff --check` passed. Snapshot/source/reader/policy hashes were verified repeatedly and remained unchanged. This script-only step does not require a Rust build, lint framework, or unit-test suite.

Step 9.2 must evaluate complete development coverage with these reviewed annotations, resolve core misses and excessive candidate burden through general evidence-backed methods, review development panel pages, and bind the complete development freeze. Historical partial-run evaluations remain historical: changed annotation hashes require new scoring runs. Held-out validation and full-corpus completion remain deferred to steps 10.1 and 10.2. After step 10.2 is committed and pushed, the repository README must gain the user-requested section describing detected issues, execution, report navigation, and methodologies.
