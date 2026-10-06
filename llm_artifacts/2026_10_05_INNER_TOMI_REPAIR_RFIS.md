# Inner tomi repair — Requests for Information (RFIs)

Companion to [the minimal repairs plan](2026_10_05_INNER_TOMI_MINIMAL_REPAIRS_PLAN.md) and
[the visual review findings](2026_10_05_INNER_TOMI_VISUAL_REVIEW_FINDINGS.md). It
enumerates every judgement a human must supply before a source region is erased or a
duplicate occurrence is dropped. The plan gates those edits on human verification
because a wrong rectangle permanently deletes genuine Latin, and OCR can locate a
passage but cannot prove preservation.

## Interview protocol

For each RFI:

1. **I measure first.** I render the candidate source page at the production DPI
   (300), locate the passage by OCR-anchoring the quoted Latin, draw the proposed
   erase rectangle (or identify the duplicate/boundary), and save an evidence crop.
2. **I pose the question** with that crop and the adjacent Latin landmarks.
3. **You judge** — confirm / correct / reject. Free-text nuance is welcome.
4. **I record** your answer in the "Decision" field here and, once frozen, in the
   numeric ledger `src/facsimile/fixtures/inner_tomi_repair_source_trace.json`.

Nothing is erased until its RFI is marked **CONFIRMED** here.

## Conventions

- **Inner-PDF page** = one-based physical page of the current `build_facsimile/tomus_N_inner.pdf`
  (what the findings cite). **Reader folio** = the printed page number visible on the
  reader page. **Source folio** = the printed page number in the 19th-c. scan.
  **Source page** = one-based physical page of `Ploutarchou_Bioi_vN.pdf` — the only
  coordinate the generator uses, and the one I must confirm.
- Source index: `0` = `Ploutarchou_Bioi_v1.pdf`, `1` = `Ploutarchou_Bioi_v2.pdf`.
- The source folio and source page differ by a drifting offset (inserted plates), so
  candidate source pages below are **pre-measurement estimates, UNCONFIRMED**; part of
  what I show you is the OCR anchor proving the page is the right one.
- Mask rectangles are normalized half-open `[left, top, right, bottom)` on the full
  source raster, matching `SOURCE_FURNITURE_MASKS` in `src/facsimile/reflow.rs`.

## Unit → source map (from `facsimile_units.json`, factual)

| Unit | id | source | start_page | shares last page with |
| --- | --- | ---: | ---: | --- |
| Theseus | `theseus` | 0 | 14 | Romulus (33) |
| Camillus | `camillus` | 0 | 167 | Pericles (195) |
| Marcius Coriolanus | `marcius_coriolanus` | 0 | 268 | Comp. Alc./Cor. (292) |
| Timoleon | `timoleon` | 0 | 295 | Aemilius Paulus (318) |
| Caius Marius | `caius_marius` | 0 | 497 | Lysander (531) |
| Lysander | `lysander` | 0 | 531 | Sylla (552) |
| Pyrrhus | `pyrrhus` | 0 | 470 | Caius Marius (497) |
| Cato Minor | `cato_minor` | 1 | 309 | Agis (363) |
| Demosthenes | `demosthenes` | 1 | 429 | Cicero (446) |
| Antonius | `antonius` | 1 | 513 | Comp. Dem./Ant. (563) |
| Eumenes | `eumenes` | 1 | 88 | Comp. Sert./Eum. (102) |
| Comp. Sertorii et Eumenis | `comparatio_sertorii_et_eumenis` | 1 | 102 | Agesilaus (106) |
| Agis | `agis` | 1 | 363 | Cleomenes (379) |
| Brutus | `brutus` | 1 | 596 | Comp. Dionis/Bruti (626) |

## Supporting artifacts

- **This session (committed trace helper output, gitignored):**
  `tmp/pdfs/inner_tomi_repairs/` — representative before-repair column crops and trace
  JSON: `duplicate.*` (Eumenes p88, Comp. p102, Agis p363), `timoleon_coriolanus.*`
  (Coriolanus p268, Timoleon p295), `remaining.*` (Brutus p596).
- **Prior review renders cited in the findings** (under `tmp/pdfs/inner_review/`,
  `review12/`, `review34/`, `review56/`; may need regeneration). Per-item pointers below.
- **Per-item crops, generated 2026-10-05** under `tmp/pdfs/inner_tomi_repairs/` (regenerate with
  the scratchpad `render_rfis.py`): `rfi-03-*` (annotated boxes, confirmed); `*-overview.png`
  (downscaled full pages) for RFI-01 (p100–105), 02 (p366–370), 05 (p181–182), 06 (p286–287),
  07 (p290), 08 (p298), 09 (p308), 10 (p549), 11 (p441), 12 (p538–539), 13 (p337,347), 16
  (p303–305), 17 (p624–626), 18 (p470–471), 19 (p23,28), 21 (p529–530); plus full-res `*-top.png`
  header crops for RFI-05/07/10/11. Still to pin a source page: RFI-04 (W2 ≈ s300), RFI-15 (G1),
  RFI-20A (O8B). Annotated candidate mask boxes are drawn per item at confirmation time.

---

# Session state and resumption guide

