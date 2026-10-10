# Step 3.1 handoff — Antonius block diagnosis & method selection

Branch: `plan/inner-tomi-audit-repairs`. Task: implement Step 3.1 of
`llm_artifacts/2026_10_09_14_45_TIER_B_OMISSION_REPAIR_PLAN.md`, commit + push, do
NOT proceed to 3.2. **Not yet committed.**

## Verdict (measured, not assumed)
Antonius is **RESOLVED via method (a) whole-block recognition** — not unresolved.

Key numbers from the real pipeline (production scale 1.2024):
- The 23-line block merges into ONE raw detector band `[249,1356]` on p562
  (production column `[906,1721]`, dims `(815,2835)`). It is dropped at the **Pass-A
  furniture gap filter** (`drop_furniture`) — this is the first loss stage.
- A plain protection entry (`observed=protected=[249,1356]`) re-adds it, but the strip
  is classified **Tall**, cut-point interval `[249,1364]`, scaled height **1341 px**.
- `body_budget = 1394 px`, `first_body_budget = 1252 px` (my earlier hand-calc of 1335
  was wrong; trust the runtime). **1341 ≤ 1394**, so the block fits one body canvas.
- With only the plain band it fill-splits across unit canvases 96/97, bisecting line
  `[948,347,1721,404]` (first failed boundary).
