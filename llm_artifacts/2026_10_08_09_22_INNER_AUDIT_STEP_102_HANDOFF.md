# Implementor handoff: complete inner-tomi audit delivery (step 10.2)

## Authorized work and stopping point

The user requested the remainder of
`llm_artifacts/2026_10_06_16_05_INNER_TOMI_PYTHON_AUDIT_IMPLEMENTATION_PLAN.md`,
with a commit and push after each step. Pushes on `inner-tomi-audit-phase-1`
are explicitly authorized. **Step 10.1 is complete and pushed** (commit
`098cd95` on `inner-tomi-audit-phase-1`); the user requested a pause after 10.1
and this concrete handoff for another LLM session to implement 10.2. Do not
treat this note as authorization to resume the paused task without the user's
next instruction. After that resume, complete 10.2, commit/push it, then add the
main README section about issues detected, invocation, reports and
methodologies and commit/push that documentation. No subagents are authorized.

Use the existing workspace:
`C:\Users\majad\coding\Plutarch\plutarch-latin-latex`.
The shell is PowerShell (the Bash tool is also available for POSIX scripts;
PowerShell has no heredoc — write temp Python to the scratchpad and run it).
The remote is `https://github.com/majadirks/plutarch-latin-latex.git`.
Read AGENTS.md and the plan's 10.1/10.2 requirements. Preserve all existing
plans and notes. Source Latin, transcription caches, PDFs/TEX, repair policy,
source trace and snapshot inputs are read-only. No new test framework or Rust
build is required for this script-only task. Do not add code comments citing
plan steps.

## Current state as of this handoff (10.1 complete)

- Branch `inner-tomi-audit-phase-1` is aligned with origin at commit `098cd95`.
  The only tracked change in that commit is the dated 10.1 calibration note
  `llm_artifacts/2026_10_08_09_18_INNER_TOMI_VALIDATION_CALIBRATION.md`. An
  unrelated untracked `BACKLOG.md` exists in the working tree; do not commit it.
- The diagnostic work tree `tmp/pdfs/inner_tomi_audit/` is gitignored. Another
  machine needs its snapshots, specs, cache and runs copied intact; Git alone
  does not deliver those multi-gigabyte artifacts.
- Revised development run `runs/step101-development-revised-1/` (the freeze
  origin): complete, 34 units / 1,338 body pages, 38 hits / 25 misses / 118
  clean negatives, 698 candidates, 56 grouped core events (4.185/100),
  93 reviewed panel pages. candidates SHA
  `6262949e6e8d8cc5aff496c29bdf82afd5eb6513c76046b07807eaada6ab1eb3`;
  manifest SHA
  `39ce6ce8e0b79a2fc2c2706ddaa36dbbf96c4240e80a9abf347272095e3a3b91`;
  evaluation SHA
  `dc2feef71f58247ea24760ae23330170f06339c0cf81cba87b9d10eca3036333`;
  reviews SHA
  `dac15fd449d258ec3f72ec1d8046a8378488edff9a1c377a59abf715fef73f89`.
- Revised validation run `runs/step101-validation-revised-1/`: complete, 34
  units / 1,340 body pages, no errors, 681 candidates, 13 case hits,
  0 negative violations, 82 clean negatives, 38 grouped core events (2.836/100),
  107/107 panel pages reviewed with zero high FP/uncertain. runtime
  3279.862859 s; run storage 144,911,460 bytes; cache 6,528,828,013 bytes.
  manifest SHA
  `aea092780dabddb3201c2c76349233a8cd5c752862c518b695f78bcfeea592c1`;
  candidates SHA
  `e65424565facace0292ddc17b4d77450bed5f4aeec48176759779509dfb2799b`;
  evaluation SHA
  `60d54d185712cd223f2a73dcae067781ed1621f0862fe67a41a2091c4d34b441`;
  reviews SHA
  `393486518811d88ccdf798a1169f4f0e294803aff182f372a060342ab1c41c7c`.
- Combined gates (both halves) pass via
  `runs/step101-validation-gold-preparation/verify-final-regression.py`, output
  `final-regression-verification.json`: mandatory core D4-D12, C1/C2/C3/C5,
  W4/W6, H1/H2 hits; S1/S2/C4/C6 hits; 2,678 pages / 68 units; 94 grouped core
  events (3.510/100); 200 panel pages, 0 high FP, 0 high uncertain; 68 complete
  endpoints with 42 algorithmic endpoint misses; 23 exploratory layout misses;
  all live + captured input hashes unchanged.
