# Context

This plan combines an independent source-line coverage ledger and a systematic
seam audit for the **facsimile body** of the eight inner tomi. The audit discovers
additional missing or partially preserved Latin after the accepted Tier-A and
Tier-B repairs. It compares original source images with the accepted reader PDFs;
it does not repair the generator or rewrite Latin.

The implementation is Python. The work consists principally of PDF inspection,
image registration, line-region analysis, JSON evidence, optional Haiku requests,
and HTML reports, and the repository already has Python infrastructure for these
operations. The initial implementation adds no Rust modules, binaries, tracing
flags, or changes to the production generator. Existing source traces are optional
supplemental evidence. Discovering where a line is missing does not require
identifying the exact generator stage that discarded it.

The preservation rule is: **every unique source Latin passage must survive
completely at least once, in the correct logical reading order.** Redundant scan
copies need not survive. A clipped duplicate cannot excuse loss of the sole
complete copy, and a page that mixes duplicate and unique prose cannot be waived
wholesale. Naturally repeated words do not establish scan equivalence.

The user does not want a new unit-test suite for this audit script. Verification
therefore uses a small set of disposable end-to-end omission/control cases,
reviewed real-source examples, full-run coverage checks, and a determinism rerun.
Do not add `test_*.py` files, new Rust tests, or a mock framework. Runtime schema,
identity, bounds, and response validation remain necessary to prevent misleading
findings. Preserve the existing repository test suites.

# Current state of the world

The branch is `plan/facsimile-line-coverage-seams`, created from
`plan/inner-tomi-audit-repairs` on 2026-10-10. The existing modification to
`llm_artifacts/2026_10_09_22_36_TIER_B_DELIVERY_REVIEW.md` belongs to earlier work;
preserve it and exclude it from this work's commits.

Relevant implementation and evidence:

- `tools/audit_inner_tomi.py` provides hashing, safe output writing, original PDF
  rendering, PDF geometry, TeX-reference parsing, displayed-image occurrence
  enumeration, reader physical pages/folio mapping, OCR, source/reader passage
  alignment, duplicate corroboration, image registration, and evidence crops.
  Reuse these implementations where their contracts fit the new audit.
- `map_reader_pages` depends on a historical snapshot and pinned reader counts.
  Extract its snapshot-independent mapping logic rather than copying it.
  `extract_source_regions` uses embedded OCR and production boundary fractions;
  it cannot define the independent expected-line inventory. Likewise,
  `read_approved_exclusions` removes whole skipped pages from expected material,
  which does not implement the new logical-passage coverage requirement.
- `tools/requirements-inner-audit.txt` pins NumPy, Pillow, OpenCV, scikit-learn,
  RapidFuzz, and PyMuPDF. These are the core Python dependencies.
- `src/facsimile/mod.rs::run_facsimile` builds the production images/TeX/PDFs.
  `reflow.rs::unit_body_strips` masks pages, detects/crops columns, selects bands,
  applies furniture/boundary trims, and restores known protected bands.
  `strips_in_reading_order` and `retain_body_canvases` apply additional ink floors;
  the composer scales, sometimes splits strips, and packs fixed-width canvases.
  These explain possible loss mechanisms but are not to be ported into Python.
- Production body rasters use 300 DPI. `columns.rs::INK_THRESHOLD` is 170.
  Fingerprint-scoped wide-column fits mean a page's actual scale can differ from
  the requested global scale. The audit must measure source/reader geometry.
- `skip_known_duplicate_source_page` is unit/source/page scoped. The numeric
  ledger `src/facsimile/fixtures/inner_tomi_repair_source_trace.json` contains
  12 reviewed duplicate records with `skip_pages`, `retain_sequence`, and source
  preservation evidence. Early records have page-pair `occurrences`; later
  records instead have `reviewed_lines`. Neither field is universal.
- `src/facsimile/fixtures/inner_tomi_audit_repair_source_trace.json` records the
  subsequent audit/repair decisions. Both ledgers and their source-region evidence
  can seed duplicate and preservation controls. A skip list alone is not a map
  proving every line of two scans equivalent.
- Existing `RetainedSourcePageTrace` and `audit_traced_unit` are test support for
  measured cases. Optional imported traces must retain their actual scope.
  Production masks whiten pixels and do not delete rows; historical deletion
  metadata does not prove that a current row-compaction operation ran.
- The existing Sonnet `--review-lines` path changes `facsimile_units.json` and
  inspects selected opening bands. Do not invoke it for this audit.
- The user's existing repository-root `secrets.json` contains
  `anthropic_api_key`. Python will load this same file only for explicit live
  Haiku review. It does not need a new credential file or an environment-variable
  setup step. The Rust client remains untouched.

