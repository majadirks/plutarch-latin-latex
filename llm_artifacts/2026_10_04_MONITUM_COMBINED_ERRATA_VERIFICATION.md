# Monitum combined errata verification

## Fixture and commands

Implemented the consecutive Monitum layout on `facsimile-boundary-bbox`.
The scoped fixture uses the first committed Theseus unit with `last_page: 14`
in the ignored `build_facsimile/monitum_combined_check_units.json` file. All other
fields are preserved. The committed unit table and transcription cache are unchanged.
The separate full-build outputs under `build_facsimile/` were left untouched.

Generated the fixture with the normal offline front-matter path:

```powershell
$env:CARGO_TARGET_DIR = "$PWD\target"
cargo build --bin facsimile
.\target\debug\facsimile.exe --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --units build_facsimile/monitum_combined_check_units.json --out build_facsimile/monitum_combined_check --offline --no-compile
```

Compiled `tomus_1_inner.tex` inside that directory with MiKTeX LuaHBTeX 1.24.0
using `-interaction=nonstopmode -halt-on-error`. Poppler `pdfinfo` validates a
32-page PDF at 297.638 by 453.543 PDF points (10.5 by 16 cm). Signature padding
absorbs the content-page reduction.

## Measured fit

A separate ignored `monitum_fit_probe.tex` copy reports the actual TeX dimensions:

- Natural stack height: 364.8006 TeX points, including the 2 mm gap.
- Stack depth: 0.0 TeX points.
- Reserved height budget: 331.27855 TeX points.
- Full text width: 241.84845 TeX points (8.5 cm).
- Shared scale: approximately 0.9081, a 9.19% reduction.
- Resulting width: approximately 7.72 cm; gap approximately 1.82 mm.

The zero depth confirms that unstarred `resizebox` measures the entire stack.
Both full-width image includes occur once, in source order, within one local box.
The combined helper and first body opening use `clearpage`; the global
`freshpage` definition and unrelated single-scan renderer are unchanged.

## Page and image checks

| Physical PDF page | Content | Page style |
| --- | --- | --- |
| 14 | Monitum prose ending in `mutanda esse significavit.` | Existing prose style |
| 15 | Six p12 errata lines above the complete p13 errata crop | Empty; no head or folio |
| 16 | First Theseus body image and inline title | Empty; no head or folio |
| 17 | Second Theseus body image | `THESEUS` running head and folio 2 |

There are no blank pages between these four pages. Text extraction locates the
sentinel on page 14 and the opening title on page 16. `pdfimages -list` reports
the 1348 by 363 p12 crop and 1359 by 1652 p13 crop exactly once each, both on
page 15. The images retain their original dimensions and aspect ratios.

Rendered pages 14 through 17 at 150 DPI and source pages 12 and 13 at 100 DPI.
Visually compared the combined page against both standalone full-resolution
crops and the source scans. The six p12 lines, first and last p13 lines, edge
accents, and final Comparatio entry are present. The small gap is visible, and
the stack is centered with comfortable side margins. No newly clipped characters,
distortion, source furniture, running head, or folio is visible on the errata page.
The Greek remains legible in the rendering scaled to the trim dimensions; physical
print readability remains available for the user's final human inspection.
The prose ends at the sentinel without duplicating the errata, and generation
reports no missing-sentinel warning.

The compile log contains no overfull vertical box. It contains the existing
horizontal prose overflows of 4.86493 and 9.78493 TeX points, and underfull vertical
boxes on the unrelated QR and Volumen scan pages.

## Automated regression

Explicitly ran the external-tool regression from the repository root:

```powershell
$env:PLUTARCH_LUALATEX = 'C:\Users\majad\AppData\Local\Programs\MiKTeX\miktex\bin\x64\lualatex.exe'
cargo test monitum_combined_compile_has_adjacent_content_pages -- --ignored --nocapture
```

It passed, reporting 32 pages with prose 14, errata 15, body 16, and folio 2 on
the next body page. It uses real cached prose and source crops with two small
synthetic body images. Assertions cover a unique sentinel match, body adjacency,
the opening empty style, the next page's running head and folio, each crop's unique
embedding on the intervening page, and at least 1% dark pixels in each page half.
Missing tools or failed subprocesses fail the test instead of silently skipping it.
The ordinary test suite leaves this external-tool regression ignored.

Formatting, clippy with warnings denied, focused renderer/assembly/Monitum/volume
tests, builds, and `git diff --check` passed for the corresponding implementation
steps.

## Final green gate

After the last Rust edit, `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build`, and
`git diff --check` all passed. The ordinary suite reports 261 library tests and
5 facsimile CLI tests passed, with zero failures and 5 ignored library tests.
The combined Monitum compile regression was executed explicitly and passed
separately, including after the final Rust cleanup.

Compared against the starting commit `9e7eba6`: the transcription cache,
committed unit table, `src/facsimile/frontmatter.rs` (including crop constants),
global `freshpage` definition, and existing `embed_scan` helper are unchanged.
The README describes the delivered two-page Monitum and direct body transition.
The separate full-build outputs were not overwritten or its process stopped.
