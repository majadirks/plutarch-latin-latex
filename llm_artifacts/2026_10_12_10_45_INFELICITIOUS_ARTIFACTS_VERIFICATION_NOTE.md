# Infelicitious artifacts: final verification, 2026-10-12

## Build and source identity

The full offline facsimile build completed on 2026-10-12 from production code commit `910a158481e79a44a88ff6df69fbd76d7d14132e`. It generated all eight inner and eight imposed PDFs in `build_facsimile/`; each PDF opens and has the page count recorded in `final_build`. LuaHBTeX 1.24.0 (MiKTeX 25.12) compiled the output. The source PDF and unit-table SHA-256 values match the frozen baseline; no source PDF or transcription cache changed.

```powershell
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
cargo run --release --bin facsimile --target-dir target -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
```

The JSON fixture `src/facsimile/fixtures/backlog_artifact_source_trace.json` retains the original baseline fields unchanged and stores final page PNG paths, PNG and rendered-pixel hashes, source-to-canvas indices, full volume manifests, and PDF hashes separately. The baseline PDF, TeX, body-image, and reader-crop evidence remains under `tmp/pdfs/backlog_artifacts/baseline/`; final rendered pages are under `tmp/pdfs/backlog_artifacts/final/`. These large generated files are local evidence and can be recreated with the commands below.

## Nineteen source-backed outcomes

| ID | Source page(s) | Frozen reader locator | Final reader locator | Installed correction and measured check |
| --- | --- | --- | --- | --- |
| I1 | v1 p17 | T1 p23 / f.7 | T1 p23 / f.7 | Source furniture mask `[1043,270,1053,2655]`; canvas 54 to 432 px; criterion `{'min_canvas_height': 400}` passed; reviewed Latin rows retained once. |
| I2 | v1 p59 | T1 p113 / f.97 | T1 p113 / f.97 | Source furniture mask `[1018,410,1028,2120]`; canvas 779 to 1304 px; criterion `{'min_canvas_height': 1200}` passed; reviewed Latin rows retained once. |
| I3 | v1 p105 | T1 p212 / f.196 | T1 p213 / f.197 | Whole-strip guard `[2188, 2613]` at 511 scaled px; complete source line owned by one canvas. |
| I4 | v1 p166 | T1 p340 / f.324 | T1 p340 / f.324 | Source furniture mask `[900,210,1750,280]`; no retained source band before row 294; first and second Latin rows retain original pixels; reviewed Latin rows retained once. |
| II1 | v1 p200 | T2 p20 / f.12 | T2 p21 / f.13 | Whole-strip guard `[2542, 2620]` at 94 scaled px; complete source line owned by one canvas. |
| II2 | v1 p229 | T2 p81 / f.73 | T2 p81 / f.73 | Source furniture mask `[1932,0,1947,2990]`; internal blank 209 to 25 px; criterion `{'max_internal_blank_rows': 50}` passed; reviewed Latin rows retained once. |
| II3 | v1 p257 | T2 p139 / f.131 | T2 p138 / f.130 | Source furniture mask `[1995,0,2010,2976]`; internal blank 208 to 18 px; criterion `{'max_internal_blank_rows': 50}` passed; reviewed Latin rows retained once. |
| II4 | v1 p276 | T2 p177 / f.169 | T2 p177 / f.169 | Source furniture mask `[1971,0,1986,2986]`; canvas 541 to 1359 px; criterion `{'min_canvas_height': 1300}` passed; reviewed Latin rows retained once. |
| II5 | v1 p292 | T2 p209 / unnumbered | T2 p209 / unnumbered | Source furniture mask `[1979,0,1994,2994]`; canvas 578 to 1247 px; criterion `{'min_canvas_height': 1200}` passed; reviewed Latin rows retained once. |
| II6 | v1 p297, 298 | T2 p223 / f.215 | T2 p221 / f.213 | Source furniture mask `[1885,0,1900,2964]`; internal blank 204 to 31 px; criterion `{'max_internal_blank_rows': 50}` passed; reviewed Latin rows retained once. |
| II7 | v1 p310 | T2 p248 / f.240 | T2 p246 / f.238 | Source furniture mask `[1975,0,1990,2992]`; canvas 491 to 1355 px; criterion `{'min_canvas_height': 1300}` passed; reviewed Latin rows retained once. |
| II8 | v1 p329, 330 | T2 p290 / f.282 | T2 p288 / f.280 | Source furniture mask `[1864,0,1879,2957]`; internal blank 214 to 20 px; criterion `{'max_internal_blank_rows': 50}` passed; reviewed Latin rows retained once. |
| III1 | v1 p519 | T3 p273 / f.265 | T3 p274 / f.266 | Whole-strip guard `[1633, 1727]` at 113 scaled px; complete source line owned by one canvas. |
| III2 | v1 p520 | T3 p274 / f.266 | T3 p275 / f.267 | Whole-strip guard `[467, 563]` at 115 scaled px; complete source line owned by one canvas. |
| V1 | v2 p105 | T5 p197 / f.179 | T5 p196 / f.178 | Guarded 1004 x 44 px ending stacked to 1412 px; final line once and following recto preserved. |
| VII1 | v2 p446 | T7 p45 / f.37 | T7 p44 / f.36 | Guarded 1004 x 44 px ending stacked to 1420 px; final line once and following recto preserved. |
| VII2 | v2 p566 | T7 p287 / unnumbered | T7 p286 / f.278 | Whole-strip guard `[1411, 1586]` at 210 scaled px; complete source line owned by one canvas. |
| VII3 | v2 p568 | T7 p291 / f.283 | T7 p290 / f.282 | Whole-strip guard `[1254, 1573]` at 384 scaled px; complete source line owned by one canvas. |
| VII4 | v2 p593 | T7 p344 / f.336 | T7 p342 / f.334 | Whole-strip guard `[2002, 2132]` at 156 scaled px; complete source line owned by one canvas. |

