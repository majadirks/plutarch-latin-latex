# Start and terminal-prose comparison verification

Implemented audit-plan step 5.2 in `tools/audit_inner_tomi.py`.
`compare_boundaries` selects Latin prose within each original source partition,
excluding standalone capital headings, and uses the shared bounded occurrence
aligner to compare first/last source passages with reader edge windows. Source
occurrence IDs, raw OCR, boxes, operations, coverage and ordered scores remain
available in `boundaries.json`. Extra output context outside the expected source
interval does not lower the edge correspondence score; deleted source tokens
remain charged. The outermost expected token must match for a matched result.

`--detectors boundaries` processes the first and last two body pages of each
selected unit. It records that limited page coverage explicitly. `prose` and
`all` also dispatch boundary comparison alongside the interior detectors. The
local `boundaries.html` page links source/reader crops and states that endpoint
correspondence does not establish interior completeness. Complete endpoint
observations use the existing evaluator's `complete_endpoint` relation; they
are separate from `terminal_loss` nominations. A loss nomination requires an
unmatched terminal line with raster source support and absence after two
300-DPI reader OCR retries, and remains low confidence for visual adjudication.

Verification used the existing Python 3.13.14 interpreter. `python` resolves in
the unrestricted execution session, although the restricted PowerShell session
cannot resolve it. No installation was needed. No Rust files, source Latin,
transcription cache, production masks or readers were changed. No unit tests
were added or run, following the diagnostic plan's explicit instruction.

## Actual development run

- Baseline: `baseline_2026_10_06`.
- Final run: `tmp/pdfs/inner_tomi_audit/runs/step52-development-final`.
- Requested/completed scope: all 34 development units, 135 distinct edge body
  pages. Interior body pages were not requested by this run.
- Starts: 25 matched, 9 inconclusive.
- Terminals: 27 matched endpoint observations, 7 inconclusive, no terminal-loss
  nominations.
- All 34 selected terminal source pages agree with the independent report
  ledger in `tmp/pdfs/reinspection/endpoint-ledger.json`.
- Source-page selection correctly places Agis at v2/378 before partition end
  379, Marius at v1/530 before partition end 531, and the Sertorius/Eumenes
  comparison at surviving v2/105 rather than approved duplicate skip 104.
- Source/reader crops for Agis's sparse ending and Marius's terminal prose were
  visually inspected. They contain the complete terminal wording. Agis spans
  two reader crops, preserving the legitimate sparse final page.
- Every image link in `boundaries.html` resolves to an existing crop.
- `python -m py_compile tools/audit_inner_tomi.py` and `git diff --check` passed.
- Explicit `--resume` completed successfully; SHA-256 hashes of
  `candidates.json`, `boundaries.json` and `boundaries.html` were identical
  before and after the resumed run.
- The run verifies captured input hashes both before processing and before
  writing the final record. Explicit resume checks baseline, scope, detector
  selection, units, settings, spec hashes and script identity.

## Remaining uncertainty

Inconclusive terminal OCR remains in Alexander, Caius Gracchus, Marius, the
Alcibiades/Coriolanus comparison, Coriolanus, Nicias and Phocion. These are not
terminal-loss findings. Some source last lines lack sufficient embedded/raster
agreement; others have reader OCR noise or a retry that finds correspondence
absent from the coarse observation. The independent report marks their endings
complete. Its conclusion is not used as a detector branch or substituted for
the diagnostic observations.

The existing endpoint labels are still pending annotations. The evaluator
therefore reports 34 pending endpoint annotations and 291 cases not requested,
with no scored hits or misses. Matching the independent source-page ledger is
a locator check, not a claim of measured endpoint recall. Gold measurement and
adjudication remain scheduled for the reporting/calibration work.

Brutus, Otho and all other validation endpoints have not been processed by this
step. The full 68-endpoint comparison and Otho's split continuation remain
deferred to controlled validation.
