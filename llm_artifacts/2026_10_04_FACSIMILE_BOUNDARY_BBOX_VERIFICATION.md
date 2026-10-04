# Facsimile boundary verification — 2026-10-04

## Scope and result

The intact `2026_10_04_FACSIMILE_BOUNDARY_BBOX_PLAN.md` was reread before implementation. Its opening sentence continues naturally through “with the end of the previous unit, divided by the previous unit's last lines, a decorative separator rule, and the next unit's **printed** display title.” No plan corruption or missing text was found. The branch was verified as `facsimile-boundary-bbox`.

Phases 3–5 are complete. Unit ranges carry their opening body fraction and the next same-source unit's title fraction. Opening trims respect an explicit band-count override before using the body fraction; shared-tail trims use the next title fraction and remove detached trailing furniture. Missing or invalid fractions retain the existing heuristic behavior. Sentinel metrics retain their conservative behavior. Boundary trims stay attached to the actual source opening and ending pages even if a page cannot be rendered.

Negligible-ink strips are removed before packing, using actual dark-pixel counts with a threshold of 2% of median line height times median line width and a minimum of eight pixels. Composed canvases receive a final ink check adjusted for the square of the image scale. Tests cover genuine short final lines, tiny specks, tall-band splitting, and small scales.

## Real-scan refinements and offline rebuilding

On volume 1 source page 56, vertical gutter noise joins the horizontal separator to a tall band. A row-projection check identifies a detached horizontal rule even when gutter noise connects it to other bands. It cuts in the preceding gap, preserving the last body line. The diagnostic retained body rows 2088–2134 and cut at row 2166, above the rule at rows 2211–2218. The resulting Romulus tail ends with “hominibus non ultra visum.” and has no horizontal separator.

The initial full-build invocation was rejected by automatic approval review because an incomplete front-matter prose entry could trigger external vision transcription. A tested `--offline` option permits rebuilding with existing front-matter cache contents, verifies source hashes, and neither invokes transcription APIs nor writes that cache. It conflicts with retagging and line-review options. The safe rebuild was approved and completed. Incomplete cached front matter remains incomplete; this work does not supply a new transcription for it.

The source PDFs and Latin transcription caches were not edited. The front-matter cache remained byte-for-byte unchanged, with SHA256:

```text
4E54048045D981D43488F655F5F2174BF2A439162255E2F6DEDA6D2E1135A17E
```

## Boundary cache

All 68 units have valid `title_top_frac < body_top_frac` values within [0, 1]. Automatic detection supplied 47 complete boundaries. The remaining 21 were checked against source text-layer bounding boxes and rendered scans, then pinned with review notes. Ten opening page indices were corrected to include the printed title and first body line. A subsequent `--redetect --offline` rebuild preserved all 21 pins and their fractions within 0.000001. Unit IDs, titles, kinds, groups, and source assignments were preserved.

The following page indices are source PDF pages, not printed folios. Fraction values are rounded here for readability; the cache contains the operational values.

| Unit | Source | Previous opening | Verified opening | Title fraction | Body fraction |
| --- | --- | ---: | ---: | ---: | ---: |
| numa | v1 | 84 | 84 | .094798 | .134648 |
| fabius_maximus | v1 | 221 | 221 | .092456 | .129701 |
| marcius_coriolanus | v1 | 270 | 268 | .548274 | .572997 |
| timoleon | v1 | 295 | 295 | .090475 | .118590 |
| comparatio_timoleontis_et_aemilii | v1 | 344 | 343 | .515785 | .615914 |
| pelopidas | v1 | 345 | 345 | .091574 | .115199 |
| comparatio_pelopidae_et_marcelli | v1 | 392 | 391 | .096119 | .143151 |
| philopoemen | v1 | 439 | 439 | .092823 | .116691 |
| comparatio_lysandri_et_syllae | v1 | 582 | 581 | .460281 | .511436 |
| comparatio_cimonis_et_luculli | v1 | 636 | 635 | .108478 | .161922 |
| crassus | v2 | 42 | 41 | .391932 | .425232 |
| comparatio_niciae_et_crassi | v2 | 68 | 67 | .546954 | .603510 |
| sertorius | v2 | 71 | 71 | .095607 | .130843 |
| agesilaus | v2 | 106 | 106 | .059772 | .126710 |
| alexander | v2 | 192 | 192 | .092749 | .132494 |
| phocion | v2 | 286 | 286 | .096244 | .124951 |
| cato_minor | v2 | 309 | 309 | .098661 | .124683 |
| comparatio_agidis_cleomenis_et_gracchorum | v2 | 426 | 425 | .587495 | .677666 |
| demosthenes | v2 | 429 | 429 | .097761 | .133646 |
| comparatio_demetrii_et_antonii | v2 | 564 | 563 | .210952 | .264948 |
| otho | v2 | 698 | 696 | .106996 | .144875 |

