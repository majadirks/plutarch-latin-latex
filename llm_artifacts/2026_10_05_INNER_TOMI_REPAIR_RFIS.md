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

The confirmed repairs are implemented and pushed in commits `99c5e83`, `61fa3ae`, and
`0e4893f`. All eight inner and eight imposed PDFs compiled on 2026-10-06. The full ordinary
suite and all selected source regressions passed. The production contract remains the 15
frozen ledger records: 13 mask records (14 rectangles) and two duplicate records.

The next decision is human review of [the post-rebuild evidence](2026_10_06_INNER_TOMI_POST_REBUILD_REVIEW.md):
twelve exact right-margin masks, plus retention of genuine Theseus chapter gaps G7/G8. Proposed
coordinates are in the ledger's `post_rebuild_review`; none is enabled in production before confirmation.

## Interview progress (as of 2026-10-06)

- **Implemented and verified:** RFI-01/02 duplication, RFI-03–13 header/watermark masks,
  RFI-21 Marius ornament, and RFI-22/23 editorial retentions.
- **Reassessed; awaiting human review:** RFI-14–19 and RFI-20A. G1/G2/G3 are omitted Latin
  columns caused by scan-edge column selection, not merely copied blanks. The source 306 edge
  merges the O8B line; the candidate restores whole-line banding. G6 is copied tail padding.
  G7/G8 match genuine chapter spacing.
- **Whitespace decision:** no row-compaction table has been introduced. Residual intervals
  contain scan-edge ink; the bounded margin-mask previews address that ink upstream.
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
- Evidence crops and local diagnostic files: `tmp/pdfs/inner_tomi_repairs/`, retained locally and uncommitted.

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
| 04 | W2, G4 | 0 | **299** | pinned via inner p227: `...Corinthiis ipsis inter`→[wm]→`se per otium dividere...`; the latter opens source p300, so the watermark is the page-tail of **p299** (folio 286). Source-page TAIL, not an interior gap → no compaction needed. |
| 05 | O1 | 0 | seam **181→182**; header `CAMILLUS.`+folio 169 atop **182** | `...jam exsul,`(181) / `neque nos cives`(182) |
| 06 | O1B | 0 | **281** | ch. XXII `Erat vir Antias... Amphidius` begins p281 (folio 268); leaked top Greek head `ΓΑΙΟΣ ΜΑΡΚΙΟΣ.`(→`ΑΡΚΙΟΣ`)+folio 268. (`inde concepisse` ends p280.) |
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
- **Measured & verified (2026-10-06):** The duplication is an **exact whole-page repeat**:
  source **p103 = p101** (folio 708, Eumenes ch. XVII–XVIII, `Antigonus non ea modo...Deinde
  impetu facto...`) and source **p104 = p102** (folio 700, Eumenes ch. XIX end `...inspicere
  posset` + comparison title + opening `Haec sunt quae de Eumene...magno in`). p101 is in the
  Eumenes unit (88–102); p102 is the shared tail; **p103/p104 appear only in the comparison
  unit (102–106) and hold NO unique prose.** The `...ducibus querentes. Deinde impetu facto`
  passage is Eumenes text on p101 (kept once). After skipping p103/p104, p102 (`...magno in`)
  joins **p105** (`pretio habitus...`) = `magno in pretio habitus` — the correct comparison
  continuation; the comparison concludes on p105.
- **Decision:** **CONFIRMED — keep 101/102, drop 103/104** (user 2026-10-06). Fix = whole-page
  duplicate exclusion (not a boundary change): `skip_known_duplicate_source_page` for comparison
  source pages 103 & 104 (Phase 2.1), identity-scoped to `comparatio_sertorii_et_eumenis`. Frozen
  in the ledger.

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
- **Measured & verified (2026-10-06):** Clean two-page whole-page duplicate in the Agis unit
  (source 1). Folio sequence: p366=951, p367=952, p368=953, **p369=952 (dup of p367)**,
  **p370=953 (dup of p368)**, p371=954, p372=955. Join p368 (`...ut unius suffragii numero`) +
  p371 (`excellerent qui antiquabant legem...`) is exact. p369/p370 hold no unique prose; the
  second occurrence has heavier right-side divider marks.
