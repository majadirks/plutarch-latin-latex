# Inner-tomi audit synthesis findings

## Scope, method, and how this synthesizes two inspections

This report synthesizes two independent inspections of the rebuilt
`build_facsimile/tomus_1_inner.pdf` through `tomus_8_inner.pdf`:

1. The broad **human visual reinspection** of 2026-10-06,
   [`2026_10_06_15_40_INNER_TOMI_REINSPECTION_FINDINGS.md`](2026_10_06_15_40_INNER_TOMI_REINSPECTION_FINDINGS.md),
   which assigned the finding IDs D4–D12, C1–C7, H1/H2, F1/F2, W4–W6, M1–M11,
   R1–R14, S1/S2, B1–B4, and the 68-row endpoint ledger E01–E68.
2. The **deterministic Python audit** of 2026-10-08,
   [`tools/audit_inner_tomi.py`](../tools/audit_inner_tomi.py), whose frozen
   full-corpus run and verification are recorded in
   [`2026_10_08_14_44_INNER_TOMI_AUDIT_STEP_102_COMPLETION.md`](2026_10_08_14_44_INNER_TOMI_AUDIT_STEP_102_COMPLETION.md)
   and whose calibration is in
   [`2026_10_08_09_18_INNER_TOMI_VALIDATION_CALIBRATION.md`](2026_10_08_09_18_INNER_TOMI_VALIDATION_CALIBRATION.md).

The two methods are complementary. The human sweep read contact sheets, edge
sheets, and targeted native pages, and was conservative about anything it could
not confirm by eye. The script builds a character n-gram TF-IDF index **from the
original source only**, maps reader OCR to source occurrences by bounded
alignment, corroborates with regional raster OCR, logo correlation, source-image
registration, connected components, and measured glyph boundaries, and scores
every candidate under fixed confidence predicates. Where the human sweep said
"the total extent is unmeasured," the script measures it.

Neither inspection repaired anything. No Latin, source code, fixture, unit
table, transcription cache, or PDF was changed; this document records findings
and priorities only and supplies no mask coordinates, deletion instructions, or
Latin corrections.

**Headline.** The script **corroborates every structural finding** of the human
sweep — all nine duplicate blocks (D4–D12) and all three body watermarks
(W4–W6) — and **extends two categories materially**: it confirms two running-head
leaks the human sweep did not (beyond H1/H2), and it raises the distinct-omission
count from the human sweep's **4 omissions / 8 source lines** to **59 confirmed
unique source-line losses totalling 113 source Latin lines across 31 units**,
including several multi-line block losses. All 68 body-section endings remain
complete; this does not establish interior completeness.

**Confirmation basis.** Script figures below are the frozen full-corpus run
`step102-final-all-5` (candidates SHA
`03f31fc68aa8829b90cdf3462578d9978b05d01a5a9c895416333dd4f2c78144`). "Confirmed"
means an independent reviewer viewed the source/reader crops and adjudicated the
candidate genuine; the 94-event core inventory was fully adjudicated (75
confirmed, 19 candidate false positives). A fixed 200-page image panel showed
0 high-confidence false-positive pages and 0 uncertain pages. Candidate-selected
precision is not a representative page rate, and a confirmed localization is not
a repair instruction.

**Locator convention.** `VII/413` means one-based physical page 413 of that
inner PDF; `v2/562` means physical page 562 of `Ploutarchou_Bioi_v2.pdf`. These
match the human report's physical-page locators (its reader-folio `r###` values
are reproduced there and not recomputed here).

## 1. Repeated, duplicated, or out-of-order scans (D4–D12)

The script's duplicate detector independently confirmed all nine repeated-scan
blocks from the human report, by the same reader start pages and source folios.
Nine confirmed `duplicate` events, precision 9/9.

| ID | Unit | Reader start (script) | Source folios | Both inspections |
| --- | --- | --- | --- | --- |
| D4 | agesilaus | V/217 | v2/115–118 ↔ 119–122 | Confirmed by eye and by script |
| D5 | alexander | VI/48 | v2/211–212 ↔ 213–214 | Confirmed; second opening incomplete (see §3) |
| D6 | phocion | VI/221 | v2/295–296 ↔ 297–298 | Confirmed; second opening incomplete |
| D7 | cato_minor | VI/332 | v2/359–360 ↔ 361–362 | Confirmed |
| D8 | agis | VII/29 | v2/375–376 ↔ 377–378 | Confirmed; co-located with H2 |
| D9 | cicero | VII/208 | v2/463–464 ↔ 465–466 | Confirmed |
| D10 | antonius | VII/328 | v2/521–522 ↔ 523–524 | Confirmed |
| D11 | aratus | VIII/225 | v2/671–672 ↔ 673–674 | Confirmed |
| D12 | otho | VIII/283 | v2/699–700 ↔ 701–702 | Confirmed; second opening clipped (C7) |

As the human report stressed, the shared boundary pages carry unique neighboring
prose and the second openings of D5/D6/D12 are incomplete, so these are not
whole pages that can be discarded wholesale.

## 2. Distinct Latin omissions — the major expansion (C-findings)