**This section exists so the interview can resume in a fresh context with no prior memory.**
Read it, then continue from the first `pending` RFI in the status tracker at the bottom.

## Interview progress (as of 2026-10-05)

- **CONFIRMED (frozen in ledger):** RFI-22 (W3 retain), RFI-23 (O8 retain + diagnose),
  RFI-03 (W1, s0 p287+p288), RFI-05 (O1, s0 p182), RFI-07 (O1C, s0 p290, no divider).
- **BOX DRAWN, awaiting confirmation** (proposed normalized coords in each RFI; box PNGs
  `rfi-NN-...-box.png`): RFI-08 (O1D s0 p298), RFI-09 (G5 s0 p308), RFI-10 (O2 s0 **p550**),
  RFI-11 (O3 s1 p441 + right rule), RFI-12 (O4 s1 p539), RFI-13 (O6 s1 **p337**; right rule TBD).
  All six are the same top-running-head-leak pattern, verified visually to clear the first body line.
- **NEEDS MORE PREP:** RFI-04 (W2, pin via inner PDF), RFI-06 (O1B, re-anchor to Coriolanus ch.
  XXII — the "XXII" OCR match was a false hit inside "XXXIII"), RFI-15 (G1, pin page).
- **JUDGEMENT-TYPE (no single box; evidence = overviews):** RFI-14 (O7 divider sweep),
  RFI-16/17/18/19 (gaps: keep vs remove), RFI-20A (O8B sliced line = pagination), RFI-21 (O5
  ornament page: confirm no Latin), RFI-01/02 (D1/D2 duplication: read-both and choose).
- **Proposed order:** confirm the six box-drawn masks → finish O6 right rule, O1B, W2 →
  O7 dividers → gaps (15–19) → O8B → O5 → duplication (01, 02).
- **Workflow per item:** I measure (resolve source page, render, draw candidate box), post the
  crop + the question via the `AskUserQuestion` tool, the human judges, I record the decision
  here and freeze confirmed numbers into the ledger. Nothing is erased until its RFI is CONFIRMED.

## Branch and files

- Branch `plan/inner-tomi-minimal-repairs`. Commit after each interview round; push.
- **Numeric ledger (production fixture):** `src/facsimile/fixtures/inner_tomi_repair_source_trace.json`
  — append one record per source page to `repairs[]` as each RFI is confirmed. The ledger-shape
  test `inner_tomi_repair_ledger_has_expected_shape` (in `src/facsimile/reflow.rs` tests) requires
  `finding_ids, unit_id, source, source_page, disposition, preservation_boundary` non-null.
- **Diagnostic infra (Phase 1.1, committed):** `RetainedSourcePageTrace` + `map_row` +
  `retained_page` + `repair_trace_maps_skipped_pages_and_deleted_rows` in `src/facsimile/reflow.rs`.
- Evidence crops: `tmp/pdfs/inner_tomi_repairs/` (gitignored).

## Tooling (Windows)

- Source PDFs in repo root: `Ploutarchou_Bioi_v1.pdf` (= source **0**), `Ploutarchou_Bioi_v2.pdf`
  (= source **1**). Production render DPI = **300** (`BODY_DPI`).
- Poppler (Windows paths, for Windows Python / PowerShell):
  `C:/Users/majad/Downloads/poppler/poppler-25.12.0/Library/bin/pdftoppm.exe` and `.../pdftotext.exe`.
  In Git Bash, `/mingw64/bin/pdftotext` also works but Windows Python cannot exec that path.
- Render a page to grayscale PNG:
  `pdftoppm -gray -r 300 -f N -l N -png Ploutarchou_Bioi_vX.pdf <prefix>`.
- Dump a page's OCR text layer: `pdftotext -f N -l N Ploutarchou_Bioi_vX.pdf -`.
  **The text layer exists but mangles `æ`/`œ` and some letters to `�`** — anchor on ASCII-safe
  fragments (e.g. `de Eumene`, `inspicere posset`, `partem secutus`, not `Græcum`/`mœrore`).
- Scripts used this session live under the scratchpad: `anchor.py` (source-page resolver),
  `render_rfis.py` (batch render + crop + watermark scan).

## Folio → source-page offset (method)

Running heads print the **source folio**; the generator uses the **physical source page**. The
offset drifts (inserted plates), so verify per region by OCR-anchoring the quoted Latin. Measured:
**folio = physical − 13** near Camillus/Coriolanus/Timoleon (v1); **−12** at Lysander (folio 537 =
physical 549). To find a page: dump running heads over a span and read the folio, or grep the text
layer for an ASCII-safe anchor phrase.

## Resolved source pages (OCR-anchored 2026-10-05) — authoritative

This table supersedes the per-RFI "candidate source page (UNCONFIRMED)" lines below.

