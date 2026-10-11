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
preserve its contents. The user authorized committing all current edits together,
including that document, when this plan revision is complete.

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
run. Each required step has local verification using features available at that
step, with realistic disposable cases prepared before coverage implementation.
Existing Rust tests remain available as the repository regression check.

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

## A. Python files and explicit reuse boundary

Add `tools/facsimile_line_audit.py` for CLI orchestration,
`tools/facsimile_line_coverage.py` for inventory/coverage/seams, and
`tools/facsimile_seam_review.py` for optional Haiku requests. Add
`tools/inner_audit_common.py` for shared low-level infrastructure. Add source
policy/settings JSON and `tools/README-facsimile-line-audit.md`. Reuse
`requirements-inner-audit.txt`; pin a working official `anthropic` SDK version in
an optional `requirements-facsimile-haiku.txt`. Core offline imports must work
without that SDK. No Rust modules, tracing flags, or new unit tests are added.

The common module owns `AuditError`, `CalibrationError`, `progress`,
`progress_heartbeat`, `hash_file`, `json_hash`, `validate_box`, `page_geometry`,
`normalized_pdf_box`, `pixel_bounds`, `relative_path`, `contained`,
`safe_write_path`, `write_json`, `run_external`, `reusable_entry`, `run_directory`,
`render_page`, `parse_tex_refs`, `enumerate_image_occurrences`,
`read_footer_folio`, and the body-asset filename regex. Move these definitions
and their standard-library dependencies once, then import/re-export their names
from `audit_inner_tomi.py`. The common module must not import that script.
Keep optional image/PDF imports lazy. Preserve historical behavior by default;
new callers pass explicit policy/settings through optional keyword arguments.

Extract `map_reader_occurrences(reader_inputs, expected_counts=None)` from the
reader-map loop; the historical adapter supplies its pinned expectations. The
new adapter supplies reader inputs shaped as `{tomus, pdf_path, tex_path,
pdf_sha256, tex_sha256, body_assets}`; `body_assets` maps available relative asset
names to decoded pixel hashes/dimensions. Count occurrences, not
unique xrefs; check decoded pixels where asset files exist and retain ambiguous
identity. Equal image bytes legitimately can appear multiple times. No `zip`
operation may silently truncate unequal reference/occurrence lists.

Keep historical indexing/alignment orchestration in `audit_inner_tomi.py`.
Import its reusable pure helpers under one module alias: `comparison_views`,
`passage_windows`, `anchor_information`, `fit_source_index`, `load_source_index`,
`find_reader_occurrences`, `chain_matches`, and `align_tokens`. Do not copy these
algorithms. The new adapter must not call `cached_source_index`,
`cached_source_regions`, `extract_source_regions`, `unit_source_tokens`, or
`read_approved_exclusions` to create expected Latin. Their production cuts/skips
are incompatible with the independent inventory.

Share `image_views` and `ink_line_height` by moving them into the common module.
Extend `ink_line_height` with optional settings preserving its current defaults:
threshold 120, 8-connectivity, screened component heights/widths/area, at least
eight components, upper-quartile height. The new inventory uses this same screened
estimator at both source thresholds; it does not add a competing median-height
algorithm. At step 5.1 extract the existing SIFT/matching/RANSAC kernel into
`register_pixel_regions`; the historical registration wrapper keeps its original
ROI, 24-inlier threshold and results. New strip callers pass different regions
and settings explicitly. Cache keys distinguish the callers and settings.

Use one source inventory, one correspondence/coverage evaluator, one reader map,
and one evidence renderer for ordinary line findings and seams. Human/model
observations are imports into reports, never parallel matching implementations.

## B. CLI, runtime identity, prerequisites, and output policy

The audit CLI has `check`, `capture`, `inventory`, `run`, `report`, `adjudicate`,
`verify`, and optional `inspect-trace`. Common arguments are `--repo-root`
(default script parent-of-tools), `--work-dir` (default repository-relative
`tmp/pdfs/facsimile_line_audit`), and explicit `--snapshot`, `--run-dir`, `--units`,
`--resume`, `--settings`, `--source-policy`, `--pdftoppm`, `--tesseract`,
`--tessdata-dir`, and `--trace-dir` where applicable. `--units` is a comma-separated
list of exact table IDs; unknown/duplicate IDs are errors. Selection is recorded.
An omitted settings/policy argument reads the captured version during a run,
not the mutable file in `tools/`. A supplied differing version requires a new
capture/run identity; it cannot be applied to an old run with `--resume`.

`check_audit_prerequisites` reuses the existing package/tool inspection pattern.
Resolve tools in CLI, corresponding `PLUTARCH_...` override, then PATH order.
Require Python 3.10+, the pinned core packages, Poppler and Tesseract `eng` for
OCR-backed runs. Source/reader raw inventory can continue without usable OCR,
with `ocr_unavailable` recorded; OCR-dependent alignment cannot claim completion.
Record resolved executables, versions, language data path/digest and OpenCV/NumPy/
PyMuPDF versions. Resolve `eng.traineddata` from the explicit directory or the
installed directory reported by Tesseract; inability to authenticate it blocks
OCR-backed calibration. Pass `--tessdata-dir` through the shared OCR adapter;
add the optional keyword and digest to historical OCR cache metadata without
changing its default segmentation behavior.

Initial OCR profile is `eng`: source 300 DPI/PSM 6, reader 180 DPI/PSM 3, focused
300-DPI retries with PSM 6 and 11. Preserve raw OCR and orthography; existing
`comparison_views` are temporary matching aids. Do not update source Latin from
OCR or a model. Source ink inventory remains authoritative even with no tokens.

`compute_runtime_identity` hashes actual executed files before comparison-cache
access: the three new audit/reviewer modules when used, `inner_audit_common.py`,
`audit_inner_tomi.py`, and any further repository-local imported audit module.
Record settings/policy digests, tool/package/language identities, deterministic
registration seed, selected scope, and snapshot identity. A mismatch with captured
code requires a new snapshot/run. New policy versions can reuse raw renders keyed
solely by source hash/page/render settings, but never old logical maps, matches,
or verdicts. Raw OCR cache includes rendered-byte, model/language, DPI/PSM and
adapter identity. Comparison caches include runtime/policy identities.

`AuditOutputPolicy` has profiles `historical`, `audit`, and `fixture`. Existing
callers default to `historical`, preserving their current roots and `.pdf` ban.
New `audit` callers can write beneath resolved `work/cache` and `work/runs` and
capture into a fresh direct child of `work/snapshots`; captured snapshots are
read-only after `snapshot-manifest.json` is atomically committed. All profiles
protect source PDFs, accepted `build_facsimile`, `src`, unit/repair ledgers,
credentials and historical archives. Propagate the optional policy to
`write_json`, `reusable_entry`, `run_directory`, and `render_page` so cache/lock
writes use the same resolved-path checks. Include the common error/progress/lock
helpers in extraction rather than leaving circular dependencies.

Verification output is a fresh owned directory beneath `work/runs/verification`.
Only `create_verification_inputs`, using a `fixture` policy bound to that exact
root and a builder-generated filename allowlist, can create its source/reader
PDFs. It cannot overwrite an existing PDF, write outside that root, or write a
PDF through ordinary audit/report commands. Common capture copying retains its
separate input-copy permission. The fixture manifest identifies `target_kind:
verification`; full captures identify `target_kind: corpus` and require both
sources, eight readers, and 68 units. New verification cases may have fewer inputs
but use the same schema and pipeline with their explicit requested counts.

