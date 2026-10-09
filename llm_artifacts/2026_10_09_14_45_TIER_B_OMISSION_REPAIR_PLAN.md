# Context

The inner-tomi audit confirmed 59 unique-loss omissions. The
[2026-10-08 repair pass](2026_10_08_16_00_INNER_TOMI_AUDIT_REPAIRS_PLAN.md) restored the
39 initial Tier-A events / 69 lines. The remaining 20 initial Tier-B events / 44 lines
have confirming human reads and promotion decisions in the
[promotion worksheet](2026_10_09_10_45_TIER_B_PROMOTION_REVIEW_WORKSHEET.md).
Promotion authorizes investigation and repair; it does not establish restoration.

The [band extraction](2026_10_09_14_30_TIER_B_BAND_EXTRACTION.md) supplies 21 installable
protection records for the 19 non-Antonius events / 21 lines: 17 clean records, three
Latin sub-ranges below merged running-head fragments, and one whitespace-tightened
Phocion record. These use the existing edition-specific SOURCE_BODY_BANDS mechanism.
Antonius's remaining event accounts for 23 lines. Its plain single-band test-install
failed the reader check, as recorded in the
[Antonius findings](2026_10_09_15_30_ANTONIUS_BLOCK_COMPOSITION_FINDINGS.md); its loss
stage and safe composition method must be established separately.

Phases 1 and 2 may proceed after Phase 0 without waiting for Antonius. Phase 3 has its
own evidence and real-eyes gate. A partial delivery must identify Antonius as
unresolved; full completion requires all 20 events / 44 lines restored.

All Latin is source material. Preserve original source pixels, spelling, punctuation,
indentation, chapter numbers, and approved in-block annotations. Never retype or
normalize body prose. Work continues on branch plan/inner-tomi-audit-repairs.
Changes are scoped by source identity and measured signatures, but adding bands can
change unit-wide metrics, composition, downstream pagination, folios, and contents
references. Source-page scope does not imply output-page scope.

# Current state of the world

- src/facsimile/reflow.rs:
  - SOURCE_BODY_BANDS has 47 installed Tier-A protection records. SourceBodyBand holds
    source, page, unit_id, dimensions, observed, and protected. At BODY_DPI = 300,
    dimensions is the cropped column's width and full page height; bands are half-open
    pixel-row intervals in that column.
  - retain_known_source_body_bands starts with kept, adds matching protected intervals,
    and merges overlapping or touching intervals. It matches observed or protected
    by exact equality. It runs after Pass-A furniture filtering with owner: None and
    after Pass-B unit trims with owner: Some(unit_id). It does not remove a parent
    observed interval that remains in kept.
  - render_and_band masks known furniture, detects the column, applies
    preserve_known_body_crop_edge, crops, bands, and protects. A trace must distinguish
    raw detected bounds from this production crop.
  - coalesce_known_body_fragments has five reviewed fragment-pair cases;
    preserve_known_body_crop_edge has five cases. The non-Antonius repairs need no
    changes to these helpers. Antonius may require a crop correction or an exact
    is_known_whole_source_strip recognition using the established fingerprint pattern.
  - compose_body_canvases packs scaled strips. lowest_ink_row_in selects the least-ink
    row in any nonempty window even if every row contains ink. Splitting retains the
    piece and remainder; retain_body_canvases subsequently applies an ink floor.
    Lack of blank rows can make cuts unsafe, but does not by itself explain total loss.
- src/facsimile/audit_repair_tests.rs:
  - known_source_body_bands_survive_furniture_filters assumes each reviewed interval
    is dropped on a synthetic column with scaffold bands at rows 1000..2000. That
    assumption fails for Antonius's tall observed interval and needs a grouped fixture.
  - assert_tier_a_unit_lines_restored already selects installed/restored events by
    unit, uses audit_traced_unit and assert_reviewed_source_lines_survive, and writes
    source traces, line coverage, and canvas hashes. Reuse its implementation.
  - audit_traced_unit records physical source identities after duplicate removal and
    the production crop. assert_reviewed_source_lines_survive can require each reviewed
    glyph rectangle to occur wholly on one output canvas with whole_line: true.
  - required_audit_references, project_audit_evidence, export_portable_fixture, and
    the semantic validators implement the existing evidence lifecycle.
    validate_omission_diagnosis_data currently requires every initial Tier-B event to
    remain deferred; validate_tier_a_completion_data requires promoted events to be
    restored. Both require deliberate updates for staged delivery.
- Fixtures:
  - src/facsimile/fixtures/inner_tomi_audit_repair_source_trace.json still marks all
    20 initial Tier-B events deferred and awaiting a confirming read.
  - src/facsimile/fixtures/inner_tomi_portable_audit_evidence.json is the tracked,
    compact projection of authenticated source documents.
  - src/facsimile/fixtures/closing_body_bands.json records source, page, dpi, column,
    body, excluded, and noise. Existing records cover seven closing/boundary pages.
- traced_real_unit in reflow.rs is a helper, not a test. A Cargo filter bearing that
  name executes no test. It currently uses raw detector bounds for pixel comparison,
  so it must use the production crop before validating a crop-adjusted Antonius page.
- extract_tier_b_source_bands is an ignored diagnostic, not a restoration regression.
  It saves crops but does not yet emit the absolute production column bounds required
  for fixture construction.
- Build command:
  cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2
  Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
  A restricted --unit build derives its scale from the selected unit and is not the
  final full-reader proof. Without callable LuaLaTeX, the generator can succeed with
  TeX output alone. The Antonius findings report that build_facsimile holds trial
  output; do not treat those PDFs as the current source configuration.

# Desired end state

1. The 19 non-Antonius events / 21 lines are restored through 21 new protection records.
   Antonius uses the measured, verified method selected in Phase 3. Its explicit-band
   fallback would add 23 records, for 44 new records in total; other methods have a
   different count. Restoration evidence determines success, not a fixed table size.
