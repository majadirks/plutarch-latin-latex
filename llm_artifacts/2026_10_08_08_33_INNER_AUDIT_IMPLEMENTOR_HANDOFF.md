# Implementor handoff: finish held-out validation and final audit delivery

## Authorized work and stopping point

The user requested the remainder of
`llm_artifacts/2026_10_06_16_05_INNER_TOMI_PYTHON_AUDIT_IMPLEMENTATION_PLAN.md`,
with a commit and push after each step. Pushes on `inner-tomi-audit-phase-1`
are explicitly authorized. The latest user instruction is to save a good checkpoint immediately because
of the weekly usage limit, and write this explicit handoff, including remaining
10.1 work. **10.1 is not complete at this checkpoint.** This checkpoint commit
is not a completed 10.1 commit. Implementation is paused for handoff; the already
launched read-only diagnostic process remains running.

When the user resumes work with the next implementor, finish 10.1 and commit/push
that completed step. Honor the standing request to pause after 10.1 unless the
new instruction authorizes continuing through 10.2. The rest of the plan and
pushes are already authorized; do not ask again for routine implementation or
branch-push approval. Complete10.2 and commit/push it when resumed for that work,
then add the requested root README section and commit/push that documentation.
No subagents are authorized.

Use the existing workspace:
`C:\Users\majad\coding\Plutarch\plutarch-latin-latex`.
The shell is PowerShell. The remote is
`https://github.com/majadirks/plutarch-latin-latex.git`.
Read AGENTS.md and the plan's 10.1/10.2 requirements. Preserve all existing
plans and notes. Source Latin, transcription caches, PDFs/TEX, repair policy,
source trace and snapshot inputs are read-only. No new test framework or Rust
build is required for this script-only task. Do not add code comments citing
plan steps.

## Exact frozen configuration and local evidence

The work root is `tmp/pdfs/inner_tomi_audit`; baseline ID is
`baseline_2026_10_06`. Use the existing captured baseline and working specs.
Do not recapture, replace working labels, reinstall dependencies unnecessarily,
change OCR language or overwrite first-validation evidence. The diagnostic work
tree is ignored by Git; another machine needs its existing snapshots, specs,
cache and runs copied intact. Git alone does not deliver those large artifacts.

Current producer SHA-256:
`748b8684b23d74c09da84f3c7a9170eaf185ca008f3b7522519ee40b13eb6227`.
Working/tracked labels SHA-256:
`a96a58c6bd924704b7b4e1686cad91851a6f8503880ba182fa836e7354c747c9`.
Chosen freeze:
`runs/step101-development-revised-1/freeze.json`, SHA-256
`77e6d6be418b169b56ff5b52432decd70d6450971cfeb06bf7bc472c1d166fda`.
Its origin contains `run-manifest.json`, `candidates.json`, `reviews.json`,
`evaluation-development.json`, `producing-script.py` and `index.html`.
The revised validation origin is `runs/step101-validation-revised-1/`.
At the saved checkpoint (2026-10-08 approximately12:36 UTC), validation is
still running as hidden native PID 33468, started12:10:46 UTC. Its actual
`process-launch.json` records arguments and start time. Seventeen unit-completion
markers are logged, and Crassus is being processed; stderr is empty. No complete
validation manifest/evaluation is claimed. Read current files rather than
assuming this PID remains alive or the run is still incomplete.

The freeze binds exact producer bytes, whole-spec hashes, settings, source-only
index, versions and immutable development manifest/candidates/evaluation.
Formatting, checkout newline conversion or changing a comment in the Python
file can invalidate it. Check hashes before running. If a checkout changed only
newlines, use the retained exact `producing-script.py` with verified frozen SHA
for the diagnostic invocation; do not weaken identity checks. Semantic changes
require new complete development/validation calibration and must be disclosed.

