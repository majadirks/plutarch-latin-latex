# Context

The eight facsimile readers are substantially complete. The remaining confirmed defects are recorded in [the inner-tomi visual findings](2026_10_05_INNER_TOMI_VISUAL_REVIEW_FINDINGS.md): duplicated or misplaced prose, retained source furniture and Google watermarks, excessive copied whitespace, divider-only body pages, and a raster boundary through a Latin line. This plan repairs those finite examples with small, edition-specific changes. It does not attempt to make the generator handle arbitrary editions or scans.

The user explicitly prefers hard-coded generator corrections where feasible. Reuse the existing source mask table and unit-boundary mechanisms; add small private helpers only when measured evidence requires them. Removing a watermark is successful only when both its pixels and its unwanted copied space disappear. The earlier [source artifact cleanup plan](2026_10_05_FACSIMILE_SOURCE_ARTIFACT_CLEANUP_PLAN.md) supplies useful implementation patterns, but its old output locators and source mappings do not identify the newly reported occurrences.

All Latin is source material. Preserve every distinct genuine Latin line, its glyphs, spelling, punctuation, capitalization, and reading order. Do not retype damaged prose, normalize text, edit transcription caches, or use OCR text as replacement content. Erasing source furniture and excluding a proven repeated scan occurrence are image/selection corrections, not permission to alter Latin. A questionable mask must remain disabled until source inspection proves that it preserves the prose.

The reviewed repair ledger freezes 13 furniture-mask records (14 rectangles) and two identity-scoped whole-page duplicate exclusions. The 2026-10-06 handoff selects these repairs for implementation. Open spacing, divider, and sliced-line findings require reassessment against rebuilt output before any additional erasure. Do not infer source PDF pages from reader folios or copy coordinates from a different scan.

# Current state of the world

- `src/facsimile/reflow.rs` defines `BODY_DPI = 300`, `SourceFurnitureMask`, `SOURCE_FURNITURE_MASKS`, `apply_source_furniture_mask`, and `mask_known_source_furniture`. `render_and_band` applies normalized, source/page-specific rectangles to an owned source image before column detection and banding. Raw render-cache images remain pristine. The existing entries cover older v1 pages 56, 233, 235, 240, and 284; preserve their successful behavior.
- `src/facsimile/columns.rs` contains column detection and conservative divider exclusion. Do not widen the general divider acceptance gates to force the new examples through.
- `src/facsimile/lines.rs` defines `Band`, `line_bands`, `drop_furniture`, and `cut_points`. The interior cut between two retained bands is the midpoint of their gap. Therefore removing an interior furniture band can still leave its entire blank interval distributed between adjacent strips.
- `unit_body_strips` in `reflow.rs` renders source pages in order, retaining a parallel `source_pages` vector. Pass A determines line metrics; Pass B trims unit openings, interior furniture, and shared tails using original source-height coordinates. `PageBands` is `(GrayImage, Vec<Band>)`.
- `strips_in_reading_order` constructs `Strip` values from retained bands and midpoint cuts. `compose_pages` scales and packs those strips. Ordinary lines wrap whole; tall merged strips can be split at low-ink rows. `split_tall_strip` can select a row with ink when no clear gap exists. A surviving divider can merge prose into a tall strip, so the reported sliced line may be repairable upstream without a packer change. This is a hypothesis to test, not a completed diagnosis.
- `src/facsimile/units.rs` defines `Unit`, `UnitRange`, `load_units`, and `ranges`. `facsimile_units.json` records `start_page`, `title_top_frac`, `body_top_frac`, `skip_top_bands`, and `pinned`. `ranges` derives an interior unit's end from the next opening; `last_page` controls the final unit of a source, not an arbitrary interior endpoint.
- `src/facsimile/detect.rs::merge_pins` preserves pinned table corrections during regeneration. Reuse pinned rows for confirmed boundary errors rather than inventing a parallel boundary configuration.
- Existing ignored source tests in `reflow.rs` provide `operational_range`, `real_source_image`, tracing, and retained-crop helpers. `src/facsimile/fixtures/artifact_source_trace.json` records the earlier cleanup. Reuse those helpers; store new evidence separately rather than rewriting the old ledger.
- `src/facsimile/frontmatter.rs` deliberately inserts the Google reproduction-notice page. `src/facsimile/praefatio.rs` provides the existing scanned Praefatio packing path and source-line inventory. The notice and mixed typography are separate from body watermark cleanup.
- Builds use `facsimile_units.json`, source PDFs `Ploutarchou_Bioi_v1.pdf` and `Ploutarchou_Bioi_v2.pdf`, existing offline front-matter caches, Poppler, and LuaLaTeX. Composed assets may be removed after compilation; diagnostics must save independent crops under `tmp/`.