2. closing_body_bands.json records exact retained bands and excluded furniture under
   the production crop. The existing audit ledger records individual reviewed glyph
   rectangles, source order, promotion provenance, status, and authenticated evidence.
   Named tests prove original pixels, unique retention, actual strip contribution,
   safe composition, and exclusion of the three head fragments.
3. All eight inner PDFs are freshly compiled offline. In a partial delivery, the 19
   non-Antonius seams pass and Antonius is explicitly unresolved. In a complete
   delivery, all 20 seams pass. Split-word continuations become continuous in reading
   order while the original printed hyphens and line layout remain source-faithful.
4. Antonius is declared restored only when all 23 reviewed lines survive in order,
   individually legible, with whole glyphs on returned canvases and in the rebuilt
   tomus 7. A verified whole-block method is acceptable; forced per-line splitting
   is not an end-state requirement.
5. No general banding/furniture thresholds, CLI, dependency, or global-scale algorithm
   changes. Raw render caches, source PDFs, and transcription caches remain untouched.
   Edition-specific data/crop/fingerprint cases and shared test helpers may change.
   cargo fmt --all, cargo fmt --all --check,
   cargo clippy --all-targets -- -D warnings, and cargo test pass locally.
6. A durable verification note and machine-readable delivery state agree:
   partial = 19 restored events / 21 lines and Antonius unresolved (1 event / 23 lines);
   complete = 20 restored events / 44 lines and no unresolved event.
   The record includes baseline and rebuilt seam locations, artifact decisions,
   output freshness, source/canvas evidence, and downstream pagination checks.

# Tech Learnings

- observed identifies detector evidence; protected identifies retained source rows.
  The three merged-head entries and the Phocion tightening have protected.top greater
  than observed.top. Multiple records may share observed. Separate protected intervals
  require strict positive gaps because touching bands merge; a surviving parent merges
  the entire group back into one interval.
- cut_points assigns surrounding half-gaps to strips. Gaps in band metadata do not
  remove pixels from those strips or manufacture safe source whitespace. On skewed
  text, line pitch alone cannot establish a glyph-safe horizontal cut.
- compose_pages packs existing raster layout and cuts at page boundaries; it does not
  reconstruct individual lines. A known whole strip that fits a fresh canvas can use
  the established exact-signature/fingerprint exception to avoid fill-splitting.
- closing_body_bands.json proves retained band pixels when a callable test reads it.
  ReviewedSourceLine rectangles and whole_line composition checks supply the additional
  proof that each restored line reaches the output intact. Use source-page traces rather
  than page-minus-start indexing after duplicate removal.

# Phase 0 — Prepare verification and staged audit state (Suggested implementer: Opus)

Complete both steps before installing the non-Antonius repairs. Each step must leave
ordinary tests green. Keep existing numbered phases and steps intact.

## 0.1 Make source and composition verification callable

Touch src/facsimile/reflow.rs and src/facsimile/audit_repair_tests.rs.

Extend extract_tier_b_source_bands to write
tmp/pdfs/inner_tomi_audit_repairs/tier_b_measurements.json and source overlays.
For all 21 distinct source pages, record unit_id, source, page, dpi, source raster
dimensions and SHA-256, raw_detected_column (original unmasked raster),
masked_detected_column (after furniture masking, before edge correction),
production_column: [left, right] (after edge correction), column dimensions,
raw bands, furniture-filtered bands before protection, Pass-A
bands after protection, and final Pass-B bands. Use the existing mask/crop helpers
in their production order; do not infer an origin from width/height. Add full-source
glyph rectangles and neighboring line rectangles to the measured event data.
Preserve this unrepaired baseline before installing entries.

Add the ignored capture_tier_b_full_work_baseline test. Before any Tier-B production
entry, crop, or fingerprint is installed, use the complete operational unit table,
audit_traced_unit, production_body_scale, and compose_pages to capture every unit.
Do not use the existing trial PDFs as the baseline. Write fresh unit JSON and a
manifest under tmp/pdfs/inner_tomi_audit_repairs/tier-b-baseline/; preserve the exact
bytes under archive/tier_b_omission_repairs_2026_10_09/baseline/full-work/.
The manifest must contain schema_version: 1, the full-work scale, source PDF and unit
table digests, digests of the relevant production Rust files, and one ordered entry
per operational unit. Each entry contains unit_id, source, its trace reference
{ path, sha256 }, metrics, canvas_count, and canvas_sequence_sha256: the SHA-256 of
serde_json::to_vec applied to its ordered canvas hash/dimension records. Keep the
full ordered records in the unit trace and verify that they reproduce this digest.
Each unit trace records the requested and retained physical source-page sequences,
production crop bounds, retained bands, retained-column pixel hashes, metrics, and
composed canvas hashes/dimensions. Generate the row/band/image measurements using the
existing pipeline and report serializers, not a second extractor or packer.

Bind the manifest to tier_b_baseline: { path, sha256 } in the audit ledger in Step 0.2.
Reject an already-repaired configuration, omitted/duplicated/reordered units, source
or configuration mismatch, failed source renders, and missing trace references.
Archive once and do not overwrite this pre-installation baseline on later diagnostic
runs. Tests operating in memory may validate synthetic bad manifests; only this
ignored test requires source rasters. Add ordinary
tier_b_baseline_manifest_rejects_missing_units_and_configuration_mismatch.
The baseline is a comparison reference, not a claim that the omitted lines survive.

Extract a test-only assert_reviewed_band_ledger helper from traced_real_unit's ledger
loop and make it use audit_traced_unit's physical identities and production crops.
Call it from traced_real_unit and the shared restoration path. Compare original
source pixels inside the production crop; assert sorted non-overlapping body entries,
exact retained-band multiplicity, and no strip overlap with excluded entries.
Support empty excluded/noise arrays. Keep special shared-unit boundary semantics.
Reuse the production crop computation rather than copying a second detector pipeline.