The accepted Tier-B delivery is documented in
`2026_10_10_08_22_TIER_B_COMPLETE_DELIVERY_VERIFICATION.md`: 20 events and
43 measured lines, all restored and accepted. The original 44-line inventory is
historical; the approved Antonius measurement is 22 lines. Preserve both records.
Every Tier-B loss touched a source-page head, foot, or page break. The historical
OCR audit found substantial additional losses but had 311 inconclusive image
registrations; its completed processing does not establish exhaustive recall.

# Desired end state

A Python CLI inventories unmasked source ink for all 68 body units, including
physical pages skipped as duplicate scans. It constructs ordered logical lines,
accounts explicitly for reviewed equivalent scan occurrences, and compares every
required line against its expected neighborhood in the accepted reader PDFs.
The coverage ledger has complete, partial, absent, and unresolved results with
source/reader locators and image evidence. It does not require a rebuild.

The same ledger drives inspection of every source-page transition, unit boundary,
duplicate bypass join, and reader-page break. Optional Haiku comparison adds
independent observations of ambiguous source/reader crop pairs. One HTML report
shows coverage, seams, proposed omissions, uncertainties, and review decisions.

The audit can discover a loss while reporting its generator cause as unknown.
Image matching and OCR propose correspondence; failed registration never proves
absence. Apparently complete endpoints never certify interior coverage. A
completed audit may contain findings and unresolved cases.

No production output, source PDF, transcription cache, unit table, or existing
repair decision changes. New output lives under
`tmp/pdfs/facsimile_line_audit/` and is preserved in a dated archive after delivery.
The work adds no unit-test suite; its verification evidence is retained with the
run. Existing Rust tests remain available as the repository regression check.

# Tech Learnings

A physical source occurrence and a logical passage are different identities.
Different scans of the same printed line may have different skew, contrast, or
clipping. Keep each physical occurrence and its measured geometry, then record
which parts correspond to the same reviewed logical passage.

The reader uses source images, so visual correspondence can establish retention
without a dependable transcription. OCR is a locator for a neighborhood;
source/reader pixels and surrounding order provide the preservation evidence.
The audit must observe actual reader images rather than reproduce the generator's
heuristics and declare that their output is complete.

Coordinates are half-open boxes with origin at the top left. Store original-source
pixels, decoded reader-image pixels, PDF points, and normalized page fractions
explicitly. A PDF can contain a complete embedded image that is clipped or hidden
on the visible page. Inspect displayed regions as well as embedded image content.

# Design contract

## A. Python files, reuse, and commands

Add `tools/facsimile_line_audit.py` for CLI orchestration and
`tools/facsimile_line_coverage.py` for source inventory, duplicate-aware coverage,
seams, and report data. Add `tools/facsimile_seam_review.py` for optional Haiku
requests. Use one coverage implementation and one evidence renderer for ordinary
line findings and seam findings.

Extract reusable PDF/render/path/hash functions into
`tools/inner_audit_common.py` only where sharing is needed. Re-export their old
names from `audit_inner_tomi.py` so existing callers keep working. Leave
historical detector algorithms and capture/freeze policy in the old script.
Avoid a wholesale reorganization of the historical audit.

Add `tools/facsimile-line-source-policy.json`,
`tools/facsimile-line-audit-settings.json`, and
`tools/README-facsimile-line-audit.md`. Reuse core dependency pins. Add
`tools/requirements-facsimile-haiku.txt` for the optional official `anthropic`
Python SDK; during implementation choose and record the installed working version
and pin it there. Offline audit imports must not require this SDK.

The audit CLI has `capture`, `inventory`, `run`, `report`, `adjudicate`, and
`verify` commands. `verify` runs a few disposable end-to-end audit cases and emits
verification evidence; it is not a unit-test framework. Optional flags are
`--units`, `--resume`, and `--trace-dir`. A trace is supplemental and never required.
Changed inputs/settings require a new run directory. Do not reuse a historical
baseline identity for new readers.

## B. Input snapshots and output identity

`capture --reader-dir build_facsimile --snapshot-dir <new-directory>` copies the
two source PDFs, eight inner PDFs and TeX files, unit table, two repair ledgers,
source policy, settings, and audit code. Record SHA-256, role, relative path,
software versions, actual source page counts, and observed reader counts in
`snapshot-manifest.json`. Do not reuse historical counts as expected current
counts. Copy referenced body PNGs when present and always record decoded body
image identities from the PDFs. Missing standalone PNGs must not prevent auditing
a complete reader.

Count displayed occurrences, including two uses of the same PDF image object.
Check each against ordered TeX references and available decoded PNG pixels.
Repeated equal images can legitimately have indistinguishable pixel identities;
record their ordered occurrences rather than pretending the hashes are unique.
An ambiguous asset correspondence remains unresolved. Missing PDFs, escaping
references, truncated reference/occurrence counts, or input changes during capture
are errors. Exclude secrets, outer imposed PDFs, and prior archives.

