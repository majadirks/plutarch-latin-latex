# Context

Antonius's promoted omission is installed using one exact body-band record and
fingerprint-scoped whole-strip recognition. The user accepted small Greek edge
fragments as source content. The historical missing-line count remains 23; the
approved measured count is 22. Rebuilt-reader human acceptance remains pending.

# Complete source-page preservation

The first complete-marker candidate used crop [895,1721), width 826. Native
inspection of the ending showed clipped initial Latin letters: the printed
column skews left, reaching x876. That candidate PDF, build provenance, source
overlay, and verification are retained in
`archive/tier_b_omission_repairs_2026_10_09/antonius-clipped-ending-candidate/`.
The failed full-column composition trial also remains in the command logs.

The selected crop is [876,1721), dimensions (845,2835), on v2 physical page 562
at 300 dpi (full raster 1850 × 2835). The permanent record is source 1, page 562,
owner `antonius`, observed [249,1356), protected [249,1356). Native raw banding
confirms the unchanged observed interval. The crop also retains the complete
`b`/`c` letters and marginal `d` in the following chapter. Source text, punctuation,
ligatures, and hyphens remain original raster pixels.

The wider crop introduces neighboring edge ink which gives the block its actual
cut-point strip [248,1364). Recognition uses the original grayscale region
[0,249,845,1364], SHA-256
`d19136f5c88476c20db6ccdc4727421dfb1a354e3d080f34ecf2e3cdfe24c2ce`.
The following chapter strip is [1364,1691), fingerprint
`22bc97d92d9b1ae5970495b3c87fcb081f96e78b1df76b4dbc685a3b6bbf0567`
over [0,1364,845,1691]. Both matching dimensions and original pixels are required.

The full-work scale remains 1.2023952. The recognized column uses proportional
scale 1004/845, approximately 1.1881657, fitting its full width in the 1,004-pixel
canvas. The block becomes 1004 × 1326 and fits the fresh 1394-pixel body budget;
the 389-pixel following chapter wraps whole onto the next canvas. The global
scale algorithm and furniture thresholds are unchanged. Ordinary width tests
share the same verifier with the existing Phocion case and reject altered source
pixels or dimensions.

# Required coverage and fresh PDF

`antonius_reviewed_lines_survive_composition_whole` passed one selected event /
22 measured missing lines, with 38 reviewed regions: 22 core lines, two approved
annotations, the following chapter cluster, all 12 remaining body regions on
p562, and the preceding p561 anchor. Modern passing evidence requires complete
single-canvas coverage for all 37 p562 regions; missing or split annotations and
chapter coverage are rejected by an ordinary regression. The entire missing
block and both annotations share one canvas. Original-pixel, unique retained
band/strip, source-order, and actual output-width checks pass.

The full offline generator took its compilation branch. Generated images were
preserved before normal cleanup and restored for stable additional LuaLaTeX
passes. All eight inner readers and imposition wrappers were compiled freshly
from the selected production configuration. The final tomus 7 reader has 448
pages and SHA-256
`bfefb3605fb474f8e93944c6a7ce5536bf2f3006dd436b4c64216c2296428e82`.

| PDF physical page | Printed folio | Antonius canvas | Reviewed content |
| --- | --- | --- | --- |
| 276 | 268 | 96 | Preceding text ends at `asser-` |
| 277 | 269 | 97 | All 22 missing lines, `b`/`c`, and accepted Greek edge fragments |
| 278 | 270 | 98 | Chapter LXXXVII intact; initial letters and `temporis` in the ending retained |
| 279 | 271 | 99 | Remaining ending through the last complete line |

Embedded grayscale samples match the passing source-report canvas hashes exactly.
The final rendered pages were inspected for intact letters, source order, and
seam continuity. These pages replace the old trial's physical locators 274–277.

# Evidence and state

The passing report, relevant canvases, final PDF, rendered pages, source overlay,
and build/PDF provenance are retained in
`archive/tier_b_omission_repairs_2026_10_09/installed-antonius/`.
The report SHA-256 is
`76aaa871cf0fcd5b0f2670fe4e74da2ba3a7ff233eaad188005fa4a1f4eed604`;
the 13-file manifest SHA-256 is
`e53c4e5c4eb77787bfd31ab237f864c426701f83fd5e23363054dc00c48a5621`.
Exact report and PDF-verification references are in the ledger. The portable
projection contains 136 documents and 434,900 bytes; all 135 prior semantic
documents compare unchanged and one passing report is added.

All 20 promoted events / 43 measured lines are `installed`, with 69 permanent
protection records (47 original plus 22 additions). Delivery remains
`in_progress`; no rebuilt-reader human review is inferred from acceptance of
the Greek fragments. The immutable baseline and historical Tier-A evidence are
unchanged. Both the 19-event / 21-line non-Antonius check and the historical
Tier-A source suite passed again after the complete-column correction.

Final ordinary validation passes 418 library tests and nine CLI tests (75 source
or archive tests remain explicitly ignored). Formatting and strict all-targets
Clippy pass. The three Antonius ordinary regressions require the exact crop,
complete-column fitting, and whole approved-region coverage; existing whole-strip
and configuration-reset regressions also pass. Portable-checker Python tests pass.