# Desired end state

1. Eumenes/comparison and Agis prose occurs once in the correct sequence, with all distinct Latin accounted for.
2. The confirmed body watermarks and safely separable leaked furniture disappear. Their associated unwanted space disappears as well.
3. Timoleon and Brutus no longer contain the reported divider-only interior pages or severe furniture-driven interruptions. The unnecessary Marius furniture-only body image disappears; normal recto-start separators remain governed by the existing assembly logic.
4. The reported Timoleon page boundary falls between complete text lines. No neighboring line or split-word continuation is lost.
5. The smaller gaps and reported divider remnants receive a documented disposition: repaired by a verified local correction, retained as genuine spacing, or retained because erasure would threaten Latin.
6. Existing successful cleanup, complete terminal sentences, global body scale, unit structure, and offline behavior remain valid. No paid vision calls, new CLI options, new dependencies, global threshold retuning, or generalized override framework are introduced.
7. The Google notice page W3 remains. The Praefatio typography transition O8 remains, with a diagnosis recorded for a possible separate layout task. These are the defaults established in the strategy discussion; body cleanup does not silently change them.
8. All local Rust checks pass, all eight inner and imposed PDFs are rebuilt, and a durable verification note records source mappings and before/after prose landmarks. Plans and findings remain on disk as historical records.

# Phase 1 — Map the finite defects and freeze safe corrections (Suggested implementer: Opus)

## 1.1 Prepare shared diagnostic support and the repair inventory

In the test module of `src/facsimile/reflow.rs`, prepare one shared trace helper and three ignored test entry points: `inner_tomi_duplicate_source_trace`, `inner_tomi_timoleon_coriolanus_source_trace`, and `inner_tomi_remaining_source_trace`. The following suffixed steps populate and inspect each group separately. Reuse `operational_range`, `real_source_image`, `render_and_band`, strip extraction, and composition. Do not copy a second bander, renderer, or packer. Establish the helper with a small representative source window in this step; do not trace all fourteen affected units in one session.

Existing `traced_real_unit` and the earlier artifact trace assume contiguous source pages and original row coordinates. Do not reuse those assumptions after duplicate exclusions or compaction. Define test-only `RetainedSourcePageTrace` records containing `source`, `page`, `retained_index: Option<usize>`, original crop edges, original-coordinate bands, and `deleted_rows: Vec<[u32; 2]>`. A skipped page has no retained index. Record deletions in original coordinates; map a surviving source row by subtracting the lengths of deletion intervals ending at or before that row, and return no mapped row for a deleted row. Use one shared lookup/mapping helper for duplication, mask, and compaction regressions. Compare source pixels using original crop/row coordinates and output pixels using their mapped coordinates. Never derive a retained index with `page - start_page` once exclusions exist. Add ordinary `repair_trace_maps_skipped_pages_and_deleted_rows` for the mapping contract. Keep existing unaffected tests and ledgers unchanged; extend their shared helpers only where needed.

Save full-resolution source/output pairs, raw/retained band intervals, crop edges, strip intervals, candidate-mask results, and seam crops under `tmp/pdfs/inner_tomi_repairs/`. Use the current findings' physical PDF pages to locate prose, then match recognizable Latin to source scans. Output canvas indices from a unit diagnostic are not physical PDF page numbers. Trace bounded source windows for pixel diagnosis; use full-unit processing when unit metrics, shared boundaries, or final pagination require it. Cache diagnostic measurements and reuse saved crops instead of rerendering whole units for each finding.

Preserve `src/facsimile/mod.rs::global_scale_for` for this task: it samples raw rendered columns across the ranges and does not apply furniture masks or the proposed duplicate exclusions. Do not automatically align that pass with body cleanup. Record the baseline and final scale and width-sampling ranges, reuse `verify_real_composition`'s existing call to `global_scale_for` for production-scale diagnostics, and check repaired columns for horizontal clipping. Unit-local scale may be recorded for diagnosis but cannot prove the reported pagination repair. Boundary corrections can change the width sample; record any resulting numerical scale change. If a width-sampling defect or material scale change is established, specify its narrow resolution in this plan before changing that algorithm.