- Method (b) crop-edge: inapplicable (horizontal crop can't fix vertical overlap).
- Method (c) explicit per-line bands: **rejected** — consecutive reviewed rectangles
  overlap vertically (min overlap **−11 px**) and every inter-line seam carries ink, so
  no horizontal cut is glyph-safe.
- Method (a): recognize the block as a whole source strip so it wraps whole onto a
  fresh canvas. **Fingerprint** of region `[0,249,815,1364]` = sha256
  `ab6e88cedbcb8117631ab2ee3d942eb10392f2da3616000cbd18f6548f5dae95`.

Candidate build (temporary protection band, since reverted) confirmed end-to-end:
`antonius 51 src pages -> 99 trim pages`; `facsimile_antonius_97.png` is the whole
23-line block, legible, in order; canvas 96 ends "...Alii in seria inclusam aspidem
asser-" and canvas 97 continues "vatam dicunt... a Caesare redemit." then LXXXVII —
split word continuous, all previously-missing words present (fistula, venenum,
Enimvero, undequadraginta, Archibius, talentis). `tomus_7_inner.pdf` compiled with
LuaLaTeX twice (exit 0), **448 pages** (416→448 is the 32-page signature padding, as the
findings predicted); the block embeds whole; the Antonius Life ends correctly at PDF
page 277 (folio 269).

## Working-tree changes (uncommitted, 3 files)
1. `src/facsimile/audit_repair_tests.rs`:
   - `antonius_block_composition_trace` (ignored diagnostic): establishes loss stage,
     measures a/b/c, emits baseline/plain/candidate JSON under
     `tmp/pdfs/inner_tomi_audit_repairs/antonius-composition/{baseline,plain,candidate}/`
     and the whole-strip fingerprint record; writes a disposition only if unresolved
     (it isn't, so none written / stale ones removed).
   - `antonius_block_fixture` + ordinary regression
     `antonius_whole_block_wraps_without_bisecting_glyphs` (recognition exact/off-by-one,
     wrong dims, tampered pixel, whole-wrap on a partial canvas, too-small-budget split).
2. `src/facsimile/reflow.rs`: refactored `is_known_whole_source_strip` into a table and
   added the Antonius case `((815,2835), Band{249,1364}, [0,249,815,1364], "ab6e88ce…")`.
   The earlier TEMP-ANTONIUS-CANDIDATE `SOURCE_BODY_BANDS` trial band **has been reverted**.
3. `src/facsimile/fixtures/inner_tomi_audit_repair_source_trace.json`: added
   `source_fingerprints["antonius_whole_strip"]` (byte-exact png_base64, 207,888 chars).
   Inserted textually to preserve non-ASCII source bytes (do NOT re-serialize this file
   with Python `json.dump` default — it escapes/garbles the source-faithful strings).

Tests already passing: `antonius_whole_block_wraps_without_bisecting_glyphs`,
`merged_strip_wraps_at_complete_line_boundary`, `audit_source_fingerprints_match_reviewed_regions`,
`unaffected_ordinary_line_packing_is_unchanged`, and the ignored
`antonius_block_composition_trace` (ran green, ~60s).

## REMAINING to finish Step 3.1 (then commit + push)
1. **Write the dated method-decision note** in `llm_artifacts/` (the explicit pre-3.2
   deliverable). Must include: exact tuple (dims (815,2835), strip band 249..1364,
   fingerprint ab6e88ce…), scale 1.2024, budgets 1394/1252, scaled strip 1341; first
   loss stage = `pass_a_furniture_filter`; first unreadability stage = compose Tall
   fill-split; a/b/c evaluation with the −11px overlap and seam-ink evidence; artifact
   decisions (section letters b/c preserved as approved content; block kept whole);
   the candidate build/compile result (canvas 97 whole, 448-pp PDF embeds, split-word
   "asser-/vatam" continuity); named passing tests
   (`antonius_block_composition_trace`, `antonius_whole_block_wraps_without_bisecting_glyphs`).
   Do not reference plan step numbers inside source/comments (AGENTS.md), but the note
   itself may.
2. Run `cargo fmt --all`; `cargo fmt --all --check`;
   `cargo clippy --all-targets -- -D warnings`;
   `cargo test known_source_body_band`; `cargo test` (full ordinary suite);
   re-run the ignored `antonius_block_composition_trace` and the ordinary
   `antonius_whole_block_wraps_without_bisecting_glyphs`.
3. Commit (branch is not main) + push. Suggested message: "Diagnose Antonius block loss
   and select the whole-strip composition method (step 3.1)".

## Scope boundary (do NOT do — these are Step 3.2)
- Permanent `SOURCE_BODY_BANDS` protection band + `closing_body_bands.json` record.
- Ledger status transition `pending_repair -> installed` for `miss_antonius_562_20869`,
  its passing_evidence/human_review, and portable-evidence refresh.
- Running `antonius_reviewed_lines_survive_composition_whole` to a pass and the full
  rebuild + gating human read of all 23 lines in the rebuilt tomus 7.
Antonius ledger event is unchanged (status `pending_repair`); delivery still
`in_progress`, `unresolved_event_ids: []`. No disposition was created (outcome resolved).

## To reproduce the candidate build (if re-verifying)
Temporarily re-add to the TOP of `SOURCE_BODY_BANDS` in reflow.rs, build, then revert:
```
SourceBodyBand { source:1, page:562, unit_id:"antonius", dimensions:(815,2835),
  observed:Band{top:249,bottom:1356}, protected:Band{top:249,bottom:1356} },
```
Build: `cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2
Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline --no-compile` (~3.5 min warm),
then `cd build_facsimile && lualatex -interaction=nonstopmode tomus_7_inner.tex` ×2.
Tooling: python3.13, Tesseract `C:/Program Files/Tesseract-OCR/tesseract.exe`,
pdftoppm `C:/Users/majad/Downloads/poppler/poppler-25.12.0/Library/bin/pdftoppm.exe`,
lualatex `C:/Users/majad/AppData/Local/Programs/MiKTeX/miktex/bin/x64/lualatex.exe`.
`build_facsimile/` is gitignored (trial PDFs/images not committed).

## Note on 22 vs 23 lines
Event `miss_antonius_562_20869`: `missing_line_count` 23 (historical) / `measured_line_count`
22 (approved inventory correction, see `2026_10_09_16_23_TIER_B_SOURCE_LINE_COUNT_DECISION.md`).
The trace works on the 22 measured rectangles; the block is referred to as "23-line".