- The two revised `reviews.json` files are the inputs the final merge requires:
  development has 56 events / 93 pages; validation has 38 events / 107 pages
  (combined 200 panel pages). Each binds its own run's candidates SHA.

## Exact frozen configuration and local evidence

The work root is `tmp/pdfs/inner_tomi_audit`; baseline ID is
`baseline_2026_10_06`. Use the existing captured baseline and working specs.
Do not recapture, replace working labels, reinstall dependencies unnecessarily,
change OCR language or overwrite first-validation/revised evidence.

Current producer SHA-256:
`748b8684b23d74c09da84f3c7a9170eaf185ca008f3b7522519ee40b13eb6227`.
Working/tracked labels SHA-256:
`a96a58c6bd924704b7b4e1686cad91851a6f8503880ba182fa836e7354c747c9`.
Chosen freeze:
`runs/step101-development-revised-1/freeze.json`, SHA-256
`77e6d6be418b169b56ff5b52432decd70d6450971cfeb06bf7bc472c1d166fda`.
Its origin contains `run-manifest.json`, `candidates.json`, `reviews.json`,
`evaluation-development.json`, `producing-script.py` and `index.html`.

The freeze binds exact producer bytes, whole-spec hashes, settings, source-only
index, versions and immutable development manifest/candidates/evaluation.
Formatting, checkout newline conversion or changing a comment in the Python
file can invalidate it. Check hashes before running. If a checkout changed only
newlines, use the retained exact `producing-script.py` with verified frozen SHA
for the diagnostic invocation; do not weaken identity checks. Semantic changes
require new complete development/validation calibration and must be disclosed.

The original 9.2 freeze and `runs/step101-validation-first/` are permanent
historical evidence. The first run's independently scored gold has 7 hits,
46 misses, 82 passing negatives; D10/D11/D12/W5/W6 were mandatory failures.
Subsequent results are post-validation regression, not a fresh independent
validation. The independent annotation seal is
`664c8ec4303adf05e095b5236570ae0cf9afae738adbf73e1931279e022b94a2`.
Post-validation D11 printed-word annotation correction is explicitly preserved.
All 68 actual terminal passages are visually complete; algorithmic endpoint
localization misses do not establish terminal truncation or interior completeness.

## Prerequisites and initial checks

Run from the repository root. `python` resolves through
`C:\Users\majad\AppData\Local\Microsoft\WindowsApps\python.exe` and runs
Python 3.13.14. Use existing Poppler/Tesseract on PATH. The frozen OCR language
is `eng`; installed `lat` is not part of the chosen frozen profile. Changing
languages requires a new matching calibration and freeze.

```powershell
git status --short --branch
python tools/audit_inner_tomi.py check
Get-FileHash tools/audit_inner_tomi.py -Algorithm SHA256
Get-FileHash tmp/pdfs/inner_tomi_audit/spec/labels.json -Algorithm SHA256
Get-FileHash tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1/freeze.json -Algorithm SHA256
```

Confirm the three hashes equal the frozen values above before running. Versions
frozen by 10.1: Python 3.13.14; NumPy 2.4.3; Pillow 12.1.1;
opencv-python-headless 4.13.0.92; scikit-learn 1.9.0; RapidFuzz 3.14.3;
PyMuPDF 1.27.2; Tesseract 5.5.0.20241111; Poppler 25.12.0. Use manifest values
in the completion note, not inferred installed versions. Do not commit unrelated
user changes (e.g. `BACKLOG.md`). Verify the branch remains authorized and
inspect any divergence before pushing.

Long runs auto-background in this harness. Launch them and poll the run
directory with short reads; the authoritative completion signal is the final
`run-manifest.json` write with `complete:true` after the process exits (an
earlier intermediate manifest can read `complete:false` while the HTML/report
is still being written). Confirm the owning Python PID has exited before reading
the final manifest.

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

For a long invocation retain its exact arguments, PID, start time and a
producing-script copy. A process PID is not an exec session ID. Do not use
arbitrary elapsed-time cutoffs; wait for actual completion or diagnose a
concrete failure. The all-unit run takes substantially longer than either 1,340-
page half.

Inspect `run-manifest.json`, `coverage.json` and `errors.json`. Require complete
requested/finished sets of 2,678 body pages and 68 units, no processing errors,
no pending detectors and matching frozen index/version/settings identities.
Keep observational/OCR/registration limitations separate from tool failures.
The runtime field is `record['runtime']`, including `elapsed_seconds`,
`run_storage_bytes_before_final_manifest` and `cache_storage_bytes`. Reused
renders/OCR from the 10.1 cache should make this run much faster than a cold run.

