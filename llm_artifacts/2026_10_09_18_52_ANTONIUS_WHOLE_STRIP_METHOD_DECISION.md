# Context

The Antonius omission remains `pending_repair`. The historical inventory records
23 missing lines; the approved source measurement records 22. This note selects
a source-preserving candidate for permanent installation after the remaining
non-Antonius repairs. It does not declare reader restoration.

The initial handoff's whole-block method preserved the missing prose but did not
fully preserve the approved marginal `c`, and its existing trial split the
following chapter LXXXVII between physical PDF pages 275 and 276. Those trials
are retained. The selected candidate includes the full annotations and prevents
that neighboring split.

# Measured configuration

- Source: volume 2, physical page 562, original raster 1850 × 2835 at 300 dpi.
- Raw detector bounds: [906,1721). Selected production bounds: [895,1721), giving
  a column of 826 × 2835. The 11-pixel extension preserves the complete approved
  `c` annotation; the `b` annotation and all Latin remain source pixels.
- One protection record: source 1, page 562, owner `antonius`, dimensions
  (826,2835), observed [249,1356), protected [249,1356).
- Actual block strip: [249,1364), including the surrounding half-gap. At the
  full-work scale 1.2023952, its dimensions are 993 × 1341.
- Fresh body budget: 1394 pixels. First body budget: 1252 pixels. This is an
  interior source page, so the block can wrap onto a fresh regular body canvas.
- Following chapter cluster: strip [1364,1691), scaled height 393. It also wraps
  whole, preventing its first glyphs from filling the small space below the block.

| Recognition | Column dimensions | Fingerprint region | SHA-256 of original grayscale pixels |
| --- | --- | --- | --- |
| Restored block | (826,2835) | [0,249,826,1364] | `5bd51a4290f3d012bcedf037f3cb8b0d67a4479bd381b3449a8165171b832632` |
| Following chapter | (826,2835) | [0,1364,826,1691] | `5a3cfc2d34e81d528e74ac84ce4c954d91dbbc65f12403579b6d665a8c868980` |

The historical narrow-column block fingerprint is
`ab6e88cedbcb8117631ab2ee3d942eb10392f2da3616000cbd18f6548f5dae95` for
[0,249,815,1364]. It identifies the earlier trial, not the selected complete-marker
configuration.

# Loss stage and method choice

The raw detector emits [249,1356). The production furniture gap filter removes
that interval before Pass A protection. This is the first loss stage of the
missing prose. The original horizontal crop additionally clips the approved
`c` marker before furniture filtering.

The plain-protection configuration uses the original 815-pixel column and
disables Antonius whole-strip recognition. It restores the band through both
protection passes, but its Tall strip is fill-split across canvases 96 and 97.
The first bisected reviewed rectangle is [948,347,1721,404], with scaled stream
interval [130693,130763). Row retention alone therefore does not prove legibility.

Whole-block recognition is selected because 1341 fits the fresh 1394-pixel
budget. The measured crop extension preserves approved source content; it does
not remove annotations or manufacture whitespace. Recognition of the following
393-pixel chapter cluster prevents the adjacent page break from cutting its glyphs.

Explicit disjoint horizontal line bands are rejected. Consecutive measured
rectangles overlap vertically, with minimum vertical gap −11 pixels. The saved
seam measurements report actual row windows and ink counts. Nominal line pitch
cannot turn these overlaps into safe horizontal cuts. A horizontal crop alone
does not fix this packing problem.

All pixels in the selected crop are retained, including the approved section
letters and small source-edge marks. No mask removes in-block source content.
The wider crop must be reviewed alongside the original source during reader
acceptance. Printed spelling, punctuation, hyphens, and source layout remain raster
content; body prose is never retyped.

# Reproducible evidence

`antonius_block_composition_trace` runs baseline, plain, and selected candidate
configurations through the production extractor and composer. Thread-local test
overrides supply temporary protection, optional crop bounds, and recognition mode.
An unwind-safe guard restores every override. The diagnostic records actual Pass A
and Pass B bands and metrics, strip sizes, packed and returned hashes, source
identities/crops, coverage, and source overlays.

The selected run proves original-pixel preservation, unique ordered strip retention,
and complete single-canvas coverage for all 22 missing lines, both annotations, and
the following chapter cluster. Historical neighboring regions keep their historical
policy. The ordinary regressions are
`antonius_whole_block_wraps_without_bisecting_glyphs`,
`antonius_following_chapter_wraps_without_bisecting_glyphs`, and
`antonius_trace_configuration_restores_on_failure`.

The earlier evidence remains under
`archive/tier_b_omission_repairs_2026_10_09/antonius-prior-handoff/`,
`antonius-block-only-candidate/`, and `antonius-narrow-crop-with-safe-chapter/`.
Fresh diagnostic output is under
`tmp/pdfs/inner_tomi_audit_repairs/antonius-composition/`. The authenticated archive
is `archive/tier_b_omission_repairs_2026_10_09/antonius-method-2026_10_09_18_58/`.
Its `manifest.json` authenticates 637 files and has SHA-256
`3ddb1569c8be3eb66b461399360976d303405e9f69c4f7112a9cc332fbe00c25`.
The archive includes full diagnostic canvases, source overlays, the temporary
production configuration, build provenance, the trial PDF, and rendered pages.

# Candidate PDF and installation boundary

The full-work candidate build supplies temporary protection and the exact crop
exception, uses the ordinary offline generator with `--no-compile`, and compiles
`tomus_7_inner.tex` twice with LuaLaTeX. Its provenance records the temporary
production-file bytes, source/unit-table digests, commands, exit codes, build start,
and PDF identity. The production file is restored byte for byte afterward.

Fresh PDF embedding and page rendering are checked against the selected diagnostic
canvas hashes. The fresh 448-page trial PDF has SHA-256
`37a1997212d3d7edf1f29ba44d1bfd7cfaaee06e4ffa808af32beb8f7f442d42`.
Its physical pages 274–277 (folios 266–269) embed candidate canvases 96–99 with
byte-identical grayscale pixels. Page 274 ends at `asser-`; page 275 contains
`vatam dicunt` through `a Cæsare redemit.` with complete `b` and `c` markers;
page 276 begins chapter LXXXVII intact; page 277 preserves the Life's ending.
The wider crop includes small neighboring Greek edge fragments, which remain
explicit source-artifact content for human acceptance before installation.
The candidate PDF is a trial artifact. Permanent protection, the
production crop exception, installed-state evidence, and rebuilt-reader human
acceptance belong to the subsequent installation step. The original immutable
full-work baseline remains unchanged.
