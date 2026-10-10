# Context

This plan combines two complementary audits of the facsimile body: an independent
inventory of original-source Latin lines, followed through the production image
pipeline into compiled inner PDFs, and a systematic inspection of page and unit
seams. Its purpose is to discover additional missing or partially preserved Latin
after the accepted Tier-A and Tier-B repairs. It does not implement repairs.

The required preservation rule is: **every unique source Latin passage must
survive completely at least once, at its correct position in the reader's logical
reading order.** Original PDFs contain repeated scans. Losing an unnecessary
copy is acceptable; losing the sole complete copy, a unique continuation on a
shared page, or part of a retained line is a finding. Naturally repeated language
does not authorize deleting a source occurrence.

Source images remain authoritative. Temporary OCR readings, model observations,
and geometric classifications are evidence for an audit, never replacement Latin.
The work concerns the 68 Life/comparison body units in the eight facsimile inner
tomi. Front matter, imposition, and transcription are outside the coverage claim;
the report must identify those exclusions explicitly. Source headings, reference
letters, and verse labels within body territory remain visible inventory objects
even when they are reported separately from prose lines.

# Current state of the world

The planning branch is `plan/facsimile-line-coverage-seams`, created from
`plan/inner-tomi-audit-repairs` on 2026-10-10. An existing working-tree change to
`llm_artifacts/2026_10_09_22_36_TIER_B_DELIVERY_REVIEW.md` belongs to earlier work;
preserve it and do not stage it with this plan or implementation commits.

Relevant current implementation:

- `src/facsimile/mod.rs::run_facsimile` chooses source ranges, computes the global
  scale, invokes extraction/composition, writes body PNGs, plans volumes, emits
  TeX, and compiles/imposes. `FacsimileArgs` and `src/bin/facsimile.rs::Args` define
  CLI options. `--offline` reads cached front matter without vision calls.
- `src/facsimile/reflow.rs::unit_body_strips` returns `PageBands` and
  `LineMetrics`. `PageBands` is `(GrayImage, Vec<Band>)`; it does not carry a
  physical source-page identity. Source pages skipped as reviewed duplicates or
  failed during rendering cannot be indexed by `page - start_page`.
- `render_and_band` masks the original page before `production_source_column`
  and `band_source_column`. Subsequent opening, interior-head, shared-tail, and
  furniture trims, plus known body-band protections, decide retained bands.
  `strips_in_reading_order` applies an additional ink floor. `compose_body_canvases`
  scales and sometimes splits strips; `retain_body_canvases` applies a final ink
  floor. `place_line_images` pastes into a fixed-width canvas. Each stage can
  affect coverage and must be observed rather than reimplemented for the audit.
- `BODY_DPI` is 300; `columns.rs::INK_THRESHOLD` is 170. `source_strip_scale`
  contains fingerprint-scoped fits for reviewed wide columns, so actual scaled
  dimensions, rather than the requested global scale alone, define placement.
- `skip_known_duplicate_source_page` is unit/source/page scoped.
  `fixtures/inner_tomi_repair_source_trace.json` records 12 reviewed duplicate
  records, including `skip_pages`, `retain_sequence`, and preservation evidence.
  Some early records have page-pair `occurrences`; later records instead have
  `reviewed_lines`. Do not assume either field exists for every record or parse
  free-text preservation notes as an authoritative mapping.
- `reflow.rs` test support has `RetainedSourcePageTrace::map_row` and
  `retained_page`; `audit_repair_tests.rs::audit_traced_unit` reconstructs physical
  identity around production extraction. These traces are test-only and limited
  to measured cases. The production mask operation whitens pixels; it does not
  delete rows. Historical `deleted_rows` fields must not be confused with a
  currently executed row-compaction operation.
- `tools/audit_inner_tomi.py` already provides source rendering, normalized PDF
  geometry, TeX-reference parsing, displayed-image occurrence enumeration, reader
  folio mapping, source partitions, duplicate corroboration, safe output writing,
  hashing, crop evidence, and HTML reports. `map_reader_pages` is coupled to a
  historical snapshot and pinned counts. `extract_source_regions` uses embedded
  OCR and production boundary fractions; it is useful context but cannot define
  the new independent expected inventory. `read_approved_exclusions` drops
  skipped pages wholesale, which is insufficient for the new logical ledger.
- `tools/requirements-inner-audit.txt` pins NumPy, Pillow, OpenCV, scikit-learn,
  RapidFuzz, and PyMuPDF. Reuse these dependencies. Ordinary Rust tests and the
  portable snapshot check require no scans, archives, credentials, or caches.
- `src/facsimile/review.rs` implements the existing Sonnet `--review-lines`
  operation. It examines selected opening bands and writes trim decisions into
  `facsimile_units.json`; it is not the new audit reviewer.
- `src/llm/mod.rs` has `Client::message`, `ContentBlock`, `HAIKU`, and existing
  transport retries. `src/llm/instrument.rs` has `UsageTracker`. The user's
  existing `secrets.json` contains the Anthropic key and is loaded through
  `crate::secrets::load`. No new credential file or Python API client is needed.

The accepted Tier-B delivery is documented in
`2026_10_10_08_22_TIER_B_COMPLETE_DELIVERY_VERIFICATION.md`: 20 events, 43 measured
lines, all restored and accepted. The original inventory's 44-line total is
historical; Antonius was measured as 22 lines. Do not modify either dated record
to manufacture an agreement. The accepted build and archives provide regression
evidence, not an independently exhaustive inventory of remaining source Latin.
All Tier-B losses touched a source-page head, foot, or page break. The historical
OCR audit had 311 inconclusive registrations and does not establish corpus recall.

# Desired end state

An offline audit produces an immutable input snapshot and a source-driven line
ledger for every requested body unit, including source pages excluded as duplicate
scans. A production trace explains what happened to each independently observed
source region. The audit binds that trace to actual PNG occurrences in the
accepted compiled readers, then verifies their visible placement. A systematic
seam inventory uses the same ledger and locators, with no separate definition of
expected Latin. Optional Haiku review supplies supplemental observations on
ambiguous source/reader crop pairs using the existing key.

The user can open one HTML report and answer: which unique lines are complete;
which are partial, absent, or out of order; which source regions or duplicate
relationships are unresolved; which page/unit seams were inspected; and what
evidence supports each result. Input identity, processing coverage, observed
retention, model judgment, and human adjudication remain separate facts. A
completed audit may legitimately contain findings or unresolved regions.

