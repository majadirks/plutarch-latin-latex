# Context

The facsimile readers reproduce source pixels, but some body images retain the Greek/Latin divider, Google digitization watermarks, or source running heads. This plan addresses those three categories exclusively in the facsimile code path. The user prefers small, cheap, specific fixes over broader parameter tuning. A doubtful divider should remain rather than risk erasing Latin.

Work is prepared on branch `plan/facsimile-artifact-cleanup`. This document is an implementation contract, not an implementation. Preserve all printed Latin glyphs, punctuation, and spelling. Do not touch the transcription pipeline or either transcription cache. Source furniture removal is image processing, not textual correction.

# Current state of the world

- `src/facsimile/columns.rs`: `detect_latin_column` uses a smoothed vertical ink projection and `exclude_left_ticks` to exclude a narrow stripe separated from the body. `INK_THRESHOLD` is 170. `crop_column` preserves the full source height.
- `src/facsimile/lines.rs`: `line_bands` finds rows above an ink floor; `drop_furniture` excludes band centers above 8% and below 95% of image height and removes detached edge bands. `cut_points` extends strips into the half-gaps between retained bands. These are band decisions, not masks on the raster.
- `src/facsimile/reflow.rs`: `BODY_DPI` is 300. `render_and_band` renders, detects the column, crops, bands, drops furniture, and calls `retain_early_aligned_body_bands`. `unit_body_strips` computes metrics in Pass A and applies opening, interior, and shared-tail trims in Pass B. `drop_interior_running_head` removes a leading band when it is narrower than 80% of the column and detached. A running head plus a distant folio can span the measure. The early-body rescue already checks ink coverage as well as span; reuse the geometry helpers rather than duplicating them.
- `src/facsimile/metrics.rs` provides `band_ink_bounds`, `band_ink_span`, `band_ink_coverage`, `classify_band`, and `LineMetrics`. `PageBands` is `(GrayImage, Vec<Band>)`; `Strip` refers to a page image and vertical interval. Composition copies actual grayscale pixels, including pixels too pale to affect detection.
- `src/facsimile/mod.rs` regenerates composed body assets from source rasters in `render_cache`. Successful compiled builds remove composed PNGs. A diagnosis should save its own crops before that cleanup. Cached rasters must stay pristine.
- `facsimile_units.json` stores unit boundaries, not general per-page furniture overrides. Source indices are zero based (`0` means v1); source PDF page numbers are one based.
- Existing ignored real-source tests in `reflow.rs` use `operational_range`, `real_source_image`, and production-DPI cached rasters. Follow this pattern for source-dependent regressions; keep ordinary tests self-contained.

Visual inspection on 2026-10-05 confirmed:

| Existing output locator (physical PDF page) | Observation | Source information |
| --- | --- | --- |
| Tomus I 95 | Divider beside upper portion of Romulus prose | v1 p51, printed p38; lower part of the leaning divider |
| Tomus I 107 | Divider and marginal debris beside opening comparison lines | v1 p56, printed p43; opening divider and Greek-column fragments |
| Tomus I 112 | Divider alongside the full composed body | v1 p59, printed p46; long leaning divider |
| Tomus II 90 | Pale Google watermark above Fabius prose, with a large blank region | v1 p233 bottom margin, before p234 begins `puram redderet.`; copied strip 2662..2953 |
| Tomus II 94 | Pale watermark between body portions; a dark horizontal artifact near the following line | v1 p235 bottom margin, before p236 begins `egregium, me non fefellit`; copied strip 2185..3013 |
| Tomus II 102 | `IO COMPARATIO.` and `227` embedded near the bottom of Fabius prose | v1 p240, printed p227; shared Fabius/comparison page |
| Tomus II 194 (reported); 193 in the inspected file | Confirmed `RIOLANUS.` and `271` embedded at the source seam before `stilis agri ...`; physical p194 continues that source page | v1 p284, printed p271 |

