# Inner-tomi audit: completed development calibration and freeze

Step 9.2 of `2026_10_06_16_05_INNER_TOMI_PYTHON_AUDIT_IMPLEMENTATION_PLAN.md`
is complete. The final development run is frozen at
`tmp/pdfs/inner_tomi_audit/runs/step92-development-final-4/`.
The authorized branch is `inner-tomi-audit-phase-1`. Pause after committing
and pushing this step. Validation detection has not started; step 10.1 is the
next implementation step. The requested main README section remains queued
for after step 10.2 is committed and pushed.

## Scope and source faithfulness

The actual final tool processed all **34 development units**, all **1,338
body pages**, and all requested detector types. Requested/completed page sets
match the captured reader inventory. There are no processing errors, pending
detectors, unreadable reader OCR pages or unreadable source OCR pages. The
report retains **166 observational limitations**, so completed processing does
not establish clean pages or successful registration everywhere.

All 21 live baseline inputs and their captured copies were independently
rehashed against the manifest. Working labels, splits and templates equal the
tracked files. Latin, transcription caches, PDFs, LaTeX, Rust, production repair
policy and snapshot inputs were not edited. Diagnostic comparison views retain
raw observations and do not rewrite source text. Source reference extraction
and TF-IDF fitting cover all 68 original-source units; validation reader output
was excluded from detector scoring and fitting. No benchmark label was changed
to accommodate a detector result.

## Complete-run comparison

Each run retains its immutable producing script, manifest, native process
launch/stdout/stderr, candidates, crops and evaluation. Metrics below concern
actual complete runs over the same development body scope.

| Run suffix | Benchmark hits | Misses | Negatives without high events | Negative violations | Core events | Core events per 100 pages | Elapsed seconds |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| final-2 | 37 | 26 | 118 | 0 | 84 | 6.278 | 20316.230 |
| final-3 | 37 | 26 | 118 | 0 | 77 | 5.755 | 6764.222 |
| final-4 | 38 | 25 | 118 | 0 | 56 | 4.185 | 5515.611 |

The final result has **38 hits, 25 misses and 118 requested negatives without
high candidates**. All development annotations in scope are reviewed. The
144 not-requested cases are 138 validation labels and six development labels
on nonbody blank/notice pages; they are not counted as processed negatives.
Core burden falls by one third, from 84 to 56 events. The complete report also
retains 605 supplementary/endpoint observations, for **661 candidates** total,
and **19 rejected logo nomination groups** with full evidence.

## Independent event and fixed-page reviews

All 84 initial core candidates were independently adjudicated against actual
source and reader crops, including expanded contexts where tiny strips were
ambiguous. Initial precision was omission 38/46, duplicate 5/5, head 4/13 and
watermark 1/20. The final review file binds the exact final candidate SHA and
transfers evidence only when the actual crop hashes identify the same inspected
images. Every final core event transferred successfully: **56 reviewed events,
zero unreviewed events**. Removed events/component groups are explicitly listed
in `review-transfer.json`; they were not converted into clean negatives.

| Final core class | Confirmed | False positive | Uncertain review | Precision denominator | Reviewed precision |
| --- | ---: | ---: | ---: | ---: | ---: |
| duplicate | 5 | 0 | 0 | 5 | 100.00% |
| head | 4 | 0 | 0 | 4 | 100.00% |
| missing_occurrence | 38 | 8 | 0 | 46 | 82.61% |
| order | 0 | 0 | 0 | 0 | undefined (no events) |
| watermark | 1 | 0 | 0 | 1 | 100.00% |

The fixed development panel comprises **93 independently inspected reader
pages**. Every core event touching those pages, including low/unknown events,
has an explicit adjudication. There are **0 high false-positive pages / 93**
and **0 high uncertain pages / 93**. These are development panel results,
separate from the candidate census. They do not establish corpus-wide absence
of false positives or held-out performance. Review certainty and detector
confidence are distinct fields.

## Detector changes and evidence