A run writes `source-inventory.json`, `logical-lines.json`,
`coverage-ledger.json`, `seams.json`, `reader-map.json`, `errors.json`,
`review-tasks.json`, `run-manifest.json`, `index.html`, and `crops/`.
Optional model observations and human decisions live in separate files. Stable
records have sorted keys/lists and no timestamps, absolute paths, or timings;
operational metadata goes in the manifest. Hash crop bytes and record sets.
Cache original renders, OCR, and image comparisons by input/settings/tool identity.

## C. Independent expected lines and source policy

`SourceOccurrence` contains `occurrence_id`, source digest/index, one-based
physical page, DPI/raster size, full-source box, crop hash, classification,
possible owners, segmentation certainty, and optional raw OCR. Classification is
`body`, `heading`, `annotation`, `furniture`, or `uncertain`. IDs derive from
source digest/page/box/detector settings and do not change with a review verdict.
Merged regions have measured children or stay unresolved; do not equate a merged
production band with one printed line.

Inventory uses unmasked full-page source renders at 300 DPI. Initial inspection
ROI is `[0.40, 0.00, 1.00, 1.00]` in fractions, preserving page heads/feet. Flag
components intersecting its left boundary and permit a reviewed source-only
widening. Estimate the Latin gutter from original-source geometry; do not use
production column crops or retained bands to define expected content. Full-page
context remains available. Fixed top/bottom percentages cannot exclude prose.
Missing OCR cannot remove visible source ink.

`SourcePolicy` is versioned and source-digest bound. It stores reviewed ownership,
furniture classifications, segmentation overrides, and duplicate maps. Production
title/body fractions and original OCR are locator suggestions; exclusions require
source-only evidence and a recorded decision. Unreviewed classifications remain
hypotheses, and uncertainties stay visible in denominators. Source labels and
annotations receive separate coverage categories rather than default exclusion.

## D. Logical passages and duplicate scans

`DuplicateGroup` records logical owner, physical source occurrences, reviewed
passage boundaries, canonical sequence, occurrence-to-logical-region mappings,
and evidence. Seed from existing approved records without parsing narrative text
or deriving source pairs solely from page arithmetic. New equivalence decisions
require source-only review. Divide shared duplicate pages by actual passage owner
before equivalence, especially Eumenes/comparison material around v2/101–105.
Keep incomplete second openings of Alexander, Phocion, and Otho explicit.

`LogicalLine` records a stable ID, unit/sequence position, required source regions,
equivalent physical mappings, classification, and source/segmentation uncertainty.
Use a complete reviewed occurrence as the reference where available. A partial
occurrence maps solely to its surviving span. Different wrapping or complementary
fragments require explicit region mappings; otherwise equivalence stays unresolved.
No reconstruction of source glyphs from Latin grammar is permitted.

Require a complete ordered occurrence or an explicitly mapped union of contiguous
pieces covering the required line in the proper reader position. Do not assemble
coverage from arbitrary occurrences elsewhere in the work. Natural repeated
language keeps distinct logical positions. Additional full copies satisfy coverage
and receive a separate repeat observation. They are not a repair instruction.
Source-partial lines are distinguished from newly lost reader material.

## E. Python correspondence and evidence strength

Reuse the existing source-only OCR passage index and ordered alignment to propose
reader neighborhoods, respecting approved logical duplicate maps. Adapt call
sites to explicit captured inputs; do not copy the aligner or let reader OCR train
the source index. Preserve alternative placements and ambiguity. Add source-line
image registration inside bounded reader neighborhoods, including adjacent pages.
Registration may operate on overlapping strips and multiple anchors because a
source page is reflowed across several reader pages. A whole-page affine transform
is not assumed to exist.

`LineCorrespondence` records source occurrence/region, reader image occurrence,
reader physical page/folio provenance, source-to-reader transform, measured scale,
matched anchors, visible region, pixel differences, and limitations. Derive scale
from matched source/reader ink, not a fixed assumed generator scale. Inspect left
and right ends plus ascenders/descenders. Split/merged lines and skew can require
piecewise correspondences. Registration failure is unresolved, never absence.

Each physical occurrence receives `complete`, `partial`, `absent`, or
`unresolved`. `complete` requires whole required line coverage in its expected
position, corroborated by ordered context and source/visible-reader image
agreement. Partial/absent findings require bounded before/after anchors and crop
review; OCR disagreement or a failed image search alone produces a candidate or
unresolved result. Retain raw measurements; scores are not probabilities.

Inspect actual rendered reader pages at 300 DPI in addition to decoded embedded
images. A complete embedded image does not clear clipping or overpainting. Use
shared PDF transforms and rendered crops to inspect the actual visible endpoints.
Overpainting, ambiguous transforms, and registration uncertainty stay visible.

Store `loss_stage` as `unknown` unless authenticated existing trace evidence
establishes a particular operation and binds to the captured reader pixels. Do not
infer an executed mask/trim solely from a source-code table or line position.
No new production tracing or rebuild is a prerequisite of this audit.