| RFI | Finding | src | Source page(s) | Anchor / note |
| --- | --- | ---: | --- | --- |
| 03 | W1 | 0 | **287** (watermark), **288** (header `MARCIUS CORIOLANUS.`+folio 275) | CONFIRMED |
| 04 | W2, G4 | 0 | **to pin via inner PDF** | "Digitized by Google" is at the bottom (~rows 2854–2925) of **every** Timoleon page and is normally dropped. W2 leaked on one page (like Fabius 233/235). Source ~300 by reader-folio interpolation was WRONG (p300 is full text, watermark far below last line). Pin by rendering `build_facsimile/tomus_2_inner.pdf` p227, reading its Latin, and OCR-anchoring to the source page; then mask that page's bottom watermark. |
| 05 | O1 | 0 | seam **181→182**; header `CAMILLUS.`+folio 169 atop **182** | `...jam exsul,`(181) / `neque nos cives`(182) |
| 06 | O1B | 0 | **re-anchor** | the "XXII" match was a false hit inside "XXXIII" (p281/286/287 are ch. XXXIII, Greek head `ΓΑΙΟΣ ΜΑΡΚΙΟΣ`/274). Find the real Coriolanus ch. XXII page (reader II/187 ≈ earlier than s288), then mask its top Greek running head. |
| 07 | O1C | 0 | **290** | `RIOLANUS.`+damaged folio 277 + right divider (folio 277=phys 290) |
| 08 | O1D, G8B | 0 | **298** | `EON.`+folio 285 (folio 285=phys 298); `…cum` then gap |
| 09 | G5 | 0 | **308** | `EON.`+folio 295 (folio 295=phys 308) + divider |
| 10 | O2 | 0 | **550** | `Haliartii perhibent,` ends p549; leaked header is `LYSANDER.`+folio 537 atop **p550** (folio 537=phys 550). p549's own head is `ΛΥΣΑΝΔΡΟΣ`/536, not the O2 leak. |
| 11 | O3 | 1 | **441** | between `…moerore suo` and `conficiat, sed et levandus…` (both on 441) |
| 12 | O4 | 1 | **538→539** | split `Eo impe-`(538)/`trato, hostes…`(539) + Greek head/divider |
| 13 | O6 | 1 | **337** | top Greek head `ΚΑΤΩΝ.` (→`ΩΝ.`) + folio 934; body `nonnulli etiam amici Pompeii…` starts row 204; partial right-margin rule (~494px) still to locate |
| 14 | O7 | 0/1 | listed pages | divider sweep; e.g. I/36 ≈ s0 p23, I/46 ≈ s0 p28 |
| 15 | G1 | 0 | **UNRESOLVED** | Timoleon `ci`/`num` split; scan s0 296–318 for the big interior gap |
| 16 | G2 | 0 | **303–305** | near-blank pages before resume `dam sublime se cursu proripuit…` (on 305) |
| 17 | G3 | 1 | **~624–626** | `partem secutus.` on 626; `subli-` split precedes |
| 18 | G6 | 0 | seam **470→471** | `…strepitu aquae`(470) / `verba non percipientibus…`(471) |
| 19 | G7, G8 | 0 | **23** (G7), **28** (G8) | G7 before ch. XXI after `et mendacia esse.`; G8 before ch. XXVIII after `…exstant.` |
| 20A | O8B | 0 | **UNRESOLVED** | Timoleon sliced line (reader folios 232–233); composition/pagination artifact, needs the production trace not a page crop |
| 21 | O5 | 0 | Marius final ornament page before Lysander (start 531); inspect **529/530** | |
| 01 | D1 | 1 | **100–105** | dup: `inspicere posset`&`de Eumene` on 102 **&** 104; `Deinde impetu facto` on 101 **&** 103 |
| 02 | D2 | 1 | **366–370** | Agis opening `non modo luxum, in quo felicitatem vitae` repeats on **367 & 369** |

## How a confirmed mask is measured

1. Render the source page at 300 DPI; note its exact raster dims (each page differs).
2. Row ink profile (`pixel value < 110` = ink) to find the last Latin row above and first body
   row below the artifact, guaranteeing the mask cannot clip text.
3. Gray watermark detection: pixels with value in ~`[130, 232]` (lighter than black text) in the
   central/bottom band give the watermark bbox.
4. Propose a normalized half-open `[left, top, right, bottom)` rectangle enclosing only furniture,
   draw it on an annotated crop, get human confirmation, freeze into the ledger.

---

# Group A — Duplication (read-both decisions)

These cannot be reduced to a rectangle; they need you to read both occurrences and
choose the retained sequence.

## RFI-01 — D1: Eumenes / comparison reading order (Tomus V, source 1)

- **Inner-PDF:** V/194–201. **Units:** `eumenes` (start 88), `comparatio_sertorii_et_eumenis`
  (start 102), next `agesilaus` (106). **Candidate source pages (UNCONFIRMED):** Eumenes
  body ~88–101; comparison body ~102–105.
- **Observed (findings D1):** 194 ends Eumenes `...Græcum mare inspicere posset.` 195 opens
  **SERTORII CUM EUMENE COMPARATIO** `Hæc sunt quæ de Eumene...`, last line `...ducibus que-`.
  196–198 return to Eumenes (196 starts `rentes. Deinde impetu facto...`; 198 repeats the
  Eumenes ending already on 194) yet carry the comparison running head. 199 repeats the
  comparison opening; comparison runs to its true end on 201. So Eumenes material **and**
  the comparison opening each appear twice, and 195→196 does not continue the sentence.