- Source extraction separates running heads and neighboring-column text using
  observed column/word geometry, while retaining raw text and observations.
  Duplicate occurrence boundaries use substantial leading words and verified
  physical source scans, including incomplete repeated openings.
- Shared source aliases require distinct physical scans with matching observed
  printed folios. Bounded context relocation and per-line coverage prevent a
  redundant physical scan from being reported as unique loss. Natural repetition
  does not receive duplicate-scan coverage from an unrelated lexical hit.
- Omission checks use informative context, original-image OCR, uniquely uncovered
  Latin lines and retained-ink comparison. Redundant or visibly retained prose
  remains unknown/inconclusive rather than being asserted as unique loss.
- Roman chapter references and components overlapping reader Roman words are
  excluded from head nominations. Independently registered original body ink
  excludes ordinary letters inside retained prose. The two body-word false
  positives in Cato Minor and Pompeius disappear; four genuine head events remain.
- A logo group requires observed refined or stitched reader glyph support after
  grouping. All weak fragments of a supported group remain attached. W4 retains
  all six components and its crossing stitch; its faint upper fragment was
  independently viewed. All 19 standalone unsupported groups were independently
  inspected as ordinary prose false positives. Rejected nominations remain
  visible in debug JSON, report JSON and HTML with hash-checked crop links.
- Lateral clipping retains individual affected-word regions alongside overview
  crops. Adjacent observations join only on consecutive physical reader pages
  and body ordinals, the same physical source page, and matching registered
  source-column boundaries within the declared reprojection tolerance. C6 now
  matches localized gold, with 20 affected words across Romulus I/68-69. Its
  grouped confidence remains low because the second page has weaker correlation.
- OCR language/DPI/page-segmentation settings bind semantic configuration and
  resume/freeze identity. Completion is recorded only after reporting, final
  input verification and runtime/storage recording succeed.

All D4-D8, C1-C3, W4 and H1/H2 match localized development gold. Duplicates,
W4 and H1/H2 remain high; C3 is high, while C1 and C2 are low. C6 is a localized
low hit. At least one localized high benchmark positive and independently
confirmed event survives in each required core class. The evaluator does not
promote detector confidence.

## Known misses and false positives

The freeze retains all **25** misses without relaxing localization or changing
gold: M1/M3/M4/M6/M7, R3/R4/R5/R6/R7/R9/R10, and
E16/E17/E18/E19/E28/E31/E38/E47/E49/E54/E61/E64/E65.
The initial miss review is retained at
`runs/step92-development-final-2/review-images/benchmark-miss-analysis.json`.
C6 is the one resolved case in the final complete evaluation.

Four endpoint observations (E18/E19/E28/E65) are matched but have oversized
evidence windows that fail strict localization. The other nine endpoint misses
remain OCR/corroboration limitations. E31's embedded OCR joins `mortem` and
`sibi` although the printed/reader spacing is visible; comparison code does
not silently split that raw word. Supplementary gap and rule misses retain
source-correspondence limitations.

Eight omission nominations are independently false positives. Seven are
low/unknown; **`miss_tiberius_gracchus_403_690` remains high**, outside the fixed
panel. Its source v2/403 prose is visibly retained on reader VII/88. The retained
ink score did not reach the declared exclusion threshold; that threshold was
not changed to fit one page. This residual false positive remains visible and
reviewed in the final report and precision denominator.

## Verification and reproducible commands

Focused actual-corpus runs were retained before installing the final producer:
head/body exclusion (366 pages, two benchmark hits, seven passing negatives),
furniture grouping (466 pages, W4/H1/H2 hits, 14 passing negatives), clipping
localization (49 Romulus pages, C6 hit), and seven-unit prose/alias review
(436 pages, 12 hits, three endpoint misses, four passing negatives).
Earlier failures and intermediate results remain in their explicit run folders.

The candidate-review protocol was also checked with actual evidence: a valid
independent review was accepted; a mismatched evidence SHA and an unadjudicated
core event on a claimed reviewed panel page were rejected. A premature final
review-binding attempt was safely rejected while the final manifest was still
incomplete; it wrote no review. Binding/evaluation then passed after finalization.
Its configuration-failure diagnostic remains separate from the error-free run.