The duplicate mention of page 107 in the request supplies one unique test locator; do not invent a fourth divider page. Output pagination can change after cleanup, so source-page identity and prose context are the durable identifiers. The source header mappings above were checked with the embedded OCR layer and rendered source pages. OCR is diagnostic evidence, never a replacement for Latin pixels.

## Measured source evidence from step 1.1

The ignored `facsimile_artifact_source_trace` test saves page crops, composed canvases, full pipeline JSON, candidate-mask experiments, and 8-connected divider envelopes under `tmp/pdfs/facsimile_artifact_cleanup/`. The committed ledger `src/facsimile/fixtures/artifact_source_trace.json` freezes the relevant records. The pipeline JSON records raw bands, Pass A bands, Pass B bands, source-coordinate strip intervals, and every diagnostic canvas index a strip crosses. Diagnostic composition uses each unit's column widths for scale; its zero-based canvas indices are not physical PDF page numbers. Physical locators above were matched by visible Latin context in the existing inner PDFs, including neighboring Coriolanus pages 193?195. Source indices are zero based; source pages are one based. All pixel rectangles below are half-open full-source coordinates at 300 DPI. Normalize each x by the recorded source width and each y by its height; the ledger stores those ratios too.

| Source page and dimensions | Verified furniture rectangles [left, top, right, bottom) | Preservation boundary |
| --- | --- | --- |
| v1 p233, 1954 ? 3005 | Watermark [1180,2870,1605,2980); bottom scan edge [0,2980,1954,3005); right scan edge [1920,0,1954,3005) | Final Latin band [2589,2624); bottom furniture is below it and the right edge is beyond the entire Latin measure |
| v1 p235, 1969 ? 3013 | Watermark [1180,2880,1605,2985); bottom scan edge [0,2985,1969,3013); right scan edge [1920,0,1969,3013) | Final Latin band after rebanding [2591,2622); the p236 first line [270,301) is on a different pristine source raster |
| v1 p240, 1978 ? 2980 | Running head [550,210,1260,270); folio [1630,210,1730,275) | First Latin band [291,327); comparison display title lower on the page is outside both masks |
| v1 p284, 2002 ? 2988 | Running head [680,190,1150,245); folio [1630,190,1730,245); right scan edge [1970,0,2002,2988) | First Latin band after rebanding [270,302); source right edge is well beyond the Latin measure |
| v1 p56, 2012 ? 2998 | Local opening divider and foreign-column fragments [946,2445,1000,2630) | Opening Latin bands after masking [2470,2518), [2525,2572), [2577,2630); inspect the first glyph of each line at full resolution |

The watermark rectangles include pale antialiased edges. Candidate experiments modify owned images alone, retain the original crop edges to isolate band changes, and save full-resolution before-region crops plus cleaned columns. Reduced previews were inspected alongside full-resolution source details; cache images were not edited. The narrow right-border masks are essential, rather than optional cosmetic additions: watermark masks alone leave connected dark scan noise inside retained bands.

**Watermark and spacing mechanisms.** On p233 the column crop is [1041,1954). Pass B retains a furniture/noise band [2665,2953); its copied strip is [2662,2953). The pale watermark is carried inside that noise band's vertical extent. The nearby [2653,2659) noise band also survives band trimming. On p235 the crop is [1050,1964): dark right-edge noise joins the last body lines and bottom edge into one raw and retained band [2194,3013), whose strip is [2185,3013). The watermark is therefore part of a merged band, rather than merely pale pixels in an otherwise ordinary half-gap. The dark horizontal output artifact is the p235 bottom scan edge; it is safely separable from all Latin on that source page. The next line belongs to p236 and is not inside the erase region.

After the verified masks and rebanding, p233's last body band ends at 2624 and p235's at 2622. Their median positive ordinary body gaps are 14 and 18 source pixels respectively. Removable furniture-only suffixes are [2624,3005) and [2622,3013). Existing `cut_points` ends the final strip at the last band edge, so the resulting source seam has zero copied furniture-padding rows and zero extra source-margin gap. Compared with the original copied endpoints, 329 rows (2953?2624) and 391 rows (3013?2622) disappear. No interior gap-compaction interval is justified by these two occurrences. Do not manufacture one or cut through the merged p235 band before removing the border noise. Step 3.2 verifies this behavior through the existing pipeline.