Rename assert_tier_a_unit_lines_restored to assert_unit_lines_restored at all call sites.
Keep its historical behavior and archived report format; add reusable explicit-event
selection, output-stage, and candidate-verification parameters. Add the test-only
ReviewedEvidencePolicy enum with HistoricalTierA and PromotedTierB variants.
Use HistoricalTierA for existing Tier-A/extra-item regions and their historical
neighbors; use PromotedTierB for each newly measured individual Tier-B line.
Existing call sites retain their installed/restored-only selection and the historical
policy. Apply the policy per reviewed region in a mixed unit rather than turning
whole_line on for every region imported into that unit. Reuse one source/composition
pass and the common pixel/coverage assertions; do not copy them for each policy.
Historical regions may describe several lines spanning canvases; the new single-line
policy requires whole_line: true. Fresh promoted reports use the versioned contract
in Step 0.2, while frozen historical reports remain unchanged.
Candidate stage tests may explicitly include pending_repair events whose code/data
are under test; they must actually prove restoration before status advances. New selectors
group selected events by unit and run extraction once per unit. They must include
existing installed/restored lines and neighbors in those units, retain historical seam
checks, and count event lines separately from additional neighbor rectangles.

Add named ignored tests, backed by that shared implementation:

| Test | Exact newly repaired selection / required event and line count |
| --- | --- |
| tier_b_head_source_lines_restored | Step 1.1's eight events / nine lines |
| tier_b_foot_source_lines_restored | Step 1.2's five events / five lines |
| tier_b_verse_source_lines_restored | Step 1.3's two events / three lines |
| tier_b_merged_source_lines_restored | Step 2.1's four events / four lines |
| tier_b_non_antonius_source_lines_restored | All 19 non-Antonius events / 21 lines |
| antonius_reviewed_lines_survive_composition_whole | Antonius's one event / 23 lines |
| tier_b_source_lines_restored | Declared partial or complete selection: 19/21 or 20/44 |

A selector must reject a missing event, wrong count, missing reviewed rectangle, or
empty selection. The full delivery selector additionally rejects pending or installed
selected statuses and in_progress delivery state. Candidate stage tests run after
protection data is added and before marking the events installed; their fresh passing
reports supply that transition's evidence. Until then the ledger remains in_progress.
Use --ignored explicitly, check that the named test ran, and retain failures.
Do not claim a passing test from zero selected tests or an ignored result.

Refactor known_source_body_bands_survive_furniture_filters to test groups sharing
(source, page, unit_id, dimensions, observed). Preserve ordinary small-band furniture
examples, ownership rejection, dimension/source/page mismatch, exact-observed mismatch,
idempotence, and source-pixel checks. Separate forced removal from already-retained
idempotence; place scaffold bands outside the reviewed interval. Assert every expected
protected interval once for a group. Add ordinary
known_source_body_band_groups_preserve_subranges_and_parent_semantics: strict-gap
sub-bands stay separate when the parent is absent, touching sub-bands merge, and a
kept parent absorbs its sub-bands. Add
tier_b_restoration_selection_rejects_missing_events_and_wrong_counts and
reviewed_band_ledger_uses_production_crop_bounds as ordinary negative regressions.
Add tier_b_validation_preserves_historical_tier_a_evidence covering both existing
historical reports and a mixed-policy unit whose historical multi-line region spans
two canvases while its new single-line region must remain whole.

**LLM verification:** Run cargo fmt --all; cargo fmt --all --check;
cargo clippy --all-targets -- -D warnings; cargo test known_source_body_band;
cargo test tier_b_restoration_selection; cargo test reviewed_band_ledger;
cargo test --release --lib extract_tier_b_source_bands -- --ignored --nocapture;
cargo test --release --lib capture_tier_b_full_work_baseline -- --ignored --nocapture;
cargo test tier_b_baseline_manifest;
cargo test tier_b_validation_preserves_historical_tier_a_evidence;
cargo test --release --lib tier_b_ -- --ignored --list. Inspect the list and
measurement JSON: all 20 event identities and 21 source pages are represented.
Run cargo test audit_tier_a_completion_requires_all_lines and existing portable_audit
tests to confirm the helper changes leave historical verification green.

**Human verification:** Compare the raw and production column overlays, the reviewed
glyph rectangles, verse indentation, and neighboring glyphs with the original crops.
The three proposed head exclusion intervals are candidates until their overlays are
confirmed. Inspect the actual saved measurement file, not an inferred coordinate.
Read the full-work baseline manifest: it must name every operational unit, the safe
pre-installation configuration, and newly measured body hashes rather than trial PDFs.

## 0.2 Record promotions and support partial delivery without weakening evidence

Touch src/facsimile/audit_repair_tests.rs,
src/facsimile/fixtures/inner_tomi_audit_repair_source_trace.json,
src/facsimile/fixtures/inner_tomi_portable_audit_evidence.json,
tools/README-inner-audit.md, tools/check_portable_tests.py, and
tools/test_check_portable_tests.py. Use existing event IDs. Preserve initial_tier: "B",
historical verdicts, candidate/review hashes, source_regions, frozen archives, and the
initial Tier-A inventory of 39 events / 69 lines.

For each of the 20 events, set promoted_tier: "A", status: "pending_repair", and
repair_read_status to the worksheet's confirming-read decision. Add promotion with
worksheet path, event section, and recorded sign-off provenance. Populate diagnosis
using Step 0.1's measured glyph rectangles, neighboring rectangles, baseline trace,
source overlay identity/origin, first-loss-stage measurements, landmarks, and the
named regression. Do not infer glyph coordinates from normalized audit boxes.
Antonius's baseline may say the first downstream loss stage is not yet established;
do not fabricate a cause or passing report.

Add a tier_b_delivery object with state: "in_progress", restored_event_count,
restored_line_count, and unresolved_event_ids. Derive counts from event statuses;
do not use declared counts as their own proof. Legal promoted event states are
pending_repair, installed, restored, and unresolved. installed denotes measured
code installed with machine passing evidence; restored additionally requires the
recorded visual review. unresolved requires a concrete reason and authenticated
disposition evidence under the contract below. A pending event has no passing report;
an installed event has machine evidence but does not yet claim visual completion.
Update all new-event states and delivery counts atomically with corresponding evidence.
Record Step 0.1's tier_b_baseline manifest reference and authenticate its unit traces
before using the baseline in any comparison.

