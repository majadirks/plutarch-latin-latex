# Context

The four merged or tightened repairs preserve Latin below running-head fragments
in Brutus, Cato Minor, and Demosthenes, and remove the non-body upper portion of
Phocion's first-line band. The source-based checks also exposed an incorrect
Demosthenes neighbor measurement and output-width clipping in Phocion. The repair
includes their measured corrections. Printed text remains original raster content.

# Protection and excluded furniture

| Unit | Source/page | Production column | Column dimensions | Observed rows | Protected rows | Excluded head rows |
| --- | --- | --- | --- | --- | --- | --- |
| brutus | v2/619 | [1014,1822) | (808,2824) | [120,225) | [193,225) | [129,158) |
| cato_minor | v2/329 | [1017,1856) | (839,2831) | [40,210) | [180,210) | [114,142) |
| demosthenes | v2/445 | [1015,1844) | (829,2842) | [44,226) | [197,226) | [127,156) |
| phocion | v2/291 | [973,1846) | (873,2836) | [173,239) | [206,239) | None |

Native-source overlays confirm the three running-head fragments lie above the
Latin. Phocion's removed upper rows contain page-edge rule ink and no body glyph.
The complete first Latin line remains protected. Brutus and Demosthenes extend
their detector crops left by three and two pixels to retain complete initial
glyphs. Actual source tests verify the observed row intervals with those crops.

The original images, row profiles, overlays, and identities are authenticated in
`archive/tier_b_omission_repairs_2026_10_09/confirmed-merged-heads/measurements.json`,
SHA-256 `f4e8dcdc09865a9dab3292c8522354dedd858ce4660e179c8bcf11da965a7292`.
Its manifest retains ten measurement files. Historical baseline bytes are unchanged.

# Demosthenes predecessor correction

The first candidate retained and composed the repaired Demosthenes line, but the
verifier's preceding neighbor [1554,2592,1579,2611] on p444 was the printed folio
`20`. Its retained detector band did not pass the unchanged production strip ink
floor. The actual preceding Latin line is [908,2545,1685,2575]. The native source
and overlay confirm that distinction.

The failed check, production stage trace, source images, canvases, and measured
correction remain in the 84-file
`archive/tier_b_omission_repairs_2026_10_09/merged-head-predecessor-failure/` archive.
Its `neighbor-correction.json` has SHA-256
`8ca7cfe0edddfa19fdf4af6cf3c2d6a0bcadb91bf8792d401c4890f3d7936ffb`.
The event's current neighbor rectangle is corrected; the immutable baseline still
records its original measurement. Neighbor measurement selects bands represented
by production-extracted strips, mapping original rows through any deleted rows.
`tier_b_neighbor_measurement_skips_uncomposed_furniture` verifies both the rejected
folio and coordinate preservation after compaction. The ignored
`demosthenes_merged_line_composition_trace` reproduces the diagnostic.

# Phocion output-width preservation

The full-work scale is 1.2023952. Phocion p291's 873-pixel column would become
1,050 pixels wide, exceeding the 1,004-pixel canvas. Its reviewed glyph right edge
would land at pixel 1,029. Cropping away whitespace cannot fix that overflow:
the complete printed line itself is wider than the available source-width budget.

`source_strip_scale` recognizes this exact (873,2836) column through the original
source region [6,206,855,239], SHA-256
`ca1e88fb55367597531166802b72bb1be361c09718cc95e4f5aa1c701e192eeb`.
Its strips use the smaller of the requested scale and 1004/873 (approximately
1.1500573), fitting the full column through ordinary proportional resampling.
This applies to the recognized column's body strips and preserves source layout,
including the first line's ending and following prose. The full-work scale
algorithm is unchanged. Other dimensions or changed source pixels reject the
exception; a smaller requested scale retains its size.

The failed width check, original source, overlay, and measurement are retained in
`archive/tier_b_omission_repairs_2026_10_09/phocion-width-measurement/`.
Its `measurement.json` has SHA-256
`4d6ea8b9a2138f888ea77e1d7f7a62159ef0e3dc473cf90a14155d3378648934`.
`phocion_wide_column_fits_without_clipping_source_glyphs` checks the original
fingerprint, full-column resampling, composed stream, canvas width, and rejection
of changed pixels and dimensions. Reviewed horizontal glyph bounds use the
actual source-column scale; the coverage stream uses actual resized strip heights.

# Verification and delivery state

`tier_b_merged_source_lines_restored` executed one ignored test and passed four
selected events / four measured lines. The combined
`tier_b_non_antonius_source_lines_restored` executed one ignored test and passed
all 19 selected events / 21 measured lines across 17 units. Its 17 reports,
reviewed canvases, command log, and production configuration are retained in
`archive/tier_b_omission_repairs_2026_10_09/non-antonius-machine-gate/`.
The 64-file `manifest.json` has SHA-256
`59d79b148718695e8eedebc04b56005668eb4ef7bc523d917a15829b46a56d54`.
The shared verifiers check original pixels, unique retained bands and strips,
source order, complete single-canvas promoted-line coverage, historical regions,
and absence of strips overlapping the confirmed head exclusions. All four merged
candidate canvases were inspected for complete lines and the removed heads.

Passing stage reports and canvases are archived under
`archive/tier_b_omission_repairs_2026_10_09/installed-merged/`; exact report references
and digests are in the event ledger. All 19 non-Antonius events / 21 measured lines
are `installed`. There are 68 permanent protection records: the original 47 plus
21 non-Antonius additions. Antonius remains `pending_repair`, and delivery remains
`in_progress`. Human review of the rebuilt reader is required before restored
status. Existing trial PDFs are not final delivery outputs.

The authenticated portable export contains 135 documents and 412,373 bytes.
Compact serialization keeps the complete projection below its one-MiB limit;
all 131 previously projected semantic documents compare unchanged, and four
passing reports are added. Eight `audit_evidence_` tests authenticated historical
archives, the immutable baseline unit traces, staged reports, and exact portable
projection equality. The ordinary suite passes 415 library tests and nine CLI
tests, with 75 explicitly ignored tests. Formatting and strict all-targets
Clippy pass. The five Python checker tests pass. The evidence-free offline
snapshot runs the same ordinary suite successfully and passes its expected
negative probe for missing historical archives.
