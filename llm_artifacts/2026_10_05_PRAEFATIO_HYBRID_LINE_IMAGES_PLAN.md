# Praefatio: faithful Latin prose and reflowed corrigenda line images

## Context

The user requests the Monitum's hybrid treatment for the Praefatio in
`Ploutarchou_Bioi_v2.pdf`, physical PDF pages 14–17. Re-typeset the opening Latin
through `Ita, ut in duarum vitarum exemplis consistam, Ages.`, reproduce the
following mixed Greek/Latin corrigenda as source pixels, and resume re-typeset
Latin on page 17 at `Interpretationem latinam subjicere placuit Xylandri`.
The corrigenda must be copied and reflowed as individual printed lines to the
10.5 × 16 cm trim reader, rather than shrinking entire source pages.

The latest request for dedicated image pages determines the layout: opening
prose, consecutive corrigenda-only pages, then closing prose. “Continuously” means
no source-page breaks within either run, no parity blanks between these parts,
and no sparse image page created merely because the source moves from p15 to p16.
Natural space below the last prose paragraph remains possible with dedicated
image pages; avoid adding vertical centering, fixed-height image canvases, or
parity padding that magnifies it. Do not put the closing prose into an image.

This is an implementation contract, not an authorization to implement during the
planning session. Preserve all existing dated plans and notes.

## Current state of the world

- `src/facsimile/frontmatter.rs` owns `SIDECAR`, `work_remains`,
  `transcribe_front_matter`, `run_front_matter`, `render_front_scans`, and
  `validate_praefatio_results`. `PRAEFATIO_PAGES` currently lists `(1,14)` through
  `(1,17)`. Source indices are zero-based; PDF page numbers are one-based.
- `plutarch.frontmatter.tags.json` has finalized page-14 heading and paragraph,
  one page-15 paragraph marked `continues_previous`, one page-16 paragraph marked
  `continues_previous`, and two separate page-17 paragraphs (prose and dateline).
  Page 15's paragraph contains both opening prose and corrigenda in one string.
  Its full-text candidate passes remain useful historical evidence.
- Source-image inspection of `tmp/pdfs/praefatio-source-015.png` shows that
  `exemplis consistam, Ages.` and `c. III, 5 præstant …` share a printed line.
  A rectangular crop beginning at that row would duplicate Latin. Split that
  first row horizontally after `Ages.`; retain all subsequent corrigenda rows.
  Page 16's entire editorial body, including embedded Latin and its final
  `cap. LXVIII, 4 …` line, belongs to the image run. Its head, folio and Google
  watermark do not. Do not resume typesetting at a Latin sentence inside p16.
- `src/facsimile/mod.rs::fill_front_matter` renders scans first, loads/transcribes
  the cache once, and calls `assign_front_prose`. That helper validates finalized
  Praefatio results before mutating volumes. Current assignment calls
  `latex::front_prose_pages` over all four pages.
- `src/latex.rs::front_prose_pages` merges continued paragraphs with
  `assemble::append_blocks`, then uses `front_prose_page_blocks` for a single
  frame with `\freshpage`, running-head reset, grouped
  `\emergencystretch=3em`, and final `\clearpage`.
  `front_prose_pages_until` is a Monitum helper: it selects whole blocks and warns
  and emits everything if the sentinel is absent. It cannot safely split p15's
  mixed block and creates source-page frames if used over multiple pages.
- Monitum excludes `(0,13)` from transcription and reconciliation using
  `MONITUM_IMAGE_PAGES`, and renders two tuned crops. `assemble::inner_document`
  appends `vol.prose`, followed on Tomus I by `latex::embed_scan_pair`.
  Do not alter Monitum's crop sizes or combined-page behavior for this work.
- `src/facsimile/lines.rs` provides `line_bands` and `cut_points`;
  `columns.rs` supplies `INK_THRESHOLD`; `metrics.rs` supplies `line_metrics`;
  `reflow.rs` supplies `global_scale`, `TrimGeom`, `BODY_DPI`, and `compose_pages`.
  Body detection assumes a right Latin column. Its furniture filters and packer
  can discard small-ink strips and split tall bands. Neither assumption is safe
  for a bounded single-column editorial extract with Greek diacritics.
- `FacsimileVolume` in `src/facsimile/assemble.rs` carries a `prose: String` and
  `emit_front_matter`. Restricted `--unit`/`--pages` runs suppress front matter.
  Assembly uses `\unitrecto` before the body and starts body numbering after
  alignment. Retain this contract, including intentional parity padding before
  the first Life; it is outside the Praefatio.
- `remove_composed_pngs` already removes files beginning `front_` after successful
  compilation. New assets should use this prefix and remain available during
  `--no-compile` builds. `src/volumes.rs::PRAEFATIO_PAGES` is an overhead estimate,
  distinct from the frontmatter source-page list.
