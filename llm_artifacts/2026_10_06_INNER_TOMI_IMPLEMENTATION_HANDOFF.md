# Inner tomi repairs — implementation handoff (for the next LLM)

You are picking up a partially-completed implementation of
[the minimal repairs plan](2026_10_05_INNER_TOMI_MINIMAL_REPAIRS_PLAN.md). The diagnosis phase
(Phase 1) is substantially done and **18 repairs are human-confirmed and frozen**. Your job is
**Option 1: implement every confirmed repair (Phases 2–3), rebuild all eight tomi (Phase 5), then
re-assess the still-open gap findings against the cleaned output.** Do **not** re-diagnose or
re-ask the confirmed items — they are locked by human review.

Work on branch **`plan/inner-tomi-minimal-repairs`** (already checked out; 11 commits of diagnosis
are pushed). Commit and push after each step, as the original task requested.

## Read these first (decision artifacts — all on disk)

1. **`src/facsimile/fixtures/inner_tomi_repair_source_trace.json`** — the **authoritative numeric
   ledger**. Its `repairs[]` array holds every confirmed repair with exact coordinates, source/page,
   adjacent prose, preservation boundary, and (for dedup) `implementation` notes. **Transcribe from
   here; do not re-measure.** 13 `disposition:"mask"` records + 2 `disposition:"duplicate_page"`
   records are present. Each `human_confirmed` field cites the RFI and date.
2. **`llm_artifacts/2026_10_05_INNER_TOMI_REPAIR_RFIS.md`** — the RFI doc. Its "Session state and
   resumption guide" has the tooling (poppler paths, render/OCR commands), the folio→source-page
   method, the OCR-anchored source-page table, and the **RFI status tracker** at the bottom (the
   quick index of what is CONFIRMED vs still open). Use it to drive the gap re-assessment later.
3. **`llm_artifacts/2026_10_05_INNER_TOMI_MINIMAL_REPAIRS_PLAN.md`** — the implementation contract.
   Phases 2–5 below map to it. Honor its constraints verbatim (next section).
4. **`llm_artifacts/2026_10_05_INNER_TOMI_VISUAL_REVIEW_FINDINGS.md`** — the original findings (IDs).
5. Already-committed Phase 1.1 infra lives in `src/facsimile/reflow.rs` tests:
   `RetainedSourcePageTrace` + `map_row` + `retained_page` +
   `repair_trace_maps_skipped_pages_and_deleted_rows` + the ledger-shape test
   `inner_tomi_repair_ledger_has_expected_shape` + three ignored `inner_tomi_*_source_trace` tracers.

## Hard constraints (from the plan — do not violate)

- **Never erase Latin.** Masks are furniture-only; every confirmed rectangle has a measured
  preservation boundary in the ledger (mask stops above the first body line / below the last Latin
  line). Do not widen them. Do not retype/normalize any Latin.
- Preserve the existing `SOURCE_FURNITURE_MASKS` entries (v1 pages 56, 233, 235, 240, 284) and all
  existing passing tests. Do not change raw render caches, source PDFs, transcription caches, global
  thresholds, body scale, the W3 notice page, or unrelated unit rows.
- Offline only: no `--retag`, `--review-lines`, paid vision, new CLI flags, new deps, or a
  generalized override framework. Masks run on owned images, so no cache deletion is needed.
- Tests must read expectations from the committed ledger; they must not regenerate-and-accept their
  own expected values.

---

# Step A — Phase 2: de-duplicate prose (D1, D2)

Both are **exact whole-page duplicates**; the human chose to keep the first occurrence. Ledger
records: `finding_ids:["D1"]` and `["D2"]`, `disposition:"duplicate_page"`.

Add a small private helper in `src/facsimile/reflow.rs`:

```rust
fn skip_known_duplicate_source_page(unit_id: &str, source: usize, page: u32) -> bool
```

with explicit, identity-scoped matches (return `false` for every unrelated identity):

- **D1** — unit `"comparatio_sertorii_et_eumenis"`, source `1`, pages **103 and 104**
  (content-identical to p101/p102; p101 lives in the Eumenes unit and p102 is the shared tail).
  After skipping, p102 `...magno in` joins p105 `pretio habitus...` = *"magno in pretio habitus"*.
- **D2** — unit `"agis"`, source `1`, pages **369 and 370** (identical to p367/p368). After
  skipping, p368 `...ut unius suffragii numero` joins p371 `excellerent qui antiquabant legem...`.

Call it in `unit_body_strips` **before rendering and before pushing to either `pages` or
`source_pages`**, so those two vectors stay aligned (Pass A). Do not add JSON schema or CLI config.
Do not touch `facsimile_units.json` — this is a duplicate exclusion, **not** a boundary correction,
so no `eumenes`/comparison `start_page`/frac/pin changes are needed.