## Automated verification and generated artifacts

`cargo fmt --all` was applied. `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` passed on the final code. The full suite reported 250 library tests passed, five facsimile CLI tests passed, and four library tests ignored. The ignored tests are the two opt-in LaTeX compilation tests and two live API tests. The real full build compiled the LaTeX outputs; live API tests were not invoked. All 40 focused reflow tests passed. `cargo build --release --bin facsimile` and `git diff --check` passed.

Cargo verification used a workspace-local target directory because the configured default target is outside the writable workspace:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release --bin facsimile
```

The cache was redetected and rebuilt with `--offline`. After the gutter-rule refinement, the final full build reused the verified cache:

```powershell
$env:PLUTARCH_LUALATEX = 'C:\Users\majad\AppData\Local\Programs\MiKTeX\miktex\bin\x64\lualatex.exe'
& .\target\release\facsimile.exe --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
```

The command exited successfully and compiled all eight inner PDFs and all eight imposed PDFs. The inner PDF page counts are:

| Tomus | Pages |
| --- | ---: |
| 1 | 416 |
| 2 | 448 |
| 3 | 416 |
| 4 | 128 |
| 5 | 416 |
| 6 | 384 |
| 7 | 448 |
| 8 | 320 |

Generated PDFs and temporary QA images remain in ignored build directories; they are not committed.

## Rendered boundary checks

Final inner PDFs were rendered with Poppler at 150 DPI and inspected. Page references below are physical PDF page indices.

| Unit | Tomus | First page | Last page | Body pages |
| --- | ---: | ---: | ---: | ---: |
| Theseus | 1 | 16 | 55 | 40 |
| Romulus | 1 | 56 | 104 | 49 |
| Comparatio Thesei et Romuli | 1 | 106 | 112 | 7 |
| Lycurgus | 1 | 114 | 162 | 49 |
| Numa | 1 | 164 | 202 | 39 |
| Nicias | 5 | 22 | 68 | 47 |
| Crassus | 5 | 70 | 122 | 53 |
| Comparatio Niciae et Crassi | 5 | 124 | 130 | 7 |

- Theseus's last page ends with “appellamus.” Romulus begins beneath the generated heading with “I. Ingens Romæ nomen”; it contains neither the preceding tail nor the embedded printed title.
- Romulus's last page contains neither the following printed Comparatio title nor the horizontal separator. The Comparatio begins beneath its generated heading with “I. Habes quæ de Romulo et Theseo”. All seven Comparatio pages (106–112) contain substantial body text; no internal blank or speck folio remains. Page 113 is the intentional parity blank between units. The former problem location, page 115, contains Lycurgus body text.
- Lycurgus's last page ends with “Atque hæc de Lycurgo.” Numa opens beneath the generated heading with “Quo tempore Numa vixerit”.
- Nicias's last page ends with its body text, without the Crassus title or separator. Crassus opens beneath the generated heading with “M. Crassus, censorio triumphalique patre natus”.
- Crassus's last page ends with its body text, without the following title or separator. Its Comparatio opens beneath the generated heading with “Initio divitiæ Niciæ si cum Crassi opibus conferantur”.

These are boundary spot-checks, not a visual review of every page in all eight volumes. Existing vertical gutter lines and occasional neighboring-column fragments remain visible in some source crops, including the Romulus tail and Comparatio opening. The boundary work does not remove them from retained body bands.

## Commit record

Each implementation step was committed and pushed to `origin/facsimile-boundary-bbox` after its validation. The two additional changes were also committed and pushed separately.

| Step | Commit | Change |
| --- | --- | --- |
| 3.1 | `2bf274e` | Carry boundary fractions through unit ranges |
| 3.2 | `f41c9c8` | Trim openings at recorded body fractions |
| 3.3 | `64b52e9` | Split shared tails at recorded title fractions |
| 4.1 | `1b9feae` | Filter negligible-ink strips |
| 4.2 | `8d73b16` | Suppress near-blank canvases |
| Offline rebuild prerequisite | `7e8c1c1` | Read-only cached prose rebuild mode |
| Real-scan separator refinement | `d9c4477` | Remove rules joined to gutter noise |
| 5.1 | `5def4bc` | Verified and pinned boundary cache |

This verification note records Step 5.2 and is committed separately.
