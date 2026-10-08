# Inner-tomi development calibration checkpoint

Created on 2026-10-07 at approximately 12:55 America/New_York. This checkpoint
records completed step 9.2 development calibration. The final verification,
metrics, identities and limitations are in
`2026_10_07_21_56_INNER_AUDIT_DEVELOPMENT_FREEZE_VERIFICATION.md`.
The earlier `2026_10_07_08_52_INNER_AUDIT_CALIBRATION_CHECKPOINT.md` is retained
as the historical record of the morning's interrupted experiments.

## Authorization and next boundary

The user authorized implementation and pushing on `inner-tomi-audit-phase-1`.
Commit/push this completed step, then pause. The preceding completed step was
9.1, commit `d2526ef`. Step 10.1 validation detection has not started. The main
README addition remains queued for after step 10.2 is committed and pushed.
No subagents were authorized. Production Latin, transcription caches, PDFs,
LaTeX, Rust, repair policy and snapshot inputs remain read-only.

## Complete development runs

All three full runs used all 34 development units, all 1,338 actual body pages,
all requested detectors, and four workers. Each completed without processing
errors and retains exact producing bytes and native process records.

| Run under `tmp/pdfs/inner_tomi_audit/runs/` | Producer SHA | Hits / misses / passing negatives | Core events | Elapsed seconds |
| --- | --- | --- | ---: | ---: |
| `step92-development-final-2` | `70fe510c00c5825cef5ee2f536e2d58e94e889506e937da3efb518a779fb0bc2` | 37 / 26 / 118 | 84 | 20,316.230 |
| `step92-development-final-3` | `2d9c11fc2fea697e37bc6f56d1af68a64ee82bb3f6e5aa2c1f9c6d69118e2c4b` | 37 / 26 / 118 | 77 | 6,764.222 |
| `step92-development-final-4` | `d9b1d05b6ad7a45777dd67f38cda892949e1a2b614a503e784970457dfbbac6a` | 38 / 25 / 118 | 56 | 5,515.611 |

Every run has zero negative violations. Final burden is 4.185 core events per
100 body pages. The final manifest binds actual coverage and was marked complete
only after reporting, final input hashing and runtime/storage recording.
There are 661 candidates and 19 rejected logo groups with accessible evidence.
The final evaluation and freeze bind all 56 independent candidate reviews and
all 93 fixed panel page reviews: no unreviewed core event, no high false-positive
or uncertain panel page. Omission precision is 38/46; duplicate 5/5, head 4/4,
watermark 1/1. There are no order events, so that precision is undefined.

Eight reviewed omission false positives remain. One is high:
`miss_tiberius_gracchus_403_690`, whose source prose is visibly retained on
reader VII/88, outside the fixed panel. Do not equate the panel result with
corpus-wide absence of false positives. Final known misses are five gap cases,
seven rule cases and 13 endpoint cases; strict benchmark localization remains
unchanged. Four endpoint misses are matched observations with overbroad windows.
C6 is now a localized low-confidence hit, with 20 affected-word regions across
Romulus I/68-69. C1/C2 are localized low hits, C3 high, D4-D8 high, W4 high and
H1/H2 high. The evaluator never promotes confidence.

## Isolated revisions and installation

Ignored immutable previews remain under
`runs/step92-development-final-2/revision-preview*/`. Head/body exclusion
preview 4 completed its six-unit/366-page actual test: H1/H2 hit, seven passing
negatives and four genuine head events. Furniture preview 5 completed eight
units/466 pages: W4/H1/H2 hit and 14 passing negatives. It preserves W4's six
components, weak upper fragment and crossing stitch. Unsupported nominations
remain visible in JSON/debug/HTML rather than being discarded without evidence.
Clipping preview 6 completed all 49 Romulus pages, preserving strict gold and
thresholds while localizing C6 and grouping the same physical source column
across consecutive physical reader pages/body ordinals.

The guarded helper `runs/step92-development-handoff.py` completed the initial
run evaluation and installed preview 3 before launching full run 3. The old
preview-5 helper was deliberately stopped before installation; its state under
`runs/step92-final-revision-handoff/` records
`stopped_for_clipping_localization_verification`. The replacement helper,
`runs/step92-final-revision-handoff-6.py`, verified completed furniture/clipping
tests, exact immutable preview/live hashes, authorized branch and complete full
run 3. It preserved evaluation, installed exactly preview 6 and launched full
run 4 as hidden native PID 28948 at 00:22:25 UTC on October 8. That process has
finished. The helpers performed no freeze, commit or push and cannot resume an
assistant turn. No audit runner or installation helper remains active.

