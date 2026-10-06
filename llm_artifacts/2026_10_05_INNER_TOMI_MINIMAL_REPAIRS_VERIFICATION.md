# Inner tomi minimal repairs verification

Verification performed on 2026-10-06 on `plan/inner-tomi-minimal-repairs`.

## Handoff review

The selected implementation follows the existing `unit_body_strips`, furniture-mask, banding, and composition paths. No unit-table boundary correction, pin change, whitespace-exclusion framework, threshold change, dependency, or CLI flag was introduced. The frozen ledger has **15 repair records: 13 mask records containing 14 rectangles, plus two duplicate records**. The handoff's final count of 14 records is stale. Its initial list of 13 mask records includes O5; O5 is not an additional fourteenth mask record. The plan's stale diagnosis placeholder has been rewritten in place.

The assumption that the watermark tails and header padding would necessarily disappear after masking proved false. Separate scan-edge ink still merges several retained bands. The confirmed rectangles remain unchanged; a separate reviewed correction is required for that ink.

## Implemented repairs and commits

- `99c5e83`: D1 excludes comparison source 1 pages 103/104 before rendering; D2 excludes Agis source 1 pages 369/370 before rendering. The parallel retained-image/source-page vectors remain aligned. Both exclusions are identity scoped.
- `61fa3ae`: all 13 confirmed furniture-mask records, including the two O3 rectangles, are transcribed exactly from the ledger. The older masks remain intact. Masks run on owned images before column detection.
- `0e4893f`: Marius terminal-extraction regression and production-scale composition evidence. The diagnostic computes seam locations from the actual composed pixel stream and resized strips; it does not duplicate the packer.

D1 retains comparison sequence **102 → 105 → 106**. Eumenes retains its shared ending on 102. The reviewed join is `magno in` / `pretio habitus`. D2 retains **363–368 → 371–379**, including one occurrence each of 367/368 and the reviewed `unius suffragii numero` / `excellerent qui antiquabant legem` join. The real-source tests compare retained glyph pixels to raw source pixels through the `RetainedSourcePageTrace::map_row` contract.

O5 retains Marius's final source-page band **271–691** on source 0 page 530. Production extraction copies no strip extending into the ornament region. The complete ending remains available in `tmp/pdfs/recto_units_verification/crops/marius-repaired-ending.png`.

## Local checks

Build artifacts use workspace-local `CARGO_TARGET_DIR=target`. Cargo commands use `--offline`. Poppler variables point to `C:/Users/majad/Downloads/poppler/poppler-25.12.0/Library/bin/`.

- `cargo fmt --all` and `cargo fmt --all --check`: pass.
- `cargo clippy --offline --all-targets -- -D warnings`: pass.
- `cargo test --offline`: pass; **364 library tests and 9 facsimile-binary tests passed**, 32 library tests ignored. The final ordinary-suite log is `tmp/inner-tomi-final-tests.log`.
- `git diff --check`: pass.

The following explicitly selected real-source tests passed with `-- --ignored --nocapture`; logs are `tmp/check-<test-name>.log`:

- `inner_tomi_duplicate_source_trace`
- `inner_tomi_timoleon_coriolanus_source_trace`
- `inner_tomi_remaining_source_trace`
- `eumenes_comparison_prose_is_complete_and_unique`
- `agis_repeated_passage_survives_once`
- `inner_tomi_known_masks_preserve_latin`
- `marius_terminal_furniture_emits_no_body_canvas`
- `reported_divider_pages_preserve_latin`
- `reported_watermark_pages_preserve_adjacent_lines`
- `fabius_shared_page_header_is_excluded_from_both_units`
- `coriolanus_source_284_header_is_excluded`
- `otho_final_source_pages_survive_reflow_once`
- `cimon_lucullus_final_source_page_survives_reflow`
- `inner_tomi_post_mask_composition_evidence`

The two conditional boundary/pin tests are omitted because no boundary or pin correction was selected. Whitespace-table tests and packer-change tests are omitted because those implementation branches were not selected. The Timoleon/Brutus furniture-only and whole-line regressions remain conditional on resolution of their open findings; current output does not meet those claims.

One initial synthetic divider fixture failed because its ink count equalled the bander's strict threshold. The corrected fixture uses four divider pixels per row and the correct height/width argument order; its final ordinary regression passes. One concurrent debug test command encountered Windows executable locking after its library test passed; the affected dedup tests were rerun successfully using `--release --lib`.

## Source-pixel preservation and copied extents

