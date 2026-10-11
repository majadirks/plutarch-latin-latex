# Facsimile calibration checkpoint

## Context

This checkpoint continues the source-line coverage and seam audit implementation
through the calibration infrastructure in step 7.1. It preserves the earlier
implementation and evidence. **Step 7.1 is not fully verified:** the real retained
controls still contain unresolved observations, the fixed random panel has not
been scored, and calibration identities have not been approved for freezing.
Step 7.2's full-corpus execution and archive remain pending.

## Implemented behavior

- `panel --snapshot <corpus-capture>` fixes 25 distinct body pages in each of eight
  tomi by input-only SHA-256 ranking. Source PDFs, reader PDFs/TeX and the unit table
  determine the identity; code, settings and source policy do not. Repeated
  selection across captures with those same inputs reproduced the identical
  200-page list. Six restored source-page controls and 48 numeric partial-duplicate
  source-page references are counted separately. Their inclusion grants no
  ownership or duplicate equivalence.
- Verification scores the independently prepared source/reader geometry, not
  merely a line's candidate status. It checks required source intersection and
  extra line height, reader page and gap/painted extent, endpoint context, and
  persistent partial deficits at both thresholds. Independent source-page and
  reader-body transition counts are also checked.
- Source grouping uses original component coordinates with local baseline slopes
  in 20-glyph-height bands. The bounded rotation search uses 61 samples over three
  degrees in each direction. Tall glyphs remain eligible; punctuation, faint
  components and residual ink remain explicit. Source pixels and Latin are
  unchanged. Source-region identities use the revised detector identity.
- `inventory` and `run` accept authenticated `--source-pages` selection. These
  records explicitly distinguish selected physical pages from complete unit
  ranges. A targeted run retains its selected units' mapped reader pages and
  cannot certify source pages excluded by the selector.

## Verified disposable cases

The final ordinary full pipeline passed all seven prepared cases. The same
invocation was repeated successfully with authenticated completed records.

| Case | Known loss targets localized | False unique-loss findings in retained/redundant controls |
| --- | ---: | ---: |
| `retained_control` | 0 / 0 | 0 |
| `head_and_foot_loss` | 4 / 4 | Not a retained control |
| `interior_block_loss` | 4 / 4 | Not a retained control |
| `redundant_copy_removed` | 0 / 0 | 0 |
| `sole_complete_copy_removed` | 16 / 16 | Not a retained control |
| `shared_page_unique_tail` | 4 / 4 | Not a retained control |
| `visible_pdf_clipping` | 2 / 2 | Not a retained control |

All 30 known loss targets were detected at the independent source/reader geometry.
There were 98 complete, 17 partial, 13 absent and 557 unresolved regions across the
cases. The unresolved count includes the sparse one-line subcase and raw parent,
faint and ancillary objects; it is not a known-loss miss count. Sparse manual
evidence does not improve automatic recall. The cases produced 99 seams and 303
neutral tasks. A disposable end-to-end probe authenticated 15,702 report/crop
links and digests, independent transition counts, and separate human-review
import/stale rejection. Mutating preserved findings into generic whole-page
warnings or changing their reader pages caused all 30 targets to fail geometry
scoring, as required.

The seven cases establish their particular sensitivity. They do not establish
complete Latin coverage of the real readers.

## Real restored-control observations

Source-only views were inspected before reader verdicts. Twenty-three complete
Latin body spans were imported through the normal source adjudication command.
Their parent objects retain unreviewed ink. This policy is separate from the
tracked default source policy and from production repair ledgers.

The targeted run inventoried six requested physical source pages and processed
all 308 mapped reader occurrences/pages in its five selected units. Processing
completed with zero recorded errors. Of 23 source-reviewed spans, four were
machine complete and 19 remained unresolved; none was partial or absent.
The complete spans are the first restored Romulus foot row, the final Pericles
prose row and verse on v1/214, and the Antonius row beginning `imperium tenuit`.
Across the whole targeted inventory, 4 of 1,594 regions were complete and 1,590
were unresolved, including all 1,571 unreviewed parent/raw objects.

The 19 reviewed unresolved spans have these causes:

- Eleven have a valid localized image fit but resolution or threshold-support
  uncertainty. The umbrella `resolution_uncertain` field includes disagreement
  between threshold observations; it must not be read solely as low projected
  pixel count.
- Three are the smaller verse rows on Pericles v1/215. Their independently
  screened whole-page glyph scale disagrees with the local strip/word scale.
- Five are the final two sampled Antonius rows and final three sampled Dio rows.
  They lack ordered two-sided anchors in this explicitly restricted source scope.

These are uncertainties, not certified retained spans or verified omissions.
Do not loosen tolerances, shrink the required reference from reader outcomes, or
activate repairs to clear them. The 48 partial-duplicate source-page references
remain unreviewed for calibration. The random panel denominator is 200, with zero
pages scored as an independent complete-source panel in this checkpoint. OCR of
some panel pages during targeted work does not score that panel.

## Evidence and identities