**Header mechanisms.** P240's raw running-head/folio band [221,256) survives both Pass A and the Fabius Pass B: the truncated head plus distant folio spans enough of the detected measure to defeat the narrow-head heuristic. Fabius retains the page through [1267,1300); the comparison opening starts at [1556,1590) and already excludes the running head by its opening trim. Masking at source-page identity protects both consumers without touching the actual comparison title. On p284, right-edge scan noise creates a band [0,847) that includes the leaked head and early body lines; it survives both passes. Header masks alone leave that merged band. Removing the verified right border too restores the first body band [270,302), beginning with the visible `stilis agri ...` landmark. The leak is on physical Tomus II p193, followed by p194's continuation.

**Divider acceptance decisions.** The first-48-column 8-connected search finds p51 components [1057,1488,1063,1836) and [1060,1838,1082,2639). The latter is 22 pixels wide and fails the 12-pixel gate; advancing the whole crop also threatens upper Latin extending left of the lower divider envelope. P59's component [1021,365,1052,2152) is 31 pixels wide and fails the same gate. Preserve these ambiguous dividers; their bounding rectangles are diagnostic envelopes, not authorized erase rectangles. P59 also merges prose into a retained band [413,2117), confirming that a stripe can alter band geometry. P56's opening component [981,2451,991,2624) is narrow, but the full page has independently meaningful Latin outside that component's vertical interval and farther left. A full-height crop advance is rejected. Use the verified local rectangle in the table instead. Its foreign-column fragments are contamination inside the detected Latin crop, not Latin glyphs. Do not widen the global divider bounds to force these three examples through.

**Verification of step 1.1.** With `CARGO_TARGET_DIR` set to the workspace `target/` because the configured external target directory is inaccessible, formatting and clippy passed; the reflow suite passed (53 tests, 5 ignored); the full ordinary suite passed (345 library tests and 9 CLI tests, 20 library tests ignored); and the explicit ignored source trace passed in both release and debug modes. The final debug trace completed in 665.49 seconds. Output pages I 95/107/112 and II 90/94/102/193/194/195 were rendered with Poppler and matched to saved source crops. Candidate cleanup is diagnostic; production behavior remains unchanged. A human should inspect the saved full-resolution rectangle crops and especially p56's first letters, p235's tail versus p236's first line, and the p284 header seam before enabling masks.

# Desired end state

Easy, isolated dividers are excluded before line banding. Ambiguous, touching, short, or noisy candidates remain. The two reported watermark occurrences and their excess vertical space disappear from copied strips without cutting adjacent prose. Source v1 p240's running head and folio are excluded from every unit using that page; v1 p284's confirmed leaked header and folio are excluded too. Cleanup must remove furniture-only strips and excess padding, rather than turning visible furniture into an equally tall blank area.

The implementation adds no paid vision or OCR dependency, no broad retuning of global furniture thresholds, no new CLI switches, and no generalized override framework. A small table keyed by `(source, source_pdf_page)` is acceptable for verified furniture regions. Preserve column dimensions and type scale when masking pixels. Ordinary tests cover preservation as carefully as removal; ignored real-source tests and compiled output provide the final evidence.

# Phase 1 - Diagnose source pixels and freeze regression evidence (Suggested implementer: Opus)

## 1.1 Trace all reported occurrences through the existing pipeline

Add an ignored test named `facsimile_artifact_source_trace` in `src/facsimile/reflow.rs`. Reuse `operational_range` and `real_source_image`; do not copy their implementations. Trace `romulus`, `comparatio_thesei_et_romuli`, `fabius_maximus`, and `marcius_coriolanus` through `render_and_band`, Pass B, strip extraction, and composition. Save diagnostic rasters and JSON under `tmp/pdfs/facsimile_artifact_cleanup/`, recording source index/page, crop edges, raw and retained bands, strip intervals, and output canvas index. Instrument test code rather than adding a production tracing subsystem.

