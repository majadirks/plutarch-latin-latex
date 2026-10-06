# Inner-tomi reinspection checkpoint (second continuation)

## Task and constraints

Resume the report-only inspection specified by **Prompt for LLM Agents** in `2026_10_06_13_49_INNER_TOMI_REINSPECTION_PROMPT.md`. Read this checkpoint together with the earlier `2026_10_06_14_27_INNER_TOMI_REINSPECTION_CHECKPOINT.md`; this note carries forward the new evidence and corrects several tentative interpretations in that note. Neither checkpoint is the final findings report.

The deliverable remains a NEW dated findings report in `llm_artifacts/`, with scope/method, eight category tables, confirmed/unconfirmed assessments, physical inner-PDF page, printed reader folio, printed source folio, exact short Latin anchor, endpoint coverage, and a short repair-priority list. **Do not repair anything or alter Latin.** No source code, fixtures, unit table, PDFs, or transcription cache has been changed. Diagnostic additions are under `tmp/pdfs/reinspection/`. Preserve both checkpoints and all historical notes.

The user prefers thoroughness over expediency and has explicitly requested this checkpoint so they can compact. Do not spawn agents: current developer instructions prohibit unsolicited delegation. The PDF skill has been read and applied for read-only visual inspection; its temporary-file cleanup advice is overridden by the user's need to preserve diagnostics. No tests/build are necessary for this read-only task.

The user asked about an internal note saying approximately 42 minutes with no progress. I clarified that this was a poorly worded reminder to give a progress update, not an assertion that inspection had stalled. Substantial inspection was completed. Avoid using elapsed time as a reason to cut the work short.

## Build, locators, and completed diagnostics

Current PDFs are `build_facsimile/tomus_1_inner.pdf` through `tomus_8_inner.pdf`. Physical counts remain 416, 448, 416, 128, 384, 352, 448, 320: **2,912 PDF pages** total. `body_map.csv` contains **2,678 body-image pages**, by tomus: 376, 411, 395, 109, 354, 331, 409, 293.

Reader folios: physical minus 16 in I; physical minus **18** in V; physical minus 8 in the other tomi. Section-opening pages can lack a printed folio; label such folios as inferred from adjacent printed numbering, rather than pretending the numeral is visible. Every individually inspected ordinary page agrees with these offsets.

All full-volume **180-DPI renders and body OCR are now complete**. All **47 edge sheets have been visually inspected** across all eight tomi. All eight low-resolution volume sweeps and all 68 high-resolution endings were completed before the first checkpoint and remain valid.

| Tomus | High PNGs | Body OCR TXT | Edge sheets |
| --- | ---: | ---: | ---: |
| I | 416 | 376 | 7 |
| II | 448 | 411 | 7 |
| III | 416 | 395 | 7 |
| IV | 128 | 109 | 2 |
| V | 384 | 354 | 6 |
| VI | 352 | 331 | 6 |
| VII | 448 | 409 | 7 |
| VIII | 320 | 293 | 5 |

At continuation, old session IDs returned unknown process. Files had stopped writing at 14:40:37, leaving VII rendering at page 21 and a partial `edges-6-0.png`. I resumed VII from page 21, rendered VIII, and reran the existing-file-aware OCR and edge scripts. The following new sessions were polled to **successful completion**: rendering VII 94702; rendering VIII 57092; OCR 42305; edges 69130. Source-column script also completed. Source render batch 77536 generated all requested files; it may still need a completion poll, although all files exist and were inspected. Join-strip session 15861 generated all files and they were inspected; a final completion poll is optional. No need to rerender complete work.

Important diagnostic caveat: edge sheets crop a fixed 44-pixel strip. Partial *second* lines at a strip's edge can be caused by the diagnostic crop. Confirm actual slicing on the full adjacent pages. There are two confirmed examples below. Pixel metrics miss very faint gray marks and are triage, not proof.

## New diagnostic files from this continuation

All under `tmp/pdfs/reinspection/`:

- `join-strips.ps1`, `join-A-B.png`: nine source joins after hypothetical omission of the second duplicate scan. Uses cached 300-DPI source renders. The script was corrected to crop from 46% of source width so the complete Latin line is included. **All nine corrected joins inspected.** Images stretch the column vertically; use raw source renders for precise typography.
- `source-columns.ps1`, `column1-P.png`, `column2-609.png`: complete Latin columns at approximately 180 DPI for printed-spacing checks. **All 15 inspected.** Source pages: v1 19,37,40,41,42,43,46,48,50,51,53,54,58,603; v2 609. Some source folio numerals sit outside the cropped column; use existing full-width source-header sheets for those.
- `gap-map.cjs`, `gap-source-map.json`: diagnostic word-set matching between output OCR and source Latin OCR. Candidate source associations, not authoritative text. The console output is useful for locating the printed-spacing cases.
- `ocr-furniture-candidates.txt`: all isolated OCR capital runs/numbers across all 2,678 body pages. All returned candidates checked against full images; most are generated headings, prose/chapter numbers, or OCR false positives. The substantive new hit was III/123. Greek fragments can escape OCR; VII/29 was found visually.
- `slice6-052.png`, `slice6-053.png`, `slice7-244.png`, `slice7-245.png`: **300-DPI** confirmation of line slicing. All inspected.
- Fresh full 180-DPI source images `src1-23,28,35,38,39,229,230,257,258,292,297,298,315,316,321,329,330,448.png` and `src2-213,299,375,480,604,623.png`. Most inspected as noted below; `src2-213`, `src2-375`, `src2-480` still need explicit raw-source visual review for the exact heading/crop/slice associations. Existing duplicate pair images already establish source folios and matching prose.

## Confirmed nine new duplicate blocks: corrected output boundaries

All occurrence beginnings, reset boundaries, and final contribution pages listed below were inspected on **native 180-DPI output**. Same prose and chapter progression repeats. Folios agree with visible output numbering except unnumbered unit openings, none of which is needed for these duplicate ranges. Keep D1-D3 reserved; use D4-D12 for these blocks.

Inclusive output ranges can overlap on a shared physical page because the end of one scan and start of the repeat occupy the same page. These are contribution ranges, not whole output pages that could safely be dropped.

| Proposed ID | Life | v2 source occurrences | Printed source folios | Confirmed output occurrences (physical; reader) | Boundary detail |
| --- | --- | --- | --- | --- | --- |
| D4 | Agesilaus | 115-118 / 119-122 | 720-723 | V/217-225 (199-207) / 225-233 (207-215) | XV restarts mid-page 225 after `ferens et patrocinans, ex inimicis familiares redegit, atque`; second occurrence ends mid-page 233 before `ita confecit`. |
| D5 | Alexander | 211-212 / 213-214 | 812-813 | VI/48-52 (40-44) / 52-56 (44-48) | First occurrence starts on 48 at `Alexander, Philippine interfectores omnes ultus esset, et`; second begins on 52 at `an omnibus esset dominaturus hominibus`, omitting the source's first Latin line. End on 56 before `magnum et regis dextram reveritus`. Second occurrence has clipped left line beginnings, particularly VI/53. |
| D6 | Phocion | 295-296 / 297-298 | 894-895 | VI/221-225 (213-217) / 225-229 (217-221) | First source opening appears on 221 at `eorum, qui Thebis huc confugerunt; sufficit tamen Græcis`; second starts on 225 at `Thebas lugere.` and omits its source's first Latin line. End before source 299 material on 229. A distinct source-299 opening line is also absent from output; see C3 below. |
| D7 | Cato Minor, later repeat | 359-360 / 361-362 | 946-947 | **VI/332-336 (324-328)** / 337-341 (329-333) | Corrects checkpoint's overlapping 332-337 range. First starts near bottom 332 at `quoque, inquit, decrevistis virum eo ætatis provectum`; first ends at bottom 336, `dato, quum admirandam suam virtutem hostibus ostendisset,`; second begins top 337 and ends in first two lines of 341, before `occubuit.` from source363. Includes remainder of **LXIX** through LXXIII, not just LXX-LXXIII. |
| D8 | Agis, later repeat | 375-376 / 377-378 | 958-959 | **VII/29-32 (21-24)** / 32-36 (24-28) | Corrects tentative first boundary 28. Page28 ends `Ibi de-`; source375 starts page29 at `mum Amphares, auctoritate muneris sui Agidi manum`. Reset on32 after complete ending `ceteris credidisset.` Same opening appears again. Last repeat ends on36 with three concluding lines. |
| D9 | Cicero | 463-464 / 465-466 | 1044-1045 | VII/208-212 (200-204) / 212-216 (204-208) | Starts `voce maris cognita exclamavit ac mulieres convocavit`; reset on212 after `...multa egisset, sed`; second ends mid216 before `tum Cæsare socero...`. |
| D10 | Antonius | 521-522 / 523-524 | 1100-1101 | VII/328-332 (320-324) / 332-336 (324-328) | Starts `in foro duceretur. Ibi quum populum duci ac magnopere`; reset332 after `...titulum ei et`; second ends336 before `honorem Imperatoris conservans.` |
| D11 | Aratus | 671-672 / 673-674 | 1248-1249 | VIII/225-229 (217-221) / 229-233 (221-225) | Starts `a se gestarum occasionem fuisse inimicitias cum superiore`; reset229 after `...in Pe-`; second ends233 before `loponnesum misit`. |
| D12 | Otho | 699-700 / 701-702 | 1274-1275 | VIII/283-287 (275-279) / 287-291 (279-283) | First starts on283 at `erant, laboremque ob consuetudinem non fugiebant,`; second begins287 later, `rimum temporis in theatris...`, with leftmost glyph clipped. Several opening source lines do not recur, but survive in first occurrence. Second ends291 before `jam essent in itinere.` |

