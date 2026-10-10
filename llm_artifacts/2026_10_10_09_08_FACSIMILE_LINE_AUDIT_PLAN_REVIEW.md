# Review of the facsimile source-line coverage and seam audit plan

Reviewed 2026-10-10 on `plan/facsimile-line-coverage-seams`.
Reviewed plan commit: `570749d`.

Plan: [2026_10_10_08_39_FACSIMILE_SOURCE_LINE_COVERAGE_AND_SEAM_AUDIT_PLAN.md](2026_10_10_08_39_FACSIMILE_SOURCE_LINE_COVERAGE_AND_SEAM_AUDIT_PLAN.md).

## Result

The Python implementation, original-source independence, passage-level duplicate
rule, separation of model observations from repairs, and omission/control checks
are appropriate. The plan needs revision before implementation: several reuse
contracts are incompatible with its requirements, and the core line-preservation
algorithm is insufficiently specified. No new unit-test suite is required to
resolve these findings. Keep verification through a small number of complete
source-to-reader cases and saved visual evidence.

This review did not implement the audit, modify the plan, inspect API credentials,
run paid calls, or change production files. It checked the saved plan against
`tools/audit_inner_tomi.py`, the unit table, numeric duplicate records, dependency
pins, and the latest accepted Tier-B delivery note. Algorithm comments below are
review inferences from those contracts, not measurements of a new detector.

## Findings

### R1 — High: the reusable source index is not independent of production exclusions

Plan locations: Design E and step 5.1.
Source: `tools/audit_inner_tomi.py::cached_source_index` (line 997),
`cached_source_regions` (line 973), `extract_source_regions` (line 1388), and
`unit_source_tokens` (line 1591).

`cached_source_index` calls `cached_source_regions`, which loads approved skip
pages, partitions with production body/title fractions, and extracts embedded
source words inside those cuts. `unit_source_tokens` skips pages marked skipped.
Calling this orchestration unchanged defeats the new inventory's independence
and removes duplicate occurrences before logical passage mapping. The plan says
both to reuse that index and not to use its expected-region producer but does not
specify the adapter separating them.

Required revision: name a new inventory-to-token adapter that retains
`occurrence_id`, `logical_line_id`, source page/box, raw OCR provider, and missing
OCR observations. Build the source-only index from those adapted observations
using existing pure `fit_source_index`, `comparison_views`, `passage_windows`,
`anchor_information`, and aligner functions. Do not call `cached_source_index` or
`cached_source_regions` for new expected material. Specify reader-token inputs,
logical alias inputs, and how OCR-free source objects remain in the coverage
queue. One disposable production-trim mutation should leave the source inventory
and the adapted expected-line set unchanged.

### R2 — High: the existing registration helper does not implement edge-aware strip registration

Plan locations: Design E/G and steps 5.1–5.2.
Source: `match_source_image` (line 3116), `missing_region_retention` (line 4309),
and `REGISTRATION` (line 1639).

`match_source_image` extracts source features from `[SOURCE_LATIN_X0, 0.10,
1.0, 0.94]` and fits one affine transform between a source page and a reader body
region. Its minimum is 24 inliers. This is not the planned full-height,
line/strip-specific, piecewise correspondence. Applying it directly to short
verse or individual edge lines can fail; applying a successful whole-page fit to
reflowed pieces does not establish one transform valid for each piece.

Required revision: explicitly extract a shared pixel-registration primitive that
accepts source and reader regions, preserving historical wrapper defaults. Define
how ordered anchors delimit local strips, which retained anchor lines estimate
scale/rotation, how source and reader splits are represented, and what constitutes
a usable transform. Source heads/feet must not be excluded from the new region.
Pin initial search bounds and invalid-transform conditions; calibrate acceptance
values later. Keep failures unresolved. This shares low-level registration logic
without copying the old wrapper or treating its page transform as a new line fit.

### R3 — High: complete/partial/absent decisions lack an executable evidence rule

Plan locations: Design D/E/G and step 5.2.

The plan requires whole glyph preservation but leaves similarity measures,
resampling tolerances, endpoint checks, and acceptance predicates to Phase 7.
There is no operational rule for how to decide complete coverage, how to detect
one missing glyph despite a high aggregate match, or how complementary duplicate
fragments map into one reference coordinate system. Avoiding a universal threshold
is reasonable; omitting the algorithm is not enough for an implementer to proceed.

