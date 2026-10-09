# Context

The Tier-B omission repair plan requires individual source glyph rectangles for
Antonius and preserves the historical audit inventory of 20 events / 44 lines.
Measurement of the original 300-dpi source raster exposes a discrepancy in that
inventory.

# Source measurement and decision

Antonius v2 p562 has 22 printed body lines between the preceding `asser-` anchor
and chapter LXXXVII. The block begins `vatam dicunt` and ends `a Cæsare redemit.`
Its original production column is [906, 1721), and the detector interval is
[249, 1356). The source lines slope downwards across the column. Source-pixel
segmentation measured their complete rectangular glyph extents using the slope
of 0.043 pixels per horizontal pixel; the resulting native-source overlay is
`tmp/antonius-measured-line-overlays.png`. These rectangles overlap vertically,
so their bounds do not authorize horizontal cuts between lines.

The user approved retaining the historical 23-line count while recording 22
measured lines and adjusting completion verification to match the source.
`missing_line_count` remains 23 in the historical event. Its explicit
`measured_line_count` is 22. The other events retain their existing counts.
The initial Tier-B inventory remains 20 events / 44 historical lines; a complete
measured delivery is 20 events / 43 source lines. Partial delivery remains
19 events / 21 lines, with Antonius unresolved as one event / 22 measured lines
(23 in the historical inventory).

Source prose, source rasters, and transcription caches remain unchanged. The
neighboring `asser-` and chapter LXXXVII lines are additional verification anchors;
neither is counted as a missing line to manufacture a 23-line measurement.

# Verification contract

Every measured missing line receives its own original-source glyph rectangle.
Completion requires all 22 Antonius rectangles to survive uniquely in reading
order and whole on returned canvases, along with source annotations and both
anchors. Pixel retention does not replace the rebuilt-reader review required by
the repair plan. The method decision and delivery record must reference this
approved count correction.

The expanded original crop of Dio v2 p580 also reveals a clipped left edge:
the first Latin `c` starts at x=889, but the unrepaired detector crop starts at
x=892. Original ink occurs at x=889, 890, and 891 in rows 2555–2571. The raster
dimensions are (1816, 2835), and the detector bounds are [892, 1704).
Its reviewed missing-line rectangle includes x=889. Installing its foot-band
protection therefore also requires a measured production crop extension to
[889, 1704), preserving the complete original glyph. The historical detector
signature (812, 2835), observed [2550, 2597), remains baseline evidence; the
candidate production width is 815. This source-pixel finding supersedes the
plan's assumption that all non-Antonius repairs need no crop-edge change.