All paths below are relative to `tmp/pdfs/facsimile_line_audit/`.

| Evidence | Path or identity |
| --- | --- |
| Fixed panel | `runs/calibration/5aef7adbc4a0a25c91a3bae5221cb6b216a778d8a7f180fbec08ab1e16e0584a/panel.json` |
| Final seven-case report | `runs/verification/cases_5_1a/results/a7967434a991502b6cf839bcfea4a14244182490ee4fc4d7ba9af3d82278eb88/verification-results.json` |
| Source-only inventory | `runs/calibration/targeted_source_inventory/` |
| Source views, review JSON and adopted policy | `runs/calibration/source-controls/` |
| Adopted control policy | `runs/calibration/source-controls/policy-reviewed-controls.json` |
| Candidate capture | `snapshots/calibration_7_1_reviewed_controls/` |
| Candidate snapshot ID | `03d42c852234a06c55f694c18c38482a83ca4b34f25ff0b76a7f359211d13e48` |
| Candidate capture runtime ID | `84222f541ddfd3ceab514501ce443e744d9926681014164ff18c6fedeac57e85` |
| Candidate settings identity | `98cb599267cc170ba43e697024d8f0772a0eb41359a735cb00b35f9fa47626a2` |
| Candidate source-policy identity | `f7db981590306d75145a14a58148c4f579e9a8b43652c65ff66adf9a43aaa07b` |
| Targeted coverage run | `runs/calibration/reviewed_controls_coverage/` |
| Targeted run runtime ID | `30ffc10e15f19da05a2962be49369e555c8c452a46e182c85db59fd5180ac37a` |

These are authenticated candidate identities, **not an approved calibration
freeze**. Settings/policy identities are hashes of canonical parsed content;
individual copied input files also retain their byte digests in capture manifests.
Earlier captures, predictions, source proposals and interrupted runs remain in
place. No earlier evidence was deleted, including the large correspondence
diagnostics. Evidence under `tmp/` is local and has not yet been placed in the
durable full-corpus archive required by step 7.2.

## Verification and limits

Changed Python modules compile. `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test` passed. The Rust suite
reported 419 library tests and nine CLI tests passing, with 76 existing tests
ignored. All 21 protected source-PDF, reader-PDF/TeX, unit-table and repair-ledger
input files were rehashed against the candidate capture; none changed.

No Rust generator or transcription changes were made. No new unit-test framework
was added. Disposable probes remain outside tracked implementation. No live
model requests or credential reads were needed for this checkpoint. Human review
of the fixed panel and representative reports remains pending; the in-app
browser was unavailable during the preceding report work.

## Continuation

1. Preserve this result set. Inspect the saved missing-ink and threshold
   observations for the eleven resolution/threshold uncertainties. Distinguish
   genuinely inadequate resolution from threshold disagreement before proposing
   any change. Higher-resolution visible-PDF measurement must retain the clipped
   whole-line and terminal-glyph sensitivity in the seven independent cases.
2. Investigate measuring typography in the original local anchor strip through
   the shared screened estimator, while retaining a sufficient reference page
   height for its component screen. Whole-page typography can differ from short
   verse typography. Retain word-box corroboration and strict transform validity.
3. Add source context on each side of the restricted control pages, using fresh
   explicit source scopes and source-only review where ownership is shared. The
   final sampled Antonius/Dio rows are interior to their units and must not be
   relabeled as unit endings to manufacture anchors. Raw renders/OCR may be reused;
   old coverage verdicts may not.
4. Inspect partial duplicate openings independently, then score the saved
   200-page panel. Report every miss, false alarm and uncertainty with its actual
   denominator. Freeze identities after the case and reviewed retained-control
   criteria are fulfilled, retaining all earlier predictions as regression
   evidence.
5. Execute and deliver step 7.2. Consider corpus memory before launching a
   monolithic process: the earlier 158-page inventory was approximately 394 MB,
   its logical map 310 MB and its compact correspondences 1.88 GB on disk. This
   machine had approximately 2.7 GB of free physical memory during the capacity
   check. A bounded execution strategy must preserve full source-index semantics
   and all cross-unit/source/reader seams; independent unit runs alone cannot
   silently stand in for the full invocation.
6. Commit and push each completed step. Preserve the user's preexisting
   `BACKLOG.md` change outside implementation commits. Phase 2 trace diagnostics
   remain optional and deferred.

The exact targeted run can be authenticated again with:

```powershell
python tools/facsimile_line_audit.py run --snapshot tmp/pdfs/facsimile_line_audit/snapshots/calibration_7_1_reviewed_controls --run-dir tmp/pdfs/facsimile_line_audit/runs/calibration/reviewed_controls_coverage --units romulus,pericles,lysander,dio,antonius --source-pages 0:38,0:214,0:215,0:535,1:580,1:562 --stop-after coverage --resume
python tools/facsimile_line_audit.py verify --out tmp/pdfs/facsimile_line_audit/runs/verification/cases_5_1a
```

Executed code or comparison-setting changes require fresh captures and run names.
