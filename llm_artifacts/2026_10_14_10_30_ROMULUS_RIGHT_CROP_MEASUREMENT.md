# Romulus III right crop measurement (2026-10-14)

The production 300 DPI render of `Ploutarchou_Bioi_v1.pdf` PDF p. 34 is
`1982×3008`. The baseline ignored test `romulus_chapter_break_deskewed`
passes with detected bounds `[950,1774)` and guarded bounds `[945,1773)`.
The installed shear produces `1982×3040`; the reviewed mask and removal of
post-shear rows `[1892,2135)` produce `1982×2797`. The retained crop is
`828×2797`. The chapter-break bands pinned by the test are `[1840,1878)`
and `[1906,1943)`.

I inspected the raw Latin at full raster size, including the whole right edge
from raw rows 250 through 2630. The rectangles below are raw-source pixel
coordinates; their right and bottom edges are exclusive. Each covers the
complete terminal letters and the old crop edge. Counts use grayscale < 200
in `[1773,1790)` and the stated row interval. The last dark `x` is the
greatest dark column in that strip. Row ranges map by
`raw_y + round((x-1400) × 0.016079632465543646) + 23 - 243` for these
post-gap lines; the shift is 6 for the terminal columns.

| Source line ending | Raw rectangle | Dark pixels beyond old edge | Last dark x | Mapped terminal rows |
|---|---|---:|---:|---|
| `narrandi` | `[1730,2254,1790,2281)` | 84 | 1777 | `[2040,2067)` |
| `Fx` (printed form) | `[1730,2304,1790,2329)` | 38 | 1776 | `[2090,2115)` |
| `regnum` | `[1730,2354,1790,2381)` | 87 | 1778 | `[2140,2167)` |
| `patrimonio` | `[1730,2403,1790,2421)` | 84 | 1780 | `[2189,2207)` |
| `re-` | `[1730,2444,1790,2468)` | 40 | 1780 | `[2230,2254)` |
| `poneret,` | `[1730,2490,1790,2517)` | 60 | 1781 | `[2276,2303)` |
| `ex` | `[1730,2535,1790,2559)` | 101 | 1782 | `[2321,2345)` |
| `eum` | `[1730,2586,1790,2605)` | 86 | 1782 | `[2372,2391)` |

The preceding chapter III heading and its next two lines have dark pixels
at x 1773–1774 too, so the repair also restores their tiny clipped tips on
reader folio 43. Across the full Latin body (raw rows 250–2630), there is
no dark pixel at x 1783–1899. The exclusive edge 1790 keeps seven white
columns after the rightmost glyph; visual inspection shows no Greek, divider,
scan edge, or other furniture there. The left edge remains 945.

At the frozen production scale 1.2023952, the old 828-column image rounds
to 996 px, while the proposed 845-column image rounds to 1016 px. The
`TrimGeom::default().content_width()` measure is 1004 px, so the widened
reviewed image needs the existing signature-scoped scale cap. The unchanged
source render and cropped detail are available in the local render cache and
`tmp/romulus_p34_raw_detail.png` for human comparison with reader folios
43–44.
