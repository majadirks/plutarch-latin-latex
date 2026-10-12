# General glyph-cut seam checkpoint

## Delivered integration

`facsimile_line_delivery.py glyph-cuts` adds separately identified seam
observations to an authenticated completed audit. It selects reviewed source
regions and adjacent reader pages from the actual correspondence neighborhoods,
without using fixed control page pairs. It shares source-backed registration,
glyph-cut measurement and report rendering with `controls`. Complete machine
ink status remains separate from a warning that glyphs are cut across pages.

Source/reader images, transforms, observation-code identities and comparison
record digests are retained in fresh JSON/HTML outputs. Completed seam runs
attach matching reader-body seam IDs. The machine ledger, source policy and
comparison caches remain unchanged. This companion report does not claim
complete readable Latin.

## Measured scope and verification

On the completed real calibration run, 28 source regions have reviewed geometry
and ownership. Sixteen have eligible adjacent-page neighborhoods; twelve do not.
Of the sixteen evaluated, one has no persistent supported glyph cut and fifteen
remain unresolved because both edge registrations are not valid. There are zero
new glyph-cut findings. The 4,307 regions outside the reviewed scope remain
uncertified. Zero warnings cannot be interpreted as all regions passing.

The retained full-path case has seventeen reviewed source regions and no
eligible two-page neighborhoods. Its empty general result is scope accounting,
not a successful test of a glyph-cut negative. The head/foot-loss case provides
eight eligible regions and eight valid reader-body seam references. All eight
supplemental edge measurements remain unresolved; its original omission findings
are preserved in the unchanged machine ledger.

The targeted real-control mode still detects 2/2 actual defects, with 22 and 24
persistent crossing components. The two intact neighboring-row negatives and
changed-input/wrong-page rejection checks are documented in the preceding
checkpoint. These targeted successes do not prove automatic corpus localization
of the same two defects.

Python compilation passes. The general real report authenticates 51 crop images,
the head/foot report 25, and the final real-control report ten; all 86 image links
resolve and match their recorded hashes. Their observation-code identities match
the executed implementation. All five comparison-code files still match the
active full-run capture. Rust is unchanged; the session's formatting, strict
Clippy and full Rust-suite gates passed before this independent Python integration.
No unit-test framework or paid model request was added.

Evidence paths are relative to `tmp/pdfs/facsimile_line_audit/runs/`:

| Evidence | Path |
| --- | --- |
| General real observations | `calibration/glyph_cuts_context_v3_2.json` and matching `.html` |
| Retained-case scope observation | `verification/retained_general_glyph_cuts_2026_10_11.json` |
| Head/foot seam-reference observation | `verification/head_foot_general_glyph_cuts_2026_10_11.json` and matching `.html` |
| Final real control observations | `verification/known_horizontal_splits_2026_10_11/controls_general_integration_4.json` and matching `.html` |

Earlier predictions remain intact. The source-reviewed scope of the current
candidate capture does not include the newly fixed source rows as normal machine
source-policy decisions. Normal corpus localization and detection of those rows
therefore remain to be verified through independently reviewed source policy
and a fresh comparison identity. The separate real-control evidence is not
silently imported as a machine policy decision.

## Full-run meaning and remaining work

The running command is an offline audit of both captured source scans against
all eight accepted inner readers and all 68 units. Its 1,315 source pages are the
inclusive requested unit ranges, not every physical page of the two scans.
Readers are not regenerated. Source Latin and production repair policy are not
changed.

The `Local strip observations` counter advances once per source-region
correspondence record in the registration loop. Its denominator is 358,744;
these are not all confirmed Latin lines, distinct strip fits or successful
matches. A record may have several local image bands or no usable fit. An
unresolved record still advances processing progress. The end of this stage
writes source-to-reader alignment observations; visible-ink coverage scoring and
systematic seam/report generation follow.

The corpus deliverable is a ledger of complete, partial, absent and unresolved
observations plus evidence images, seam tasks and an HTML report. A successful
processing exit does not certify corpus completeness. The full candidate is
still incomplete at this checkpoint. Finish its run, verify exact resume and
scope/evidence counts, assess the fixed panel and controls, then create the final
authenticated archive. Calibration freeze remains unapproved. The preexisting
user `BACKLOG.md` edit remains outside implementation commits.