- **Decision:** **CONFIRMED — keep p367/p368, drop p369/p370** (user 2026-10-06). Fix = whole-page
  duplicate exclusion: `skip_known_duplicate_source_page` for Agis source pages 369 & 370
  (Phase 2.2, same mechanism as D1). Frozen in the ledger.

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
- **Measured (pinned via inner PDF):** inner `tomus_2_inner.pdf` p227 shows `...repulsis Corinthiis
  ipsis inter` → [watermark] → `se per otium dividere insulam liceret...`. `se per otium dividere...`
  is the first Latin line of source **page 300**, so the watermark is the **page-tail** of source
  **page 299** (folio 286, 1900×2963). Last Latin row 2568; watermark `Digitized by Google` rows
  ~2649–2921 (core x≈1220–1590). It is a source page TAIL (not an interior gap), so no whitespace
  compaction is needed — the tail below the last line simply is not copied once masked.
- **PROPOSED mask (normalized, page 299):** `[0.6000, 0.8775, 0.8632, 1.0000)`. Box:
  `rfi-04-W2-p299-box.png`. Confirmed; see the decision below.
- **Decision:** **CONFIRMED** (2026-10-06) — watermark mask s0 p299 frozen in the ledger.

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
- **Measured (pinned via inner PDF p187):** `...inde concepisse.` (end ch. XXI) is on source
  p280; ch. XXII `Erat vir Antias... Amphidius` begins source **page 281** (folio 268, verso,
  1948×2974). Leaked top Greek running head `ΓΑΙΟΣ ΜΑΡΚΙΟΣ.` (→ `ΑΡΚΙΟΣ`) + folio `268` ink rows
  ~193–221; body (ch. XXII) starts row 260. A far-right edge rule exists but is at x≈1935 (page
  edge), outside the Latin measure.
- **PROPOSED mask (normalized, page 281):** `[0.0857, 0.0615, 0.9589, 0.0807)`. Box:
  `rfi-06-O1B-p281-box.png`. Confirmed; see the decision below.
- **Decision:** **CONFIRMED** (2026-10-06) — header mask s0 p281 frozen in the ledger.

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
  Confirmed; see the decision below. (Whitespace/divider = RFI-20/O7, separate.)
- **Decision:** **CONFIRMED** (2026-10-06) — frozen in the ledger.

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
  Confirmed; see the decision below.
- **Decision:** **CONFIRMED** (2026-10-06) — frozen in the ledger.

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
  Confirmed; see the decision below.
- **Decision:** **CONFIRMED** (2026-10-06) — header mask s0 p550 frozen in the ledger.

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
  `[0.9878, 0.0, 1.0, 1.0)`. Box: `rfi-11-O3-p441-box.png`. Confirmed; see the decision below.
- **Decision:** **CONFIRMED** (2026-10-06) — frozen in the ledger.

## RFI-12 — O4: Antonius Greek header + divider at split (Tomus VII/369)

- **Unit:** `antonius` (source 1, 513–563). **Observed (O4):** a partial Greek running head and
  divider intrude between the split `Eo impe-` and continuation `trato, hostes...`.
- **Prior artifacts:** `inner_review/detail_t7_369.png`. **Generated (2026-10-05):**
  `rfi-12-O4-p538-overview.png`, `rfi-12-O4-p539-overview.png`.
- **Confirm:** the header + divider rectangles preserve the `Eo impe-`/`trato` split word.
- **Measured:** source 1 **page 539** (1824×2838); `Eo impe-` ends page 538; `trato, hostes irruentes
  repulit...` is the FIRST Latin line of page 539. Leaked top Greek running head `ΑΝΤΩΝΙΟΣ.`+folio `1116`
  ink rows ~140–167; body starts ~198. A far-right photographic scan edge remains outside
  the confirmed header mask; its separate correction is proposed in the post-rebuild review.