Update validate_omission_diagnosis_data to retain deferred handling for unpromoted
historical Tier-B data and validate actual promoted diagnoses. Keep initial Tier-A
counts separate from initial Tier-B counts (20 events / 44 lines). A pending or
unresolved event cannot provide a restoration claim. Extract
validate_restored_event_evidence from validate_tier_a_completion_data's per-event
checks and give it an explicit ReviewedEvidencePolicy parameter. Share unit/event
identity, source-region membership, source-evidence identity, and line-count checks.
HistoricalTierA preserves the current evidence requirements for the original 39/69
and additional source items: no new human_review field or one-canvas requirement is
imposed on their frozen records. PromotedTierB additionally requires the versioned
line-coverage evidence and human_review reference specified below. Do not retrofit
claims into old reports or weaken historical validations to make them fit new rules.
Add tier_b_validation_preserves_historical_tier_a_evidence to the Step 0.2 checks.
Move current promoted-delivery completeness into validate_tier_b_repair_state_data:

- in_progress permits any explicit mixture of pending_repair, installed, restored,
  and unresolved events and is never a publishable completion claim. Validate every
  restored event under its policy even before delivery is declared; derive the
  in-progress restored counts from those verified event records;
- partial requires exactly 19 non-Antonius events / 21 lines restored and exactly
  miss_antonius_562_20869 unresolved with its 23 lines;
- complete requires all 20 events / 44 lines restored, including the 23 individual
  Antonius rectangles, and an empty unresolved list.

Use the shared validator with the correct policy for every restored event.
For promoted events, reject invented counts, omitted events, duplicate required line
regions, missing human_review references, and unresolved events declared complete.
An installed promoted event must pass the same machine-evidence checks as restored,
but human_review becomes mandatory only at the restored transition.

Define the promoted restoration report as schema_version: 2 with
verification_policy: "promoted_tier_b". Require unit_id, events, reviewed_lines,
scale, line_coverage, canvas_ranges, canvas_hashes, and source_pages. Keep the existing
producer's representations: a coverage entry contains source, source_page,
region: [left, top, right, bottom], scaled_row_stream: [lo, hi], and canvas_owners;
canvas_ranges contains [canvas_number, start, end] tuples in the concatenated scaled
row stream; canvas_hashes contains canvas, width, height, and sha256.
source_pages retains source, page, retained_index, crop, bands, and deleted_rows.
The report's events may include historical neighbors' events, but event/line delivery
counts include only the selected initial Tier-B events. Match policy to required
regions, not to every rectangle merely appearing in a shared unit report.

For every required promoted line, validate exactly one matching reviewed_lines record
and exactly one line_coverage record keyed by (source, source_page, region).
Require a nonempty scaled interval, exactly one valid canvas owner, complete interval
containment in that owner's canvas range, and no competing range that overlaps it.
Require unique valid canvas identifiers, consistent hash/dimension records, ordered
non-overlapping stream ranges, and corresponding retained source identity/crop/band
coverage. Reject missing/extra owners, wrong-region coverage, duplicate required
coverage, gaps, out-of-range owners, and required lines spanning canvases.
Use the same machine policy for candidate/installed and restored reports. The restored
event's human_review identifies the dated verification note and event entry; it must
refer to the configuration/report being accepted. Legacy historical multi-line
regions retain HistoricalTierA checks when included in these fresh reports.

Extend project_audit_evidence's existing Restoration branch using schema_version and
verification_policy: legacy reports retain the existing unit_id/events/reviewed_lines
projection; version-2 promoted reports retain all fields listed above, projecting
source_pages to its listed semantic fields for the source/page pairs referenced by
reviewed_lines, in source order. Validate exactly one retained source record for
every required pair; keep full source-page traces in the authenticated raw reports.
This deterministic filtering keeps ordinary-test fixtures compact without dropping
any source record needed to check reviewed coverage. Missing modern fields are errors,
not a fallback to legacy mode. Require the version-2 policy for initial Tier-B installed/
restored events so deleting a version/policy field cannot downgrade their checks.
Use the same projector and loader for both policies. Store no raster payloads and keep
portable_audit_evidence_schema_and_size_are_valid's existing size limit.

For unresolved promoted events, require disposition_evidence: { path, sha256 } and
disposition_trace: { path, sha256 } in the ledger, distinct from passing_evidence.
Archive the disposition JSON with schema_version: 1, event_id, unit_id,
status: "unresolved", reason, tested_configuration (source/unit-table/production-file
digests), failure_stage (or "not_established" with the remaining uncertainty), and
trace_evidence: { path, sha256 }. Its trace reference must equal disposition_trace.
The trace JSON contains schema_version: 1, event_id, unit_id, tested_configuration,
and the measured stage records from Step 3.1. In partial delivery, its event must be
miss_antonius_562_20869; authenticate both documents and verify their event, unit,
configuration, and reason agree with the ledger. A failed or incomplete investigation
must record what remains unknown, without claiming missing source pixels as proven.

Add AuditDocumentKind::Disposition, AuditDocumentKind::DispositionTrace, and
AuditDocumentKind::FullWorkBaseline to the existing reference/projection machinery.
Project the disposition fields above and the trace's identity/configuration/stage
records; the full-work baseline projection retains the manifest fields and ordered
unit summaries/references, not the large unit trace bodies. Authenticate baseline
trace bytes through the existing evidence-root helpers when performing source
comparisons. Extend the existing audit_evidence_ tests to cover these new kinds and
stream-verify every referenced baseline unit trace; do not introduce another loader.
Missing files, malformed references, digest mismatches, or wrong event/configuration
must fail, even when a delivery is partial. Ordinary tests validate the compact
semantic projections and require no archives or rasters.