`inner_tomi_known_masks_preserve_latin` verifies exact raster dimensions, ledger/table rectangle agreement, white pixels inside the masks, and **byte-identical source pixels everywhere outside the masks**, including faint type. Raw and masked evidence and measured bands are under `tmp/pdfs/inner_tomi_repairs/masks/`.

The following extents are source-coordinate spans after zone filtering, **before unit boundary/interior trims**. They are not all blank rows or final canvas heights. The final full-unit bands are recorded separately in the production composition evidence. Cato's increased span reflects rescue of an early body line after header removal.

| Source/page | Finding | Before span | After span |
| --- | --- | ---: | ---: |
| 0/287 | W1 watermark | 2680 | 2680 |
| 0/288 | W1 header | 2422 | 2363 |
| 0/182 | O1 | 2399 | 2330 |
| 0/290 | O1C | 2389 | 2324 |
| 0/299 | W2/G4 | 2679 | 2679 |
| 0/281 | O1B | 2537 | 2537 |
| 0/298 | O1D/G8B | 2609 | 2609 |
| 0/308 | G5 | 2590 | 2590 |
| 0/550 | O2 | 2424 | 2353 |
| 1/441 | O3 | 2527 | 2332 |
| 1/539 | O4 | 2493 | 2493 |
| 1/337 | O6 | 2285 | 2332 |
| 0/530 | O5 | 920 | 420 |

Full-unit extraction still copies source 287 through row 2940 and source 299 through row 2922. Relative to the reviewed last-Latin boundary at row 2568, these are **372 and 354 residual tail rows**, respectively. They contain surviving scan-edge ink; treating them as an authorized inkless interior interval would be incorrect. No row-compaction table has been added.

## Production composition and open findings

The measured production body scale is **1.2023952007293701**. The width pass samples raw source pages independently of masks and duplicate exclusions. Its input ranges and algorithm are unchanged, so the same measurement applies before and after these selected repairs. The complete sampling ranges, retained sequences, source bands, strips, and within-strip canvas seams are in `tmp/pdfs/inner_tomi_repairs/post_mask/composition.json`. Independent production-scale canvas PNGs are retained in that directory.

The actual composed pixel streams are checked against concatenated resized source strips for
all thirteen diagnostic units. The source-to-canvas locators use the measured strip and canvas
stream ranges, including skipped-page sequences. No rows are lost at the reported Timoleon
slice; the line is distributed across two canvases.

- **G1:** source 0 page 296 is 1990×2984. Raw column detection selects **x=1980–1990**, a 10-pixel right scan edge, rather than Latin. The generated canvases therefore omit the page's Latin prose. This is a preservation defect, not genuine chapter spacing. Additional source/output evidence and a bounded correction require human review.
- **G2:** near-blank Timoleon canvases 20/21 and the large blank before `dam sublime...` on canvas 22 survive the selected masks. The composition trace maps their tall strip to source 304, rows 58–1603. Source 305 supplies the continuation. Reassessment must account for source 304's Latin as well as the divider.
- **O8B:** Timoleon canvases **24/25** split source **306**, strip **333–441**, at resized-strip row **23** (approximately original row **352.12848**). The preceding and following resized rows contain **520 and 495 ink pixels**. The slice crosses the line beginning `segniter negligenterque agere...`. A general packing or scale change has not been made.
- **G5/G8B:** upper merged bands remain on source 308 (2–1070) and 298 (1–633), despite header removal. Their separate edge/divider components require review.
- **G3:** rebuilt VIII/130–132 omit source 1 page 625's Latin because its detected crop is x=1803–1816. The separate margin-mask candidate restores x=980–1767, the `subli-`/`dens` continuation, and all intervening prose before source 626's `partem secutus.`.
- **G6:** rebuilt III/175 retains the source 470→471 sentence gap. Source 470's edge connects its final band through row 2782. The separate candidate margin mask restores ordinary seam spacing.
- **G7/G8:** rebuilt I/36 and I/46 match genuine chapter spacing on source 23 and 28; retention is recommended for human confirmation.
- **O7:** V/195–196 and the reported VII/21 second-occurrence divider are resolved by D1/D2. VI/358 maps to source 1 page 361's far-right rule. It is distinct from the user-confirmed absent O6 rule on page 337.

The twelve precise candidate margin masks, separate page-347 header occurrence, and source/output evidence are saved in
[the post-rebuild review](2026_10_06_INNER_TOMI_POST_REBUILD_REVIEW.md) and the ledger's
`post_rebuild_review`. They remain **pending human confirmation**, with none enabled in production.
The previews repair the omitted columns and sliced line through the existing mask path;
no row-compaction or general splitter change is proposed. The frozen rectangles are unchanged.

