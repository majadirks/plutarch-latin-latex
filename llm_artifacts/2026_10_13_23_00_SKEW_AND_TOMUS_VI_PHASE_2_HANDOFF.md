# Skew and Tomus VI Phase 2 Handoff

## Context

The user asked for Phase 2 of `2026_10_13_SKEW_AND_TOMUS_VI_PHASE_1_HANDOFF.md`. This note records the implementation state so another LLM can resume. The source transcription cache has not been changed. Preserve existing unrelated edits and temporary artifacts when continuing.

## Current state

Phase 2 implementation work is in the shared working tree, but final full-suite verification and the corpus review are not confirmed. No commit or push has been made.

The source shear guards are in `src/facsimile/reflow.rs` for v1 source 0 pages 34 (Romulus) and 448 (Philopoemen), and v2 source 1 page 266 (Caesar). The Philopoemen slope is positive (`+0.03048780487804878`); the earlier negative sign was contradicted by the paired-gap evidence and caused the bands to remain merged. The phase 1 handoff and plan were amended with this diagnosis.

The implementation also adds sheared-coordinate furniture masks, maps the existing Philopoemen raw-coordinate folio mask through the shear, and adjusts guarded crop edges for all three affected pages. Caesar p266 has a fingerprint guarded strip-scale cap to retain its rightmost glyphs. A guarded whole-strip repair was added for v2 source 1 page 398 (Cleomenes). The JSON repair ledger and its validation tests cover the shear, sheared masks, and whole-strip record. The audit path now applies the known shear before masking/excision and measures deleted rows in post-shear coordinates.

Ignored visual/pixel regressions were added for the reviewed page cases. Relevant source files include:

- `src/facsimile/reflow.rs`
- `src/facsimile/audit_repair_tests.rs`
- `src/facsimile/inner_tomi_gap_split_tests.rs`
- `src/facsimile/tomi_gap_split_repairs_source_trace.json`
- `llm_artifacts/2026_10_13_SKEW_AND_TOMUS_VI_PHASE_1_HANDOFF.md`
- `llm_artifacts/2026_10_13_SKEW_AND_TOMUS_VI_PHASE_2_PLAN.md` (check exact name in the directory before relying on it)

Do not edit the Latin transcription cache or normalize the source text.

## Verification completed

- `cargo fmt` was run; `cargo fmt --all --check` passed after the implementation edits.
- `cargo clippy --all-targets -- -D warnings` passed after resolving the helper argument-count lint.
- Focused tests passed for the shear ledger, sheared-mask ledger, raw-mask mapping, whole-strip ledger, and TOMI gap-split ledger shape.
- Ignored visual regressions passed for Romulus p34 and Philopoemen p448 (`cargo test deskewed -- --ignored`), and Caesar p266 (`tomus6_p266_right_glyphs_restored`). Other ignored checks for the complete quidem line, consi continuation, and whole-strip Latin preservation passed in earlier runs; they should be repeated against the final tree where practical.
- A prior full `cargo test` before the final edits passed 440 library tests and 12 binary tests.

## Unfinished verification and next actions

1. Check `git status --short` and preserve pre-existing user changes, especially `BACKLOG.md`, `prompts.txt`, and the existing `tmp/` artifacts. Additional shear preview PNGs may be present in `tmp/pdfs/skew/`.
2. Run `cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings` again if the tree changed after the last recorded pass.
3. Run the renamed ignored test `cargo test tomus6_quidem_line_complete -- --ignored` and repeat the focused final-tree ignored regressions as needed.
4. Run a fresh full `cargo test` to completion. The last full run was interrupted while polling; it had reached the 555-test library suite and showed passing progress, but the process could no longer be queried (session id 22083 was unavailable). Do not report it as passed.
5. Consider rerunning the potentially long `cargo test inner_tomi_known_masks_preserve_latin -- --ignored`; an earlier run was manually interrupted before the final raw-mask mapping adjustment.
6. Inspect the generated shear previews under `tmp/pdfs/skew/` and, where practical, regenerate/inspect representative `.tex` or PDFs. This is human visual review; no completed final corpus PDF review is recorded.
7. Review the Phase 2 plan and handoff for remaining checklist items, update the plan's completion state only as appropriate, then report the verified result. Do not commit or push unless the existing session instructions authorize it; the Phase 1 handoff requested commits/pushes, so that authorization appears to exist, but first finish verification and preserve unrelated changes.

## Important diagnostic detail

For Philopoemen p448, the corrected paired gap near x=1083 is `[822,840)` (not `[868,886)`, which came from the following line); the paired interval near x=1739 is `[841,873)`. This supports the positive shear sign. The source trace fixture records expected post-shear dimensions and crop/band measurements.

For Caesar p266, shear changes the detected crop and requires the guarded extension to x=1730 plus the guarded strip-scale cap. Regression assertions verify survival of source pixels in five right-edge line intervals. The transcription remains unchanged.

## Phase 2 verification completed (resume session, 2026-10-13)

Verification was finished against the final working tree. The transcription cache
and all pre-existing unrelated edits (`BACKLOG.md`, `prompts.txt`, `tmp/`) were
preserved; no commit or push was made.

- `cargo fmt --all --check` — clean.
- `cargo clippy --all-targets -- -D warnings` — clean, no warnings.
- Full `cargo test` — **442 library + 12 binary passed, 0 failed, 113 ignored**
  (re-run after the fixture fix below; same result).
- Ignored release regressions all pass: `romulus_chapter_break_deskewed`,
  `philopoemen_victory_line_deskewed`, `tomus6_p266_right_glyphs_restored`,
  `tomus6_quidem_line_complete`, `tomus6_consi_continuation_gap_closed`,
  `inner_tomi_shears_preserve_latin`, `inner_tomi_whole_strips_preserve_latin`,
  and `inner_tomi_known_masks_preserve_latin`.

**One stale fixture fixed.** `inner_tomi_known_masks_preserve_latin` initially
failed at `reflow.rs:6291`: the pre-existing H1 Philopoemen p448 folio-mask record
in `src/facsimile/fixtures/inner_tomi_repair_source_trace.json` carried
`expected_first_band: [300,1501]` — the giant *pre-shear* merged band. Because
`audit_traced_unit` now shears p448 (shear→mask→excise→crop), the retained first
body band is `[380,415]`, which the tomi shear fixture independently records as the
second entry of p448's `expected_bands` (`[[330,366],[380,415]]`, the `[330,366]`
head being dropped by `drop_furniture`). The record's `expected_first_band` was
updated to `[380,415]`. `expected_pass_a_bands` was left unchanged because the
Pass-A check still runs on the masked-but-unsheared crop and legitimately remains
`[300,1501]`.

**Still outstanding (Phase 3, human-in-the-loop).** No offline facsimile rebuild
was run and no final PDF/visual corpus review is recorded. The shear preview PNGs
under `tmp/pdfs/skew/` exist but have not been visually signed off here. Phase 3
(offline `--offline` rebuild with MiKTeX/Poppler, page-count table, dated
`…_SKEW_AND_TOMUS_VI_REPAIRS_VERIFICATION.md`, then commit + branch PR) remains.
