# Theseus p21 bleed-through excision — verification note

Records the implementation and verification of
[`2026_10_13_08_15_THESEUS_P21_BLEEDTHROUGH_EXCISION_PLAN.md`](2026_10_13_08_15_THESEUS_P21_BLEEDTHROUGH_EXCISION_PLAN.md):
removing the five bled-through lines that stood alone between Theseus §17
(`… tradit agi.`) and §18 (`XVIII. Sorte ducta …`) on v1 source page 21, and
closing the vertical space they occupied.

Branch: `plan/theseus-p21-bleedthrough-excision`.

## What was implemented

- **Mechanism (Phase 2.1).** `SourceRowExcision` / `SOURCE_ROW_EXCISIONS` in
  `src/facsimile/reflow.rs`, a coordinate-pinned table analogous to
  `SOURCE_FURNITURE_MASKS` but which **deletes** rows (via `splice_out_rows`, a
  single-pass copy of the surviving spans) rather than whitening them, closing the
  gap. `excise_known_source_rows` runs in `render_and_band` immediately after
  `mask_known_source_furniture`, before column detection.
- **Installed record.** One entry: `source 0, page 21, top 2012.0/2975.0,
  bottom 2237.0/2975.0` — deletes rows `[2012, 2237)` (225 px) of the 2975-row
  raster.
- **Trace support (Phase 2.2).** `audit_traced_unit` now mirrors the
  mask→excise→detect order and records the excised interval as the page's
  `deleted_rows` (original-source coordinates), so `RetainedSourcePageTrace::map_row`
  shifts every row below the slab and the existing pixel/line survival helpers
  validate the compacted output.

## Measured source evidence (Phase 1.1)

From `theseus_p21_bleedthrough_trace` at production scale (crop 789 × 2975):

| quantity | value |
| --- | --- |
| last real §17 line band (`… tradit agi.`) | `[1971, 2008)`, ink/row ≈ 273 |
| bleed slab (faint) | `[2022, 2234)`, ink/row ≈ 26–58 |
| first real §18 line band (`XVIII …`) | `[2240, 2275)`, ink/row ≈ 323 |
| median inter-line gap | 7 px |
| pre-fix §17→§18 gap | 232 px |
| excision `[top, bottom)` | `[2012, 2237)` (225 px) |
| residual gap after excision | 7 px (one median inter-line gap) |

No real-glyph row lies inside `[2012, 2237)` (real lines end at 2008, resume at
2240; every intervening band is faint bleed).

## Tests

All run with the Poppler and MiKTeX overrides; `CARGO_TARGET_DIR=target`.

| test | kind | result |
| --- | --- | --- |
| `source_row_excision_splices_configured_rows` | ordinary | pass |
| `source_row_excision_absent_or_degenerate_is_identity` | ordinary | pass |
| `repair_trace_maps_skipped_pages_and_deleted_rows` | ordinary | pass |
| `inner_tomi_masks_match_reviewed_ledger` | ordinary | pass (masks unchanged) |
| full ordinary `cargo test` | ordinary | 433 passed, 0 failed, 97 ignored |
| `backlog_theseus_p21_bleedthrough_excised` | ignored | pass (≈ 59 s) |
| `theseus_p17_analysis_keeps_original_latin_pixels` | ignored | pass (page 17 byte-identical) |

`cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings` are
clean.

The regression test `backlog_theseus_p21_bleedthrough_excised` works in the
band/trace/pixel domain only (no `production_body_scale()` / recompose), so it runs
in ~1 min rather than >10. It has teeth by construction: with the excision record
removed, `deleted_rows` is `[]` (fails the `assert_eq!`) and the residual gap is
~232 px (fails the ≤ 20 px bound).

### No disturbance elsewhere (Phase 2.4)

The excision is page-21-only and is **not** a mask, so `SOURCE_FURNITURE_MASKS` and
the mask-ledger tests are untouched (page 21 carries no mask; the ordinary
`inner_tomi_masks_match_reviewed_ledger` stays green). `theseus_p17_analysis_keeps_original_latin_pixels`
confirms page 17 is byte-identical, so the earlier page-17 (I1) Theseus repair and
its canvases — all composed before page 21 — are unaffected.

## Visual confirmation (Phase 3)

A targeted `--unit theseus --no-compile` build produced 40 composed body canvases.
The §17/§18 transition lands on `facsimile_theseus_16.png`: `… honorem horum
tradit agi.` is followed, at one ordinary inter-line gap, directly by `XVIII. Sorte
ducta quum accepisset ex Prytaneo Theseus …`. The five bled lines are gone. The
faint specks that remain to the right of `tradit agi.` and left of `XVIII` lie on
the real text rows (not in the excised slab), so they are preserved per the policy
"where the bleeding overlaps text, it may stand."

## Full rebuild and page-count ripple

The full offline build
(`cargo run --release --bin facsimile -- --v1 … --v2 … --out build_facsimile/
--offline`, with `PLUTARCH_LUALATEX` and the Poppler overrides) completed cleanly:
`Volumes: 8`, `Output: build_facsimile/`, all **16** PDFs freshly written
(8 `tomus_N_inner.pdf`, 8 `tomus_N.pdf`). The build log has no `error`, `Fatal`,
`panic`, `failed`, or `lualatex not run` lines; every tomus compiled its inner and
imposed wrapper.

- `theseus`: 20 source pages → **40** trim pages (same as the targeted unit build).
- `tomus_1_inner.pdf`: **416** pages (= 13 × 32-page signatures).
- Volume split: unchanged at **8** volumes.

**Ripple.** The excision removes 225 px (≈ 270 px at the 1.2023952 body scale) from
one Theseus page — under 0.2 of the ≈ 1429 px body budget — so no whole body canvas
is dropped: Theseus stays at 40 canvases and the §17/§18 transition keeps its
physical location. In the rebuilt `tomus_1_inner.pdf` the transition is on **pdf
page 30 (folio 16)**, the same page that carried the five bled lines before the fix
(rendered at the start of this work). The before/after comparison at that page:
pre-fix, five faint bled lines separated `tradit agi.` from `XVIII. Sorte ducta …`;
post-fix, they are gone and the two run at one ordinary inter-line gap, matching
`facsimile_theseus_16.png` from the unit build. No totals were forced.

## Status

All plan phases (1.1, 2.1–2.4, 3.1) are complete and verified. Generated
`build_facsimile/` PDFs and body PNGs are local-only (gitignored); the committed
changes are the mechanism and tests in `src/facsimile/`, the plan, and this note.