Create `src/facsimile/fixtures/inner_tomi_repair_source_trace.json` as the new durable numeric ledger, following the existing fixture's representation where applicable. Every repair record must contain finding IDs, unit ID, zero-based source index, one-based source PDF page, production raster dimensions, exact adjacent prose landmarks, original crop/bands, proposed normalized half-open mask rectangles or row exclusions, and a preservation boundary. Repeated-scan records must identify every occurrence and the source-page sequence to retain. Include measured copied height before/after, ordinary neighboring line gaps, and the proposed final seam spacing. Source pixels establish safety; OCR can help locate a passage but cannot prove preservation.

Cover the complete inventory below. Several IDs may share one source record and correction.

| Finding IDs | Current physical inner-PDF locator | Required diagnosis |
| --- | --- | --- |
| D1 | V/194–201 | Account for the Eumenes ending, premature comparison opening, return to Eumenes, and second comparison opening. Distinguish wrong unit boundaries from repeated/misordered scans. |
| D2 | VII/17–24 | Compare both Agis occurrences line by line, including the start/end seams; select the complete occurrence with the safer image. |
| O8B | II/240–241; user reported 241–242 | Reproduce the sliced line and identify the original source band and selected cut row. |
| G1 | II/219–220 | Account for both sides of the `ci` / `num` interruption and all intervening source lines. |
| G2 | II/235–238 | Identify retained divider/noise regions and prove what precedes `dam sublime se cursu proripuit...`. |
| G3 | VIII/130–132 | Account for `subli-`, all intervening source material, and `partem secutus.` before removing any region. |
| W1 | II/199, reader folio 191 | Locate watermark, `RIOLANUS.` / source folio 275, and the following prose line. |
| W2, G4 | II/227 | Locate watermark and both adjoining prose blocks; establish whether the unwanted space is a source-page tail or an interior gap. |
| O1 | I/371 | Locate `LUS.` / source folio 169 between the cited Camillus lines. |
| O1B | II/187 | Classify the Greek header before chapter XXII and measure its associated gap. |
| O1C | II/204 | Locate `RIOLANUS.` / damaged source folio 277 and right divider. |
| G8B, O1D | II/222 | Locate `EON.` / source folio 285 and the gap after `...cum`. |
| G5 | II/245 | Locate `EON.` / source folio 295, divider fragment, and preceding/following Latin. |
| O2 | III/337 | Locate `NDER.` / source folio 537 after `Haliartii perhibent,`. |
| O3 | VII/167 | Locate header/divider between `...mœrore suo` and `conficiat, sed et levandus...`. |
| O4 | VII/369 | Locate header/divider between `Eo impe-` and `trato, hostes...`. |
| O5 | III/298 | Prove the rule/fleck has no Latin and Marius ends completely on the preceding body page. |
| O6 | VI/329 | Classify `ΩΝ.` and the right rule against the full source page before authorizing erasure. |
| G6 | III/175 | Determine whether the gap within the sentence is copied margin/furniture space. |
| G7, G8 | I/36, I/46 | Distinguish chapter spacing from scan-artifact padding. |
| O7 | Listed examples, including VI/358 | Inventory safe divider rectangles on the listed pages; do not expand this into an exhaustive corpus cleanup. |
| W3, O8 | All tomi/8; V/14–17 | Record intentional retention and explain the source of the Praefatio transition. |

Fill the `Measured repair decisions` subsection during steps 1.1A–1.1C with one disposition per inventory row. For each enabled repair, give exact source IDs/coordinates and the responsible implementation step. Resolve whether Phase 2 needs boundary pins, duplicate-page exclusions, or a unit-specific partial-page trim. Resolve whether Phase 3.2 is needed. If the line split survives safe cleanup, specify the narrowly bounded fallback in Phase 4.1 before implementation. Rewrite affected plan text in place, preserving step numbers. A fixture containing guessed coordinates or unresolved alternatives is not sufficient to proceed with that repair. Tests must read reviewed expectations from the committed ledger; they must not regenerate and accept their own expected results during a regression run.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test repair_trace_maps_skipped_pages_and_deleted_rows`; `cargo test facsimile::reflow`. Validate the fixture structure and the representative trace. Keep diagnostic support production-neutral. For every filtered command in this plan, inspect the summary and require at least one selected test; zero tests is a failed verification. Choose conditional tests according to the recorded implementation branch and record explicit omissions.

**Human verification:** Inspect the representative trace and its source/output coordinate mapping. Confirm that saved records distinguish original source coordinates from retained image coordinates.

## 1.1A Diagnose Eumenes/comparison and Agis duplication

Populate `inner_tomi_duplicate_source_trace` using the shared helper for `eumenes`, `comparatio_sertorii_et_eumenis`, and `agis`. Resolve D1/D2 by comparing the complete repeated passages and their neighboring seams. Record the selected source sequence, all unique prose, required boundary pins, and any whole-page exclusions or partial-page trims. Populate these ledger records and their measured decisions before Phase 2.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test inner_tomi_duplicate_source_trace -- --ignored --nocapture`; `cargo test facsimile::reflow`. Inspect selected-test counts and saved full-resolution evidence.