The original 9.2 freeze and `runs/step101-validation-first/` are permanent
historical evidence. The first run's independently scored gold has 7 hits,
46 misses,82 passing negatives; D10/D11/D12/W5/W6 were mandatory failures.
Subsequent results are post-validation regression, not a fresh independent
validation. The independent annotation seal is
`664c8ec4303adf05e095b5236570ae0cf9afae738adbf73e1931279e022b94a2`.
Post-validation D11 printed-word annotation correction is explicitly preserved.
All 68 actual terminal passages are visually complete; algorithmic endpoint
localization misses do not establish terminal truncation or interior completeness.

## Prerequisites and initial checks

Run from the repository root. `python` currently resolves through
`C:\Users\majad\AppData\Local\Microsoft\WindowsApps\python.exe` and runs
Python 3.13.14. Use existing Poppler/Tesseract on PATH. The actual frozen OCR
language is `eng`; installed `lat` is not part of the chosen frozen profile.
Changing languages requires a new matching calibration and freeze.

```powershell
git status --short --branch
python tools/audit_inner_tomi.py check
Get-FileHash tools/audit_inner_tomi.py -Algorithm SHA256
Get-FileHash tmp/pdfs/inner_tomi_audit/spec/labels.json -Algorithm SHA256
Get-FileHash tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1/freeze.json -Algorithm SHA256
```

Versions frozen by10.1: Python 3.13.14; NumPy 2.4.3; Pillow 12.1.1;
opencv-python-headless4.13.0.92; scikit-learn1.9.0; RapidFuzz 3.14.3;
PyMuPDF 1.27.2; Tesseract 5.5.0.20241111; Poppler 25.12.0.
Use manifest values in the completion note, not inferred installed versions.
Do not commit unrelated user changes. Verify the branch remains the authorized
branch and inspect any divergence before pushing.

## Finish 10.1 before 10.2

The revised development run is genuinely complete: 34 units/1338 body pages,
all detectors/workers 4,3572.794345 seconds,698 total candidates/56 core,
38 hits/25 documented exploratory or endpoint misses/118 passing negatives,
zero processing errors/pending annotations/negative violations. All 93 fixed
panel pages and all 56 core events are reviewed. Actual source/output regions
and crop hashes match the independently reviewed9.2 evidence; panel high FP
and uncertain counts are zero. Strong required core positives remain high.
Its regenerated HTML has 8457 resolving local references and3772 distinct
hash-checked candidate crop files. Browser-rendered layout was not inspected.

Required development positives D4-D8/C1/C2/C3/W4/H1/H2 are localized; S1/C6
are localized. First-validation D9/C5 pass, but D10/D11/D12/W5/W6 failed.
General revisions address boundary short words/incomplete openings, source
footer scan scale and raster context around clipping words. Focused actual
corpus replays pass all D4-D12, W5/W6 and S2. These replays support the changes;
they do not replace a complete revised validation run. No source data changed.
All 187 development annotation objects are unchanged, and D11 is the sole
post-independent-seal annotation correction. The original seal is retained.

1. Inspect the already running validation process and its actual output:

```powershell
Get-Process -Id 33468 -ErrorAction SilentlyContinue
Get-Content tmp/pdfs/inner_tomi_audit/runs/step101-validation-revised-1/process-stdout.log -Tail 20
Get-Content tmp/pdfs/inner_tomi_audit/runs/step101-validation-revised-1/process-stderr.log -Tail 20
```

Read `run-manifest.json` when present. Require `complete:true`,1340 actual
completed/requested pages,34 units, empty processing errors and matching
freeze/config/producer identities. It may finish while this task is paused.
Do not launch a duplicate invocation into that directory. If it stopped
incomplete, diagnose stderr/`failure-*.json` first. After verifying no owner
process and resolving any specific stale lock, resume the same identity:

```powershell
python -u tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope validation --detectors all --freeze tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1/freeze.json --lang eng --workers 4 --run-dir tmp/pdfs/inner_tomi_audit/runs/step101-validation-revised-1 --resume
```