- The earlier continuity/cache repair plan is implemented. Its cache-faithfulness,
  protocol-artifact validator, paragraph-continuation handling, and scoped
  emergency stretch remain relevant. Some tests currently require page 16 to be
  emitted as prose; update those expectations rather than preserving obsolete
  behavior. Existing source transcription must never be normalized downstream.

## Desired end state

1. On the first emitting tomus of source 1 (currently Tomus V), one display
   `PRÆFATIO.` opens the typeset p14–p15 prose. Its last text is exactly the cached
   source prefix ending in `Ages.`; p14→p15 continuation remains a single paragraph.
2. Every corrigenda line from the remainder of p15 and all editorial content of
   p16 appears exactly once as a scaled line image, in reading order. Preserve
   Greek accents, Latin labels, punctuation, indentations and source line endings.
   “Line reflow” here means moving intact printed lines onto new trim pages; it
   does not mean OCR, rewriting Greek, or splitting lines into words.
3. Corrigenda pages follow immediately, use one common aspect-preserving scale,
   omit source furniture, and pack lines across the p15→p16 join. Balance page
   loads to avoid an almost-empty final image page. Do not enlarge short lines
   independently, stretch glyphs, crop right edges, or force all errata onto one page.
4. After the image run, page-17 Latin and dateline are typeset faithfully, with
   separate paragraphs and no repeated title or intervening parity blank.
5. New transcriptions and reconciliation of p15 see the Latin region alone;
   `(1,16)` never enters either API stage. A complete migrated cache builds offline
   without constructing a vision client. Historical candidate text is retained.
6. Monitum, other tomi, restricted runs, first-Life recto alignment, body folios,
   flyleaves, and signature padding retain their existing behavior. Actual compiled
   page counts drive imposition; no fixed expected Praefatio page count is imposed.

## Tech Learnings

A source line can contain the prose/image boundary horizontally. A row-level crop
alone cannot express that boundary. Store both the transition row and the first
corrigendum's x coordinate, and use the same region definition for transcription
masking and image extraction. LaTeX `\clearpage` advances to the next page without
parity padding; `\freshpage` and `\unitrecto` can add a blank. Image-page height
must represent actual ink and line spacing, not a full blank source-page canvas.

## Phase 1 — Define regions and prepare the cache migration (Suggested implementer: Opus)

Phases 1–3 prepare and test helpers without changing the active cache, API scope,
validator, or document assignment. Step 4.1 activates those changes together with
the finalized-cache migration and existing-test updates. Every preparatory step
must leave the current complete Praefatio output and full test suite passing.
Behavior-preserving extraction of shared body-image primitives in step 3.2 may
run in production immediately; its pixel-equality tests must prove no output change.
Keep unused preparatory definitions under temporary `#[cfg(test)]` gates until
step 4.1 connects them to production; do not suppress dead-code warnings or copy
helpers into a second implementation. Remove those gates when activating callers.

### 1.1 Pin down one shared source-region specification

In `src/facsimile/frontmatter.rs`, add
`PRAEFATIO_OPENING_PAGES = [(1,14),(1,15)]`,
`PRAEFATIO_CLOSING_PAGES = [(1,17)]`,
`PRAEFATIO_IMAGE_PAGES = [(1,16)]`, and
`PRAEFATIO_PROSE_SENTINEL = "Ita, ut in duarum vitarum exemplis consistam, Ages."`.
Retain the existing four-page list temporarily while dependent callers migrate.
Add `PraefatioRegions` and `praefatio_regions(width, height, page)` containing
validated pixel bounds for the body and, on p15, the transition line top/bottom
and `errata_start_x`. Define tuned normalized source coordinates in one place,
convert by rounding at render resolution, and reject empty/out-of-bounds regions.

Render p15 and p16 at `TRANSCRIBE_DPI` with `render::render_page`. Visually tune the
coordinates at that resolution; the existing lower-resolution screenshots are
guidance, not authoritative crop coordinates. Retain small white safety margins
around accents and descenders. The transition-line x boundary lies in the white
space after `Ages.` and before `c. III, 5`. Record the final coordinates and crop
previews in this plan's implementation record. Do not infer the boundary from OCR.

**LLM verification:** Add `praefatio_regions_scale_with_render_dimensions` and
`praefatio_regions_reject_invalid_bounds`. Run `cargo fmt --all`,
`cargo fmt --all --check`, `cargo clippy --target-dir target --all-targets -- -D warnings`,
and `cargo test --target-dir target praefatio_regions`.

**Human verification:** Compare the annotated transition line and full p16 body
box with the PDF: the last prose period, first corrigendum, final editorial line,
and every accent must be included in the appropriate region; furniture excluded.

### 1.2 Prepare the deliberate prose-boundary cache edit

Calculate and review the exact proposed edit to source 1 page 15's finalized
`result.blocks[0].text`, but leave `plutarch.frontmatter.tags.json` unchanged in
this step. In a test-only cloned sidecar, preserve the exact prefix through the
unique sentinel and remove the following corrigenda suffix. Keep
`continues_previous` true. Do not change spelling, ligatures, spacing or
punctuation. Retain page 16's record and every candidate pass, image hash and
verification record. Step 4.1 applies this edit to the committed cache together
with the replacement image renderer, so no intermediate build loses corrigenda.
Record the proposed boundary and its source-image justification in this plan.
Do not introduce a runtime trimming helper or silently trim strings in LaTeX.

