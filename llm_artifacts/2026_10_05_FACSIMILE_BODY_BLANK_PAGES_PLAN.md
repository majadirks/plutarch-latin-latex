# Context

The facsimile reader should have no blank pages between body units. In the current `build_facsimile/tomus_5_inner.pdf`, physical PDF pages 67 and 121 are blank. Both interrupt reading at unit boundaries. This plan removes parity padding from facsimile body openings while retaining a separate page for each unit's first image and inline heading.

This is a layout change. Preserve all source images, image order, Latin titles, cached transcription, front-matter prose, and transcription orthography exactly. Do not repair the PDF by deleting pages downstream: fix the generator and regenerate the imposition wrappers from the compiled page counts.

# Current state of the world

## Confirmed diagnosis

The diagnosis uses physical, one-based PDF page numbers, not displayed body folios. Poppler rendering confirms that pages 67 and 121 are entirely white. Text extraction identifies the adjacent titles; the LuaLaTeX log identifies the image placed on each page:

| Physical PDF page | Content | Evidence in `build_facsimile/tomus_5_inner.log` |
| --- | --- | --- |
| 66 | Last Nicias image, `facsimile_nicias_47.png`; displayed folio 48 | Lines 829-836 |
| 67 | Blank | Lines 836-839 |
| 68 | Crassus heading and `facsimile_crassus_1.png` | Lines 840-847 |
| 120 | Last Crassus image, `facsimile_crassus_53.png`; displayed folio 102 | Lines 1261-1268 |
| 121 | Blank | Lines 1268-1271 |
| 122 | Comparatio heading and `facsimile_comparatio_niciae_et_crassi_1.png` | Lines 1272-1281 |

The generated `.tex` ends each image with `\par\clearpage`, then emits `\freshpage` before the next unit. The shared preamble in `src/latex.rs::inner_preamble` defines:

```tex
\newcommand{\freshpage}{\clearpage\ifodd\value{page}\thispagestyle{empty}\hbox{}\clearpage\fi}
```

After physical page 66 ships, the page counter is 67. The initial `\clearpage` has no content to ship. The odd-page branch puts an empty box on page 67 and ships it, so Crassus starts on page 68. Exactly the same mechanism inserts page 121 after page 120. `\beginbodynumbering` changes the displayed `\thepage`, not the underlying physical page counter used by `\ifodd`.

These blanks are deliberate output from the custom parity macro, not missing scans or failed image layout. The rule forces even physical page starts; do not misdiagnose it as LaTeX's usual odd-page chapter-opening behavior. The same log shows further empty boundary pages at 129, 195, 203, and 373. Those numbers describe the current build and will move after the fix.

## Relevant implementation and conventions

- `src/facsimile/assemble.rs::inner_document` emits flyleaves, optional front matter, `\beginbodynumbering`, body units, back flyleaves, and signature padding.
- Its body loop computes `force_parity_start = !(index == 0 && vol.emit_front_matter && vol.tomus == 1)` and passes it to private `unit_block(unit, force_parity_start)`. Thus every later body unit uses `\freshpage`; the first Tomus I unit after the combined Monitum images is the existing exception.
- `unit_block` already has a consecutive-page path using `\clearpage`. It emits an inline Latin heading and suppresses the opening page's running head and folio with `\thispagestyle{empty}`. Later images retain the running head and normal folio.
- Existing tests in this module use `unit`, `unit_kind`, `volume`, and `volume_full` fixture helpers. Several tests explicitly require parity starts and must change with the intended contract.
- `monitum_combined_compile_has_adjacent_content_pages` is an ignored test in the same module. It uses real front-matter resources, `tempfile`, generated PNG fixtures, the existing compiler, `pdftotext`, and `pdfimages`. Follow these patterns for a small compiled body regression rather than building a parallel test framework.
- `src/latex.rs` shares `\freshpage` with front matter and the transcription reader. Keep that macro and those call sites unchanged. The scope here is facsimile body openings, including the first body opening in every tomus and restricted runs.
- Front/back endpapers come from `latex::blank_pages`. `latex::SIGNATURE_PADDING` pads at the document end to a multiple of 32 physical pages. These are intentional structural blanks.
- `src/facsimile/mod.rs::compile_and_impose` compiles inners, reads actual page counts with `compile::inner_page_count`, and regenerates wrappers through `latex::write_imposition_wrappers`. Imposition uses physical page indices, so it does not require each body unit to start on an even page.
- `write_wrappers_from_estimate` supplies approximate counts when no compiler runs. Leave its existing estimate contract in place; production validation must use actual compiled PDFs and counts.

