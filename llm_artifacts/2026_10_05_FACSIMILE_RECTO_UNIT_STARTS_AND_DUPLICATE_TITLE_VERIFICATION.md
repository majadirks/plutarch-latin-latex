# Verification record

Implementation follows the recto and duplicate-title plan on branch `plan/facsimile-body-blank-pages`. Each completed step is committed and pushed independently. Source text is reproduced from its existing pixels; transcription text is unchanged.

## Baseline

Evidence is saved under `tmp/pdfs/recto_units_verification/baseline/`: existing inner TeX and logs, compiled page counts and image listings, decoded PDF images, source and boundary renders, OCR boxes, and SHA-256 manifests. `source_bands_and_widths.json` records production column widths and ordered retained source-band hashes from unchanged filtering. The baseline global 95th-percentile column width is 837 pixels at 300 DPI.

## Completed checks

- Step 1.1: formatting, Clippy with warnings denied, and 23 assembler tests passed; one source compile test remains ignored until the compiled-regression step. Commit `8994687`.
- Step 2.1: the page-434 regression failed with the old midpoint predicate, then all six title-box tests passed with the corrected predicate. Formatting and Clippy passed. Commit `9eef738`.
- Step 2.2: eight range/cache tests and 42 reflow tests passed. The explicitly requested real-source Cato boundary test passed at 300 DPI. Retained crops were visually inspected: Cato includes its full final sentence; the Comparatio starts at its first body line, with neither copied title line. Formatting and Clippy passed. Commit `37dffe1`.

## Lysander diagnostic, step 2B.1A

The ignored regression fails on unchanged filtering with source ink present. The complete trace is `tmp/pdfs/recto_units_verification/lysander_trace_before.json`, and test output is `lysander_before_test.txt` beside it. Source 0/page 551 at 300 DPI has column bounds `[1063,1859)`. The closing band's rows are `[2566,2595)`, its horizontal ink span is 359 pixels, and unit metrics are height median 33, height MAD 2, width median 779. The median inter-band gap is 10 pixels.

Zone and gap filtering retain the closing line. `drop_interior_running_head` loses it through its call to `drop_trailing_furniture`; direct tracing of that trailing function confirms the same loss. This selects the planned trailing-furniture repair, without changing zone/gap filtering or metric sampling. No filtering predicate was changed for the diagnostic step.

## Lysander repair, step 2B.1

The proposed alignment predicate passes the real 300-DPI regression without widening its thresholds. The closing gap is 17 pixels, below the 22-pixel ceiling. `lysander_trace_after.json` records no loss; the retained closing crop was visually checked through `commemoranda habuimus.`. Four metric tests, five line tests, and 45 nonignored reflow tests pass, as do formatting and Clippy. Centered signatures and thin rules remain excluded in synthetic regressions. The exhaustive scaled-composition checks are added in the following shared-verification step.

## Shared source/composition checks, step 2B.2

The raw 300-DPI Latin columns for 434, 551, 637, and 705–708 were inspected in full before freezing `src/facsimile/fixtures/closing_body_bands.json`. The ledger enumerates all substantive body bands, source furniture, and tiny noise intervals separately. The trace requires inclusive render and successful-detection ledgers before indexing pages. Every reviewed body interval must survive once with byte-identical unscaled pixels. Furniture must contribute no packing strip.

The independent stream assertion resizes source cut-point intervals once, places each on white rows with the real horizontal clipping, and compares dimensions and bytes against decoded canvas rows after removing their leading inset. It covers noninteger scaling, ordinary wrapping, tall splitting, and a rejected control signature. Cato and Lysander both passed these real-source assertions at the measured production global scale; neither test silently skipped prerequisites. Synthetic stream checks and the 47 nonignored reflow tests passed, alongside formatting and Clippy.

The plan records two measured refinements: rejected noise can influence surrounding whitespace cut points without contributing text, and Otho's first body bands on pages 706/707 need a narrow geometry-based rescue from the top zone. The shared 8% furniture API remains unchanged. These refinements follow source-column inspection rather than weakening the exhaustive body ledger.

## Terminal endpoints and restored pages

Step 2C.1 (`7ed6d76`) passed the explicit endpoint/range/request and pin-precedence fixtures, all 26 nonignored detector tests, and nine unit-table tests. Real-source detection with an absent temporary cache and no pins passed, ending at 637/708. The operational cache was never redetected by a test. Invalid endpoints fail before saving, and OCR detection is bounded before duplicate terminal scans.