The ordinary facsimile build remains offline and produces the same body pixels
and TeX with tracing disabled or enabled. New Rust and Python tests run on
synthetic/test-created fixtures without local evidence. Real-source checks are
explicitly selected integration checks. Historical plans, notes, evidence, source
PDFs, transcription caches, duplicate decisions, and accepted outputs remain
intact. New output lives under `tmp/pdfs/facsimile_line_audit/` and a dated archive
after delivery.

# Tech Learnings

A physical scan occurrence and a logical passage are different identities. Two
photographs of the same printed line can differ in geometry and completeness.
The ledger therefore retains both occurrences and records their reviewed
relationship; it does not pretend their pixels or line ordinals are identical.

Tracing is observation of the actual production path. Both traced and ordinary
entry points must call the same extraction and packing implementation. An audit
that independently reproduces the packer can agree with its own predictions
while missing what production actually does.

Coordinates are half-open rectangles `[left, top, right, bottom)`, with the origin
at the top left. Keep full-source pixels, column pixels, scaled-strip pixels,
canvas pixels, and PDF points distinct. Scaling includes integer rounding and
resampling; recording actual dimensions and offsets avoids guessing from a
single floating-point scale. PDF image presence is not proof of visible presence:
clipping or subsequent drawing can hide an otherwise complete embedded image.

# Design contract

## A. Files and responsibilities

Add `src/facsimile/coverage_trace.rs` for production observation types and the
trace sink; expose it from `src/facsimile/mod.rs`. Keep image decisions in
`reflow.rs`. Add `src/facsimile/seam_review.rs` and
`src/bin/facsimile_seam_review.rs` for the optional Haiku adapter and thin CLI.

Add `tools/facsimile_line_audit.py` as the CLI and orchestration layer, and
`tools/facsimile_line_coverage.py` for source inventory, logical coverage, seam
construction, and report data. Add `tools/inner_audit_common.py` for infrastructure
shared with `audit_inner_tomi.py`, extracting existing implementations instead of
copying them. Add `tools/test_facsimile_line_coverage.py` and
`tools/test_inner_audit_common.py`, using `unittest` and temporary directories.
Keep reviewed source policy in `tools/facsimile-line-source-policy.json` and
algorithm settings in `tools/facsimile-line-audit-settings.json`. Add setup and
operation documentation in `tools/README-facsimile-line-audit.md`.

The new CLI has `capture`, `inventory`, `run`, `report`, and `adjudicate`
subcommands. `run` writes the ledger, reader verification, all seams, crops, and
review tasks. A `--resume` option reuses valid content-addressed observations;
changed inputs/settings require a new run directory. Do not alter the old audit's
capture/freeze identities or use its historical baseline name for current files.

## B. Input identity and report artifacts

`capture --reader-dir build_facsimile --snapshot-dir <new-directory>` copies the
two source PDFs, eight inner PDFs and TeX files, unit table, two repair ledgers,
source policy, settings, relevant production Rust sources, and audit Python
sources. Record file SHA-256, role, relative path, software versions, source page
counts, and observed reader counts in `snapshot-manifest.json`. Copy referenced
body PNGs when present; always derive decoded body-image identities from the PDFs
so missing standalone PNGs do not make a complete reader unauditable. Reject a
missing inner PDF, ambiguous image mapping, escaping reference, or changed file
during capture. Exclude secrets, outer imposed PDFs, and historical archives.

A run writes `run-manifest.json`, `source-inventory.json`, `logical-lines.json`,
`coverage-ledger.json`, `seams.json`, `reader-verification.json`, `errors.json`,
`review-tasks.json`, `reviews.json` when adjudicated, `index.html`, and `crops/`.
Source rasters and extraction traces are cached separately by content identity.
Stable records use sorted keys/lists and exclude timestamps, timings, absolute
paths, and worker scheduling; operational metadata goes into the run manifest.
Hash task crops and every record set to bind later review to exact evidence.

## C. Inventory, policy, and duplicates

`SourceOccurrence` contains `occurrence_id`, source PDF digest/index, one-based
physical page, raster dimensions/DPI, full-source box, raw crop hash,
`classification` (`body`, `heading`, `annotation`, `furniture`, `uncertain`),
`owner_unit_ids`, boundary evidence, segmentation confidence, and optional raw OCR
observations. IDs derive from source digest, page, detector-version/settings
digest, and box; labels and reviews do not affect them. A merged region has
explicit child observations or stays unresolved; it must not be called one line
merely because the production band detector returned one band.

Expected inventory uses unmasked full source renders. Initial generous inspection
ROI is `[0.40, 0.00, 1.00, 1.00]` in page fractions at 300 DPI; inspect the left
ROI boundary for cut components and record uncertainty or a reviewed source-only
widening. Derive the Latin gutter from original-source geometry/components, never
the production crop. Keep full-page context available for ownership decisions.
Do not use fixed top/bottom percentages to remove expected prose. Original OCR
and production unit fractions may suggest labels, but cannot independently
exclude a region. Missing OCR does not remove visible ink from the inventory.

`SourcePolicy` is versioned, source-digest bound, and contains independently
reviewed ownership intervals, furniture classifications, segmentation overrides,
and `DuplicateGroup` entries. Every exclusion records a source region, reason,
evidence crop digest, and human decision. Unreviewed automatic classifications
are hypotheses; unresolved objects stay in the denominator and review queue.
Headings/annotations are separate coverage categories, not furniture by default.

`DuplicateGroup` contains `group_id`, logical owner, physical source occurrences,
reviewed passage boundaries, `canonical_sequence`, occurrence-to-logical-span
maps, and evidence/provenance. A fragment maps to a fraction of a logical line;
a missing clipped portion cannot be waived by calling its page a duplicate.
Use original-source context to establish equivalence. Import existing approved
records as seeds without parsing prose or asserting missing occurrence pairs.
New relationships require source-only review. Cross-unit material on a shared
duplicate page must be assigned to its actual owner before deduplication.

`LogicalLine` has `logical_line_id`, unit and sequence position, expected content
region(s), equivalent occurrence mappings, required classification, and any
`source_partial`/segmentation uncertainty. The canonical reference is a reviewed
complete source occurrence where available. Different wrapping or genuinely
fragmentary scans require explicit span mappings; otherwise equivalence stays
unresolved. The audit never reconstructs illegible or absent source glyphs.

## D. Production trace and coverage semantics