Note on scale: `mod.rs::global_scale_for` renders each page in `start_page..=end_page`
independently (it still samples 103/104 and 369/370), so the global body scale is **unchanged** by
this exclusion. Record the baseline and final scale anyway (ledger `scale` block) and confirm no
material change.

Tests (per plan §2.1/§2.2):
- ordinary `known_duplicate_page_exclusions_are_identity_scoped` (assert true only for the four
  configured identities, false otherwise).
- ignored `eumenes_comparison_prose_is_complete_and_unique` — load the real unit table, run the
  comparison unit, assert the retained source sequence excludes 103/104, p102→p105 join present,
  one comparison opening, complete Eumenes ending; compare retained glyph regions to raw source
  pixels using the committed `RetainedSourcePageTrace::map_row` contract (not output page count).
- ignored `agis_repeated_passage_survives_once` — assert the Agis sequence excludes 369/370, one
  occurrence of folios 952/953, and the p368→p371 join.
- Do **not** add `eumenes_comparison_ranges_use_corrected_boundary` or
  `corrected_comparison_pin_survives_redetection` (no boundary/pin change was selected — record this
  omission).

# Step B — Phase 3.1: extend `SOURCE_FURNITURE_MASKS` (13 masks)

Transcribe every `disposition:"mask"` record's `mask_rectangles` from the ledger into new
`SourceFurnitureMask` entries (`source`, `page`, `left/top/right/bottom`). Each entry's comment
should name the furniture and preservation boundary (do not cite plan phases). The records to add:

| Finding | source | page | rectangles |
| --- | ---: | ---: | --- |
| W1 | 0 | 287 | watermark (page tail) |
| W1 | 0 | 288 | running head `MARCIUS CORIOLANUS.` + folio 275 |
| W2/G4 | 0 | 299 | watermark (page tail) |
| O1 | 0 | 182 | `CAMILLUS.` + folio 169 |
| O1B | 0 | 281 | `ΓΑΙΟΣ ΜΑΡΚΙΟΣ.` + folio 268 (ch. XXII page) |
| O1C | 0 | 290 | `MARCIUS CORIOLANUS.` + folio 277 |
| O1D/G8B | 0 | 298 | `TIMOLEON.` + folio 285 |
| G5 | 0 | 308 | `TIMOLEON.` + folio 295 |
| O2 | 0 | 550 | `LYSANDER.` + folio 537 |
| O3 | 1 | 441 | `ΔΗΜΟΣΘΕΝΗΣ.` + folio **and** far-right vertical rule (2 rects) |
| O4 | 1 | 539 | `ΑΝΤΩΝΙΟΣ.` + folio 1116 |
| O6 | 1 | 337 | `ΚΑΤΩΝ.` + folio 934 (no right rule) |
| O5 | 0 | 530 | Marius end-of-Life ornament (so the terminal strip is inkless → no furniture-only canvas) |

Reuse the existing `apply_source_furniture_mask` / `mask_known_source_furniture` call path in
`render_and_band` (masks apply to the owned page image before column detection). Do not apply a
global threshold or erase any component that includes Latin.