The human sweep confirmed four distinct omissions totalling eight source Latin
lines (C1 Romulus, C2 Coriolanus, C3 Phocion, C5 Brutus) and explicitly left the
full extent unmeasured. The script's bounded-occurrence alignment, corroborated
by source-raster agreement and reader-OCR retries, located and crop-confirmed
**59 unique source-line losses across 31 units, totalling 113 source Latin
lines** (37 high-confidence, 22 low-confidence; a further 18 nominated omissions
were reviewed and rejected as false positives — all 77 candidates were
adjudicated). Each confirmed event carries `unique_loss` and a crop-viewed
reason; a loss is counted only where the line does not survive at any
scan-equivalent occurrence (a missing redundant-copy opening is excluded).

The largest block losses, each verified against full source and reader pages:

| Unit | Reader seam | Source | Lines | Confidence | What is dropped |
| --- | --- | --- | ---: | --- | --- |
| antonius | VII/413 | v2/562 | 23 | low | Cleopatra section between the `asser-` fragment and chapter LXXXVII |
| iulius_caesar | VI/133 | v2/252 | 12 | high | `enim et Crassum` … `felicior consultor haberetur` before chapter XIV |
| theseus | I/20 | v1/15 | 7 | high | section VI `Quum solemne etiam tum esset` … `Abantes autem primi hunc in modum tonsi` |
| demosthenes | (4 events) | — | 8 | mixed | four distinct seams |
| theseus | (2 events) | — | 8 | — | includes the 7-line block above |
| romulus | I/68 (C1) + 1 | v1/38–39 | 5 | — | C1's four lines plus one further line |
| brutus | VIII/128 (C5) + 2 | v2/623–624 … | 5 | — | C5's line plus two further seams |
| phocion | VI/229 (C3) + 4 | v2/298–299 … | 5 | — | C3's line plus four further seams |

Per-unit totals for all 31 affected units (confirmed events / source lines):

| Unit | ev / lines | Unit | ev / lines | Unit | ev / lines |
| --- | --- | --- | --- | --- | --- |
| aemilius_paulus | 1 / 1 | eumenes | 1 / 2 | pericles | 2 / 3 |
| agesilaus | 2 / 2 | fabius_maximus | 1 / 1 | phocion | 5 / 5 |
| alcibiades | 3 / 3 | iulius_caesar | 1 / 12 | pyrrhus | 1 / 1 |
| alexander | 1 / 1 | lucullus | 2 / 2 | romulus | 2 / 5 |
| antonius | 1 / 23 | lycurgus | 1 / 1 | solon | 2 / 3 |
| artoxerxes | 1 / 1 | lysander | 1 / 2 | theseus | 2 / 8 |
| brutus | 3 / 5 | marcellus | 1 / 1 | tiberius_gracchus | 3 / 4 |
| caius_marius | 1 / 1 | marcius_coriolanus | 1 / 2 | timoleon | 1 / 1 |
| cato_minor | 4 / 4 | nicias | 2 / 2 | **total** | **59 / 113** |
| cleomenes | 2 / 2 | numa | 1 / 1 | | |
| crassus | 2 / 2 | dio | 4 / 4 | | |

The four human C-findings are subsumed: Romulus, Coriolanus, Phocion, and Brutus
all reappear here with equal or larger confirmed loss. This category is the most
consequential for faithful reading and is the first repair priority. Each event
has located source/reader crops in the run's `index.html`; the 22 low-confidence
events in particular warrant a confirming human read before any repair.

## 3. Lateral clipping and line slicing (C4/C6/C7, S1/S2)

The human sweep confirmed lateral glyph clipping (C4 Alexander, C6 Romulus, C7
Otho's clipped repeat opening) and two horizontal line slices (S1 Alexander
VI/52–53, S2 Demosthenes/Cicero VII/244–245), and warned the total clipping
extent was unmeasured. The script's `slicing` detector localizes S1 and S2 as
gold hits; its `lateral_clipping` screen nominates many per-page candidates
(1,110 across the corpus) that are overwhelmingly exploratory — the fixed
200-page panel found 0 high-confidence false positives, but these nominations are
not individually adjudicated and must not be read as 1,110 defects. Clipping
remains a real but only partially quantified failure mode; treat the human C4/C6/C7
neighborhoods as confirmed and the broader clipping screen as a review queue.

## 4. Leaked running heads, folios, and Greek fragments (H1/H2, F1/F2 — two new)

The script's `head` detector confirmed four leaks: the two the human sweep found,
and **two it did not**.

| ID | Unit | Reader | Source | Finding |
| --- | --- | --- | --- | --- |
| H1 | philopoemen | III/123 | v1/448 | end of `PHILOPOEMEN` head retained in body (human + script) |
| H2 | agis | VII/29 | v2/375 | Greek `ΑΓΙΣ` head end-glyph in body, beside D8/R9 (human + script) |
| — (new) | caius_marius | III/293 | v1/528 | clipped `MARIUS` running-head suffix (`IUS` fragment) in the body |
| — (new) | cato_major | III/89 | v1/432 | source folio numeral `419` leaked into the body |