Exit codes: 0 requested processing finished (findings/unresolved observations
allowed), 2 invalid inputs/configuration/prerequisites, 3 interrupted or incomplete
requested processing, and 4 failed explicitly requested verification/calibration
cases. `run-manifest.processing_complete` is separate from coverage/review status.
Persist partial progress and errors, then exit 3 if any requested page could not
be processed. `--resume` requires an exact run identity and reuses only completed,
hash-verified entries. An unexplained registration failure is a completed
observation with an unresolved verdict, not a tool-processing failure.

## C. Immutable captures and stable records

`capture --reader-dir build_facsimile --snapshot-dir <new-directory>` copies the
two original PDFs, eight inner PDFs/TeX files, unit table, two repair ledgers,
source policy, settings, and transitively executed audit code. Record role,
relative paths, SHA-256, observed counts, tool identities and decoded body-image
identities. Verify original/copy hashes before and after capture; refuse overwrite.
Copy body PNGs when available, but missing standalone PNGs do not prevent auditing
a complete reader. Reject escaping references, missing required corpus inputs,
or ambiguous reference counts. Image identity ambiguity is retained explicitly.
Source page ranges use table endpoints (v1 final body page 637, v2 708) as reviewed
scope, not an assertion about all pages of the PDFs; report excluded front/back
matter. Shared boundary pages are rendered once but inspected for both owners.

A run writes `source-inventory.json`, `source-index.json`, `logical-lines.json`,
`coverage-ledger.json`, `seams.json`, `reader-map.json`, `errors.json`,
`review-tasks.json`, `run-manifest.json`, `index.html`, and `crops/`. All records
carry schema version 1 and snapshot/runtime IDs. Sort keys/lists with deterministic
ties. Stable evidence excludes timestamps, absolute paths and runtime timings;
operational metadata stays in the manifest. Hash crop bytes and saved record sets.
Snapshot IDs hash canonical manifest content excluding capture timestamps and
self-digest. Human/model review cannot mutate machine evidence.

## D. Independent source column and line inventory

`SourceOccurrence` contains occurrence ID, source digest/index/physical page,
DPI/raster size, full-source box, crop hash, classification, possible owners,
segmentation status, parent/child IDs, and raw OCR observations when present.
Classification is `body`, `heading`, `annotation`, `furniture`, or `uncertain`.
IDs hash source/page/box/detector identity and do not change with adjudication.
Record every component or uncertainty region. A merged region is not one line
unless separate baseline evidence supports that interpretation.

`inventory_source_page` reads an unmasked full page at 300 DPI, retaining a
full-page context image. Initial inspection ROI is `[0.40,0.00,1.00,1.00]`; record
components touching its left edge and permit a source-only reviewed widening.
No fixed top/bottom trim or production retained-band input is permitted.
Perform column separation before row grouping:

1. At threshold 170 form 8-connected components, retaining all of them. Identify
   likely rules by height greater than eight estimated glyph heights and width
   below 0.3 glyph heights. Use them as gutter hypotheses, never erase their ink.
   A rule touching glyph-like components remains an unresolved object.
2. Smooth the full-height per-column ink profile over nine pixels. Find low-ink
   runs in normalized x `[0.40,0.62]` at most 15% of the profile's local 90th
   percentile, at least 0.75 glyph heights wide. Rank first by separation of two
   text-dense sides, then by width, then smaller x. A valid candidate has repeated
   text-like components on both sides and agrees within one glyph height with the
   threshold-210 estimate. Estimate the Latin body left edge from the repeated
   leftmost text-component starts to the right of the candidate, rather than
   assuming all gutter/reference components belong to the prose column.
3. If no distinct gutter candidate exists, retain the broad ROI as uncertain.
   A boundary-crossing component is never divided or dropped automatically.
   Components confidently left of a measured gutter are separate neighbor-column
   observations; glyph-like material near the proposed Latin left edge remains
   ambiguous until source-only ownership/classification review.

Use shared screened `ink_line_height` for glyph scale at both thresholds. If its
inputs are insufficient, retain grouping uncertainty. On the measured Latin side,
form provisional baselines from glyph-sized component centers, grouping only
within 0.5 glyph heights vertically and 2.5 glyph heights horizontally. Lines may
have separate word clusters; join them only when their fitted baselines agree
within 0.25 glyph heights and no second baseline intervenes. Fit each baseline
against its original member components, not a transitive graph that can bridge
several printed lines. Wider word components are allowed with consistent height.
Attach punctuation/diacritics to a unique nearest baseline within 0.75 glyph
heights and horizontal distance 1.5 glyph heights; multiple possible parents stay
uncertain. Tall/rule-connected components with multiple baselines become an
unresolved group with measured child candidates. Retain verse indentation.

Thresholds 170/210 are separate observations; disagreements are uncertain regions.
Create a residual-ink mask: every pixel selected at either threshold in the
inspection ROI must belong to a line, furniture/annotation hypothesis,
neighbor-column observation, or uncertainty object. Unassigned components become
uncertainty objects rather than disappear. Preserve raw grayscale overlays so
fainter ink outside these masks can be noticed during source review. A source
page cannot be marked fully inventoried if its gutter or residual assignments
remain unresolved. This is an observed-ink accounting check, not a claim to detect
all arbitrarily faint source glyphs.

`SourcePolicy` is source-digest bound and versioned. It contains reviewed ownership,
furniture classifications, measured gutter/child-box overrides and duplicate maps.
Source OCR and production title/body fractions may suggest locators; they cannot
exclude expected prose. Imported decisions bind crop hashes and source regions.
Keep original source labels/annotations as separate coverage categories. Missing
OCR never deletes an occurrence. Source-partial scans remain distinguishable from
additional reader loss.

## E. Duplicate equivalence and canonical logical coordinates

`DuplicateGroup` stores logical owner, reviewed physical passage regions,
canonical source occurrence, logical sequence, region maps and source-only evidence.
Seed existing approved numeric pairs/regions; a skip list, page arithmetic, or
similar words does not establish equivalence. Split mixed/shared pages by actual
owner before equivalence, especially Eumenes/comparison v2/101–105. Keep incomplete
second openings in Alexander, Phocion and Otho explicit. New equivalence requires
source-only review; no new production skip decision is installed.

`LogicalLine` stores ID, owner/sequence position, canonical required box, required
ink components, physical occurrence maps, classification and source/segmentation
uncertainty. Canonical coordinates are pixels relative to the reviewed complete
reference line crop at source DPI; its width/height and grayscale hash are fixed.
Each `OccurrenceRegionMap` specifies original full-source fragment box, 2x3 affine
transform to canonical coordinates, reviewed covered canonical polygon, evidence
hash, and maximum registration residual. Different wraps use multiple maps for
one physical occurrence. Do not assume identical line ordinals or pixel dimensions.
If no complete reference exists, explicitly reviewed complementary source regions
may define separate canonical parts; boundaries with no trustworthy common
reference stay unresolved. Illegible/source-absent glyphs cannot be invented.

Only actual visible reader pieces mapped into this canonical frame contribute to
coverage. Their union must cover every required reference ink component and lie
on one monotonic reader path for that logical position. Order pieces by canonical
part/left-to-right extent and reader page/y/x. Reject gaps, conflicting overlap,
backwards placement, unresolved transforms or arbitrary repeated wording elsewhere.
An overlap may corroborate retention but does not double-count coverage. One full
authorized occurrence is enough; a redundant incomplete occurrence can be waived
only after full logical coverage is established. Extra full copies are repeat
observations. Keep physical, logical, redundant, source-partial, annotation and
unresolved denominators separate.

