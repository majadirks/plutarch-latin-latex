# Antonius 23-line block — test-install findings

Written 2026-10-09. This records a test-install of the Antonius `SOURCE_BODY_BANDS`
entry proposed in the
[Tier-B repair plan](2026_10_09_14_45_TIER_B_OMISSION_REPAIR_PLAN.md) (Phase 3) and the
[band extraction](2026_10_09_14_30_TIER_B_BAND_EXTRACTION.md), to de-risk the one entry
flagged as uncertain before committing to the full plan. **Result: a plain single-band
protection entry does not restore the block. Antonius needs a different, carefully
verified approach.** The other 21 entries are unaffected by this finding.

## What was tried

Installed exactly the tabulated entry — one `SourceBodyBand` for antonius v2 p562,
`dimensions: (815, 2835)`, `observed: 249..1356`, `protected: 249..1356` (the whole
23-line conclusion is a single `line_bands` band because the inter-line gaps never fall
below `LINE_MIN_INK`) — then did a full offline rebuild of all eight inner readers
(`cargo run --release --bin facsimile -- … --offline`, ~5 min with the render cache warm).

## What happened

**The protection fired, but the block never reached the page.**

- The protection mechanism works: the diagnostic `extract_tier_b_source_bands` now reports
  `survives_render_and_band=true` for `249..1356`, and a direct trace
  (`debug_antonius_p562`) confirms the band is in the unit's final bands
  (`class=Tall`) and a Tall strip `249..1364` is built for it.
- But in the rebuilt `tomus_7_inner.pdf` (which grew 416 → 448 pages), the Antonius
  ending still reads straight from `… Alii in seria inclusam aspidem asser-` to
  `LXXXVII. Antonius ex tribus uxoribus …` — **the 23 lines are absent from their
  reading-order position**, and none of the block's distinctive words (`Archibius`,
  `talentis`, `venenum`, `fistula`, `undequadraginta`, `Enimvero`, `aureo fuso`) appear
  anywhere in the Antonius tail. (The page-count change is signature padding plus the
  disruption from the mis-composed tall strip, not the block appearing.)

## Root cause

`compose_pages` cannot turn one 1107-px `Tall` strip into readable lines when the strip
has **no internal low-ink rows**. The same property that makes `line_bands` merge the 23
lines into one band — the ink profile never clears between lines — also defeats the
composer's fill-split (`lowest_ink_row` / `split_tall_strip`), which relies on low-ink
rows to cut a tall strip into page-sized, line-aligned pieces. With no such rows the
strip is cut at arbitrary budget boundaries (mid-line), and the block does not survive as
legible, correctly separated lines at its position.

Why the ink never clears was probed directly: the block has **no run of ≥6 near-blank
rows**, and even a dense-coverage (≥0.30) separation yields only ~4 groups, not 23. The
marginal section letters (`b`, `c`) down the left edge and the tight leading keep ink
present in every inter-line row. So the block cannot be split into lines by its ink
profile alone.

## Recommended approach (supersedes Phase 3 as written)

A plain protection entry is insufficient. The block must be **split into individual line
bands before composition**, so each line becomes an ordinary strip. Because the gaps are
not detectable automatically, the line boundaries must be supplied explicitly:

1. **Hand-measured line bands at the printed pitch.** The 23 lines sit at a regular pitch
   of ≈48 px across `249..1356` (line centres from the ink profile step ~47–52 px).
   Derive 23 non-overlapping sub-bands (small gaps between them, so the retain merge step
   keeps them separate) and install them as 23 `SOURCE_BODY_BANDS` records that all share
   `observed: 249..1356` but carry one line each in `protected`. The existing
   `retain_known_source_body_bands` pushes every matching record's `protected`, so one
   merged `observed` band can re-insert 23 separate lines — no new mechanism, but the
   boundaries are hand-verified, not detector-derived.
2. **Alternative to investigate:** whether excluding the left-margin section letters from
   the column crop (a `preserve_known_body_crop_edge`-style edge adjustment on p562) lets
   `line_bands` clear the inter-line rows and split the block naturally. If it does, the
   block reduces to ordinary single-line entries. This is cheaper if it works but changes
   column detection on that page, so it needs its own regression check.

Either way, **Antonius must be verified real-eyes in the rebuilt tomus 7** — all 23 lines
present, individually legible, in order — before it is declared done. Until then it stays
an unresolved Tier-A event and does not gate the other 21 repairs, which proceed
independently.

## State left behind

The test-install entry and the scratch `debug_antonius_p562` trace have been reverted from
`src/facsimile/reflow.rs`; the diagnostic `extract_tier_b_source_bands` is kept (the plan
references it). `build_facsimile/` currently holds the test-install rebuild and should be
rebuilt cleanly before use. The plan's Phase 3 is updated to require the split approach
above.