Add `praefatio_cache_migration_preserves_prefix_and_candidates` in
`frontmatter.rs`, exercising the test-only edited clone. Preserve current
committed-cache tests and output expectations until step 4.1. That step adds
`praefatio_cache_opening_ends_at_ages` against the migrated committed cache and
preserves heading/continuation checks for p14, p15 and p17. Historical p16 tests
may continue to verify its retained ending without implying it remains prose.

**LLM verification:** Parse the JSON, assert the new finalized text equals the
old prefix byte-for-byte and candidate passes remain equal in the clone; assert
the committed cache has no diff. Run `cargo fmt --all`,
`cargo fmt --all --check`, `cargo clippy --target-dir target --all-targets -- -D warnings`,
`cargo test --target-dir target praefatio_cache`, and
`cargo test --target-dir target`; inspect the proposed prefix and unchanged cache.

**Human verification:** Read the preserved p15 prose against the source through
`Ages.` and confirm the removal changes representation, not editorial content.

## Phase 2 — Prepare transcription and validation policies (Suggested implementer: Sonnet)

### 2.1 Prepare image exclusions, mixed-page masking and legacy-cache protection

Add `is_front_image_page(source,page)` in `frontmatter.rs`, combining the Monitum
and Praefatio image-only lists. Test this predicate now; connect it to
`work_remains` and the transcription scope filter in step 4.1 so classification
as `FrontProse` cannot admit p16 to either API stage. Keep p15 in scope. Add
`prepare_front_prose_image(source,page,bytes) -> Result<Vec<u8>>`: return other
pages unchanged; decode p15, white out all pixels below the transition line and
the transition-line portion at/after `errata_start_x`, then encode PNG. Use the
shared region specification. Preserve the prose portion of that row, including
`Ages.`. White out pixels outside the shared p15 editorial-body rectangle as
well, excluding the running head, folio and watermark. Keep original dimensions;
the complementary first-row masks must neither erase nor duplicate glyph ink.

In step 4.1, wrap the injected `render_full` inside `transcribe_front_matter`
once, and pass that prepared renderer to both `transcribe_all` and
`reconcile_all_in_scope`.
This keeps hashing and arbitration tied to the identical masked image and makes
mock-renderer tests exercise the actual policy. Do not create another API path or
add a Praefatio-specific model prompt. A finalized cache result must not trigger
retranscription merely because input masking has changed. Explicit retagging
uses masked images and records their new hashes through existing machinery.

Prepare `validate_praefatio_prose_results`, checking opening and closing pages;
step 4.1 makes the existing `validate_praefatio_results` delegate to it. Add a
boundary validator requiring a unique sentinel at the end of p15's finalized
prose whenever that result is present. Reject an un-migrated full mixed paragraph
with actionable source/page/block context; never warn and emit its suffix.
Preserve incomplete-cache behavior for absent records, but if a Praefatio is to be
emitted, require all three finalized prose pages in Phase 4 rather than silently
producing an image-only or incomplete section. Existing p16 rejection tests
retain their present expectations until the integration step.

Add `validate_praefatio_candidate_state(sidecar) -> Result<()>`. For source 1
p15 with no finalized `result`, reject any retained `pass_a` or `pass_b` whose
block stream does not contain exactly one sentinel ending the final nonempty
prose block with no subsequent nonempty blocks. Do not trim, promote, remove or
rewrite those candidates. If a finalized result exists, this guard leaves its
historical candidates alone; the finalized-prose validator checks the result.
Missing candidates are permitted for fresh masked-image transcription.
Extract `validate_praefatio_prose_boundary(doc)` for the sentinel rule and reuse
it in both candidate-state and finalized-prose validation.

In step 4.1, invoke the candidate guard in `run_front_matter` after `load_or_init`
and before the lazy vision factory, and in `transcribe_front_matter` before
classification or either API stage, including direct/test callers. A legacy
candidate must never be fed to masked-image arbitration or copied into a new
result by free reconciliation. Return source/page context and two explicit
recovery options: restore a reviewed finalized prose prefix in the cache, or
deliberately regenerate using `--retag`. Explain that the existing `--retag`
resets the entire front-matter sidecar and discards its corrections and candidate
evidence; advise preserving a copy first. Do not invoke retagging automatically.
This guard defines recoverable behavior without another ingestion path or schema.

