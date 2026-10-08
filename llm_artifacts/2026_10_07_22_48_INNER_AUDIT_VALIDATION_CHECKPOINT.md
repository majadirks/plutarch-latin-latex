# Inner-tomi validation continuation checkpoint

Step 9.2 is committed and pushed as `a3b53a2` on
`inner-tomi-audit-phase-1`. The user requested completion of the remainder of
the plan, with a commit/push after each step and the main README addition after
10.2 is pushed. The latest instruction is to save a good checkpoint immediately because of
the weekly usage limit and hand off unfinished10.1 plus10.2 to another LLM.
Implementation is paused before 10.1 completion. Save/commit/push this checkpoint
without claiming the step is finished. The already running read-only diagnostic
process remains active. The concrete handoff is
`llm_artifacts/2026_10_08_08_33_INNER_AUDIT_IMPLEMENTOR_HANDOFF.md`.
No subagents are authorized. Unrelated untracked `BACKLOG.md` is excluded.

## First validation run

`tmp/pdfs/inner_tomi_audit/runs/step101-validation-first/` completed its hidden native
process PID 16520, launched at 02:23 UTC on October 8. It used the retained
producer SHA
`d9b1d05b6ad7a45777dd67f38cda892949e1a2b614a503e784970457dfbbac6a`,
the 9.2 development freeze SHA
`c99fd09855268cbddc197513051e1bc1fe37f6c5d073f1e4a778028dc4581650`,
all 34 validation units/1,340 body pages, all detectors and four workers.
Native launch/stdout/stderr are retained. The finalized manifest accounts for
34 units and 1,340 body pages with no processing errors. Elapsed runtime was
14,540.419356 seconds; run size before its final manifest was 121,464,108 bytes
and cache size 6,527,365,590 bytes. Original specs, producing script and first
complete evaluation were preserved before the live script/specs changed.

Original pending-label evaluation: 1 hit, 3 misses, 131 pending, 190 not
requested. Separate unchanged-matcher score against the sealed independent
reviewed gold: 7 hits, 46 misses, 82 clean negatives, zero negative violations,
190 not requested. Mandatory failures are D10/D11/D12 localization/flag and
W5/W6 source-footer scale handling. D9 and C5 pass. Supplemental S2 is not
fully localized in that first run. All 34 validation endings remain visually
complete; most endpoint misses are localization/OCR limitations, not confirmed
terminal losses. Thirty-six grouped validation core events plus development's
56 give 92/2,678 = 3.44 per 100 pages before revisions.

First manifest SHA:
`37e2f24359793b470b058ec91aec14182326a9c7e55e47b916d8d21ce71bd719`;
candidate SHA:
`9a7c9e5662f54f9bb637c9c3d55b829d5eba8aaf5c75d087c71f720f7a259645`;
official first evaluation SHA:
`7667f6550972bec2f69f14b6ac7066a8dda4ce5321dc282f862f81a85eb530f8`;
reviewed-gold retrospective score SHA:
`23a8d9f6781b1b898d48d4e8e0567c1c2b0d94c67fc03fbc39908547b4c45ea6`.

## Independent validation annotations

Annotation-only `runs/step101-validation-gold-overviews/` completed all 138
validation cases at 300 DPI. Independent original-source observations and
reader native images were reviewed before opening detector candidate/debug
results. `runs/step101-validation-gold-preparation/labels-proposed.json`
contains all 325 structurally validated reviewed cases, including all 138
validation cases. All 187 development cases are unchanged in that proposal;
the original sealed proposal is retained separately from subsequently installed working labels.

Retained helpers and native evidence cover four duplicates, 34 endpoints and
their sparse-page negatives, 34 generated-heading negatives, four gaps, six
rule labels, three opening/scrap cases, five spacing negatives, five repaired
negatives, three notices, Otho exclusion and the legitimate chapter marker.
`image-review-ledger.json` records completed image review. Annotation status is
not detector performance. Proposed localized gold rendering completed all 138
cases at 300 DPI in `runs/step101-validation-gold-final/`, with a complete
annotation manifest and empty stderr. The proposed gold SHA is
`664c8ec4303adf05e095b5236570ae0cf9afae738adbf73e1931279e022b94a2`.
The independent evidence inventory hashes 699 PNG files; its SHA is
`ec847d19528472af54713d64f776a3d9c8aec2d078abac4f8cd44304fb94e0d0`.