Required revision: define the region transform and missing-ink measurement, then
spell out all predicates required for machine `complete`. Include expected left/
right ink and above/below-baseline components, uncertainty when resampling changes
small ink components, and the required before/after context. Keep machine evidence
status separate from human adjudication. Give initial settings and calibration
criteria instead of a blanket instruction to choose them later. Define canonical
logical-line coordinates and explicit fragment-to-reference transforms for union
coverage; reject incomplete maps. Include one small-glyph clipping control in the
existing visible-clipping scenario, without adding a unit-test framework.

### R4 — High: the source line-grouping recipe does not specify column separation

Plan locations: Design C/G and step 3.1.

The generous right-side ROI deliberately reaches the gutter and can include Greek
edge ink, divider fragments, annotations, and running furniture. The stated
vertical-overlap rule can put components from both columns on one proposed line.
The plan asks the implementer to estimate the Latin gutter but gives no algorithm
or ownership rule before grouping. The median of all 8–80-pixel component heights
also lacks width/area screening; component chains can connect apparent baselines.

Required revision: specify source-only column/gutter separation before line
assembly, retaining excluded-side/ambiguous components as observations. Define
horizontal grouping constraints, punctuation/diacritic attachment, treatment of
rules touching glyphs, and merged-baseline fallback. Reuse or deliberately adapt
`ink_line_height` rather than quietly adding a competing glyph-scale estimator
(see R9). Make the source overlay, residual unassigned ink, and uncertainty counts
part of inventory verification. The detector must not silently omit uncertain
Latin while producing a nominally complete inventory.

### R5 — High: shared path guards reject the planned fixture output

Plan locations: step 1.1, step 7.1, and the example `verify --out` command.
Source: `safe_write_path` (line 4839), `render_page` (line 4550),
`reusable_entry` (line 5008), and `write_json` (line 4872).

The existing guard permits ordinary writes beneath `work/cache` and `work/runs`,
not `work/verification`. It also rejects ordinary `.pdf` writes. The planned
verification command creates fixture PDFs under a verification directory, so
preserving this helper's contract unchanged cannot support that command.
`render_page` and cache reuse additionally depend on the guard, error classes,
progress callbacks, lock handling, and other helpers; extraction is not just
moving the listed functions.

Required revision: define the shared-helper dependency boundary and output policy.
Keep historical defaults intact. A narrow explicit fixture policy may authorize
new PDFs inside a newly owned verification directory; it must not authorize
writing captured/production PDFs. Alternatively put cases in an approved run root
and keep fixture generation under its own path validation. Provide exact adapters
and ensure the guard checks resolved paths. Check the complete verification CLI
against that layout, not merely helper importability.

### R6 — High: terminal omissions conflict with the two-sided anchor requirement

Plan locations: Design E/F, step 5.2, and `head_and_foot_loss` in step 7.1.

Partial/absent findings require bounded before/after anchors, but a missing first
line of a unit has no preceding body anchor, and a missing last line has no
following body anchor. Sparse one-line units or page fragments are harder still.
The plan prohibits borrowing another unit's prose and gives no endpoint-specific
rule. These cases can remain unresolved even when the planned scenario expects
a correctly located loss candidate.

Required revision: define separate evidence rules for interior gaps, unit
openings, unit endings, and wholly missing sparse passages. Allow a source-reviewed
unit boundary plus one strong body-side anchor and a bounded reader extent;
record weaker evidence rather than forcing the interior rule. Distinguish a
source-page edge inside a unit from an actual unit endpoint. Exercise both true
unit endpoints in the existing head/foot case and include a short passage in the
retained control.

### R7 — Medium: several steps verify features implemented later

Plan locations: steps 2.1–2.2, 3.2, 4.1, and 5.2.

Step 2.1 runs `inventory`, which is introduced in 3.1. Step 2.2 compares line
coverage and seams, introduced in Phases 5 and 6. Source/duplicate adjudication
appears in steps 3.2 and 4.1, while step 6.1 also introduces `adjudicate` without
clarifying which submodes exist earlier. Step 5.2 defers its concrete defect checks
to fixtures introduced in 7.1. This violates the session-sized verification
contract even if the final implementation could work.