- **What I will produce:** source renders of the Eumenes tail and both comparison openings,
  with OCR anchors for `posset.`, `Hæc sunt quæ`, `que-`/`rentes.`, and the second
  comparison opening; a line-by-line table of what is unique vs. repeated.
- **Generated (2026-10-05):** `rfi-01-D1-p100-overview.png` … `rfi-01-D1-p105-overview.png`
  (source pages 100–105; `inspicere posset`+`de Eumene` on 102 & 104, `Deinde impetu facto` on 101 & 103).
- **I need you to confirm:** (a) the correct reading order; (b) which physical occurrence
  of the comparison opening to keep; (c) that the `que-` → `rentes.` join belongs to the
  comparison (not Eumenes), so the retained sequence loses no unique line; (d) whether the
  fix is a boundary correction (fixing `eumenes`/comparison `start_page`/fracs + pin) or a
  whole-/partial-page duplicate exclusion.
- **Decision:** _pending._

## RFI-02 — D2: Agis repeated passage (Tomus VII, source 1)

- **Inner-PDF:** VII/17–24 (first occ. 17–20, second 21–24). **Unit:** `agis` (start 363),
  next `cleomenes` (379). **Candidate source pages (UNCONFIRMED):** ~363–370 region.
- **Observed (findings D2):** 17 and 21 both open `non modo luxum, in quo felicitatem vitæ...`
  and both carry chapter VIII; the same progression repeats across the next three pages. The
  second occurrence has heavier right-side divider marks. Repeated textual content from
  differing image appearances — not necessarily byte-identical rasters.
- **What I will produce:** source renders of both occurrences with OCR anchors on the shared
  opening and chapter VIII; a comparison of the start/end seams of each occurrence.
- **Generated (2026-10-05):** `rfi-02-D2-p366-overview.png` … `rfi-02-D2-p370-overview.png`
  (Agis opening `non modo luxum, in quo felicitatem vitae` repeats on source p367 **&** p369).
- **I need you to confirm:** (a) which occurrence to retain (cleaner image vs. divider marks);
  (b) that the retained occurrence's beginning and end join correctly to the surrounding
  distinct passages; (c) whether this is repeated whole source pages (→ duplicate-page
  exclusion) or a source-selection/boundary error (→ fix the boundary instead).
- **Decision:** _pending._

---

# Group B — Watermark & leaked furniture ("my page + my box are right" — yes/no each)

For each: I show the source render with the proposed whiten-rectangle outlined and the
nearest Latin line above/below. You confirm the box covers **only** the artifact (Google
watermark / running head / folio / divider) and touches no glyph or pale type.

## RFI-03 — W1: Coriolanus watermark + `RIOLANUS.`/folio 275 (Tomus II/199)

- **Unit:** `marcius_coriolanus` (source 0, 268–292). **Source folio cited:** 275.
  **Candidate source page (UNCONFIRMED):** ~275–285.
- **Observed (W1):** large gray **Digitized by Google** below the main prose block, then a
  copied running-head fragment `RIOLANUS.` and source folio **275**, then another prose line.
- **Prior artifacts:** `review12/` II/199; `inner_review/user_t2_near-199.png`–`near-203.png`,
  `user_t2_191.png`.
- **Confirm:** the watermark rectangle, and separately the running-head/folio rectangle,
  each exclude the prose line that follows.
- **Measured (OCR-anchored):** folio = physical − 13 near Coriolanus. Source folio 275 =
  **physical page 288** (running head + folio, 1980×2984); the watermark is the bottom
  margin of **physical page 287** (folio 274, 1900×2975). Last Latin on 287 ends row 2568;
  watermark rows ~2721–2934. On 288 header ink ends row 262, blank gap 263–278, first body
  line `junctis, ad Volumniae...` starts row 279.
- **Proposed masks (normalized half-open):** p287 watermark `[0.6000, 0.9042, 0.8579, 1.0000)`;
  p288 header+folio `[0.0909, 0.0687, 0.8737, 0.0932)`. Evidence:
  `tmp/pdfs/inner_tomi_repairs/rfi-03-p287-watermark.png`, `rfi-03-p288-header-folio.png`.
- **Decision:** **CONFIRMED — both masks** (2026-10-05). Frozen in the ledger for source 0
  pages 287 & 288; to be added to `SOURCE_FURNITURE_MASKS` in Phase 3.1.

## RFI-04 — W2 / G4: Timoleon watermark in the gap (Tomus II/227)

- **Unit:** `timoleon` (source 0, 295–318). **Candidate source page (UNCONFIRMED):** ~227-region
  of the source — to be OCR-anchored on the two lines above and the continuation below.
- **Observed (W2/G4):** two lines near the top, then a large empty interval containing a gray
  **Digitized by Google**, then continuing prose. Interior gap + watermark together.
- **Prior artifacts:** `review12/` II/227. **Generated (2026-10-05):**
  `rfi-04-W2-p299-overview.png`, `rfi-04-W2-p300-overview.png`, `rfi-04-W2-p301-overview.png`
  (none is the W2 page — see tracker note; pin via inner PDF p227).
- **Confirm:** the watermark rectangle; and whether the surrounding empty interval is a source
  page tail or an interior gap (decides whether Phase 3.2 whitespace compaction is needed —
  see RFI-20).