Add ordinary tests tier_b_restoration_projection_requires_coverage_fields,
tier_b_coverage_rejects_missing_wrong_region_and_invalid_owners,
tier_b_coverage_rejects_duplicate_and_split_lines,
tier_b_delivery_rejects_missing_unresolved_evidence, and
tier_b_disposition_rejects_wrong_event_and_digest_mismatch.
Test that the in_progress state accepts verified restored events mixed with pending,
installed, or unresolved events, yet rejects false restored counts.

Update required_audit_references to require diagnosis baseline traces for every
promoted event, the tier_b_baseline manifest, disposition and trace references for
unresolved events, and passing reports for installed/restored events. Missing required
fields or documents are errors; do not skip a reference because its file is absent.
Pending/unresolved states must never be forced to invent a passing report. Preserve
failed-trial report/trace reference pairs after a later successful repair in a
historical_dispositions array. Authenticate each retained pair through the same
Disposition/DispositionTrace machinery, using its tested historical configuration
and reason rather than requiring agreement with the current restored status.
They are historical evidence and never current passing evidence. Keep original Tier-A
post-dedup/post-mask references unchanged. Extend the existing projections and
authentication tests for the new promoted documents and delivery checks; do not add
a second evidence loader. Archive new baseline/restoration JSON and overlays in
archive/tier_b_omission_repairs_2026_10_09/<stage>/<unit>/ using existing digest/reference
helpers. Never overwrite the previous repair archives.

Add ordinary tests tier_b_promotions_preserve_initial_inventory,
tier_b_delivery_accepts_partial_and_complete,
tier_b_delivery_rejects_missing_lines_and_evidence, and
tier_b_delivery_rejects_unresolved_complete_claim. Extend
portable_audit_evidence_has_every_required_document and
portable_audit_projection_is_deterministic for staged promoted references.
Add all new ordinary baseline, compatibility, coverage, and disposition validation
tests from Steps 0.1 and 0.2 to tools/check_portable_tests.py REQUIRED_TESTS and update
its Python test expectations in tools/test_check_portable_tests.py.

Refresh the portable projection through export_portable_fixture, first exporting to
tmp/tier-b-portable-evidence-review.json via PLUTARCH_PORTABLE_FIXTURE_OUT. Compare
it with the tracked fixture, then install the reviewed projection. Use the same
export/compare/install procedure after every status/evidence change in later steps.
For delivery evidence, save fresh reports in a new stage directory, archive their
exact bytes, and update path/SHA-256 references; do not modify frozen report bytes.
Keep the archive/overlay identities in the ledger; the compact portable fixture
contains semantic JSON projections and hashes, never raster payloads.

**LLM verification:** Run cargo fmt --all; cargo fmt --all --check;
cargo clippy --all-targets -- -D warnings; cargo test tier_b_delivery;
cargo test tier_b_promotions; cargo test tier_b_validation;
cargo test tier_b_restoration_projection; cargo test tier_b_coverage;
cargo test tier_b_disposition; cargo test tier_b_baseline_manifest;
cargo test audit_omission;
cargo test audit_tier_a_completion_requires_all_lines; cargo test portable_audit;
cargo test portable_evidence_document. Set PLUTARCH_PORTABLE_FIXTURE_OUT to the
absolute temporary review path, run cargo test --lib
facsimile::reflow::tests::export_portable_fixture -- --ignored --exact --nocapture,
review/install its output, and clear that task-specific variable.
Run cargo test --lib audit_evidence_ -- --ignored --nocapture --test-threads=1 with
the authenticated archives available. Run python -m unittest discover -s tools
-p test_check_portable_tests.py and python tools/check_portable_tests.py --offline
--log tmp/tier-b-portable-tests.log after adding the required test names.

**Human verification:** Read several promoted records, including Antonius, Pericles,
and Dio p580, against the worksheet. Confirm historical initial tiers are unchanged
and no pending/unresolved repair is described as restored. Inspect the portable diff:
legacy reports retain their original contract, modern reports contain coverage fields,
and an unresolved disposition has authenticated report and trace identities.

# Phase 1 — Install the clean single-line entries (Suggested implementer: Sonnet)

Install 17 clean records with protected = observed. For each step, retain the
Step 0.1 baseline and use the shared report/export lifecycle from Step 0.2.
Status advances to installed only after the machine checks; final restored status
is recorded after the rebuilt-reader review.

## 1.1 Head-region clean entries

Append the extraction tuples for aemilius_paulus p326, caius_marius p519, crassus p44,
lucullus p631, marcellus p377, nicias p25, timoleon p318, and both lysander p535 lines.
This is eight events / nine lines.

Add the protected intervals to closing_body_bands.json. Obtain absolute production
column bounds from tier_b_measurements.json; dimensions give no horizontal origin.
Create missing records with source, page, dpi: 300, column: [left, right], sorted
body intervals, excluded: [], and noise: []. Extend existing page records without
duplicating them or losing their prior body/exclusion assertions. Keep baseline and
candidate intervals distinguishable in diagnostic evidence. Populate the event
diagnoses and passing report references, then refresh the portable projection.

**LLM verification:** Run cargo fmt --all; cargo fmt --all --check;
cargo clippy --all-targets -- -D warnings; cargo test known_source_body_band;
cargo test --release --lib tier_b_head_source_lines_restored -- --ignored --nocapture;
cargo test tier_b_delivery; cargo test portable_audit. Confirm the executed stage
test reports eight selected new events / nine lines and checks their glyphs on actual
returned canvases. Existing installed/restored neighbors in each unit must also pass.

**Human verification:** Read the saved source/canvas overlays for one restored head line
and both Lysander lines. Defer the complete reader-seam review to Step 4.2.

## 1.2 Foot-region clean entries

Append agesilaus p110's foot entry (keep its existing head entry), artoxerxes p633,
cato_minor p352, dio p566, and dio p580: five events / five lines. Update sorted
ledger intervals, installed statuses, measured reports, and the portable projection.

For Dio p580, inspect the full-source skewed line and its composed glyphs. If padding
is needed, keep observed fixed at 2550..2597, measure protected.bottom rather than
guessing it, and update the ledger's exact retained interval and glyph/neighbor
evidence. Account for any merging with adjacent bands. The event-specific preference
for extra source space over clipping does not waive source-pixel, neighbor, or
duplicate-coverage checks; record any accepted adjacent artifact explicitly.
Any adjustment during Step 4.2 requires regeneration, these checks, and the final
Step 4.1 gate again before declaring delivery.

