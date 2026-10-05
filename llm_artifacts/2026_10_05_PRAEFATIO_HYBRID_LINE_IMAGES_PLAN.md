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

## Phase 1 — Define regions and migrate the finalized cache (Suggested implementer: Opus)

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

### 1.2 Make the prose boundary a deliberate cache edit

Edit only source 1 page 15's finalized `result.blocks[0].text` in
`plutarch.frontmatter.tags.json`: preserve its exact prefix through the unique
sentinel and remove the following corrigenda suffix. Keep `continues_previous`
true. Do not change the prefix's spelling, ligatures, spacing or punctuation.
Retain page 16's historical record and every candidate pass, image hash and
verification record. They cease to be emitted, rather than being deleted.
Record the exact cache edit and its source-image justification in this plan.
Do not silently trim strings inside a LaTeX renderer.

Update committed-cache tests in `frontmatter.rs`: add
`praefatio_cache_opening_ends_at_ages`, and preserve heading/continuation checks
for p14, p15 and p17. Historical page-16 tests may continue to verify that its
retained record ends at the source period, but must not imply it remains prose.

**LLM verification:** Parse the JSON, assert the new finalized text equals the
old prefix byte-for-byte and candidate passes remain equal. Run `cargo fmt --all`,
`cargo fmt --all --check`, `cargo clippy --target-dir target --all-targets -- -D warnings`,
and `cargo test --target-dir target praefatio_cache`; inspect the cache diff.

**Human verification:** Read the preserved p15 prose against the source through
`Ages.` and confirm the removal changes representation, not editorial content.

## Phase 2 — Scope transcription and validation (Suggested implementer: Sonnet)

### 2.1 Exclude image-only pages and mask mixed p15 for both API stages

Add `is_front_image_page(source,page)` in `frontmatter.rs`, combining the Monitum
and Praefatio image-only lists. Use it in both `work_remains` and the transcription
scope filter so classification as `FrontProse` cannot admit p16 to transcription
or reconciliation. Keep p15 in scope. Add
`prepare_front_prose_image(source,page,bytes) -> Result<Vec<u8>>`: return other
pages unchanged; decode p15, white out all pixels below the transition line and
the transition-line portion at/after `errata_start_x`, then encode PNG. Use the
shared region specification. Preserve the prose portion of that row, including
`Ages.`. Mask furniture consistently with the existing front-prose prompt.

Wrap the injected `render_full` inside `transcribe_front_matter` once, and pass
that prepared renderer to both `transcribe_all` and `reconcile_all_in_scope`.
This keeps hashing and arbitration tied to the identical masked image and makes
mock-renderer tests exercise the actual policy. Do not create another API path or
add a Praefatio-specific model prompt. A finalized cache result must not trigger
retranscription merely because input masking has changed. Explicit retagging
uses masked images and records their new hashes through existing machinery.

Restrict `validate_praefatio_results` to opening and closing prose pages. Add a
boundary validator requiring a unique sentinel at the end of p15's finalized
prose whenever that result is present. Reject an un-migrated full mixed paragraph
with actionable source/page/block context; never warn and emit its suffix.
Preserve incomplete-cache behavior for absent records, but if a Praefatio is to be
emitted, require all three finalized prose pages in Phase 4 rather than silently
producing an image-only or incomplete section.

**LLM verification:** Add `work_remains_ignores_praefatio_image_page`,
`transcribe_and_reconcile_exclude_praefatio_image_page`,
`praefatio_mask_preserves_prose_and_removes_errata`,
`praefatio_validation_rejects_unmigrated_mixed_result`, and
`praefatio_validation_ignores_image_only_result`. Use synthetic PNGs and mock API
backends; prove both stages receive the same prepared image. Preserve existing
Monitum exclusion and lazy-client tests. Run `cargo fmt --all`,
`cargo fmt --all --check`, `cargo clippy --target-dir target --all-targets -- -D warnings`,
`cargo test --target-dir target praefatio`, and
`cargo test --target-dir target work_remains`.

**Human verification:** Inspect the actual prepared p15 image: its final Latin
word remains complete, no corrigendum is visible, and its transcription prompt
still preserves original orthography. Review a rejection message.

## Phase 3 — Extract and pack intact corrigenda lines (Suggested implementer: Opus)

### 3.1 Build a bounded editorial-line extractor

Add `src/facsimile/praefatio.rs` and declare it in `src/facsimile/mod.rs`.
Implement `extract_errata_lines(page, image, regions) -> Result<Vec<GrayImage>>`.
Use the explicit full editorial-column bounds, not `detect_latin_column`.
Reuse `lines::line_bands`, `lines::cut_points`, `columns::INK_THRESHOLD`, and
`metrics::line_metrics` for projection/cut calculations. Restrict p15 to the
transition line and rows below it; retain only the right-hand corrigenda portion
of the first line. Place that portion at the left of a white canvas of the same
column width as subsequent lines. Subsequent lines preserve source indentation.
Page 16 contributes every body line. Concatenate p15 then p16, with no artificial
source-page separator or extra gap.

