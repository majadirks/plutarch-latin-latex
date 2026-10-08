# Inner-tomi audit calibration checkpoint

Saved on 2026-10-07 at approximately 08:52 America/New_York (12:52 UTC).

## Resume contract

The user requested a pause after saving this checkpoint because the session is close to its usage limit. **Step 9.2 is unfinished.** Do not treat this checkpoint as a completed calibration or a development freeze. The working implementation is saved in `tools/audit_inner_tomi.py`, with uncommitted changes. This checkpoint is also uncommitted. No step 9.2 commit or push has been made.

The active branch is `inner-tomi-audit-phase-1`. The last completed, committed and pushed step is 9.1, commit `d2526ef` (`Produce audit evidence reports and finish development gold annotations`). Earlier steps 5.2 through 8.2 were committed and pushed individually. The user has authorized pushing on this branch; no new push permission is needed.

When resumed, finish 9.2, verify it, commit and push it, then honor the user's instruction to pause after that current step unless the user changes that instruction. Steps 10.1 and 10.2 have not started. The additional main README section must be added **after 10.2 is committed and pushed**; it must explain detected issues, execution, reports and methodologies. Do not add that section early.

## Scope and invariants

- Plan: `llm_artifacts/2026_10_06_16_05_INNER_TOMI_PYTHON_AUDIT_IMPLEMENTATION_PLAN.md`, especially 9.2.
- Read the repository AGENTS instructions. Preserve production Latin, transcription caches, PDFs, LaTeX, repair policy and captured snapshot inputs exactly. Only diagnostic implementation, tool documentation, independent annotations and dated notes are in scope.
- No unit-test framework, Python lint framework or Rust checks for this diagnostic-only change. Use actual corpus verification, `python -m py_compile tools/audit_inner_tomi.py`, and `git diff --check`.
- No subagents are authorized. Python commands require escalated execution in this Windows environment; the available interpreter is WindowsApps Python 3.13.14. Installed scientific/image dependencies and Tesseract/Poppler are recorded in prior verification notes.
- Diagnostic root: `tmp/pdfs/inner_tomi_audit`; baseline: `baseline_2026_10_06`. The corpus has 68 units / 2678 body pages, split into 34 development and 34 validation units.
- Original-source extraction/index fitting across all 68 units is permitted. Validation reader OCR, alignment, scoring and image detection remain prohibited until a complete development freeze exists. No validation detector run has occurred.
- Gold is independent of detector predictions. All 187 development cases were independently reviewed in 9.1. Validation annotations are unchanged from that step, with 134 pending of 138 cases. Do not relabel gold to make a detector pass.
- Labels under `tools/inner-audit-labels.json` match work `spec/labels.json`. The 9.1 note documents all independent image reviews and annotation corrections: `llm_artifacts/2026_10_07_07_18_REPORTING_AND_ANNOTATION_VERIFICATION.md`.

## Current implementation changes

The uncommitted script contains these changes; inspect the diff before proceeding:

1. Bounded alignment allows word joins/splits only at an observed hyphenated line/page boundary or an explicit join hypothesis. Source and reader token metadata survive comparison views. An indexed exact-token matcher preserves run ordering and scores while avoiding comparisons of unequal tokens. Equivalence was checked on 204 actual corpus window pairs; evidence is in `runs/step92-exact-alignment-equivalence.json`.
2. Source duplicate nominations retain exact matched intervals. Independently observed original printed folios corroborate token-pair scan equivalence. Conflicting/uncertain pairs are retained as limitations, never aliased as coverage. Canonical coverage maps only verified matched token intervals; partly aliased anchors receive separate handling. Missing prose cannot receive coverage from an unrelated consumed occurrence.
3. Repeated-visit nomination checks the shared passage against the repeat floor, rather than requiring it to occupy half of a reading run that also contains a long continuing tail. Development D4/D5/D6 nominations recovered in a targeted run.
4. Duplicate contribution boundaries are refined against original source-page edge text and local reader alignment. The latest unverified refinement records first-line coverage, detects an incomplete second opening from that coverage, and expands a matched start to the first reader token on the same observed line. Its local coverage threshold is currently 0.60. Review its behavior before accepting it: D5 previously had correct boxes but the wrong incomplete-opening flag; D6 previously started too far right on both opening lines.
5. Omissions require informative contexts, a placement margin, source-raster agreement and absence in reader regional PSM 6/11 retries for high confidence. Oversized local matrices become explicit alignment limitations. Source regions now receive 0.25 line-height vertical padding to include actual glyph extent. This padding and source-header corrections are not yet evaluated in a finished run.
6. Original source extraction infers the Latin-column border from Latin word observations and retains neighboring/uncertain words separately. Greek quotations inside the Latin column remain expected text. Cache identity includes the extraction helper code and actual extracted source data. The latest helper separates right-column uppercase running heads near the top, and excludes neighboring words left of the inferred border even when their embedded OCR resembles Latin. It retains the adapter name `source-latin-column-consensus-v2`, but helper/data hashes distinguish revisions. The latest all-source extraction was interrupted before completion.
7. Watermark event nomination requires source-seam/footer correspondence or supported distinctive logo fragments; every rejected coarse/refined peak is retained in a debug JSON. The source seam allowance is now 128 tokens, accommodating coarse passage anchors at a two-window distance. A 32-token allowance wrongly rejected W4. The latest optimization skips a 300-DPI retry only when the coarse group lacks two observed glyphs and source-seam correspondence; all coarse evidence is retained. Shape-cache metadata excludes the two event-predicate settings, since they do not alter template matching. This optimization is not yet verified in a finished run.
8. Clipping reference words can begin at source y=0.06 rather than y=0.12, admitting the legitimate `educati` near y=0.095 in C6. No new clipping run has verified this change.
9. `evaluate --reviews FILE` accepts independent event adjudications bound to actual candidate-file bytes and fixed panel page reviews. It reports per-class confirmed/false-positive/uncertain counts and precision denominators separately from panel rates. Every core event touching a reviewed page requires a verdict; unreviewed candidates cannot make a panel page negative. The JSON protocol needs actual-corpus verification and documentation.
10. Freeze requires complete fixed development-panel review, actual candidate adjudications, the current producing-script hash, complete all-development/all-detector coverage, reviewed gold/negatives, and a strong localized high positive for each of missing occurrence, duplicate, watermark and head. Frozen JSON records development metrics in addition to settings/templates/index/versions/spec/input/run identities. Known misses remain explicit.
11. Run records include elapsed time and measured storage. Cache storage measurement tolerates transient renames. Render cache contention waits up to 180 seconds for a complete valid entry. Failed runs record explicit failure JSON. Coverage reports distinguish unreadable OCR from an attempted page; HTML links nomination/alignment diagnostics and shows OCR status. The added diagnostics glob includes `alignment_debug.json`; check the actual boundary filename before relying on its link.

## Finished calibration experiments

Paths below are relative to the diagnostic `runs/` directory. None is a final freeze.

- `step92-prose-refinement-2`: completed earlier source profile; 24 candidates. Evaluation: 5 hits, 9 misses, 4 clean requested negatives. It accidentally requested `comparatio_alcibiadis_et_coriolani` instead of Coriolanus, so C2 was not requested. Preserved for comparison.
- `step92-prose-refinement-3`: completed seven-unit run with the first column-consensus revision. 25 candidates; 6 hits, 9 misses, 4 clean requested negatives. Hits: D8, C1, E02, E44, E50, E51. Misses: D4-D7, C2, C3, E16, E47, E49. D4-D7 had verified duplicate blocks and repeated reading runs, but the half-run overlap heuristic suppressed nominations. C2 was overbroad because source furniture/neighbor words entered its region. C3 is high with correct literal source line and reader seam, but its embedded source word box covers less than 80% of the independently measured glyph region; the latest padding addresses this geometric mismatch.
- `step92-duplicate-refinement-4`: completed Agesilaus/Alexander/Phocion run after shared-passage nomination correction, 16 candidates. Evaluation: D4 and E44 hit; D5/D6/C3/E47/E49 miss; two clean requested negatives. D4 boundaries match independent gold. D5 boundaries match gold but `incomplete_second_opening` was false. D6 boundaries started at a word later in the correct opening line; latest refinement addresses both failures. Gold unchanged.
- `step92-watermark-refinement-1`: completed Aemilius/Coriolanus run, two low Coriolanus ordinary-curve candidates and no W4. Evaluation: W4 miss and two repaired-logo negatives clean. The strict 32-token source-seam allowance rejected W4. This failure was explicitly acknowledged to the user; do not claim this revision preserved the positive.
- `step92-prose-refinement-1` failed on concurrent raster rendering; failure evidence retained.
- `step92-prose-refinement-5` failed before detection because the then-running final experiment owned `cache/source_regions/run.lock`; its failure JSON is retained. Do not score it or resume it as a complete run.
- The initial full-development experiment ran code from the 9.1 commit. It was interrupted before writing a complete manifest/candidate file. Its partial image diagnostics are useful as exploratory evidence, not full-run metrics.

Some newly nominated high omissions may be real losses of source opening/closing lines; do not automatically classify unlabelled candidates as false positives. For example source v2/301 contains the opening `XXV. Quum Micio ...`, while reader VI/233 visibly joins preceding prose to `tum manu ...`. This is preliminary image evidence, not a finished adjudication. Read the full source/reader crops for every reviewed verdict.