**LLM verification:** Run cargo fmt --all; cargo fmt --all --check;
cargo clippy --all-targets -- -D warnings; cargo test known_source_body_band;
cargo test --release --lib tier_b_foot_source_lines_restored -- --ignored --nocapture;
cargo test tier_b_delivery; cargo test portable_audit. Require five events / five lines
and unmodified neighboring glyphs. Repeat after any padding change.

**Human verification:** Inspect Dio p580's ascenders/descenders and adjacent lines on
the source and composed crop. Confirm Agesilaus retains both its head and foot repairs.

## 1.3 Verse-line clean entries

Append alcibiades p242 and pericles p214/p215: two events / three lines. Preserve
source indentation and punctuation exactly; do not resolve differences between
worksheet paraphrases and the printed verse by rewriting source text.
Update the band ledger, installed statuses, reports, and portable projection.

**LLM verification:** Run cargo fmt --all; cargo fmt --all --check;
cargo clippy --all-targets -- -D warnings; cargo test known_source_body_band;
cargo test --release --lib tier_b_verse_source_lines_restored -- --ignored --nocapture;
cargo test tier_b_delivery; cargo test portable_audit. Require two events / three lines,
correct source order across Pericles p214/p215, and retained verse indentation.

**Human verification:** Compare both verse passages with the source, including their
indentation and the surviving next verse line. Complete the reader review in Step 4.2.

# Phase 2 — Install the merged-head entries (Suggested implementer: Opus)

Install three Latin sub-ranges below merged running-head fragments (brutus p619,
cato_minor p329, demosthenes p445), plus Phocion p291's whitespace tightening.
All four have protected.top > observed.top. Phocion has no reviewed head fragment
to exclude.

## 2.1 Append the four entries with sub-range protected

Append the four measured tuples from the extraction. Add exact protected intervals
to body. The candidate head exclusions are brutus [129,158], cato_minor p329
[114,142], and demosthenes [127,156]. Their provenance is the repair-plan proposal;
the original extraction note did not record these groups. Step 0.1 must remeasure
them from the source row profile, save source overlays, and confirm the furniture
identity and separation from Latin before installation. If a coordinate differs,
record the new measured interval and reason in the extraction note and ledger.
Do not describe an unverified candidate as a frozen measurement.

For Phocion, add its protected body interval and verify the removed upper portion
contains no body glyph. Update installed states, reports, and portable evidence.
This step completes the 19 non-Antonius events / 21 lines at the machine-check stage.

**LLM verification:** Run cargo fmt --all; cargo fmt --all --check;
cargo clippy --all-targets -- -D warnings; cargo test known_source_body_band;
cargo test --release --lib tier_b_merged_source_lines_restored -- --ignored --nocapture;
cargo test --release --lib tier_b_non_antonius_source_lines_restored -- --ignored
--nocapture; cargo test tier_b_delivery; cargo test portable_audit.
Require four merged/tightened events / four lines, then 19 events / 21 lines.
Assert no strip overlaps any confirmed excluded head interval.

**Human verification:** Inspect all three head exclusions and the Latin immediately
below them. Confirm no Greek/head scrap or extra blank gap appears above the repaired
lines, and Phocion's complete first line remains intact.

# Phase 3 — Diagnose and restore Antonius (Suggested implementer: Opus)

The plain single-band trial failed the reader check. The reported detector band is
249..1356 in an (815, 2835) column, with a reported strip interval 249..1364.
The absence of broad blank runs and the approximately 48-pixel pitch are observations,
not a proven explanation of total disappearance or a safe-cut specification.
Preserve the confirming human read and the failed-trial record. Antonius remains a
promoted event with initial_tier: "B"; its failure does not block Phases 1 and 2.

## 3.1 Establish the loss stage and choose a measured composition method

Add an ignored antonius_block_composition_trace test in audit_repair_tests.rs using
audit_traced_unit, audit_save_unit_trace, production_body_scale, compose_body_canvases,
retain_body_canvases, ranges_for_composed_canvases, and the existing line assertions.
Extend shared diagnostics where needed; do not implement a parallel bander or packer.

Capture both the current configuration and a temporary plain-entry configuration.
Save them separately under
tmp/pdfs/inner_tomi_audit_repairs/antonius-composition/{baseline,plain,candidate}/.
Archive the reproducible JSON/source overlays under the new evidence archive.
For an unresolved outcome, create the disposition report and disposition_trace
references defined in Step 0.2, with the actual tested configuration and remaining
uncertainty. Keep this local Antonius baseline separate from the immutable pre-installation
full-work baseline captured in Step 0.1.
Restore the production table after a failed trial; retained generated files must
identify their configuration and must not be presented as final deliverables.

For all 23 measured source glyph rectangles and their preceding/following anchors,
trace raw/production crop, raw bands, furniture-kept parent presence, protected
Pass-A bands, Pass-B bands, actual strip interval/class/ink, scaled strip dimensions,
production scale, page budgets, packed canvases before filtering, returned canvases
after filtering, and canvas hashes/row coverage. Report the first failed boundary.
A diagnostic may record failure without passing a restoration assertion; it must
not swallow a missing source/render error or turn an unexplained failure into success.

Keep composed images with a full --no-compile build, then compile the emitted
tomus_7_inner.tex with callable LuaLaTeX, twice, in build_facsimile. Verify the new
PDF embeds the candidate images and inspect the affected output pages. Link the
source/strip/canvas evidence to PDF physical pages. Check freshness and configurations
before interpreting absent landmarks; OCR absence alone is not a pixel-loss proof.

Evaluate these methods in order of source preservation and measured feasibility:

- (a) Whole-block recognition: measure the actual strip including surrounding half-gaps,
  its scaled height, and the fresh body budget. If it fits, test the existing
  is_known_whole_source_strip pattern with exact column dimensions, actual strip
  interval, and a measured source-region fingerprint. This preserves source layout
  and avoids fill-splitting into partial pages. Require the strip to wrap whole and
  all 23 line glyph rectangles to remain on returned canvases. Matching data must
  be extracted from original source pixels; no guessed signature or global change.
- (b) Crop-edge adjustment: test a measured preserve_known_body_crop_edge case to
  remove inter-line interference only if it preserves all reviewed Latin and the
  approved in-block annotation content. Save full-page before/after overlays,
  leftmost glyph rectangles, and any excluded marginal artifact with its identity
  and disposition. Section letters b/c were approved as harmless source content;
  do not silently remove them. Reject a crop that sacrifices approved content.
  Verify the following chapter and all other p562 body glyphs, not just the block.
  Use production crop bounds for all ledger/pixel assertions.
- (c) Explicit bands: hand-measure 23 full glyph rectangles, protected intervals,
  and actual cut_points midpoints. Approximately 48-pixel pitch is a locator only.
  Require positive gaps between protected bands and cuts outside every adjacent
  glyph rectangle. If skew makes neighboring vertical glyph extents overlap,
  reject disjoint horizontal splitting rather than bisecting glyphs or inventing
  blank gaps. Preserve source pixels and approved marginal annotations.

For (c), all 23 records may share observed 249..1356 only while that interval remains
the actual raw evidence. The production helper adds bands to kept; it does not replace
a kept parent. Require measured proof that the original parent is absent before
Pass-A protection, exactly 23 distinct sub-bands after Pass A, and exactly 23 after
Pass B. Add ordinary antonius_shared_observed_bands_survive_both_passes using an
edition-specific grouped fixture, plus real-source assertions in the trace.
Check owner/source/page/dimensions/observed mismatch and idempotence. If a parent
survives, reject this data-only method unless a separate reviewed design explicitly
specifies its removal. Do not weaken the merge rule for this case.

For (a), add ordinary antonius_whole_block_wraps_without_bisecting_glyphs covering
the exact fingerprint, mismatches, a partially filled canvas, and a too-small budget.
For (b), add ordinary antonius_crop_adjustment_preserves_all_reviewed_glyphs covering
the exact crop signature and mismatches, and full-page real-source assertions.
Implement only the selected method's ordinary regression, plus the shared band tests.

Before Step 3.2, save a dated method-decision note in llm_artifacts with exact tuples
or all 23 intervals/cuts, original-pixel identities, glyph overlays, measured scale
and budgets, the first loss/unreadability stage, artifact decisions, and the named
passing candidate tests. If no method passes, remove failed candidate changes,
archive/authenticate the disposition report and trace, record Antonius unresolved
with matching reason and configuration, refresh portable evidence, and proceed to
the partial delivery path.

**LLM verification:** Run cargo fmt --all; cargo fmt --all --check;
cargo clippy --all-targets -- -D warnings; cargo test known_source_body_band;
cargo test --release --lib antonius_block_composition_trace -- --ignored --nocapture;
cargo test antonius_shared_observed_bands_survive_both_passes for method (c),
cargo test antonius_whole_block_wraps_without_bisecting_glyphs for (a), or
cargo test antonius_crop_adjustment_preserves_all_reviewed_glyphs for (b).
Verify that the chosen ordinary test exists and executes; do not run an absent
method test and accept zero matches. Inspect the trace's reported first failure,
all 23 glyph rectangles, actual cuts, and the candidate PDF embedding. The trace
must use full-work production scale, not a restricted --unit scale.

**Human verification:** Compare all candidate glyph overlays with the source.
Inspect every page break touched by the block and both anchors in tomus 7.
Confirm the selected method preserves complete glyphs, source layout, and approved
annotations. A reviewer must be able to reproduce the decision from saved artifacts.

## 3.2 Install and verify

Install the selected measured method in reflow.rs and its ordinary regression in
audit_repair_tests.rs. Add exact retained band intervals to closing_body_bands.json
using production crop bounds. A whole-block method has one body band plus 23 separate
reviewed glyph rectangles in the audit ledger; do not pretend it produces 23 detector
bands. Update installed status and authenticated candidate passing evidence.

Rebuild all units at the production scale and freshly compile tomus_7_inner.pdf.
Run antonius_reviewed_lines_survive_composition_whole with whole_line: true for each
of the 23 rectangles, plus reviewed neighboring source regions. For explicit bands,
require exactly 23 distinct final bands and glyph-safe midpoint cuts. For a whole
strip, require the recognized strip to wrap whole; for a crop correction, require
all reviewed p562 glyphs and the next chapter to survive. Record actual entry counts.

If the source/canvas/PDF gate fails, restore the safe pre-candidate configuration,
retain the failed evidence, and set Antonius unresolved with a concrete reason and
the authenticated disposition_evidence/disposition_trace pair from Step 0.2.
Do not leave failed entries installed in a partial delivery.

**LLM verification:** Run cargo fmt --all; cargo fmt --all --check;
cargo clippy --all-targets -- -D warnings; cargo test known_source_body_band;
run the selected ordinary Antonius regression from Step 3.1;
cargo test --release --lib antonius_reviewed_lines_survive_composition_whole
-- --ignored --nocapture; cargo test --release --lib
tier_b_non_antonius_source_lines_restored -- --ignored --nocapture;
cargo test tier_b_delivery; cargo test portable_audit.
Require one Antonius event / 23 lines, plus unchanged prior 19-event / 21-line coverage.
Refresh the portable projection after recording candidate evidence.

**Human verification (gating):** Read all 23 original lines from the source and rebuilt
reader, including the interval from the worksheet's preceding asser- anchor to chapter
LXXXVII. Confirm legibility, intact ascenders/descenders, order, source annotations,
neighboring prose, and every touched page break. Record the result before declaring
Antonius restored; a passing retention test alone is insufficient.

# Phase 4 — Rebuild, verify delivery scope, and record evidence (Suggested implementer: Opus)

## 4.1 Full rebuild and whole-suite green