C5 source 2/624's first Latin line introduces Lucilius. Full reader VIII/127,
128 and 129 show that introduction missing; page 128 resumes the second source
line. W5/W6's gray upper logo fragments remain visible on VIII/87 and VIII/128.
S2's line has upper fragments on VII/244 and its lower remainder on VII/245.
Native evidence and raw observations remain separate from detector results.

R8 is an independently confirmed annotation error: the source has an outer
bar, but current reader IV/48 has no vertical divider. The proposed label is
negative; copied-gap positive M9 remains. Original findings and snapshots are
untouched. `R8-annotation-discrepancy.json` preserves native evidence and the
zero-component check. R11 source 674 and R13 sources 604/624 are contextual
pages without strokes, retained as context provenance outside positive stroke
regions. D12's actual source word breaks `plu-` / `rimum`, forming `plurimum`;
the second copy starts at `rimum`, and total lateral extent remains unconfirmed.
Source Otho 709/710 repeat ending scans; the selected reader retains the ending
once. They do not establish a unique loss.

Annotation measurement helpers corrected shortest-window ties that included
printed-number suffixes after E21/E68 and a short global reference mistaken
for the Caesar chapter anchor. Native images confirm the corrected boxes.
These are independent annotation corrections, with no detector predicate,
confidence or threshold change. All 34 validation endings are visibly complete.

## Fixed page panel

All 107 fixed validation reader page images were viewed on 54 contact sheets
under `runs/step101-validation-panel-images/`; PNG hashes are checked and
`panel-manifest.json` records `reader_image_reviewed` for every page. All 36 first-run core candidates have independent source/reader verdicts in
`step101-validation-first/first-independent-reviews.json`. The 31 omission
warnings comprise 21 confirmed and ten false positives; all four duplicate
passages are confirmed, and the one high-confidence head warning is a false
positive (the final letter of ordinary body word `aiunt.`). Its page VIII/131
is outside the fixed panel. All 107 panel pages and their touching candidates
are reviewed, with zero high-confidence false-positive or uncertain pages.
These results describe the preserved first run; revised candidates require
matching evidence or new review.
Development's 93 reviewed panel pages remain in the immutable 9.2 run.

## Post-validation revisions and current run

The first complete frozen evaluation and independent gold were preserved before
changing the producer or installing annotations. Current live producer SHA:
`748b8684b23d74c09da84f3c7a9170eaf185ca008f3b7522519ee40b13eb6227`.
Working and tracked labels SHA:
`a96a58c6bd924704b7b4e1686cad91851a6f8503880ba182fa836e7354c747c9`.
Do not edit these while the current development process is running.

`runs/step101-development-revised-1/` completed all 34 development units,
1,338 body pages and all detectors with workers 4. Its runtime is 3572.794345
seconds; no processing errors or negative regressions. Official evaluation:
38 hits,25 misses,118 clean negatives,zero pending/violations. All required
D4-D8/C1/C2/C3/W4/H1/H2 remain localized, with the strong core positives high.
S1/C6 remain localized. All 56 core source/output regions and actual crop hashes
match the independently reviewed9.2 evidence, allowing exact verdict transfer;
all 93 panel pages have zero high-confidence false-positive/uncertain pages.

New official complete development freeze SHA:
`77e6d6be418b169b56ff5b52432decd70d6450971cfeb06bf7bc472c1d166fda`.
`runs/step101-validation-revised-1/` is the complete post-validation regression
invocation using that freeze, all 34 validation units/1,340 pages/all detectors,
workers 4. Hidden native PID 33468 and `process-launch.json` identify it.
Retained producer SHA matches the live producer above. At this checkpoint (approximately12:36 UTC),17 unit-completion markers
are logged and Crassus is being processed, with empty stderr. Final validation
metrics remain pending; inspect the live log/manifest when resuming.
Do not edit the live producer or working specs during this invocation.

The general source-footer matcher searches the configured scan-scale bank in
.025 increments, preserving the .80 high threshold and actual reader glyph
support gates. Identity-checked cache entries bind original raster pixels,
source templates, settings, OpenCV and helper code. Source-only measurements
for W5/W6 at scale .925 correlate .9521487/.9519556; exact scale1.0 has no
supported peak. Development template pages287/299/321 retain exact-scale1.0
matches. `source-only-footer-proposal-results.json` retains those measurements.