## Transfer independent reviews, then evaluate and regenerate HTML

Helpers retained under `runs/step101-validation-gold-preparation/` are one-off
corpus verification scripts, not a test framework. Read a helper before using
it. `merge-final-reviews.py` requires both revised 10.1 `reviews.json` files,
asserts each binds its run's `candidates_sha256`, transfers a verdict only when
the final-run candidate's `unit_id/type/source_regions/output_regions/
crop_sha256/contributions/incomplete_second_opening` identity is identical (and
re-hashes every transferred crop), and asserts the merged panel is exactly 200
distinct pages.

```powershell
python tmp/pdfs/inner_tomi_audit/runs/step101-validation-gold-preparation/merge-final-reviews.py step102-final-all-1
```

This creates `reviews-draft.json` and `review-transfer.json` in the final run.
Inspect `pending_events`. The 10.1 precedent: a core event is pending only if
its candidate identity changed between a revised 10.1 run and the all-unit run
(it should not, if the producer/freeze/cache are unchanged), or if it is newly
nominated. For any pending event, view its actual source/reader images and
adjoining pages and supply an explicit confirmed/false_positive/uncertain
verdict with a reason and diagnostic file/SHA evidence (paths under `cache/` or
`runs/`, each hash-verified). Reuse the 10.1 adjudications where the evidence
matches: the 10.1 duplicate verdicts reference
`duplicate-boundary-replay.json` (bound to labels SHA
`a96a58c6…`) and the W5/W6 verdicts reference
`runs/step101-reader-logo-replay/independent-reviews.json`. Never infer a
negative from an unreviewed event. Every core candidate touching a reviewed
panel page must be adjudicated (the evaluator raises otherwise), and candidate
evidence must bind the final `candidates_sha256`. Once the draft is complete:

```powershell
Copy-Item -LiteralPath tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/reviews-draft.json -Destination tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/reviews.json
python tools/audit_inner_tomi.py evaluate --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1 --split all --reviews tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1/reviews.json
python tools/audit_inner_tomi.py report --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-all-1
```

Do not add `--freeze` to final-all evaluation; the development freeze already
exists. Do not blindly add `--enforce-gates`: it treats every exploratory layout
or endpoint localization miss as failure, whereas the plan permits explicitly
reported exploratory misses. Note that `evaluate_cases` derives case hit/miss
purely from candidate localization; the `--reviews` file drives only the
reviewed precision and the 200-page panel false-positive/uncertain rates.
Inspect the actual required gates:

- Localized correct-type hits for D4-D12; C1/C2/C3/C5 (eight distinct source
  lines); W4-W6; H1/H2. Strong development core positives remain high.
- S1/S2 and C4/C6 are localized when supported; preserve qualified low outcomes
  and unresolved C7 extent. Process all M/R/F/S/C annotated layout regions.
- Zero negative regressions on generated/repaired exemptions, genuine spacing,
  complete endings and separators. Approved source exclusions create no unique
  loss. A redundant-copy loss differs from unique source loss.
- All 200 panel pages reviewed; high-confidence false-positive pages <= 2;
  uncertain page counts reported separately. Core burden <= 5 per 100 body pages
  (10.1 combined was 3.510, so the all-unit figure should be comparable).
- Separate previously unlabeled / up-to-10 low-confidence-per-type candidate
  sample and precision/uncertain counts. Keep its actual eligible event count;
  do not manufacture 50 when fewer exist (10.1 validation had 31 eligible). The
  fixed-page FP rate has a different denominator.
- All 68 independent endpoint comparisons retained. List algorithmic
  localization misses explicitly; they do not negate confirmed visual terminal
  completeness.

Read the final HTML as well as JSON. Verify every crop/link resolves and crop
bytes match recorded SHA; include rejected probes and low findings. (For 10.1
validation: 3,867 distinct single-quoted `src='crops/…png'` references, all
resolving; 3,648 candidate crop records all present and hash-matched; 219 extra
references are retained rejected watermark probes.) Open the HTML in an available
local browser for visual QA when possible; disclose if browser layout was not
inspected. Save report and review identities in the completion note. A
successful process is not evidence that the corpus is defect-free.

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

Require `identical:true` (the helper asserts it), retaining complete candidate
identities/regions/scores, confidence predicates, OCR/provenance, linked events
and crop hashes. Candidate records currently contain no timing keys; the helper
excludes listed timestamp/duration keys and sorts by event ID. Do not discard
semantic drift to make a comparison pass. If candidate bytes change despite
identical normalized semantics, rebind reviews to actual new bytes only after
verifying evidence. Evaluate and regenerate the HTML again after resume: its
manifest SHA changed.

## Final-producer worker comparison

Original producer workers 1/4 already passed a 92-page Agesilaus/Agis
comparison; those folders are `step102-workers1-frozen` /
`step102-workers4-frozen` and are retained history. They do not verify the
changed final producer. Repeat the same subset, using separate new folders and
the chosen freeze:

```powershell
python -u tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope development --units agesilaus agis --detectors all --freeze tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1/freeze.json --lang eng --workers 1 --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-workers1
python -u tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope development --units agesilaus agis --detectors all --freeze tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1/freeze.json --lang eng --workers 4 --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-workers4
python tmp/pdfs/inner_tomi_audit/runs/step101-validation-gold-preparation/compare-candidate-records.py tmp/pdfs/inner_tomi_audit/runs/step102-final-workers1/candidates.json tmp/pdfs/inner_tomi_audit/runs/step102-final-workers4/candidates.json tmp/pdfs/inner_tomi_audit/runs/step102-final-workers-comparison.json --normalize-run-paths
```

Require both complete 92-page manifests, no processing errors and
`identical:true` on normalized candidates, scores, regions, confidence and crop
hashes. The helper normalizes each own run prefix in `path`/`*_path` fields to
`runs/WORKER_RUN/`; it does not remove crop hashes, raw observations or semantic
evidence. Do not create a freeze for these partial development runs. Workers
must retain separate PyMuPDF objects as implemented (PyMuPDF calls stay in the
main process; subprocess workers open their own documents).

## Input integrity, completion note and commit/push 10.2

Rehash every live and captured input against
`snapshots/baseline_2026_10_06/manifest.json`, including both source PDFs,
8 reader PDFs, 8 TEX files, unit map, repair trace and findings report. There
are 21 live plus 21 captured checks. The one-off `verify-final-regression.py`
records these hashes, but it verifies the two 10.1 halves; do not mislabel that
output as verification of the new all-unit run. Verify the all-unit manifest
against the same input map, working spec and frozen source-index identity.

Save a new dated completion note under `llm_artifacts/` using local authoring
date/time. Record chosen producer/freeze/spec/report/review hashes, actual first
full-run and resume runtimes/storage, dependencies/OCR profile, actual complete
coverage, total/core/rejected candidate counts, mandatory positives, clean
negatives, layout/endpoint misses and their reasons, selected-sample precision,
200-page panel rates, resume/worker comparison hashes and unchanged input
hashes. Retain all older notes. Describe limitations plainly: known-case hits do
not prove corpus recall, low confidence is not automatic assurance, and matched
endpoints do not prove interior completeness. Source Latin remains untouched.

```powershell
python -m py_compile tools/audit_inner_tomi.py
git diff --check
git status --short
```

Stage only intended 10.2 documentation/evidence summaries or necessary script
changes within the authorized plan scope (normally just the new completion note;
the diagnostic tree is gitignored). Commit 10.2 on the authorized branch and
push:

```powershell
git add llm_artifacts/<new-10.2-completion-note>.md
git commit -m "Complete inner-tomi audit delivery verification"
git push origin inner-tomi-audit-phase-1
```

Use an appropriate `git add` for exact intended files before the commit; do not
blindly add the entire workspace, and do not add `BACKLOG.md`. Confirm
successful push and branch alignment. The ignored multi-gigabyte diagnostic tree
stays local.

## Main README addition after 10.2 is pushed

Then edit the existing root `README.md` in place. A prepared draft exists at
`tmp/pdfs/inner_tomi_audit/runs/step101-validation-gold-preparation/README-audit-section-draft.md`;
read it and adapt it to actual final paths/results and surrounding README
structure. This edit remains explicitly requested after 10.2, with its own
commit/push. Include concrete detected issues (duplicate/reordered passages,
unique missing lines, possible terminal problems, copied source heads/logos,
Greek scraps, copied gaps/rules, unexpected blank body pages and glyph
clipping/slicing), prerequisites and the existing PowerShell invocation,
capture/spec setup, the full-run command with the chosen freeze,
resume/report/evaluate commands, how to open `index.html` and use
CSV/JSON/coverage/errors/debug/boundary reports, source-only TF-IDF/occurrence
alignment, regional raster OCR, logo correlation, glyph/layout comparisons,
localization/confidence conditions and limitations. State clearly that the tool
performs inspection and does not repair or normalize source Latin. Link the
detailed tools guide and dated completion/calibration notes. Validate command
names, report links and Markdown with `git diff --check`, then commit/push the
README addition. Do not claim independent validation for post-validation
revisions.