If semantic changes are necessary, preserve this run and use new complete
DEV/VAL runs and a new development freeze. Do not overwrite the immutable
first validation or the chosen frozen development evaluation.

2. Transfer unchanged first-validation review evidence into the completed
revised validation directory:

```powershell
python tmp/pdfs/inner_tomi_audit/runs/step101-validation-gold-preparation/transfer-revised-reviews.py step101-validation-first first-independent-reviews.json step101-validation-revised-1
```

Read `review-transfer.json` and `reviews-draft.json`. The helper compares actual
source/output regions, crop hashes, duplicate contribution records and
incomplete-opening flags. Pending events require explicit new image review.
All 36 first-run core events already have independent verdicts:31 omissions
(21 confirmed/10 FP),four semantic duplicates confirmed,one head FP. All 107
panel pages and touching candidates are reviewed, with zero high FP/uncertain
panel pages. The head FP is the body-word-final `t` in `aiunt.` on VIII/131,
outside the fixed panel; preserve this limitation rather than forcing low
confidence or silently discarding the event.

For changed duplicate boundaries, inspect actual new contribution start/end
boxes against independently measured gold, including native first-word `a`
in Aratus D11 and D12's incomplete `rimum` opening. Reuse confirmed semantic
passage observations only where their evidence matches; evaluate actual new
boundaries with the official matcher. Independent gold/full-page/native
contribution evidence is in `step101-validation-gold-overviews/`,
`step101-validation-gold-final/` and `step101-validation-gold-preparation/`.
`duplicate-boundary-replay.json` localizes all 9 cases but is not the full run.

For new W5/W6 events, actual gray glyph crops were viewed in
`runs/step101-reader-logo-replay/independent-reviews.json`. Transfer those
verdicts only after comparing final source/output regions and every actual
crop hash to `replay-result.json`; otherwise view new actual crops. Both
replays are localized low-confidence findings, not high-confidence assurances.
Do not simply install that two-event file as whole-run reviews; append valid
verdicts/evidence to the bound draft. `step101-split-line-replay-2/` similarly
retains S2 complementary VII/244-245 glyph evidence at low confidence.

The full first-validation expanded review has 32 events (31 omission/one head)
with original source columns and three adjacent full reader pages each in
`step101-validation-expanded-review/review-inventory.json`. Four duplicates
were independently inspected during gold preparation. All 107 fixed panel
images are in `step101-validation-panel-images/panel-manifest.json`; the
immutable93-page development evidence is preserved through revised reviews.
New or changed core events touching a panel page must be adjudicated before
that page can count in the final denominator.

3. Once every required event/panel review is complete, save final
`reviews.json`, evaluate actual revised candidates and regenerate the report:

```powershell
Copy-Item -LiteralPath tmp/pdfs/inner_tomi_audit/runs/step101-validation-revised-1/reviews-draft.json -Destination tmp/pdfs/inner_tomi_audit/runs/step101-validation-revised-1/reviews.json
python tools/audit_inner_tomi.py evaluate --run-dir tmp/pdfs/inner_tomi_audit/runs/step101-validation-revised-1 --split validation --reviews tmp/pdfs/inner_tomi_audit/runs/step101-validation-revised-1/reviews.json
python tools/audit_inner_tomi.py report --run-dir tmp/pdfs/inner_tomi_audit/runs/step101-validation-revised-1
python tmp/pdfs/inner_tomi_audit/runs/step101-validation-gold-preparation/verify-final-regression.py
```

Read the verification helper before running it. It verifies both actual
revised halves, not manual replay candidates. It requires mandatory localized
core hits, strong development confidence, S1/S2/C4/C6 hits, zero negative
violations/pending annotations, exact2678-page/68-unit coverage, all 200 fully
reviewed panel pages (<=2 high FP and no unresolved high uncertain pages),
core burden<=5 per 100, unchanged source-only index/tool versions and all 42
live/captured input hashes. Its output is
`runs/step101-validation-gold-preparation/final-regression-verification.json`.
If a check fails, investigate concrete evidence and fix generally; do not
weaken assertions, localization or the freeze to claim completion.

