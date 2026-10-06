# Inner tomi minimal repairs verification

Verification performed on 2026-10-06 on `plan/inner-tomi-minimal-repairs`.

## Handoff review

The selected implementation follows the existing `unit_body_strips`, furniture-mask, banding, and composition paths. No unit-table boundary correction, pin change, whitespace-exclusion framework, threshold change, dependency, or CLI flag was introduced. The initial handoff covered 15 repair records: 13 mask records containing 14 rectangles, plus two duplicate records; its final count of 14 was stale. The user approved thirteen additional masks and both spacing retentions on 2026-10-06. The current ledger has **28 repair records: 26 mask records containing 27 rectangles, plus two duplicate records**. Every approved mask is installed, the four final composition regressions are implemented and pass against current production extraction/composition, and all sixteen PDFs have been rebuilt from current source and verified.

The assumption that the watermark tails and header padding would necessarily disappear after masking proved false. Separate scan-edge ink still merges several retained bands. The confirmed rectangles remain unchanged; a separate reviewed correction is required for that ink.

## Implemented repairs and commits

- `99c5e83`: D1 excludes comparison source 1 pages 103/104 before rendering; D2 excludes Agis source 1 pages 369/370 before rendering. The parallel retained-image/source-page vectors remain aligned. Both exclusions are identity scoped.
- `61fa3ae`: all 13 confirmed furniture-mask records, including the two O3 rectangles, are transcribed exactly from the ledger. The older masks remain intact. Masks run on owned images before column detection.
- `0e4893f`: Marius terminal-extraction regression and production-scale composition evidence. The diagnostic computes seam locations from the actual composed pixel stream and resized strips; it does not duplicate the packer.

D1 retains comparison sequence **102 → 105 → 106**. Eumenes retains its shared ending on 102. The reviewed join is `magno in` / `pretio habitus`. D2 retains **363–368 → 371–379**, including one occurrence each of 367/368 and the reviewed `unius suffragii numero` / `excellerent qui antiquabant legem` join. The real-source tests compare retained glyph pixels to raw source pixels through the `RetainedSourcePageTrace::map_row` contract.

O5 retains Marius's final source-page band **271–691** on source 0 page 530. Production extraction copies no strip extending into the ornament region. The complete ending remains available in `tmp/pdfs/recto_units_verification/crops/marius-repaired-ending.png`.

All twelve additional margin masks and the separate source-347 header mask are installed with
the exact approved pixel bounds. They are appended as separate ledger records; the original
rectangles remain unchanged. The tests group reviewed rectangles by source/page and verify
their union, so a header repair and margin repair on the same page share one preservation contract.
Normalized values are checked with a machine-precision allowance and exact raster-coordinate
equality; this permits harmless floating-point parsing differences without accepting a pixel change.

## Local checks

Build artifacts use workspace-local `CARGO_TARGET_DIR=target`. Cargo commands use `--offline`. Poppler variables point to `C:/Users/majad/Downloads/poppler/poppler-25.12.0/Library/bin/`.

- `cargo fmt --all` and `cargo fmt --all --check`: pass.
- `cargo clippy --offline --all-targets -- -D warnings`: pass.
- `cargo test --offline`: **364 library tests and 9 facsimile-binary tests passed**, 36 library tests ignored. The ignored count rose from 32 to 36 with the four added composition regressions. Log: `tmp/approved-masks-full-tests.log`.
- `cargo test --offline --release --lib inner_tomi_known_masks_preserve_latin -- --ignored --nocapture`: pass with every additional mask installed. Log: `tmp/check-approved-inner-tomi-masks.log`.
- `cargo build --offline --release --bin facsimile`: pass.
- `git diff --check`: pass.

The four required composition regressions are implemented in `src/facsimile/reflow.rs` and
pass against current production extraction/composition:

- `inner_tomi_watermark_cleanup_removes_pixels_and_space`
- `inner_tomi_severe_gaps_preserve_all_lines`
- `timoleon_reported_page_break_preserves_whole_lines`
- `timoleon_and_brutus_have_no_furniture_only_body_canvases`

They read reviewed expectations from the committed ledger (restored crop edges, first/last
retained bands, the recorded sliced-line strip) and compare retained glyph pixels to the raw
source; they do not regenerate their own expected values. Two first-run failures were scope
diagnoses, not repair defects: the sliced-line record lives under `post_rebuild_review`, and
the production crop of source 0 page 298 detects left edge 949 — one pixel right of the ledger's
margin-only-preview 948 — because the installed header-plus-margin union shifts that edge, exactly
the divergence recorded for pages 281/298. The crop assertion fixes the right edge and allows that
documented one-pixel left shift rather than altering the frozen ledger.

The full ignored sweep (`cargo test --offline --release --lib -- --ignored`) ran all four new
regressions and every explicitly selected real-source test and they pass:

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