# Desired end state

Every nonempty facsimile body unit begins on the next available physical page, regardless of parity, tomus number, `LifeKind`, or presence of front matter. Unit boundaries retain page breaks and the existing inline heading; images remain in the same order, with one body image per page. No parity blanks occur between body images or between the final front-matter content and the first body unit.

All front-matter layout rules, flyleaves, and final signature padding remain intact. Compiled inners have page counts divisible by 32 and regenerated printing wrappers refer to the new physical page positions. The total PDF page count may remain unchanged when removed body blanks are absorbed by final signature padding.

For the current Tomus V fixture, Nicias should still start on physical page 20: its initial page currently needs no parity blank. With all unit parity starts removed, Crassus should start on page 67 and its Comparatio on page 120. Page 121 will therefore contain the second Comparatio image. Validate content adjacency rather than treating the old blank-page numbers as universal assertions.

# Tech Learnings

`\clearpage` ships pending content and floats; calling it on an already empty page does not itself create a blank. In contrast, `\hbox{}` contributes an empty box, making the page real enough to ship. Changing `\thepage` affects printed folios, while `\value{page}` remains the counter used for physical pagination here. End padding can keep the final PDF length fixed even when body pagination changes.

# Phase 1 - Consecutive facsimile body openings (Suggested implementer: Sonnet)

## 1.1 Simplify the body emitter and update its contract tests

Before editing Rust or invoking any build that overwrites generated outputs, capture the baseline evidence specified in Step 3.1. Establish any missing baseline with the unchanged generator at this point so the final fidelity comparison does not depend on recovering deleted artifacts.

Edit `src/facsimile/assemble.rs`:

- In `inner_document`, iterate directly over `&vol.units`, remove the index and `force_parity_start` calculation, and call `unit_block(unit)`.
- Change private `unit_block` to accept only `&PlacedUnit`; initialize its output with `String::from("\\clearpage\n")`. Remove the conditional command selection. Retain the leading break so the unit helper's contract is independent of whether the caller has already shipped the preceding page.
- Update its doc comment to describe consecutive page starts, inline headings, and running-head behavior. Keep all image emission, escaping, page style, and final document structure unchanged.
- Update direct test calls to the new signature.
- Rename `inner_tex_starts_each_unit_fresh_page` to `inner_tex_starts_each_unit_on_next_page`. Assert the `\clearpage\n\markboth` opening for each unit; keep heading and ordering assertions.
- Rename `first_body_keeps_parity_start_without_monitum` to `first_body_uses_next_page_with_or_without_frontmatter`. Cover normal Tomus I, a later tomus with front matter, and a restricted volume. Assert the consecutive opening in each.
- In `monitum_errata_and_first_body_use_consecutive_pages`, change the later Romulus assertion to the consecutive opening. Keep the existing Monitum-to-first-body assertions. Preserve `monitum_without_cached_prose_still_uses_consecutive_pages`.
- Add `facsimile_body_contains_no_parity_starts`: use a volume containing Life and Comparatio units; inspect the substring after `\beginbodynumbering`, where no `\freshpage` calls should remain. Test both front-matter flags and tomus 1 and 5. Do not search the whole document, since the preamble definition and front-matter calls remain valid.
- Retain image-count, title, running-head, flyleaf, and padding assertions. Do not introduce a new page-start abstraction or duplicate emitter logic: the existing helper is the single definition for all units.

### LLM verification

Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test facsimile::assemble`. Inspect `git diff -- src/facsimile/assemble.rs` and confirm no cached Latin or shared preamble changed.

### Human verification

Read the emitter diff and confirm that units still receive their inline heading and page break. Confirm the new rule applies to Lives and Comparatio units and to the first body unit in every volume.

# Phase 2 - Prove compiled adjacency (Suggested implementer: Sonnet)

## 2.1 Add a small compiled regression using existing tools

In the existing test module in `src/facsimile/assemble.rs`, add `facsimile_body_compile_has_no_blank_pages_between_units`, marked `#[ignore = "requires MiKTeX and Poppler"]`. Like the existing ignored compile test, assert tool availability and fail clearly when prerequisites are absent; do not silently return success.