The final live script and final run's retained `producing-script.py` have the
same preview-6 SHA. The assistant subsequently bound final reviews, evaluated,
created the development freeze and regenerated the report. A review attempt
made before final manifest completion was rejected safely; no review was written
until the complete manifest passed. Its request-failure diagnostic remains
separate from the completed audit's empty processing-error list.

## Independent evidence and retained experiments

The initial `event-review-draft.json` contains 74 event reviews and 13 component
review groups, binding the independently inspected image evidence. Together they
adjudicate all 84 initial grouped core events. `reviews-census84.json` and the
full-census evaluation are retained; the earlier selected-67 review/evaluation
and first no-review evaluation remain separate. `runs/step92-bind-reviews.py`
transfers a review only to actual identical crop evidence; every final event
transferred, and removed components are listed in `review-transfer.json`.

All 93 full fixed-panel reader pages were inspected in 47 contact sheets under
`step92-development-panel-images/`, whose manifest retains image hashes.
Expanded omission, head and watermark contexts and duplicate source/reader
pairs are under the initial run's `review-images/`. All 19 unsupported watermark
groups were inspected as ordinary prose, and both body-word head false positives
were inspected in native expanded contexts. Cato Minor's letter is within
`pariturum`; Pompeius's components are within `natus undesexaginta`. Independent
source registration corroborates those retained body words. No gold was edited.

Actual review-protocol checks under `step92-duplicate-refinement-4` accepted
`step92-review-protocol.json` and rejected
`step92-review-invalid-evidence.json` and
`step92-review-unadjudicated-page.json`. The latter tried to declare VI/49
reviewed while its core duplicate event had no adjudication.

Earlier actual-corpus runs remain under their explicit folders, including
`step92-prose-refinement-7` through `-10`,
`step92-watermark-refinement-2` and `-3`,
`step92-duplicate-refinement-4`, `step92-head-profile-preview-1`,
`step92-head-alias-preview-3`, `step92-head-body-preview-1`,
`step92-furniture-group-preview-1`, and
`step92-clipping-localization-preview-1`. The morning note records earlier
stopped experiments. The `prose-refinement-9` finalization failure is retained
and is not used as a complete run or a freeze. Failure diagnostics and partial
cache entries were never treated as clean processing.

Source/body/alias diagnostic helpers and their output remain under `runs/`,
including `step92-head-body-debug.py`, `step92-omission-local-pixel-search.json`,
`step92-phocion-context-debug.json`, `step92-review-contexts.py`, and
`step92-head-contexts.py`. The initial 26-case miss review is
`step92-development-final-2/review-images/benchmark-miss-analysis.json`.
All 25 surviving misses are explicitly retained in the final freeze.

## Recovery identities and verification

The final run folder contains `run-manifest.json`, `candidates.json`,
`reviews.json`, `evaluation-development.json`, `freeze.json`, `index.html`,
CSV/JSON report data, coverage/errors, native logs and exact producing bytes.
Freeze SHA is
`c99fd09855268cbddc197513051e1bc1fe37f6c5d073f1e4a778028dc4581650`.
The complete identities are in the final verification note.

Current source cache entries are
`cache/source_regions/1d08f50d7678daa72c064264c1d1f6221af5d29295b17e32884a21f1bcc7f4b1.json`
and
`cache/source_index/05da3ab0beda2486d1ebed5c393517067cba0334c5f3536ccf9a7b6534e2d577.json`.
All 68 original-source units are fitted; validation reader output is absent.
All 21 live/captured baseline input hashes and unchanged tracked/working spec
identities passed verification. Freeze checks verified settings, versions,
source-index/profile identities and actual run/candidate/evaluation hashes.
All 3,121 accepted/rejected report crop references passed actual SHA/link checks.

The PDF skill's read-only image-verification workflow was applied. Native PNGs
and expanded contexts were viewed; browser inventory was empty, so browser layout
remains unexercised. Syntax/diff checks accompany this diagnostic-only change;
no Python test/lint framework or Rust checks were required. Escalated native
PowerShell/Python execution was needed in this environment. Generated work
artifacts remain local and ignored; durable notes, script and documentation
are retained in Git.

On continuation, preserve this development freeze and start step 10.1 under
its exact settings in a new validation run folder. Do not rerun development
or score validation before deliberately resuming that next step. The current
instruction is to pause after this step's commit/push.