Do not apply `drop_furniture`, body-title heuristics, narrow-line deletion, or
minimum-ink rejection to these already bounded regions. Projection fragments
from isolated accents must be grouped with their printed baseline before using
mid-gap cuts. Tune grouping on actual rendered pages and expose its threshold in
one definition. Fail with source/page context on empty extraction or ambiguous
merged baselines; do not silently omit a line. Keep a debug preview showing
numbered line boxes so the implementer and human can audit completeness.

**LLM verification:** Add `praefatio_first_errata_line_excludes_ages`,
`praefatio_line_extraction_preserves_accents_and_short_lines`, and
`praefatio_errata_preserves_source_page_order`. Synthetic tests must account for
all dark pixels inside selected regions, with none from the prose half-row or
furniture. Run `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy --target-dir target --all-targets -- -D warnings`, and
`cargo test --target-dir target praefatio`.

**Human verification:** Compare numbered crops row by row against both source
pages, including the page join and last line. Confirm accents and descenders are
attached to their lines and no content is duplicated or missing.

### 3.2 Pack balanced image pages with one scale and actual content heights

Implement `compose_errata_pages(lines, geom) -> Result<Vec<GrayImage>>` in the
new module. Reuse `TrimGeom::new(BODY_DPI)` and its body height budget. Use one
global scale bounded by `geom.content_width() / maximum_source_column_width`;
the p95 body scale alone can overflow an unusually wide line. Preserve aspect
ratios and a consistent source-derived interline gap. Do not scale individual
short lines to full width. Do not use body-header allowance.

Extract reusable image scaling/placement primitives from `reflow.rs` if needed,
with body behavior unchanged. Do not call `compose_pages` unchanged: its filtering
and tall-strip cuts can lose editorial material. Share low-level mechanics;
keep the bounded editorial preservation policy explicit.

Find the minimum page count that fits whole scaled lines into the safe height
budget. Partition the ordered lines into that many contiguous groups, choosing
breaks to minimize differences in occupied page heights subject to the budget.
Use deterministic tie breaking (earlier break). Never split a baseline to fill
space or stretch vertical gaps to fill a page. If a line cannot fit an empty page,
return an error with its source identity instead of clipping. Each output canvas
has the text-block width and its group's actual occupied height. This balances a
short tail without changing type size or order. Save assets as
`front_praefatio_errata_001.png`, etc., in `out_dir`; return ordered names from
`render_praefatio_errata(pdf,out_dir) -> Result<Vec<String>>`.

**LLM verification:** Add `praefatio_errata_uses_one_scale`,
`praefatio_errata_balances_pages_without_splitting_lines`,
`praefatio_errata_packs_across_source_page_boundary`,
`praefatio_errata_has_no_empty_canvas`, and
`praefatio_errata_rejects_oversized_line`. Verify pixel bounds, stable ordering,
line-count conservation, and deterministic breaks. Run `cargo fmt --all`,
`cargo fmt --all --check`, `cargo clippy --target-dir target --all-targets -- -D warnings`,
`cargo test --target-dir target praefatio`, and
`cargo test --target-dir target facsimile::reflow`.

**Human verification:** Read every composed image at intended print size, compare
Greek readability with Monitum, and confirm no page has a tiny tail caused by the
source-page boundary. Record actual scale, line count, heights and page count.

## Phase 4 — Compose the hybrid section and preserve book structure (Suggested implementer: Opus)

### 4.1 Integrate ordered assets and one section frame

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
`latex.rs`. Review `src/volumes.rs`'s Praefatio overhead estimate against actual
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
`cargo test --target-dir target assign_front_prose`, and
`cargo test --target-dir target monitum`.

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
Do not hard-code the real Praefatio's physical starting page. Run this test when
the local tools are available; a string-only assertion cannot prove pagination.

Run all local gates:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy --target-dir target --all-targets -- -D warnings
cargo test --target-dir target
cargo build --target-dir target
cargo test --target-dir target praefatio_hybrid_compile_has_consecutive_content_pages -- --ignored --nocapture
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

Not implemented. Planning inspection found the shared-line boundary on p15,
confirmed the single mixed finalized p15 block, and identified existing line,
geometry, continuation, validation and recto-start helpers to reuse. Fill this
section during implementation with the tuned region coordinates, exact cache
edit, verification commands/results, line inventory and rendered output paths.