Log: `tmp/final-ignored-tests.log`. That sweep reported one unrelated failure,
`praefatio_full_build_matches_current_volume_estimate`, which reads a separate offline QA build
output (`build_facsimile/praefatio_hybrid_check/`) that does not exist in this environment. It is
not part of the inner-tomi repair set and is recorded here as an **unavailable check**, not a
regression. The two conditional boundary/pin tests, the whitespace-table tests, and the
packer-change tests are omitted because those implementation branches were not selected.

One initial synthetic divider fixture failed because its ink count equalled the bander's strict threshold. The corrected fixture uses four divider pixels per row and the correct height/width argument order; its final ordinary regression passes. One concurrent debug test command encountered Windows executable locking after its library test passed; the affected dedup tests were rerun successfully using `--release --lib`.

## Source-pixel preservation and copied extents

`inner_tomi_known_masks_preserve_latin` verifies exact raster dimensions, ledger/table rectangle agreement, white pixels inside the masks, and **byte-identical source pixels everywhere outside the masks**, including faint type. Raw and masked evidence and measured bands are under `tmp/pdfs/inner_tomi_repairs/masks/`.

The following table records the **first repair pass**, before the thirteen additional masks.
Its extents are source-coordinate spans after zone filtering, before unit boundary/interior
trims. They are not all blank rows or final canvas heights. The first-pass full-unit bands are
recorded separately in the composition evidence. Cato's increased span reflects rescue of an
early body line after header removal.

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

The first pass copied source 287 through row 2940 and source 299 through row 2922, leaving
372/354 rows beyond the approximate reviewed Latin boundary. With the additional masks,
`inner_tomi_watermark_cleanup_removes_pixels_and_space` confirms the full-unit copied
endpoints are **2575** (marcius_coriolanus source 287, 365 rows removed) and **2573**
(timoleon source 299, 349 rows removed): no strip is copied below the last Latin line, and
the composed seam at the production scale carries no footer padding. The restored Latin crops,
proven by `inner_tomi_severe_gaps_preserve_all_lines`, are **940–1727 on source 296**,
**917–1709 on source 304**, **972–1805 on source 470**, and **980–1767 on source 1 page 625**;
the full-unit retained first/last bands match the ledger, source 470's opening trim yields a
first band at 421 and a last band at 2638 (well above the former copied tail at row 2782), and
every retained band is pixel-identical to the raw source. Zone-band measurements are in
`tmp/pdfs/inner_tomi_repairs/masks/measurements.json`; those are raw per-page banding and differ
from the full-unit expectations where an opening or trailing-furniture trim applies (notably
source 304 and 470). No row-compaction table has been added.

## Production composition and open findings

The measured production body scale is **1.2023952007293701**. The width pass samples raw source pages independently of masks and duplicate exclusions. Its input ranges and algorithm are unchanged, so the same measurement applies before and after these selected repairs. The complete sampling ranges, retained sequences, source bands, strips, and within-strip canvas seams are in `tmp/pdfs/inner_tomi_repairs/post_mask/composition.json`. Independent production-scale canvas PNGs are retained in that directory.

The actual composed pixel streams are checked against concatenated resized source strips for
all thirteen diagnostic units in `inner_tomi_post_mask_composition_evidence`, which re-ran with
the installed masks and reports the production scale above. The source-to-canvas locators use the
measured strip and canvas stream ranges, including skipped-page sequences. Each finding below is
now resolved by the installed masks and verified by a named regression; the first-stage column,
slice, and near-blank-canvas measurements are retained to show the before state.

- **G1:** source 0 page 296 is 1990×2984. First-stage column detection selected **x=1980–1990**, a 10-pixel right scan edge, so the canvases omitted the page's Latin. The installed right-margin mask restores the Latin crop **x=940–1727**; `inner_tomi_severe_gaps_preserve_all_lines` confirms 46 retained lines (265..2584), each pixel-identical to the raw source.
- **G2:** the first stage produced near-blank Timoleon canvases from source 304's divider, whose tall strip mapped to rows 58–1603. The mask restores source 304's Latin crop **x=917–1709** (first band 278, last band 2608, 46 lines); `timoleon_and_brutus_have_no_furniture_only_body_canvases` confirms every composed canvas carries a full scaled line of body ink.
- **O8B:** the first stage split source **306**, strip **333–441**, across Timoleon canvases 24/25 at resized-strip row 23. The mask restores separate line bands; `timoleon_reported_page_break_preserves_whole_lines` confirms the merged interval is now two distinct lines, neither classified as a splittable tall strip, each composing whole within a single canvas at the production scale, with the full stream equal to the concatenated resized strips.
- **G5/G8B:** source 308 and 298 are restored to their Latin crops (**x=949–1734** and **x=948–1741**); the 298 production left edge reads 949 under the installed header-plus-margin union, as recorded for pages 281/298. No furniture-only canvas survives.
- **G3:** first-stage rebuilt VIII/130–132 omitted source 1 page 625's Latin (crop x=1803–1816). The mask restores **x=980–1767**, the `subli-`/`dens` continuation, and all intervening prose before source 626's `partem secutus.`; the severe-gaps regression confirms 47 retained lines (185..2527) and pixel identity, with neighbors 624 and 626 retained.
- **G6:** first-stage rebuilt III/175 retained the source 470→471 sentence gap, source 470's edge connecting its final band through row 2782. The mask restores the Pyrrhus opening trim (first retained band 421) and a last band at 2638, below the former tail; ordinary seam spacing is restored.
- **G7/G8:** rebuilt I/36 and I/46 match genuine chapter spacing on source 23 and 28; retained as approved.
- **O7:** V/195–196 and the reported VII/21 second-occurrence divider are resolved by D1/D2. VI/358 maps to source 1 page 361's far-right rule, now masked. It is distinct from the user-confirmed absent O6 rule on page 337.