```powershell
python tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope development --detectors all --workers 4 --run-dir tmp/pdfs/inner_tomi_audit/runs/step92-development-final-4
python tools/audit_inner_tomi.py evaluate --run-dir tmp/pdfs/inner_tomi_audit/runs/step92-development-final-4 --split development --reviews tmp/pdfs/inner_tomi_audit/runs/step92-development-final-4/reviews.json --freeze
python tools/audit_inner_tomi.py report --run-dir tmp/pdfs/inner_tomi_audit/runs/step92-development-final-4
python -m py_compile tools/audit_inner_tomi.py
git diff --check
```

The first command records the completed run's invocation; preserve that folder
and use a new explicit folder for another detector run. The existing freeze is
immutable. Report regeneration uses saved OCR and does not rescore detectors.
No Python test/lint framework was introduced, and Rust checks were unnecessary
because this change touches the diagnostic Python layer.

Freeze checks verified the current/retained producer, exact semantic settings,
source-index identity, tool versions, split/template/label identities, run
manifest, candidates, evaluation and saved reviewed metrics. Every one of the
**3,121 accepted/rejected report crop references** passed actual SHA and HTML
link checks. Native crops and expanded image contexts were visually inspected.
Browser inventory was empty during verification; browser layout remains
unexercised. The PDF skill's read-only image-verification workflow was applied.

## Final identities and resources

| Artifact | SHA-256 |
| --- | --- |
| current/retained producer | `d9b1d05b6ad7a45777dd67f38cda892949e1a2b614a503e784970457dfbbac6a` |
| run-manifest.json | `c6c6157acb92ae6a140460ba290c99318e8313dce7acbb4cbdb04c38f5ae28f4` |
| candidates.json | `81c9a16ed6f90d7e34a1f071ff974b6991a2e127a4c613d96e462ceb2c3d807d` |
| reviews.json | `10e8bf0ce11503de8cf0cf4c91f1529ff61e356728aecc420c6cd3fc32a4f9d0` |
| evaluation-development.json | `557ffca71869d5c628888533b44052c4d459d4c696c2d123434dd85d77131da3` |
| freeze.json | `c99fd09855268cbddc197513051e1bc1fe37f6c5d073f1e4a778028dc4581650` |

The final producer is the exact tested preview-6 script. Source extraction cache
identity is
`1d08f50d7678daa72c064264c1d1f6221af5d29295b17e32884a21f1bcc7f4b1`;
source index cache identity is
`05da3ab0beda2486d1ebed5c393517067cba0334c5f3536ccf9a7b6534e2d577`.
The index contains 35,324 windows over 68 source units; its vocabulary, IDF and
source-token identities are also saved in the freeze. Installed tool/package
versions and the actual English OCR profile (source 300 DPI/PSM 6, reader
180 DPI/PSM 3, retry 300 DPI/PSM 6 and 11) are recorded in the manifest/freeze.

Final elapsed runtime is **5,515.611 seconds**. Measured run storage before the
final manifest was **139,242,006 bytes**; cache storage was **3,700,958,572 bytes**.
Review/evaluation/report files written afterward are additional run storage.
Snapshots, caches, run reports and freeze are local ignored artifacts; the
script, documentation and durable verification notes are committed.

## Human inspection and next work

Open the final run's `index.html` locally and inspect the grouped W4 fragments,
localized C6 words, Phocion's unique first-line omission, and the known Tiberius
false positive. Inspect the rejected-logo section to confirm that ordinary prose
probes remain accessible. JSON/CSV, `coverage.json`, `errors.json`,
`evaluation-development.json`, `reviews.json` and `freeze.json` supply the
machine-readable evidence and identities.

Pause after this step's commit/push. The next work is the first controlled
validation run under this freeze, followed by final inspection in 10.2.
Post-validation changes must be recorded as revisions rather than fresh
independent validation. Add the main README section only after 10.2 is pushed.