## F. Seams and Haiku review

Derive every seam from the source/logical inventory and reader placements:
source-page transitions, unit openings/closings and shared boundaries, duplicate
bypass joins, and adjacent body-reader pages. Include apparently complete seams.
Physical duplicate edges remain inventory entries, marked redundant only after
logical coverage is established. Task deduplication preserves all seam reasons.

Include the last three owned body lines before a seam and the first three after
it, uncertain objects between anchors, and relevant headings/annotations. Extend
sparse context within its unit, without borrowing another unit's prose. Include
original source edges that a production crop may have omitted. Save all requested,
inspected, and unresolved seams.

Haiku receives labeled source context and corresponding visible-reader crops with
numbered immutable line IDs. Hide machine loss verdicts and historical repair
labels. Ask whether each line is complete, partial, absent, or unclear, plus
object type, matching reader crop IDs, and a brief reason. It must see both sides
of reader-page breaks and relevant equivalent scans for duplicate questions.
Without a located reader neighborhood, the task is inconclusive rather than an
invitation to infer absence. Limit each task to six body lines with overlapping
context for longer spans.

Haiku observations cannot change machine coverage, source policy, or production
settings. Human adjudication is separate and bound to exact evidence. A model
finding requires a source/reader read before becoming a confirmed loss.

## G. Settings and review interchange

Initial settings: `source_dpi: 300`, `reader_dpi: 300`,
`source_roi: [0.40, 0.00, 1.00, 1.00]`, `ink_thresholds: [170, 210]`,
`component_connectivity: 8`, `seam_context_lines: 3`, and
`task_max_body_lines: 6`. Estimate glyph height from median component heights
between 8 and 80 source pixels; fewer than ten suitable components makes grouping
uncertain. Propose line groups when vertical overlap is at least half the smaller
component height or centers differ by at most half typical glyph height. Preserve
cross-threshold disagreements and unresolved merged baselines for review.

Start registration with source/reader scale estimates from contextual matches,
refine inside a bounded neighborhood, and retain measured transformation residuals.
Put search bounds and tolerances in settings. Their final values require the
Phase 7 calibration; do not hard-code a universal pixel or similarity threshold
as proof of complete glyph retention. Deterministic registration seeds and tie
breaking must be recorded. Changed settings invalidate cached comparisons.

Source-review documents contain schema version, snapshot/inventory digest,
reviewer, and decisions. Each decision binds occurrence ID, crop hash,
classification, owner or explicit uncertainty, reason, and any measured child
boxes. Duplicate reviews add source-only evidence and explicit logical-region
maps. Human loss reviews also bind run/ledger identities and source/reader crop
hashes, with `confirmed_loss`, `retained`, or `uncertain`. Reject stale IDs or
hashes. Model observations bind their task identity and are stored separately.

# Implementation sequence

Retain phase/step numbers when revising this plan. Every step has commands and
human verification, but no step requires adding unit tests. The small `verify`
command exercises the complete audit on disposable evidence and writes a case
report. Existing source-PDF archives supply optional controls, not prerequisites
for running these checks. Avoid changing Rust or the historical audit algorithms.

## Phase 1 — Shared Python infrastructure and immutable inputs — Suggested implementer: Sonnet

### 1.1 Reuse PDF and evidence helpers with minimal extraction

Create `inner_audit_common.py` and extract the needed implementations of
`hash_file`, `json_hash`, `validate_box`, `page_geometry`, `normalized_pdf_box`,
`pixel_bounds`, `parse_tex_refs`, `enumerate_image_occurrences`,
`read_footer_folio`, path/write guards, and render/cache helpers from the old
audit. Preserve old signatures and re-export names. Keep optional imports lazy.
Extract the snapshot-independent reader-map loop as
`map_reader_occurrences(reader_inputs, expected_counts=None)`; the historical
adapter supplies its existing pinned counts. The new adapter supplies actual
captured inputs. Require matching occurrence/reference counts and explicitly
record ambiguous image identity rather than silently truncating `zip` inputs.

**LLM verification:** Run `python tools/audit_inner_tomi.py check` when its tools
are installed. In a separate temporary directory, compare the adapted reader map
for one current tomus with PDF image counts and ordered TeX references. If local
historical evidence exists, compare the old adapter's semantic map with its saved
map without writing into the archive. Record unavailable optional replay evidence.

**Human verification:** Inspect representative body, repeated-image, separator,
front-matter, and padding map entries, including folio provenance.

### 1.2 Add schemas, capture, and safe output identity

Create the core CLI/module and policy/settings files. Implement
`capture_snapshot`, `load_snapshot`, `validate_source_policy`, and the `capture`
command. Copy to a fresh snapshot, compare pre/post-copy digests, record actual
counts, and refuse overwrite. Decode embedded body images even without PNG files.
Keep caches outside the immutable snapshot and never capture `secrets.json`.
Initialize known duplicate seeds without claiming their missing region maps exist.