Tests (per plan §3.1):
- ignored `inner_tomi_known_masks_preserve_latin` — driven by the ledger: for each mask record,
  assert mask identity, that all **unmasked** source pixels are unchanged, that the adjacent Latin
  (the ledger's `adjacent_prose.below`/the first retained band) survives, and that the furniture is
  gone. Reuse `assert_real_mask_preservation`-style helpers already in `reflow.rs`.
- ordinary `inner_tomi_masking_restores_separate_line_bands` — a synthetic divider-connected page
  showing the band-merge failure is fixed and every synthetic body pixel is preserved.
- Extend the existing mask unit tests (`known_source_furniture_*`, `source_furniture_mask_*`) for
  the new identities without copying fixtures.

# Step C — Phase 3.2: whitespace compaction — **likely NOT needed**

The confirmed watermarks (W1 p287/p288, W2 p299) are **page tails** (the next Latin line opens the
following source page), so once masked the tail simply is not copied — no interior compaction. The
leaked headers are at page tops above the first body line. **After Step B, run the real
extraction/composition for the watermark and header pages and measure residual copied blank rows.**
If every verified unwanted interval is already gone, mark Phase 3.2 **unnecessary**, add no
`SourceWhitespaceExclusion` type/table/tests, and record the measured evidence. Only implement 3.2
if a post-mask interior interval demonstrably remains (none is expected from the confirmed set).

# Step D — Phase 4.1: furniture-only canvases (O5) and the sliced line (O8B)

- **O5** is handled by the ornament mask in Step B (the terminal region becomes inkless, so
  `compose_pages`' ink floor drops the canvas). Add ignored
  `marius_terminal_furniture_emits_no_body_canvas` asserting Marius's complete ending is retained
  and the ornament page contributes no body canvas. Also add
  `timoleon_and_brutus_have_no_furniture_only_body_canvases` once the gap work (below) is settled.
- **O8B** (Timoleon sliced line, II/240–241) is **still open** — it needs the production
  composition trace, not a page crop. First rebuild (Step E) and check whether the header/divider
  masks already fix it; if not, follow plan §4.1 (prefer restoring real line bands upstream; no
  broad new split heuristic, no font-scale change). Add `timoleon_reported_page_break_preserves_
  whole_lines` when resolved.

# Step E — Phase 5: rebuild all eight tomi and verify

Follow plan §5.1 exactly. Use a workspace-local target dir if the external one is inaccessible:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
cargo fmt --all; cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Then the full facsimile build (offline), per plan §5.1, with the MiKTeX LuaLaTeX path. Render the
repaired passages with Poppler and compare against the source scans. Verify the required ignored
tests individually (`-- --ignored --nocapture`): the three `inner_tomi_*_source_trace` tracers,
`eumenes_comparison_prose_is_complete_and_unique`, `agis_repeated_passage_survives_once`,
`inner_tomi_known_masks_preserve_latin`, `marius_terminal_furniture_emits_no_body_canvas`, and the
existing `reported_divider_pages_preserve_latin`, `reported_watermark_pages_preserve_adjacent_lines`,
`fabius_shared_page_header_is_excluded_from_both_units`, `coriolanus_source_284_header_is_excluded`,
`otho_final_source_pages_survive_reflow_once`, `cimon_lucullus_final_source_page_survives_reflow`.

Write `llm_artifacts/2026_10_05_INNER_TOMI_MINIMAL_REPAIRS_VERIFICATION.md` (commands/results,
source mappings, before/after copied heights, the dedup decisions, preserved landmarks, retained
items, and any unavailable checks). Update the plan's "Measured repair decisions" in place without
renumbering.

---

# Step F — RE-ASSESS the still-open gaps (after the rebuild)

These were deliberately deferred because several will shrink or vanish once Steps A–D are applied
(the leaked headers/dividers/watermarks that *created* some gaps are now masked, and the D1/D2
interruptions are gone). **Do this against the rebuilt `build_facsimile/tomus_N_inner.pdf`, not the
raw source.** For each, render the inner-PDF page beside the source page, decide keep (genuine
chapter/paragraph spacing) vs remove (copied scan space / divider), and record the disposition in
the ledger + RFI doc. These are **human-judgement calls** — surface them to the user with evidence;
do not erase anything near Latin on your own authority.

Open items (see the RFI tracker for IDs/pages):
- **G1** (Timoleon II/219–220) — source page ~296–297, a prominent left vertical divider rule in a
  big blank; exact page still to be matched visually (OCR anchor failed). The divider is an O7 rule.
- **G2** (Timoleon II/235–238) — near-blank pages; `dam sublime se cursu proripuit...` is on source
  p305. Confirm word-continuity before removing blank/divider.
- **G3** (Brutus VIII/130–132) — divider-only page; `partem secutus.` on source p626, `subli-`
  split precedes. Confirm the continuation is preserved.
- **G6** (Pyrrhus III/175) — mid-sentence gap at the p470→471 seam.
- **G7/G8** (Theseus I/36, I/46) — source p23/p28 are **densely packed with no copied blank**, so
  the output gaps are likely chapter spacing or reflow effects, not removable scan space. Verify on
  rebuilt output; likely "retain as genuine spacing" with a recorded reason.
- **O7** standalone dividers (V/195–196, VI/358, VII/21) — source pages unresolved; each divider
  must be proven separable from all adjacent Latin before masking (dividers fused to glyphs are
  retained and recorded).

Note: VI/329's right rule (O6) was **confirmed absent** by the user; do not hunt for it.

# Quick status snapshot (2026-10-06)

- **Frozen & ready to implement (18):** W3, O8 (editorial retain); 13 furniture masks
  (W1×2, W2, O1, O1B, O1C, O1D, G5, O2, O3+rule, O4, O6); O5 ornament mask; D1 & D2 dedup.
- **Open (judgement, after rebuild):** G1, G2, G3, G6, G7, G8, O7 standalone dividers, O8B.
- Ledger `repairs[]` = 14 records (W1 is two). RFI tracker rows marked CONFIRMED match these.