Use `tempfile::tempdir`, the existing volume helpers, `image::GrayImage`, `write_inner`, and `compile::compile` with `compile::INNER_PASSES`. Create six distinct, visibly inked PNGs with different widths and heights and safe short aspect ratios. Put them in three units with image counts 2, 3, and 1; make the third unit a Comparatio. Set `emit_front_matter = false` and use tomus 5. Four initial flyleaf pages mean the fixture's body images should occupy physical pages 5 through 10, exercising both odd and even unit openings. Do not use a real source PDF or a vision call for this fixture.

Invoke `pdfimages -list` using the `PLUTARCH_PDFIMAGES` override pattern already present. Parse image rows as in the existing test and match each fixture by its unique dimensions. Assert each occurs exactly once and on pages 5, 6, 7, 8, 9, and 10 respectively. This proves actual image adjacency, which total page count cannot establish because end padding absorbs removed blanks.

Assert `render::pdftotext_available()` and use `render::page_text(&inner_pdf, page)` separately for each required physical page, matching `monitum_combined_compile_has_adjacent_content_pages`. Assert inline titles on pages 5, 7, and 10; opening pages have no standalone numeric folio line, and continuation pages 6, 8, and 9 have their unit running head and body folios 2, 4, and 5 respectively. Do not add a separate whole-document text-extraction subprocess or form-feed parser.

Assert the compiled count is exactly 32, which also establishes divisibility by 32 for this fixture. Its layout is four front flyleaf pages (1-4), six body-image pages (5-10), four back flyleaf pages (11-14), and signature padding (15-32). Assert the image listing contains no images outside pages 5-10; together with the per-image assertions, this establishes that every fixture image precedes the back flyleaves. Pages 11-14 are identified by the emitter's structural order, not by distinguishing their appearance from final padding. No rendered-blank assertion is required for these unchanged structural pages; empty extracted text alone would not prove that a rendered page is blank. Keep image-list parsing local to the tests; if both compile tests need identical parsing, extract one test helper and reuse it.

### LLM verification