`CoverageTrace` schema version 1 records input/configuration identities, requested
ranges, actual global scale, completion, page observations, strip observations,
canvas placements, and dropped-canvas events. `SourcePageTrace` identifies the
unit, source, physical page, raw dimensions/hash, applied mask rectangles, actual
crop, raw and retained bands, each stage's discarded regions/reasons, failure or
duplicate-skip disposition, and retained vector index. Keep raw band identities
and parent/child overlap mappings through coalescing and protections.

`StripTrace` records source page identity, full-source/column interval, contributing
band IDs, requested scale, actual scaled dimensions, resampling method, and any
ink-floor/empty-strip rejection. `CanvasPlacement` records stable piece identity,
its scaled-strip rectangle, canvas rectangle, canvas ordinal/hash, and clipping
intersection. Record pending remainders and all `split_tall_strip` pieces; after
canvas filtering, record old-to-final ordinal mapping and rejected placements.
Trace optional row removals only if a production operation actually executes them.

The ledger links source occurrences to production observations by geometry and
pixel evidence, not line ordinal. Trace rectangles propose mappings but do not
certify pixels. Measure unmasked source ink against masks/crops and compare the
actual resized source fragments against their placed canvas regions, preserving
left/right ends and above/below-baseline ink. Include a halo equal to resampling
support and retain ambiguous rounding as an explicit limitation. Pixel predicates
and measured similarity are versioned settings, not probabilities.

Each physical occurrence reports `complete`, `partial`, `absent`, or `unresolved`,
plus its earliest observed loss stage. `complete` requires whole required ink
coverage, matching placed pixels, a trace bound to the captured reader asset,
and successful visible-PDF verification. A confirmed direct mask/crop/packing
loss can be `absent` or `partial`; failure to locate an image alone is unresolved.
If an observed source region is already incomplete in every scan, retain
`source_partial` and distinguish edition/scan damage from generator loss.

Logical coverage is evaluated along an order-preserving path of authorized source
occurrences. Accept one complete occurrence, or explicitly mapped complementary
pieces whose union covers the required line without gaps and whose reader order
is correct. Pieces of an ordinary retained line split across adjacent reader pages
can cover it completely but also produce a `sliced_line` seam warning if a cut
crosses glyph ink. Do not combine unrelated repeated wording or arbitrary reader
positions. Multiple full copies satisfy coverage and receive a separate repeat
observation; this plan does not authorize deleting them.

## E. Systematic seams and Haiku observations

`build_seam_inventory` derives all seams from source/logical inventory and actual
reader placements, never just detected failures. Include source-page transitions,
unit openings/closings and shared boundaries, duplicate-block bypass joins, and
every adjacent body-canvas transition in a unit. Separate distinct physical and
logical seam IDs; deduplicate identical review tasks with multiple seam reasons.
Include physical duplicate edges as inventory entries marked redundant only after
their complete logical passage coverage is established.

For each seam, include the last three owned body lines before it and first three
after it, all uncertain source regions between those anchors, and relevant
heading/annotation objects. If a page owns fewer than three lines, extend context
within that unit; never silently borrow another unit's prose. Include original
source edges, even when the production crop omits them. Sparse verse and large
chapter gaps receive the same treatment. Record requested and inspected seams
and each side's coverage; unresolved ownership makes the seam unresolved.

Haiku receives labeled original-source context plus corresponding visible reader
crop(s), numbered immutable line IDs, and a neutral instruction to compare each
line without inferring missing Latin from grammar. Hide machine loss verdicts and
historical repair labels to limit anchoring. Ask for `complete`, `partial`,
`absent`, or `unclear`, source object type, supporting reader crop/line IDs, and a
brief reason. The reviewer must see both neighboring reader pages for a page
break and all required equivalent source occurrences for a duplicate question.
When a complete corresponding reader neighborhood cannot be located, export an
inconclusive task rather than asking the model to confirm absence.

`review-tasks.json` includes run/ledger/settings identities, crop hashes, line and
seam IDs, and expected JSON response schema. Limit a task to six numbered body
lines; use overlapping tasks for longer uncertain spans and preserve continuity.
The deterministic core can finish without model calls. Haiku results live in
`haiku-observations/`, separately from machine coverage and human reviews.
Disagreement creates a review item and never silently overwrites either result.

## F. Settings and review interchange

The initial settings file contains `schema_version: 1`, `source_dpi: 300`,
`source_roi: [0.40, 0.00, 1.00, 1.00]`, `ink_thresholds: [170, 210]`,
`component_connectivity: 8`, `seam_context_lines: 3`, `task_max_body_lines: 6`,
`reader_dpi: 300`, `registration_max_offset_px: 2`, and
`registration_rotation_degrees: [0]`. Estimate typical glyph height from the
median component height between 8 and 80 source pixels; if fewer than ten such
components exist, mark line grouping uncertain. Components form a proposed line
when their vertical overlap is at least half the smaller height, or their vertical
centers differ by at most half the estimated glyph height. Apply both thresholds
and retain disagreements. Split a proposed merged line at white-row valleys;
if no reliable valley separates apparent baselines, retain an unresolved group
for a reviewed segmentation override. These are initial source hypotheses, not
authority to suppress source ink. Save every parameter in the settings identity.

Within a bound canvas, compare expected resized piece pixels exactly after the
same production resampling; report differing coordinates rather than silently
accepting a percentage of missing ink. For visible PDF comparison, construct a
temporary reference page containing the bound canvas at the captured image
transform with the same page dimensions, and render reference/target through the
same engine at 300 DPI. Initially permit a luminance difference of at most 12
at aligned pixels and offsets within two pixels. Larger differences or
registration ambiguity make the affected line unresolved unless independent
geometry/pixels prove a loss. Do not use aggregate similarity alone to accept
a line. Freeze calibrated render tolerances and source detector settings in
Phase 7; changes invalidate comparisons and cached verdicts.

All review interchange documents contain `schema_version`, `snapshot_sha256`,
`inventory_sha256`, `reviewer`, and `decisions`. A source decision contains
`occurrence_id`, `crop_sha256`, `classification`, `owner_unit_id` (or explicit
unresolved owners), `reason`, and optional measured `child_boxes`. A duplicate
decision contains `group_id`, source-only crop references/hashes, logical owner,
canonical sequence, and explicit maps of occurrence rectangles to logical
regions; absence of a complete mapping stays unresolved. A human loss decision
also binds `run_sha256`, `ledger_sha256`, line/seam IDs, and the source/reader
crop hashes. The CLI validates all supplied identities before importing any
decision. Model observations use their separately hashed task identity instead
of pretending to be these human decisions.