Duplicate opening-line coverage is recorded even when a genuinely incomplete
later opening has less than .60 overall edge coverage. Boundary refinement
retains the printed one-letter Latin preposition `a` and extends a terminal
line only when its last expected source word was not covered. Actual cached
reader/source replay localizes all D4-D12 and retains D5/D6/D12 incomplete
opening distinctions (`duplicate-boundary-replay.json`).

D11 is Aratus. Its independently prepared raw OCR `ase` box covered printed
`a se`. After opening the first validation failure, native black-ink projection
split that locator at the visible word space. The revised gold marks printed
`a`, preserving raw OCR and the original sealed annotation. Both original
model boundaries still fail this corrected gold; the general short-word fix
is necessary. `D11-printed-opening-word-correction.json` records this
post-validation annotation correction explicitly.

Clipping comparisons add quarter-glyph-height raster context around raw
embedded word boxes and admit two-letter words, including Roman `IV`.
The original word box remains separately recorded. Pixel correlation, crossing
components, outside ink and white-space gates remain unchanged. Actual S2
replay localizes complementary VII/244-245 fragments at low confidence using
supported `IV` correspondence; Exsilii remains below the .65 correlation gate.
`runs/step101-split-line-replay-2/replay-result.json` and native pixel evidence
preserve the measurements. `runs/step101-reader-logo-replay/replay-result.json`
also localizes W5/W6 against actual reader fragments and preserved original
occurrence context; both remain low confidence, with independently confirmed
actual gray glyph crops. These focused replays are not complete validation runs.

## Remaining10.1 work for the next implementor

The revised complete development regression,93-page review and new freeze
are finished. Preserve the original 9.2 freeze and first validation results
unchanged. Finish the current complete revised validation invocation, labeled
post-validation regression. Require all
D4-D12, C1/C2/C3/C5, W4-W6 and H1/H2 correctly localized, with strong development
positives still high. Process all supplemental layout cases, preserving
qualified low-confidence results and unresolved extent. Both halves' 68 endings
have independently been viewed as complete; distinguish algorithmic endpoint
localization misses and OCR limitations from actual terminal loss.

Finish review of all 200 fixed panel pages against revised candidates and the
separate previously unlabeled/low-confidence candidate sample. The first-run
review includes all available 36 core candidates, including known benchmark
cases; describe the eligible new-event cohort separately and do not invent 50
when fewer exist. Candidate-selected precision does not establish a page rate
or corpus recall. Verify combined 2,678-page/68-unit actual coverage, burden
<=5 grouped core events per 100 pages, panel high-confidence FP<=1 per 100,
unchanged live and captured input hashes, and empty processing failures.

Save a dated calibration note recording original and revised performance,
annotation provenance, profile/producer/review identities, runtime and
limitations. Run Python compilation and `git diff --check`. The concrete handoff includes exact remaining10.1 commands and10.2 delivery
checks. After resuming, finish the actual10.1 checks and commit/push that
completed step. Honor the standing pause after 10.1 unless the next user
instruction authorizes continuation. The current checkpoint commit does not
complete 10.1. The main README addition remains scheduled after 10.2 is pushed.

## Work reserved for the later resume

Workers1/4 with the original frozen producer already processed Agesilaus and
Agis (92 pages) in separate `step102-workers1-frozen` and
`step102-workers4-frozen` folders, producing 50 identical normalized candidates
including scores/regions/confidence/crop hashes. Comparison SHA:
`cf8ffd0ae62673b08c4f1cea136da3324d391c6abcd86a426486c05729e968a9`.
The final producer changed; repeat this check for that producer in10.2.

Run all units with chosen final settings in an explicit new/resumable folder;
evaluate all cases, verify resume identity stability and workers 1/4 agreement,
record runtime/storage/dependencies and actual metrics in a dated completion
note, then commit/push10.2. Add and commit/push the main README section about
issues, invocation, reports and methodologies after 10.2 is pushed. An ignored
`README-audit-section-draft.md` is prepared but not installed. Source Latin,
production inputs, repair policy and captured snapshots remain read-only.