**Human verification:** Compare both occurrences and their beginning/end seams. Confirm that every distinct Latin line has an identified retained occurrence.

## 1.1B Diagnose Timoleon, Coriolanus watermarks, and the sliced line

Populate `inner_tomi_timoleon_coriolanus_source_trace` for `timoleon` and `marcius_coriolanus`. Resolve G1/G2/G4/G5/G8B, W1/W2, O1B/O1C/O1D, and O8B, including associated O7 divider examples. Establish exact mask rectangles and any residual row exclusions. Test candidate cleanup at production scale and identify the actual split-line cut. Specify any necessary Phase 4 fallback before production changes. Populate these ledger records and measured decisions.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test inner_tomi_timoleon_coriolanus_source_trace -- --ignored --nocapture`; `cargo test facsimile::reflow`. Inspect selected-test counts, neighboring prose bands, copied heights, and the baseline global scale.

**Human verification:** Inspect both watermark seams, complete Timoleon split words, all leaked headers, and the sliced line at full resolution. Confirm the proposed exclusions contain no Latin.

## 1.1C Diagnose Brutus and the remaining furniture/spacing examples

Populate `inner_tomi_remaining_source_trace` for `theseus`, `camillus`, `pyrrhus`, `caius_marius`, `lysander`, `cato_minor`, `demosthenes`, `antonius`, and `brutus`, using bounded source windows first. Resolve G3/G6/G7/G8, O1/O2/O3/O4/O5/O6, and the remaining listed O7 examples. Account for the Brutus continuation before any exclusion. Record W3/O8 retention; use the existing Praefatio source inventory if needed to explain O8, rather than adding a second front-matter tracer. Complete the inventory dispositions and resolve conditional implementation choices.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test inner_tomi_remaining_source_trace -- --ignored --nocapture`; `cargo test facsimile::reflow`. Inspect selected-test counts and verify that every inventory row has a measured decision or a specific preservation reason.

**Human verification:** Review the Brutus word continuation, Marius ending, and every proposed mask near Latin. Distinguish genuine chapter spacing from removable scan space.

### Measured repair decisions

The authoritative reviewed coordinates are in `src/facsimile/fixtures/inner_tomi_repair_source_trace.json`: 13 mask records, 14 rectangles, and two duplicate records.

- D1: comparison source 1 pages 103/104 are excluded; retain 102 → 105 → 106. Eumenes retains its shared ending on 102. No unit-table boundary or pin changes are selected; the conditional boundary/pin tests are omitted.
- D2: Agis source 1 pages 369/370 are excluded; retain the first occurrences 367/368 and their 368 → 371 continuation.
- W1/W2/G4, O1/O1B/O1C/O1D/G8B, G5, O2/O3/O4/O6, O5: apply exactly the confirmed mask rectangles, preserving the existing mask entries.
- W3 and O8: retain the notice page and Praefatio transition.
- Phase 3.2's row-compaction branch is not selected. Full-unit production evidence still copies source 287 through row 2940 and source 299 through 2922: 372/354 rows beyond the reviewed Latin boundary. These regions contain scan-edge ink. Upper merged bands remain on 281, 298, and 308. Separate margin-mask candidates address that ink upstream without changing the confirmed rectangles.
- G1/G2/G3: rebuilt II/219–220, II/235–238, and VIII/130–132 omit the Latin columns on source 0 pages 296/304 and source 1 page 625 because column detection selects narrow scan edges. The measured margin-mask candidates restore those columns and their continuations.
- O8B: the production trace identifies source 306, strip 333–441, canvases 24/25, resized cut row 23, and 520/495 ink pixels on either side. A margin-mask candidate restores separate body bands and the whole line; no general packer fallback is selected.
- G6: source 470's copied tail causes the rebuilt III/175 sentence gap; its margin-mask candidate restores ordinary spacing. G7/G8 match genuine source chapter gaps on pages 23/28 and are recommended for retention.
- O7: comparison and Agis repeated-occurrence dividers are resolved by D1/D2. VI/358 maps to the right edge of source 1 page 361; O6 page 337 remains confirmed without a rule.
- O6's frozen source-337 mask is applied on fresh VI/308–310. The original VI/329 finding is a second occurrence of the same folio-934 passage on source 347; its separate measured header-only mask is recorded as RFI-13B. The source-337 rectangle and no-right-rule decision remain unchanged; no right-edge mask is proposed for 347.
- All sixteen PDFs compiled successfully. The production scale is 1.2023952007293701; raw width inputs and sampling ranges are unchanged. The exact twelve proposed margin masks, separate page-347 header mask, and spacing retentions are recorded in the ledger's `post_rebuild_review` and [the post-rebuild review](2026_10_06_INNER_TOMI_POST_REBUILD_REVIEW.md), **pending human confirmation**. No additional masks or row exclusions are enabled by this recommendation.