## F. Inventory-to-token adapters and ordered neighborhoods

`source_inventory_to_tokens(inventory, logical_lines, ocr_observations)` returns
`SourceTokenBundle`: per-physical-occurrence tokens plus explicit tokenless objects,
logical maps, canonical token sequences, and source-only passage windows. Each
token preserves raw text/provider, occurrence ID, logical-line ID when resolved,
source page/box, physical line ID, logical position, `strict`/`tolerant` temporary
views, and any observed hyphen-join hypothesis. Bind fresh source raster OCR words
to inventory objects by box overlap; ambiguous words stay unassigned observations.
Production cuts/skips are not consulted. A tokenless source object remains in the
expected line ledger and image-comparison queue with `ocr_missing: true`.

For a reviewed equivalent group, choose canonical text from the reference/source
maps. Map other occurrences to canonical token intervals only when bounded local
alignment supports the reviewed region relation; unequal or uncertain token
mappings stay geometric maps, not forced equal-length aliases. Keep all physical
OCR. `source_bundle_to_index_unit` produces the existing index shape
`{unit_id, source_index, tokens, strict, tolerant, anchors}` plus a reverse
canonical-token-to-occurrence lookup. Use `comparison_views`, `passage_windows`,
`anchor_information`, and `fit_source_index` to build a source-only vocabulary/IDF.
If there are no valid source windows, skip fitting and record `no_source_anchors`;
do not call TF-IDF with an empty vocabulary or substitute invented tokens.

`reader_observations_to_tokens(reader_map, ocr_observations)` preserves raw views,
physical reader image/page, body box, pixel/normalized word boxes, PDF folio
provenance, observed line identity and hyphen joins. Run
`find_reader_occurrences` then `chain_matches` against canonical source sequences.
Pass empty old-style aliases because the new source sequence is already canonical;
explicit region maps drive physical duplicate coverage. Reuse `align_tokens` in
local mode for token refinement. Existing index/retrieval/window defaults seed
settings, but old omission thresholds never determine new line completeness.
Do not refit the index on reader text.

A candidate neighborhood is bracketed by ordered anchors at the expected logical
position. Use up to three nearest matched lines on each side and at most two
adjacent reader body pages per local comparison; larger gaps become overlapping
local tasks with explicit uncertainty. Interior source-page edges are not unit
endpoints. Tokenless lines use neighboring registered image anchors; a unit with
no source/reader anchors stays unresolved until image/manual localization provides
a bounded reader extent. Do not search arbitrary repeated prose to clear a loss.

## G. Shared local registration and whole-ink decision rules

`register_pixel_regions(source_gray, reader_gray, source_box_px, reader_box_px,
settings, initial_scale)` returns original-array-coordinate affine transform,
inlier pairs/extent, reprojection residual, measured scale/rotation, alternate
placements, and validity reasons. Extract the existing SIFT/BF/RANSAC kernel once;
historical `match_source_image` retains its current source ROI/settings through
this primitive. New callers include the source head/foot region and local anchor
strips explicitly. Cache identity includes actual region boxes, caller profile,
pixel hashes and thresholds.

Build local strips from consecutive matched lines with consistent source/reader
spacing. A source page may need many independent strips. Do not extrapolate one
transform across a changed gap, source page, reader page, or scale fit. Estimate
initial scale as reader/source glyph height; use matched word-box heights as a
second observation. If these disagree by more than 10%, retain ambiguity. Search
scale within +/-10% of the estimate, rotation +/-3 degrees, and translation within
two glyph heights of the projected anchor box. Require at least eight inliers,
60% of ratio-filtered matches, reprojection residual at most 0.2 reader glyph
heights, two distinct glyph groups spanning at least half the comparison width,
and a runner-up placement margin of at least 0.05 normalized correlation.
Use SIFT `nfeatures=4000`, BFMatcher L2, ratio cutoff 0.70, and RANSAC
`maxIters=2000` with the glyph-relative reprojection bound above. Seed RANSAC
at 0; break score ties by reader page/y/x then source position.
These initial bounds are calibratable and saved; invalid fits stay unresolved.

For a short line lacking features, register its adjacent retained anchor lines
first, then restrict translation search to its bounded gap. Nominate image matches
using normalized correlation on source grayscale and `image_views` contrast,
but never declare complete from correlation alone. No valid inherited/local
transform means unresolved. For omitted lines, the expected gap/search region can
still produce an omission candidate even though the missing line cannot itself
supply registration features. Record this distinct evidence type.

`measure_required_ink` evaluates expected reference ink after the measured
transform against actual **visible** reader ink at both 170 and 210 thresholds.
Warp grayscale once with linear interpolation; retain original reference
components, expected transformed masks and a one-reader-pixel tolerance halo.
For each required component compute expected transformed area, unsupported ink
inside that halo, largest unsupported connected component, bounding box and
whether it touches an end/ascender/descender. Components whose projected area is
below four reader pixels or whose support changes materially between threshold/
resampling observations are `resolution_uncertain`, never silently ignored.
Compare to the decoded body image as supplementary evidence; visible PDF pixels
must pass independently so clipping or overpainting is not cleared by embedding.