Identify the exact source pages and source-coordinate furniture rectangles behind the watermark and divider examples by comparing recognizable first/last Latin lines. Inspect production-DPI pixels as well as reduced previews. Specifically check whether pale watermark pixels survive outside dark-pixel bands in half-gap strip padding, whether a divider merges bands, and which stage retains each header. Trace v1 p240 as both the Fabius tail and comparison opening and p284 through Coriolanus. The Coriolanus leak is visually reproduced on physical p193 of the inspected file, immediately before `stilis agri ...`; inspect neighboring pages when matching the user's p194 locator.

For each watermark, record whether it is a retained furniture band, pixels carried in padding, or part of a merged band. Record the furniture-only vertical interval and the exact adjacent body bands. After masking and rebanding, record the resulting gap, the median ordinary body gap, and how many excess rows would still be copied. A count of removed dark pixels is insufficient evidence of a spacing fix.

Record results in this document in place: fill the unresolved source locators, list measured safe rectangles and removable padding intervals, and explain each verified failure mechanism. If a proposed rectangle or removable row interval touches a glyph, reject it. This evidence is required before enabling a page-specific removal. Keep the test passing with diagnostic assertions about page availability and valid coordinates; tests asserting removal belong to subsequent phases.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test facsimile_artifact_source_trace -- --ignored --nocapture`; `cargo test facsimile::reflow`. Render reported output pages with `pdftoppm -f N -l N -png -singlefile` and compare them with saved source crops.

**Human verification:** Review source/output pairs and the proposed erase regions and padding exclusions. Confirm every region contains furniture alone, especially the first line after the watermark on Tomus II p94. Confirm the Coriolanus leak on the page before the p194 continuation.

# Phase 2 - Conservative divider exclusion (Suggested implementer: Sonnet)

## 2.1 Exclude a demonstrably isolated divider before banding

In `src/facsimile/columns.rs`, add `exclude_isolated_left_divider(img: &GrayImage, column: Column) -> Column` and call it at the end of `detect_latin_column`, after the existing tick exclusion. Reuse the current projection helpers where applicable; keep `exclude_left_ticks` for its existing tick-column contract. The new helper examines a narrow region at the detected left edge and advances the crop edge only when all criteria hold:

- The candidate is an 8-connected component of pixels at or below `INK_THRESHOLD`, with height at least `max(150, height / 20)` at production 300 DPI. Connectivity permits a slight lean; a projection peak alone does not establish a divider. Traverse only the first 48 pixels of the detected crop, using a local visited buffer rather than a page-wide component-labeling system.
- Its horizontal envelope is at most 12 pixels wide. Reject components touching the search region's right edge.
- At least 8 wholly white columns (all pixels greater than `INK_THRESHOLD`) separate the candidate's envelope from retained body ink over the candidate's vertical interval. Verify at least three distinct text bands in the proposed retained crop using the existing `line_bands` function. No full-column ink removal or erosion is permitted.
- Advancing the left edge must discard no dark pixels outside the isolated candidate's vertical interval except independently verified non-body marginal debris. Without that verification, leave the crop unchanged. This avoids clipping lower lines that extend farther left than upper lines, as on the opening comparison.

These deliberately strict starting bounds are acceptance gates, not knobs to widen until every example disappears. If Phase 1 shows an example cannot meet them, retain that line and record why. If the existing tick filter already solves a verified case, test that behavior rather than introducing a second solution. For a partial divider with lower prose under its x envelope, use a verified local furniture rectangle in Phase 3 rather than advancing the entire crop edge.

Add tests `isolated_left_divider_is_excluded`, `partial_divider_with_lower_left_body_is_preserved`, `divider_touching_glyph_is_preserved`, `short_vertical_glyph_is_preserved`, `ambiguous_left_noise_is_preserved`, and `clean_column_detection_is_unchanged`. Assert preservation of dark body pixels and valid crop bounds, not merely the disappearance of the stripe. Add ignored `reported_divider_pages_preserve_latin` using Phase 1's source IDs and production renders. Assert removals only for cases accepted as safe.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test facsimile::columns`; `cargo test reported_divider_pages_preserve_latin -- --ignored --nocapture`; `cargo test facsimile::reflow`.