**LLM verification:** Run `python tools/facsimile_line_audit.py --help`,
`python -m py_compile tools/facsimile_line_audit.py
 tools/facsimile_line_coverage.py tools/inner_audit_common.py`, and a current-build
capture into a new directory. Inspect source/copy hashes, actual reader counts,
and the captured path list. Retry capture into the same directory and require a
clear refusal. Check a disposable missing-PDF input fails without changing originals.

**Human verification:** Read the manifest, confirming accepted current inner PDFs,
correct source scans, no credentials, and visible duplicate-policy uncertainties.

## Phase 2 — Read existing provenance without generator changes — Suggested implementer: Sonnet

### 2.1 Implement an optional source-trace adapter

Implement `load_optional_traces` in `facsimile_line_coverage.py`, enabled solely
by explicit `--trace-dir`. Read existing trace JSON through schema-aware adapters;
keep unit/source/page identities, original boxes, retained indices, and declared
scope. Never infer retained index as page minus unit start. A trace that lacks
source/build bindings is a diagnostic locator, not authoritative coverage evidence.
Do not run ignored Rust tests to generate new expected inventory or add production
tracing. A missing trace directory must not block a normal offline audit.

**LLM verification:** Inspect one existing trace when available. Run an inventory
command with no trace directory and confirm normal progress. With a deliberately
mismatched trace in a disposable folder, require an explicit stale/inconclusive
record rather than replacing snapshot identity.

**Human verification:** Read the trace scope label and one mapped physical source
page. Confirm historical coordinates are not displayed as current reader locators.

### 2.2 Separate coverage evidence from causal diagnosis

Implement `attach_trace_diagnostics`. Leave coverage based on source/reader images.
Populate loss-stage metadata only where provenance and decoded reader pixels bind
the existing trace to the inspected build. Otherwise report `unknown` and preserve
any suggested stage in a clearly labeled hypothesis field. Do not port the Rust
cropper, packer, mask logic, or resampling rules into Python. This step remains
small even when no authenticated trace is available.

**LLM verification:** Compare the same source/reader result with and without the
optional trace adapter; line coverage and seam counts must agree. Inspect a stale
trace case and require `loss_stage: unknown`. Run Python compilation checks.

**Human verification:** Inspect a finding whose missing line is clear while its
cause remains unknown. Confirm the report does not demand a fresh rebuild to
establish that finding.

## Phase 3 — Independent source-line inventory — Suggested implementer: Opus

### 3.1 Inventory unmasked source ink on every owned source page

Implement `inventory_source_page`, `inventory_source_units`, and `inventory`.
Render all inclusive source ranges, including skipped scans; share rendering of
physical pages shared by units. Use the full-height ROI and two-threshold component
analysis in the Design contract. Preserve page-edge ink, short verse, faint
fragments, and ambiguous merged lines. Keep classification separate from detection.
Source OCR is optional annotation; absent OCR does not remove ink. Do not call
production band/furniture routines or use `extract_source_regions` as expectations.
Save full-source overlays and every detected/uncertain object.

**LLM verification:** Compile the Python files, then inventory
`--units theseus,pericles,cato_minor,antonius` into a fresh run. Inspect counts
of represented source regions and uncertainty records at each source-page edge.
Change a disposable copy of production crop metadata and confirm the independent
inventory is identical. Retain the real-source overlays for review.

**Human verification:** Inspect source heads/feet in these units, including verse,
Antonius v2/562, and clipped/Greek-gutter uncertainties. Confirm each visible Latin
line is represented by a line or unresolved region.

### 3.2 Review source ownership and classifications independently

Implement `apply_source_policy`, `validate_source_inventory`, and source-only
review sheets. `adjudicate --source-reviews <file>` imports source-digest/crop-bound
ownership, classification, and child-box decisions into a new policy version.
Production boundary fractions are locator hypotheses rather than exclusions.
Require a unique logical owner for reviewed body regions; unresolved boundaries
retain possible owners and cannot count as complete. Detect visible ink between
or outside reviewed line boxes. Keep headings/annotations distinct from furniture.

**LLM verification:** Import a small reviewed source-only boundary file and inspect
its resulting policy. Change a crop hash in a disposable review and require
refusal. Inspect ownership coverage on a shared page, an indented verse region,
and a sparse ending. Do not overwrite the prior snapshot.

**Human verification:** Review boundaries before viewing reader outcomes. Confirm
source body starts/ends around display titles and that uncertain regions have
not been removed merely to simplify matching.

## Phase 4 — Logical passages and reviewed duplicates — Suggested implementer: Opus

### 4.1 Resolve existing duplicate groups at passage level

