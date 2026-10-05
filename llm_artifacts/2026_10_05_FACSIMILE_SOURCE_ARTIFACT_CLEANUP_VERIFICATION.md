# Facsimile source artifact cleanup verification

Verified on 2026-10-05 on `plan/facsimile-artifact-cleanup`.

## Plan review and implementation

The saved plan was reviewed against column detection, banding, metrics, strip extraction, composition, cache handling, and the committed source-evidence ledger. No blocking design gap was found. The existing pipeline supports cleanup before banding and excludes tail space through its existing cut points. No spacing metadata, unit override, OCR dependency, or global threshold change was needed.

Step 2.1 adds a bounded 8-connected divider search with the specified height, width, whitespace, and text-band gates. Every discarded dark pixel must belong to the accepted component; extra marginal ink causes rejection. The existing tick detector retains its contract. Source pages 51 and 59 retain their ambiguous dividers. Page 56 uses the verified local mask rather than a full-height crop advance.

Step 3.1 applies twelve normalized, page-specific rectangles to owned images before column detection and banding. The source render cache is never overwritten. Separate watermark, header, folio, bottom-edge, and right-edge rectangles preserve source dimensions. Pixels outside the rectangles, including faint type, are checked byte for byte in the real-source regressions.

Step 3.2 verifies the actual Fabius unit trims, strips, and seam composition. Page 233 ends at row 2624 rather than 2953, removing 329 copied rows. Page 235 ends at row 2622 rather than 3013, removing 391 copied rows. Neither strip reaches the furniture-only suffix. The ordinary positive body-gap medians remain 14 and 18 pixels. Page 234's first band [266,306) and page 236's first band [270,301) survive. The synthetic connected-footer fixture preserves all body bands and its 50-row paragraph gap, and reduces its composed height from 340 to 180 pixels.

## Commands and automated results

Set `CARGO_TARGET_DIR` to the workspace `target/`: the configured external target directory is inaccessible in the restricted execution environment.

- `cargo fmt --all` and `cargo fmt --all --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo build`: passed.
- `cargo test`: passed, 359 library tests and 9 CLI tests; 24 environment-dependent library tests ignored by the ordinary run.
- `cargo test facsimile::columns`: 12 passed.
- `cargo test facsimile::reflow`: 58 passed and 9 ignored before the three spacing tests were added.
- `cargo test facsimile`: 201 library tests and 7 matching CLI tests passed; 20 library tests ignored.
- Named synthetic mask and spacing tests: all passed.
- `cargo test --release reported_divider_pages_preserve_latin -- --ignored --nocapture`: passed.
- `cargo test --release reported_watermark_pages_preserve_adjacent_lines -- --ignored --nocapture`: passed, including actual unit strip endpoints and composed seam pixel checks.
- `cargo test --release fabius_shared_page_header_is_excluded_from_both_units -- --ignored --nocapture`: passed for both Fabius and its comparison.
- `cargo test --release coriolanus_source_284_header_is_excluded -- --ignored --nocapture`: passed.

The divider regression also checks raw crop bounds against the committed Phase 1 ledger. The pale strip-padding fixture verifies every neighboring body pixel and masks a region entirely inside the gap. The full ordinary suite was rerun after these final test refinements. The explicit source regressions were rerun against the final production Rust implementation. The diagnostic Phase 1 trace remains available; its evidence ledger is unchanged.

## Durable source mappings and visual checks

Source indices are zero based; source PDF pages are one based. Coordinates are full-source pixels at 300 DPI, half open.

| Original output locator | Source identity | Result |
| --- | --- | --- |
| Tomus I 95 | v1 p51, printed p38 | Ambiguous leaning divider retained: lower envelope is 22 pixels wide, and a whole-column advance threatens upper Latin. |
| Tomus I 107 | v1 p56, printed p43 | Local divider and foreign-column fragments [946,2445,1000,2630) removed; all three opening bands and their first letters preserved. |
| Tomus I 112 | v1 p59, printed p46 | Ambiguous 31-pixel-wide leaning divider retained. |
| Tomus II 90 | v1 p233 followed by p234 | Pale watermark and scan borders removed; furniture space excluded; `puram redderet.` survives. |
| Tomus II 94 | v1 p235 followed by p236 | Watermark, separable bottom scan rule, and right-edge noise removed; `egregium, me non fefellit` survives. |
| Tomus II 102 | v1 p240, printed p227 | Source running head and folio removed for both Fabius tail and comparison opening; comparison display title protected. |
| Tomus II 193, with continuation on 194 | v1 p284, printed p271 | Source running head, folio, and right-edge noise removed; first body band [270,302) survives. |