**Human verification:** Compare before/after crops for Tomus I 95, 107, and 112. Inspect every first character and lower line left margin. A retained ambiguous divider is acceptable and must be explained.

# Phase 3 - Specific watermark and header corrections (Suggested implementer: Opus)

## 3.1 Remove verified furniture from the in-memory source raster

In `src/facsimile/reflow.rs`, add a small private `SourceFurnitureMask` definition with `source: usize`, `page: u32`, and normalized full-page rectangle coordinates `left`, `top`, `right`, `bottom`. Define `SOURCE_FURNITURE_MASKS` with only the verified regions recorded in Phase 1. Add `mask_known_source_furniture(source: usize, page: u32, img: &mut GrayImage)` that matches source identity, converts coordinates to clamped half-open pixel bounds, and paints those pixels white (`Luma([255])`). Invalid or empty rectangles must do nothing. Add a descriptive reason comment per entry, without plan references. Use one helper and table for all three furniture categories.

Call this helper on the owned image returned by `render_fn` inside `render_and_band`, before column detection and banding. Never write the cleaned image back to `render_cache`. This removes pale pixels and prevents removed headers from contaminating metrics, early-body rescue, or either side of a shared unit boundary. Include the verified right-border and bottom-edge rectangles: without them the watermark and header bands stay merged. Step 3.2 must prove that rebanding and existing strip endpoints remove the furniture space as well as its pixels. Do not add per-unit overrides for page-level furniture.

Use the smallest safe rectangles for the two watermark source occurrences, including their pale edges. Do not use a global grayscale whitening threshold: legitimate faint type must survive. Mask the header and folio of v1 p240 and p284 with separate rectangles if necessary, protecting their first Latin lines. A safe, narrowly scoped partial-divider rectangle can use this same table. Do not automatically remove the horizontal artifact on Tomus II p94: remove it only if source inspection proves a separable furniture region; document an unresolved overlap rather than erasing prose.

Tests: `known_source_furniture_masks_only_matching_page`, `known_source_furniture_preserves_pixels_outside_rectangle`, `known_source_furniture_removes_pale_watermark_pixels`, `source_furniture_mask_bounds_are_safe`, and `masked_furniture_cannot_return_in_strip_padding`. The final test must use the real strip extraction/composition path on a synthetic page with pale furniture in a copied gap. Add ignored `reported_watermark_pages_preserve_adjacent_lines`, `fabius_shared_page_header_is_excluded_from_both_units`, and `coriolanus_source_284_header_is_excluded`. Use Phase 1's source coordinates to assert furniture absence and neighboring glyph preservation on retained images; do not infer success from OCR or a reduced band count alone.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test known_source_furniture`; `cargo test source_furniture_mask`; `cargo test masked_furniture_cannot_return_in_strip_padding`; run each new real-source test with `cargo test <test_name> -- --ignored --nocapture`; `cargo test facsimile::reflow`.

**Human verification:** Compare raw and cleaned source crops, first/last Latin lines at each seam, and both unit views of p240. Confirm that printed source headings used as unit titles remain intact and p284's first body line survives.

## 3.2 Verify furniture space disappears through existing strip extraction

The measured watermark occurrences are source-page tails, not interior body gaps. Keep the existing `PageBands` tuple, metric signatures, `cut_points`, and composition path. No spacing-metadata migration, compaction table, or Praefatio change is needed for the verified source pages. Full-height source rasters and band coordinates must remain unchanged so opening and shared-tail fractions stay valid.