Quote anchors carefully in final report: these are source glyphs, including æ, not normalized text. Some OCR uses ae or incorrect letters. Use visibly clear short anchors if unsure. The exact Alexander source/output says **Philippine**, not checkpoint's `Philippine` versus possible guess `Philippi`—the visible VI/48 line reads `Alexander, Philippine interfectores...`; preserve the scan's wording. Do not silently fix it.

### Source joins after hypothetical removal: visually checked

All nine corrected `join-A-B.png` images were read. No distinct source-page block is lost by the source sequence omitting the repeated *second* block. This does not mean whole output pages can be dropped: boundary pages contain distinct neighboring prose. Join evidence:

| Source join | Visible continuity |
| --- | --- |
| 118 -> 123 | `...ferens et patrocinans, ex inimicis familiares redegit, atque` -> `ita confecit, uti neminem sibi in republica adversum habe-` |
| 212 -> 215 | `...si dominus tibi adhuc est Darius, dic mihi lumen Mithræ` -> `magnum et regis dextram reveritus, num nam ego minima...` (use a shorter faithful anchor in report; do not rely on this paraphrased spacing) |
| 296 -> 299 | `XXI. Scripserat Atheniensibus Alexander, ut triremes sibi mitterent. Id quum impugnarent oratores postulatum,` -> `a senatu dicere sententiam Phocion jussus, Jubeo, inquit,` -> `vos aut armis superare, aut amicitia superiorum uti.` |
| 360 -> 363 | `...dato, quum admirandam suam virtutem hostibus ostendisset,` -> `occubuit.` |
| 376 -> 379 | Agis finishes `ceteris credidisset.`; Cleomenes title and new Life follow. Complete unit ending. |
| 464 -> 467 | `...multa egisset, sed` -> `tum Cæsare socero...` |
| 522 -> 525 | `...titulum ei et` -> `honorem Imperatoris conservans.` |
| 672 -> 675 | `...in Pe-` -> `loponnesum misit` |
| 700 -> 703 | `...exspectandas e Mysia copias censuit, quæ` -> `jam essent in itinere.` |

No further source duplicate block was discovered in this continuation. Existing raw source candidate scan plus complete folio sweep cover the body ranges, as recorded in the first checkpoint. Otho's extra concluding scans709-710 are outside its deliberately pinned unit range ending708; do not report them as surviving output duplicates.

## New confirmed furniture and watermark findings

### Running heads / neighboring-column fragments