W3's Google notice and O8's Praefatio typography transition are retained. O6's alleged right rule was confirmed absent and has not been pursued.

## Full PDF rebuild and visual review

The initial sandboxed `cargo run --offline --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline` generated all eight tomi but reported `lualatex not run`. That run does **not** count as a completed PDF build. Its log is `tmp/inner-tomi-rebuild.log`.

The installed MiKTeX compiler is available with access to its installation and caches: LuaHBTeX 1.24.0, MiKTeX 25.12. The complete offline rebuild with that access **finished successfully**, compiling all sixteen PDFs. Its log is `tmp/inner-tomi-rebuild-compiled.log`. The executable was `target/release/facsimile.exe`, with the same source paths, output directory, and `--offline` arguments as the Cargo command above, and `PLUTARCH_LUALATEX` set to `C:/Users/majad/AppData/Local/Programs/MiKTeX/miktex/bin/x64/lualatex.exe`.

| Tomus | Compiled inner pages | Compiled imposed sides |
| --- | ---: | ---: |
| I | 416 | 208 |
| II | 448 | 224 |
| III | 416 | 208 |
| IV | 128 | 64 |
| V | 384 | 192 |
| VI | 384 | 192 |
| VII | 448 | 224 |
| VIII | 320 | 160 |

Poppler `pdfinfo` verifies these counts. All inner counts are multiples of 32. Every generated
wrapper's `innercount` matches the compiled PDF; every inner page appears once in its
`imposedsheetside` calls; each signature's recto/verso pair sequence matches the folding order.
Changed-sheet renders were inspected for II/101–102, II/124, II/128, III/153–154,
V/99–100, VII/13–14, and VIII/66–68. The imposed PDFs faithfully carry the current inner
content, including the unresolved slice and divider pages; imposition does not fix those defects.

Fresh render inspection confirms the D1 join on **V/195** and D2 join on **VII/20**.
Marius ends completely on **III/297**; **III/298** is an ordinary unnumbered empty separator
with no source ornament or body image. Reassessment locators are II/219–220 (G1), II/235–238
(G2), II/240–241 (O8B), III/175 (G6), I/36 and I/46 (G7/G8), VI/358 (O7), and
VIII/130–132 (G3). Those defects persist in this first-stage output except the genuine-spacing
retentions recommended for review and the D1/D2-related divider removals.

An unchanged page 100 was rendered and inspected in each tomus: Romulus, Fabius Maximus,
Aristides/Cato comparison, Lucullus, Crassus, Alexander, Tiberius Gracchus, and Brutus, respectively.
The page canvases, generated headings, folios, and margins remain consistent.

The final source-to-output trace maps the frozen masks to these **physical PDF page spans**.
A span lists all canvases receiving that source page's retained strips, not merely the artifact's
single display location. The measured locators are also recorded in the ledger.

| Frozen source/page | Finding | Fresh PDF span |
| --- | --- | --- |
| 0/287 | W1 | II/197–199 |
| 0/288 | W1 | II/199–201 |
| 0/182 | O1 | I/371–373 |
| 0/290 | O1C | II/204–206 |
| 0/299 | W2/G4 | II/225–227 |
| 0/281 | O1B | II/187–189 |
| 0/298 | O1D/G8B | II/222–225 |
| 0/308 | G5 | II/245–247 |
| 0/550 | O2 | III/337–340 |
| 1/441 | O3 | VII/163–165 |
| 1/539 | O4 | VII/365–367 |
| 1/337 | O6 | VI/308–310 |
| 0/530 | O5 | III/297 |

The locator verification exposed a separate O6 occurrence: **VI/329 is source 1 page 347**,
which repeats the same folio-934 passage found on page 337. The frozen source-337 mask is
correctly implemented, but it does not cover page 347's Greek header. The independently
measured header-only candidate is `[140,145,1720,205)` on a 1787×2866 raster: header ink
154–193, first body ink 219, 14 pixels clear below the mask. Outside pixels are unchanged.
It is recorded as **RFI-13B, pending human confirmation**, without changing the source-337
decision or proposing a right-edge mask for page 347.

The handoff's final human-judgment gate remains open. Source glyphs have not been retyped,
and no pending margin candidate was silently enabled. The current PDFs are provisional because
G1/G2/G3 still omit prose, O8B still splits a line, and the additional page-347 header remains;
the review document supplies the concrete
corrections for confirmation and a second verified rebuild.

Source PDFs, raw raster caches, transcription caches, `facsimile_units.json`, global thresholds, and body scale are unchanged by the implementation.