- **Decision:** _pending._

## RFI-05 — O1: Camillus `LUS.`/folio 169 (Tomus I/371)

- **Unit:** `camillus` (source 0, 167–195). **Source folio cited:** 169.
- **Observed (O1):** running-head fragment `LUS.` + source folio **169** intrude between
  `...non enim neque is jam exsul,` and `neque nos cives sumus...`.
- **Prior artifacts:** `inner_review/user_t1_371.png`. **Generated (2026-10-05):**
  `rfi-05-O1-p181-overview.png`, `rfi-05-O1-p182-overview.png`, `rfi-05-O1-p182-top.png`,
  `rfi-05-O1-p182-box.png` (annotated mask).
- **Confirm:** the `LUS.`/folio rectangle sits between those two Latin lines and clips neither.
- **Measured:** source 0 **page 182** (1974×2985); running head `CAMILLUS.` + folio `169` ink
  rows 226–257; blank gap 258–286; first body line `neque nos cives sumus...` starts row 287.
- **Proposed mask (normalized):** `[0.0861, 0.0710, 0.9625, 0.0951)`.
- **Decision:** **CONFIRMED — one mask** (2026-10-05). Frozen in the ledger for source 0 page 182.

## RFI-06 — O1B: Coriolanus Greek running head before ch. XXII (Tomus II/187)

- **Unit:** `marcius_coriolanus` (source 0). **Observed (O1B):** a Greek running-head fragment
  (user read “ARKIOS”) inside a large gap immediately before chapter XXII.
- **Prior artifacts:** `inner_review/user_t2_187.png`. **Generated (2026-10-05):**
  `rfi-06-O1B-p286-overview.png`, `rfi-06-O1B-p287-overview.png`.
- **Confirm:** the Greek header rectangle contains no Latin and the gap handling (RFI-20) is
  separate; measure the associated gap.
- **Decision:** _pending._

## RFI-07 — O1C: Coriolanus `RIOLANUS.`/folio 277 + right divider (Tomus II/204)

- **Unit:** `marcius_coriolanus` (source 0). **Source folio cited:** 277.
- **Observed (O1C):** `RIOLANUS.` + damaged folio **277** after the top prose line and before
  its continuation; a vertical divider survives at the right.
- **Prior artifacts:** `inner_review/user_t2_204.png`. **Generated (2026-10-05):**
  `rfi-07-O1C-p290-overview.png`, `rfi-07-O1C-p290-top.png`.
- **Confirm:** the header/folio rectangle and the right-divider rectangle each exclude the
  top line and its continuation.
- **Measured:** source 0 **page 290** (1974×2986); top running head `MARCIUS CORIOLANUS.`+folio
  `277` ink rows 220–252; gap 254–281; first body line `ter, ut mecum egisti!...` starts row 282.
  No divider ink in the right margin.
- **Proposed mask (normalized):** `[0.0861, 0.0710, 0.9625, 0.0931)`. Annotated: `rfi-07-O1C-p290-box.png`.
- **Decision:** **CONFIRMED — one header/folio mask; no divider** (2026-10-05). Frozen in the
  ledger for source 0 page 290. The finding's "right divider" has no separable rule (edition
  reference letters / scan edge).

## RFI-08 — O1D / G8B: Timoleon `EON.`/folio 285 in the gap (Tomus II/222)

- **Unit:** `timoleon` (source 0). **Source folio cited:** 285.
- **Observed (O1D/G8B):** prose ends mid-page `...cum`; a large blank interval with a thin
  divider, then `EON.` + folio **285** near the bottom.
- **Prior artifacts:** `inner_review/user_t2_222.png`. **Generated (2026-10-05):**
  `rfi-08-O1D-G8B-p298-overview.png`.
- **Confirm:** the `EON.`/folio rectangle and divider fragment exclude the `...cum` line and
  the next page's continuation; classify the blank interval (RFI-20).
- **Measured:** source 0 **page 298** (1990×2982) is a full page; top running head `TIMOLEON.`+folio
  `285` ink rows 202–235; gap; body starts ~267. (The `...cum` + blank interruption seen in output
  II/222 is the output placement; the leak to mask is this top header.)
- **PROPOSED mask (normalized):** `[0.08593, 0.0644, 0.9628, 0.0842)`. Box: `rfi-08-O1D-G8B-p298-box.png`.
  Awaiting confirmation. (Whitespace/divider = RFI-20/O7, separate.)
- **Decision:** _pending (box drawn)._

## RFI-09 — G5: Timoleon `EON.`/folio 295 + divider (Tomus II/245)

- **Unit:** `timoleon` (source 0). **Source folio cited:** 295.
- **Observed (G5):** large top blank, then `EON.` + folio **295** immediately above continuing
  prose; a divider fragment in the blank region.
- **Prior artifacts:** `review12/` II/245; `inner_review/user_t2_245.png` (180 DPI).
  **Generated (2026-10-05):** `rfi-09-G5-p308-overview.png`.
- **Confirm:** header/folio + divider rectangles exclude the continuing prose line; classify
  the blank (RFI-20).