# Implementation sequence

Each numbered step is a session-sized deliverable. Keep all existing tests green.
Every Rust-changing step must apply `cargo fmt --all`, then pass
`cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings` alongside
its listed test commands. These commands are repeated below deliberately so no
step's verification depends on remembering an earlier checklist. Use new output
directories; never regenerate fixtures from their own actual values in a test.

## Phase 1 — Shared audit infrastructure and immutable inputs — Suggested implementer: Sonnet

### 1.1 Extract reusable infrastructure without changing historical behavior

Create `tools/inner_audit_common.py`. Move the implementations of `hash_file`,
`json_hash`, `validate_box`, `page_geometry`, `normalized_pdf_box`, `pixel_bounds`,
`parse_tex_refs`, `enumerate_image_occurrences`, `read_footer_folio`,
`relative_path`, `contained`, `safe_write_path`, `write_json`, `run_external`, and
`render_page` from `audit_inner_tomi.py` into it. Preserve signatures/defaults and
import/re-export them under their current names. Move any necessary constants or
private helpers once as dependencies; avoid importing the audit CLI back into the
common module. Preserve lazy optional imports and the CLI main guard.

Add `map_reader_occurrences(reader_inputs, expected_counts=None)` in the common
module by extracting the snapshot-independent inner logic of `map_reader_pages`.
`reader_inputs` explicitly supplies tomus, PDF/TeX paths and identity. The old
function remains an adapter and passes its existing pinned expectations. The new
function permits arbitrary explicit tomus subsets for fixtures and focused runs,
uses actual occurrences rather than unique xrefs, and preserves ordinal checks,
page kinds, physical locators and folio provenance. Counts/asset order mismatches
are errors, not a zip that truncates data. Verify decoded pixel hashes against
available PNGs; without PNGs retain mapping ambiguity rather than claiming an
asset identity from ordinal alone.

**LLM verification:** Run `python -m unittest discover -s tools -p
test_inner_audit_common.py`; include `shared_map_counts_reused_xref_twice`,
`shared_map_rejects_reference_count_mismatch`,
`shared_map_distinguishes_padding_and_body`, `shared_map_preserves_folio_provenance`,
`shared_map_rejects_reordered_asset_pixels`, and
`shared_writer_rejects_protected_output`. Run `python tools/audit_inner_tomi.py
check` when its existing tool prerequisites are installed, and `cargo test` to
confirm the ordinary suite. If a historical replay is available, compare semantic
reader maps without writing to its archive; otherwise document that optional
comparison as unavailable.

**Human verification:** Inspect the small synthetic reader map and confirm page
kinds and repeated image occurrences. Compare one existing historical reader-map
entry with the adapter result when local evidence is available.

### 1.2 Implement schemas, capture, and content identity

Create the two new Python files, source-policy/settings files, and test module.
Implement `validate_source_policy`, `validate_trace`, `capture_snapshot`, and
`load_snapshot` with the Design contract fields. Add the CLI `capture` command;
other commands may report clearly that their implementation is pending. Initial
policy has no new reviewed exclusions; import known duplicate records as seeds,
and preserve their original record hash. Record all observed counts instead of
copying historical 2,678-body-page assumptions. Require all eight tomi for the
full snapshot, and hash their decoded body images. Copy inputs to a new snapshot
directory, verify source/copy digests before completion, and refuse overwriting a
snapshot. Keep source-raster cache writes outside snapshots. Never read, hash,
copy, or log `secrets.json` during capture.

**LLM verification:** Run `python -m unittest discover -s tools -p
test_facsimile_line_coverage.py`, covering `capture_rejects_changed_input`,
`capture_excludes_secrets`, `capture_rejects_missing_inner_pdf`,
`capture_accepts_pdf_without_standalone_pngs`,
`capture_rejects_ambiguous_image_identity`, `source_policy_rejects_unbound_exclusion`,
and `trace_schema_rejects_inconsistent_coordinate_spaces`. Run `cargo test`.

**Human verification:** Read a synthetic manifest and the current snapshot's
observed counts; confirm the input build is the accepted current reader rather
than the 2026-10-06 baseline. Inspect source-policy seeds without approving new
equivalence from a skip list alone.

## Phase 2 — Observe production extraction and composition — Suggested implementer: Opus

### 2.1 Add extraction tracing through the existing production path

Implement typed serializable records in `coverage_trace.rs`: `CoverageTrace`,
`SourcePageTrace`, `BandObservation`, `StripTrace`, `CanvasPlacement`, and
`TraceEvent`, plus a `TraceSink` with a disabled/default implementation. Add
`unit_body_strips_with_trace`; the existing `unit_body_strips` calls the same
internal implementation with no sink. Observe raw raster identity, masks,
production crop, raw/coalesced/retained bands, each boundary/furniture decision,
known band protections, duplicate skips, and rendering failures at their actual
call sites. Use before/after overlap records when existing helpers return bands;
do not recalculate their classifications or thresholds inside the tracer.

For skipped duplicates, emit identity and disposition without requiring production
to render them; their original images come from the independent inventory.
For failures, emit `render_failed`/`detect_failed` and no retained index. Record
unit/source/page identity through retained vector indices explicitly. Factor the
test-only row-mapping primitive into the trace module if used by production and
tests; update test adapters to call it, retaining their frozen serialization
through an adapter. Do not turn historical row deletions into current transforms.

**LLM verification:** Run `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test coverage_trace`.
Add `coverage_trace_extraction_matches_untraced_pixels`,
`coverage_trace_records_mask_before_crop`,
`coverage_trace_maps_skipped_page_by_identity`,
`coverage_trace_records_render_failure_without_false_coverage`,
`coverage_trace_tracks_coalesced_band_children`, and
`coverage_trace_whitening_does_not_shift_rows`. Run `cargo test repair_trace` and
`cargo test inner_tomi_duplicate` for existing contracts.

**Human verification:** Inspect one synthetic trace through masking and a skip;
confirm coordinates are original source coordinates and no dropped region becomes
expected furniture solely from the production verdict.

### 2.2 Trace strips, actual scaling, splitting, and canvas rejection

Add `compose_pages_with_trace`; both entry points use one internal packer. Carry
piece metadata beside the actual image through `strips_in_reading_order`,
`scale_strip`, `pending`, `split_tall_strip`, `place_line_images`, and
`retain_body_canvases`. Instrument shared implementations rather than creating an
audit packer. Record ink-floor skips, zero-height skips, actual scaled widths and
heights, exact scaled-piece intervals, canvas offsets, right-edge clipping, and
old/final ordinals. `place_line_images` must report the pasted intersection; a
nominal placement larger than a fixed canvas is not a complete placement. Make
resampling and rounding identifiers explicit in the trace.