Machine `complete` requires: reviewed body ownership/required geometry; a unique
ordered neighborhood; valid per-piece transforms; every required component covered
with unsupported area at most max(1 pixel, 2% of that component's area); no
unsupported component of at least four pixels; no unsupported component touching
a line end or upper/lower glyph extremity; no resolution uncertainty; and the
canonical union rule for any split/duplicate pieces. Both threshold observations
must agree. Small tolerated differences are recorded. These are conservative
initial observed-ink predicates, not probability or character-perfect proof;
source grayscale uncertainty and calibration limits remain in the report.

Machine `partial` is a candidate when retained context/part of the line is located
but a supported endpoint/glyph-region deficit of at least four projected pixels
persists across both threshold observations. `absent` is a candidate when the
required bounded reader extent is inspected, contextual placement is valid, no
complete source-line match survives focused OCR/image retries, and none of the
required line regions has established visible retention. Otherwise use
`unresolved` with reasons. An aggregate high match can never override a failing
required-component check. `evidence_status` is `observed` or `candidate`;
`human_verdict` is a separate field. Source damage uses `source_partial` and never
waives additional loss. A sliced line with complete ordered union may be complete
with a separate `sliced_line` warning when the break crosses required ink.

Endpoint-specific evidence replaces impossible two-sided anchors. Interior gaps
need strong matching contexts on both sides. A unit opening uses its independently
reviewed source start, first mapped reader body extent, and a strong following
body anchor; a unit ending uses its reviewed source end, final reader body extent,
and a strong preceding anchor. A strong anchor initially means eight informative
ordered OCR tokens or a valid image strip covering two distinct retained lines.
Mark one-sided candidates with `context_kind: opening` or `ending` and reduced
evidence strength. Do not borrow prose from another unit. A wholly missing sparse
passage with no body anchor needs independently established unit ownership and
reader boundary mapping; it remains unresolved unless a reviewer locates the
complete bounded extent and confirms the loss. Source-page head/foot losses
inside a unit still use the interior rule.

Existing traces can attach a cause only when source/build identities and decoded
reader pixels authenticate their declared scope. Store `loss_stage: unknown`
otherwise, with hypotheses separate. Traces cannot change matching verdicts.
No production rebuild or new tracer is required.

## H. Seams, review interchange, and Haiku delivery

Build all source-page, unit/shared-page, duplicate-bypass and adjacent-reader-body
seams from the ledger, including apparently complete seams. Include three owned
body lines on either side, uncertain regions between anchors, and relevant
headings/annotations. Extend sparse context within the unit. Distinguish endpoint
seams from interior page edges. Physical duplicate edges remain visible; waive a
redundant copy only after logical complete coverage. Deduplicate tasks without
losing seam reasons. One renderer creates untouched hashed crops plus separately
annotated display copies.

Human source reviews bind snapshot/inventory IDs, occurrence ID, source crop hash,
classification, owner/uncertainty, reason and measured overrides. Duplicate reviews
add canonical-region transforms, polygons, source-only evidence and reviewer.
Loss reviews bind run/ledger and source/reader hashes with `confirmed_loss`,
`retained` or `uncertain`. Reject stale identities. Write a new policy version for
source/duplicate decisions; never edit a captured run. Source and duplicate modes
of `adjudicate` are implemented when their respective steps need them; human loss
mode comes with report generation.

Haiku receives neutral labeled source and visible-reader images, immutable line
IDs, and no machine verdict/repair label. Ask for complete/partial/absent/unclear,
object type, supporting reader crop IDs and reason. Show both reader-page sides
and relevant equivalent scans. Limit six body lines per overlapping task. A task
without localized reader extent is inconclusive, not a request to infer Latin.
Validate JSON locally, preserve raw responses, and keep model observations
separate from the machine ledger and human adjudication.

The Python SDK client/key are loaded lazily only for live review, from existing
repository-root `secrets.json`. SDK defaults are `max_retries=0`, timeout 120
seconds, `max_tokens=2000`, model `claude-haiku-4-5`. Live calls require positive
`--max-calls`; cap counts uncached task submissions. Cache by model/prompt/schema/
task/crop identity. `--resume` reuses valid completed observations and may submit
pending tasks within the explicit cap; it never resubmits unknown delivery.
Persist `in_flight` atomically before submission. Interrupted/ambiguous delivery
becomes `delivery_unknown`. An explicit retry is
`--task-ids <comma-separated-ids> --retry-unknown --attempt-id <new-id>`; archive
the old attempt, require all three flags together, and reject reused attempt IDs.
Dry-run/offline submit nothing and inspect pending/cached/unknown states without
loading the key. Record requested/returned model and token usage. Never snapshot,
hash, print, or serialize credentials.

## I. Settings and calibration criteria

Settings JSON contains the ROI, component/gutter/grouping values in Design D,
OCR profiles in Design B, source-index windows/retrieval/chain defaults imported
from `detector_settings`, and registration/ink predicates in Design G. Store all
numeric values explicitly, including threshold pair `[170,210]`, connectivity 8,
three-line seam context, six-line task limit, deterministic seed 0, and task image
longest side 1,400 pixels. Record calibration changes under new identities.

Before changing tolerances, inspect saved transform residuals, missing-ink masks,
threshold disagreements and endpoint deficits. Calibration acceptance requires
all seven named end-to-end cases produce expected localized candidate/retention
results, including the small-glyph clipping subcase, with no redundant-copy loss.
Unresolved known-loss cases are misses, not successes. Real source controls must
not show confirmed loss on accepted restored lines. Report fixed-panel false
alarms, misses and uncertainties separately with denominators. Threshold revisions
may reduce nuisance differences but cannot remove endpoint predicates, silently
ignore small required glyphs, or waive unresolved logical maps to achieve a green
report. Preserve earlier predictions when revising settings.

# Implementation sequence

Retain the existing phase/step identifiers. Phase 2 is optional diagnostic work and
may be deferred until a finding needs trace provenance; skip it without blocking
Phases 3–7. Each required step has executable verification available at that step.
The `verify` command develops incrementally: fixture preparation in 5.1a, coverage
checks in 5.2, and complete coverage/seam calibration in 7.1. No new unit-test suite
or mock framework is added. Existing repository tests remain intact.

## Phase 1 — Shared Python infrastructure and immutable inputs — Suggested implementer: Sonnet

### 1.1 Extract shared infrastructure with historical adapters intact

Create `inner_audit_common.py` and move exactly the infrastructure definitions
listed in Design A, including errors, progress, context managers, locks, path
policy, cache reuse and the body-asset regex. Import/re-export them from the old
script; avoid imports back from the common module. Add `AuditOutputPolicy` and
optional policy propagation from Design B while preserving historical defaults.
Extract `map_reader_occurrences(reader_inputs, expected_counts=None)`, keeping
historical count/folio/body-ordinal checks in the old adapter. Extract/share
`image_views` and parameterizable `ink_line_height` with unchanged defaults.
Do not move the old index/omission orchestrators or write a second reader mapper.

**LLM verification:** Run `python -m py_compile tools/inner_audit_common.py
 tools/audit_inner_tomi.py` and `python tools/audit_inner_tomi.py check` when its
prerequisites exist. From a small Python invocation, call the shared reader mapper
for one current tomus and inspect reference/occurrence counts, kinds and folios.
Compare the historical adapter's semantic output with an existing saved reader map
in a new run directory when available; never regenerate it inside the archive.
Inspect ordinary `.pdf` write refusal and fixture-policy refusal outside its bound
root through a disposable script invocation. Record unavailable optional evidence.

**Human verification:** Inspect mapped body/separator/padding and repeated-image
occurrences. Read the new path-profile rules, confirming historical defaults
still reject writes to production inputs.

### 1.2 Implement capture, runtime configuration and prerequisite checks

Create the core CLI/module, settings and source-policy files. Implement
`capture_snapshot`, `load_snapshot`, `compute_runtime_identity`,
`check_audit_prerequisites`, and `validate_source_policy`. Expose `check` and
`capture`; parse later commands with clear unavailable-stage errors until their
steps exist. Apply Design B/C runtime/tool/language/code identity and path rules.
Capture actual counts and decoded assets without requiring standalone PNGs.
Use corpus manifests with two sources, eight readers and 68 units; do not reuse
historical page counts. Verify copies atomically and refuse completed overwrite.
Seed known duplicate records without claiming complete region equivalence.

**LLM verification:** Run `python tools/facsimile_line_audit.py check`, `--help`,
and Python compilation of the three existing audit/common files. Capture current
readers to a fresh snapshot. Inspect copied paths, hashes, runtime identity and
observed counts; retry the same destination and require exit 2. In a disposable
input directory, omit a required PDF and require a clear failure. Change a copied
settings/code byte and ensure `load_snapshot` or comparison preflight refuses
using the old identity. Confirm no captured path is a secret.

**Human verification:** Confirm the snapshot targets the accepted current readers,
source scans and reviewed body endpoints. Inspect unresolved duplicate seeds and
OCR prerequisites; absence of OCR must never label a page clean.

## Phase 2 — Optional existing-provenance diagnostics — Suggested implementer: Sonnet

### 2.1 Add a scope-aware trace inspection command when needed

Implement `load_optional_traces` and `inspect-trace --snapshot <path> --trace-dir
<path> --out <new-run-directory>`. This command reads existing trace JSON and
emits diagnostic metadata without needing inventory, coverage or seams. Keep
unit/source/physical-page identity, retained indices, raw boxes, declared scope,
input digests and uncertain historical/current build relationships. Never derive
retained index as page minus start. Do not generate new expected inventory through
ignored Rust tests or modify the generator. If this optional phase is deferred,
normal audit runs report `trace_diagnostics: not_requested` and need no adapter.

**LLM verification:** Run `inspect-trace` against one available trace, or a minimal
disposable schema-shaped record, into an approved run directory. Inspect source
identity/scope and a stale-input diagnostic. Run the core `check` command without
any trace directory, confirming no trace prerequisite. No future inventory/coverage
command is required for this step.

**Human verification:** Confirm historical trace boxes are labeled with their actual
scope and cannot appear as asserted current-reader locators.

### 2.2 Add authenticated causal attachment without altering coverage

Implement `attach_trace_diagnostics` and
`inspect-trace ... --bind-reader-assets`, reusing the reader mapper from Phase 1.
Bind declared trace canvas/pixel identities to captured reader assets where supplied.
Only authenticated executed operations can populate `loss_stage`; other records
retain `unknown` with separate hypotheses. Missing pixel/build evidence is a normal
diagnostic limitation. Integration into coverage is a callback used in Phase 5,
not a second matcher. Do not port the cropper/packer/masks into Python.

**LLM verification:** Inspect a bound trace if available and a disposable stale
trace. Require unknown cause on the latter and inspect the returned diagnostic
record locally. Compile the module. The with/without-coverage comparison is a
Phase 7 check after coverage and seams exist.

**Human verification:** Read a candidate cause with its build/pixel evidence and
an unknown cause without that evidence. Confirm neither forces a rebuild.

## Phase 3 — Independent source-line inventory — Suggested implementer: Opus

### 3.1 Detect source columns, lines and residual uncertain ink

Implement `inventory_source_page`, `inventory_source_units`,
`separate_source_columns`, `assemble_source_lines`, and `account_residual_ink`.
Expose `inventory --snapshot <path> --run-dir <new-path> [--units ...]`.
Render every inclusive physical source page, including duplicate skips; deduplicate
shared-page renders. Execute source-only gutter separation before baseline grouping
exactly as Design D. Reuse the shared screened glyph-scale helper. Retain original
components, rule-connected/neighbor-column objects, threshold disagreements, page
edges, short verse and residual ink. Source OCR adds observations but cannot remove
objects. Do not call production crop/band routines or historical expected-region
wrappers. Save untouched source crops and full-page component/line/gutter overlays.

**LLM verification:** Compile changed Python files. Inventory Theseus, Pericles,
Cato Minor and Antonius in a fresh run. Inspect raw component accounting, gutter
confidence, parent/child assignments, residual mask and head/foot uncertainty counts.
Change a disposable copy of production trim metadata while preserving source and
inventory settings; raw inventory IDs/boxes and expected raw objects must agree.
The mutation must not alter or overwrite the accepted unit table.

**Human verification:** Inspect full-source overlays, particularly Pericles verse,
Antonius v2/562, Greek/gutter fragments and touching divider/glyph objects. Every
visible Latin line must have a line or explicit uncertainty object; unresolved
gutters cannot be presented as completely inventoried pages.

### 3.2 Resolve source-only ownership and implement source adjudication

Implement `apply_source_policy`, `validate_source_inventory`, source-only review
sheets, and the **source mode** of `adjudicate` now. Command:
`adjudicate --inventory-dir <path> --source-reviews <file> --policy-out <new-file>`.
The destination must be a fresh file under an approved run directory. Validate
snapshot/raw-inventory/crop identities, review classifications, unique logical
owners, gutter/child-box overrides and residual ink. Keep unresolved owners and
source-partial regions. Production fractions remain candidate locators, never
exclusion authority. Imported policy records retain decision provenance; a new
snapshot can adopt the new source-bound policy without mutating the old run.

**LLM verification:** Import a small reviewed boundary file with the new source-mode
command. Inspect resulting policy decisions, one shared page, indented verse and a
sparse ending. Mutate a review crop hash in a disposable copy and require exit 2.
Confirm source/neighbor/furniture/annotation/uncertain assignments account for all
observed ROI ink and prior snapshots remain unchanged.

**Human verification:** Review source-only boundaries before reader outcomes,
including source starts/ends, labels and uncertain gutter objects. Do not exclude
an object merely to simplify matching.

## Phase 4 — Logical passages and reviewed duplicates — Suggested implementer: Opus

### 4.1 Resolve passage equivalence and implement duplicate adjudication

Implement `seed_duplicate_groups`, `resolve_duplicate_groups`, source-only paired
review sheets, and the **duplicate mode** of `adjudicate` now. Command:
`adjudicate --inventory-dir <path> --duplicate-reviews <file> --policy-out
<new-file>`. Seed existing numeric evidence, including records without universal
`occurrences` fields. Validate source-bound canonical-reference IDs, original
fragment boxes, affine maps, covered polygons, evidence hashes and reviewed residuals
from Design E. Do not infer equivalence from skip lists or forced equal token
counts. Split Eumenes/comparison shared-page material by its true owner; retain
partial Alexander/Phocion/Otho copies and unique continuations. No production
skip decision is installed.

**LLM verification:** Export all 12 known duplicate seed records and inspect numeric
provenance. Import a small measured group, then inspect its canonical-region maps.
Reject disposable mappings with escaping boxes, uncovered required spans,
conflicting owners or partial coverage claimed complete. Source-only region maps
may remain unresolved rather than be guessed from reader behavior.

**Human verification:** View each relevant original scan pair and the canonical
reference overlay. Confirm a complete passage exists in its source group and each
partial occurrence maps solely to its actually visible span.

### 4.2 Build ordered logical regions and stable denominators

Implement `build_logical_lines`, `validate_logical_sequence`, and canonical-region
coverage metadata. Preserve physical occurrences while assigning logical positions,
reference ink components and ordered part mappings. Reject cycles/contradictions,
unreviewed owners and incomplete required spans as complete logical maps. Keep
physical/logical/redundant/source-partial/annotation/uncertain counts separate.
Build source index adapter contracts so text and geometry map to the same logical
IDs; actual index fitting is introduced in step 5.1. Tokenless lines remain required.

**LLM verification:** Export and inspect ordered logical sequences through a shared
boundary and a duplicate block. Validate fragment maps in canonical pixels and
required-component unions independent of detector-generated verdicts. A disposable
partial-only group must leave the missing complete portion required/unresolved.
Compile the modules and check stable IDs survive a review-verdict-only change.

**Human verification:** Read source/logical sequence overlays, checking unique
continuations, full reference geometry and legitimate copy waivers. Confirm source
scan damage remains distinct from later reader loss.

## Phase 5 — Accepted-reader localization and visible-ink coverage — Suggested implementer: Opus

### 5.1 Adapt independent observations and share local registration

Implement `source_inventory_to_tokens`, `source_bundle_to_index_unit`,
`reader_observations_to_tokens`, `locate_reader_neighborhoods`, and
`build_line_correspondences`. Build the source-only index using the exact pure
helpers listed in Design A/F, never the historical cut/skip orchestrators.
Preserve physical/region reverse lookup, canonical logical sequences, ambiguous
OCR assignments and tokenless expected objects. Fit no empty vocabulary. Pass
empty old-style aliases to `chain_matches` on already canonical source sequences;
geometric duplicate maps remain separate coverage authority.

Extract `register_pixel_regions` from the existing registration kernel and adapt
historical `match_source_image` with identical historical arguments/settings.
Implement piecewise `register_local_strips` using explicit anchor regions and
bounds from Design G. A registration strip cannot span changed source/reader gaps
or scale fits. Keep short-line inherited-transform and tokenless-anchor fallbacks
conservative. Share one kernel, not the production ROI wrapper or a duplicate
matching implementation.

Expose `run --snapshot <path> --run-dir <new-path> --stop-after locate` for this
step. It writes inventory/logical/index/reader/correspondence artifacts and marks
that declared stage complete; it does not claim coverage/seam processing exists.
Default full `run` remains unavailable until Phase 6. Include the new stop stage
in run identity. Preserve historical source/index/window definitions for old runs.

**LLM verification:** Compile changed Python files, inspect source-adapter/index
payloads and OCR-free objects, then run scoped localization for the four inventory
units and one duplicate unit. Compare a disposable production-trim mutation:
independent raw expected objects and tokenless queue must agree. Inspect measured
scales, anchor strips, invalid fits and alternative placements at source heads/
feet. Compare the historical registration wrapper on saved unchanged examples
in a new diagnostic run if evidence is available.

**Human verification:** View source/reader anchor overlays near restored lines,
short verse, gaps and duplicate joins. Confirm full-height source boundaries were
not hidden and image transforms apply to the actual local pieces.

### 5.1a Prepare seven small verification cases before coverage implementation — Suggested implementer: Sonnet

Implement `create_verification_inputs` and `verify --prepare --out <fresh-root>`
using the narrow fixture policy in Design B. This suffixed step preserves existing
identifiers. Create source and reader PDFs/TeX/PNG assets and same-schema fixture
snapshots for the seven named cases listed in 7.1. Use legible text rendered with
Pillow's packaged scalable default font at 300-DPI-equivalent size, with at least
three lines of twelve distinct words around interior loss regions. Synthetic
sentences are fixture content, never transcription or replacement source Latin.
Embed source text as pixels and provide no fabricated expected OCR observations.
The normal Tesseract/index/image path must process these fixtures.

Use full-width source pages with separate readable left/right text columns and a
clear measured gutter. The Latin-like right-column fixture preserves line boxes,
punctuation, short verse and sparse passage objects. Build reader images from the
same known source pixels with independently recorded crop/placement boxes. Store
expected source/reader geometry in `case-expectations.json` written by the fixture
builder; never derive it from the audit's detected lines. Fixture source/duplicate
policy decisions carry `provenance: synthetic_ground_truth` and may substitute
for human source decisions solely when `target_kind: verification`; reject that
provenance in corpus policies. Thus the cases do not require a human to approve
builder-known ownership, and they cannot confer production repair eligibility. Each ordinary case has
enough distinct retained text for the configured anchor requirements. Keep a
source-partial duplicate map explicit. The retained control includes a one-line
sparse passage whose correct outcome may require documented manual localization;
that subcase must not masquerade as an automatically detected omission.

The visible-clipping case includes an embedded complete image with a visibly
clipped/overpainted whole-line region **and** a small terminal-glyph fragment
removed while the rest of its line remains. The latter deficit must retain at
least eight unsupported projected pixels after the one-pixel halo, so it tests
the declared detectable small-component predicate rather than an invisible
subpixel change. Head/foot loss includes actual unit
opening/ending variants plus source-page seams inside a unit. Keep seven top-level
case IDs, with these variants as subcases. There is no separate test framework.

**LLM verification:** Run `verify --prepare` and inspect permitted created paths,
PDF/reference counts, fixture manifests, independently specified boxes, and
source OCR word counts. Require sufficient real OCR/image anchors rather than
all-unresolved nontext patterns. Retry the same fixture root and require refusal;
try a disposable output outside `work/runs/verification` and require exit 2.
Preparation does not require the future coverage implementation to pass.

**Human verification:** View all source/reader fixture pairs, confirm text legibility,
known edge/interior/duplicate losses, and the small clipped glyph. Verify the
case expectations describe the deliberately created geometry.

### 5.2 Implement component coverage and endpoint-specific candidates

Implement `measure_required_ink`, `evaluate_occurrence_coverage`,
`evaluate_logical_coverage`, and `verify_reader_visibility` following Design E/G.
Compare required canonical ink components against actual visible reader regions,
retaining threshold/resampling uncertainty and deficits at endpoints/upper/lower
extents. Use embedded image bytes only as supplementary evidence. Required
per-component deficits override aggregate correlation. Apply explicit interior,
opening, ending and sparse-passage rules; do not require unavailable two-sided
body anchors at unit endpoints. Separate evidence status from human verdict.

For split/duplicate pieces require reviewed transforms, monotonic order and full
required-component union; overlap does not double-count. A redundant partial scan
is waived only after complete logical coverage. Unresolved maps/fits/ownership
stay unresolved. If optional Phase 2 exists, attach scope-authenticated diagnoses
without changing these verdicts; otherwise leave cause unknown.

Expose `run ... --stop-after coverage` and
`verify --out <prepared-root> --stop-after coverage`. In the prepared fixture root,
write new result directories without overwriting builder inputs. At this point
verify the coverage/localization part of all seven cases; seam-specific checks
are added in Phase 6/7, and records state the actual completed stage.

**LLM verification:** Run the prepared seven cases through normal source/reader
OCR, localization and coverage; inspect localized deficits, endpoint variants,
small-glyph clipping, accepted complete/redundant controls and sparse uncertainty.
Known detectable omissions unresolved at this stage are failures to investigate,
not successful detections. Run one scoped current-reader coverage command and
inspect component masks/differences. Confirm a high overall match cannot clear the
small-glyph deficit. Compile changed Python files.

**Human verification:** Inspect source/visible-reader pairs for accepted head/foot
restorations, Pericles verse, Antonius and proposed losses. Read endpoint evidence
and required-component masks rather than inferred Latin. Confirm no production
output or Latin changed.

## Phase 6 — Systematic seams and Python Haiku reviewer — Suggested implementer: Sonnet

### 6.1 Complete seam/report and human-loss adjudication modes

Implement `build_seam_inventory`, `build_review_tasks`, `write_coverage_report`,
and `report`. Enable default full `run` now. Add **human-loss mode** of `adjudicate`:
`adjudicate --run-dir <path> --loss-reviews <file> --reviews-out <new-file>`.
Source/duplicate modes from earlier steps remain unchanged. Source policy changes
always start a new snapshot/run; loss adjudications produce separately versioned
reviews for the same immutable machine ledger. Check exact source/reader crop and
record hashes, and reject stale decisions.

Build every source/unit/shared/duplicate-bypass/reader-body seam with Design H
context. Use one evidence renderer and the same ledger/matching verdicts. Annotate
copies while preserving untouched crop hashes. Show complete and unresolved seams
and all uncertainty denominators. Compare the optional Phase 2 callback's results
with and without diagnostic attachment only once full coverage/seams exist.

**LLM verification:** Run the prepared cases through full coverage/seams and inspect
requested/completed seam counts, source/reader endpoint context and duplicate
joins. Compare totals against explicit source/logical transitions and adjacent
reader body occurrences. Inspect HTML escaping/evidence links. Import one bound
human-loss review and reject a mutated crop-hash copy. If optional trace adapters
exist, confirm they do not change line coverage/seam counts.

**Human verification:** Browse clean and uncertain seams, shared boundaries and
skip joins. Check reader endpoints are labeled as one-sided evidence and sparse
context never borrows another unit's prose.

### 6.2 Implement explicit Python Haiku submissions and resume semantics

Implement `review_seam_tasks` using the official Anthropic SDK and existing
repository-root `secrets.json`. CLI includes `--tasks`, `--out`, `--max-calls`,
`--model`, `--offline`, `--resume`, `--dry-run`, `--task-ids`, `--retry-unknown`,
and `--attempt-id`, with exact behavior from Design H. Core audit imports never
load the SDK/key. Pin a working installed SDK version separately and record it.
Use neutral versioned PNG/image prompts with six line IDs, preserve transforms,
and validate complete response IDs/verdicts locally. Focused image tiles have at
most a 1,400-pixel longest side and enough overlapping context for glyph legibility.

Persist attempts, raw responses, usage/errors and parsed observations atomically.
Default live transport has no SDK retries and a 120-second timeout. A failed/
interrupted ambiguous request cannot be silently resubmitted; the explicit unknown
retry triple creates a distinct attempt and preserves old evidence. Dry-run/offline
inspect cached/pending/unknown tasks without accessing credentials. Neither model
results nor SDK errors modify source policy, unit table or machine coverage.

Official implementation references:
[Anthropic Python SDK](https://github.com/anthropics/anthropic-sdk-python) and
[Claude vision documentation](https://platform.claude.com/docs/en/build-with-claude/vision).
Verify SDK/model support when implementing. No price assumption or paid pilot is
required to complete offline delivery.

**LLM verification:** Compile the reviewer and dry-run real tasks, inspecting neutral
prompt, line IDs, crop dimensions and task cap. Use an explicitly saved example
JSON response in an offline replay command to inspect valid/missing/unknown line
ID rejection; add `--replay-response <file> --task-ids <single-id> --offline` for
this local validation path, which labels its result `replay` rather than a live
observation. Inspect cached/unknown resume counts and explicit retry argument
validation with disposable metadata. No unit-test mocks are added. A small live
pilot is optional when explicitly invoked with the cap and existing key.

**Human verification:** Read a prompt/source-reader pair and replay/cached result,
checking model observations remain supplemental. Compare any live pilot result
with a human image read and retain disagreements.

## Phase 7 — Calibration, full-corpus execution and durable delivery — Suggested implementer: Opus

### 7.1 Run the seven complete cases and calibrate measured evidence

Complete `run_verification_cases` behind `verify --out <prepared-root>` using the
already prepared assets and the normal inventory/index/correspondence/coverage/
seam pipeline. Require these seven case IDs with independently saved geometry:

- `retained_control`: complete lines and short verse survive in order; sparse
  one-line subcase is explicitly complete after bounded localization or remains
  reported uncertainty, never an invented anchor or false loss.
- `head_and_foot_loss`: flag actual unit-first/unit-last lines using one-sided
  evidence, and source-page edge losses inside a unit using interior evidence.
- `interior_block_loss`: localize a distinctive missing multi-line passage.
- `redundant_copy_removed`: one complete authorized copy survives; no unique loss.
- `sole_complete_copy_removed`: a clipped remaining copy does not waive the
  required missing canonical regions.
- `shared_page_unique_tail`: duplicate prose survives but its unique omitted
  continuation remains a loss candidate.
- `visible_pdf_clipping`: detect the hidden complete embedded-image line and the
  small terminal-glyph deficit despite high whole-line similarity.

Each known loss must be a correctly localized candidate/partial finding at the
required source/reader boxes; unresolved known losses are misses. Use source box
intersection/required-line membership plus expected reader seam from builder
geometry; a whole-page generic warning is insufficient. Retained/redundant
controls must have no unique-loss candidate. Save case report, masks, transforms
and images, including processing failures. `verify` exits 4 on failed cases. It
can additionally read the known sparse manual-localization decision as separately
labeled review evidence; that subcase cannot improve automatic recall counts.
No `unittest`, `pytest`, new Rust tests or extensive per-function assertions.

Calibrate source grouping and Design G predicates on source overlays and real
restored controls. Before scoring, fix a 25-body-page-per-tomus panel by sorting
SHA-256(reader-input identity, tomus, page). Reader-input identity hashes the
source PDFs, reader PDFs/TeX and unit table, excluding evolving code/settings/
source-policy versions. Save the selected list in `panel.json` before detector
scoring and reuse it for those same reader inputs across calibration revisions. Add targeted Romulus v1/38, Pericles
v1/214–215, Lysander v1/535, Dio v2/580, Antonius v2/562 and partial duplicate
openings, counted separately. Inspect missing-ink masks and transform/threshold
uncertainties before adjusting parameters. Freeze settings/policy/runtime IDs
when all case criteria and reviewed retained controls pass. Keep raw earlier
predictions on parameter revision; post-revision results are regression, not a
new independent validation. Report every panel miss/false alarm/uncertainty with
its actual denominator.

**LLM verification:** Run `verify --out <prepared-root>` and inspect all seven case
results/variants and exact geometry. Confirm no omitted glyph passes through an
aggregate score and no redundant copy becomes a unique loss. Inspect real panel
results, policy/settings/code freeze IDs and stable output records. Compile
changed Python files. If Phase 2 exists, compare coverage and seams with/without
trace diagnostics; this is the deferred integration comparison.

**Human verification:** View all disposable pairs and targeted real-source controls,
then the fixed panel. Confirm source damage and weak observations remain visible.
Do not change tolerances to suppress genuine endpoint deficits or unresolved maps.

### 7.1a Verify two real horizontal-split acceptance controls

Keep the two input-bound real defects in
`tools/facsimile-real-acceptance-controls.json`, separate from both the seven
constructed cases and the fixed random panel. They are Tomus I physical pages
212–213 / folios 196–197 (source v1/105, Lycurgus–Numa comparison), and Tomus II
physical pages 20–21 / folios 12–13 (source v1/200, Pericles). The latter's quoted
text and folios identify pages 20–21 in the current authenticated PDF; retain
the user's originally reported 21–22 as provenance. Source boxes come from
independent visual inspection of intact source rows, never inferred reader cuts.

Require a specific `glyph_cut_across_reader_pages` finding for each source row
and both reader-page edges. A generic unresolved result, wrong page pair, or
complete combined ink without an explicit glyph-cut warning is a missed control.
An ordered union can establish surviving ink while glyphs remain unreadable as
upper/lower halves on different pages. Report those two properties separately.
Normal breaks between intact glyphs/words must not become glyph-cut findings.

Use the existing source adjudication and visible-reader measurement helpers.
Add a separately versioned `controls` command to
`tools/facsimile_line_delivery.py`, accepting `--controls`, `--run-dir` and a
fresh `--out`. Authenticate the control input hashes, source geometry, actual
reader-page pair and measurement evidence. Preserve misses/unprocessed controls
and exit 4 unless both controls are specifically detected. Reuse a shared
glyph-cut predicate across control scoring and general seam reporting; do not
hard-code the two page numbers into the detector. Keep active-run comparison
code immutable; any comparison-code revision requires a new capture/run.

`facsimile_line_delivery.py glyph-cuts --run-dir <completed-run> --out <fresh.json>`
provides separately identified general seam evidence from reviewed source
regions and observed correspondence neighborhoods, using that shared predicate.
Attach matching reader-body seam IDs and retain machine ink status alongside
glyph-cut warnings. Record excluded/unlocalized source regions and failed fits;
zero warnings with unevaluated regions cannot establish readable completeness.
Targeted controls supplied with known page pairs do not establish automatic
corpus localization of those same controls.

**LLM verification:** Compile changed Python files, validate control input
hashes and geometry, and score both controls through real visible PDF evidence.
Require 2/2 specific findings with source and both reader-edge crops. Inspect a
normal intact-line page join as a retained control. Reject changed input hashes
and wrong page pairs. Rerun the seven complete cases if comparison predicates
change, preserving earlier predictions. Neither unresolved results nor review
labels alone count as automatic detection.

**Human verification:** Compare the intact source rows with both reader halves.
Confirm the findings identify the actual split characters, and distinguish
surviving combined ink from readable Latin. Header leakage is outside scope.

### 7.1b Bound registration to source pages that can support reviewed coverage

In `tools/facsimile_line_coverage.py`, `register_local_strips` must retain all
source-region correspondence records while deferring image fits on physical
source pages containing no `geometry_reviewed` logical line. Deferred records
have no registrations, unresolved image status and an explicit deferral reason.
Register every region on pages containing reviewed geometry, including unreviewed
neighbors: shared continuous fits used by coverage have the same source index
and physical page as their reviewed target. Preserve source boxes, ink masks,
ownership rules and registration thresholds. Report eligible/deferred record
counts and reviewed physical-page counts, and label progress as source-region
records rather than successful strip observations.

**LLM verification:** Compile Python, run all seven existing full-path cases,
and compare loss/control verdicts with the preceding captured implementation.
Use a disposable real-input probe to check that unreviewed neighboring regions
on reviewed pages retain their fits and wholly unreviewed pages retain all
regions with unresolved, deferred registrations. Run `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test`. Create a fresh
capture for changed comparison code; retain the failed corpus capture/log and
reuse authenticated raw renders/OCR rather than old verdicts.

**Human verification:** Read the registration-scope counts alongside the source
inventory. Confirm deferred pages remain unresolved and are not counted as
preserved Latin. Inspect the existing controlled omission findings for unchanged
source rectangles and reader-page evidence.

### 7.2 Execute all eight readers, verify identities and archive findings

Finish the setup guide and update the existing README audit section in place.
Run a full offline audit against a fresh accepted-reader snapshot containing
frozen policy/settings/runtime identities. Account for both source scans, all
68 units, physical pages including skipped scans, mapped reader occurrences,
and all derived seams. Separate requested/processed/failed/uncertain/unreviewed
counts and body/front-matter exclusions. A processing exit 0 is not complete
Latin coverage; report all unresolved source regions and localizations.

Example operation order (PowerShell, repository root; fresh names as needed):

```powershell
python tools/facsimile_line_audit.py check
python tools/facsimile_line_audit.py capture --reader-dir build_facsimile --snapshot-dir tmp/pdfs/facsimile_line_audit/snapshots/accepted_2026_10_10
python tools/facsimile_line_audit.py inventory --snapshot tmp/pdfs/facsimile_line_audit/snapshots/accepted_2026_10_10 --run-dir tmp/pdfs/facsimile_line_audit/runs/source-review-1
python tools/facsimile_line_audit.py verify --prepare --out tmp/pdfs/facsimile_line_audit/runs/verification/cases-1
python tools/facsimile_line_audit.py verify --out tmp/pdfs/facsimile_line_audit/runs/verification/cases-1
python tools/facsimile_line_audit.py run --snapshot tmp/pdfs/facsimile_line_audit/snapshots/accepted_2026_10_10 --run-dir tmp/pdfs/facsimile_line_audit/runs/all-1
python tools/facsimile_line_audit.py report --run-dir tmp/pdfs/facsimile_line_audit/runs/all-1
python tools/facsimile_seam_review.py --tasks tmp/pdfs/facsimile_line_audit/runs/all-1/review-tasks.json --out tmp/pdfs/facsimile_line_audit/runs/all-1/haiku-observations --max-calls 10 --dry-run
```

Document exact source/duplicate/loss adjudication commands, adopting reviewed
policy through `capture --source-policy <new-policy>` and settings through
`--settings <file>`. Updated captures/runs reuse raw renders but not stale verdicts.
Document tool/language paths, SDK install, optional trace inspection, stop stages,
live caps, unknown-delivery retry, replay and exit meanings. No production rebuild
or LuaLaTeX compilation is required.

Repeat the exact full invocation with `--resume`; stable inventory, logical/index,
ledger, seams and tasks must be byte-identical. Exclude timing/model-response
variability. Rehash accepted source/reader/unit/repair files before/after. Check
executed helper-code identity before cache access, including a disposable changed
code preflight that must fail for the old run. Verify every report/crop link and
actual scope count. Archive snapshot, settings/policy, case inputs/outputs/reports,
source reviews, crops, ledger/seams/report and model/human observations to a new
dated archive; authenticate all digests without altering older evidence. Write a
dated completion note with findings, limitations and identities. Confirm new
losses are a reviewed worklist rather than activated repairs.

**LLM verification:** Compile Python files, run the seven complete cases and full
resume comparison, inspect manifest/error counts and archive digests. Run existing
repository gates: `cargo fmt --all --check`, `cargo clippy --all-targets -- -D
warnings`, and `cargo test`; Rust remains unchanged. Run the existing portable
snapshot check if shared Python changes affect it, staging intended implementation
files first. Inspect Git scope for audit/document changes and any explicitly
user-authorized preexisting document edit; production/source files must be unchanged.

**Human verification:** Open representative complete seams from every tomus and
inspect every proposed new loss and unresolved ownership/equivalence case. Read
completion denominators and limits. Seven successful scenarios establish their
specific sensitivity, not exhaustive character-by-character corpus completeness.

# Acceptance checklist and continuation instructions

- The implementation is Python; production Rust/readers and source Latin remain
  unchanged. No new unit-test suite or mock framework is introduced.
- Expected source ink/tokens are independent of production cuts/skips, with explicit
  tokenless objects and full physical duplicate inventory.
- Source-only gutter/baseline grouping accounts for residual ink and uncertainty;
  it shares the screened scale estimator and never silently drops uncertain Latin.
- Canonical duplicate transforms and ordered component unions permit one complete
  copy while preserving partial-copy deficits and unique shared-page continuations.
- One shared region-registration primitive and explicit whole-ink/end predicates
  drive coverage; failed fits stay unresolved, and unit endpoints have one-sided rules.
- Shared output profiles preserve historical defaults and narrowly authorize fixture
  PDF creation inside fresh verification roots; snapshots/production inputs remain protected.
- Executed helper/tool/language/settings identities govern comparison caches and resume.
  CLI stages, adjudication modes, exit meanings and Haiku retry states are explicit.
- All seams use the same ledger/renderer. Haiku uses the existing key lazily,
  submits only explicit capped requests, and cannot alter source or repair policy.
- Seven realistic full-path cases, reviewed source controls, scope checks and a
  determinism rerun provide verification. Existing repository suites remain green.
- Each step is independently verifiable with features available at that point;
  optional provenance does not block the core audit.
- Findings, uncertainty and limits are reported honestly. Historical plans,
  review notes and all durable evidence remain on disk.

An implementer should read the current helper APIs and latest delivery note, then
follow the preserved identifiers (including 5.1a); Phase 2 may be deferred.
Record progress in a separate dated implementation note. Revise affected plan
text in place when actual APIs differ, without renumbering existing steps. Reuse
shared helpers and explicit adapters; do not port production heuristics or copy
matchers. Code comments describe current behavior, never plan phase/step IDs.
