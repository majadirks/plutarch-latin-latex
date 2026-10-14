# Skew & Tomus VI facsimile repairs — verification note (2026-10-13)

Covers the five reviewed defects from
`llm_artifacts/2026_10_13_19_00_SKEW_AND_TOMUS_VI_REPAIRS_PLAN.md`. Phase 2
installation (commit `6657d87`) and the automated verification below are
complete. **Human visual corpus sign-off is still outstanding** (see the last
section). The Latin transcription cache was not touched.

## Dispositions installed

| # | Defect | Source | Reader | Disposition | Key values |
|---|--------|--------|--------|-------------|-----------|
| 1 | Romulus chapter-break skew | v1 p34 | Tomus I p58 | `shear` | slope `-0.016079632465543646`, `x0=1400`, shifts `[-23,9]`, `top_pad=23`; + sheared-coord gutter mask |
| 2 | Philopoemen victory-line skew | v1 p448 | Tomus III p121 | `shear` | slope `+0.03048780487804878`, `x0=1400`, shifts `[-18,43]`, `top_pad=18`; + sheared-coord Greek-bleed mask; pre-existing H1 folio mask re-mapped through shear |
| 3 | reported `quidem` straddle | v2 p231 | Tomus VI pp86–87 | **no repair** | line already complete on p86; completeness regression only |
| 4 | angled v2 p266 right-edge clip | v2 p266 | Tomus VI p158 | `shear` + guarded crop edge | slope `+0.03745928338762215`, `x0=1350`, shifts `[-20,51]`, `top_pad=20`; crop extended to x1730, capped by `source_strip_scale`; + sheared-coord Greek-bleed mask |
| 5 | `consi-` continuation gap | v2 p398 | Tomus VI pp394–395 | `whole_strip` | column `818×2835`, strip `[1022,1837)` (980 scaled px < 1394 budget) |

All three shears, their sheared-coordinate furniture masks, the whole-strip
entry, and the matching `repairs` ledger records are installed and cross-checked
against the Rust tables by `inner_tomi_shears_match_reviewed_ledger` and
`inner_tomi_whole_strips_match_reviewed_ledger`.

## Per-defect evidence (from passing ignored regressions)

- **Romulus p34** — `romulus_chapter_break_deskewed`: the `III. Ceterum` heading
  and its preceding line survive once with no left-gutter fragments after the
  shear straightens the baselines and the gutter mask removes the divider
  fragments.
- **Philopoemen p448** — `philopoemen_victory_line_deskewed`: the merged
  pre-shear bands (`[300,1501]`,`[1527,2663]`) resolve into individual straight
  lines (post-shear first body band `[380,415]`); the victory line and its
  continuation survive once and the reader gap is no longer inflated relative to
  neighbouring source inter-line gaps.
- **Caesar p266** — `tomus6_p266_right_glyphs_restored`: exact source pixels for
  the five clipped endings (`sive`, `praeter`, `(3)`, `auferro,`, `idem`) are
  present once; line order and the widened crop width (`x1730`) are correct.
- **`quidem` p231** — `tomus6_quidem_line_complete`: `quidem passus
  Callisthenem` is complete and unique across pp86–87; no guard installed.
- **`consi-` p398** — `tomus6_consi_continuation_gap_closed`: the `consi-` /
  `lioque` strip wraps whole; the gap no longer breaks across the reader page
  boundary, and the scaled strip height (980 px) fits the 1394-px page budget.
- **Pixel faithfulness** — `inner_tomi_shears_preserve_latin`,
  `inner_tomi_whole_strips_preserve_latin`, and
  `inner_tomi_known_masks_preserve_latin` confirm reviewed Latin source pixels
  survive the sheared/whole-strip/masked pipeline at their mapped coordinates.

## Commands and results (final tree, commit `6657d87`)

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean, no warnings.
- `cargo test` — **442 library + 12 binary passed, 0 failed, 113 ignored** (77.8s).
- Ignored release regressions (source PDFs present): `romulus_chapter_break_deskewed`,
  `philopoemen_victory_line_deskewed`, `tomus6_p266_right_glyphs_restored`,
  `tomus6_quidem_line_complete`, `tomus6_consi_continuation_gap_closed`,
  `inner_tomi_shears_preserve_latin`, `inner_tomi_whole_strips_preserve_latin`,
  `inner_tomi_known_masks_preserve_latin` — all pass.
- Offline rebuild (exit 0, 16 PDFs, 8 volumes):
  ```
  cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf \
    --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
  ```
  with `PLUTARCH_LUALATEX` = MiKTeX x64 `lualatex.exe` and the Poppler
  `PLUTARCH_PDFTOPPM/PDFINFO/PDFTOTEXT/PDFIMAGES` binaries set.

### Stale fixture fixed during verification

`inner_tomi_known_masks_preserve_latin` failed on first run: the pre-existing H1
Philopoemen p448 folio-mask record in
`src/facsimile/fixtures/inner_tomi_repair_source_trace.json` carried the
**pre-shear** merged band `[300,1501]` as `expected_first_band`. Because
`audit_traced_unit` now shears p448 (shear→mask→excise→crop), the retained first
body band is `[380,415]` — independently recorded as the second entry of the
tomi shear fixture's p448 `expected_bands` (`[[330,366],[380,415]]`, the
`[330,366]` head dropped by `drop_furniture`). The field was corrected to
`[380,415]`; `expected_pass_a_bands` was left unchanged because that check runs
on the masked-but-unsheared crop and legitimately remains `[300,1501]`. Full
suite re-run after the fix: same 442/12, 0 failed.

## Inner-PDF page counts (before → after rebuild)

| Volume | Before | After | Change |
|--------|--------|-------|--------|
| tomus_1_inner | 416 | 416 | — |
| tomus_2_inner | 448 | 448 | — |
| tomus_3_inner | 416 | 416 | — |
| tomus_4_inner | 128 | 128 | — |
| tomus_5_inner | 384 | 384 | — |
| tomus_6_inner | 480 | 480 | — |
| tomus_7_inner | 448 | 448 | — |
| tomus_8_inner | 192 | 192 | — |

**Explanation of the (absence of) change.** None of the five repairs add or
remove reader pages. The three shears and the two sheared-coordinate masks fix
*within-page* line banding and crop so the straightened Latin lines and the
right-edge glyphs land correctly; the `consi-` whole-strip wraps an
adjacent-line pair onto a single page instead of fill-splitting it, which
reflows the composer's greedy fill without shifting the volume's total page
count; and `quidem` is a no-repair completeness check. The faithfulness
improvements are therefore at the pixel/line level (proven by the ignored
regressions above), not at the page-count level. No `source_spacing_retained`
disposition was used.

## Outstanding — human visual sign-off

The faithfulness gate for this phase is a visual comparison of the five rebuilt
reader passages and their adjacent page breaks against the source scans:

- Tomus I p58 (Romulus `III. Ceterum` heading + left gutter) vs v1 p34.
- Tomus III p121 (`… sed propter victoriam` + continuation + left margin) vs v1 p448.
- Tomus VI pp86–87 (`quidem passus Callisthenem` complete on p86) vs v2 p231.
- Tomus VI p158 (five right-edge endings + left margin) vs v2 p266.
- Tomus VI pp394–395 (`consi-`/`lioque` continuation) vs v2 p398.

Source and reader preview PNGs are under `tmp/pdfs/skew/`. The rebuilt PDFs are
in `build_facsimile/` (gitignored). This note records the automated result; the
visual sign-off and the branch PR remain to be done by the human owner.