- **III/123, reader115, Philopoemen, v1/448 printed435:** confirmed leaked head fragment `LOPŒMEN.` (the initial clipped shape may be only part of L; use photograph rather than pretending complete PHILO is present) and isolated printed source folio **435** between `plane e mari aufuisse rati, superbe Gythium oppugnabant,` and `subito illis nihil tale exspectantibus, sed propter victoriam`. Full source `src1-448.png` confirms PHILOPOEMEN running head and435. Greek scraps also visible at left below the leak. Native output and raw source inspected.
- **VII/29, reader21, Agis, v2/375 printed958:** confirmed isolated **Greek sigma-like terminal heading fragment plus period** above `mum Amphares, auctoritate muneris sui Agidi manum`. There is also a heavy right rule. This is not the generated AGIS head. `dup7-029.png` inspected; raw `src2-375.png` exists but needs explicit inspection to pin the exact heading glyph. Do not transcribe a complete Greek head from inference.
- **III/173, reader165, Pyrrhus, v1/470 printed457:** confirmed extensive neighboring Greek-column scraps at left of opening `Traditum est memoriæ, secundum diluvium Phaethon-`. These are vertical scraps, **not** a leaked running-head word. Section markers b,c,d are edition reference letters; distinguish them from unintended Greek scraps. Native output inspected.
- **I/24, reader8, Theseus, v1/17 printed4:** confirmed Greek-column slivers beside long left divider and prose `labatur, apprehendentem ipsum et vetantem progredi`. The OCR candidate `TSE` is a false positive; no isolated Latin running head visible here.
- VII/169 and244 low-DPI candidates were examined at180. No new isolated header was established there;244 has a real bottom line cut (below). Edition reference e and small flecks are not themselves leaked running heads.

### Watermarks (all confirmed against source)

- **II/273-274, readers265-266, Aemilius Paullus, v1/321 printed308:** raw source bottom Google watermark is split by output pagination. Page273 has upper curved/stem fragments near bottom under `enim intelligebant, Philippum victum potentiam Macedo-`; page274 has clear gray `Digitized by Google`/logo material above and touching opening `num longe robustiorem et bellicosiorem reddidisse`. A heavy right source rule also spans273 and continues on274. This is new, distinct from W1/W2; proposed W4.
- **VIII/87, reader79, Brutus, v2/603 printed1180:** confirmed gray curved/stem logo fragment above `quod statuitis perficere, hortorque ne cunctimini; neque`. Raw source604 was inspected and confirms the following prose's printed1181; source603 watermark attribution follows the seam immediately before source604 opening. **Raw source603 still needs direct inspection** before the final report claims exact source watermark position; `src2-603.png` can be rendered or cache inspected. Proposed W5; output artifact unmistakably Google-shaped but source association should be finished.
- **VIII/128, reader120, Brutus, v2/623 printed1200:** confirmed faint gray curve at top of `Is quum videret barbaros quosdam quies in persequendo` (use shorter anchor `Is quum videret barbaros quosdam` if exact `quietes` glyph uncertain). It lies in a blank interval after `etiam optimus quisque pro Bruto propugnans cecidit.` Raw `src2-623.png` shows the complete Google logo at its page tail; source624 prose follows. Proposed W6.
- Full intentional Google notice at physical8 in all tomi remains exempt. No QR fragments or additional watermark examples were confirmed in the sweeps; absence is limited by resolution and faintness, not an absolute proof of cleanliness.

## Confirmed slicing and crop losses

### Actual horizontal page-boundary cuts

- **VI/52-53, readers44-45, Alexander, v2/213 printed812:** confirmed at **300DPI**. Tiny upper portions of `XXVIII. Omnino autem ita se erga barbaros superbe ge-` sit at the bottom of52; lower portions appear at top53. This is in D5's second occurrence. No claim that the whole line is absent: it is horizontally split between pages. `slice6-052/053.png`.
- **VII/244-245, readers236-237, Demosthenes/Cicero comparison, v2/480 printed1059:** confirmed at **300DPI**. Upper slivers of `IV. Exsilii vero calamitas turpis Demostheni accidit,` at bottom244, lower remainder at top245. `slice7-244/245.png`. Raw `src2-480.png` exists; verify source folio visually before finalizing (mapping is consistent with source sequence).
- All47 edge sheets inspected. No other horizontal cut was confirmed; avoid claiming pixel-perfect absence. Prior O8B is repaired at current II/239 around `obside-` / `bantur` within the page; full page was inspected, and neighboring edges inspected.

### Distinct Latin omissions found during continuity checks

These extend beyond merely awkward spacing and deserve explicit report entries (possibly a small supplemental continuity table within category5 or7, preserving eight main category tables). Do not repair them.