# Phase 2 — Restore unique prose in the correct sequence (Suggested implementer: Opus)

## 2.1 Correct the Eumenes/comparison transition

Apply the Phase 1 decision for D1. If the opening is wrong, correct the existing comparison row in `facsimile_units.json` using `start_page`, `title_top_frac`, `body_top_frac`, and/or `skip_top_bands`, and set `pinned: true`. Correct the Eumenes row if its opening itself is wrong. Let `units::ranges` derive the shared ending from the corrected next opening; do not misuse an interior `last_page`. Include a concise provenance explanation in the existing `review_notes` field. Preserve all unrelated rows.

If repeated scans remain inside the correctly bounded range, add a small private `skip_known_duplicate_source_page(unit_id: &str, source: usize, page: u32) -> bool` in `reflow.rs`, with explicit verified matches. Call it in `unit_body_strips` before rendering and pushing either `pages` or `source_pages`. Keep those vectors aligned. This helper is used only when diagnosis establishes whole-page repetition; it must return false for every unrelated identity. Add no JSON schema or CLI configuration. If a page contains distinct prose as well as duplication, Phase 1 must specify an exact unit-specific band trim applied in Pass B using original coordinates; do not discard the whole page.

Add ordinary `known_duplicate_page_exclusions_are_identity_scoped` if page exclusions are introduced. Add `eumenes_comparison_ranges_use_corrected_boundary` only if a boundary correction is selected: use explicit old and corrected fixture rows to demonstrate the diagnosed shared-end/opening regression, rather than testing a fabricated row that merely repeats the general `ranges` contract. Add ignored `eumenes_comparison_prose_is_complete_and_unique`: load the actual committed unit table, compare its selected sequence/boundaries with the independently reviewed ledger, and verify retained shared-page intervals, complete Eumenes ending, one comparison opening, and the correct continuation after `que-`. Use the shared source-page/row mapping to compare retained glyph regions to raw source pixels; output page count alone is insufficient. Keep existing pin-preservation tests passing; add `corrected_comparison_pin_survives_redetection` if the corrected case exercises behavior not already covered.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test eumenes_comparison`; `cargo test known_duplicate_page`; `cargo test facsimile::units`; `cargo test facsimile::detect`; `cargo test eumenes_comparison_prose_is_complete_and_unique -- --ignored --nocapture`; `cargo test facsimile::reflow`. Run conditional filters only when their tests exist, and record the chosen branch.

**Human verification:** Read the final Eumenes paragraph and the complete comparison transition against the source scans. Confirm that the text after the old premature opening is neither lost nor retained twice.

## 2.2 Retain one complete Agis occurrence

Apply the Phase 1 decision for D2. Reuse `skip_known_duplicate_source_page` for verified repeated whole source pages. If the defect comes from source selection rather than repeated source scans, fix the existing selection/boundary mechanism at its cause; do not add a duplicate exclusion for a page that contains unique prose. Use the same Pass B trim mechanism as step 2.1 for a verified partial-page duplicate. Phase 1 must state which occurrence is retained and why its beginning/end join correctly.

Add ignored `agis_repeated_passage_survives_once`, asserting the selected source sequence and one retained occurrence of the complete chapter progression in the old VII/17–24 range. Extend the identity-scoping test from step 2.1 with the Agis exclusions rather than duplicating it. Verify that the preceding and following distinct passages remain intact.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test known_duplicate_page`; `cargo test agis_repeated_passage_survives_once -- --ignored --nocapture`; `cargo test facsimile::reflow`.