All nineteen fixture records have `status: repaired`. For source masks, `inner_tomi_known_masks_preserve_latin` checks exact changed-pixel coverage and protected Latin regions. The dedicated complete-unit regressions check row ownership and output bounds. The reader page numbers above come from current source-to-strip-to-canvas traces joined to the final TeX/PDF manifest; the printed folios were extracted from the final PDFs. The comparison opening II5 intentionally has no printed folio.

## Complete volume comparison

| Tomus | Unit order | Baseline / final inner pages | Baseline / final imposed pages | Baseline / final trailing signature blanks | Changed unit body canvases |
| --- | --- | ---: | ---: | ---: | --- |
| 1 | unchanged | 416 / 416 | 208 / 208 | 12 / 12 | none |
| 2 | unchanged | 448 / 448 | 224 / 224 | 21 / 23 | `marcius_coriolanus` 48 to 47; `comparatio_alcibiadis_et_coriolani` 7 to 6; `timoleon` 50 to 49; `aemilius_paulus` 52 to 51 |
| 3 | unchanged | 416 / 416 | 208 / 208 | 3 / 3 | none |
| 4 | unchanged | 128 / 128 | 64 / 64 | 6 / 6 | none |
| 5 | unchanged | 384 / 384 | 192 / 192 | 14 / 16 | `comparatio_sertorii_et_eumenis` 3 to 2 |
| 6 | unchanged | 480 / 480 | 240 / 240 | 20 / 20 | none |
| 7 | unchanged | 448 / 448 | 224 / 224 | 29 / 29 | `demosthenes` 37 to 36; `dio` 62 to 63 |
| 8 | unchanged | 192 / 192 | 96 / 96 | 24 / 24 | none |

Every unit remains in the same tomus and order. Volume II loses four body canvases across Coriolanus, its comparison, Timoleon, and Aemilius Paulus. The V comparison loses one and Agesilaus opens on physical page 197 rather than 199. Demosthenes loses one and Cicero opens on physical page 45 rather than 47; Dio gains one complete canvas from its protected edge-line repair. Both following openings are rectos. Signature padding accounts for unchanged total page counts despite those local shifts.

## Executed verification

- `cargo fmt --all` and `cargo fmt --all --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test`: 424 library tests and 9 binary tests passed; 0 failed. The 92 source/compile tests marked ignored were selected separately where required.
- `cargo build --release --bin facsimile`: passed.
- Full offline `cargo run --release --bin facsimile ... --offline`: passed; 8 inner and 8 imposed PDFs compiled and opened.
- `backlog_furniture_masks_source_trace`, `backlog_gap_masks_source_trace`, and `backlog_whole_strip_final_source_trace`: each executed 1 test and passed, preserving distinct current traces.

Each final ignored regression below executed exactly one test and passed (0 failed):

| Regression test | Executed / passed |
| --- | ---: |
| `backlog_lycurgus_pericles_lines_compose_whole` | 1 / 1 |
| `backlog_marius_two_seams_compose_whole` | 1 / 1 |
| `backlog_dio_edge_lines_compose_whole` | 1 / 1 |
| `backlog_shared_source_neighbors_preserve_latin` | 1 / 1 |
| `backlog_theseus_orphan_line_rejoins_body` | 1 / 1 |
| `backlog_themistocles_source_header_is_excluded` | 1 / 1 |
| `backlog_reported_gaps_preserve_continuous_latin` | 1 / 1 |
| `backlog_eumenes_comparison_short_ending_merges` | 1 / 1 |
| `backlog_demosthenes_short_ending_merges` | 1 / 1 |
| `backlog_short_ending_merge_rejects_changed_source_identity` | 1 / 1 |
| `backlog_short_endings_keep_following_rectos` | 1 / 1 |
| `facsimile_units_compile_on_rectos_with_single_boundary_blanks` | 1 / 1 |
| `eumenes_comparison_prose_is_complete_and_unique` | 1 / 1 |
| `agesilaus_second_scan_survives_once` | 1 / 1 |
| `cicero_second_scan_survives_once` | 1 / 1 |
| `timoleon_reported_page_break_preserves_whole_lines` | 1 / 1 |
| `inner_tomi_known_masks_preserve_latin` | 1 / 1 |

The ordinary suite also covers `backlog_artifact_fixture_has_expected_shape`, the exact mask ledger, the scoped short-ending merge guards, and the previously failing Antonius checks. The source-pixel and whole-line tests above provide the production-DPI checks for the changed behavior.

## Evidence reproduction and visual review

```powershell
python tools/collect_backlog_baseline.py
python tools/render_backlog_baseline_pages.py
python tools/copy_backlog_baseline_body_images.py
cargo test --release backlog_furniture_masks_source_trace -- --ignored --nocapture
cargo test --release backlog_gap_masks_source_trace -- --ignored --nocapture
cargo test --release backlog_whole_strip_final_source_trace -- --ignored --nocapture
python tools/collect_backlog_final.py
python tools/map_backlog_final_locators.py
python tools/measure_backlog_latin_row_bounds.py
```

I inspected rendered final pages I/213, II/177, V/196, and VII/44 at page resolution. The joined `pa-` / `tricios` phrase is on II/177; the two short final lines remain legible inside the prior body pages. A human reviewer should compare all nineteen final PNGs with the corresponding preserved baseline crops and original source-page rasters at full resolution, especially I/212-213, II/20-21, III/273-275, and VII/287, 291, 344. The fixture supplies the exact paths and hashes for that review. No item remains marked `unresolved` or `source_spacing_retained`.