- **C1: I/68, reader52, Romulus, v1/38->39 printed25->26.** After `eo potentiæ progressuram.` output jumps across a large gap directly to source39 `educati, condendæ : nam hæc esse causa honestissima vi-`. Raw source38 contains **four intervening Latin lines**, opening `IX. Amulio interfecto rebusque compositis, neque Albæ` and ending `nore, ipsis consilium fuit seorsum urbis, ubi primum fuerant`. These four lines are missing at the join. Native output68 and fullsource38/39 visually inspected. OCR67/68/69 read; a full-volume exact/fuzzy OCR search did not find the opening elsewhere. Before final claim, optionally visually recheck neighboring output67/69; they clearly contain earlier/later source material, no chapterIX opening in OCR.
- **C2: II/207, reader199, Coriolanus, v1/291->292 printed278->279.** Output ends `Idque tempus longissimi luctus Numa de-`, then gap and `cum Æquis sociis atque amicis suis contendentes usque ad`. Raw source292 has **two intervening Latin lines**, `finierat, ut in ejus Vita scripsimus. (11) At res Volscorum` and `statim Marcium desideraverunt. Primo enim de principatu`. Both are absent at this join. Native207 and raw292 inspected. This is real lost continuation, not just copied margin.
- **C3: VI/229, reader221, Phocion, v2/299 printed896.** After repeated folio895 ending `Id quum impugnarent oratores postulatum,`, output jumps to `vos aut armis superare, aut amicitia superiorum uti.` The unique source896 first Latin line `a senatu dicere sententiam Phocion jussus, Jubeo, inquit,` is missing. Raw299 and output229 inspected. Full VI OCR search for `a senatu dicere`, `Phocion jussus`, `Jubeo` yielded none. Distinct from D6's second scan omitting its duplicate first line.
- **C4: VI/53, reader45, Alexander, v2/213 printed812:** **confirmed lateral clipping of many Latin line beginnings**, visible in300DPI source213 contribution (e.g. the line following first ends `ge-`, next begins visibly `rebat` with initial portion clipped; numerous subsequent leftmost characters cut along same vertical boundary). Raw213 exists but needs direct visual comparison to enumerate precise glyph loss. The slicingS1 and missing duplicate opening line are separate phenomena. Other lateral clipping is visible at I/68-70 and in second Otho opening; treat scope beyond the clear VI/53 example as unconfirmed unless source comparison performed.

## Gaps: corrected classifications and checked source continuity

The earlier checkpoint over-attributed several gaps to source-page seams. **I/36, I/46, and I/61 reproduce large chapter spacing already printed on the same source page. They are not confirmed copied-margin defects.** Keep their rules as separate findings.

### Confirmed copied-margin gaps with intact local continuation

