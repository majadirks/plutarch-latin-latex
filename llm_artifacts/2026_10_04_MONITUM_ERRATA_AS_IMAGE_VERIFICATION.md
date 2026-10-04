# Monitum prose and errata image verification

The implementation follows `2026_10_04_MONITUM_ERRATA_AS_IMAGE_PLAN.md` on branch
`facsimile-boundary-bbox`. The complete plan decodes as UTF-8, with no replacement
characters or missing final section. The Latin transcription cache is unchanged.

## Review findings

- The renderer emits its own missing-sentinel warning, matching the step's explicit
  behavior rather than the example doc comment's assignment to the caller.
- Reconciliation independently selects unfinished pages. It therefore needs the
  same scope as transcription to exclude an image page with a cached first pass.
  `reconcile_all_in_scope` supplies that scope; `reconcile_all` delegates to it with
  an unrestricted predicate for existing callers. The exclusion test covers both
  an empty p13 record and one with a cached first pass, and verifies the p13 record
  stays unchanged.
- Crop comments describe their source regions without implementation-plan references.
- The actual cached p12 paragraph contains `mutanda esse significavit.` in one block.
  The sentinel requires no adjustment or transcription edit.

## Render and visual inspection

A temporary unit table under the ignored `build_facsimile/` directory selects the
existing Theseus opening at v1 p14 and sets its last page to 14. This exercises the
normal build path with front matter enabled, while limiting the body to one source
page. The committed unit table is unchanged.

```powershell
.\target\debug\facsimile.exe --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --units build_facsimile/monitum_check_units.json --out build_facsimile/monitum_check --offline --no-compile
& 'C:\Users\majad\AppData\Local\Programs\MiKTeX\miktex\bin\x64\lualatex.exe' -interaction=nonstopmode -halt-on-error tomus_1_inner.tex
pdfinfo build_facsimile/monitum_check/tomus_1_inner.pdf
```

The LaTeX command runs inside `build_facsimile/monitum_check/`. The build makes no
vision calls. `pdfinfo` reports 32 pages at 297.638 × 453.543 pt, including the existing
recto breaks, flyleaves, two body image pages, and signature padding.

The source pages, both cropped PNGs, and PDF pages 14, 16, and 18 were visually
inspected after rendering with Poppler:

- Page 14 contains the single re-typeset Monitum prose page ending at the sentinel.
- Page 16 contains all six printed lines of the p12 errata tail, starting at
  `Thesei cap. 1` and ending at the continuation into p13. No prose line, signature,
  or source folio survives.
- Page 18 contains all p13 errata lines, including the first line's accents and the
  final Comparatio entry. No running head or ornament survives.
- The two source scans preserve the source pixels and fit the trim measure. The
  short p12 crop leaves vertical whitespace, as intended by the plan.
- The generated TeX contains each errata image once, after the prose, and contains
  no re-typeset `Thesei cap` entry. No missing-sentinel warning was emitted.

Final edge fractions `(left, top, right, bottom)`:

| Scan | Crop |
| --- | --- |
| v1 p12 errata tail | `(0.13, 0.769, 0.185, 0.109)` |
| v1 p13 errata | `(0.18, 0.094, 0.14, 0.350)` |

MiKTeX compiles successfully. It reports two overfull prose boxes (4.86493 pt and
9.78493 pt); visual inspection shows small right-margin protrusions with no clipped
text. Prose typography is outside this plan's changes.

## Local checks

Cargo's configured target directory is outside the writable workspace. Checks use
`$env:CARGO_TARGET_DIR = "$PWD\target"` without changing the repository configuration.
The focused prose, front-matter, reconciliation, volumes, and assembly tests pass;
`cargo build`, clippy with warnings denied, and the formatting check pass for the
crop-tuned implementation.
