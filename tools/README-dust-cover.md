# Dust-cover generator

Build the facsimile first so `build_facsimile/tomus_N_inner.tex` exists for the
chosen Tomus. The script copies its title-page center block to the front panel
and its Lives to the back; it does not change the source text. It renders a cover
for any Tomus the build produced, up to Tomus 20. Run from the repository root:

```powershell
python tools/dust_cover.py --tomus 1 --total-width 15.5 --total-height 9.5 --spine-width 1.25 --cover-width 5.5 --paper-size tabloid --exlibris "Alex Smith" --out build_facsimile
```

The result is `build_facsimile/tomus_1_cover.tex` and, when compiled,
`build_facsimile/tomus_1_cover.pdf`. Print the PDF at 100% scale on the chosen
sheet. The dotted rectangle is the cut boundary. The four panel seams have
short fold ticks above and below that boundary, so fold positions remain
visible before trimming and leave no marks on the finished jacket.

| Argument | Meaning | Required | Default |
|----------|---------|----------|---------|
| `--tomus` | Tomus number — any Tomus the build produced | Yes | — |
| `--total-width` | Finished jacket width, inches | Yes | — |
| `--total-height` | Finished jacket height, inches | Yes | — |
| `--spine-width` | Spine width, inches | Yes | — |
| `--cover-width` | Width of each front/back panel, inches | Yes | — |
| `--paper-size` | Landscape `tabloid` (11 × 17 in) or `a3` (297 × 420 mm) | No | `tabloid` |
| `--exlibris` | Owner name at the foot of the spine | No | None |
| `--printable-margin` | Unprintable margin on each sheet edge, inches | No | `0.25` |
| `--out` | Input/output directory | No | `build_facsimile` |
| `--inner-tex` | Override path for the inner `.tex` | No | `<out>/tomus_N_inner.tex` |
| `--no-compile` | Skip `lualatex`, writing only `.tex` | No | Compile PDF |

The jacket width is `2 × flap + 2 × cover + spine`; the flaps adjust equally
to fill `--total-width`. Each flap must be at least 1.5 inches. Both dimensions
must fit the printable sheet after subtracting twice `--printable-margin`.
The exterior fold ticks extend 0.12 inches beyond the jacket at the top and
bottom, so the printable-height check reserves that clearance. Invalid geometry
stops before any cover output is written. A blank `--exlibris` value is invalid.

Use `--inner-tex` when the source title page is in a different directory from
`--out`. The parsed Tomus numeral must match `--tomus`; the tool never guesses
which Lives belong to a Tomus from a hard-coded list. LuaLaTeX and the
`OldStandard` font used by the inner book must be available for PDF compilation.