- **Measured:** source 0 **page 308** (1998×2982); top running head `TIMOLEON.`+folio `295` ink
  rows 202–235; body starts ~264.
- **PROPOSED mask (normalized):** `[0.0856, 0.0644, 0.9630, 0.0835)`. Box: `rfi-09-G5-p308-box.png`.
  Awaiting confirmation.
- **Decision:** _pending (box drawn)._

## RFI-10 — O2: Lysander `NDER.`/folio 537 (Tomus III/337)

- **Unit:** `lysander` (source 0, 531–552). **Source folio cited:** 537.
- **Observed (O2):** `NDER.` at left and folio **537** at right below the line ending
  `Haliartii perhibent,`.
- **Prior artifacts:** `review34/` III/337. **Generated (2026-10-05):**
  `rfi-10-O2-p549-overview.png`, `rfi-10-O2-p549-top.png`.
- **Confirm:** the head/folio rectangles exclude `Haliartii perhibent,` and the following line.
- **Measured (page corrected 549→550):** `Haliartii perhibent,` ends source **page 549**; the leaked
  header is the TOP running head of source **page 550** (folio 537, recto, 1980×3003): `LYSANDER.`+folio
  `537` ink rows ~213–267; body `habitasse apud ipsos Rhadamanthum...` starts ~270. (p549's own head is
  `ΛΥΣΑΝΔΡΟΣ`/536 and is not the O2 leak.)
- **PROPOSED mask (normalized, page 550):** `[0.0859, 0.0683, 0.9626, 0.0892)`. Box: `rfi-10-O2-p550-box.png`.
  Awaiting confirmation.
- **Decision:** _pending (box drawn, page 550)._

## RFI-11 — O3: Demosthenes header/divider in gap (Tomus VII/167)

- **Unit:** `demosthenes` (source 1, 429–446). **Observed (O3):** a partial running head
  ending in Greek letters inside a gap between `...mœrore suo` and `conficiat, sed et levandus...`;
  a right divider remains.
- **Prior artifacts:** `inner_review/detail_t7_167.png`. **Generated (2026-10-05):**
  `rfi-11-O3-p441-overview.png`, `rfi-11-O3-p441-top.png`.
- **Confirm:** header + divider rectangles exclude both framing lines; classify the gap (RFI-20).
- **Measured:** source 1 **page 441** (1879×2840); `...moerore suo` ends page 440; `conficiat, sed et
  levandus...` is the FIRST Latin line of page 441. Leaked top Greek running head `ΔΗΜΟΣΘΕΝΗΣ.`+folio
  ink rows ~130–157; body starts ~188. A real vertical **rule at the far-right edge** (x≈1860–1879,
  ~half-height) is the finding's "divider"; the Latin text ends ~x1760 so the rule is in the right margin.
- **PROPOSED masks (normalized):** header `[0.0745, 0.0415, 0.9047, 0.0627)`; right rule
  `[0.9878, 0.0, 1.0, 1.0)`. Box: `rfi-11-O3-p441-box.png`. Awaiting confirmation.
- **Decision:** _pending (box drawn)._

## RFI-12 — O4: Antonius Greek header + divider at split (Tomus VII/369)

- **Unit:** `antonius` (source 1, 513–563). **Observed (O4):** a partial Greek running head and
  divider intrude between the split `Eo impe-` and continuation `trato, hostes...`.
- **Prior artifacts:** `inner_review/detail_t7_369.png`. **Generated (2026-10-05):**
  `rfi-12-O4-p538-overview.png`, `rfi-12-O4-p539-overview.png`.
- **Confirm:** the header + divider rectangles preserve the `Eo impe-`/`trato` split word.
- **Measured:** source 1 **page 539** (1824×2838); `Eo impe-` ends page 538; `trato, hostes irruentes
  repulit...` is the FIRST Latin line of page 539. Leaked top Greek running head `ΑΝΤΩΝΙΟΣ.`+folio `1116`
  ink rows ~140–167; body starts ~198. Right margin has only a short section marker (~315px), not a
  prominent rule — proposing header mask only.
- **PROPOSED mask (normalized):** `[0.0768, 0.0444, 0.9265, 0.0677)`. Box: `rfi-12-O4-p539-box.png`.
  Awaiting confirmation.
- **Decision:** _pending (box drawn)._

## RFI-13 — O6: Cato Minor `ΩΝ.` + right rule (Tomus VI/329)

- **Unit:** `cato_minor` (source 1, 309–363). **Observed (O6):** isolated Greek **`ΩΝ.`** above
  the Latin opening `nonnulli etiam amici Pompeji...`, plus a vertical black rule at right.
  Source classification not proven visually — could be a neighboring-column fragment.
- **Prior artifacts:** `review56/` VI/329. **Generated (2026-10-05):**
  `rfi-13-O6-p337-overview.png`, `rfi-13-O6-p347-overview.png` (inspect which has `ΩΝ.`+rule).
- **Confirm:** that `ΩΝ.` and the rule are source furniture (not part of the Latin column) and
  the rectangles exclude `nonnulli etiam...`.
