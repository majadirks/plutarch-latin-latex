# Facsimile audit delivery checkpoint

## Delivered behavior

The independent Python audit is runnable. `tools/facsimile_line_audit.py`
captures accepted inputs, inventories source geometry, localizes source regions,
measures visible-reader ink, reports seams and imports separately authenticated
reviews. `tools/facsimile_seam_review.py` provides capped optional model review
with explicit delivery/resume states. Production generation and source Latin
remain unchanged. The setup and operation guide is
`tools/README-facsimile-line-audit.md`.

`tools/facsimile_line_delivery.py` authenticates completed-run scope and evidence,
associates observations with the previously fixed panel, and copies authenticated
evidence into immutable archives. It records its own code identity separately
from comparison code, allowing delivery verification without invalidating the
running comparison. It does not approve calibration or certify complete Latin.

This is a bounded tool-delivery checkpoint for step 7.2. The entire plan is not
complete: step 7.1's real-source calibration freeze and step 7.2's completed full
run, exact full resume and final corpus archive remain outstanding. Unresolved
regions are explicit audit findings; eliminating every uncertainty is not a
prerequisite for handing over the runnable diagnostic tool.

## Verification evidence

The comparison code remains the code committed in `1e6b743`. The seven complete
controlled cases already pass, detecting 30/30 independently localized known
loss targets with no false unique losses in retained/redundant controls. Their
complete report is:

```text
tmp/pdfs/facsimile_line_audit/runs/verification/cases_5_1a/results/7c772b7ecdfff1cb14fb4b5cd90406bc1aadf53f8e04eed4846edbedffdcdedb/verification-results.json
```

Disposable end-to-end delivery checks use copies of actual completed-case
records, not mocked results. A 1.1-million-character Unicode JSON record crosses
streaming chunk boundaries correctly. A valid scope report succeeds; altered
crop bytes fail digest verification; falsified source counts fail independent
requested-range accounting even after their record hash is updated. Reusing an
existing archive is rejected without changing its manifest.

A separate copy of the comparison CLI with changed coverage code rejects the
captured run before creating a run directory or accessing comparison caches.
The live comparison files remain unchanged. Python compilation,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` and
`cargo test` pass. The Rust suite reports 419 library and nine CLI tests passed,
with 76 existing tests ignored. All
21 protected source-PDF, reader-PDF/TeX, unit-table and repair-ledger files retain
their captured hashes.

The existing real-control archive authenticates 7,250 files. The final delivery
tool also creates and authenticates a fresh controlled-clipping archive with
414 files. These are evidence-copy checks, not a final full-corpus archive.

| Evidence | Identity or location |
| --- | --- |
| Delivery code SHA-256 | `92e6a199a542f9e48e1adb675192b36a9d48b59521132dd582b51b963a7179a5` |
| Delivery tamper/streaming probe | `tmp/pdfs/facsimile_line_audit/runs/verification/delivery_workflow_probe_2026_10_11/` |
| Changed-code preflight | `tmp/pdfs/facsimile_line_audit/runs/verification/changed_code_preflight_2026_10_11/` |
| Real-control archive | `tmp/pdfs/facsimile_line_audit/snapshots/archive_delivery_probe_2026_10_10/` |
| Real-control archive manifest SHA-256 | `612b8132f22aa7b27d363e213817875eb394791041bffd0b3b4b1f6681468f90` |
| Controlled-clipping archive | `tmp/pdfs/facsimile_line_audit/snapshots/archive_delivery_fixture_2026_10_11/` |
| Controlled-clipping archive manifest SHA-256 | `7aa143e15d26d12532d9a56c253429344bf191c1fc318c6fec36414a7a9c09ce` |

Large captures, images and diagnostic archives remain local under `tmp/`;
versioned implementation, guides and this checkpoint are committed and pushed.
The preexisting user edit to `BACKLOG.md` is excluded.

## Real-source results and convergence limits

The completed targeted run processes 16 source pages and all 308 reader pages
in its five requested units. Its scope report authenticates 4,487 crop files.
There are six complete and 4,329 unresolved source regions. Among the 28 reviewed
regions, six are complete and 22 unresolved; 4,307 regions have unreviewed required
geometry. The report is:

```text
tmp/pdfs/facsimile_line_audit/runs/calibration/delivery_scope_2026_10_11.json
```

The fixed panel has 200 body pages, of which 18 are processed in this targeted
run. Its source scope is incomplete. Independent ground truth is pending, so
misses and false alarms are null, not zero. The existing 54 targeted controls
remain separate from the panel. Earlier restricted and expanded source scopes
do not establish an independent accuracy improvement; the expanded scope also
left previously localized Romulus material unresolved.

At this checkpoint the full candidate run has inventoried all 1,315 requested
physical source pages across both scans and is processing local strip
observations. Its manifest still marks processing incomplete. Its requested
scope contains all 68 units and all eight readers. Processing progress is not
a coverage certificate, and a final coverage/seam denominator is not yet
available.

| Candidate identity | Value |
| --- | --- |
| Capture directory | `tmp/pdfs/facsimile_line_audit/snapshots/calibration_7_1_context_v3/` |
| Snapshot ID | `4b786b5b49b0bd71e16f125a310015a7495b9350ef267d4b0da595dc6c19613d` |
| Full run directory | `tmp/pdfs/facsimile_line_audit/runs/corpus_7_2_candidate/` |
| Full runtime ID | `cf526ac425260e5ad6a8edc5453f8d137cd641a8bb03ac9c98932b22a9f666a4` |
| Progress log | `tmp/pdfs/facsimile_line_audit/runs/corpus_7_2_candidate.log` |

## Remaining acceptance work

Finish the full run and authenticate actual source/reader/seam counts, every
report link and crop hash. Repeat its exact invocation with `--resume` and
compare stable record bytes. Preserve incomplete runs and earlier predictions.

Review and score the fixed panel and remaining restored/partial-duplicate
controls. Report each miss, false alarm and uncertainty against its real
denominator. Inspect localization and required-ink evidence before adjusting
predicates; preserve uncertainty rather than loosen tolerances to manufacture
passes. An approved settings/policy/runtime freeze requires those reviewed
controls to meet the plan's criteria.

Create the final dated corpus archive after full-run verification, including
case evidence, source reviews, reports and any separately authenticated human
or model observations. Proposed losses remain a reviewed worklist; no repairs
are activated automatically. No paid model call has been made. Human visual
review of the real panel and representative corpus seams remains outstanding.