Extend `reported_watermark_pages_preserve_adjacent_lines` to run the actual `unit_body_strips`, strip extraction, and composition path with the masks enabled. Assert that p233's final retained band ends at source row 2624, p235's at 2622, and no extracted strip reaches their furniture-only suffixes [2624,3005) and [2622,3013). Confirm the following pages' first Latin bands survive (p234 [266,306), p236 [270,301)). Check original neighboring body pixels against the cleaned raster before scaling. The resulting seam must contain zero source-margin padding rows; allow only resampling rounding in composed pixels. Ordinary inter-line gaps on those pages have medians 14 and 18 pixels. Save before/after seam crops. Compare the reduction in actual copied heights, not merely band counts or erased ink totals.

Add `masked_tail_cut_points_exclude_furniture_rows` using a synthetic page with a pale watermark, a bottom-edge rule, and narrow right-border noise that connects the last body line to the footer. Apply the real mask helper, banding, and strip extraction. Assert complete body-band preservation and the exact copied endpoint. Add `watermark_cleanup_removes_pixels_and_space` that composes this fixture at scale 1 and checks furniture absence, every neighboring body pixel, and exact reduced canvas height. Keep unaffected paragraph gaps unchanged, checked by `watermark_cleanup_preserves_unrelated_paragraph_gap`. Use existing helpers and one shared fixture definition rather than duplicating a second bander or packer. Synthetic page identity and rectangles must match a table entry or use the same low-level rectangle application helper as production.

If an additional verified occurrence later proves to be an interior furniture gap, stop before inventing spacing metadata and revise this step with its source identity, safe interval, adjacent body bands, and measured copied-row excess. None is established by the current evidence. A surviving band overlapping a proposed removal interval signals unresolved merged furniture; resolve it with a safe pixel mask and rebanding, never by clipping a body band.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy --all-targets -- -D warnings`; `cargo test masked_tail_cut_points_exclude_furniture_rows`; `cargo test watermark_cleanup`; `cargo test reported_watermark_pages_preserve_adjacent_lines -- --ignored --nocapture`; `cargo test facsimile`.

**Human verification:** Inspect both former watermark seams, especially the first p236 line that followed the horizontal artifact. Confirm large furniture spaces disappear, all Latin glyphs survive, unaffected paragraph spacing is preserved, and normal type scale and unit boundaries remain intact.

# Phase 4 - Verify the rebuilt readers (Suggested implementer: Sonnet)

## 4.1 Rebuild offline and document the outcome

Run `cargo build`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and the full `cargo test`. Run the explicit ignored source regressions added above. Use the README's Windows LuaLaTeX setup, then:

```powershell
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
& $env:PLUTARCH_LUALATEX --version
if ($LASTEXITCODE -ne 0) { throw "LuaLaTeX unavailable" }
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
if ($LASTEXITCODE -ne 0) { throw "Facsimile build failed" }
```

Verify all eight inner PDFs and eight imposed PDFs have fresh timestamps; `lualatex not run` is not successful PDF verification. Keep raw render caches; `--clean-cache` is unnecessary because cleanup runs on every in-memory render. Compare Latin context rather than relying on unchanged PDF page numbers. Inspect the reported locations, surrounding pages, unit openings, and a small selection of unaffected pages in both tomi. Confirm normal type scale and absence of new blank body regions. Inspect emitted `.tex` for valid asset references and unit structure.

Write `llm_artifacts/2026_10_05_FACSIMILE_SOURCE_ARTIFACT_CLEANUP_VERIFICATION.md` with commands/results, durable source mappings, before/after output locations, specific removals, and any intentionally retained ambiguous lines. Preserve this plan and all historical notes. If README documentation is needed, change its existing facsimile description in place; explain conservative cleanup briefly, without implementation detail in reader-facing flows.

**LLM verification:** The commands above must pass. Confirm the diff contains only intended facsimile code/tests and documentation; use `git diff --check` and `git status --short`. Confirm transcription caches and unit boundaries were not changed as a side effect. List any unavailable environment-dependent checks explicitly rather than marking them passed.

**Human verification:** Open the rebuilt inner PDFs and inspect the formerly affected passages against the original scans. Check whole words at erase-region edges, watermark-adjacent prose, and the Fabius/comparison transition. Review the verification note before considering the work finished.