- **Measured (page is 337, not 347):** source 1 **page 337** (1846×2838); top Greek running head
  `ΚΑΤΩΝ.` (→ `ΩΝ.` fragment) + folio `934` ink rows ~142–170; clean gap 174–203; body
  `nonnulli etiam amici Pompeii...` starts row 204. A partial vertical rule (~494 px) sits in the
  right margin (not at the very edge) — still to locate precisely.
- **PROPOSED mask (normalized):** header `[0.0845, 0.0472, 0.9621, 0.0691)`. Box: `rfi-13-O6-p337-box.png`.
  Right rule pending precise measurement. Awaiting confirmation.
- **Decision:** _pending (header box drawn; right rule TBD)._

## RFI-14 — O7: safe divider rectangles on listed pages

- **Observed (O7):** thin/thick/broken vertical source dividers survive beside Latin on, e.g.,
  I/36, I/46, II/219–220, II/236–238, V/195–196, VI/329, **VI/358**, VII/21, VIII/130–132.
- **Confirm:** for each divider I measure, that its rectangle is safely separable from all
  adjacent Latin pixels; dividers fused to glyphs are **retained** and recorded. This item is a
  bounded sweep of the listed pages, not a corpus-wide cleanup.
- **Decision:** _pending._

---

# Group C — Gap classification (removable copied space vs. genuine spacing)

## RFI-15 — G1: Timoleon word-split gap (Tomus II/219–220)

- **Unit:** `timoleon` (source 0). **Observed (G1):** 219 has two top lines then a large empty
  area with a divider; 220 has blank space before prose resumes. `ci` / `num` are the two
  halves of one word split across the interruption.
- **Prior artifacts:** `review12/` II/219–220. **Generated:** none yet — source page unresolved
  (scan s0 296–318 for the big interior gap page, or anchor via inner PDF p219–220).
- **Confirm:** that the intervening source rows are copied blank/furniture (removable) and the
  `ci`+`num` word halves are both preserved and rejoin.
- **Decision:** _pending._

## RFI-16 — G2: Timoleon near-blank pages (Tomus II/235–238)

- **Unit:** `timoleon` (source 0). **Observed (G2):** 235 ends with prose; **236 and 237 carry
  no prose** (dividers + generated furniture only); 238 is mostly blank until two bottom lines
  `dam sublime se cursu proripuit...`. Two effectively blank body pages + a wasted third.
- **Prior artifacts:** `review12/` II/235–238. **Generated (2026-10-05):**
  `rfi-16-G2-p303-overview.png`, `rfi-16-G2-p304-overview.png`, `rfi-16-G2-p305-overview.png`
  (`dam sublime se cursu proripuit...` is on source p305).
- **Confirm:** what Latin precedes `dam sublime se cursu proripuit...`; that the 236–237 regions
  are furniture/blank only; and that removing them loses no line.
- **Decision:** _pending._

## RFI-17 — G3: Brutus divider-only page + split word (Tomus VIII/130–132)

- **Unit:** `brutus` (source 1, 596–626). **Observed (G3):** 130 has four top lines ending
  `subli-`, then a long divider through empty space; **131 has no prose** (divider + generated
  BRUTUS head/folio); 132 has a top gap + divider fragments then `partem secutus.` and the
  concluding paragraph.
- **Prior artifacts:** `inner_review/detail_t8_130.png`–`132.png`; `remaining-s1-p0596-column.png`.
  **Generated (2026-10-05):** `rfi-17-G3-p624-overview.png`, `rfi-17-G3-p625-overview.png`,
  `rfi-17-G3-p626-overview.png` (`partem secutus.` on p626; `subli-` precedes).
- **Confirm:** the material between `subli-` and `partem secutus.` — is the word continuation
  present in the source and preserved? Which rows are removable furniture/blank?
- **Decision:** _pending._

## RFI-18 — G6: Pyrrhus mid-sentence gap (Tomus III/175)

- **Unit:** `pyrrhus` (source 0, 470–497). **Observed (G6):** ~3 line-pitches of gap between
  `...strepitu aquæ` and `verba non percipientibus...` — within a continuous sentence.
- **Prior artifacts:** `review34/` III/175. **Generated (2026-10-05):**
  `rfi-18-G6-p470-overview.png`, `rfi-18-G6-p471-overview.png`.
- **Confirm:** whether the gap is copied margin/furniture space (remove) or genuine spacing (keep).
- **Decision:** _pending._

## RFI-19 — G7 / G8: Theseus chapter-transition gaps (Tomus I/36 and I/46)

- **Unit:** `theseus` (source 0, 14–33). **Observed:** G7 — ~3 line-heights of whitespace after
  `et mendacia esse.` before chapter XXI, with a scan divider beside it (I/36). G8 — ~4
  line-heights after `...Cynoscephalas exstant.` before chapter XXVIII, with a retained divider (I/46).
- **Prior artifacts:** `review12/` I/36 and I/46. **Generated (2026-10-05):**
  `rfi-19-G78-p23-overview.png` (G7), `rfi-19-G78-p28-overview.png` (G8).
- **Confirm (each):** chapter spacing (keep) vs. scan-artifact padding (remove); and the divider
  rectangle is separable from the Latin.
- **Decision:** _pending._

---

# Group D — Sliced line