**LLM verification:** Add `work_remains_ignores_praefatio_image_page`,
`transcribe_and_reconcile_exclude_praefatio_image_page`,
`praefatio_mask_preserves_prose_and_removes_errata`,
`praefatio_validation_rejects_unmigrated_mixed_result`,
`praefatio_validation_ignores_image_only_result`,
`praefatio_missing_result_rejects_legacy_candidates_before_api`, and
`praefatio_valid_result_preserves_candidates_when_other_page_needs_work`.
In this preparatory step, exercise new pure policy helpers and a test-only
prepared-renderer closure without changing production callers. Step 4.1 wires
mock-backend integration tests proving both stages receive the same prepared
image, legacy candidates fail before any API call or sidecar mutation, and a
valid finalized prefix and its candidates remain byte-for-byte unchanged while
another page is processed. Preserve existing
Monitum exclusion and lazy-client tests. Run `cargo fmt --all`,
`cargo fmt --all --check`, `cargo clippy --target-dir target --all-targets -- -D warnings`,
`cargo test --target-dir target praefatio`,
`cargo test --target-dir target work_remains`, and `cargo test --target-dir target`.

**Human verification:** Inspect the actual prepared p15 image: its final Latin
word remains complete, no corrigendum is visible, and its transcription prompt
still preserves original orthography. Review a rejection message.

## Phase 3 — Extract and pack intact corrigenda lines (Suggested implementer: Opus)

### 3.1 Build a bounded editorial-line extractor

Add `src/facsimile/praefatio.rs` and declare it in `src/facsimile/mod.rs`.
Define `ErrataLine { image: GrayImage, source_page: u32, line_index: usize,
source_rect: (u32,u32,u32,u32) }`; source index is always 1, `line_index` is
one-based within each source page, and `source_rect` records the full-page
`(x,y,width,height)` crop before the first-line horizontal relocation. For that
first strip the rectangle starts at `errata_start_x` and excludes the prose
portion. The image
contains surrounding half-gaps defined below. Implement
`extract_errata_lines(page, image, regions) -> Result<Vec<ErrataLine>>`.
Use the explicit full editorial-column bounds, not `detect_latin_column`.
Reuse `lines::line_bands`, `lines::cut_points`, `columns::INK_THRESHOLD`, and
`metrics::line_metrics` for projection/cut calculations. Restrict p15 to the
transition line and rows below it; retain only the right-hand corrigenda portion
of the first line. Place that portion at the left of a white canvas of the same
column width as subsequent lines. Subsequent lines preserve source indentation.
Page 16 contributes every body line. Mask the first row's prose portion before
band detection or ink-coverage checks, using the shared transition-line bounds.
Concatenate p15 then p16 with the normal line gap specified below and no added
source-page separator.

Do not apply `drop_furniture`, body-title heuristics, narrow-line deletion, or
minimum-ink rejection to these already bounded regions. Projection fragments
from isolated accents must be grouped with their printed baseline before using
mid-gap cuts. Compute raw bands with `line_bands(region, 0)` so every row with
dark ink is represented. Add one tuned normalized `ACCENT_JOIN_GAP_FRAC`
constant, converted to pixels using full source-page height. Merge successive
raw bands whose separating blank gap is at most that threshold, transitively.
Choose the threshold by inspecting p15 and p16 at 300 DPI and record it alongside
the region coordinates. It must join isolated accents while keeping neighboring
printed baselines separate; do not use `line_metrics` before grouping to infer it.

If no such threshold works, or the preview has a merged baseline, missing line,
or uncertain grouping, use reviewed normalized per-line vertical ink intervals
for the affected page, stored alongside the shared region definitions. Convert
them with the same rounding rule; require disjoint, ordered intervals, exactly
one visually verified printed line per interval, and coverage of all dark pixels
in the selected region. Record the reviewed line inventory and whether each page
uses automatic grouping or explicit intervals. Empty extraction, overlap, or
uncovered ink is an error with source/page/line context, never silent deletion.

Use `cut_points` on the grouped ink intervals. Each strip carries the whitespace
up to the adjacent mid-gap cuts; do not add an independent gap between those
strips. To make the p15→p16 join match ordinary spacing, compute
`join_gap_px` as `metrics::median_u32` over the blank gaps between grouped lines
within the two pages. Add `floor(join_gap_px/2)` white rows after p15's final
strip and the remaining rows before p16's first strip. Do not include a source
footer or header in that gap. This padding becomes part of the two `image`s;
`source_rect` remains the original crop. No gap is added again by the packer.
Keep a debug preview and inventory showing source page, line index, crop rectangle
and grouping mode for every strip.

**LLM verification:** Add `praefatio_first_errata_line_excludes_ages`,
`praefatio_line_extraction_preserves_accents_and_short_lines`, and
`praefatio_errata_preserves_source_page_order`. Synthetic tests must account for
all dark pixels inside selected regions exactly once before resampling, with
none from the prose half-row or furniture. Add
`praefatio_reviewed_intervals_reject_overlap_and_uncovered_ink` and
`praefatio_page_join_adds_one_normal_gap`. Check line identity separately from
ink conservation: preserving pixels alone does not prove correct grouping.
Run `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy --target-dir target --all-targets -- -D warnings`,
`cargo test --target-dir target praefatio`, and `cargo test --target-dir target`.

**Human verification:** Compare numbered crops row by row against both source
pages, including the page join and last line. Confirm accents and descenders are
attached to their lines and no content is duplicated or missing.