- **PROPOSED mask (normalized):** `[0.0768, 0.0444, 0.9265, 0.0677)`. Box: `rfi-12-O4-p539-box.png`.
  Confirmed; see the decision below.
- **Decision:** **CONFIRMED** (2026-10-06) — frozen in the ledger.

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
  `nonnulli etiam amici Pompeii...` starts row 204. The user confirmed that the alleged right
  rule is absent; no rule rectangle is authorized for this page.
- **PROPOSED mask (normalized):** header `[0.0845, 0.0472, 0.9621, 0.0691)`. Box: `rfi-13-O6-p337-box.png`.
- **Right rule:** **confirmed absent by user (2026-10-06)** — no separable right rule; header mask only.
- **Decision:** **CONFIRMED** (2026-10-06) — header mask s1 p337 frozen in the ledger; no right rule.

## RFI-13B — O6: second Cato header occurrence on source 347

- **Verified production mapping:** the confirmed source-337 mask appears on fresh VI/308–310. VI/329 still leaks the Greek header from the same folio-934 passage on source **347**.
- **Separate measured occurrence:** source 1 page 347 is 1787×2866; header/folio ink spans rows 154–193; body begins at row 219. Proposed header rectangle: `[140, 145, 1720, 205)` at 300 DPI, leaving a 14-pixel gap before body ink.
- **Preservation:** all outside pixels are unchanged in the independent preview. The page-337 rectangle and no-right-rule decision remain intact. No right-edge mask is proposed for page 347.
- **Evidence:** [post-rebuild review](2026_10_06_INNER_TOMI_POST_REBUILD_REVIEW.md), with the fresh VI/329, marked source crop, and header-only candidate.
- **Decision:** **pending human review of the separate header occurrence**.

## RFI-14 — O7: safe divider rectangles on listed pages

- **Fresh output:** V/195–196 has one comparison opening and the `magno in pretio habitus` join after D1. The reported VII/21 second-occurrence divider is excluded by D2.
- **Mapped surviving standalone rule:** VI/358 is source 1 page **361**, whose far-right edge lies beyond the Latin crop. The candidate rectangle is `[1785, 0, 1810, 2848)` at 300 DPI. O6 page 337 remains confirmed without a right rule and is not revisited.
- **Related cases:** G1/G2/G3 map to source 0 pages 296/304 and source 1 page 625; scan-edge column selection omitted Latin. I/36 and I/46 retain genuine chapter spacing; their near-column rules are not erased by a spacing disposition.
- **Evidence:** [post-rebuild review](2026_10_06_INNER_TOMI_POST_REBUILD_REVIEW.md).
- **Decision:** D1/D2 cases resolved by the confirmed exclusions; the specific remaining mask and spacing dispositions are **pending human review**.

---

# Group C — Gap classification (removable copied space vs. genuine spacing)

## RFI-15 — G1: Timoleon word-split gap (Tomus II/219–220)

- **Unit:** `timoleon`, source 0.
- **Fresh output:** II/219 ends with source 295, followed by a rule; II/220 resumes with source 297. **Source 296's Latin is omitted.**
- **Measured:** source 296 is 1990×2984. The detector selects x=1980–1990, a 10-pixel scan edge. Its Latin contains the complete intervening chapters II/III and the continuation to source 297.
- **Proposed correction:** mask `[1965, 0, 1990, 2984)` at 300 DPI. The candidate detects Latin crop x=940–1727 and restores the intervening prose.
- **Evidence:** [post-rebuild review](2026_10_06_INNER_TOMI_POST_REBUILD_REVIEW.md), with fresh II/219–220 and independent candidate images.
- **Decision:** **pending human review of the measured margin mask**.

## RFI-16 — G2: Timoleon near-blank pages (Tomus II/235–238)