Set `$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"` as documented in README; adjust only if the installation path differs. Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test facsimile::assemble`, and `cargo test facsimile_body_compile_has_no_blank_pages_between_units -- --ignored --nocapture`. Run the existing `cargo test monitum_combined_compile_has_adjacent_content_pages -- --ignored --nocapture` to preserve the front-matter regression. Required external tests must actually run for completion; report missing prerequisites rather than claiming validation.

### Human verification

Inspect the test's page expectations and verify that the 2/3/1 image distribution exercises both parities and a Life-to-Comparatio transition. Confirm no real transcription or source-image transformation is part of the fixture.

# Phase 3 - Regenerate and inspect the real reader (Suggested implementer: Opus)

## 3.1 Rebuild inners and printing wrappers and verify the reading sequence

Use the existing documented offline full-build route so front-matter caches and source rasters are reused. Do not retag, redetect units, clear caches, edit cached prose, or hand-edit generated `.tex` files. Before rebuilding, create `tmp/pdfs/facsimile_body_blank_pages_verification/` with separate `baseline/` and `rebuilt/` directories. A full build calls `clean_generated`, deleting existing generated `.tex`, `.pdf`, and composed PNG files; successful `compile_and_impose` also removes the newly composed PNGs. Capture the following baseline evidence before invoking the build:

- Copy all eight inner `.tex` files and their LuaLaTeX `.log` files into `baseline/`; preserve each inner's physical page count and `pdfimages -list` output there as well.
- Record SHA-256 hashes with `Get-FileHash -Algorithm SHA256` for `facsimile_units.json`, `plutarch.frontmatter.tags.json`, and both source PDFs. Record relative paths and hashes for every existing file under `build_facsimile/render_cache/` in a separate manifest.
- Extract embedded images from all eight baseline inner PDFs using `pdfimages -png`, honoring `PLUTARCH_PDFIMAGES` when set. Associate extraction indices with the corresponding `pdfimages -list` rows. Use the ordered body filenames from the inner `.tex` and their page shipments in the log to select body images. Save a manifest containing tomus, body filename, sequence index, physical page, dimensions, and a SHA-256 hash of the decoded grayscale pixel bytes for each body placement. Use one verification script stored in the verification directory for both baseline and rebuilt manifests; do not compare PNG file bytes, since encoding differences need not indicate changed pixels.

If any baseline inner, log, or required cache is missing, report the missing evidence before rebuilding and establish a complete baseline with the unchanged generator before implementing Phase 1. If Phase 1 has already been applied, use a separate checkout of the pre-change revision and a separate output directory to obtain that baseline without reverting the working changes. Do not claim an exact before/after fidelity comparison without the baseline evidence.

```powershell
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
& $env:PLUTARCH_LUALATEX --version
if ($LASTEXITCODE -ne 0) { throw "LuaLaTeX is unavailable." }
cargo run --release --bin facsimile --target-dir target -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
if ($LASTEXITCODE -ne 0) { throw "The facsimile build failed." }
```

The generator may otherwise finish with LaTeX alone when the compiler is unavailable; require an actual successful PDF compile and regenerated wrappers. Save rebuilt inner `.tex` files, logs, physical page counts, image listings, and decoded body-image manifests under `rebuilt/`. For each of all eight inners, read the ordered `facsimile_*.png` body-image references after `\beginbodynumbering` in the generated `.tex`. Correlate each filename with the physical page shipment that contains it in the LuaLaTeX log, handling wrapped log lines. Cross-check those pages against the `image` rows of `pdfimages -list`; do not use dimensions as unique production image identifiers, because multiple body images can share dimensions. Exclude front-matter images using the expected body filename sequence and its log-derived pages, rather than guessing from image dimensions.

Assert every expected body filename is placed exactly once and in the expected order, each body page contains exactly one body image, and physical body-page indices form a consecutive sequence across the entire body. Reject missing or additional body placements rather than truncating comparisons to the shorter sequence. For each volume with front matter, identify the final front-matter content page from the pre-body `.tex`, log, image listing, and per-page text extraction, then assert the first body image is on that page plus one. For restricted volumes without front matter, assert the first body image follows the four front flyleaf pages directly. This checks the front-matter-to-body boundary as well as the first-to-last body-image range. A facsimile body is raster content, so empty `pdftotext` output is not evidence of a blank page; use image placement and rendering together.

Render Tomus V physical pages 66-68 and 119-122 with `pdftoppm`, and inspect the full pages. Confirm Nicias-to-Crassus and Crassus-to-Comparatio adjacency, intact headings and images, and correct continuation running heads and folios. Check all remaining unit boundaries in Tomus V and audit ordered image adjacency in all eight inners. Blanks may remain within front matter under existing layout rules and after the body as endpapers/padding; there should be none between the final front-matter content page and the first body image or within the first-to-last body-image range.

Verify each inner page count is divisible by 32, each wrapper's `\innercount` equals its freshly compiled inner count, and all eight imposed PDFs were regenerated. Run the existing imposition tests; no imposition mapping change is planned. Compare saved baseline and rebuilt inner `.tex` files: the only expected differences are body-opening `\freshpage` commands becoming `\clearpage`; preamble definitions, front-matter calls, Latin titles, unit order, image references, flyleaves, and signature-padding code must remain byte-identical. Compare the ordered decoded body-image manifests by tomus, filename, sequence index, dimensions, and pixel hash, allowing physical pages to change. Require identical image counts and pixels with no missing or duplicated placements.

Recompute the source-PDF, unit-table, and transcription-cache hashes and require exact matches with baseline values. For the raster-cache manifest, require every previously existing entry to remain present with the same hash; newly populated entries are acceptable and must be listed in the verification note. `git diff` supplements these comparisons but cannot prove fidelity for ignored or untracked artifacts. Visually inspect representative imposed spreads around a moved boundary.

Record commands, tool versions, baseline/rebuilt manifest locations, fidelity-comparison results, newly populated raster-cache entries, old/new boundary positions, page counts, front-matter-to-body adjacency results, visual findings, and any limitations in `llm_artifacts/2026_10_05_FACSIMILE_BODY_BLANK_PAGES_VERIFICATION.md`. Retain this plan and the verification note as historical records.

### LLM verification

Run `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo build` before declaring completion. Inspect the actual build outputs and wrapper counts as described above; file existence alone can reflect stale PDFs. Require passing baseline/rebuilt `.tex`, decoded body-image, source/cache hash, and all-volume adjacency comparisons. Inspect `git diff` and `git status --short` to confirm the intended scope, supplementing the explicit fidelity manifests.

### Human verification

Open the rebuilt Tomus V reader and read through the Nicias/Crassus and Crassus/Comparatio transitions. Inspect every other Tomus V unit opening and representative transitions in other tomi. Confirm no intervening blanks, no lost or duplicated Latin lines, and legible inline headings. Inspect an imposed spread crossing a moved boundary and confirm endpapers and final blank padding still serve the binding layout.