Step 2C.2 (`c35f6bc`) passed the complete page-637 body ledger, first/middle/last OCR-location checks, and exact scaled stream. Its old endpoint 636 fails the same endpoint assertion on an isolated cloned row. The retained page was visually inspected through `fuisse.`, with no detached ornament.

Step 2D.1 passed the ignored real-source Otho test after demonstrating failure against the actual old operational endpoint 704. Every source page in 696–708 renders and detects successfully once; 709/710 are outside the range. Every reviewed body band on 705–708 survives with source pixels intact, and the entire resized row stream matches composition. The concluding crop includes chapter XVIII and the last hyphenated sentence. Pages 706/707 also retain their first body lines through the geometry-based top-zone rescue.

The page-708 ledger initially failed on the library stamp at rows 2080–2117. The plan now specifies removal of a distant narrow furniture cluster after aligned body text. Its synthetic stamp/rule test passes, as do all 50 nonignored reflow tests, formatting, and Clippy. The remedy shares the ink-alignment helper with short closing-line protection; neither page numbers nor Latin strings drive production filtering.

## Compiled recto fixtures, step 3.1

Commit `825558a` passed all four explicitly requested assembler compile regressions with LuaLaTeX and Poppler. The unit fixture places its seven images on physical pages 5, 6, 7, 8, 9, 11, and 12, confirming odd starts and consecutive continuation pages. Repeated recto commands, four front-matter parity cases, and the real Monitum transition also pass. The full suite before the subsequent audit correction passed 301 library tests and five CLI tests, with 13 environment-dependent tests ignored.

## Body-preservation correction during the final audit

Comparison against the baseline caught two false furniture classifications: the two normal-height verse lines on Lysander source 0/page 550 and a normal-height short final line on Tiberius Gracchus source 1/page 414. A detached furniture cluster must contain at least two bands and at least one abnormal-height band. The synthetic `detached_tail_cluster_preserves_spaced_verse_and_short_final_body` regression protects these shapes while the stamp/rule fixture continues to pass.

The corrected production-DPI trace matches all 40 retained baseline bands on Lysander 550 and all 33 on Tiberius Gracchus 414 exactly, including coordinates, dimensions, and pixel hashes. All four explicitly requested real-source reflow regressions pass with the correction, including their complete reviewed ledgers and independent scaled-stream comparisons. Expected assertion panics in the Cimon and Otho tests demonstrate rejection of isolated old endpoint controls; both tests finish successfully.

## Early-line rescue correction during the final audit

The resumed visual audit identified sparse running heads admitted by the early-line rescue: a title fragment and distant source folio can have a full-width bounding span while most intervening horizontal coordinates contain no ink. `metrics::band_ink_coverage` counts coordinates containing ink with the existing source-ink threshold. Early rescue requires coverage of at least half the column, alongside the existing normal-height, full-span, alignment, top-zone, and spacing checks. It has no unit, page-number, or text special cases. `band_ink_coverage_excludes_whitespace_between_head_and_folio` and `early_body_rescue_excludes_wide_sparse_running_head_and_folio` protect this distinction. The four real-source regressions still pass, including Otho's complete first body lines on 706/707.

## All-volume verification, step 4.1

Step 4.1 is complete. The resume revoked the pause, and the fresh compiled build, all-volume placement audit, source/cache comparisons, visual source-band review, decoded-PDF comparison, and independent all-unit source-stream audit all pass. README wording states recto starts, zero or one boundary blank, and matching body-folio parity. Temporary diagnostics are preserved under `tmp/pdfs/recto_units_verification/diagnostic_sources/`; none remain registered in `src/bin`.

### Build and environment

The accepted candidate is a fresh complete offline build with both preservation corrections. `rebuilt/corrected_full_build.log` ends with successful compilation of all eight inner PDFs, all eight imposed PDFs, and `Volumes: 8`. The interrupted earlier build and the completed candidate preceding the sparse-head correction are not accepted output. The normal compile branch removes composed PNGs after embedding them; final pixel verification therefore uses images decoded from the completed inner PDFs.