- **Unit:** `timoleon`, source 0.
- **Fresh output:** II/236–237 still carry scan-edge ink without prose; II/238 resumes with source 305's `dam sublime se cursu proripuit...`.
- **Measured:** source 304 is 1988×2985. The detector selects x=1977–1988, so **source 304's Latin is omitted**. The production trace maps the blank canvases to its tall strip, rows 58–1603.
- **Proposed correction:** mask `[1963, 0, 1988, 2985)` at 300 DPI. The candidate detects x=917–1709 and restores the intervening Latin and `...ad saxum quod-` / `dam sublime...` seam.
- **Evidence:** [post-rebuild review](2026_10_06_INNER_TOMI_POST_REBUILD_REVIEW.md).
- **Decision:** **pending human review of the measured margin mask**.

## RFI-17 — G3: Brutus divider-only page + split word (Tomus VIII/130–132)

- **Unit:** `brutus`, source 1, 596–626.
- **Fresh output:** VIII/130 ends with `subli-`; VIII/131 carries only a scan edge; VIII/132 resumes at source 626's `partem secutus.`. **Source 625's Latin is omitted.**
- **Measured:** source 625 is 1816×2842. The detector selects x=1803–1816. The missing page contains the `dens` continuation, complete intervening Latin, and the sentence continuing onto 626.
- **Proposed correction:** mask `[1791, 0, 1816, 2842)` at 300 DPI. The candidate detects x=980–1767 and restores the complete page without retyping Latin.
- **Evidence:** [post-rebuild review](2026_10_06_INNER_TOMI_POST_REBUILD_REVIEW.md), including candidate Brutus canvas 60 and fresh VIII/130–132.
- **Decision:** **pending human review of the measured margin mask**.

## RFI-18 — G6: Pyrrhus mid-sentence gap (Tomus III/175)

- **Unit:** `pyrrhus`, source 0, 470–497.
- **Fresh output:** III/175 retains the gap between the end of source 470 and `verba non percipientibus...` on 471.
- **Measured:** source 470 is 2016×3023. Its final retained band extends through row 2782 because right-edge ink connects the body and lower margin.
- **Proposed correction:** mask `[1991, 0, 2016, 3023)` at 300 DPI. The candidate detects x=972–1805 and restores ordinary sentence-seam spacing.
- **Evidence:** [post-rebuild review](2026_10_06_INNER_TOMI_POST_REBUILD_REVIEW.md), including fresh III/175 and candidate Pyrrhus canvas 3.
- **Decision:** **pending human review of the measured margin mask**.

## RFI-19 — G7 / G8: Theseus chapter-transition gaps (Tomus I/36 and I/46)

- **Unit:** `theseus`, source 0, 14–33.
- **Fresh output/source comparison:** I/36 matches source 23's genuine gap above chapter XXI; I/46 matches source 28's genuine gap above chapter XXVIII. The same chapter boundaries and spacing appear in the edition. Nearby vertical rules are separate findings.
- **Recommendation:** **retain both gaps as genuine source chapter spacing**; introduce no row exclusions.
- **Evidence:** [post-rebuild review](2026_10_06_INNER_TOMI_POST_REBUILD_REVIEW.md), with fresh I/36 and I/46 and source overviews.
- **Decision:** **pending human confirmation of both retentions**.

---

# Group D — Sliced line

## RFI-20A — O8B: Timoleon line cut across the page break (Tomus II/240–241)

- **Unit:** `timoleon`, source 0.
- **Fresh output:** II/240–241 still slice the line beginning `segniter negligenterque agere...`.
- **Production trace:** source **306**, original strip **333–441**, canvases **24/25**, resized-strip cut row **23**, estimated original row **352.12848**. Adjacent rows contain **520/495 ink pixels**, proving the cut crosses type. The full scaled pixel-stream regression passes: the line is split, not missing.
- **Proposed upstream correction:** mask the far-right scan edge `[1976, 0, 2001, 2988)` at 300 DPI. It restores separate body bands; candidate Timoleon canvas 23 holds both early source lines whole. No general splitter or scale change is proposed.
- **Evidence:** [post-rebuild review](2026_10_06_INNER_TOMI_POST_REBUILD_REVIEW.md) and `tmp/pdfs/inner_tomi_repairs/post_mask/composition.json`.
- **Decision:** **pending human review of the measured margin mask**.

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
- **Measured:** source 0 **page 530** (1963×2984, folio 517) is Marius's LAST page. Prose ends rows
  202–687 with `...αὐτὸς αὑτὸν ἀπέκτεινεν.` / `posset, mortem sibi ipse conscivit.` (complete); then a
  ~485-row blank; then a decorative end-of-Life ornament (horizontal flourish) at rows 1172–1190,
  x 168–1166; then blank + the page-bottom watermark. No Latin below the ending.