4. Complete the candidate-selected sample and ending/limitation reporting.
First-run `previously-unlabeled-sample.json` has 31 eligible events after
excluding known D9-D12/C5 targets:20 confirmed/11 FP/zero uncertain, with
nine overlapping low-confidence omission events. This covers all available
first-run new events rather than inventing50. Bind the revised eligible cohort
and its actual new events/review verdicts, selecting up to50 previously
unlabeled core events and up to10 low events per type. Keep its precision and
uncertain counts separate from the200-page fixed-panel FP rate. All 68 actual
terminal passages were independently viewed as complete; record every
algorithmic endpoint miss as localization/OCR limitation rather than claiming
truncation. Supplemental M/R/F/C layout misses may remain explicitly exploratory;
C7 extent remains unconfirmed. Mandatory core misses cannot be accepted.
Check revised HTML crop links and every referenced crop hash, including rejected
probes. Retain browser QA limitations if no browser is available.

5. Save a new dated10.1 calibration note with actual first-versus-revised
metrics, hashes, runtimes/storage, confidence, candidate burden, both panel
halves, sample selection, layout/endpoint limitations and unchanged inputs.
A substantial draft is retained as
`runs/step101-validation-gold-preparation/calibration-note-draft.md`; replace
its pending sections with actual results before publishing it to
`llm_artifacts/`. The saved earlier checkpoint and annotation notes are
historical artifacts and must be retained. Rewrite their current continuation
sentences in place if updating them. Compile the Python script and run
`git diff --check`; commit and push completed 10.1. The current checkpoint
commit does not satisfy that completion. Respect the user's next instruction
about the standing pause after 10.1 before beginning10.2.

## Full final run

Use explicit new folder `runs/step102-final-all-1/`. If it already exists,
inspect its manifest and active process before choosing a new suffix or
resuming the exact same identity. Do not run two invocations against one folder.
The `run.lock` is an exclusive marker, not a PID ledger. Check actual Python
process command lines and retained launch metadata before removing a specific
stale lock; never delete a diagnostic directory to clear a lock.

```powershell
python -u tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope all --detectors all --freeze tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1/freeze.json --lang eng --workers 4 --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1
```

For a long invocation use a hidden native process with separate stdout/stderr
logs and retain its exact arguments, PID, start time and producing-script copy.
Poll with short reads; give meaningful progress updates. A process PID is not
an exec session ID. Do not use arbitrary elapsed-time cutoffs. Wait for actual
completion or diagnose a concrete failure.

Inspect `run-manifest.json`, `coverage.json` and `errors.json`. Require complete
requested/finished sets of2678 body pages and68 units, no processing errors,
no pending detectors and matching frozen index/version/settings identities.
Keep observational/OCR/registration limitations separate from tool failures.
The runtime field is `record['runtime']`, including `elapsed_seconds`,
`run_storage_bytes_before_final_manifest` and `cache_storage_bytes`.

## Transfer independent reviews, then evaluate and regenerate HTML

Actual helpers retained under
`runs/step101-validation-gold-preparation/` are one-off corpus verification
scripts, not a test framework. Read a helper before using it. The merge helper
requires both revised 10.1 `reviews.json` files and transfers verdicts only when
actual source/output regions, duplicate contribution/opening records and crop hashes are identical.

```powershell
python tmp/pdfs/inner_tomi_audit/runs/step101-validation-gold-preparation/merge-final-reviews.py step102-final-all-1
```