**Human verification:** Compare the retained passage and both seams with both original scan occurrences. Confirm chapters and split words proceed continuously and appear once.

# Phase 3 — Remove known furniture and its unwanted space (Suggested implementer: Opus)

## 3.1 Extend verified source masks and reband

In `reflow.rs`, extend `SOURCE_FURNITURE_MASKS` with the exact normalized rectangles established in Phase 1 for W1–W2, confirmed header/folio fragments, and safely separable divider/border/ornament regions. Reuse `apply_source_furniture_mask` and its existing call before detection. Define a rectangle once even when it resolves several findings or serves multiple units. Each entry's comment must describe the source furniture and preservation boundary; do not cite plan phases or narrate changes.

Include pale watermark edges and verified connecting scan noise when necessary. Do not apply a global grayscale threshold or erase the bounding envelope of a component that includes Latin. Preserve crop height and original band coordinates through Pass B. Do not change raw render-cache files, source PDFs, transcription caches, general furniture thresholds, body scale, or the notice page.

Add ignored `inner_tomi_known_masks_preserve_latin`, driven by the shared new fixture. It must verify mask identity, all unmasked source pixels, adjacent Latin regions, retained first/last bands, and absence of the specified furniture. Extend existing mask unit tests for the new identities without copying their fixtures/helpers. Add ordinary `inner_tomi_masking_restores_separate_line_bands` with a shared synthetic divider-connected page that demonstrates the relevant band-merging failure and preservation of every synthetic body pixel.

After masking, run the real extraction/composition path for every severe gap and watermark. Record actual remaining copied blank rows. A disappearing artifact or reduced band count does not establish that spacing is repaired. If existing endpoints eliminate all verified unwanted space, mark step 3.2 unnecessary for those records and add no compaction code for them.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test known_source_furniture`; `cargo test source_furniture_mask`; `cargo test inner_tomi_masking`; `cargo test inner_tomi_known_masks_preserve_latin -- --ignored --nocapture`; run the existing ignored watermark and shared-page cleanup regressions; `cargo test facsimile::reflow`.

**Human verification:** Inspect masks overlaid on raw source images and cleaned seams. Check pale watermark edges, faint Latin, every glyph adjacent to a divider, and unit titles elsewhere on a shared page. Review intentional retained artifacts explicitly.

## 3.2 Compact confirmed residual interior whitespace, if necessary

Implement this step only for records whose post-mask composition still copies a verified unwanted interior interval. Use a small private `SourceWhitespaceExclusion` table in `reflow.rs`, keyed by source/page and containing normalized half-open top/bottom row bounds. Include a unit ID only if the source evidence establishes a unit-specific exclusion. Phase 1 must provide the exact bounds and the number of ordinary gap rows to retain at the seam. Do not infer removable space from a global gap threshold.

Add `compact_known_source_whitespace(unit_id: &str, source: usize, page: u32, col: &mut GrayImage, bands: &mut Vec<Band>) -> Result<()>`. Pass `&range.unit.id` inside the existing source-page/Pass B loop after opening, interior, and shared-tail trims for that page. Give table entries `unit_id: Option<&'static str>`: `None` matches all uses of that source page; `Some(id)` matches that unit alone. Use one matching helper and table for both scopes. Boundary fractions must have finished operating on the full-height image before compaction. This timing preserves the existing `PageBands`, `Strip`, and `compose_pages` interfaces and avoids passing source metadata through the packer.

The helper converts the verified normalized bounds using the original cropped image height; column cropping preserves source height. Validate all intervals against that original coordinate system before applying any changes. Sort the matching intervals, reject overlap, invalid bounds, or intersections with any retained band, and require every pixel in the rows proposed for deletion to equal `Luma([255])` exactly. This intentionally conservative check does not treat gray paper texture as white. If a proposed interval contains texture/noise, enable compaction only after a separately reviewed mask proves that whitening the relevant region removes no Latin. Never substitute `INK_THRESHOLD` or another background threshold: pale Latin must survive. If such a mask cannot be established safely, retain the interval and record the limitation. Masks must have removed furniture first; compaction never erases ink. Return contextual errors for unsafe configured intervals instead of silently clipping or widening them. Then copy the surviving row segments in order into one new column image and subtract the cumulative deleted rows from each retained band's top/bottom. Apply multiple intervals against original coordinates in one pass, not sequentially against already shortened images. Empty-band pages need no compaction: the existing extraction should emit no strips.