Extend `FacsimileArgs` and the existing CLI with `--coverage-trace <directory>`.
Write one unit trace atomically and a completed trace manifest after the build;
an interruption leaves the overall manifest incomplete. `run_facsimile` binds
trace canvases to written body names/hashes and emitted TeX identities. Tracing
must work with `--offline --no-compile`, require no secrets, and avoid changing
normal output. Refuse a trace destination equal to or nested in source/snapshot
inputs. Do not use `--clean-cache` or production output cleanup against accepted
`build_facsimile/` for this work.

**LLM verification:** Run `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test coverage_trace`, and
`cargo test facsimile_run_args`. Add
`coverage_trace_composition_matches_untraced_pixels`,
`coverage_trace_records_actual_wide_column_fit`,
`coverage_trace_preserves_pending_piece_order`,
`coverage_trace_records_tall_strip_piece_intervals`,
`coverage_trace_records_rejected_canvas`,
`coverage_trace_exposes_lateral_canvas_clipping`, and
`coverage_trace_cli_is_offline_and_default_disabled`. Run `cargo test` and
`cargo build --bin facsimile` before ending the phase.

**Human verification:** Build one unit into a new temporary output directory with
tracing, inspect its TeX, PNGs, and trace, and compare decoded PNGs and TeX against
the same untraced build. Include an indented verse or known wide-column example.

## Phase 3 — Independent source-line inventory — Suggested implementer: Opus

### 3.1 Inventory unmasked source ink without production exclusions

Implement `inventory_source_page` and `inventory_source_units` in
`facsimile_line_coverage.py`, plus the `inventory` CLI. Render every physical page
in each requested unit's inclusive source range, including duplicate skips;
deduplicate shared-page render work by source/page identity. Use the full-height
generous ROI and original page context specified above. Initial segmentation uses
8-connected components at ink thresholds 170 and 210, grouping components with
overlapping vertical support and neighboring baselines. Keep fragments and
cross-threshold disagreements instead of removing them with production height,
span, or ink floors. Expose all detector parameters in settings; tiny components
may be uncertainty objects rather than prose lines, never invisible exclusions.
No segmentation result is discarded merely because it touches a page/ROI edge.

Separate `classify_source_objects` from segmentation. Use source geometry and
optional source OCR for body/head/annotation hypotheses. Missing OCR or uncertain
gutter/title separation yields `uncertain` with crops and source-page context.
Merged multi-line regions need measured child baselines or an unresolved group.
Save every inventory object, not just likely losses. The independent pass may
reuse infrastructure and elementary pixel operations, but must not call
`render_and_band`, `drop_furniture`, `extract_source_regions`, or load production
retained bands as its expected inventory.

**LLM verification:** Run the Python coverage suite; add
`source_inventory_keeps_first_and_last_ink`,
`source_inventory_keeps_faint_and_short_verse_lines`,
`source_inventory_keeps_ink_without_ocr`,
`source_inventory_marks_merged_baselines_unresolved`,
`source_inventory_flags_cut_roi_components`,
`source_inventory_includes_skipped_duplicate_pages`, and
`source_inventory_is_unchanged_by_production_trim_mutation`. Inspect saved
inventory JSON on Theseus, Pericles, Cato Minor, and Antonius using `inventory
--snapshot <path> --units theseus,pericles,cato_minor,antonius --run-dir <new-path>`.
Run `cargo test` to retain the full Rust regression suite.

**Human verification:** Inspect full-source overlays at every head/foot in these
four units, including Pericles verse and Antonius v2/562. Confirm visible body ink
is represented even when line segmentation is uncertain; inspect Greek/gutter
uncertainties without suppressing them to make the report look clean.

### 3.2 Resolve source ownership and classifications independently

Implement `apply_source_policy` and `validate_source_inventory`. Export source-only
review sheets for ambiguous opening/shared-tail regions, furniture, merged lines,
and annotations. `adjudicate --source-reviews <file>` imports explicit human
classification/ownership decisions bound to inventory/source/crop identities;
write a new version of the source policy, never rewrite an existing run's inputs.
Production `title_top_frac`/`body_top_frac` are candidate locators and disagreement
checks, not exclusion authority. Each source body region must have exactly one
logical owner; approved equivalence handles multiple physical copies separately.
If the owner is unresolved, keep all plausible owners and block complete coverage
for the affected boundary. Detect uncovered visible ink between reviewed objects.

**LLM verification:** Run the Python coverage suite with
`source_ownership_shared_page_preserves_both_units`,
`source_ownership_rejects_unassigned_body`,
`source_ownership_detects_overlapping_exclusions`,
`source_policy_rejects_stale_review_crop`,
`source_policy_keeps_annotations_separate_from_furniture`, and
`source_policy_flags_ink_outside_reviewed_line_boxes`. Run `cargo test`.

**Human verification:** Review source-only boundary sheets before viewing reader
outcomes. Verify the exact ownership cut around each display title, short final
line, and verse label. Leave genuinely uncertain regions unresolved.

## Phase 4 — Logical passages and reviewed scan equivalence — Suggested implementer: Opus

### 4.1 Build passage equivalence from existing evidence and source-only review

Implement `seed_duplicate_groups` and `resolve_duplicate_groups`. Read known
records from the captured original and audit repair ledgers; validate references
to the captured unit/source identities. Explicit page pairs can seed source
comparison. A `skip_pages` list, neighboring page arithmetic, or a reader match
cannot establish complete equivalence. Use reviewed regions and independently
viewed source pairs to write occurrence-to-logical-span maps. Export unresolved
known duplicate groups as source-only sheets; import approved maps through
`adjudicate --duplicate-reviews <file>` into a new source-policy version.

For shared duplicate pages, split into actual owning passages before equivalence;
in particular comparison/Eumenes material on v2/101–105 crosses unit territory.
Keep the incomplete second openings of Alexander, Phocion, and Otho explicit.
Retain all source inventory objects, including unique margins of purported
duplicates. Unknown repetition is an observation, not permission to collapse it.

**LLM verification:** Run the Python coverage suite with
`duplicates_complete_copy_satisfies_logical_passage`,
`duplicates_partial_copy_does_not_waive_missing_head`,
`duplicates_shared_page_preserves_unique_continuation`,
`duplicates_natural_repeated_words_stay_distinct`,
`duplicates_seed_without_occurrences_stays_unresolved`,
`duplicates_equivalence_requires_source_evidence`, and
`duplicates_eumenes_comparison_assigns_cross_unit_content`. Run `cargo test`.