### 3.2 Pack balanced image pages with one scale and actual content heights

Implement `compose_errata_pages(lines, geom) -> Result<Vec<GrayImage>>` in the
new module, accepting `&[ErrataLine]`. Reuse `TrimGeom::new(BODY_DPI)` and set
`budget = geom.body_budget()`, including its existing bottom slack. Fix one
global scale to `geom.content_width() as f32 / maximum_source_column_width as f32`;
the p95 body scale alone can overflow an unusually wide line. Preserve aspect
ratios and the half-gaps already present in each strip. Add no other line spacing.
Do not scale individual short lines to full width. Do not use body-header allowance.

Extract `pub(crate) fn resize_line_image(raw: &GrayImage, scale: f32) -> GrayImage`
from `reflow.rs::scale_strip`, preserving its rounded dimensions (minimum one
pixel) and `FilterType::Lanczos3`. Make `scale_strip` crop then call that helper.
Extract `pub(crate) fn place_line_images(strips: &[GrayImage], geom: &TrimGeom)
-> GrayImage` from `pack_canvas`, preserving its width, actual height, white
background, offsets and placement order; make `pack_canvas` delegate to it.
The editorial compositor must use both helpers, not copy their resize or overlay
loops. Body output must remain pixel-identical in representative fixtures.
Do not call `compose_pages` unchanged: its filtering and tall-strip cuts can
lose editorial material. Share low-level mechanics;
keep the bounded editorial preservation policy explicit.

Resize once at the fixed scale, then calculate all loads from the final rounded
image heights, including carried half-gaps and join padding. For a group define
`load = geom.pad_y + sum(strip.height())`, matching `place_line_images` exactly;
require `load <= budget` and every rounded width within the usable measure.
Greedily fill ordered whole strips to find the minimum page count. For that
fixed count, choose contiguous nonempty groups minimizing
`max(group_loads) - min(group_loads)` over all feasible partitions. On ties choose
the lexicographically smallest vector of exclusive end indices, excluding the
fixed final end. Implement `balanced_errata_breaks(heights, pad_y, budget)`:
build prefix sums and the sorted unique feasible loads of every nonempty
contiguous group. Enumerate candidate `[low,high]` load bounds from that set in
increasing `high-low` order. For each range use a dynamic-programming table
`can_finish(start, groups_remaining)` whose transitions try every exclusive end
index with a group load in range and enough lines left for the remaining groups.
Base case: zero groups succeeds exactly at the end of the line list. Reconstruct
the lexicographically earliest feasible break vector by trying end indices in
ascending order. Evaluate every range at the first feasible difference and choose
the smallest resulting vector; then stop. This bounded search is appropriate
for this short extract, avoids enumerating all partitions, and defines the exact
objective. Compare its result with exhaustive partition enumeration on small
fixtures. Never split a baseline to fill space or stretch vertical gaps to fill
a page. If a line cannot fit an empty page,
return an error with its source identity instead of clipping. Each output canvas
has the text-block width and its group's actual occupied height. This balances a
short tail without changing type size or order. Save assets as
`front_praefatio_errata_001.png`, etc., in `out_dir`; return ordered names from
`render_praefatio_errata(pdf,out_dir) -> Result<Vec<String>>`.

**LLM verification:** Add `praefatio_errata_uses_one_scale`,
`praefatio_errata_balances_pages_without_splitting_lines`,
`praefatio_errata_packs_across_source_page_boundary`,
`praefatio_errata_has_no_empty_canvas`,
`praefatio_errata_rejects_oversized_line`,
`praefatio_errata_partition_matches_exhaustive_optimum`,
`praefatio_errata_budget_uses_rounded_strip_heights`, and
`shared_line_primitives_preserve_body_pixels`. Verify pixel bounds, stable ordering,
line-count conservation, deterministic breaks and retained source identities for
oversized-line errors. Run `cargo fmt --all`,
`cargo fmt --all --check`, `cargo clippy --target-dir target --all-targets -- -D warnings`,
`cargo test --target-dir target praefatio`,
`cargo test --target-dir target facsimile::reflow`, and `cargo test --target-dir target`.

**Human verification:** Read every composed image at intended print size, compare
Greek readability with Monitum, and confirm no page has a tiny tail caused by the
source-page boundary. Record actual scale, line count, heights and page count.

## Phase 4 — Compose the hybrid section and preserve book structure (Suggested implementer: Opus)

### 4.1 Integrate ordered assets and one section frame

Activate the prepared helpers and policies from steps 1.1–3.2, removing temporary
test-only gates. In this same step apply the reviewed finalized p15 cache edit
from step 1.2, connect the image exclusions, masked renderer and pre-API candidate
guard from step 2.1, switch finalized validation to opening/closing pages, and
enable the complete hybrid document renderer. Do not leave a completed step or
commit with the shortened active cache and the old all-prose renderer.