Keep source caches pristine. All surviving grayscale pixels must be byte-identical before scaling. Preserve the existing Pass A `LineMetrics` for both Pass B and the returned result. `line_metrics` measures band heights and ink widths, not gaps; deleting rows strictly between bands changes neither measurement. Recomputing from Pass B bands would change the sampled band population and could alter classification/filtering elsewhere in the unit. Require compaction to preserve every retained band's height and ink width, and introduce a metrics change only if a separately diagnosed defect requires it. Do not rebuild an image when no exclusion matched. Use the shared diagnostic row mapping for all source-pixel comparisons after compaction.

Add ordinary `known_whitespace_exclusion_is_identity_scoped` (including unit-specific and page-wide matching), `known_whitespace_compaction_preserves_retained_pixels`, `known_whitespace_compaction_preserves_band_geometry_and_pass_a_metrics`, `known_whitespace_compaction_rejects_body_overlap`, `known_whitespace_compaction_rejects_residual_ink` (including a pixel of value 254 and pale glyph pixels), `known_whitespace_compaction_maps_multiple_intervals`, `known_whitespace_compaction_preserves_unrelated_paragraph_gap`, and `known_whitespace_compaction_runs_after_boundary_trims`. Use one shared synthetic fixture and existing extraction/composition functions. Assert exact image-height reduction, transformed bands, surviving pixels, unchanged band heights/ink widths and returned metrics, and final seam spacing. Add ignored `inner_tomi_watermark_cleanup_removes_pixels_and_space` and `inner_tomi_severe_gaps_preserve_all_lines`, using the measured fixture and production pipeline. These real-source tests remain required even if no compaction helper is needed.

**LLM verification:** If Rust changes are needed: `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test known_whitespace`; `cargo test facsimile::reflow`. In either branch: `cargo test inner_tomi_watermark_cleanup_removes_pixels_and_space -- --ignored --nocapture`; `cargo test inner_tomi_severe_gaps_preserve_all_lines -- --ignored --nocapture`. Verify W1/W2 seams, G1–G3 continuity, and smaller gaps classified as removable. If this step is unnecessary, record measured evidence and omit its unused types/helpers/tests.

**Human verification:** Inspect full prose seams after cleanup, including the split words in G1/G3. Confirm continuous ordinary spacing and unchanged genuine chapter/paragraph gaps. Whitening followed by a large empty interval is a failed repair.

# Phase 4 — Verify whole-line pagination and furniture-only endings (Suggested implementer: Opus)

## 4.1 Resolve the sliced line and unnecessary body images

First exercise the Phase 3 cleanup on O8B, G2, G3, and O5. Add ignored `timoleon_reported_page_break_preserves_whole_lines`, `timoleon_and_brutus_have_no_furniture_only_body_canvases`, and `marius_terminal_furniture_emits_no_body_canvas`. Reuse trace/composition helpers and the new fixture. For O8B, identify the actual original line intervals and prove each appears intact on one composed canvas at the production global scale. Check original source bands as well as composed pixels; resampling alone must not hide a split. For the other cases, account for all Latin and show that the verified rule-only source regions contribute no body strips/canvases.

If source cleanup resolves the sliced line, leave `compose_pages`, `lowest_ink_row`, and `split_tall_strip` unchanged. If it does not, Phase 1 must already have amended this step with the exact verified remaining mechanism and smallest implementation. Prefer a unit/source-page-specific correction restoring real line bands before packing. Do not accept a broad new split heuristic, silently delete part of a tall strip, or shrink the global font scale. Any unavoidable packer behavior change requires an ordinary regression named `reported_merged_strip_wraps_at_complete_line_boundary`, demonstrating the real failure shape and preserving every line; add `unaffected_ordinary_line_packing_is_unchanged` for a representative unaffected fixture. No unresolved fallback may be marked implemented.

The empty-body-page checks concern composed body assets, not legitimate recto separators or signature padding. Do not disable assembly's recto starts or remove sparse final pages that contain genuine concluding lines. Record O7 artifacts not safely separable from Latin and the retained W3/O8 decisions in the ledger.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test facsimile::reflow`; run each new ignored test explicitly with `--ignored --nocapture`. Run any conditional ordinary pagination regression added above. Inspect saved source bands, cuts, and composed canvases at full resolution.

**Human verification:** Check the formerly sliced Timoleon line across the new output boundary. Read the complete Timoleon and Brutus repair seams and Marius-to-Lysander transition. Confirm no prose was removed to eliminate a blank page.