Run the full offline build command from Current state. Use callable LuaLaTeX; its
binary may be configured through PLUTARCH_LUALATEX. Record build start time,
command, scale, source configuration, compilation output, and output identities.
Require all eight tomus_N_inner.pdf files to be freshly generated after build start
and parseable with their new page counts. Confirm the log takes the compile branch.
Fresh TeX files or an exit-zero command without PDF compilation do not pass this gate.
Preserve the raw render cache; do not use --clean-cache or refresh transcription.

Run the completed phase tests and existing ignored audit_tier_a_source_lines_restored.
During the first delivery pass, the new full-delivery test waits for Step 4.2's
declared partial/complete state. After that declaration, rerun this gate including
tier_b_source_lines_restored. Re-export portable evidence and validate authenticated
archives whenever statuses/reports change.

**LLM verification:** cargo build --release --bin facsimile; full offline build;
cargo test --release --lib tier_b_non_antonius_source_lines_restored -- --ignored
--nocapture; cargo test --release --lib audit_tier_a_source_lines_restored
-- --ignored --nocapture; the Antonius regression if that repair is included;
cargo test --release --lib tier_b_source_lines_restored -- --ignored --nocapture
after delivery declaration; cargo fmt --all; cargo fmt --all --check;
cargo clippy --all-targets -- -D warnings; cargo test.
Run cargo test --lib audit_evidence_ -- --ignored --nocapture --test-threads=1 after
evidence refresh, and python tools/check_portable_tests.py --offline --log
tmp/tier-b-portable-tests-final.log. Inspect execution counts and PDF freshness.

**Human verification:** Confirm the eight fresh inner PDFs open, the build provenance
matches installed code, and no trial PDF is mistaken for a delivery artifact.

## 4.2 Verify restored seams and write the delivery record

Use worksheet physical pages as baseline search hints. Locate rebuilt seams by
source page identity, unit, and prose anchors; adding lines can move every subsequent
physical page. Verify the 19 non-Antonius events and, if included, Antonius's event.
Record baseline and rebuilt physical pages/folios, source page(s), the broken baseline
jump, continuous source-faithful after anchors, reviewed line count, and source/canvas/PDF
evidence references. Preserve source punctuation and hyphens when quoting anchors.

Write a dated verification note under llm_artifacts recording the three confirmed
excluded heads, Phocion tightening, all Antonius candidate outcomes and selected method,
Dio p580 skew/padding, verse layout, marginal-artifact dispositions, actual protection
record count, build freshness, and the accepted delivery scope. Do not delete the
extraction note, findings, plans, or failed-trial evidence.

Advance each visually passed event from installed to restored with human_review pointing
to its entry in this verification note. Archive fresh passing reports, update references,
refresh the portable projection, and set tier_b_delivery to partial or complete using
Step 0.2's exact count rules. Partial delivery requires Antonius explicitly unresolved;
it must never be labeled full plan completion.

Use Step 0.1's immutable full-work baseline manifest and unit traces as the comparison
reference. Authenticate the manifest and each referenced trace before comparison;
do not substitute trial PDFs or newly overwritten diagnostic output. Require the
same source PDF/unit-table identities and full-work scale. Record the reviewed
production-file digest changes as the intended implementation configuration.

Add the ignored tier_b_full_work_matches_baseline_outside_repaired_units test using
the shared baseline serializers and production pipeline. Derive affected unit IDs
from the 20 initial Tier-B event records and the delivery's installed/restored repairs.
For every unrelated unit, require the same retained physical source identities,
production crops, retained bands and column hashes, metrics, canvas count, and ordered
canvas hashes/dimensions. Reject missing or extra units. For affected units, report
expected band/metric/canvas changes and verify original pixels and unique ordered
retention of every previously retained body interval, allowing a reviewed enlarged
or coalesced band to contain an old interval rather than demanding old band equality.
Use existing source-pixel/strip/composition helpers and the recorded source coordinates.
Investigate any removed/duplicated previously retained Latin or unrelated body change.
Keep historical multi-line regions under their historical coverage policy.
Pass-A additions can change affected units' metrics and composition legitimately. In changed tomi, inspect later unit starts, recto placement, running folios,
contents references, and signature padding. Expected pagination changes are recorded;
missing/duplicated Latin or unrelated body changes require investigation.

Any late Dio padding, crop/fingerprint change, or other code/fixture adjustment invalidates
the earlier final gate: update all corresponding measurements/evidence, rerun the
affected source and composition tests, regenerate affected outputs at full-work scale,
repeat visual checks, and repeat Step 4.1's final fmt/clippy/test and freshness checks.
Documentation/evidence changes require their validators and ordinary suite again.

**LLM verification:** Run cargo test tier_b_delivery; cargo test portable_audit;
cargo test portable_evidence_document; cargo test --release --lib
tier_b_source_lines_restored -- --ignored --nocapture and confirm exactly 19/21
for partial or 20/44 for complete. Run authenticated evidence checks after refreshing
the portable projection, and repeat Step 4.1 for the final recorded configuration.
Run cargo test --release --lib tier_b_full_work_matches_baseline_outside_repaired_units
-- --ignored --nocapture against the authenticated pre-installation manifest.
Inspect source/canvas comparisons and reconcile all recorded PDF page locators.

**Human verification:** Read every delivered seam against its source and confirm the
note matches the fresh PDFs. Inspect changed page breaks and downstream references.
For partial delivery, confirm the unresolved Antonius disposition is explicit and no
failed candidate block is shipped as a successful repair.

# Durable record

The extraction note retains original detector coordinates and the 21 non-Antonius
installation tuples; its Antonius tuple is failed-trial diagnostic evidence.
The promotion worksheet preserves confirming reads and historical locators.
The Antonius findings preserve the observed failed trial and distinguish it from
unproven causal explanations. Step 3.1's method note, the authenticated evidence
archive (including the pre-installation full-work baseline and unresolved disposition
reports), the policy-aware portable projection, and Step 4.2's delivery record establish
the actual implementation and verification. Keep all these dated artifacts after completion.
