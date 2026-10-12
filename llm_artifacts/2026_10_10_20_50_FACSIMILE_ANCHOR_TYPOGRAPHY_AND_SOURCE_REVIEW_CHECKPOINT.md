# Anchor typography and repeated source review checkpoint

## Context

This record continues the calibration work described in the 20:13 checkpoint.
Calibration freeze, scoring of the fixed 200-page panel, partial-duplicate source
review and full-corpus delivery remain pending. The implementation preserves all
earlier captures and predictions; subsequent observations are regression evidence.

## Changes and verification

Local registration measures glyph height from corresponding anchor-word ink on
each original page through the shared screened estimator. A page-sized white
measurement frame preserves the estimator's component-height bounds; original
anchor pixels are copied without resampling. The full required reference ink is
unchanged. Registration metadata binds the actual word boxes and the
`paired_anchor_words` scope. Word-box corroboration, the 10% scale tolerance,
rotation, translation and inlier predicates remain intact.

Source-policy application updates existing reviewed children rather than
duplicating them, preserves their authentic untouched crop metadata, and removes
superseded derived children from the new inventory. Original raw objects and ink
components remain present. Ancestor decisions precede descendant decisions.
An end-to-end probe using the actual 16-page inventory confirmed that applying the
new policy twice produced identical inventories: 23 existing child crops remained
unchanged and 28 regions were source reviewed. A deliberate geometry revision
removed its superseded derived child while preserving original inventory bytes.

The real comparison also reached focused OCR recovery with nonempty source and
reader words. Its bounded alignment receives the captured source-index detector
settings. Both retry modes, PSM 6 and 11, completed in the final control run.

All seven full disposable cases passed, with 30/30 independently localized known
loss targets and no false unique losses in retained/redundant controls. Generic
page warnings and wrong reader-page findings failed the geometry probe. Evidence
verification checked 15,702 report/crop links and hashes, transition counts and
separate human-review import/stale rejection. Python compilation, formatting,
strict Clippy and the full Rust suite passed: 419 library and nine CLI tests, with
76 existing tests ignored. All 21 protected source-PDF, reader-PDF/TeX, unit-table
and repair-ledger files retained their captured byte hashes.

## Source context and real observations

Source-only inspection of v2/563 identified five Antonius continuation rows above
the separator and the `DEMETRII CUM ANTONIO COMPARATIO` title. They were imported
through normal adjudication, with the actual final Antonius row marked as its
source ending. The sampled v2/562 rows remain interior to Antonius. This review
does not approve the comparison body or surrounding Greek/furniture.

The final targeted run processed these 16 physical source pages:

```text
v1: 37, 38, 39, 213, 214, 215, 216, 534, 535, 536
v2: 561, 562, 563, 579, 580, 581
```

It processed all 308 mapped reader pages/occurrences in the five selected units,
completed with zero recorded processing errors, and authenticated exact resume.
The 28 reviewed regions have six complete and 22 unresolved outcomes. Among the
original 23 restored-control spans, four are complete and 19 unresolved; among
the five added continuation rows, two are complete and three unresolved. Across
the full targeted inventory, six of 4,335 regions are complete and 4,329 unresolved.
No partial or absent findings were produced.

Paired-word typography supplies valid transforms for the first and third small
verse rows on Pericles v1/215. They remain uncertain in ink measurement. The
middle verse row is complete through a valid shared continuous context fit;
its own failed scale observation remains in the correspondence records.

The broader source-index scope also changes localization: the Romulus foot spans
have no eligible ordered local image measurement in this run. One Romulus span
was complete in the earlier restricted six-page run. The complete-region count
has therefore not improved across the original 23 controls, and these scopes
must not be presented as an independent accuracy comparison. Preserve both
predictions. Investigate source-only gutter/ownership hypotheses and ordered
localization before changing confidence predicates or required geometry.

A separate 300-DPI diagnostic projected the earlier 180-DPI transforms into
higher-resolution visible reader renders. None of its eleven regions became
complete, and six had persistent apparent deficits. Those observations have
not changed authoritative verdicts: rerasterization alone is insufficient;
registration and tiny-component/threshold uncertainty require inspection.

## Evidence and candidate identities

Paths are relative to `tmp/pdfs/facsimile_line_audit/`.

| Record | Path or identity |
| --- | --- |
| Final cases | `runs/verification/cases_5_1a/results/7c772b7ecdfff1cb14fb4b5cd90406bc1aadf53f8e04eed4846edbedffdcdedb/verification-results.json` |
| Candidate capture | `snapshots/calibration_7_1_context_v3/` |
| Snapshot ID | `4b786b5b49b0bd71e16f125a310015a7495b9350ef267d4b0da595dc6c19613d` |
| Capture runtime ID | `8bea0c77778a6cf059ebdf9b21162f26113b2a9a247f5a4eada1a1b52e0826dd` |
| Source-policy identity | `ef1391b351176915d8ad131934ff9caf552ca9bc4e031beb5e330e2f3cc20da7` |
| Adopted source policy | `runs/calibration/source-controls/policy-context-v3.json` |
| Source-only continuation review | `runs/calibration/source-controls/context-source-reviews.json` |
| Complete targeted run | `runs/calibration/context_v3_coverage/` |
| Targeted runtime ID | `c6022112055e5f2df3a818f0fff63761c85631d0c1a0f6cd134197681f1f6bc7` |
| Higher-resolution diagnostic | `runs/calibration/diagnostic_300dpi/observations.json` |

The preceding `anchor_typography_context` run failed during focused recovery and
remains explicitly incomplete. Its source-only review evidence was authenticated
for the continuation import. No failed processing run is counted as complete
coverage. The fixed panel remains the same input-only 200-page selection, with
zero pages scored as a complete-source independent panel at this checkpoint.
These identities are authenticated candidates, not an approved calibration freeze.

No production Rust, source Latin, accepted readers or repair ledgers changed.
The user's `BACKLOG.md` edit remains outside implementation commits. No live
model request was made, and no new unit-test framework was introduced.
