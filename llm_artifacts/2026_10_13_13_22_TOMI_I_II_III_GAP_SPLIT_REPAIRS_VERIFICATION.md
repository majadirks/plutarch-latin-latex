# Tomi I–III gap/split/short-ending repairs — verification

Dated record for the repairs in
`llm_artifacts/2026_10_13_10_12_TOMI_I_II_III_GAP_SPLIT_REPAIRS_PLAN.md`,
implemented on branch `fix/tomus-1-gaps-split-shortending`. Seven of the twelve
reviewed defects are fixed, verified, and committed; five remain (documented
below). The three Tomus VI items are deferred to a separate plan per the owner.

## Repairs installed and verified

All coordinates are v1 (`source == 0`), full-page 300-DPI raster. Masks are
`SOURCE_FURNITURE_MASKS` entries (normalized), with reviewed records in
`src/facsimile/fixtures/tomi_gap_split_repairs_source_trace.json`. Excisions are
`SOURCE_ROW_EXCISIONS`. The merge is a `REVIEWED_SHORT_ENDINGS` entry.

| ID | Source | Mechanism | Pinned coordinates |
| --- | --- | --- | --- |
| G1 | p23 (2007×2993) | left divider mask | x [1047,1065), full height |
| G2 | p30 (1984×2998) | left divider mask | x [936,955), full height |
| S1 | p41 (1990×2974) | left divider mask | x [1037,1072), full height |
| T2b | p315 (1883×2961) | right scan-edge mask | x [1880,1883), full height |
| G4 | p36 (1975×2978) | row excision | rows [2414,2596) → ~12 px residual |
| G5 | p37 (2004×2990) | row excision | rows [2304,2490) → ~12 px residual |
| M1 | comparatio_solonis_et_poplicolae | short-ending merge | final canvas 1004×49, sha256 111bae67… |

The four bridged pages (G1/G2/S1/T2b) each had a thin vertical ink streak (the
Greek/Latin divider or a right scan edge) at the column-crop edge that added
>3 ink to nearly every row, merging the whole Latin column into one giant band
(e.g. p23 `[804,2028)` h=1224; p41 `[1371,2530)` h=1159) that `compose` then
split badly. Masking the streak restores normal ~33-px banding. G4/G5 are
parallel-typesetting alignment blanks (the Latin column finishes a chapter early
while the longer Greek column continues) that are meaningless in a Latin-only
reader; the slabs are ink-free (max 2–3 per row) and were closed to one
inter-line gap.

### Automated verification

- `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`: clean.
- `cargo test --lib` (full ordinary suite): 434 passed, 0 failed.
- Ignored production regressions (source PDFs + 300-DPI renders):
  - `inner_tomi_known_masks_preserve_latin` — faithfulness gate over every masked
    page (historical + the four new): every `preserved_latin_regions` box survives
    the mask byte-identically. PASS (407 s).
  - `inner_tomi_masks_match_reviewed_ledger` — each installed mask matches a
    reviewed ledger rectangle. PASS.
  - `backlog_tomi_divider_masks_restore_banding` — after masking, p23/p30/p41/p315
    re-band into ≥30 lines with no band taller than 60 px. PASS (3.8 s).
  - `backlog_romulus_chapter_gaps_excised` — p36/p37 slabs deleted, bracketing
    Latin lines survive once byte-identical, residual gap ≤ 20 px. PASS (72 s).
  - `backlog_solon_poplicola_short_ending_merges` — merge returns true, one canvas
    fewer, both stacked regions byte-identical, within text block. PASS (11 s).
    (Composed at the frozen production scale 1.2023952 to skip the >15 min
    whole-corpus `production_body_scale()` sampling.)

### Full rebuild and visual verification

`cargo run --release --bin facsimile -- --v1 … --v2 … --out build_facsimile/
--offline` with `PLUTARCH_LUALATEX` set: exit 0, all sixteen inner+imposed PDFs
freshly written, LuaLaTeX compiled each (no "lualatex not run"). Inner page
counts: T1 416, T2 448, T3 416, T4 128, T5 384, T6 480, T7 448, T8 192 (each a
32-page-signature multiple). T1 stayed 416: M1 removes one body page, which the
signature padding absorbs as a trailing blank.

Eyeballed in the rebuilt PDFs (page numbers are the rebuilt inner pages):

- **G1** (T1 p33): the page is now full — text flows past "… excepisse Ariadnam,"
  through "… argenteum unum, alterum æneum. (9)"; the early break is gone.
- **G2** (T1 p48): full through "… Theseum in vinculis servavit." into
  "XXXII. Interea Menestheus Petei …".
- **S1** (T1 p70): "… genethliacus : cui scientiæ ita se spe-" renders whole and
  legible; no horizontal split.
- **G4** (T1 p62): "… Hinc adolescentibus magnum nomen." is followed by one normal
  gap then "VII. Exorta aliquando …".
- **G5** (T1 p64): "… remotis arbitris de his ageret." → one gap → "VIII. At
  Faustulus …".
- **T2b** (T2 p258): "… operam suam nava-/vit, ut eum perinde …" flows with a
  normal line gap.
- **M1** (T1 p295/296): "… jungeret, et omnem belli apparatum tolleret." now sits
  at the foot of p295; p296 is blank.

## Remaining (not fixed here)

These are a harder class than a single edge streak and were left for follow-up
(see the plan's "Implementation status" section for the measured evidence):

- **G3 (p34)** and **T3a (p448)**: the source page is **skewed** (baselines not
  level), so the column bands into one block regardless of edge masking; p448 also
  has a bled Greek column and an over-wide detected crop. Need a per-page deskew or
  a whole-strip guard.
- **T3b (p451)**, **T3c (p462)**: horizontal splits from a skew/tight bridge at a
  page foot; the merged band should wrap whole via `is_known_whole_source_strip`
  (the fingerprint is scale-independent, so this is tractable without the slow
  compose path — measure the column dims / band / crop / sha256 and confirm the
  scaled band fits a budget).
- **T2a (p289→p290)**: the "O ma-/ter" split straddles the source page break; the
  right-edge streak (abs x≈1905) is in the margin outside the Latin crop and the
  two specks are too sparse to band, so the cause is not yet a simple streak mask —
  the diagnostic now covers p289/p291 to pin it.

## Tomus VI (separate plan, deferred)

Per the owner: pdf 86-87 "quidem" straddling split; v2 p266 skew losing right-edge
glyphs + left Greek bleed; gap on pdf 394 after "… summae rerum consi-". To be
addressed in their own plan after this branch lands.
