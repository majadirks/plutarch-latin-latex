# Context

Source-to-canvas checks must cover the complete printed glyphs. A rectangle
measured inside a clipped detector crop cannot establish that requirement.
Native-source inspection of the promoted lines reveals small glyph portions
outside several original detector bounds. These edition-specific corrections
extend the crop without changing source pixels or detector thresholds.

# Original source measurements

The source rasters are the original 300-dpi cached renders. The immutable
baseline remains under `archive/tier_b_omission_repairs_2026_10_09/baseline/`.
The diagnostic sheet is
`tmp/pdfs/inner_tomi_audit_repairs/new-glyph-crop-edge-review.png`; it shows
expanded original-source crops, including the whole first/last glyph. Measured
ink bounds use grayscale <=200 and are confirmed against the native images.
Unrelated ink beside the indented Caius Marius and Nicias lines is outside the
Latin glyphs and does not authorize enlarging those crops.

| Unit | Source/page | Raster dimensions | Original detector bounds | Corrected production bounds | Corrected column width | Preserved glyph edge |
| --- | --- | --- | --- | --- | ---: | --- |
| aemilius_paulus | v1/326 | 1966 × 2982 | [922,1712) | [921,1712) | 791 | initial `d` |
| lysander | v1/535 | 1978 × 2969 | [1049,1845) | [1045,1845) | 800 | initial `g` and `n` in the two reviewed lines |
| marcellus | v1/377 | 1952 × 2957 | [1048,1856) | [1048,1857) | 809 | final `m` in `militiam` |
| timoleon | v1/318 | 1937 × 2988 | [900,1691) | [899,1691) | 792 | initial `l` |
| cato_minor | v2/352 | 1830 × 2842 | [879,1683) | [876,1683) | 807 | initial `d` |
| dio | v2/566 | 1813 × 2838 | [892,1697) | [888,1697) | 809 | initial `s` |
| dio | v2/580 | 1816 × 2835 | [892,1704) | [889,1704) | 815 | initial `c` |
| brutus | v2/619 | 1822 × 2824 | [1017,1822) | [1014,1822) | 808 | initial `q` |
| demosthenes | v2/445 | 1844 × 2842 | [1017,1844) | [1015,1844) | 829 | initial `n` |

Dio p580 has source ink through row 2596, with no glyph ink in rows 2597–2607
even at grayscale <=220. Its measured half-open bottom remains 2597; no bottom
extension is required. The correction restores its left edge.

# Installation and verification contract

Each crop exception matches source index, physical page, full raster dimensions,
and both original detector edges. Mismatches retain the detected crop. Install
each exception with its corresponding repair step and change protection record
dimensions to the measured production width. Observed/protected row bounds
must be checked against the resulting real-source detector and retained trace;
they are not inferred from the new width.

Reviewed glyph rectangles include the expanded original bounds. Existing
unrepaired baseline JSON and archived trial reports remain unchanged. Fresh
candidate reports and original-source overlays identify the corrected bounds.
A candidate must pass the complete source/pixel/strip/canvas checks before its
event can use the fresh report as installed evidence. Rebuilt-reader review is
still required for restored status.

Timoleon p318 is also requested by Aemilius Paulus. Alcibiades p242 is also
requested by `comparatio_periclis_et_fabii`; Dio p566 is also requested by
`comparatio_demetrii_et_antonii`. Pass A precedes unit ownership trims, so new
protection can affect these units' metrics. Baseline comparison must identify
all units reading repaired physical pages and verify their previously retained
Latin, including the owner-specific shared-page boundaries.