- **PROPOSED fix:** mask the ornament `[0.0703, 0.3877, 0.6093, 0.4038)` so the furniture-only strip
  carries no ink and emits no body canvas (alternative: the Phase 4.1 empty-canvas check). Box:
  `rfi-21-O5-p530-box.png`. Confirmed; see the decision below.
- **Decision:** **CONFIRMED** (2026-10-06) — ornament mask s0 p530 frozen in the ledger; Marius
  ends complete, no Latin below.

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
| 01 | D1 | Duplication | **CONFIRMED** — skip comparison s1 p103/p104 (dups of p101/p102) |
| 02 | D2 | Duplication | **CONFIRMED** — skip Agis s1 p369/p370 (dups of p367/p368) |
| 03 | W1 | Watermark/furniture | **CONFIRMED** — 2 masks (s0 p287, p288) |
| 04 | W2, G4 | Watermark/furniture | **CONFIRMED** — watermark mask s0 p299 (page tail) |
| 05 | O1 | Watermark/furniture | **CONFIRMED** — 1 mask (s0 p182) |
| 06 | O1B | Watermark/furniture | **CONFIRMED** — header mask s0 p281 |
| 07 | O1C | Watermark/furniture | **CONFIRMED** — 1 mask (s0 p290), no divider |
| 08 | O1D, G8B | Watermark/furniture | **CONFIRMED** — header mask s0 p298 |
| 09 | G5 | Watermark/furniture | **CONFIRMED** — header mask s0 p308 |
| 10 | O2 | Watermark/furniture | **CONFIRMED** — header mask s0 p550 |
| 11 | O3 | Watermark/furniture | **CONFIRMED** — header + right rule s1 p441 |
| 12 | O4 | Watermark/furniture | **CONFIRMED** — header mask s1 p539 |
| 13 | O6 | Watermark/furniture | **CONFIRMED** — header mask s1 p337 (no rule) |
| 13B | O6 second occurrence | Watermark/furniture | s1 p347 header on VI/329; separately measured mask pending review |
| 14 | O7 | Dividers | D1/D2 cases resolved; s1 p361 margin mask pending review |
| 15 | G1 | Gap | s0 p296 omitted Latin; restoring margin mask pending review |
| 16 | G2 | Gap | s0 p304 omitted Latin; restoring margin mask pending review |
| 17 | G3 | Gap | s1 p625 omitted Latin; restoring margin mask pending review |
| 18 | G6 | Gap | s0 p470 copied tail; margin mask pending review |
| 19 | G7, G8 | Gap | recommend genuine-spacing retention; pending review |
| 20A | O8B | Sliced line | s0 p306 traced cut; upstream margin mask pending review |
| 20 | (Phase 3.2 whitespace compaction need) | Gap | no row compaction proposed; residual scan-edge ink is handled upstream |
| 21 | O5 | Furniture-only page | **CONFIRMED** — ornament mask s0 p530 (Marius ends complete) |
| 22 | W3 | Editorial | **CONFIRMED** — retain |
| 23 | O8 | Editorial | **CONFIRMED** — retain + diagnose |

_Proposed interview order:_ warm up with Group F (RFI-22, 23), then Group B watermarks/furniture
(RFI-03…14), Group C gaps (RFI-15…19), the sliced line (RFI-20A), the furniture-only page
(RFI-21), and finally the two read-both duplication decisions (RFI-01, 02).
