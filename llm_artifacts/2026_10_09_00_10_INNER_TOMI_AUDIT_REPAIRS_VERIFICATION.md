# Inner-tomi audit repairs verification

Created 2026-10-09T00:10. Branch: `plan/inner-tomi-audit-repairs`.

Implementation steps 2.1 through 5.1 are complete. All initial Tier-A omissions pass: **39 events / 69 lines restored, zero unresolved Tier-A events, zero promotions**. The separate Cato Minor verse label is restored as one additional source item; C6's two clipped words are preserved without changing the frozen omission counts. The **20 Tier-B events / 44 lines remain unpromoted and pending the confirming human read**. Their detector confidence and historical confirmed verdicts are unchanged.

All eight inner readers and eight imposed PDFs compiled fresh offline. The source PDFs, raw render caches, transcription caches, and `facsimile_units.json` were not edited. Masks act on owned rasters; source-body protections copy original rows. No Latin was retyped, corrected, normalized, or reconstructed.

## Checks and commands

- `cargo fmt --all` and `cargo fmt --all --check`: passed. Included test sections were formatted with `rustfmt --edition 2021` as well.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test`: passed, 380 library tests and 9 CLI tests; 54 ignored tests require deliberate invocation.
- `cargo build`: passed.
- The final source-PDF sweep below passed **30 tests, zero failures**, in 429.88 seconds. Every filter selected at least one test. New ignored tests also passed their individual per-step invocations. The endpoint regressions deliberately catch old-endpoint assertion failures as negative controls; those printed panic messages are expected and both tests pass.
- `cargo test audit_tier_a_completion_requires_all_lines`: passed; unresolved status, a missing reviewed line, and missing passing evidence each reject completion.
- `cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline`: passed; LuaLaTeX compiled all readers and imposed wrappers. No `--retag`, `--review-lines`, paid vision, or OCR regeneration was used.
- `python tmp/pdfs/inner_tomi_audit_repairs/verify_readers.py`: passed; exact verified grayscale canvas SHA-256 values match the images actually embedded in the rebuilt PDFs. All 39 event locators, 69 line regions and 68 terminal endpoints were resolved.

Tools: Rust `1.96.1 (31fca3adb 2026-06-26)`, Poppler `25.12.0`, LuaHBTeX `1.24.0 (MiKTeX 25.12)`. Windows LuaLaTeX was selected with `PLUTARCH_LUALATEX=$env:LOCALAPPDATA/Programs/MiKTeX/miktex/bin/x64/lualatex.exe`.

## Final source-PDF regression sweep

The command was:

```powershell
cargo test --release --lib -- --ignored --nocapture --test-threads=1 second_scan_survives_once repeated_passage_survives_once repeated_folios_survive_once eumenes_comparison_prose_is_complete_and_unique otho_final_source_pages_survive_reflow_once cimon_lucullus_final_source_page_survives_reflow romulus_c1_c6_lines_restored coriolanus_c2_lines_restored phocion_c3_line_restored brutus_c5_line_restored audit_tier_a_source_lines_restored comparison_s2_line_is_whole inner_tomi_known_masks_preserve_latin inner_tomi_watermark_cleanup_removes_pixels_and_space reported_watermark_pages_preserve_adjacent_lines reported_divider_pages_preserve_latin inner_tomi_severe_gaps_preserve_all_lines timoleon_reported_page_break_preserves_whole_lines timoleon_and_brutus_have_no_furniture_only_body_canvases marius_terminal_furniture_emits_no_body_canvas fabius_shared_page_header_is_excluded coriolanus_source_284_header_is_excluded lysander_closing_line_survives_reflow cato_comparatio_source_boundary_excludes_duplicate_title
```

Exact selected names (each under `facsimile::reflow::tests`):

- `agesilaus_second_scan_survives_once`
- `agis_repeated_passage_survives_once`
- `alexander_second_scan_survives_once`
- `antonius_second_scan_survives_once`
- `aratus_second_scan_survives_once`
- `audit_tier_a_source_lines_restored`
- `brutus_c5_line_restored`
- `cato_comparatio_source_boundary_excludes_duplicate_title`
- `cato_minor_repeated_folios_survive_once`
- `cicero_second_scan_survives_once`
- `cimon_lucullus_final_source_page_survives_reflow`
- `comparison_s2_line_is_whole`
- `coriolanus_c2_lines_restored`
- `coriolanus_source_284_header_is_excluded`
- `eumenes_comparison_prose_is_complete_and_unique`
- `fabius_shared_page_header_is_excluded_from_both_units`
- `inner_tomi_known_masks_preserve_latin`
- `inner_tomi_severe_gaps_preserve_all_lines`
- `inner_tomi_watermark_cleanup_removes_pixels_and_space`
- `lysander_closing_line_survives_reflow`
- `marius_terminal_furniture_emits_no_body_canvas`
- `otho_final_source_pages_survive_reflow_once`
- `otho_second_scan_survives_once`
- `phocion_c3_line_restored`
- `phocion_second_scan_survives_once`
- `reported_divider_pages_preserve_latin`
- `reported_watermark_pages_preserve_adjacent_lines`
- `romulus_c1_c6_lines_restored`
- `timoleon_and_brutus_have_no_furniture_only_body_canvases`
- `timoleon_reported_page_break_preserves_whole_lines`

## Fresh PDFs and signatures

Every inner count is divisible by 32. Each imposed PDF has half the inner page count, giving 16 imposed sides per 32-page signature. First-signature and central imposed-sheet samples were visually inspected for every tomus. Physical page numbers below are one-based PDF pages; they are not source printed folios or reader running folios.

| Tomus | Previous inner pages | Fresh inner pages | Fresh imposed sides | Inner PDF SHA-256 |
| --- | ---: | ---: | ---: | --- |
| 1 | 416 | 416 | 208 | `15ea94e83095356f7aa050d0399bf3ea2743937f938a05b5bf94828c113b9a85` |
| 2 | 448 | 448 | 224 | `f337078a20b6c07a86f72f509200c34ca3b5d5c05b8ff97e0e0a2214495afb7a` |
| 3 | 416 | 416 | 208 | `1e19ed70651c8dfd8cc3bfe0569230a550bcba5379be9068db67be891327afd5` |
| 4 | 128 | 128 | 64 | `ca563a7c1f2f3cf7c77ac91437c7fc957637065630ec57195fc167fc3dc47b2b` |
| 5 | 384 | 384 | 192 | `0217a56ddd61a58cc12c70ecde5d63ad6976c7767fbf1e7616e888727b52ef7a` |
| 6 | 352 | 480 | 240 | `b8fc69c2c3cac03d69b90e1d972cb5cebd43b070eb342dd035d4cc00f627e15d` |
| 7 | 448 | 416 | 208 | `da04aa9a822a6776eb856cec09889a7e01577ecabaa2782ba9fb4b3c4b5a5359` |
| 8 | 320 | 192 | 96 | `cfc874b4b437e3ac88a315c479b1c67be3bf81528d665e6dc4ef0b4edeac4885` |

Total: 2880 physical inner pages and 2639 body-image pages. All sixteen output hashes, timestamps, dimensions, body-image mappings and source-region ownership are in `archive/inner_tomi_audit_repairs_2026_10_08/final/verification.json`.

## Original source identity and global scale

Production body scale: **1.2023952007293701**, unchanged. `global_scale_for` samples unmasked source rasters across complete ranges, including excluded repeated pages. Furniture masks, local crop safeguards and exclusions do not change that sampling pass. Praefatio uses its existing independent front-matter scale.

- `Ploutarchou_Bioi_v1.pdf`: `e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15`.
- `Ploutarchou_Bioi_v2.pdf`: `fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684`.

## Repeated scans

Each second occurrence is excluded within its exact owning unit and source 1. The first occurrence and reviewed join landmarks pass unique unscaled extraction and returned-canvas preservation. Agis and Cato Minor each have two disjoint skip blocks, tested through one union per unit. D1-D3 records and predicates remain intact. No strip derives from excluded pages.

| Finding | Unit | Retained first source pages | Excluded second source pages | Retained identity seam |
| --- | --- | --- | --- | --- |
| D4 | `agesilaus` | [115, 116, 117, 118] | [119, 120, 121, 122] | [118, 123] |
| D5 | `alexander` | [211, 212] | [213, 214] | [212, 215] |
| D6 | `phocion` | [295, 296] | [297, 298] | [296, 299] |
| D7 | `cato_minor` | [359, 360] | [361, 362] | [360, 363] |
| D8 | `agis` | [375, 376] | [377, 378] | [376, 379] |
| D9 | `cicero` | [463, 464] | [465, 466] | [464, 467] |
| D10 | `antonius` | [521, 522] | [523, 524] | [522, 525] |
| D11 | `aratus` | [671, 672] | [673, 674] | [672, 675] |
| D12 | `otho` | [699, 700] | [701, 702] | [700, 703] |

D5 resolves C4/S1 on source 213; D12 resolves C7/R12 on source 701. D6 leaves source 299 interior to Phocion; its separately restored first line is verified. D7's source-360 `Et:` label passes the exact original-glyph fingerprint exemption and complete duplicate-regression coverage. Otho closing pages 705-708 are indexed through retained source identities after the 701/702 exclusion.

## Watermark pixels and copied space

| Finding | Source | Copied page height before | Final copied page height | Copied furniture-tail rows before | After |
| --- | --- | ---: | ---: | ---: | ---: |
| W4 | v1/321 | 2630 | 2321 | 309 | 0 |
| W5 | v2/603 | 2512 | 2351 | 161 | 0 |
| W6 | v2/623 | 2321 | 2299 | 128 | 0 |

These totals sum actual extracted intervals. Endpoint subtraction would overstate W5/W6 tail removal. W4's furniture-only candidate copied 2314 rows; the reviewed fragment join restores the seven-row interval `[2526,2533]`, yielding 2321. W6's furniture-only candidate copied 2298; the separately protected first glyph row `[221,222]` yields 2299. Both added intervals belong to preserved source body, and every copied furniture tail remains zero. W6's source-624 first line and the three top lines on 623 pass separately in `brutus_c5_line_restored`.

## Restored omissions and current reader locators

Each row below is one frozen event. Its exact independently reviewed full-source pixel regions, source overlay and original confidence/verdict remain in the diagnostic fixture. Native source-line/compiled-reader pairs for all 69 lines were rendered and visually compared on six contact sheets. Coordinates and crop images are prose landmarks that can be checked against the printed edition without introducing OCR as replacement Latin. The C1 chapter-IX, C2 two-line seam, C3 first-line seam, C5 chapter-L first line, and all large blocks have explicit source-to-canvas regressions.

| Event ID | Unit | Source physical page | Lines | Historical missing seam | Rebuilt physical reader pages |
| --- | --- | --- | ---: | --- | --- |
| `miss_agesilaus_110_1649` | `agesilaus` | v2/110 | 1 | T5/207 | T5/207 |
| `miss_alcibiades_251_4338` | `alcibiades` | v1/251 | 1 | T2/126 | T2/126 |
| `miss_alcibiades_263_9839` | `alcibiades` | v1/263 | 1 | T2/151 | T2/151 |
| `miss_alexander_231_16439` | `alexander` | v2/231 | 1 | T6/89 | T6/85 |
| `miss_brutus_623_11958` | `brutus` | v2/623 | 3 | T8/126 | T7/400 |
| `miss_brutus_624_12394` | `brutus` | v2/624 | 1 | T8/128 | T7/402 |
| `miss_cato_minor_327_8321` | `cato_minor` | v2/327 | 1 | T6/289 | T6/281 |
| `miss_cato_minor_331_9707` | `cato_minor` | v2/331 | 1 | T6/296 | T6/288 |
| `miss_cleomenes_391_5043` | `cleomenes` | v2/391 | 1 | T7/61 | T6/379 |
| `miss_cleomenes_393_5916` | `cleomenes` | v2/393 | 1 | T7/65 | T6/383 |
| `miss_crassus_67_11548` | `crassus` | v2/67 | 1 | T5/119 | T5/119 |
| `miss_demosthenes_437_3468` | `demosthenes` | v2/437 | 4 | T7/155 | T7/25 |
| `miss_demosthenes_438_4274` | `demosthenes` | v2/438 | 1 | T7/159 | T7/29 |
| `miss_demosthenes_443_6045` | `demosthenes` | v2/443 | 2 | T7/167 | T7/37 |
| `miss_dio_577_4979` | `dio` | v2/577 | 1 | T8/31 | T7/305 |
| `miss_dio_579_5825` | `dio` | v2/579 | 1 | T8/35 | T7/309 |
| `miss_eumenes_91_1126` | `eumenes` | v2/91 | 2 | T5/170 | T5/170 |
| `miss_fabius_maximus_228_2889` | `fabius_maximus` | v1/228 | 1 | T2/77 | T2/77 |
| `miss_iulius_caesar_252_2835` | `iulius_caesar` | v2/252 | 12 | T6/133 | T6/129 |
| `miss_lucullus_607_2890` | `lucullus` | v1/607 | 1 | T4/55 | T4/55 |
| `miss_lycurgus_79_8882` | `lycurgus` | v1/79 | 1 | T1/153 | T1/155 |
| `miss_marcius_coriolanus_292_10538` | `marcius_coriolanus` | v1/292 | 2 | T2/207 | T2/207 |
| `miss_nicias_25_3357` | `nicias` | v2/25 | 1 | T5/34 | T5/34 |
| `miss_numa_100_7129` | `numa` | v1/100 | 1 | T1/197 | T1/199 |
| `miss_pericles_200_2252` | `pericles` | v1/200 | 1 | T2/19 | T2/19 |
| `miss_phocion_294_3556` | `phocion` | v2/294 | 1 | T6/219 | T6/215 |
| `miss_phocion_299_5777` | `phocion` | v2/299 | 1 | T6/229 | T6/221 |
| `miss_phocion_301_6648` | `phocion` | v2/301 | 1 | T6/233 | T6/225 |
| `miss_phocion_303_7514` | `phocion` | v2/303 | 1 | T6/237 | T6/229 |
| `miss_pyrrhus_475_2507` | `pyrrhus` | v1/475 | 1 | T3/185 | T3/183 |
| `miss_romulus_36_1670` | `romulus` | v1/36 | 1 | T1/64 | T1/66 |
| `miss_romulus_38_2595` | `romulus` | v1/38 | 4 | T1/68 | T1/70 |
| `miss_solon_108_832` | `solon` | v1/108 | 1 | T1/219 | T1/221 |
| `miss_solon_115_3532` | `solon` | v1/115 | 2 | T1/231 | T1/233 |
| `miss_theseus_15_774` | `theseus` | v1/15 | 7 | T1/20 | T1/20 |
| `miss_theseus_25_5427` | `theseus` | v1/25 | 1 | T1/41 | T1/42 |
| `miss_tiberius_gracchus_407_2016` | `tiberius_gracchus` | v2/407 | 1 | T7/94 | T6/412 |
| `miss_tiberius_gracchus_411_3764` | `tiberius_gracchus` | v2/411 | 1 | T7/103 | T6/421 |
| `miss_tiberius_gracchus_414_5143` | `tiberius_gracchus` | v2/414 | 2 | T7/109 | T6/427 |

All 69 reviewed glyph regions remain inside source bands and extracted strips once, in source order, with unchanged original pixels. Their scaled rows occur once in returned canvases. The compiled PDF image hashes match those verified canvases. Pixel ownership and reviewed neighbors are bound in `archive/inner_tomi_audit_repairs_2026_10_08/restoration/step-4.2/<unit>.json`. The completion gate requires every event and its full reviewed line count.

## S2 and furniture heads

S2's complete skewed source-480 chapter-IV cluster appears on **Tomus VII, physical page 111**. The exact original strip fingerprint bypasses Tall fill-splitting when it fits a fresh canvas. Classification, scale, `lowest_ink_row`, `split_tall_strip`, oversized fallback and ordinary packing remain unchanged. The positive, changed-pixel, changed-dimension/interval, ordinary-packing and oversized-stream tests pass. Full-page rendering was visually inspected.

H1 on v1/448, H2 on retained v2/375, Marius on v1/528 and the Cato Major folio on v1/432 pass full-unit head-extraction checks. Their rebuilt reader neighborhoods were visually inspected at T3/121-125, T6/348-353, T3/289-293 and T3/87-92 respectively. The three watermark joins were inspected in extracted seam crops and rebuilt neighborhoods T2/270-274, T7/358-362 and T7/399-403. These contact sheets and their physical-page selections are recorded in `final/furniture-watermark-visuals.json`. Source 432's exact crop-edge guard preserves the original rightmost body pixel. Every installed mask is checked against the union of rectangles and every original outside pixel. Faint fragment joins retain original rows and glyphs through the unchanged ink floor.

## Retained artifacts and spacing classifications

F1 (Theseus v1/17) and F2 (Pyrrhus v1/470) remain deferred: no complete safe boundary beside Latin and legitimate reference letters was proven. The exploratory foreign-scrap, lateral-clipping and rule screens were not treated as defect inventories.

R1/R2/R3, R4/R5, R7/R8 and the R9 neighborhoods on sources 374/376 receive no additional mask where no complete reviewed boundary was frozen. Existing verified masks remain in place. R2's printed chapter gaps are genuine spacing; R3's overlapping prose prevents a broad left-rule mask. The source-375 portion of R9, R6 on 321 and the watermark-page portions of R13 on 603/623 are cleaned. R10/R11/R12 second-scan pages disappear through D6/D11/D12. R14's original source-361 mask remains installed, with that second scan excluded by D7.

| Finding | Source seam | Classification and disposition |
| --- | --- | --- |
| M1 | v1/229-230 | Copied source margin; no new row compaction. Genuine paragraph/chapter spacing is preserved. |
| M2 | v1/257-258 | Copied source margin; no new row compaction. Genuine paragraph/chapter spacing is preserved. |
| M3 | v1/289-290 | Copied source margin; no new row compaction. Genuine paragraph/chapter spacing is preserved. |
| M4 | v1/295-296 | Copied source margin; no new row compaction. Genuine paragraph/chapter spacing is preserved. |
| M5 | v1/297-298 | Copied source margin; no new row compaction. Genuine paragraph/chapter spacing is preserved. |
| M6 | v1/307-308 | Copied source margin; no new row compaction. Genuine paragraph/chapter spacing is preserved. |
| M7 | v1/315-316 | Copied source margin; no new row compaction. Genuine paragraph/chapter spacing is preserved. |
| M8 | v1/329-330 | Copied source margin; no new row compaction. Genuine paragraph/chapter spacing is preserved. |
| M9 | v1/603-604 | Copied source margin; no new row compaction. Genuine paragraph/chapter spacing is preserved. |
| M10 | v2/603-604 | Copied watermark margin removed with W5; continuation intact. |
| M11 | v2/623-624 | Copied watermark margin removed with W6; missing first line restored with C5. |

M1-M11 describe copied source margins rather than a license to compact all white intervals. Body paragraphs, printed chapter/verse spacing and unrelated gaps remain unchanged. The watermark tail cases remove zero source Latin rows.

B1 (Timoleon divider-only), B2 (Brutus divider-only) and B3 (Marius terminal furniture) pass their prior real-source regressions. B4's rebuilt-body screen inspects all 2639 embedded body images and finds zero empty, <=20-pixel-wide or <=8-pixel-high ink candidates. This screen and visual samples support absence of furniture-only pages; they are not character-by-character corpus collation. White recto separators, flyleaves, signature padding and genuinely sparse complete endings are retained. W3's Google notice and the established Praefatio transition remain intentional non-findings.

## Terminal endings E01-E68

All 68 terminal reader views were rendered and visually inspected against the existing source endpoint ledger. Sparse endings remain complete. The two source-final endpoints, v1/637 and v2/708, pass real-source regressions; Otho's source 705-708 continuation survives once.

| Endpoint | Unit | Historical reader | Current physical reader | Source endpoint | Terminal source landmark |
| --- | --- | --- | --- | --- | --- |
| E01 | `theseus` | I/56 | T1/57 | v1/33, f20 | `appellamus.` |
| E02 | `romulus` | I/105 | T1/107 | v1/56, f43 | `homines non ultra visum.` |
| E03 | `comparatio_thesei_et_romuli` | I/113 | T1/115 | v1/59, f46 | `non fuisse diis cordi natales Thesei` |
| E04 | `lycurgus` | I/164 | T1/166 | v1/83, f70 | `Atque hæc de Lycurgo.` |
| E05 | `numa` | I/203 | T1/205 | v1/102, f89 | `siquidem traditur fulmine percussus conflagrasse.` |
| E06 | `comparatio_lycurgi_et_numae` | I/213 | T1/215 | v1/106, f93 | `redegit atque composuit.` |
| E07 | `solon` | I/259 | T1/262 | v1/128, f115 | `tum Aristoteles etiam philosophus.` |
| E08 | `poplicola` | I/291 | T1/293 | v1/143, f130 | `mortuus inde aufertur.` |
| E09 | `comparatio_solonis_et_poplicolae` | I/298 | T1/300 | v1/146, f133 | `jungeret, et omnem belli apparatum tolleret.` |
| E10 | `themistocles` | I/340 | T1/342 | v1/167, f154 | `cum degeremus una apud Ammonium philosophum.` |
| E11 | `camillus` | I/398 | T1/400 | v1/195, f182 | `dolorem attulerit.` |
| E12 | `pericles` | II/61 | T2/62 | v1/220, f207 | `omni remedio superior evaderet.` |
| E13 | `fabius_maximus` | II/102 | T2/102 | v1/240, f227 | `quæ vitæ responderent, invenit.` |
| E14 | `comparatio_periclis_et_fabii` | II/106 | T2/106 | v1/242, f229 | `periorem habuerunt.` |
| E15 | `alcibiades` | II/160 | T2/160 | v1/268, f255 | `proripientem telis confecisse.` |
| E16 | `marcius_coriolanus` | II/208 | T2/208 | v1/292, f279 | `spondent s.` |
| E17 | `comparatio_alcibiadis_et_coriolani` | II/215 | T2/215 | v1/294, f281 | `honestatisque negligentissimo homine.` |
| E18 | `timoleon` | II/266 | T2/266 | v1/318, f305 | `citate exegerunt.` |
| E19 | `aemilius_paulus` | II/318 | T2/317 | v1/343, f330 | `Hi mores et vita Æmilii Pauli fuisse dicuntur.` |
| E20 | `comparatio_timoleontis_et_aemilii` | II/321 | T2/321 | v1/344, f331 | `pusilli tamen animi est.` |
| E21 | `pelopidas` | II/372 | T2/372 | v1/368, f355 | `tiis passus judicari potest.` |
| E22 | `marcellus` | II/417 | T2/417 | v1/390, f377 | `Cæsar ipse theatrum Marcello inscriptum.` |
| E23 | `comparatio_pelopidae_et_marcelli` | II/423 | T2/423 | v1/393, f380 | `virtus amatur.` |
| E24 | `aristides` | III/51 | T3/51 | v1/414, f401 | `merito in admiratione est atque suspicitur.` |
| E25 | `cato_major` | III/95 | T3/94 | v1/434, f421 | `ætatis homines fuit clarissimus.` |
| E26 | `comparatio_aristidis_et_catonis` | III/104 | T3/102 | v1/438, f425 | `satis idoneus affinis futurus.` |
| E27 | `philopoemen` | III/135 | T3/133 | v1/453, f440 | `bonis semper deberi.` |
| E28 | `flamininus` | III/167 | T3/165 | v1/468, f455 | `restat ut comparationem ipsam inspiciamus.` |
| E29 | `comparatio_philopoemenis_et_flaminini` | III/171 | T3/169 | v1/469, f456 | `probene arbitrium tulerimus.` |
| E30 | `pyrrhus` | III/229 | T3/227 | v1/497, f484 | `præbuit.` |
| E31 | `caius_marius` | III/297 | T3/295 | v1/530, f517 | `mortem sibi ipse conscivit.` |
| E32 | `lysander` | III/342 | T3/340 | v1/551, f538 | `commemoranda habuimus.` |
| E33 | `sylla` | III/403 | T3/401 | v1/581, f568 | `mico maleficiis superatum fuisse.` |
| E34 | `comparatio_lysandri_et_syllae` | III/411 | T3/409 | v1/584, f571 | `fortitudinis primam laudem deferamus.` |
| E35 | `cimon` | IV/39 | T4/39 | v1/600, f587 | `Talis quidem Græcorum erat dux.` |
| E36 | `lucullus` | IV/112 | T4/112 | v1/634, f621 | `que mortis ab obitu ejus non multum distans sortitus est.` |
| E37 | `comparatio_cimonis_et_luculli` | IV/118 | T4/118 | v1/637, f624 | `fuisse.` |
| E38 | `nicias` | V/66 | T5/66 | v2/41, f648 | `quæ eventura sæpe prædixerat.` |
| E39 | `crassus` | V/120 | T5/120 | v2/67, f674 | `pendio utens eum suffocavit.` |
| E40 | `comparatio_niciae_et_crassi` | V/127 | T5/127 | v2/70, f677 | `adjecit.` |
| E41 | `sertorius` | V/164 | T5/164 | v2/88, f695 | `exosus consenuit.` |
| E42 | `eumenes` | V/194 | T5/194 | v2/102, f709 | `cum mare inspicere posset.` |
| E43 | `comparatio_sertorii_et_eumenis` | V/197 | T5/197 | v2/105, f710 | `potemque fecit.` |
| E44 | `agesilaus` | V/262 | T5/254 | v2/137, f738 | `Leonidas interfecit.` |
| E45 | `pompeius` | V/368 | T5/360 | v2/188, f789 | `Pompeii sibi allatas Cornelia in Albano humavit.` |
| E46 | `comparatio_agesilai_et_pompeii` | V/374 | T5/366 | v2/191, f792 | `defectionem ad hostes eorum fecit.` |
| E47 | `alexander` | VI/117 | T6/113 | v2/244, f843 | `ptus, sanæ mentis jacturam fecisse.` |
| E48 | `iulius_caesar` | VI/202 | T6/198 | v2/285, f884 | `ictui vehementiam addente mortuus est.` |
| E49 | `phocion` | VI/249 | T6/241 | v2/308, f905 | `et error fuit et Athenarum detrimentum.` |
| E50 | `cato_minor` | VI/341 | T6/329 | v2/363, f948 | `apud Philippos periit.` |
| E51 | `agis` | VII/36 | T6/354 | v2/378, f959 | `ceteris credidisset.` |
| E52 | `cleomenes` | VII/83 | T6/402 | v2/402, f983 | `verunt.` |
| E53 | `tiberius_gracchus` | VII/110 | T6/428 | v2/414, f995 | `Vita Scipionis singulatim scriptum.` |
| E54 | `caius_gracchus` | VII/132 | T6/450 | v2/425, f1006 | `nostros moderate ferendi facultatem ei non adimit.` |
| E55 | `comparatio_agidis_cleomenis_et_gracchorum` | VII/138 | T6/456 | v2/428, f1009 | `haud paullo inferiorem fuisse judicaverim.` |
| E56 | `demosthenes` | VII/174 | T7/44 | v2/446, f1027 | `accepimus conscriptam.` |
| E57 | `cicero` | VII/240 | T7/106 | v2/478, f1057 | `intulit.` |
| E58 | `comparatio_demosthenis_et_ciceronis` | VII/246 | T7/112 | v2/481, f1060 | `Antipatri quasi subsannans crudelitatem.` |
| E59 | `demetrius` | VII/311 | T7/177 | v2/513, f1092 | `tempus est ut Romana in scenam mittatur.` |
| E60 | `antonius` | VII/414 | T7/276 | v2/563, f1140 | `Quintus ab Antonio ratione stirpis fuit.` |
| E61 | `comparatio_demetrii_et_antonii` | VII/419 | T7/281 | v2/565, f1142 | `sese vitæ subduxit.` |
| E62 | `dio` | VIII/70 | T7/344 | v2/596, f1173 | `sumus in Vita Timoleontis prosecuti.` |
| E63 | `brutus` | VIII/132 | T7/406 | v2/626, f1203 | `ris et mortis modum significat.` |
| E64 | `comparatio_dionis_et_bruti` | VIII/137 | T7/411 | v2/628, f1205 | `loco relinqui.` |
| E65 | `artoxerxes` | VIII/176 | T8/46 | v2/647, f1224 | `Ochus, crudelitate et paricidiis omnibus superior.` |
| E66 | `aratus` | VIII/242 | T8/108 | v2/679, f1254 | `propagatum est.` |
| E67 | `galba` | VIII/275 | T8/141 | v2/695, f1270 | `reliquit.` |
| E68 | `otho` | VIII/303 | T8/164 | v2/708, f1281 | `se conjunxerunt.` |

For repeated endpoint scans the prose landmark, retained source identity and duplicate predicate determine the preserved occurrence; the historical endpoint row does not authorize a second copy.

## Durable evidence and review limits

Historical audit archive: `archive/inner_tomi_audit_2026_10_08`, manifest SHA-256 `9844828f4aaebaa0eae7498994792ae8ede899adefd73629bbc213c9a864f8b4`; 20361 verified files, no missing evidence.

Frozen source/candidate/baseline diagnostics: `archive/inner_tomi_audit_repairs_2026_10_08/baseline`, manifest SHA-256 `8ffd341b1a5c8b821f22698a0089e85f576aede01c86e7b57ecc2e6f457eab50`; 4404 files / 1,657,727,176 bytes. Separate post-dedup and post-mask manifests are bound in the fixture. Passing restoration and S2 evidence is stored outside those frozen baselines.

Final compiled-reader mappings, image hashes, all 69 source/reader comparisons, all 68 endpoint views, and samples from each inner/imposed volume: `archive/inner_tomi_audit_repairs_2026_10_08/final`. The verification script and pre-build PDF identities are preserved there. These are ordinary local archive paths; no OneDrive or remote archival claim is made.

No required environment-dependent check was unavailable. Automated original-pixel preservation, native source inspection and compiled-PDF visual checks are documented separately. Human acceptance remains the project's feedback loop: open the rebuilt PDFs and compare the listed source passages, retained ambiguous artifacts and changed signature sheets. This verification establishes the finite repaired cases and endpoints; it does not claim exhaustive corpus collation or promote Tier-B losses.