# Phase 5 — Rebuild the readers and record the outcome (Suggested implementer: Sonnet)

## 5.1 Run final checks and verify all eight tomi

Use a workspace-local target directory if the configured external target is inaccessible:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
cargo fmt --all
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
```

Check the exit status of each command; do not let a later command obscure an earlier failure. An ordinary `cargo test` skips ignored tests and is insufficient source verification. Use filtered tests during development and reserve the full suite for final verification. For each command below, require the named test to run and pass; a zero-test result fails verification. The three diagnostic tests inspect/save evidence but must not overwrite reviewed expectations during regression runs.

Run each of these required ignored tests individually with `cargo test <name> -- --ignored --nocapture`:

- `inner_tomi_duplicate_source_trace`
- `inner_tomi_timoleon_coriolanus_source_trace`
- `inner_tomi_remaining_source_trace`
- `eumenes_comparison_prose_is_complete_and_unique`
- `agis_repeated_passage_survives_once`
- `inner_tomi_known_masks_preserve_latin`
- `inner_tomi_watermark_cleanup_removes_pixels_and_space`
- `inner_tomi_severe_gaps_preserve_all_lines`
- `timoleon_reported_page_break_preserves_whole_lines`
- `timoleon_and_brutus_have_no_furniture_only_body_canvases`
- `marius_terminal_furniture_emits_no_body_canvas`
- `reported_divider_pages_preserve_latin`
- `reported_watermark_pages_preserve_adjacent_lines`
- `fabius_shared_page_header_is_excluded_from_both_units`
- `coriolanus_source_284_header_is_excluded`
- `otho_final_source_pages_survive_reflow_once`
- `cimon_lucullus_final_source_page_survives_reflow`

The full ordinary suite includes the shared trace mapping regression. Record the selected branches and confirm that their conditional ordinary regressions exist and ran: `known_duplicate_page_exclusions_are_identity_scoped` for whole-page exclusions; `eumenes_comparison_ranges_use_corrected_boundary` for a boundary correction; `corrected_comparison_pin_survives_redetection` if additional pin coverage was needed; the `known_whitespace` tests for compaction; and `reported_merged_strip_wraps_at_complete_line_boundary` plus `unaffected_ordinary_line_packing_is_unchanged` for a packer change. Omit tests only when their corresponding branch was not selected, with the reason recorded; a filter selecting zero tests is not an omission record.

Use the README's Windows LuaLaTeX setup and the existing offline caches:

```powershell
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
& $env:PLUTARCH_LUALATEX --version
if ($LASTEXITCODE -ne 0) { throw 'LuaLaTeX unavailable' }
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
if ($LASTEXITCODE -ne 0) { throw 'Facsimile build failed' }
```

Do not invoke `--retag`, `--review-lines`, or paid vision. Raw render caches need no deletion because masks run on owned images. Verify fresh compilation of all eight `tomus_N_inner.pdf` and eight imposed PDFs; a build that reports LuaLaTeX was not run is incomplete. If offline front-matter prerequisites are missing, document the specific blocker rather than silently using network operations.

Render affected passages and neighbors with Poppler, locating them by source/prose landmarks instead of old output page numbers. Inspect all findings' dispositions, all repaired unit openings/endings, and a small unchanged sample in each tomus. Inspect generated `.tex` structure and asset references. Check the previously repaired IV/VIII terminal sentences. Review imposed sheets around changed pagination and confirm signatures remain valid; do not require the old page counts to remain unchanged.

Create `llm_artifacts/2026_10_05_INNER_TOMI_MINIMAL_REPAIRS_VERIFICATION.md` containing exact commands/results, source mappings, final output locators, before/after copied heights for watermarks and severe gaps, duplication decisions, preserved line landmarks, retained artifacts/editorial items, and unavailable checks. Update this plan's measured decisions in place without renumbering steps. Preserve all previous plans and findings.

**LLM verification:** All commands and explicit source tests above pass. Run `git diff --check` and `git status --short`; inspect the diff for intended code/fixture/unit-table/document changes. Confirm source PDFs, raw caches, transcription caches, and unrelated unit rows were not edited. Report any environment-dependent check that could not run as unverified.

**Human verification:** Open the rebuilt inner PDFs and compare every repaired passage with the original scans. Inspect watermark seams, complete split words, both deduplication transitions, and the whole-line Timoleon boundary. Check representative imposed sheets and review the verification note before accepting the repairs.