- II/81 r73 Fabius Maximus: `quæ in sum-` -> `mitatibus montium sedebat`; v1/229->230, printed216->217. Full raw sources and nativeoutput inspected; no intervening Latin line, split word continues correctly. Gap ~130rows at180DPI.
- II/139 r131 Alcibiades: `non in plebis id quidem gra-` -> `tiam (neque enim se fidem iis habere), sed optimatum`; v1/257->258 printed244->245. Raw sources and nativeoutput inspected; no intervening line, correct split.
- II/203 r195 Coriolanus: `O ma-` -> `ter, ut mecum egisti!`; v1/289->290 printed276->277. Native203 inspected and source continuity corroborated by fresh source OCR/ledger. Raw290 not newly rendered this turn; cache available. Prior header mask fired, but large gap and heavy rule survive.
- II/219 r211 Timoleon: `seque ei` -> `crediderunt et belli ducem cum delegerunt`; v1/295->296 printed282->283. Native219 inspected. Header/rule mask succeeds, but ~77row interior gap remains. Source wording from OCR/ledger; raw seam still merits visual final check.
- II/223 r215 Timoleon: `diris imprecationibus se devoveri, cum` -> `deprecatum ivisset`; **v1/297->298 printed284->285** (corrects first checkpoint's suggested298->299). Full raw297/298 and native223 inspected. No missing intervening line; large gap/right divider remains even though head285 is removed.
- II/243 r235 Timoleon: `quod autem nemo Corinthiorum ipsorum neque periit,` -> `neque vulnus accepit`; v1/307->308 printed294->295. Native243 inspected; prior head mask succeeds but gap/right rule survives. Fresh source OCR/ledger matches; source307/308 raw visual check can still be completed if needed.
- II/260-261 r252-253 Timoleon: `nava-` -> `vit, ut eum perinde atque conditorem diligerent.`; v1/315->316 printed302->303. Native260/261 and raw315/316 inspected. Page261 has copied top blank interval and thin rightdivider; no missing line.
- II/290 r282 Aemilius Paullus: `Ceterum, rex Macedo-` -> `num, ut Polybius refert`; v1/329->330 printed316->317. Native290 and raw329/330 inspected; correct split, gap/heavy right rule survives.
- VIII/128 r120 Brutus: gap after complete ending from source623, then source624 opening; watermark and rule, seeW6. Prose resumes without an obvious missing local line; direct raw624 head review would finish continuity proof.
- IV/48 r40 Lucullus: after `ea nullam magnam rerum gerendarum materiam habere.` -> `(4) Maxime autem Pompeius in Hispania...`; source603->604 printed590->591. Native48 and source603 column inspected. Blank margin/rule at603tail supports copied seam; source604 opening still merits direct check before asserting no omitted line.

### Genuine printed spacing: do not flag as defect

All following native outputs inspected, then corresponding source columns/full pages inspected. They show the same within-source chapter/verse gaps, with no intervening Latin line:

| Output physical / reader | Source v1 physical / printed | Anchor after gap |
| --- | --- | --- |
| I/28 /12 | 19 /6 | `XIV. Porro Theseus quum res agere vellet` |
| I/36 /20 | 23 /10 | `XXI. Solvens ex Creta Delum appulit` after `et mœstitia esse.` (prior report's `mendacia` is wrong) |
| I/46 /30 | 28 /15 | `XXVIII. Hæc memoratu digna de Amazonibus fuere.` |
| I/61 /45 | 35 /22 | `IV. Erat juxta ficus` after `quoniam fratres appellant germanos.` |
| I/62 /46 | 35 /22 | `V. Alteram item colunt Larentiam` |
| I/66 /50 | 37 /24 | `VIII. At Faustulus` |
| I/71 /55 | 40 /27 | `XI. Romulus postquam Remum` |
| I/72 /56 | 40 /27 | `XII. Conditam urbem` |
| I/73 /57 | 41 /28 | `XIII. Urbe jam condita` |
| I/75 /59 | 42 /29 | `XIV. Ab Urbe condita` |
| I/77 /61 | 43 /30 | `XV. Inter virginum raptores` |
| I/83 /67 | 46 /33 | verse `At Capitolinam prope quæ Tarpeia colebat` |
| I/87 /71 | 48 /35 | `XX. Geminata urbe` |
| I/92 /76 | 50 /37 | `XXII. Fama est Romulum` |
| I/94 /78 | 51 /38 | `XXIV. Inde pestilentia ingruit` |
| I/100 /84 | 54 /41 | `XXVIII. Inter hæc memorant` |
| I/109 /93 | 58 /45 | `IV. At Romulo hoc primum` |
| I/111 /95 | 58 /45 | `VI. Ad hæc` |

**I/97 r81**, at `XXVI. Hoc novissimum, quod gessit Romulus, bellum`, was inspected on output, but the requested column53 does not contain XXVI (it is source52 instead). Source52 must be checked to close that printed-spacing classification. VIII/98 r90 chapterXXV spacing matches raw source609 column (printed1186); legitimate. V/263 and273, VI/129 metric candidates were inspected: ordinary verse/chapter spacing; no large copied-margin gap established. V263 folio245 is unprinted unitopening, infer only.

## Rules

Confirmed examples remain numerous inI and at source seams/repeats; a complete per-rule catalog has not been generated. Final report should distinguish representative confirmed examples from an exhaustive count. Native inspected examples:

- I/24 r8 v1/17 f4: long leftdivider, neighboring Greek scraps; visibly close but generally separated from Latin.
- I/28 r12 v1/19 f6; I/36 r20 v1/23 f10; I/46 r30 v1/28 f15: left rules through genuine printed chapter gaps; rules are defects separately from spacing.
- I/68 r52 v1/38 f25: left rule down through gap; following crop also cuts left Latin pixels (continuityC1).
- II/203,223,243,261,273,290: right rules, sources/folios above; most are visibly separated from Latin, no erase-safety measurement performed.
- VII/28-30 r20-22, Agis: conspicuous heavy right rules, first duplicate scan; 29 runs alongside prose and heading fragment. Source374->375->376, printed957->958->959.
- VI/225-227 r217-219, Phocion secondrepeat source297-298 fol894-895; VIII/229-230 r221-222, Aratus secondrepeat source673-674 fol1248-1249; VIII/287 r279, Otho secondrepeat source701 fol1274: broken right rules.
- VIII/87 r79 and128 r120: right rules through watermark/margin portions.
- VI/337 r329 has no visible heavy right rule at reviewed source361 mask neighborhood, while duplicate prose remains (D7).

## Prior repairs: visual checks completed at fresh output anchors

Read-only ledger has25 mask records/26 rectangles plus D1-D3. Several findingIDs have two source-page masks. A final verification table can group records by visible neighborhood while explicitly accounting for all25 records. Do not claim masks verified merely from configuration.

| Prior ID / source record(s) | Current output inspected | Result |
| --- | --- | --- |
| W1 source287 watermark +288 head; additional287 margin mask | II/199 r191, `sibi ad-`/`junctis, ad Volumniæ...` | Watermark/head absent; prose survives. |
| O1 source182 | I/371 r355, `non enim neque is jam exsul,`/`neque nos cives sumus` | Head/169 absent. |
| O1C source290 | **II/203 r195**, `O ma-`/`ter, ut mecum egisti!` | Head/277 absent, gap/right rule survives. II204 is not actual fresh seam. |
| W2/G4 source299 watermark and extra margin mask | II/227 r219, `inter`/`se per otium dividere insulam liceret.` | Watermark absent, seam compact. |
| O1B source281 two masks | II/187 r179, chapterXXII | Greekhead/268 absent; compact prose. |
| O1D/G8B source298 two masks | II/223 r215 | Head/285 absent; gap/rightdivider remains. |
| G5 source308 two masks | **II/243 r235** | Head/295 absent; gap/rightdivider remains. |
| O2 source550 | III/337 r329, `Haliartii perhibent,`/`habitasse apud ipsos` | Head/537 absent. |
| O3 source441 two rectangles (head and right rule) | **VII/163 r155**, `mœrore suo`/`conficiat, sed et levandus` | Head and rule absent; compact seam. |
| O4 source539 two mask records | **VII/365 r357**, `Eo impe-`/`trato, hostes irruentes` | Greekhead/rule absent; word continuation survives. |
| O6 source337 | **VI/308 r300**, `nonnulli etiam amici Pompeii` | Greekfragment/right rule absent. |
| O5 source530 | III/297 ending inspected before checkpoint and onedges;298 white separator at lowDPI | Furniture-only body page gone. Full180 white298 check still easy to do. |
| G1/O7 source296 | II/219 r211 | Obstructing rule absent; some gap remains. |
| G2/O7 source304 | **II/235** OCR locates `vestem excutiebat`; II/236-237 fullimages inspected | 236-237 now prose, no divider-only pages. **Full235 visual check still needed** to verify actual mask neighborhood, rather than neighboring pages alone. |
| O8B source306 | II/239 r231 | Full adjacent `obside-`/`bantur` line portions preserved in page; prior slicing absent at neighborhood. |
| G6 source470 | III/175 r167 | `strepitu aquæ`/`verba non percipientibus` adjacent top lines; old largegap absent. |
| O7 source361 | VI/337 r329 | Heavyright rule absent; D7 still repeats prose. |
| G3/O7 source625 | VIII/130-132 r122-124 | `subli-`/`dens, Eptum est` and later `partem secutus.` preserved; no divider-only page. |
| D1 excluded103-104 | V/195 r177 | `magno in`/`pretio habitus` contiguous; old return toEumenes absent. |
| D2 excluded369-370 | VII/20 r12 | `unius suffragii numero`/`excellerent qui antiquabant legem` contiguous; oldrepeat absent. |
| D3 excluded339-348 | VI/312 r304 | `ejusdem esse`/`dicens magna mala inferre et depellere` contiguous; old10page reset absent. |

No remaining old header leak was confirmed. Newly found leaks above are elsewhere. Latin at masked joins is occasionally tightly spaced; report a visual collision only if actually established, not from a generic impression.

## Endpoint report preparation

All68 endpoints remain visually confirmed to conclude complete sentences. `endings.csv` contains exact physical pages, reader folios, and unit IDs. First checkpoint lists per-tomus endpoint pages. Do not copy old2026-10-05 endpoint locations.

For final report, include all68 endpoint identifiers, preferably a table of unit/physical/reader/sourcefolio/status (short exact anchor per endpoint is ideal; at minimum a category8 table with clear representative anchors plus the full coverage ledger). **Source endpoint must not be guessed simply from nextunit start.** The ledger `scale.width_sampling_ranges` gives ranges ending at nextunit start, often sharedsourcepage, but not always actual lastprose page. Marius ends onv1/530 even though range ends531. `facsimile_units.json` is the actual unit table, and `last_page` overrides exist (e.g.Otho708). `closing_body_bands.json` has7 endpointbody overrides; it is not a complete68unit endpoint map. Use final-output OCR matching and sourcefinalprose to locate others. No new terminal truncation found despite confirmed interior omissions.

Useful printed v2 source-folio progression (visually established in earlier sweep, current duplicate pairimages, and rawsources; OCRCSV has digiterrors):

- Before D1: physicalp +607 (e.g41=648,67=674,71=678,102=709). D1 secondblock103-104 repeats708-709;105=710;106=711.
- 106-118: p+605;119-122 repeat720-723;123=724, then p+601 through212.
- 213-214 repeat812-813;215=814 then p+599 through296.
- 297-298 repeat894-895;299=896 then p+597 through338.
- 339-348 repeat926-935;349=936 then p+587 through360.
- 361-362 repeat946-947;363=948 then p+585 through368.
- 369-370 repeat952-953;371=954 then p+583 through376.
- 377-378 repeat958-959;379=960 then p+581 through464.
- 465-466 repeat1044-1045;467=1046 then p+579 through522.
- 523-524 repeat1100-1101;525=1102 then p+577 through672.
- 673-674 repeat1248-1249;675=1250 then p+575 through700.
- 701-702 repeat1274-1275;703=1276 then p+573 through708=1281.

v1 bodyfolio normally physicalp-13. Printed folio is often at farleft while chapter reference numbers appear at right; do not mistake `(19,20)` forfolio. SourcefolioCSV is rough embeddedOCR and sometimes wrong (102709 read700). Visual headers supersede arithmetic/OCR.

## Remaining work after compaction

1. Read both checkpoints and original prompt; continue, do not restart completed sweeps.
2. Finish easy raw-source checks: source2/603 watermark forW5, source2/375 exactGreekfragment, source2/480 sourcefolio forS2, source2/213 lateralclipping; source1/52 printedXXVIgap; sources1/295-296,289-290,307-308,603-604 and2/624 for remaining gapcontinuity qualifications. Rawcache is available for everypage; render under tmp ifneeded.
3. Verify currentII/235 at180 (G2mask actual `vestem excutiebat` neighborhood), III/298 white at180, neighboring I/67-69 forC1, II/207-208 forC2, VI/228-230 forC3. Most relevant pages already rendered, so no backgroundrender needed.
4. Prepare all68 endpointlocators/sourcefolios and faithful shortanchors without normalizingLatin. Do not make source-end assertions from widthsampling rangesalone.
5. Write the final newdated findingsreport with eightcategorytables, confirmed/unconfirmed limits, nineD4-D12 duplicates andjoin evidence, threewatermarks, twoheaderleaks plus Greekcolumnscraps, no divider-only bodypages, genuine-spacing nonfindings, survivingrules, twoslices, and completeendings. Include interior omissionsC1-C3 prominently; C4clipping if sourcecomparisonfinished. No implementationproposal.
6. Keep rulecatalog language honest: representative examplesconfirmed, not exhaustive tally. No-example-found wording for faint artifacts; endpointcomplete is a stronger verifiedclaim.
7. Add short priorities: distinctLatin loss / slicing first, duplicate readingorder, watermarks/heads, margin gaps/rules; do not recommend compressing genuineprinted chapterspace.
8. Final reportcheck: eightcategories, all68endpoints, all25maskrecords accountedfor, dual locators, sourcefolios, faithfulanchors, correctVoffset18; `git diff --check` andstatus. Only newreport/checkpoint and diagnosticfiles should be added; existing AGENTS.md modification and originalprompt are userchanges. Preserve everything underllm_artifacts.

At checkpointcreation (15:12Eastern), gitstatus: modifiedAGENTS.md; untrackedoriginalprompt, firstcheckpoint, tmp/. No finalfindingsreport yet. This newcheckpoint is the only durablefile authored in this continuation.