## Fixed development panel

All **93** fixed development panel reader pages were directly viewed in 47 two-page contact sheets at `runs/step92-development-panel-images/panel_01.png` through `panel_47.png`. `panel-manifest.json` records all 93 as `reader_image_reviewed`, with actual render hashes and a note that candidate adjudication remains separate.

The review included sparse complete conclusions, generated titles, ordinary prose, edition reference letters, quoted/dialogue text, verses and genuine chapter spacing. Reader VII/29 visibly contains the known Sigma head remnant and vertical rule; its repeated prose contributes D8. Reader VI/103 contains an intentional Greek quotation inside the Latin prose. These observations must not become generic negative assumptions.

**No per-page false-positive rate has been computed.** A completed final run must supply every core event touching those pages for source/reader evidence adjudication. The remaining 107 fixed panel pages belong to validation and have not been used for development scoring.

## Processes stopped for the pause

On the user's explicit pause request, the following identified Python process trees were stopped on 2026-10-07 around 12:51 UTC. A process inventory afterward showed no running Python audit invocation:

- PID 32708, session 30691: `step92-development-initial`, development/all/34 units, old committed implementation. Last observed processing Agis after Aemilius and Agesilaus. Partial run directory exists; no complete run manifest.
- PID 32412, session 24886: `step92-watermark-refinement-2`, development/watermark/Aemilius+Coriolanus, 128-token seam allowance before the latest coarse-retry optimization. Last observed through Coriolanus page 174. Partial directory exists; no complete evaluation. W4 recovery is **not yet established**.
- PID 13412, session 36680: `step92-development-final-1`, development/all/34 units, source extraction still in progress. The run directory had not yet been created. Its producing-script hash predates the latest duplicate-boundary change, so it could not freeze the current script even if completed.

Zero-byte stale `run.lock` files remain in `cache/source_regions/`, `runs/step92-development-initial/` and `runs/step92-watermark-refinement-2/`. They were not removed. Before resuming, confirm no audit invocation owns them, then remove only those proven stale locks using native PowerShell `Remove-Item -LiteralPath`. Do not recursively delete a cache or run folder. Completed hash-validated cache entries remain reusable; interrupted partial entries are not proof of completion.

## Next work

1. Inspect the uncommitted diff and this checkpoint. Verify source/header and duplicate-edge refinements on development evidence using a fresh run folder. The failed `--final-development` flag is not a CLI option: final development is inferred from `--scope development --detectors all`.
2. Use a new complete run after the script settles, e.g.:

   ```powershell
   python tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope development --detectors all --workers 4 --run-dir tmp/pdfs/inner_tomi_audit/runs/step92-development-final-2
   python tools/audit_inner_tomi.py evaluate --run-dir tmp/pdfs/inner_tomi_audit/runs/step92-development-final-2 --split development
   ```

3. Inspect every missed/overbroad development case in mapping/extraction/OCR/retrieval/alignment order. Verify D4-D8 contribution boundaries and incomplete openings; C1-C3 localized source/seams; W4 high; H1/H2 high; no high candidate at a reviewed negative. Preserve all successive experiments and known limitations. Supplemental rule candidates previously had empty source regions and missed source-correspondence localization; do not hide these misses.
4. Review final-run candidates touching all 93 development panel pages against actual source/reader images. Supply `--reviews` JSON with `schema_version`, `baseline_id`, `candidates_sha256`, `sample_description`, `events` and `pages`. Events need `event_id`, `verdict` (`confirmed`, `false_positive`, `uncertain`), `reason`, `evidence`. Pages need `page_id` (`tomus:physical_page`), `status: reviewed`, `reason`, `evidence`. Report real per-class denominators, candidate burden and panel high false-positive/uncertain counts. Never infer candidate precision directly from gold hits.
5. Verify the review protocol with actual saved evidence and document it in `tools/README-inner-audit.md`. Check current script/input/spec identities, explicit processing errors, and complete requested actual development coverage. A frozen run must be produced by the exact current script; edits require a fresh run. Finish calibration before invoking `evaluate ... --reviews ... --freeze`.
6. Save a dated **completed** 9.2 verification note after these requirements pass. Run `py_compile` and `git diff --check`; commit only the diagnostic changes and note; push to the authorized branch. Pause afterward. Do not enter validation before the freeze or before the user's next resume instruction.

## Checkpoint verification

`python -m py_compile tools/audit_inner_tomi.py` passed after the latest saved changes. `git diff --check` passed. Production files and benchmark labels were not edited in 9.2. These checks establish a readable, syntactically valid checkpoint, **not completion or detector accuracy**.
