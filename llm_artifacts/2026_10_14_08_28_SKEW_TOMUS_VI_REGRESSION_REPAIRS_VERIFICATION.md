# Skew / Tomus VI regression repairs — verification (2026-10-14)

This records the completed repairs in
`2026_10_13_22_47_SKEW_TOMUS_VI_REGRESSION_REPAIRS_PLAN.md`. The Latin
transcription cache was not changed. Production changes remain scoped to reviewed
source pages and recorded in `tomi_gap_split_repairs_source_trace.json`.

## Passage evidence

| Defect | Before | After rebuild and regression |
|---|---|---|
| Philopoemen, v1 p448 | Tomus III p121 jumped from `superbe Gythium oppugnabat` to `secure agentibus`; Pass B dropped the source band `[330,366)`. | `philopoemen_victory_line_present` finds the band once, on one canvas, between its neighboring lines. The rebuilt Tomus III p121 visibly reads `subito illis nihil tale exspectantibus, sed propter victoriam` between those lines. |
| Romulus, v1 p34 | The source's parallel-column chapter alignment left 271 blank rows between the preceding Latin band and `III. Ceterum`, yielding a large gap. | The post-shear row excision deletes `[1892,2135)`, leaving 28 source rows. `romulus_chapter_break_gap_normal` checks actual composed ink spacing, source order, single placement, and byte-identical pixels in reviewed raw-source regions. In the rebuilt Tomus I p57 (folio 43), the heading follows the prior line with an ordinary gap; the adjacent p58 continues the chapter. No Greek letter fragments are visible at the heading's left edge. |
| Alexander, v2 p231 | The `quidem passus Callisthenem` strip `[2224,2317)` was fill-split across Tomus VI pp86–87. | `tomus6_quidem_not_split` places the entire reviewed strip on one canvas with 53 px of preceding bottom space, below its 96 px bound. The rebuilt line is whole at the top of Tomus VI p87; p86 ends before it. |
| Cleomenes, v2 p398 | A whole 815-row merged strip avoided the `consi-` split but left 741 px of page-bottom space after `agerentur pensitantis` on Tomus VI p394. | The reviewed p398 shear separates lines, and `tomus6_consi_page_reads_continuously` checks each passage in source order on whole canvases. Its measured bottom space is 10 px, below the 116 px bound. Rebuilt p394 reads from `agerentur pensitantis` through `consi-` and `lioque` without the injected gap or a mid-word page break; p395 continues the text. |

The rebuilt images inspected were Tomus I pp57–58, Tomus III p121, and Tomus VI
pp86–87 and 394–395. The source p34 preview and pre-repair composer canvas under
`tmp/pdfs/skew/` show the original chapter alignment gap; the new rendered
previews there have the `phase5_` prefix. The final PDFs are in
`build_facsimile/` (generated, not committed).

## Commands and results

- `cargo fmt` applied, then `cargo fmt --all --check` passed.
- `cargo clippy --all-targets -- -D warnings` passed without warnings.
- `cargo test` passed: 444 library tests and 12 binary tests, zero failures; 117 library tests are ignored by default.
- Each of these ignored tests passed separately with `cargo test --release <name> -- --ignored`: `philopoemen_victory_line_present`, `philopoemen_victory_line_deskewed`, `romulus_chapter_break_gap_normal`, `romulus_chapter_break_deskewed`, `tomus6_quidem_not_split`, `tomus6_quidem_line_complete`, `tomus6_consi_page_reads_continuously`, `tomus6_consi_continuation_gap_closed`, `tomus6_p266_right_glyphs_restored`, `inner_tomi_shears_preserve_latin`, `inner_tomi_whole_strips_preserve_latin`, and `inner_tomi_known_masks_preserve_latin`.
- `tomi_gap_split_ledger_has_expected_shape` and the installed-table checks, including `inner_tomi_row_excisions_match_reviewed_ledger`, passed in the full suite. The first shear pixel-survival run exposed an old test assumption that a sheared page could have no row excision. Its final form maps reviewed source pixels through the recorded deletion; it passes.
- The offline rebuild exited 0 with Poppler `pdftoppm`, `pdfinfo`, `pdftotext`, and `pdfimages` and MiKTeX x64 `lualatex.exe` set through the `PLUTARCH_*` environment variables:

  ```text
  cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
  ```

  The command produced eight volumes. Its local log is `tmp/phase5-skew-rebuild.log`.

## Inner PDF page counts

| Volume | Before | After | Change |
|---|---:|---:|---:|
| Tomus I | 416 | 416 | 0 |
| Tomus III | 416 | 416 | 0 |
| Tomus VI | 480 | 480 | 0 |

The p34 excision closes space within a composed unit; the three earlier repairs
also reflow lines within their volumes. The affected passages and adjacent page
breaks were inspected after the rebuild, and none changes the total inner-page
count.

## Owner review

The rendered previews and automated pixel checks support the repair. The project
owner's visual sign-off against the source scans remains the human faithfulness
gate, particularly for the exact Latin reading and the left gutter on Tomus I
p57. Keep this note and its plan as historical records.