**Human verification:** Inspect all 12 existing duplicate records against source
pairs, reviewing partial openings and block bypass joins. Confirm no new skip
decision is installed in the production generator.

### 4.2 Construct ordered logical lines and coverage denominators

Implement `build_logical_lines` and `validate_logical_sequence`. Give each unique
owned source line an ordered logical position; equivalent physical occurrences
refer to that position. Fragment/whole-block mappings must explicitly account for
every required region. Reject contradictory/cyclic maps and unresolved boundaries
as a complete canonical sequence. Keep source-partial lines, headings/annotations,
uncertain objects, physical copies, unique logical lines, and waived redundant
occurrences as separate counts. Do not delete duplicate source rows from the
inventory and report only the smaller denominator.

**LLM verification:** Run the Python coverage suite with
`logical_lines_count_unique_passages_and_all_occurrences`,
`logical_lines_reject_cyclic_aliases`,
`logical_lines_keep_source_partial_distinct_from_reader_loss`,
`logical_lines_reject_uncovered_duplicate_span`, and
`logical_lines_ids_stable_across_review_verdicts`. Run `cargo test`.

**Human verification:** Read the ordered logical sequence across a duplicate block
and a shared unit boundary. Confirm that duplicate collapse affects the expected
logical sequence while retaining all physical evidence and unique source text.

## Phase 5 — Coverage from original ink to visible reader — Suggested implementer: Opus

### 5.1 Bind production traces to the accepted reader, not a guessed rebuild

Implement `bind_trace_to_reader` and the offline `run` orchestration. Generate a
fresh full traced build in a dedicated temporary directory with the captured
source/unit identities and matching production code. Use the existing production
scale over all selected corpus ranges; a focused unit build's scale is not assumed
to equal the full-work scale. For full accepted-reader verification require the
full-build scale and unit set. The independent inventory can be scoped, but any
report must describe that scope explicitly.

Compare generated traced canvases against decoded embedded body images of each
captured inner PDF, by full grayscale pixels/dimensions and ordered asset
occurrence, not PDF bytes, xref alone, filename alone, or stale page numbers.
Require each body occurrence to bind uniquely. Use hashes of source and relevant
code/unit settings to bind trace identity. A mismatch is `trace_binding_failed`;
preserve it and do not silently switch the audit target to the fresh build.
Standalone PNGs are supplementary cross-checks. Expose a separate explicit
capture if a user later chooses to audit a new build.

**LLM verification:** Run Python tests
`trace_binding_rejects_stale_canvas`, `trace_binding_handles_reused_pdf_image`,
`trace_binding_rejects_reordered_body_assets`,
`trace_binding_uses_decoded_pixels_without_png`, and
`trace_binding_rejects_wrong_global_scale`. Run a scoped current-reader binding
using traces from the full temporary build. Run `cargo test`.

**Human verification:** Inspect at least one accepted reader page against its
fresh trace canvas, confirming the physical page and printed-folio locator refer
to the captured current PDF. Inspect a deliberately mismatched binding result.

### 5.2 Evaluate actual line pixels and compiled visibility

Implement `evaluate_occurrence_coverage`, `evaluate_logical_coverage`, and
`verify_reader_visibility`. Partition expected source ink by trace masks, crop
edges, retained bands, emitted strips, placed pieces, and final canvases; report
the earliest observed loss with full-source geometry and stage evidence. For
surviving strips use actual resampling dimensions/piece offsets and compare placed
pixels. Reuse existing source-pixel checks where their production primitives are
applicable; do not copy repair-specific geometry or use their finite inventory as
the new expected inventory. Preserve slice-through-ink warnings separately from
complete union coverage.

Use the shared reader occurrence mapper and PDF transforms to project canvas
regions into reader page coordinates. Inspect page CropBox/MediaBox and actual
visible rendering at 300 DPI using shared raster/cache infrastructure. Compare
projected line-region renderings against the same canvas rendered through its
PDF placement, allowing explicitly configured interpolation/antialiasing
tolerances. Check left/right glyph endpoints and ascenders/descenders as well as
overall similarity. Additional clipping paths/overpainting or uncertain transform
registration yield a visible failure or unresolved observation; an embedded image
hash alone cannot produce `complete`. Save measured differences and crops.

Initial tolerances remain provisional until Phase 7 calibration. Restrict local
registration to the projected reader neighborhood and configured small offsets;
never search arbitrary text elsewhere to clear an omission. Classify geometric
loss directly when proven. Do not equate failed similarity/registration with
absence. Compute logical coverage along the authorized order-preserving path,
including complementary pieces only when their mapped span union is complete.

**LLM verification:** Run Python tests `coverage_finds_masked_first_line`,
`coverage_finds_cropped_last_line`, `coverage_finds_strip_ink_floor_loss`,
`coverage_finds_rejected_canvas_loss`, `coverage_finds_lateral_glyph_loss`,
`coverage_marks_registration_failure_unresolved`,
`coverage_accepts_ordered_complementary_duplicate_fragments`,
`coverage_rejects_disordered_or_gapped_fragment_union`,
`reader_visibility_detects_clipped_embedded_image`,
`reader_visibility_detects_overpainting`, and
`reader_visibility_keeps_missing_scan_ink_source_partial`. Run `cargo test`.

**Human verification:** Review source/canvas/visible-PDF triplets for known
restored head/foot lines, verse, a merged Antonius region, and a synthetic clipped
PDF. Confirm that complete coverage requires visible whole glyphs, not an OCR
reading or a favorable aggregate score.

## Phase 6 — Systematic seam report and optional Haiku review — Suggested implementer: Sonnet

### 6.1 Build every seam, shared evidence cards, and human adjudication

Implement `build_seam_inventory`, `build_review_tasks`, `write_coverage_report`,
and the `report`/`adjudicate` commands. Apply the three-lines-per-side context
contract to every seam, including uncertain objects between anchors, all duplicate
bypass joins, and all reader canvas transitions. Derive coordinates from the ledger
and use one evidence renderer for ordinary coverage findings and seams. Reuse
shared crop/save/HTML escaping primitives; do not create a second reader mapper
or separate matching algorithm for seams. Task images are labeled externally or
on display copies; untouched crop bytes stay available and hashed.