Implement `seed_duplicate_groups` and `resolve_duplicate_groups`, with source-only
comparison sheets. Import numeric approved pairs/regions where supplied; records
without explicit region equivalence stay incomplete until reviewed. Never derive
complete equivalence solely from `skip_pages`, similar OCR, or page arithmetic.
`adjudicate --duplicate-reviews <file>` writes source-bound mappings in a new
policy version. Split shared duplicate pages by passage owner before collapsing
logical material. Preserve partially scanned openings and unique continuations.

**LLM verification:** Export all 12 known duplicate records and compare their seeds
with the numeric ledgers. Inspect Eumenes/comparison v2/101–105, Alexander,
Phocion, and Otho source pairs. Reject a disposable map that leaves a unique span
uncovered or claims a clipped occurrence covers a complete line.

**Human verification:** Confirm source equivalence and preservation boundaries for
each group. No new production skip decision is installed by this step.

### 4.2 Construct the ordered logical inventory and denominators

Implement `build_logical_lines` and `validate_logical_sequence`. Link equivalent
physical occurrences to one logical line position, including explicit partial
span mappings. Reject cyclic/contradictory relationships. Keep physical source
copies, unique logical lines, source-partial regions, headings/annotations,
uncertain objects, and waived redundant occurrences as distinct counts.
Source-partial status never excuses additional loss from the reader.

**LLM verification:** Print logical sequences across a reviewed duplicate block and
shared unit boundary. Confirm all required regions are mapped and each physical
copy remains in evidence. Inspect a disposable partial-only map: the missing
complete portion must remain required or unresolved, not waived.

**Human verification:** Read the canonical passage sequence and source pairs,
checking that logical collapse loses neither a unique continuation nor a line
missing from the surviving partial scan.

## Phase 5 — Coverage against accepted visible readers — Suggested implementer: Opus

### 5.1 Locate ordered reader neighborhoods using existing alignment

Implement `locate_reader_neighborhoods` and `build_line_correspondences`. Reuse
source-only OCR indexing, ordered anchor alignment, and existing registration
primitives from the audit, adapting explicit snapshot inputs. Source inventory
still includes lines OCR misses. Apply reviewed duplicate logical maps before
alignment. Locate overlapping strips/lines rather than requiring one transform
for an entire reflowed source page. Keep alternative placements and registration
residuals. When source/reader OCR is poor, use image anchors and adjacent established
matches; otherwise leave correspondence unresolved. No Rust rebuild is required.

**LLM verification:** Run scoped alignment for the four inventory units and one
reviewed duplicate unit against captured current PDFs. Inspect raw matches,
ambiguous alternatives, measured scales, and physical reader locators. Confirm
that a reference/occurrence mismatch cannot silently drop a reader page.

**Human verification:** Read source/reader correspondence cards around known
restored lines, checking that the expected neighborhood is correctly located and
that the source line is not being matched to similar prose elsewhere.

### 5.2 Evaluate whole-line visible coverage and uncertain omissions

Implement `evaluate_occurrence_coverage`, `evaluate_logical_coverage`, and
`verify_reader_visibility`. Compare original source-line regions to corresponding
visible reader pixels, with piecewise transforms where reflow demands them. Inspect
both line ends and vertical glyph extent. For split lines, evaluate the union of
adjacent ordered pieces and flag slicing through ink separately. Compare embedded
body pixels and visible PDF rendering so clipping/overpainting cannot be cleared
merely by image-object presence.

Complete logical coverage requires the proper ordered occurrence or an approved
contiguous complete union. Partial/absent candidates carry before/after context and
source/reader crops. Failed registration, bad OCR, missing source glyphs, and
uncertain ownership stay explicit. Keep machine candidates separate from human
confirmed losses; the script never reconstructs or corrects Latin.

**LLM verification:** Run scoped coverage and inspect complete/partial/absent/
unresolved counts. Use the disposable cases introduced in Phase 7 to check a
removed edge line, clipped glyph, and overpainted PDF region when available.
Keep differences and source/reader images as run evidence.

**Human verification:** Inspect source/reader triplets for known restored head/foot
lines, verse, the Antonius block, and any proposed new loss. Complete means whole
visible glyphs at the right position, not readable Latin inferred by OCR.

## Phase 6 — Systematic seams and Python Haiku reviewer — Suggested implementer: Sonnet

### 6.1 Report all seams and support evidence-bound adjudication

Implement `build_seam_inventory`, `build_review_tasks`, `write_coverage_report`,
and `report`/`adjudicate`. Build every source/unit/duplicate/reader seam with
three lines per side and all intervening uncertain objects. Use one ledger,
reader mapper, and evidence renderer. Untouched crop bytes remain hashed;
annotations appear on display copies. Apparently complete seams remain browsable.
Separate machine coverage, model observation, and human verdict columns. Reject
stale review identities; import decisions without changing machine records or
production policies.

**LLM verification:** Compare seam totals against source/logical transitions and
adjacent reader body pages. Browse a source page with fewer than three body lines
and confirm context stays within its unit. Import a disposable stale review and
require refusal. Check HTML evidence links and escaping.