The twelve approved margin masks, separate page-347 header occurrence, and source/output evidence are saved in
[the post-rebuild review](2026_10_06_INNER_TOMI_POST_REBUILD_REVIEW.md) and the ledger's
`post_rebuild_review`, and their production expectations are appended in `repairs[]`.
They were approved on 2026-10-06 and are installed. The source-pixel preservation check passes,
the four full composition regressions pass, and the sixteen PDFs have been rebuilt from current
source. The original rectangles are unchanged, and no compaction or general splitter change is
selected.

W3's Google notice and O8's Praefatio typography transition are retained. O6's alleged right rule was confirmed absent and has not been pursued.

## Full PDF rebuild and visual review

The complete offline rebuild from current source ran with
`cargo run --offline --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline`,
with `PLUTARCH_LUALATEX` set to `C:/Users/majad/AppData/Local/Programs/MiKTeX/miktex/bin/x64/lualatex.exe`
and the Poppler tool variables set. It **finished successfully** (exit 0, no `lualatex not run`),
compiling all sixteen PDFs with LuaHBTeX 1.24.0 / MiKTeX 25.12; the log is
`tmp/inner-tomi-rebuild-final.log` and the `build_facsimile/*.pdf` files are freshly dated. The
MiKTeX install and cache access required running the build with the sandbox disabled.
(A first-stage build with the thirteen masks not yet installed is retained for history in
`tmp/inner-tomi-rebuild-compiled.log`; an earlier sandboxed attempt reported `lualatex not run`
and does not count, in `tmp/inner-tomi-rebuild.log`.)

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

Poppler `pdfinfo` verifies these counts on the fresh PDFs; the repairs removed furniture and
copied space without shifting pagination, so the counts match the first-stage build. All inner
counts are multiples of 32. Every generated wrapper's `innercount` matches its compiled inner
PDF (416/448/416/128/384/384/448/320); every inner page `1..innercount` appears exactly once
across the `imposedsheetside` calls, and each signature's recto/verso sequence matches the
folding order. The imposed PDFs faithfully carry the current repaired inner content.

Fresh render inspection confirms the repairs in the final output. The O8B Timoleon page (II/240)
reads as complete justified prose with no sliced line. The G1 restored Timoleon column (II/219–220)
carries full Latin prose and chapter III, previously omitted. Marius ends completely on **III/297**
(`...mortem sibi ipse conscivit.`), and **III/298** is an unnumbered blank recto separator with no
ornament or body image. The source-347 header is removed on **VI/329**, which now opens directly
with `nonnulli etiam amici Pompeii...` under the `CATO MINOR` running head. The D1 join on **V/195**
and D2 join on **VII/20**, the G2/G3/G5/G6 restorations, and the retained G7/G8 chapter spacing on
I/36 and I/46 are all present; no furniture-only divider canvases remain. Renders are saved under
`tmp/pdfs/final_verify/`.

An unchanged page 100 was rendered and inspected in each tomus: Romulus, Fabius Maximus,
Aristides/Cato comparison, Lucullus, Crassus, Alexander, Tiberius Gracchus, and Brutus, respectively.
The page canvases, generated headings, folios, and margins remain consistent.

The source-to-output trace maps the frozen masks to these **physical PDF page spans**.
A span lists all canvases receiving that source page's retained strips, not merely the artifact's
single display location. These spans were measured on the first-stage build and remain valid for
the final PDFs because pagination is unchanged; the spot-rendered repair pages above confirm the
content at the corresponding locators. The locators are also recorded in the ledger.

| Frozen source/page | Finding | PDF span |
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
It is recorded as **RFI-13B, approved and installed on 2026-10-06**, without changing the
source-337 decision or adding a right-edge mask for page 347.

The user closed the mask/spacing decision gate by approving all proposals, and source glyphs
have not been retyped. **The rebuilt PDFs reflect the current generator**: the formerly omitted
columns, sliced line, and page-347 header are repaired. The four composition regressions, the
rerun of the required source tests, the rebuild of all sixteen PDFs from current source, and the
final locator/copied-extent evidence are all complete, so the original plan is finished. No further
mask approval is needed.

The one check that could not run in this environment is the ignored
`praefatio_full_build_matches_current_volume_estimate`, which depends on a separate offline QA
output (`build_facsimile/praefatio_hybrid_check/`) that is absent here; it is unrelated to the
inner-tomi repairs.

Source PDFs, raw raster caches, transcription caches, `facsimile_units.json`, global thresholds, and body scale are unchanged by the implementation.