Show all seams in an expandable list and prioritize partial/absent/unresolved
items without hiding apparently complete seams. Cards include source physical
pages, reader physical pages/folio provenance, unit, equivalent copies, earliest
loss stage, full-source and reader boxes, and evidence identities. Use separate
columns for machine result, Haiku observation, and human verdict. Human imports
are `confirmed_loss`, `retained`, or `uncertain`, with exact line IDs, reviewer,
reason and crop hashes; a stale run/crop identity is rejected. Maintain revisioned
reviews rather than changing immutable machine records. No adjudication installs
a production repair or rewrites source Latin.

**LLM verification:** Run Python tests `seams_include_every_source_transition`,
`seams_include_unit_and_duplicate_bypass_boundaries`,
`seams_include_reader_breaks_without_machine_flags`,
`seams_keep_sparse_verse_and_uncertain_edge_objects`,
`seams_do_not_borrow_other_unit_context`,
`seams_deduplicate_tasks_without_losing_reasons`,
`report_keeps_uncertain_objects_and_seams_visible`, and
`adjudication_rejects_stale_evidence`. Run `cargo test`.

**Human verification:** Browse several complete and unresolved seam cards,
including a shared page and a skipped duplicate block. Confirm both source edges
and adjacent visible reader pages are legible and numbered line IDs are stable.

### 6.2 Add a capped Haiku reviewer using the existing Anthropic key

Implement `SeamReviewer`, `ApiSeamReviewer`, `SeamObservation`, and
`review_seam_tasks` in `seam_review.rs`; expose the module and add the thin
`facsimile_seam_review` binary. Reuse `crate::secrets::load`, `llm::Client`,
`ContentBlock`, `HAIKU`, `b64`, and `UsageTracker`; do not extend the unrelated
`Vision::review_line` trait or mutate `review.rs` behavior. Construct/load secrets
lazily only for explicit live review. Tests use a stub `SeamReviewer`.

CLI contract: `--tasks <review-tasks.json> --out <new-observation-directory>
--max-calls <positive-count> [--model <id>] [--offline] [--resume] [--dry-run]`.
Default model is the existing `HAIKU` constant. Default secret location is the
repository's existing `secrets.json`, using the repository-root path explicitly;
do not rely on the caller's current directory. No key setup is required. Offline
and dry-run modes construct no client and expose task count/crop sizes/schema;
offline can inspect cached observations. Live review is an explicit command,
not an automatic consequence of `run`, report generation, or implementation.
The task cap counts logical uncached submissions; record client transport retries
as the existing client's behavior and do not promise an exact dollar cap. Stop
at the cap with remaining tasks marked pending. Set maximum output to 2,000
tokens per task, validate complete enumerated line IDs, and retain malformed raw
responses as failures without manufacturing a verdict.

Use a neutral versioned prompt implementing the Design contract. Pass the JSON
schema through the existing client where supported; ordinary local response
validation remains mandatory. Send focused PNGs with a longest side of at most
1,400 pixels, preserving enough glyph resolution; tile oversized neighborhoods
with overlap and record exact crop-to-original transforms. Cache observations by
model, prompt/schema digest, task metadata and crop hashes. Persist response,
usage, and error atomically after each task. Persist `in_flight` before a request;
an interrupted uncertain request becomes `delivery_unknown` and is not silently
resubmitted by `--resume`. Explicitly selecting it in a new attempt is required.
Never log the key or serialize an HTTP Authorization header.

**LLM verification:** Run `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test seam_review`, and
`cargo build --bin facsimile_seam_review`. Add
`seam_review_offline_never_loads_secrets`,
`seam_review_reuses_existing_secret_loader`,
`seam_review_caps_uncached_calls`, `seam_review_resume_reuses_hashed_observations`,
`seam_review_does_not_resubmit_unknown_delivery`,
`seam_review_rejects_missing_or_unknown_line_ids`,
`seam_review_keeps_model_and_machine_verdicts_separate`, and
`seam_review_does_not_write_unit_table`. Run the Python suite for observation
import validation, including `haiku_import_rejects_stale_task_hash`. Run
`cargo test` before ending the phase. No paid API calls are required to pass.

**Human verification:** Read one exported neutral prompt and its crop pair. Run
dry-run against real tasks and inspect the call cap. An explicitly chosen small
live pilot may then use the existing key; compare its observations with source
images and a human read. Pilot absence does not block offline audit delivery.

## Phase 7 — Calibration, full-corpus execution, and durable delivery — Suggested implementer: Opus

### 7.1 Calibrate against independent losses and preservation controls

Create synthetic integration fixtures with removed head/foot/interior lines,
short verse, partial glyph clipping, merged blocks, missing retained canvases,
out-of-order placement, and PDF overpainting. Include harmless removal of a
redundant scan and harmful removal of the sole complete copy. Require correct
type and source/reader localization for each known injected defect; count
uncertain results as unresolved, not hits. Use test-created images/PDFs, without
copying local archive evidence into ordinary tests.

For real-source calibration use known restored neighborhoods as retained
controls, source-only inventory review, and an independent review panel selected
before detector outcomes. Select 25 body pages per tomus deterministically by
hashing snapshot identity plus tomus/page, and add distinct mandatory seam cases:
Romulus v1/38, Pericles v1/214–215, Lysander v1/535, Dio v2/580, Antonius v2/562,
and partial-duplicate openings in Alexander/Phocion/Otho. Record that mandatory
cases are an additional targeted panel, not a random sample. Split reviewed units
into fixed development and validation sets by stable unit-ID hash; expose the
split before tuning. Tune segmentation/render-comparison tolerances on development
evidence, then freeze code/settings/source-policy/input hashes and validate.
If validation leads to revisions, preserve predictions and label later results
post-validation regression; do not claim new independent validation.

Add ignored Rust source-PDF checks `coverage_trace_real_units_match_production`
and `coverage_trace_full_corpus_pixels_unchanged`; they require exact scans and
Poppler, fail clearly on missing prerequisites, and compare traced/untraced
decoded canvases/TeX without historical archives. Full fresh-trace binding and the
source policy remain separately verified even after these tests pass. Report
per-type injected sensitivity, reviewed candidate precision, independent-panel
misses, inventory uncertainty, and uninspected regions with denominators.