Required revision: preserve existing identifiers. Give Phase 2 local adapter
inspection commands and postpone its with/without-coverage comparison to final
verification. Explicitly introduce source and duplicate adjudication submodes in
their respective steps. Introduce the few verification assets before the affected
coverage step, using a suffixed step if needed, or require a saved focused real
case then. Do not renumber the plan or reintroduce unit-test requirements.

### R8 — Medium: runtime identity, CLI settings, and failure semantics are incomplete

Plan locations: Design A/B/G, steps 1.2 and 6.2, and step 7.2.

The plan snapshots audit code but does not explicitly bind execution to the bytes
of all imported helpers, notably the historical audit and extracted common module.
A run can otherwise use edited live code with old cache identity. OCR language,
PSM profiles, Poppler/Tesseract paths and prerequisites, work-directory derivation,
policy-version inputs, and run failure/resume exit behavior are also not pinned.
The old code already provides prerequisite checks and content-addressed cache
patterns. Haiku's `delivery_unknown` state has no specified CLI selector for the
new explicit attempt mentioned in the plan.

Required revision: enumerate transitively executed source files and hash their
actual bytes before any comparison-cache access. Record executable versions and
OCR language/model/settings, preferably preserving the existing `eng` profile as
a starting configuration. Define explicit settings/policy paths, default work
root, partial-run completion status, and failure codes. Specify how updated
policies start a new run while reusing raw renders. Add a concrete task selector
and attempt flag for unknown Haiku delivery, keeping default resume non-submitting.
No special approval flow or new credential setup is required.

### R9 — Medium: reuse and verification need to avoid parallel algorithms and unrealistic fixtures

Plan locations: Design G, steps 3.1 and 5.1, and step 7.1.
Source: `ink_line_height` (line 2379), `fit_source_index` (line 1548),
`find_reader_occurrences` (line 2000), and `chain_matches` (line 2068).

The plan supplies a new glyph-height estimator while an existing helper already
rejects elongated rules and estimates typography scale. It also calls broadly
for reuse of the matcher without naming the shared primitives and adapter shapes.
An implementer could accidentally create a second estimator, aligner, or evidence
path, or reuse a wrapper with the wrong policy. The seven verification cases
specify distinct ink but do not say how these assets produce usable OCR anchors;
nontext patterns may cause the normal TF-IDF/OCR path to be empty and all cases to
remain unresolved. The helper count and optional trace phase should not expand
into a second audit platform.

Required revision: list exact pure functions shared across callers and name the
new adapters. Parameterize the existing scale-estimation primitive if independent
inventory needs different thresholds, preserving the old defaults. Use small
legible text fixtures and enough distinctive surrounding context for normal OCR
alignment, or explicitly choose an already-supported image-anchor path with its
actual required inputs. Keep expected boxes independent of detector output.
Retain seven compact end-to-end cases; no new unit tests or mock framework.
Optional provenance adaptation can be deferred until a finding needs it, while
preserving Phase 2 identifiers and making its optional status explicit.

## What is consistent with current patterns

- Python dependencies, original-source observations, bounded reader localization,
  content-addressed renders/OCR, explicit uncertainty, hashed crops, and review
  reports follow the existing audit's direction.
- Source independence is the correct requirement; the reuse adapters need to
  protect it rather than adopting historical production-derived cuts.
- Passage-level duplicates, distinct physical/logical denominators, partial scans,
  and shared-unit ownership match the measured source behavior. The approved
  numeric records are useful seeds, not interchangeable complete line maps.
- Existing code/policy and accepted readers remain untouched; Haiku observations
  cannot automatically activate a repair.
- The unit-test exemption is coherent with the user's instruction. Small complete
  audit scenarios and visual controls provide the relevant verification.
- Phase/step identifiers, implementer suggestions, required opening sections,
  and per-step LLM/human verification are present. Historical artifact retention
  and the unrelated working-tree edit are explicitly preserved.

## Recommendation

Revise R1–R6 before implementation. Resolve R7–R9 in the same in-place plan pass,
keeping all existing phase and step identifiers. Keep the Python-only scope and
no-new-unit-test instruction. The next pass should make the source adapter,
registration primitive, evidence predicates, output policy, and endpoint rules
concrete; it should not increase framework scope or require new production traces.