Full-resolution page 56 opening and both before/after watermark seam images were inspected. The watermark and horizontal rule disappear; adjoining Latin pixels remain intact. The source-page seams contain zero copied source-margin padding rows, as required by the existing endpoint contract. Actual globally scaled body assets were also inspected for the comparison opening, both watermark seams, Fabius tail, and Coriolanus seam before the successful-build asset cleanup. The seven affected production-DPI cached raster hashes match their SHA-256 hashes captured before rebuilding.

Diagnostic artifacts remain under `tmp/pdfs/facsimile_artifact_cleanup/`; header-unit crops are under `tmp/pdfs/recto_units_verification/crops/`. These are ignored local review artifacts. The plan and committed Phase 1 ledger remain historical records.
## Rebuilt PDF locations

Poppler renders at 150 DPI were inspected against the saved pre-build pages and source crops. Physical page numbers refer to the inner PDFs, rather than imposed printing sheets.

| Source passage | Before | After |
| --- | --- | --- |
| v1 p51 divider | Tomus I 95 | Tomus I 95, rendered PNG byte-identical |
| v1 p56 comparison opening | Tomus I 107 | Tomus I 107, local divider and debris absent |
| v1 p59 divider | Tomus I 112 | Tomus I 112, rendered PNG byte-identical |
| v1 p233/p234 watermark seam | Tomus II 90 | Tomus II 89, `puram redderet.` visible without watermark or blank furniture space |
| v1 p235/p236 watermark and scan-rule seam | Tomus II 94 | Tomus II 93, `egregium, me non fefellit` visible without watermark or rule |
| v1 p240 header seam | Tomus II 102 | Tomus II 102, with comparison opening on 103; source header and folio absent |
| v1 p284 header seam | Tomus II 193; continuation on 194 | Tomus II 191; continuation on 192; source header, folio, and blank header space absent |

The nearby pages were rendered too: Tomus I 94-96, 106-108, and 111-113; Tomus II 89-91, 93-95, 101-105, and 190-196. The affected seam pages, continuations, comparison opening, and Fabius opening on Tomus II 63 were inspected. Unaffected sample pages I 55/I 150 and II 55/II 150 retain normal running heads, folios, type scale, and paragraph spacing. No new blank body region appeared in the inspected pages. Source markings elsewhere, including marginal letters and ambiguous dividers, remain outside this cleanup's authorized erase regions.
## Complete offline build

The README's exact LuaLaTeX path was checked successfully: LuaHBTeX 1.24.0, MiKTeX 25.12. The restricted environment cannot access this executable and its package/font caches; the full build ran with approved expanded filesystem access. Git commits and pushes likewise used expanded access because the sandbox makes `.git` read-only and restricts network connections.

```powershell
$env:CARGO_TARGET_DIR = Join-Path $PWD target
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
& $env:PLUTARCH_LUALATEX --version
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
```

The build exited successfully, composed all 68 units, compiled all eight inner PDFs, and compiled all eight imposed wrappers. It reported `Volumes: 8`. The UTF-8 log is `tmp/pdfs/facsimile_artifact_cleanup/rebuild.log`. No `lualatex not run` fallback occurred. All sixteen PDFs have timestamps later than the recorded build start (2026-10-05T13:54:48.7616358-04:00). The source cache was retained; no `--clean-cache` was used.

| Tomus | Inner pages | Imposed pages |
| --- | --- | --- |
| I | 416 | 208 |
| II | 448 | 224 |
| III | 416 | 208 |
| IV | 128 | 64 |
| V | 384 | 192 |
| VI | 384 | 192 |
| VII | 448 | 224 |
| VIII | 320 | 160 |

`pdfinfo` successfully parsed every PDF. Each imposed PDF has exactly half its inner PDF's page count. Every wrapper's `\imposedsheetside` arguments cover each inner page exactly once and remain within its compiled page count. Emitted inner LaTeX retains valid `\includegraphics` syntax, per-unit image sequences, and recto-boundary structure; successful compilation confirms asset references resolved before the generator's normal composed-PNG cleanup. No generated top-level PNGs remain after the completed build. The source render cache still exists.

The first and 44th imposed sheet sides of Tomus II were rendered and inspected: book pages, trim/fold guides, and alignment are intact. Representative inner pages and source seams were inspected as described above. This is sample-based visual verification, not a claim to have inspected every page of all eight tomi.

`git diff --check` passed. The final diff contains facsimile code/tests, the in-place README description, and this verification note. Neither transcription cache, `facsimile_units.json`, the source evidence ledger, nor any historical plan/note was modified. No required environment-dependent check was unavailable. A human should still compare the cited rebuilt passages against the original scans as the plan's final scholarly review.