## RFI-20A — O8B: Timoleon line cut across the page break (Tomus II/240–241)

- **Unit:** `timoleon` (source 0). **Observed (O8B):** the bottom line on 240 is sliced
  horizontally through its letters; the lower halves appear clipped at the top of 241. A raster
  page break fell through a text line instead of between lines.
- **Prior artifacts:** `inner_review/user_t2_cut_240.png`, `user_t2_split-241.png`–`242.png`.
- **What I will produce:** the source band for that line and the selected cut row, at production
  global scale, before/after candidate cleanup (the cut may be an upstream divider-merge, fixed
  by Group B/C, not a packer change).
- **Confirm:** the identity of the single sliced source line, so the fix keeps both halves on
  one composed canvas; and whether Group B/C cleanup already resolves it.
- **Decision:** _pending._

---

# Group E — Furniture-only page

## RFI-21 — O5: Marius terminal ornament page (Tomus III/298)

- **Units:** `caius_marius` (source 0, 497–531) ending; `lysander` opens 531. **Observed (O5):**
  no prose — only a black source ornament/rule, a tiny fleck, and generated C. MARIUS head/folio.
  Marius's prose concludes on 297; Lysander opens on 299.
- **Prior artifacts:** `review34/` III/298. **Generated (2026-10-05):**
  `rfi-21-O5-p529-overview.png`, `rfi-21-O5-p530-overview.png` (inspect which is the ornament page).
- **Confirm:** that Caius Marius ends completely on the preceding body page and the ornament/fleck
  page carries no Latin, so it can emit no body canvas.
- **Decision:** _pending._

---

# Group F — Editorial retentions (confirm the default)

## RFI-22 — W3: Google Books notice page (all tomi, physical page 8)

- **Observed (W3):** full Google Books reproduction notice with logo, URL, QR code — a separately
  inserted front-matter page, not a watermark over prose (`src/facsimile/frontmatter.rs`).
- **Confirm:** retain as-is (plan desired end-state #7 default).
- **Decision:** **CONFIRMED — retain as-is** (2026-10-05). Body cleanup does not touch the
  W3 notice page in any tomus.

## RFI-23 — O8: Praefatio typography transition (Tomus V/14–17)

- **Observed (O8):** page 14 ends mid-page after `...in duarum vitarum exemplis consistam, Ages.`;
  15–16 continue in denser/darker scanned typography; ordinary typeset prose resumes on 17.
  Conspicuous scale/weight change + wasted space; not blank, not missing text.
- **Confirm:** retain as-is, with a recorded diagnosis for a possible separate layout task
  (using the existing Praefatio source inventory in `src/facsimile/praefatio.rs`, not a new tracer).
- **Decision:** **CONFIRMED — retain as-is + record diagnosis** (2026-10-05). The denser 15–16
  scanned block stays; I record its cause from the existing `praefatio.rs` inventory as a
  candidate separate layout task. No new front-matter tracer; no scale/packing change here.

---

# RFI status tracker

| RFI | Finding(s) | Group | Status |
| --- | --- | --- | --- |
| 01 | D1 | Duplication | pending |
| 02 | D2 | Duplication | pending |
| 03 | W1 | Watermark/furniture | **CONFIRMED** — 2 masks (s0 p287, p288) |
| 04 | W2, G4 | Watermark/furniture | pending — pin page via inner PDF |
| 05 | O1 | Watermark/furniture | **CONFIRMED** — 1 mask (s0 p182) |
| 06 | O1B | Watermark/furniture | pending — re-anchor ch. XXII |
| 07 | O1C | Watermark/furniture | **CONFIRMED** — 1 mask (s0 p290), no divider |
| 08 | O1D, G8B | Watermark/furniture | **BOX DRAWN** — header mask s0 p298 |
| 09 | G5 | Watermark/furniture | **BOX DRAWN** — header mask s0 p308 |
| 10 | O2 | Watermark/furniture | **BOX DRAWN** — header mask s0 p550 |
| 11 | O3 | Watermark/furniture | **BOX DRAWN** — header + right rule s1 p441 |
| 12 | O4 | Watermark/furniture | **BOX DRAWN** — header mask s1 p539 |
| 13 | O6 | Watermark/furniture | **BOX DRAWN** — header mask s1 p337 (rule TBD) |
| 14 | O7 | Dividers | pending |
| 15 | G1 | Gap | pending |
| 16 | G2 | Gap | pending |
| 17 | G3 | Gap | pending |
| 18 | G6 | Gap | pending |
| 19 | G7, G8 | Gap | pending |
| 20A | O8B | Sliced line | pending |
| 20 | (Phase 3.2 whitespace compaction need) | Gap | derived from 04/08/09/11/15–18 |
| 21 | O5 | Furniture-only page | pending |
| 22 | W3 | Editorial | **CONFIRMED** — retain |
| 23 | O8 | Editorial | **CONFIRMED** — retain + diagnose |

_Proposed interview order:_ warm up with Group F (RFI-22, 23), then Group B watermarks/furniture
(RFI-03…14), Group C gaps (RFI-15…19), the sliced line (RFI-20A), the furniture-only page
(RFI-21), and finally the two read-both duplication decisions (RFI-01, 02).