Add `praefatio_errata_images: Vec<String>` to `FacsimileVolume`; initialize it
empty in every constructor and test fixture. In `fill_front_matter`, render these
assets only for `emit_front_matter && is_first_of_source && source == 1` and store
their names before loading prose. Keep `render_front_scans`' existing signature
and Monitum behavior; invoke the new renderer beside it. Restricted runs must
neither render nor reference these assets. Use the existing `front_` cleanup rule.

In `latex.rs`, extract shared block collection from `front_prose_pages` into
`collect_front_prose_blocks` without changing its missing-record/empty-document
continuation semantics. Extract the reusable unframed block-rendering portion
from `front_prose_page_blocks`; preserve its existing wrapper output for current
callers. Add `front_praefatio(sidecar,image_names) -> Result<String>`: require
nonempty image names and all opening/closing finalized results, run shared
validation, and collect the opening and closing blocks separately. A missing
prose page is an actionable error here, not a license to omit Latin.

Emit one opening `\freshpage` and `\markboth{}{}`; render the merged opening
blocks with the existing scoped emergency stretch; end the paragraph and group.
Use `\clearpage` to start the image run. Each image page uses `\noindent`,
`\includegraphics[width=\textwidth,keepaspectratio]{...}`, paragraph termination,
and `\clearpage`. No `center` environment, vertical centering, `\freshpage`,
`\unitrecto`, or redundant `\hbox{}` inside the run. Start closing prose on the
next consecutive page without a new title or parity command, preserve its two
paragraphs with scoped emergency stretch, and finish with `\clearpage`.

In `assign_front_prose`, call `front_praefatio` for the eligible source-1 volume.
Validate/build all new strings before assigning any volume, retaining the existing
no-partial-mutation guarantee. When cache loading fails in `fill_front_matter`,
return the error if an emitting Praefatio volume is present: continuing without
its prose or images cannot satisfy this hybrid section. Preserve existing warning
behavior for unrelated volumes. Keep `inner_document` appending `vol.prose` once;
it need not have another Praefatio-specific rendering branch. Correct the field
documentation to say that `prose` can contain the composed hybrid section.

Update obsolete page-16-as-prose tests in `mod.rs`, `frontmatter.rs` and
`latex.rs` here, alongside activation. Specifically change
`assign_front_prose_composes_finalized_praefatio` to expect the Latin prefix,
ordered image names and closing prose, rather than `consentiente ipso Plutarcho`.
Move the injected protocol marker in
`assign_front_prose_rejects_poisoned_praefatio_without_mutating_volumes` from
p16 to emitted p17; retain a separate test proving historical p16 contamination
is ignored. Supply image names in eligible fixtures and add the committed-cache
`praefatio_cache_opening_ends_at_ages` assertion. Run the Phase 2 integration tests
against actual production callers. Confirm the only cache change is the reviewed
p15 finalized suffix removal, with all candidates and p16 evidence intact.
Review `src/volumes.rs`'s Praefatio overhead estimate against actual
new pagination; adjust its estimate/comment only if warranted. Update README's
front-matter strategy and offline requirements in place. Keep previous artifacts.

**LLM verification:** Add `praefatio_hybrid_emits_prose_images_prose_once`,
`praefatio_hybrid_has_no_internal_parity_breaks`,
`praefatio_hybrid_keeps_page_seventeen_paragraphs_separate`,
`praefatio_hybrid_rejects_missing_prose_or_images`,
`assign_front_prose_hybrid_failure_does_not_mutate_volumes`, and
`praefatio_images_are_excluded_from_other_and_restricted_volumes`.
Assert the sentinel occurs once, image names are in order, p16 cached text is
absent, the closing prose follows the last image, and one heading/head reset is
emitted. Run `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy --target-dir target --all-targets -- -D warnings`,
`cargo test --target-dir target praefatio`,
`cargo test --target-dir target assign_front_prose`,
`cargo test --target-dir target monitum`, and `cargo test --target-dir target`.

**Human verification:** Read the generated section source, identifying its three
parts and consecutive breaks. Confirm no page furniture or image-era Greek is
typeset and all closing Latin/dateline text remains visible as text.

## Phase 5 — Compile and inspect the reader (Suggested implementer: Opus)

### 5.1 Run final gates, compiled regression coverage and full offline QA

Add an ignored integration test
`praefatio_hybrid_compile_has_consecutive_content_pages` in
`src/facsimile/assemble.rs`, following the existing LuaLaTeX/Poppler fixtures.
Use synthetic opening/closing prose and several unequal-height image pages to
verify physical adjacency, no blank inside the hybrid section, intact image
dimensions, first-Life recto alignment, and folio 2 on the second body page.
Do not hard-code the real Praefatio's physical starting page. Add a second ignored
compiled test, `praefatio_hybrid_compile_fits_maximum_image_budget`, with an errata
canvas whose height equals `TrimGeom::body_budget()` and width equals `tb_w`.
Verify that it stays on one content page, retains bottom ink inside the printable
margin, and introduces no extra page from TeX line-box height. Run both tests when
the local tools are available; string-only assertions cannot prove pagination.