Commands were run from the repository in PowerShell with workspace-local Cargo output:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
& $env:PLUTARCH_LUALATEX --version
cargo run --release --bin facsimile --target-dir target -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
cargo fmt --all
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
cargo test facsimile::reflow --release -- --ignored --nocapture --test-threads=1
cargo test facsimile::assemble --release -- --ignored --nocapture --test-threads=1
git diff --check
```

LuaLaTeX identifies itself as LuaHBTeX 1.24.0, MiKTeX 25.12, development id 7707. The installed Poppler tools are from `poppler-25.12.0`. LuaLaTeX commands ran with elevated execution. No caches were cleared or redetected, and no vision review or transcription call was made.

Final Rust checks pass: 304 library tests and five CLI tests, with 13 environment-dependent tests ignored by the normal suite. All four explicitly invoked real-source reflow tests pass, and all four explicitly invoked compiled assembler tests pass. The Cimon/Otho endpoint panics printed by the reflow tests are caught negative controls; their test results are successful. Logs are `rebuilt/final_cargo_test.log`, `coverage_real_tests.log`, and `final_compiled_fixtures.log`.

### Compiled pagination and imposition

`audit.ps1` correlates generated TeX references, whitespace-unwrapped LuaLaTeX shipment logs, and `pdfimages -list`. `rebuilt/unit_audit.json` records all 68 expected units and all 2,710 body placements. Every unit opens on an odd physical page and odd body folio; the first body origin is inferred and checked against continuation folios. Every continuation is consecutive. Every inter-unit transition is E+1 after an even ending or E+2 after an odd ending. All 29 inserted boundary versos, including Tomus I's front/body filler, were rendered, contain no text or images, and pass the decoded white-pixel check (every grayscale pixel at least 245). Body TeX contains exactly one generated heading per unit, excluding front-matter headings.

| Tomus | Units | Body images | First body physical page | Last body physical page | Compiled inner pages | Imposed PDF pages |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| I | 11 | 376 | 17 | 398 | 416 | 208 |
| II | 12 | 413 | 9 | 427 | 448 | 224 |
| III | 11 | 396 | 9 | 411 | 416 | 208 |
| IV | 3 | 109 | 9 | 118 | 128 | 64 |
| V | 9 | 358 | 19 | 378 | 384 | 192 |
| VI | 4 | 352 | 9 | 362 | 384 | 192 |
| VII | 11 | 413 | 9 | 423 | 448 | 224 |
| VIII | 7 | 293 | 9 | 303 | 320 | 160 |

All inner counts are divisible by 32 and happen to equal the baseline signature-padded counts. Each fresh wrapper's `\innercount` equals its compiled inner count; each compiled imposed PDF has half that count. Wrapper slots cover every inner page exactly once, with even pages left and odd pages right. Cato's opening is inner page 53 in the right slot of imposed PDF page 28, paired with inner page 44. Its Comparatio opens on inner page 97 in the right slot of imposed PDF page 49, paired with inner page 128. These representative imposed sheets were rendered and inspected. The imposed PDF page number itself is not a book-page parity assertion.

Tomus I retains both combined errata scans on physical page 15, followed by empty verso 16 and body recto 17. Tomus V's prose ends on page 18 and body recto 19 follows directly. The other six tomi end their front matter on page 8 and begin their bodies on page 9. First continuation folios confirm body origin 1 in every volume. `front_transition_t*.json` records these transitions.

| Unit | Baseline opening | Rebuilt opening | Rebuilt opening body folio | Rebuilt final physical page | Body images |
| --- | ---: | ---: | ---: | ---: | ---: |
| Cato Major | 54 | 53 | 45 | 95 | 43 |
| Aristides/Cato Comparatio | 98 | 97 | 89 | 104 | 8 |
| Lysander | 300 | 299 | 291 | 342 | 44 |
| Cimon/Lucullus Comparatio | 114 | 113 | 105 | 118 | 6 |
| Otho | 278 | 277 | 269 | 303 | 27 |

### Source coverage, cache integrity, and visual inspection

All 1,315 baseline source/cache file hashes match exactly, including both source PDFs, transcription/front-matter caches, and existing render-cache entries. `rebuilt/hashes.json` records the recheck. Metadata-cache comparison against `baseline/facsimile_units.json` identifies exactly `comparatio_aristidis_et_catonis`, `comparatio_cimonis_et_luculli`, and `otho`; no fourth row differs. `generated_manifest.json` records hashes and sizes of the fresh generated PDF/TeX/log files.

The corrected trace contains 1,381 unit/source-page contributions and 62,490 retained bands, compared with 1,376 contributions and 62,115 bands in the baseline. Five new source-page contributions are precisely Cimon/Lucullus 637 and Otho 705–708. The ordered source-band comparison identifies 401 additions and 26 removals on 199 changed source pages. All other 62,089 baseline bands preserve coordinates, dimensions, pixel hashes, and order. All 40 bands on Lysander 550 and all 33 on Tiberius Gracchus 414 match exactly.

The latest comparison was used to regenerate `contact_sheets_latest/sheet_01.png` through `sheet_17.png`, with green added-band and red removed-band bounds and JSON labels. All 17 sheets were inspected; stale later sheet files and the original `contact_sheets/` are not evidence for the final code. Enlarged crops additionally confirm the separated top/closing restorations on Alexander 223, Caesar 255, and Demetrius 495. The removals are source furniture, scan-edge/noise bands, and Cato's copied title boundary; no removed interval contains Latin body text. The 401 additions include the complete restored terminal-page ledgers, genuine early body lines, and genuine short body endings. The exhaustive frozen ledgers separately distinguish body from tiny source noise on the terminal pages.

The global 95th-percentile column width changes from 837 to 835 pixels at 300 DPI; default-trim scale changes from 1.1995221376419067 to 1.2023952007293701, approximately +0.2395%. The new inclusive source ranges change the width sample population. Scale was not frozen to match baseline composed hashes. `pdfimages -png` successfully decoded all eight completed inner PDFs; `baseline/pixel_map.json` and `rebuilt/pixel_map.json` correlate their extraction indices to shipment pages and body filenames. The final comparison actually reads all 2,710 new body rasters: 2,687 names overlap baseline and all differ in decoded dimensions or pixels at the changed scale; 23 names are new and one baseline name is absent. The baseline has 2,688 body rasters, so the rebuilt stream has 22 additional trim pages overall. `rebuilt/pixel_comparison.json` records these comparisons; no absent maps or zero-comparison diagnostic was accepted.

The added names are one final canvas apiece for Lycurgus, Pelopidas, Caius Marius, Cimon/Lucullus Comparatio, Nicias, Crassus, Eumenes, Pompeius, Alexander, Iulius Caesar, Cato Minor, Agis, Antonius, Artoxerxes, and Aratus, plus Otho canvases 20–27. Timoleon canvas 52 is absent after repacking into 51 canvases. Its final page 269 was rendered and retains the complete concluding sentence; its source bands preserve every baseline body band and add the reviewed source-309 closing line. Cato still has 43 canvases, but its last canvas contains its genuine final sentence rather than the copied title: equal canvas counts do not establish content equality.

The independent all-unit diagnostic reads the completed-PDF pixel map, requires every source page to render/detect successfully, resizes the source cut-point intervals once at rebuilt scale, applies the real horizontal clipping, and compares complete ordered row-stream dimensions and SHA-256 bytes with the decoded PDF content after its leading inset. It separately reruns production composition and requires each individual decoded canvas to have exactly its expected dimensions and bytes. All 68 units and 2,710 canvases pass, covering 3,640,092 independently resized source rows. `rebuilt/scaled_stream_audit.json` stores each unit's inclusive source range, canvas/row counts, and stream hash; `embedded_scaled_stream_audit.log` records every passing unit. Cimon/Lucullus ends at source 637 and Otho at 708. Together with the exhaustive terminal ledgers and unchanged ordered unscaled bands, these checks account for the changed composed pixels without permitting unexplained source loss, duplication, reordering, or alternative packing. The earlier diagnostic attempt reading removed standalone PNGs failed explicitly and was superseded by this complete embedded-PDF audit.

Completed-PDF renders confirm Cato page 95 contains its genuine final sentence and no copied title. Empty verso 96 precedes the generated heading once above Comparatio body text on page 97. Lysander page 342 contains the complete `commemoranda habuimus.` line. Cimon/Lucullus page 117 contains the source join `contra Græcos` / `tulerunt,`; its conclusion through `fuisse.` appears on page 118 before endpapers. Otho page 303 contains the final hyphenated sentence through `cinæ se conjunxerunt.`; its full retained columns 705–708 were inspected through chapter XVIII. Its operational range excludes duplicate scans 709/710, and the Cimon/Lucullus range excludes 638. Other rendered Comparatio openings—Pelopidas/Marcellus, Sertorius/Eumenes, and Demosthenes/Cicero—retain generated headings above substantive opening text. Exact scanned glyphs, punctuation, capitalization, and line breaks are preserved as pixels; identifying excerpts here are not replacement transcriptions.

Physical printing, folding, and binding have not been performed. Their parity interpretation is checked by the exhaustive wrapper mapping and representative imposed-sheet inspection. Structural flyleaves and final signature blanks remain intentional.