**Human verification:** Inspect complete and unresolved seam cards, shared-page
boundaries, skipped-duplicate joins, and reader page breaks. Both original edges
and adjacent visible-reader context must be legible.

### 6.2 Call Haiku from Python with the existing key

Implement `review_seam_tasks` in `facsimile_seam_review.py`, using the official
Anthropic Python SDK. Add CLI flags `--tasks`, `--out`, `--max-calls`, `--model`,
`--offline`, `--resume`, and `--dry-run`. Default model is `claude-haiku-4-5`,
matching the repository's current Haiku setting; record the requested and returned
model identities. Default key file is the repository root resolved from the script
path, followed by `secrets.json`; read its `anthropic_api_key` only for live calls.
Do not print, snapshot, or hash credential contents. SDK import and client creation
are lazy so core audit/dry-run/offline work without a key or SDK.

Use a neutral versioned prompt and PNG image blocks. Require JSON with precisely
the requested line IDs and allowed verdicts; validate locally regardless of
structured-output support. Focused crops have a longest side of at most 1,400
pixels; tile larger context with overlap rather than making glyphs illegible.
Record image transforms. Save raw response, token usage, parsed observations,
errors, model/prompt/task/crop identities after each call.

Require a positive explicit `--max-calls` for live review. The cap counts uncached
logical tasks, not an exact currency budget. Use SDK `max_retries=0` and an explicit
120-second request timeout for simple submission accounting; retain request errors
without automatic retry. Set `max_tokens=2000`. Cache successful observations by
model/prompt/schema/task/crop identity. Persist `in_flight` before submission;
interruption or ambiguous network delivery becomes `delivery_unknown` and is not
silently resubmitted on resume. A new explicit attempt can select that task.
Dry-run and offline submit nothing and show pending/cached task counts.

