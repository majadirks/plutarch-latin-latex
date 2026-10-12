# Registration scope and pause checkpoint

## Delivered behavior

`register_local_strips` preserves all source-region correspondence records and
defers expensive image registrations on physical source pages without reviewed
geometry. Such records retain their localization/token evidence and source ink,
have unresolved image status, and record
`registration_deferred_reason=source_page_has_no_reviewed_geometry`.
`registration_scope` reports reviewed physical-page and eligible/deferred-record
counts. Progress counts source-region records rather than successful image fits.

Every region on a physical source page containing reviewed geometry remains
eligible, including unreviewed neighbors. Coverage can inherit their shared
continuous fits only on the same physical source page, so this retains all
registrations that the current scorer can use to certify reviewed lines.
No source rectangles, required-ink masks, registration/ink thresholds, Latin,
accepted readers or production repair decisions change.

The failed corpus capture and log remain intact. This change reduces unnecessary
SIFT work; its success at full-corpus scale still requires a fresh run. It does
not clear unreviewed geometry or the automatic real-control misses.

## Verification and identities

A disposable real-observation replay checks an actual reviewed source line,
an unreviewed neighbor on the same page, and a bounded region on a wholly
unreviewed page. Both eligible registrations are identical to their preceding
fits apart from runtime-dependent IDs; the unreviewed neighbor retains a matched
fit. The deferred region retains its identity and correspondence evidence with
no registrations and unresolved image status. Input records remain unchanged.
This is a software behavior probe, not a new corpus coverage verdict.

Probe: `tmp/verify_registration_scope.py`; evidence:
`tmp/pdfs/facsimile_line_audit/runs/verification/registration_scope_probe/probe-results.json`.
Python compilation and `git diff --check` pass. Formatting, strict Clippy and
Rust tests pass: 419 unit tests, 9 integration tests, 76 intentionally ignored
unit tests. All 21 protected source/reader/TeX/unit/repair inputs match capture
hashes. The preexisting user `BACKLOG.md` edit remains outside implementation
commits.

All seven full-path cases pass under fresh current-code captures: 30/30
deliberately introduced defects detected, zero false unique-loss findings and
zero retained-control failures. Every region's logical identifier, source
rectangle and coverage status matches the preceding implementation. Aggregate
counts remain 98 complete, 17 partial, 13 absent and 557 unresolved. The cases
exercise 147 deferred records, without converting any unresolved region to a
completeness finding. Historical case inputs and preceding predictions remain.
The aggregate report is
`tmp/pdfs/facsimile_line_audit/runs/verification/cases_5_1a/results/48dc07869ffed54f0740a4f99846daf616aab127933b33165921d548a40e2552/verification-results.json`.

The fresh capture uses the existing source-only context policy with forty review
decisions, including the twelve bounded acceptance/context spans. Authenticated
raw reader geometry and decoded pixels are reused; logical maps and verdicts
are not reused. Python is already 64-bit 3.13.14 with OpenCV 4.13.0, so an
interpreter switch is unnecessary.

- Capture directory: `tmp/pdfs/facsimile_line_audit/snapshots/registration_scope_context_v5`.
- Snapshot identity: `a5c903bbef6bf0a0333e945b74134114d2a09f8d6098db740188aad1d531f319`.
- Runtime identity: `5dfaef28869c000d93b669ecf8b4274ec972d53d16857d218b58abfb89be462a`.
- Scope: two source scans, eight readers, 68 units, 2,912 physical reader pages,
  2,642 mapped body pages.
- Capture log: `tmp/registration-scope-capture.log`.
- Regression log: `tmp/registration-scope-verification.log`.
- Rust gate logs: `tmp/registration-scope-clippy.log` and
  `tmp/registration-scope-cargo-test.log`.

## Remaining work after resume

The previous full run, `runs/corpus_7_2_candidate`, failed in OpenCV SIFT with an
allocation error after its last printed counter of 280,801/358,744. Its 1,315-page
source inventory is complete; its full coverage ledger and seam report do not
exist. Its old comparison code must not be resumed after this change.

No new full corpus run is started before the requested pause. Resume with a
fresh run directory against the new capture, using task-local thread limits:

```powershell
$env:OPENBLAS_NUM_THREADS='1'
$env:OMP_NUM_THREADS='1'
$env:MKL_NUM_THREADS='1'
py -3.13 tools/facsimile_line_audit.py run --snapshot tmp/pdfs/facsimile_line_audit/snapshots/registration_scope_context_v5 --run-dir tmp/pdfs/facsimile_line_audit/runs/corpus_registration_scope_v5
```

If processing completes, verify actual scope, record and crop/report identities,
then repeat the exact invocation with `--resume` and compare output hashes.
Generate the general glyph-cut companion observations and score the fixed real
acceptance controls from automatic neighborhoods. Preserve misses and uncertain
regions. Review the fixed panel and controls, then create an authenticated
archive with the delivery tool. Human review and calibration freeze remain
pending; a completed processing run is not a no-loss certificate.

The bounded source-context experiment is finished: targeted detection is 2/2,
automatic localization/detection remains 0/2. Do not enter another indefinite
manual tuning loop. The next full report should quantify unresolved scope so
the cost and feasibility of further verification can be assessed concretely.
No additional real source-Latin omission has been confirmed beyond the two
user-supplied damaged lines. The thirty controlled omissions are deliberately
introduced verification defects, not corpus findings.