This creates `reviews-draft.json` and `review-transfer.json` in the final run.
Inspect `pending_events`; if any remain, view their actual source/reader images
and adjoining pages, supply explicit confirmed/false_positive/uncertain
verdicts with reasons and diagnostic file/SHA evidence. Never infer a negative
from an unreviewed event. All 200 fixed panel pages must have actual image
review evidence and every touching core event must be adjudicated. Their full
page imagery is unchanged if input hashes are unchanged, but candidate evidence
must bind the final `candidates_sha256`. Once the draft is complete:

```powershell
Copy-Item -LiteralPath tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/reviews-draft.json -Destination tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/reviews.json
python tools/audit_inner_tomi.py evaluate --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1 --split all --reviews tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/reviews.json
python tools/audit_inner_tomi.py report --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1
```

Do not add `--freeze` to final-all evaluation; the development freeze already
exists. Do not blindly add `--enforce-gates`: it treats every exploratory layout
or endpoint localization miss as failure, whereas the plan permits explicitly
reported exploratory misses. Inspect the actual required gates:

- Localized correct-type hits for D4-D12; C1/C2/C3/C5 (eight distinct source
  lines); W4-W6; H1/H2. Strong development core positives remain high.
- S1/S2 and C4/C6 are localized when supported; preserve qualified low outcomes
  and unresolved C7 extent. Process all M/R/F/S/C annotated layout regions.
- Zero negative regressions on generated/repaired exemptions, genuine spacing,
  complete endings and separators. Approved source exclusions create no unique
  loss. A redundant-copy loss differs from unique source loss.
- All 200 panel pages reviewed; high-confidence false-positive pages<=2;
  uncertain page counts reported separately. Core burden<=5 per 100 body pages.
- Separate previously unlabeled/up-to10 low-confidence-per-type candidate sample
  and precision/uncertain counts. Keep its actual eligible event count; do not
  manufacture50 when fewer exist. The fixed-page FP rate has a different denominator.
- All 68 independent endpoint comparisons retained. List algorithmic localization
  misses explicitly; they do not negate confirmed visual terminal completeness.

Read the final HTML as well as JSON. Verify every crop/link resolves and crop
bytes match recorded SHA; include rejected probes and low findings. Open it in
an available local browser for visual QA when possible; disclose if browser
layout was not inspected. Save report and review identities in the completion
note. A successful process is not evidence that the corpus is defect-free.

## Resume verification of the same final directory

Before resuming, retain `candidates.json`, `run-manifest.json`,
`evaluation-all.json`, runtime/storage and reviews hashes as named pre-resume
copies in the final run. Use `Copy-Item -LiteralPath`, never move or overwrite
first-validation/freeze evidence. Preserve the initial full-run runtime because
resume records a different elapsed time.

```powershell
Copy-Item -LiteralPath tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/candidates.json -Destination tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/candidates-pre-resume.json
Copy-Item -LiteralPath tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/run-manifest.json -Destination tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/run-manifest-pre-resume.json
Copy-Item -LiteralPath tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/evaluation-all.json -Destination tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/evaluation-all-pre-resume.json
python -u tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope all --detectors all --freeze tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1/freeze.json --lang eng --workers 4 --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1 --resume
python tmp/pdfs/inner_tomi_audit/runs/step101-validation-gold-preparation/compare-candidate-records.py tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/candidates-pre-resume.json tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/candidates.json tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/resume-comparison.json
```

Require `identical:true`, retaining complete candidate identities/regions/scores,
confidence predicates, OCR/provenance, linked events and crop hashes. Candidate
records currently contain no timing keys; the helper excludes listed timestamp/
duration keys and sorts by event ID. Do not discard semantic drift to make a
comparison pass. If candidate bytes change despite identical normalized
semantics, rebind reviews to actual new bytes only after verifying evidence.
Evaluate and regenerate the HTML again after resume: its manifest SHA changed.

## Final-producer worker comparison

Original producer workers 1/4 already passed a92-page Agesilaus/Agis comparison;
those folders are `step102-workers1-frozen` / `step102-workers4-frozen` and are
retained history. They do not verify the changed final producer. Repeat the same
subset, using separate new folders and the chosen freeze:

```powershell
python -u tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope development --units agesilaus agis --detectors all --freeze tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1/freeze.json --lang eng --workers 1 --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-workers1
python -u tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope development --units agesilaus agis --detectors all --freeze tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1/freeze.json --lang eng --workers 4 --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-workers4
python tmp/pdfs/inner_tomi_audit/runs/step101-validation-gold-preparation/compare-candidate-records.py tmp/pdfs/inner_tomi_audit/runs/step102-final-workers1/candidates.json tmp/pdfs/inner_tomi_audit/runs/step102-final-workers4/candidates.json tmp/pdfs/inner_tomi_audit/runs/step102-final-workers-comparison.json --normalize-run-paths
```

Require both complete 92-page manifests, no processing errors and identical
normalized candidates, scores, regions, confidence and crop hashes. The helper
normalizes each own run prefix in path fields; it does not remove crop hashes,
raw observations or semantic evidence. Do not create a freeze for these partial
development runs. Workers must retain separate PyMuPDF objects as implemented.

## Input integrity, completion note and commit/push10.2

Rehash every live and captured input against
`snapshots/baseline_2026_10_06/manifest.json`, including both source PDFs,
8 reader PDFs,8 TEX files, unit map, repair trace and findings report. There
are21 live plus21 captured checks. The one-off final-regression helper records
these hashes, but it verifies the two10.1 halves; do not mislabel that output as
verification of the new all-unit run. Verify the all-unit manifest against the
same input map, working spec and frozen source-index identity.

Save a new dated completion note under `llm_artifacts/` using local authoring
date/time. Record chosen producer/freeze/spec/report/review hashes, actual first
full-run and resume runtimes/storage, dependencies/OCR profile, actual complete
coverage, total/core/rejected candidate counts, mandatory positives, clean
negatives, layout/endpoint misses and their reasons, selected-sample precision,
200-page panel rates, resume/worker comparison hashes and unchanged input hashes.
Retain all older notes. Describe limitations plainly: known-case hits do not
prove corpus recall, low confidence is not automatic assurance, and matched
endpoints do not prove interior completeness. Source Latin remains untouched.

```powershell
python -m py_compile tools/audit_inner_tomi.py
git diff --check
git status --short
```

Stage only intended10.2 documentation/evidence summaries or necessary script
changes within the authorized plan scope. Commit10.2 on the authorized branch and push:

```powershell
git commit -m "Complete inner-tomi audit delivery verification"
git push origin inner-tomi-audit-phase-1
```

Use an appropriate `git add` for exact intended files before the commit; do not
blindly add the entire workspace. Confirm successful push and branch alignment.
The ignored multi-gigabyte diagnostic tree stays local.

## Main README addition after 10.2 is pushed

Then edit the existing root `README.md` in place. A prepared draft exists at
`runs/step101-validation-gold-preparation/README-audit-section-draft.md`; read it
and adapt it to actual final paths/results and surrounding README structure.
This edit remains explicitly requested after 10.2, with its own commit/push.
Include concrete detected issues (duplicate/reordered passages, unique missing
lines, possible terminal problems, copied source heads/logos, Greek scraps,
copied gaps/rules, unexpected blank body pages and glyph clipping/slicing),
prerequisites and existing PowerShell invocation, capture/spec setup, full-run
command with chosen freeze, resume/report/evaluate commands, how to open
`index.html` and use CSV/JSON/coverage/errors/debug/boundary reports, source-only
TF-IDF/occurrence alignment, regional raster OCR, logo correlation, glyph/layout
comparisons, localization/confidence conditions and limitations. State clearly
that it performs inspection and does not repair or normalize source Latin.
Link the detailed tools guide and dated completion note. Validate command names,
report links and Markdown with `git diff --check`, then commit/push the README
addition. Do not claim independent validation for post-validation revisions.