Official SDK usage reference:
[Anthropic Python SDK](https://github.com/anthropics/anthropic-sdk-python).
Image request/legibility reference:
[Claude vision documentation](https://platform.claude.com/docs/en/build-with-claude/vision).
Verify working SDK/model support during implementation; no price assumption is
part of this plan.

**LLM verification:** Compile the reviewer, then dry-run real tasks and inspect
line IDs, crop sizes, cap, and neutral prompt. Run offline replay with a saved
sample response and inspect strict parsing of missing/unknown line IDs. Check that
no model observation changes the ledger or unit table. An explicitly invoked small
live pilot may use the existing key; implementation completion requires no paid
calls. No unit-test mocks are added.

**Human verification:** Read one prompt/crop pair and cached observation. If a
pilot is run, compare its complete/absent judgments with a human source/reader read.
Disagreements remain visible and do not become automatic repair decisions.

## Phase 7 — End-to-end calibration and full-corpus delivery — Suggested implementer: Opus

### 7.1 Verify the audit on a few known omissions and controls

Implement `run_verification_cases` behind the CLI `verify`. Create small disposable
source/reader images and PDFs with distinct known ink regions and independently
specified expected boxes. Exercise the normal inventory/alignment/coverage/seam
path rather than individual helpers. Retain input images, outputs, and a JSON
case report in the selected verification directory. No `unittest`, `pytest`,
new Rust tests, or large per-function case inventory is needed.

Use these named end-to-end cases as the checklist:

- `retained_control`: complete lines, including short verse, survive in order.
- `head_and_foot_loss`: remove the first and last required body line.
- `interior_block_loss`: remove a distinctive interior multi-line passage.
- `redundant_copy_removed`: retain one complete duplicate; no unique loss.
- `sole_complete_copy_removed`: retain a clipped duplicate; required missing
  portions are flagged rather than waived.
- `shared_page_unique_tail`: preserve duplicate prose but omit its unique
  continuation; the continuation remains a loss candidate.
- `visible_pdf_clipping`: embed a complete image but visibly clip/overpaint a line.

Require correctly located omission candidates or partial findings for known losses
and no unique-loss warning on the retained/redundant controls. An unresolved case
is reported as such and does not count as successful detection. These cases
establish sensitivity to specific examples, not corpus-wide recall. Keep scenario
definitions small and independent of detector-generated expectations.

Calibrate on reviewed real-source controls and a fixed independent panel selected
before scores: 25 body-reader pages per tomus ordered by hash of snapshot/tomus/
page. Add targeted Romulus v1/38, Pericles v1/214–215, Lysander v1/535, Dio v2/580,
Antonius v2/562, and partial-duplicate openings. Record targeted cases separately
from the hash-selected panel. Freeze source policy, settings, code and identities
after calibration. If later review changes parameters, retain earlier predictions
and rerun under a new identity; do not describe the rerun as independent validation.

**LLM verification:** Run `python tools/facsimile_line_audit.py verify --out
 tmp/pdfs/facsimile_line_audit/verification/cases-1`. Inspect each named case's
input/output crops and verdict/localization. Compile changed Python files. Inspect
real-source control results and report the number of unresolved panel regions,
missed known losses, and false alarms with their denominators.

**Human verification:** Read the seven disposable case comparisons and the real
page/seam panel. Confirm the harmless duplicate case is accepted, the partial-only
case is flagged, and actual source damage is distinguished from reader loss.

### 7.2 Run all tomi, verify reproducibility, and archive evidence

Finish the setup guide and update the existing README audit section in place.
Run a full offline audit of the accepted snapshot with the frozen policy/settings.
Account for all 68 body units, requested physical source pages including
duplicates, mapped reader body occurrences, and derived seams. Keep requested,
completed, failed, unreviewed and unresolved counts separate. Input/output identity
and processing completion are not proof of complete Latin.

Example commands after implementation (PowerShell, repository root; choose fresh
folders if these names exist):

```powershell
python tools/facsimile_line_audit.py capture --reader-dir build_facsimile --snapshot-dir tmp/pdfs/facsimile_line_audit/snapshots/accepted_2026_10_10
python tools/facsimile_line_audit.py inventory --snapshot tmp/pdfs/facsimile_line_audit/snapshots/accepted_2026_10_10 --run-dir tmp/pdfs/facsimile_line_audit/runs/source-review-1
python tools/facsimile_line_audit.py verify --out tmp/pdfs/facsimile_line_audit/verification/cases-1
python tools/facsimile_line_audit.py run --snapshot tmp/pdfs/facsimile_line_audit/snapshots/accepted_2026_10_10 --run-dir tmp/pdfs/facsimile_line_audit/runs/all-1
python tools/facsimile_line_audit.py report --run-dir tmp/pdfs/facsimile_line_audit/runs/all-1
python tools/facsimile_seam_review.py --tasks tmp/pdfs/facsimile_line_audit/runs/all-1/review-tasks.json --out tmp/pdfs/facsimile_line_audit/runs/all-1/haiku-observations --max-calls 10 --dry-run
```

Source/duplicate adjudication produces a new policy identity; capture a new
snapshot/run with that policy before final execution. Document exact adjudication,
SDK installation, optional trace, dry-run/live-review, and resume commands. Core
commands need no key, SDK, production rebuild, or LuaLaTeX compilation.

Repeat the exact run with `--resume`; require byte-identical stable inventories,
logical lines, ledger, seams, and tasks, excluding operational timings and model
responses. Verify accepted source/reader/unit/repair-policy hashes before and after.
Archive the snapshot, settings/policy, verification case reports, source review,
crops, ledger, seams, HTML, optional model/human observations, and manifest in a
new dated `archive/` directory. Verify digests without modifying older archives.
Write a dated completion note under `llm_artifacts/` with identities, findings,
uncertainties and limits. New losses become a separate reviewed repair worklist.

**LLM verification:** Run Python compilation and the seven end-to-end cases,
inspect full requested/completed counts and evidence links, and compare resume
record hashes. Run existing repository checks at completion:
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test`; no Rust formatting changes should be necessary because Rust is
untouched. Run the existing portable snapshot check if shared Python changes could
affect it, staging intended new files first. Confirm the Git diff contains solely
intended audit/document files and no production/source changes.

**Human verification:** Open representative complete seams from every tomus and
read every proposed new loss plus unresolved ownership/equivalence cases. Inspect
the completion note's denominators. Accepting the finite verification cases does
not establish exhaustive character-by-character corpus completeness.

# Acceptance checklist and continuation instructions

- The audit, reporting, and optional Haiku client are Python; production Rust,
  accepted reader outputs, and source Latin remain unchanged.
- Expected inventory uses unmasked source pixels including page edges and duplicate
  scans; production exclusions cannot define expected content.
- Duplicate handling requires complete unique passage coverage and correct order,
  preserving shared-page continuations and partial-copy limitations.
- Source/reader registration and visible-pixel evidence drive the ledger; failed
  matching stays unresolved and causal diagnosis may remain unknown.
- All source/unit/duplicate/reader seams are inventoried, including apparently
  complete seams, using the same ledger and evidence renderer.
- Haiku uses the existing key via a lazy Python SDK client; calls are explicit,
  capped and cached, and observations never modify preservation decisions.
- There is no new unit-test suite. Seven small end-to-end cases, real-source
  controls, run coverage checks and a resume comparison provide verification.
  Existing repository suites remain green.
- Findings, uncertainty, scope and identities are reported honestly and all dated
  historical artifacts/evidence are retained.

An implementer should read current audit helper signatures and the latest delivery
note, then proceed through the retained step numbers. Document implementation
progress in a separate dated note under `llm_artifacts/`. If actual APIs differ,
revise affected plan text in place without renumbering. Reuse existing helpers;
do not port the generator or copy an existing matcher. Code comments describe
current behavior and do not cite plan phases, step numbers, or this filename.