**LLM verification:** Run both new Python suites, including
`calibration_detects_injected_head_foot_and_interior_losses`,
`calibration_distinguishes_redundant_and_sole_complete_copy`,
`calibration_keeps_unresolved_separate_from_hits`,
`calibration_panel_selection_precedes_scores`, and
`calibration_freeze_rejects_changed_settings`. Run `cargo fmt --all`,
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test`, and `cargo build --release --bin facsimile`. Select the real tests
explicitly: `cargo test --release --lib coverage_trace_real_units_match_production
-- --ignored --nocapture` and `cargo test --release --lib
coverage_trace_full_corpus_pixels_unchanged -- --ignored --nocapture`.

**Human verification:** Review the independent page panel and all mandatory seam
examples against source and compiled readers. Check that false positives,
source-damage cases, segmentation failures, and missed injected losses remain
visible. Agree the frozen tolerances before interpreting full-corpus findings.

### 7.2 Execute all eight tomi, document limitations, and archive evidence

Finish `README-facsimile-line-audit.md` with concrete commands and prerequisites,
then update the current README audit section in place to link it. Run a full
offline audit using the accepted current snapshot, full-corpus trace, and frozen
settings/policy. Inspect all 68 units, every requested source page including
duplicates, every body-image occurrence, and every derived seam. Write
requested/completed/unresolved counts and failures to the manifest. Record runtime,
cache size, and scope; do not reuse historical page counts or claim that an exit
code of zero means complete Latin coverage.

Suggested command sequence after all CLI steps exist (PowerShell, repository
root; choose fresh folders if these names already exist):

```powershell
python tools/facsimile_line_audit.py capture --reader-dir build_facsimile --snapshot-dir tmp/pdfs/facsimile_line_audit/snapshots/accepted_2026_10_10
python tools/facsimile_line_audit.py inventory --snapshot tmp/pdfs/facsimile_line_audit/snapshots/accepted_2026_10_10 --run-dir tmp/pdfs/facsimile_line_audit/runs/source-review-1
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out tmp/pdfs/facsimile_line_audit/traced-build-1 --offline --no-compile --coverage-trace tmp/pdfs/facsimile_line_audit/traces/full-1
python tools/facsimile_line_audit.py run --snapshot tmp/pdfs/facsimile_line_audit/snapshots/accepted_2026_10_10 --trace-dir tmp/pdfs/facsimile_line_audit/traces/full-1 --run-dir tmp/pdfs/facsimile_line_audit/runs/all-1
python tools/facsimile_line_audit.py report --run-dir tmp/pdfs/facsimile_line_audit/runs/all-1
cargo run --bin facsimile_seam_review -- --tasks tmp/pdfs/facsimile_line_audit/runs/all-1/review-tasks.json --out tmp/pdfs/facsimile_line_audit/runs/all-1/haiku-observations --max-calls 10 --dry-run
```

Source/duplicate adjudication and freezing may produce a new policy/settings
identity; capture a new snapshot/run with those versions before final execution.
The example folders show operation order, not authority to ignore an identity
mismatch. Include exact adjudication and optional live-review syntax in the guide.
No LuaLaTeX rebuild is required merely to audit captured current PDFs. If a new
reader is deliberately captured, compiling it and inspecting its TeX/PDF are
required and its identity must be reported as a new target.

Repeat with `--resume` and compare byte-identical stable inventories, logical
lines, ledger, seams, and tasks; timings and model responses are excluded. Compare
one-worker/four-worker results if parallel inventory is implemented; do not add
worker complexity merely to satisfy a check. Rehash accepted inputs and production
policies before/after. Archive the snapshot, source policies/reviews, settings,
trace, crops, report, observations, adjudications and manifest into a new dated
directory under `archive/`; verify every archived digest without modifying prior
archives. Add a dated completion note in `llm_artifacts/` with findings, unresolved
work and exact identities. Newly discovered losses become a separate reviewed
repair worklist; this implementation does not activate repairs.

**LLM verification:** Run `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`,
`python -m unittest discover -s tools -p test_inner_audit_common.py`,
`python -m unittest discover -s tools -p test_facsimile_line_coverage.py`, and
`python -m unittest discover -s tools -p test_check_portable_tests.py`.
Stage intended new implementation files before
`python tools/check_portable_tests.py --offline --log
tmp/facsimile-line-audit-portable-tests.log`; preserve the unrelated preexisting
Markdown modification. Add `full_audit_reports_requested_and_completed_scope`,
`full_audit_resume_has_identical_stable_records`,
`full_audit_never_claims_complete_with_unresolved_source_inventory`, and
`archive_verification_rejects_changed_crop`. Inspect the full manifest for all
eight tomi/68 units and the actual captured counts, verify all evidence links and
archive digests, and confirm zero unexpected production/source changes.

**Human verification:** Open the full report, inspect representative complete
seams from every tomus, and read every proposed loss plus all unresolved
ownership/equivalence cases before a repair is requested. Review the completion
note's denominators and limitations. Confirm a source line is visibly present in
the reader for each accepted retained control. Findings and unresolved observations
are acceptable audit outcomes; silence is not evidence of exhaustive completeness.

# Acceptance checklist and continuation instructions

- Independent source inventory includes unmasked head/foot ink and skipped scans;
  production crops/bands cannot define its expected content.
- Reviewed logical-passage equivalence permits one complete duplicate occurrence,
  preserves unique continuations, and never waives a missing part using a partial
  copy or naturally repeated wording.
- Production tracing shares execution with ordinary extraction/packing and leaves
  canvas pixels and TeX unchanged; no test-only reconstruction is mistaken for the
  executed production trace.
- Coverage binds to actual captured reader occurrences, verifies visible PDF
  regions, records earliest loss stages, and keeps unresolved observations explicit.
- Every source/unit/reader seam is inventoried, including apparently clean seams;
  both seam and line reports use one ledger and one evidence renderer.
- Haiku is optional, uses the existing secret loader/client, is capped and cached,
  preserves unknown delivery, and writes supplemental observations without changing
  production decisions or Latin.
- Ordinary and portable tests pass without scans/secrets/archives; real-source
  verification is explicitly invoked; formatting, strict Clippy, and the full Rust
  suite pass locally.
- Full-corpus identities, reviewed samples, findings, uncertainties, and durable
  evidence are recorded honestly. Historical artifacts remain on disk.

An implementer resuming this plan should read the current production APIs and the
latest dated delivery note first, then work through the numbered steps without
renumbering them. Record completed steps and evidence in a separate dated
implementation note under `llm_artifacts/`. If an API differs from this plan,
revise the affected plan text in place and retain identifiers; do not invent a
parallel helper to avoid examining the existing implementation. Code comments
describe current behavior and must not cite plan phases, step numbers, or this
filename.