The two new leaks carry a reviewer note to verify final body-region ownership and
source-folio provenance; both were adjudicated confirmed against native crops.
The `foreign_scraps` detector nominates 14 Greek-column-sliver candidates
(exploratory), consistent with the human F1 Theseus and F2 Pyrrhus neighborhoods;
edition reference letters are not themselves defects.

## 5. Google watermarks in body prose (W4–W6)

The script's logo-correlation `watermark` detector confirmed exactly the three
body watermarks of the human report, precision 3/3:

| ID | Unit | Reader | Source |
| --- | --- | --- | --- |
| W4 | aemilius_paulus | II/273 | v1/321 |
| W5 | brutus | VIII/87 | v2/603 |
| W6 | brutus | VIII/128 | v2/623 |

The intentional front-matter Google notice on physical page 8 of every tomus
(the human report's W3) is a retained non-finding, not searched as body material.

## 6. Copied-margin gaps, divider rules, and blank body pages (M1–M11, R1–R14, B1–B4)

These layout categories are where the two inspections differ most in kind. The
human sweep enumerated representative confirmed gaps (M1–M11), rules (R1–R14, an
explicitly non-exhaustive catalog), and verified the absence of divider-only
body pages (B1–B4). The script treats these as **exploratory layout nominations**:
`copied_gap` 8 candidates (2 gold-confirmed), `rule` 68 candidates (reported as
exploratory misses, i.e. not individually adjudicated), `blank_body` nominations
all resolved to legitimate recto separators / signature padding (consistent with
the human B4 "no residual example found"). The 200-page panel's zero
high-confidence false positives covers these screens. Use the human M/R tables as
the confirmed representative set and the script's `rule`/`copied_gap` debug JSON
as the review queue; neither inspection claims an exhaustive rule count.

## 7. Section-ending completeness (E01–E68)

Both inspections agree: **all 68 body-section endings are complete**. The human
ledger E01–E68 matched each terminal sentence against source scans. The script
performed 68 independent endpoint comparisons (one per unit): 26 were
algorithmically localized and 42 were algorithmic localization misses —
predominantly inconclusive source-image registration, **not** truncation. All 68
terminal passages were confirmed visually complete in both inspections. A matched
or complete endpoint establishes only that the ending is present; it does not
establish that every interior line is present or unique (Coriolanus, Phocion, and
Brutus end completely yet carry confirmed interior omissions; Agis, Cato Minor,
and others end completely yet carry surviving duplicates).

## 8. Quantitative summary and reliability

- Coverage: 68 units, 2,678 body pages (requested == completed); reader OCR
  2,678/2,678 and source OCR 1,367/1,367 with 0 unreadable; 0 processing errors.
- Core burden: 94 grouped core events / 2,678 pages = 3.510 per 100 pages.
- Candidate-selected precision (94 core events): duplicate 9/9 (1.00), watermark
  3/3 (1.00), head 4/5 (0.80), missing_occurrence 59/77 (0.766).
- Fixed 200-page panel: 0 high-confidence false-positive pages, 0 uncertain pages.
- Determinism: a `--resume` rerun produced byte-identical candidates; a worker
  1-vs-4 comparison on agesilaus+agis was identical after path normalization.
- Input integrity: all 21 live and 21 captured inputs rehash clean against the
  baseline manifest.

## Suggested repair priorities (synthesized)

1. **Distinct Latin loss — now the dominant category.** 59 confirmed unique-line
   losses / 113 source lines across 31 units (superseding the human sweep's 4/8).
   Start with the multi-line block losses (antonius VII/413 ≈23 lines,
   iulius_caesar VI/133 ≈12 lines, theseus I/20 ≈7 lines, demosthenes) and the
   high-confidence subset; give the 22 low-confidence events a confirming read.
2. **Lateral clipping and line slices.** Human C4/C6/C7 and S1/S2 are confirmed;
   the broad `lateral_clipping` screen is a review queue, not a defect list.
3. **Repeated reading order.** D4–D12 repeat 20 source pages in nine blocks;
   D5/D6/D12 have incomplete second openings and must not be treated as two
   perfect copies.
4. **Furniture inside prose.** Running-head/folio leaks H1, H2, and the two new
   leaks (caius_marius III/293, cato_major III/89); watermarks W4–W6; Greek
   scraps F1/F2; copied margins M1–M11 and divider rules R1–R14, keeping genuine
   printed chapter/verse spacing (the human non-findings table) distinct from
   copied source margin.

## Limitations

Known-case regression hits do not establish corpus-wide recall; a low-confidence
or inconclusive observation is not automatic assurance; a matched endpoint does
not prove interior completeness; candidate-selected precision is not a
representative page false-positive rate; and the exploratory clipping/rule/scrap
screens require image review before action. Faint furniture and individual
clipped glyphs can still escape both methods. Full located evidence — source and
reader crops, confidence reasons, and raw OCR — lives in the frozen run's
`index.html` and `boundaries.html` under the ignored
`tmp/pdfs/inner_tomi_audit/runs/step102-final-all-5/`. The source Latin remains
untouched.