Run all local gates:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy --target-dir target --all-targets -- -D warnings
cargo test --target-dir target
cargo build --target-dir target
cargo test --target-dir target praefatio_hybrid_compile_has_consecutive_content_pages -- --ignored --nocapture
cargo test --target-dir target praefatio_hybrid_compile_fits_maximum_image_budget -- --ignored --nocapture
```

Use a separate output directory, preserving existing generated readers:

```powershell
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
cargo run --target-dir target --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/praefatio_hybrid_check --offline
if ($LASTEXITCODE -ne 0) { throw "Praefatio hybrid build failed" }
```

Require compiled inner and imposed PDFs, not the LaTeX-only fallback. Locate
Praefatio by text, then use `pdftoppm` to render every page in the section and its
neighbors. `pdftotext -layout` proves the Latin is selectable and the title occurs
once, but cannot prove Greek image completeness: compare rendered images with
the source line inventory. Inspect the two prose/image transitions, p15→p16
image join, all Greek diacritics, the dateline, body opening and a representative
imposed sheet. Inspect Monitum's unchanged combined errata page as a regression.
Require no clipping, overlap, missing-character warnings or overfull boxes in
the section, and no interior blank pages. Record remaining natural prose-end
space separately from artificial gaps; do not sacrifice readability to force a
fixed page count.

**LLM verification:** Record gates, actual section span, scale/line counts,
compiled layout findings, actual inner/imposed counts, cache diff and offline
client-call evidence below. Check `git diff --check` and verify generated outputs
remain ignored. The new PNG names must be removed after successful compile but
retained for a no-compile run. Do not overwrite the original output directory
without a separate user request.

**Human verification:** Compare the complete Praefatio against PDF pages 14–17:
all opening Latin through `Ages.`, every corrigendum once, all page-17 prose and
the dateline. Assess image type size and balanced page loads at actual trim size.
Confirm body numbering and imposition remain suitable for printing.

## Implementation record

Step 1.1: rendered source pages at 300 DPI into `tmp/praefatio/source-015.png`
(1829 × 2827) and `source-016.png` (1840 × 2828). The normalized body edge
coordinates are p15 `(0.145, 0.074, 0.919, 0.907)` and p16
`(0.078, 0.078, 0.852, 0.916)`. The p15 transition uses y=1595–1654
and x=795 at its native render size; normalized values use those exact ratios.
The x boundary occupies the white gap after the period of `Ages.` and before
`c. III, 5`. Body margins retain the final editorial lines and exclude the
head, folio and Google watermark. Preview paths: `tmp/praefatio/transition.png`
and `tmp/praefatio/annotated-15.png`, `annotated-16.png`.
The definitions remain test-only until integration. Formatting, clippy with
warnings denied, and both region tests passed. The complete test suite is also
passed (306 library tests, 5 CLI tests; 13 ignored).

Step 1.2: the cloned cache edit preserves the exact byte prefix through the
unique `Ita, ut in duarum vitarum exemplis consistam, Ages.` sentinel. The
removed suffix begins ` c. III, 5`; the transition preview confirms that it
belongs to the right-hand image portion of the same printed row. A whole-sidecar
JSON comparison permits exactly that finalized-text edit and proves preservation
of continuation, candidates, hashes, page 16 and all remaining evidence. The
committed cache remains unchanged. Formatting, clippy and cache tests passed;
the complete suite is run for this step as well.

Step 2.1: prepared the shared image-page predicate, unchanged-dimension PNG
mask, finalized prose boundary validation and legacy-candidate guard, all under
test-only gates. Seven policy tests pass; the existing production behavior is
unchanged. `tmp/praefatio/masked-15.png` confirms the complete final `Ages.` and
no visible corrigendum or page furniture. The guard names source/page/block,
offers a reviewed finalized-prefix restoration or deliberate `--retag`, and
explains the whole-sidecar reset and need to preserve evidence first.
Formatting, clippy, Praefatio and work-remains tests passed; full-suite results
are checked before committing.

Step 3.1: automatic accent grouping uses `ACCENT_JOIN_GAP_FRAC = 0.001`
(3 pixels at native height). A 6-pixel trial merged p15's first two baselines
and was rejected; 3 pixels joins its detached final fragment while leaving all
neighboring baselines separate. Both pages use automatic grouping, with 16
reviewed p15 lines and 39 reviewed p16 lines. All 55 numbered crops were compared
row by row against the source; accents, descenders, short lines and the final
editorial period remain present. No explicit intervals are needed for these scans.
The durable crop inventory is `2026_10_05_PRAEFATIO_LINE_INVENTORY.txt`; individual
previews are `tmp/praefatio/lines/p15-01.png` through `p16-39.png` and contact
sheets `tmp/praefatio/sheet-{0,8,16,24,32,40,48}.png`. The first crop starts at
x=795; the source-page join adds one median normal gap, split across the two
adjacent strips. Five synthetic extraction/coverage/order tests and the real
source inventory test passed. Formatting and clippy passed; the full suite is
checked before committing. Preparatory extraction remains test-only.
Steps 1.1–3.1 verification: commits `844ca5f`, `c1bbc99`, `08377fd`, and
`ec19977` contain the reviewed regions, cloned migration, pure transcription
policies, and bounded editorial extractor respectively. The initial worktree
was clean. The starting full suite passed with 319 library tests and 5 CLI tests
(14 ignored). The complete mixed p15 finalized paragraph remained active until
the hybrid renderer and policy integration were connected.

Step 3.2: shared `resize_line_image` and `place_line_images` primitives preserve
body fixture pixels exactly, including rounded dimensions, Lanczos3, offsets,
and actual canvas height. The editorial compositor scales each strip once,
uses the widest source-column canvas, and chooses the minimum-page partition
with the minimum load range and lexicographically earliest breaks. Exhaustive
small-fixture comparisons verify the objective and tie rule. The real extract
has 55 lines (16 from p15 and 39 from p16), common scale 0.705056, and two
1004-pixel-wide canvases with heights 1192 and 1154. No source-page separator or
additional strip spacing is introduced. Both full-resolution canvases and their
trim-page renders were compared with source pages 15 and 16: the horizontal
boundary excludes `Ages.`, the source-page join preserves ordinary spacing,
and all accents, short lines, and the last editorial period are present.

Step 4.1: the prepared regions, masks, exclusions, and validators are active in
production. Both API stages receive the identical prepared p15 PNG; the mock
integration test forces actual arbitration through a synthetic numbering gap.
Legacy candidates fail before rendering, classification, any API request, or
sidecar mutation. Valid finalized p15 prose and its historical candidates remain
unchanged while another page is processed. Source 1 p16 is excluded from both
stages and finalized prose validation. An emitting Praefatio requires all three
finalized prose pages and image names; assignment builds every string before
mutating any volume. Other and restricted volumes do not reference these assets.
The complete-cache test makes client construction panic if attempted, and passes.
A whole-sidecar comparison against HEAD permits exactly the finalized p15 suffix
removal and proves that its retained prefix is byte-for-byte exact; candidates,
p16, continuation, hashes, and verification evidence are unchanged. README now
describes the hybrid and offline requirements. The measured six-page section
sets `PRAEFATIO_PAGES_EST` to 6. The QA build began with the previous estimate;
`praefatio_full_build_matches_current_volume_estimate` confirms that all eight
emitted volume memberships match the final estimate, so the generated documents
retain the final code's grouping.

Step 5.1: `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy --target-dir target --all-targets -- -D warnings`,
`cargo build --target-dir target`, and the full `cargo test --target-dir target`
passed: 336 library tests and 5 CLI tests, with 18 ignored. Both required ignored
compiled tests passed against the final code. They verify consecutive content
pages, intact image dimensions, recto body alignment, folio 2, and a maximum
budget canvas's bottom ink within the printable margin. The real-section preview
and final-volume-membership ignored checks passed too.

The full offline release build completed in
`build_facsimile/praefatio_hybrid_check`, preserving the original output directory.
Existing source raster caches were copied into its separate cache to avoid
unnecessary rasterization. All eight inner and imposed PDFs compiled:

| Tomus | Inner pages | Imposed pages |
| --- | ---: | ---: |
| I | 416 | 208 |
| II | 448 | 224 |
| III | 416 | 208 |
| IV | 128 | 64 |
| V | 384 | 192 |
| VI | 384 | 192 |
| VII | 448 | 224 |
| VIII | 320 | 160 |

Praefatio occupies physical Tomus V pages 12–17: opening prose 12–14,
corrigenda images 15–16, closing prose and separate dateline 17. The title and
prose sentinel each occur once in extracted selectable text. There is no blank
inside the section. Page 18 is the intentional first-Life alignment blank;
Nicias opens on recto 19 and the next body page carries folio 2. Every section
page and its neighbors were rendered. The six final section rasters are
pixel-identical to the individually inspected real-section preview. Tomus V has
no overfull boxes or missing-character warnings. Ordinary underfull warnings
remain for loose prose lines and ragged page ends; the natural space after the
opening prose is not a fixed-height canvas or vertical centering artifact.
The two composed image pages preserve the reviewed 55-line inventory, including
the p15→p16 join. Imposed sheet side 16 was inspected with image page 16 and
closing prose page 17. Monitum retains one prose page at physical 14, both crops
together at 15, one alignment blank at 16, and Theseus on recto 17; these renders
were inspected too.

Successful compilation removed the new `front_praefatio_errata_*.png` assets;
the uncompiled `build_facsimile/praefatio_section_preview` retains both assets.
Generated PDFs, PNGs, previews, and logs are ignored. The UTF-8 build log and
verification logs are under `tmp/praefatio/`. Task-scoped `git diff --check`
passed; unrelated concurrent `prompts.txt` edits were left untouched. Physical
print-size assessment remains available to the human reviewer using the inner
and imposed PDFs.
