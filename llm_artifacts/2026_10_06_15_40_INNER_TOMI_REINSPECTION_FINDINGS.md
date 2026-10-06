# Inner-tomi reinspection findings

## Scope, method, and reference conventions

This is the completed report-only reinspection requested by **Prompt for LLM Agents** in [the original prompt](2026_10_06_13_49_INNER_TOMI_REINSPECTION_PROMPT.md), continued through [the first checkpoint](2026_10_06_14_27_INNER_TOMI_REINSPECTION_CHECKPOINT.md) and [the second checkpoint](2026_10_06_15_12_INNER_TOMI_REINSPECTION_CHECKPOINT.md). It inspects the rebuilt `build_facsimile/tomus_1_inner.pdf` through `tomus_8_inner.pdf` on 2026-10-06. Findings below describe that build, not the older pagination in the 2026-10-05 report. No repairs were performed and no Latin, source code, fixture, unit table, transcription cache, or PDF was changed.

The strongest residual findings are **nine additional repeated scan blocks**, **four distinct omissions totaling eight source Latin lines**, lateral clipping of Latin, **two horizontal text-line slices**, **three additional body watermarks**, and **two running-head leaks**. Source margin gaps, divider rules, and neighboring Greek-column scraps also survive. All **68 body-section endings** contain their complete terminal prose; this does not establish completeness of every interior passage. No divider-only or prose-free body-image page was found in the current build.

The inspection covered all eight volumes in labeled 45-DPI contact sheets; all 2,912 physical PDF pages were also rendered at 180 DPI. All 2,678 body-image pages received OCR and top/bottom edge triage. All 47 edge sheets, every source running-head sheet across the body ranges of both source volumes, all candidate duplicate source pairs, all duplicate output boundaries, all nine hypothetical surviving source joins, and all 68 section endings were visually inspected. Suspect horizontal cuts were confirmed on full adjacent pages at 300 DPI. Source comparisons use the original source PDFs and their cached 300-DPI renders. OCR and ink/gap metrics were used to locate candidates, not as authoritative Latin or proof of a defect.

Background render/OCR/edge jobs recorded in the checkpoints completed successfully. The two remaining source-render/join sessions were checked on this continuation and had completed. Counts on disk agree with the expected 2,912 full renders, 2,678 body OCR files, and 47 edge sheets. Existing process listings alone were not treated as evidence that an inspection job remained active. No completed sweep was restarted.

| Tomus | Physical PDF pages | Body-image pages | Last body page | Reader folio of last body page |
| --- | ---: | ---: | ---: | ---: |
| I | 416 | 376 | 398 | 382 |
| II | 448 | 411 | 423 | 415 |
| III | 416 | 395 | 411 | 403 |
| IV | 128 | 109 | 118 | 110 |
| V | 384 | 354 | 374 | 356 |
| VI | 352 | 331 | 341 | 333 |
| VII | 448 | 409 | 419 | 411 |
| VIII | 320 | 293 | 303 | 295 |

**Locator convention:** `VI/53, r45` means one-based physical page 53 of the inner PDF, printed reader folio 45. Reader folios equal physical page minus 16 in I, minus **18 in V**, and minus 8 elsewhere. A unit-opening folio can be unprinted; any such locator below is explicitly called inferred. `v2/213, f812` means physical page 213 of `Ploutarchou_Bioi_v2.pdf`, printed source folio 812. Source folios come from the visually inspected running-head progression and source comparisons, not unreliable OCR digit readings. These locators are not imposed-sheet page numbers.

Quoted Latin is a short visual landmark, preserving the visible spelling and ligatures. Fragments separated by an arrow identify a seam and are not a reconstructed transcription. A missing line is quoted from the source image; it is not an instruction to edit the transcription. Finding IDs D1-D3 and W1-W3 remain reserved for the earlier findings. New IDs identify this report's observations and do not renumber historical records.

The rule list is a representative catalog of confirmed surviving rules, not an exhaustive count of every rule segment. “No example found” for faint furniture or slicing is limited by the reviewed resolution. The inspection is a broad visual reinspection with targeted source continuity checks, not a character-by-character collation of all Latin. Diagnostic evidence is preserved under `tmp/pdfs/reinspection/`; the final report is self-contained, while the checkpoints give a fuller diagnostic inventory.

## 1. Repeated, duplicated, or out-of-order scans

All nine blocks below repeat printed source folios and Latin prose. The two scans often differ in skew and divider position; identical raster hashes are not required. The physical output ranges are **inclusive contribution ranges**. Some share a boundary page because unique neighboring prose and both occurrences occupy that page. They are not whole pages that can safely be discarded.

| ID | Life and current output locations | Source occurrences and printed folios | Latin landmark and visible repetition | Assessment |
| --- | --- | --- | --- | --- |
| D4 | Agesilaus: V/217-225, r199-207; V/225-233, r207-215 | v2/115-118 and 119-122; f720-723 twice | `XV. Mota Asia`; chapters XV-XX recur. XV restarts within 225 after `ex inimicis familiares redegit, atque`. Repeat ends within 233 before `ita confecit`. | **Confirmed.** Four source pages repeat. |
| D5 | Alexander: VI/48-52, r40-44; VI/52-56, r44-48 | v2/211-212 and 213-214; f812-813 twice | First starts `Alexander, Philippine interfectores`; second starts later at `an omnibus esset dominaturus hominibus` within 52. Chapters XXVIII-XXX recur; repeat ends within 56 before `magnum et regis dextram reveritus`. | **Confirmed.** Two source pages repeat. The second occurrence omits its first source Latin line and clips left line beginnings. Its omitted duplicate opening survives in the first occurrence; this differs from distinct omissions C1-C3/C5. See S1 and C4. |
| D6 | Phocion: VI/221-225, r213-217; VI/225-229, r217-221 | v2/295-296 and 297-298; f894-895 twice | First starts `eorum, qui Thebis huc confugerunt`; second starts `Thebas lugere.` within 225. Chapters XVIII-XXI recur. | **Confirmed.** Two source pages repeat. Second scan's first Latin line is omitted but survives in the first occurrence. The unique first line of the following source page is also missing: C3. |
| D7 | Cato Minor: VI/332-336, r324-328; VI/337-341, r329-333 | v2/359-360 and 361-362; f946-947 twice | `quoque, inquit, decrevistis virum`; remainder of LXIX through LXXIII recurs. First ends at bottom 336; second starts at top 337 and ends in the first two lines of 341, before `occubuit.` | **Confirmed.** Separate from repaired D3 (f926-935). The first contribution does not extend to 337. |
| D8 | Agis: VII/29-32, r21-24; VII/32-36, r24-28 | v2/375-376 and 377-378; f958-959 twice | `mum Amphares, auctoritate muneris sui Agidi manum`; XX-XXI recur, ending `ceteris credidisset.` First starts on 29 after page 28's `Ibi de-`; repeat starts within 32 and ends with the three concluding lines on 36. | **Confirmed.** Separate from repaired D2 (f952-953). First contribution begins on 29, not 28. |
| D9 | Cicero: VII/208-212, r200-204; VII/212-216, r204-208 | v2/463-464 and 465-466; f1044-1045 twice | `voce maris cognita exclamavit`; XXIX-XXXI recur. Reset within 212 after `multa egisset, sed`; repeat ends within 216 before `tum Cæsare socero`. | **Confirmed.** Two source pages repeat. |
| D10 | Antonius: VII/328-332, r320-324; VII/332-336, r324-328 | v2/521-522 and 523-524; f1100-1101 twice | `in foro duceretur.`; XV-XVIII recur. Reset within 332 after `titulum ei et`; repeat ends within 336 before `honorem Imperatoris conservans.` | **Confirmed.** Two source pages repeat. |
| D11 | Aratus: VIII/225-229, r217-221; VIII/229-233, r221-225 | v2/671-672 and 673-674; f1248-1249 twice | `a se gestarum occasionem`; XLIV-XLVI recur. Reset within 229 after `in Pe-`; repeat ends within 233 before `loponnesum misit`. | **Confirmed.** Two source pages repeat. |
| D12 | Otho: VIII/283-287, r275-279; VIII/287-291, r279-283 | v2/699-700 and 701-702; f1274-1275 twice | First starts `erant, laboremque ob consuetudinem`; second starts later within 287 at the clipped fragment `rimum temporis`. VI-VIII recur. Repeat ends within 291 before `jam essent in itinere.` | **Confirmed.** Two source pages repeat. Some opening lines do not recur but survive in the first occurrence; the second opening has lateral clipping. |

These are 20 additionally repeated source pages in nine blocks. No other new repeated block was confirmed in the body source ranges, including v1. Extra concluding Otho scans v2/709-710 are outside the pinned current unit ending at 708 and do not survive as an output duplicate.

The prompt also requested inspection of the join that would survive omission of each second source block. All nine following source joins were visually compared in full Latin-column strips. They connect the expected prose or the next Life without loss of a distinct source block. This is source-sequence evidence, not a repair proposal or proof that the current output seam already preserves every line; C3 shows why that distinction matters.

| Duplicate | Hypothetical source join | Printed source folios | Visible continuity |
| --- | --- | --- | --- |
| D4 | v2/118 -> 123 | f723 -> 724 | `atque` -> `ita confecit` |
| D5 | v2/212 -> 215 | f813 -> 814 | Darius/Mithræ question continues with `magnum et regis dextram reveritus`. |
| D6 | v2/296 -> 299 | f895 -> 896 | `Id quum impugnarent oratores postulatum,` -> `a senatu dicere sententiam Phocion jussus, Jubeo, inquit,` -> `vos aut armis superare`. The middle source line is absent in the current output. |
| D7 | v2/360 -> 363 | f947 -> 948 | `hostibus ostendisset,` -> `occubuit.` |
| D8 | v2/376 -> 379 | f959 -> 960 | Agis ends `ceteris credidisset.`; Cleomenes begins as the next Life. |
| D9 | v2/464 -> 467 | f1045 -> 1046 | `sed` -> `tum Cæsare socero` |
| D10 | v2/522 -> 525 | f1101 -> 1102 | `et` -> `honorem Imperatoris conservans.` |
| D11 | v2/672 -> 675 | f1249 -> 1250 | `in Pe-` -> `loponnesum misit` |
| D12 | v2/700 -> 703 | f1275 -> 1276 | `quæ` -> `jam essent in itinere.` |

D1-D3 exclusions were checked at their fresh output prose seams, not inferred solely from configuration: V/195 r177 (`magno in` -> `pretio habitus`), VII/20 r12 (`unius suffragii numero` -> `excellerent qui antiquabant legem`), and VI/312 r304 (`ejusdem esse` -> `dicens magna mala inferre et depellere`). Their old resets are absent. Full prior-repair accounting appears below.

## 2. Leaked running heads, source folios, and Greek fragments

| ID | Location, source folio, and Latin anchor | Visible finding | Assessment |
| --- | --- | --- | --- |
| H1 | Philopoemen III/123, r115; v1/448, f435; `superbe Gythium oppugnabant,` -> `subito illis nihil tale exspectantibus` | Clipped running-head fragment `LOPŒMEN.` and isolated **435** inside the prose flow; Greek scraps at left below it. Raw source has the Philopoemen head and folio. | **Confirmed new head/folio leak.** Initial fragment edge is clipped; the report does not expand it into the complete head. |
| H2 | Agis VII/29, r21; v2/375, f958; above `mum Amphares, auctoritate muneris sui Agidi manum` | Isolated **Σ.**, with additional clipped left stroke, from the Greek source head ΑΓΙΣ. It sits below the generated AGIS head, beside a heavy right divider. | **Confirmed new Greek head leak.** Source 375 and current output were directly compared. |
| F1 | Theseus I/24, r8; v1/17, f4; `labatur, apprehendentem ipsum` | Neighboring Greek-column slivers beside the long left divider. | **Confirmed foreign-column furniture.** OCR's isolated `TSE` candidate is not a Latin head leak. |
| F2 | Pyrrhus III/173, r165; v1/470, f457; `Traditum est memoriæ` | Extensive neighboring Greek-column scraps at left of the Latin opening. | **Confirmed foreign-column furniture.** These are vertical column scraps, not a running-head word. The edition's b/c/d reference letters are not themselves defects. |

All isolated OCR capital/number candidates were reviewed against images. Generated Life titles, chapter numbers, and edition reference letters were excluded from the leak list. No surviving instance of the old masked Latin or Greek heads was confirmed at the actual current mask neighborhoods. Small flecks at VII/169 and VII/244 did not establish an additional running head. “No additional head found” remains limited by resolution and OCR's inability to recognize some cropped/faint Greek fragments.

## 3. Google watermarks in body prose

| ID | Location, source folio, and Latin anchor | Visible finding | Assessment |
| --- | --- | --- | --- |
| W4 | Aemilius Paullus II/273-274, r265-266; v1/321, f308; `potentiam Macedo-` -> `num longe robustiorem` | Upper gray logo curves/stems at the foot of 273, then recognizable Google watermark material above/touching the first line on 274. Heavy right divider continues through the area. | **Confirmed new watermark.** Source 321 bottom watermark is split by output pagination. |
| W5 | Brutus VIII/87, r79; watermark from v2/603, f1180, before v2/604, f1181; `quod statuistis perficere` | Gray Google-shaped curves/stems immediately above the opening line, with a right divider and copied blank interval. | **Confirmed new watermark.** Direct raw-source 603 review confirms the complete bottom Google logo; source 604 supplies the following prose. |
| W6 | Brutus VIII/128, r120; watermark from v2/623, f1200, before v2/624, f1201; `etiam optimus quisque pro Bruto propugnans cecidit.` -> `Is quum videret barbaros quosdam` | Faint gray curved logo fragment in the gap above the resumed Latin, beside right divider fragments. | **Confirmed new watermark.** Source 623 has the complete bottom logo. Source 624 comparison also establishes omitted Latin line C5. |
| W1 / W2 recheck | Coriolanus II/199, r191, v1/287-288, f274-275, `sibi ad-` -> `junctis`; Timoleon II/227, r219, v1/299, f286, `inter` -> `se per otium dividere insulam liceret.` | Earlier body watermarks are absent at the fresh inspected seams. | **Confirmed absent at these neighborhoods.** This does not imply every watermark is removed: W4-W6 survive elsewhere. |
| W3 exemption | Physical page 8 in all eight tomi; front matter, no body Latin anchor or source prose folio applies | Full Google reproduction notice/logo/QR page. | **Intentional retained front matter; not a defect.** |

No additional body QR fragment or watermark was confirmed in the completed sweeps. Gray curves can be too faint for pixel thresholds, so this is not an absolute absence claim.

## 4. Divider-only or effectively blank body pages

| ID / coverage | Current location and anchor | Visible result | Assessment |
| --- | --- | --- | --- |
| B1 / prior G2 | Timoleon II/235-237, r227-229; v1/304, f291, mask neighborhood `vestem excutiebat` on 235 | All three pages contain flowing prose. The exact235 mask neighborhood was checked at 180 DPI; neighboring236-237 were also inspected. | **Confirmed prior divider-only pages absent.** |
| B2 / prior G3 | Brutus VIII/130-132, r122-124; v2/625-626, f1202-1203; `subli-`, `partem secutus.`, and closing prose | Current pages carry prose; no divider-only131 remains. | **Confirmed prior divider-only page absent.** |
| B3 / prior O5 | Marius III/297, r289, v1/530, f517, `mortem sibi ipse conscivit.`; III/298 white separator | Ending 297 contains the complete conclusion. Full180-DPI298 is entirely white. | **Confirmed old furniture-only body page absent.** White298 is legitimate recto separation, not a blank body page. |
| B4 / all-tomus sweep | All 2,678 body-image pages; endpoint coverage in category8 | No body page with generated heading/folio plus only a source divider and no prose was found. Sparse complete endings, white recto separators, and signature padding were distinguished. | **No residual example found at reviewed resolution.** |

## 5. Large copied-margin gaps and interior continuity

Source comparisons distinguish genuine printed chapter spacing from blank source-page margins copied across a seam. The confirmed copied-margin gaps below retain the local continuation except where the separate C findings establish missing Latin.

| ID | Current physical / reader location | Source physical pages / printed folios; visual anchor | Visible finding and assessment |
| --- | --- | --- | --- |
| M1 | Fabius Maximus II/81, r73 | v1/229 ->230, f216 ->217; `quæ in sum-` -> `mitatibus montium sedebat` | **Confirmed copied-margin gap**, about130 rows at 180 DPI. Split word continues; no intervening source Latin line. |
| M2 | Alcibiades II/139, r131 | v1/257 ->258, f244 ->245; `gra-` -> `tiam` | **Confirmed copied-margin gap.** Local split-word continuation is intact. |
| M3 | Coriolanus II/203, r195 | v1/289 ->290, f276 ->277; `O ma-` -> `ter, ut mecum egisti!` | **Confirmed copied-margin gap and right rule.** Old head277 is removed. Raw289/290 directly confirm the uninterrupted split word. |
| M4 | Timoleon II/219, r211 | v1/295 ->296, f282 ->283; `seque ei` -> `crediderunt et belli ducem cum delegerunt` | **Confirmed residual copied-margin gap**, about77 rows at 180 DPI. Old head/rule mask succeeds; no Latin line between these source pieces. |
| M5 | Timoleon II/223, r215 | v1/297 ->298, f284 ->285; `cum` -> `deprecatum ivisset` | **Confirmed copied-margin gap and right divider.** Old head285 is absent. Correct source seam is297/298, not298/299. No intervening source Latin line. |
| M6 | Timoleon II/243, r235 | v1/307 ->308, f294 ->295; `neque periit,` -> `neque vulnus accepit` | **Confirmed copied-margin gap and right rule.** Old head295 is absent; raw307/308 confirm intact local continuation. |
| M7 | Timoleon II/260-261, r252-253 | v1/315 ->316, f302 ->303; `nava-` -> `vit, ut eum perinde atque conditorem diligerent.` | **Confirmed copied top margin on 261 and right divider.** Split word is intact;261 contains prose. |
| M8 | Aemilius Paullus II/290, r282 | v1/329 ->330, f316 ->317; `Macedo-` -> `num, ut Polybius refert` | **Confirmed copied-margin gap and heavy right rule.** Split word continues without an intervening line. |
| M9 | Lucullus IV/48, r40 | v1/603 ->604, f590 ->591; `ea nullam magnam rerum gerendarum materiam haberet.` -> `(4) Maxime autem Pompeius in Hispania` | **Confirmed copied-margin seam and divider.** Raw603/604 comparison confirms no omitted local Latin line. |
| M10 | Brutus VIII/87, r79 | v2/603 ->604, f1180 ->1181; `vos feliciter` -> `quod statuistis perficere` | **Confirmed copied-margin gap**, including W5 and right rule. Source 604 starts with the visible continuation; no local omitted line established here. |
| M11 | Brutus VIII/128, r120 | v2/623 ->624, f1200 ->1201; `etiam optimus quisque pro Bruto propugnans cecidit.` -> `Is quum videret barbaros quosdam` | **Confirmed gap, watermark, and rule with a missing first source Latin line.** See C5; this seam must not be recorded as intact. |

### Distinct Latin omissions and lateral clipping

The following are fidelity defects, not merely awkward spacing. Four distinct omissions total **eight source Latin lines**. The count excludes the opening lines missing only from a second duplicate occurrence, which survive in the first occurrence. Neighboring output pages were visually checked for C1, C2, C3, and C5; source comparisons establish the missing material at each seam. Full OCR searches support localization but are not the primary proof.

| ID | Current location and source folios | Source-to-output evidence | Assessment |
| --- | --- | --- | --- |
| C1 | Romulus I/68, r52; v1/38 ->39, f25 ->26 | Output goes from `eo potentiæ progressuram.` to `educati, condendæ`. Source 38 has four intervening Latin lines, beginning `IX. Amulio interfecto rebusque compositis, neque Albæ` and ending `nore, ipsis consilium fuit seorsum urbis, ubi primum fuerant`. Output67-69 contain earlier/later prose, not those lines. | **Confirmed omission of four distinct source Latin lines**, including chapter IX's opening. |
| C2 | Coriolanus II/207, r199; v1/291 ->292, f278 ->279 | Output `Numa de-` jumps to `cum Æquis sociis atque amicis suis`. Source 292 first has `finierat, ut in ejus Vita scripsimus. (11) At res Volscorum` and `statim Marcium desideraverunt. Primo enim de principatu`. Output208 has only the genuine terminal continuation. | **Confirmed omission of two distinct source Latin lines.** The split `de-` is left without its expected continuation. |
| C3 | Phocion VI/229, r221; v2/298 ->299, f895 ->896 | After `Id quum impugnarent oratores postulatum,` output jumps to `vos aut armis superare`. Source 299 first contains `a senatu dicere sententiam Phocion jussus, Jubeo, inquit,`. Neighbors228/230 and the full VI OCR search do not supply it. | **Confirmed omission of one distinct source Latin line.** Separate from D6's incomplete duplicate opening. |
| C4 | Alexander VI/53, r45; v2/213, f812 | Many source line beginnings extend left of the output's vertical crop boundary. Source `rebat, ut qui deo parente` loses its initial edge; subsequent beginnings including `Atheniensibus scribens` and `et nobilem urbem` are clipped. Full300-DPI output and raw213 were compared. | **Confirmed lateral clipping of numerous Latin line beginnings.** This is distinct from horizontal slice S1. Exact total glyph count was not measured. |
| C5 | Brutus VIII/128, r120; v2/623 ->624, f1200 ->1201 | After source 623's `etiam optimus quisque pro Bruto propugnans cecidit.` output resumes at source 624's second Latin line `Is quum videret barbaros quosdam`. Source 624 first line is **`L. Erat inter amicos Bruti Lucilius quidam, vir fortis.`** Output127/128/129 and OCR do not contain it. | **Confirmed omission of one distinct source Latin line**, including chapter L marker and introduction of Lucilius. Found during this final continuation. |
| C6 | Romulus I/68-69, r52-53; v1/39, f26; `educati, condendæ` and `raptas.` | The left boundary cuts initial glyphs in the source 39 contribution. Source 39 shows complete `educati` and `raptas`; current68/69 clip their leftmost letters. | **Confirmed lateral clipping at these pages.** The full extent into additional Romulus pages has not been counted. |
| C7 | Otho VIII/287, r279; v2/701, f1274; clipped `rimum temporis` | Second duplicate opening loses its leftmost glyph and begins later than the source opening. | **Confirmed clipped repeat opening; unconfirmed total extent of lateral loss.** Distinct source opening survives in first occurrence D12; no additional unique-line omission is asserted. |

### Genuine printed spacing: retained non-findings

The large gaps at **I/36, I/46, and I/61 are within the original printed source pages**, not copied page margins. Direct source comparison corrects the earlier tentative classification and the prior report's mistaken `mendacia` anchor atI/36: the visible text is `mœstitia`. Their divider rules remain defects independently of the spacing.

| Current physical / reader | Source physical / printed folio | Anchor after the genuine chapter/verse gap | Assessment |
| --- | --- | --- | --- |
| I/28, r12 | v1/19, f6 | `XIV. Porro Theseus` | Printed chapter spacing; retain. |
| I/36, r20 | v1/23, f10 | `XXI. Solvens ex Creta Delum appulit`, after `et mœstitia esse.` | Printed chapter spacing; retain. |
| I/46, r30 | v1/28, f15 | `XXVIII. Hæc memoratu digna` | Printed chapter spacing; retain. |
| I/61-62, r45-46 | v1/35, f22 | `IV. Erat juxta ficus`; `V. Alteram item colunt Larentiam` | Both within-source chapter gaps; retain. |
| I/66, r50 | v1/37, f24 | `VIII. At Faustulus` | Printed chapter spacing; retain. |
| I/71-72, r55-56 | v1/40, f27 | `XI. Romulus postquam Remum`; `XII. Conditam urbem` | Printed chapter spacing; retain. |
| I/73, r57 | v1/41, f28 | `XIII. Urbe jam condita` | Printed chapter spacing; retain. |
| I/75, r59 | v1/42, f29 | `XIV. Ab Urbe condita` | Printed chapter spacing; retain. |
| I/77, r61 | v1/43, f30 | `XV. Inter virginum raptores` | Printed chapter spacing; retain. |
| I/83, r67 | v1/46, f33 | `At Capitolinam prope quæ Tarpeia colebat` | Printed verse spacing; retain. |
| I/87, r71 | v1/48, f35 | `XX. Geminata urbe` | Printed chapter spacing; retain. |
| I/92, r76 | v1/50, f37 | `XXII. Fama est Romulum` | Printed chapter spacing; retain. |
| I/94, r78 | v1/51, f38 | `XXIV. Inde pestilentia ingruit` | Printed chapter spacing; retain. |
| I/97, r81 | v1/52, f39 | `XXVI. Hoc novissimum, quod gessit Romulus, bellum` | Source 52 directly checked on this continuation; genuine printed chapter spacing. |
| I/100, r84 | v1/54, f41 | `XXVIII. Inter hæc memorant` | Printed chapter spacing; retain. |
| I/109 and111, r93 and95 | v1/58, f45 | `IV. At Romulo hoc primum`; `VI. Ad hæc` | Printed chapter spacing; retain. |
| VIII/98, r90 | v2/609, f1186 | chapter XXV | Printed chapter spacing; retain. |

Metric candidates V/263 (r245 inferred on unit opening), V/273 r255, and VI/129 r121 were inspected and did not establish a large copied-margin defect; ordinary verse/chapter spacing accounted for the flagged intervals. Old G6 atIII/175 r167, v1/470 f457, now has `strepitu aquæ` and `verba non percipientibus` on adjacent top lines; the old large gap is absent.

## 6. Surviving vertical divider rules

Rules below are source scan furniture. “Separated” describes visible spacing at the reviewed neighborhood and is not an erase-safety measurement. No mask rectangle or repair implementation is proposed.

| ID | Current physical / reader location | Source / printed folio; Latin anchor | Visible finding and assessment |
| --- | --- | --- | --- |
| R1 | I/24, r8 | v1/17, f4; `labatur, apprehendentem ipsum` | **Confirmed long left rule**, close to Latin and Greek scraps; generally a visible separation. |
| R2 | I/28, r12; I/36, r20; I/46, r30 | v1/19 f6,23 f10,28 f15; XIV/XXI/XXVIII anchors in spacing table | **Confirmed left rules through genuine printed chapter gaps.** The rules and spacing have different assessments. |
| R3 | I/68, r52 | v1/38-39, f25-26; `eo potentiæ progressuram.` | **Confirmed left rule through gap**, followed by Latin crop lossC1/C6. Pixels near prose require particular care in any later review. |
| R4 | II/203, r195 | v1/290, f277; `ter, ut mecum egisti!` | **Confirmed heavy right rule**, generally separated from prose; M3. |
| R5 | II/223, r215; II/243, r235; II/261, r253 | v1/298 f285,308 f295,316 f303; `deprecatum ivisset`, `neque vulnus accepit`, `vit, ut eum` | **Confirmed right rules** in copied-margin areas; visible separation at reviewed neighborhoods. |
| R6 | II/273-274, r265-266 | v1/321, f308; `Macedo-` / `num longe robustiorem` | **Confirmed heavy right rule**, extending through blank/watermark region across pagination; W4. |
| R7 | II/290, r282 | v1/330, f317; `num, ut Polybius refert` | **Confirmed heavy right rule** beside copied seam; M8. |
| R8 | IV/48, r40 | v1/603-604, f590-591; `Maxime autem Pompeius` | **Confirmed divider furniture** in copied seam; M9. |
| R9 | VII/28-30, r20-22 | v2/374-376, f957-959; `Ibi de-` / `mum Amphares` | **Confirmed conspicuous heavy right rules**, including alongside Agis prose and H2 on 29. The repeated scan also clips some Latin edges. |
| R10 | VI/225-227, r217-219 | v2/297-298, f894-895; `Thebas lugere.` | **Confirmed broken right rules** in second Phocion occurrence D6. |
| R11 | VIII/229-230, r221-222 | v2/673-674, f1248-1249; `a se gestarum occasionem` | **Confirmed broken right rules** in second Aratus occurrence D11. |
| R12 | VIII/287, r279 | v2/701, f1274; `rimum temporis` fragment | **Confirmed broken right rule** at Otho repeat D12. |
| R13 | VIII/87, r79; VIII/128, r120 | v2/603-604 f1180-1181;623-624 f1200-1201; anchorsW5/W6 | **Confirmed right rule segments** through margin/watermark areas. |
| R14 recheck | VI/337, r329 | v2/361, f946; `quoque, inquit, decrevistis virum` | Prior heavy right-rule mask succeeds at reviewed neighborhood; **no heavy rule visible there**. DuplicateD7 nevertheless survives. |

Further divider fragments are visible across the full-volume sweeps. The table records confirmed representative locators rather than claiming that these fourteen grouped rows enumerate every segment. Where lateral Latin clipping is confirmed, it is recorded separately; no conclusion that all rules abut Latin or that all are safely separated is warranted.

## 7. Raster page breaks slicing through Latin lines

| ID | Physical / reader boundary | Source / printed folio; Latin line | Visible finding | Assessment |
| --- | --- | --- | --- | --- |
| S1 | Alexander VI/52-53, r44-45 | v2/213, f812; `XXVIII. Omnino autem ita se erga barbaros superbe ge-` | Tiny upper portions of the line sit at the foot of 52; lower portions appear at top53. | **Confirmed horizontal line slice at 300 DPI.** The line is split, not wholly absent. D5 second occurrence; C4 separately clips its left side. |
| S2 | Demosthenes/Cicero comparison VII/244-245, r236-237 | v2/480, f1059; `IV. Exsilii vero calamitas turpis Demostheni accidit,` | Upper letter slivers at foot244, lower remainder at top245. | **Confirmed horizontal line slice at 300 DPI.** Raw480 source folio and line directly verified. |
| O8B recheck | Timoleon II/239, r231 | v1/306, f293; `obside-` / `bantur` | Both portions of the local word/line survive within the current page. | **Confirmed old slicing absent at the repaired neighborhood.** |
| All-tomus edge sweep | All 2,678 body pages,47 edge sheets | Source locators differ by page; confirmed examples above | No other horizontal slice was confirmed. | **No additional example found at reviewed resolution.** Fixed44-pixel diagnostic strips can themselves crop a second line; the findings above were confirmed on full pages. |

Lateral clipping C4/C6/C7 is a different failure from horizontal page-boundary slicing and must not be hidden under a statement that only two fidelity defects remain.

## 8. Section-ending completeness

All **68** current body-section endings were inspected at 180 DPI, then their terminal prose was matched against source scans for the endpoint ledger below. All conclude with the complete source terminal sentence/prose; no terminal truncation was confirmed. Short anchors below are visually checked terminal landmarks, not OCR output. Source endpoints are actual matched prose pages, not guesses from the next unit's start or width-sampling range. In particular, Marius ends onv1/530 rather than531; the Sertorius/Eumenes comparison ends on survivingv2/105 rather than the excluded duplicate104.

Every ledger row has assessment **Confirmed complete at endpoint**. This means no missing terminal continuation, not that every earlier line in the Life is present or unique. Coriolanus, Phocion, and Brutus end completely despite confirmed interior omissions; Agis and Cato Minor end completely despite surviving duplicates. Legitimately sparse endings are retained non-findings. At Otho's final pagination, `Ca-` at the preceding page continues with `cina` on VIII/303; the complete terminal `se conjunxerunt.` survives.
| Endpoint ID | Unit | Physical inner page | Reader folio | Source physical / printed folio | Short terminal Latin anchor | Assessment |
| --- | --- | --- | ---: | --- | --- | --- |
| E01 | theseus | I/56 | 40 | v1/33, f20 | `appellamus.` | Confirmed complete |
| E02 | romulus | I/105 | 89 | v1/56, f43 | `homines non ultra visum.` | Confirmed complete |
| E03 | comparatio thesei et romuli | I/113 | 97 | v1/59, f46 | `non fuisse diis cordi natales Thesei` | Confirmed complete |
| E04 | lycurgus | I/164 | 148 | v1/83, f70 | `Atque hæc de Lycurgo.` | Confirmed complete |
| E05 | numa | I/203 | 187 | v1/102, f89 | `siquidem traditur fulmine percussus conflagrasse.` | Confirmed complete |
| E06 | comparatio lycurgi et numae | I/213 | 197 | v1/106, f93 | `redegit atque composuit.` | Confirmed complete |
| E07 | solon | I/259 | 243 | v1/128, f115 | `tum Aristoteles etiam philosophus.` | Confirmed complete |
| E08 | poplicola | I/291 | 275 | v1/143, f130 | `mortuus inde aufertur.` | Confirmed complete |
| E09 | comparatio solonis et poplicolae | I/298 | 282 | v1/146, f133 | `jungeret, et omnem belli apparatum tolleret.` | Confirmed complete |
| E10 | themistocles | I/340 | 324 | v1/167, f154 | `cum degeremus una apud Ammonium philosophum.` | Confirmed complete |
| E11 | camillus | I/398 | 382 | v1/195, f182 | `dolorem attulerit.` | Confirmed complete |
| E12 | pericles | II/61 | 53 | v1/220, f207 | `omni remedio superior evaderet.` | Confirmed complete |
| E13 | fabius maximus | II/102 | 94 | v1/240, f227 | `quæ vitæ responderent, invenit.` | Confirmed complete |
| E14 | comparatio periclis et fabii | II/106 | 98 | v1/242, f229 | `periorem habuerunt.` | Confirmed complete |
| E15 | alcibiades | II/160 | 152 | v1/268, f255 | `proripientem telis confecisse.` | Confirmed complete |
| E16 | marcius coriolanus | II/208 | 200 | v1/292, f279 | `spondent s.` | Confirmed complete |
| E17 | comparatio alcibiadis et coriolani | II/215 | 207 | v1/294, f281 | `honestatisque negligentissimo homine.` | Confirmed complete |
| E18 | timoleon | II/266 | 258 | v1/318, f305 | `citate exegerunt.` | Confirmed complete |
| E19 | aemilius paulus | II/318 | 310 | v1/343, f330 | `Hi mores et vita Æmilii Pauli fuisse dicuntur.` | Confirmed complete |
| E20 | comparatio timoleontis et aemilii | II/321 | 313 | v1/344, f331 | `pusilli tamen animi est.` | Confirmed complete |
| E21 | pelopidas | II/372 | 364 | v1/368, f355 | `tiis passus judicari potest.` | Confirmed complete |
| E22 | marcellus | II/417 | 409 | v1/390, f377 | `Cæsar ipse theatrum Marcello inscriptum.` | Confirmed complete |
| E23 | comparatio pelopidae et marcelli | II/423 | 415 | v1/393, f380 | `virtus amatur.` | Confirmed complete |
| E24 | aristides | III/51 | 43 | v1/414, f401 | `merito in admiratione est atque suspicitur.` | Confirmed complete |
| E25 | cato major | III/95 | 87 | v1/434, f421 | `ætatis homines fuit clarissimus.` | Confirmed complete |
| E26 | comparatio aristidis et catonis | III/104 | 96 | v1/438, f425 | `satis idoneus affinis futurus.` | Confirmed complete |
| E27 | philopoemen | III/135 | 127 | v1/453, f440 | `bonis semper deberi.` | Confirmed complete |
| E28 | flamininus | III/167 | 159 | v1/468, f455 | `restat ut comparationem ipsam inspiciamus.` | Confirmed complete |
| E29 | comparatio philopoemenis et flaminini | III/171 | 163 | v1/469, f456 | `probene arbitrium tulerimus.` | Confirmed complete |
| E30 | pyrrhus | III/229 | 221 | v1/497, f484 | `præbuit.` | Confirmed complete |
| E31 | caius marius | III/297 | 289 | v1/530, f517 | `mortem sibi ipse conscivit.` | Confirmed complete |
| E32 | lysander | III/342 | 334 | v1/551, f538 | `commemoranda habuimus.` | Confirmed complete |
| E33 | sylla | III/403 | 395 | v1/581, f568 | `mico maleficiis superatum fuisse.` | Confirmed complete |
| E34 | comparatio lysandri et syllae | III/411 | 403 | v1/584, f571 | `fortitudinis primam laudem deferamus.` | Confirmed complete |
| E35 | cimon | IV/39 | 31 | v1/600, f587 | `Talis quidem Græcorum erat dux.` | Confirmed complete |
| E36 | lucullus | IV/112 | 104 | v1/634, f621 | `que mortis ab obitu ejus non multum distans sortitus est.` | Confirmed complete |
| E37 | comparatio cimonis et luculli | IV/118 | 110 | v1/637, f624 | `fuisse.` | Confirmed complete |
| E38 | nicias | V/66 | 48 | v2/41, f648 | `quæ eventura sæpe prædixerat.` | Confirmed complete |
| E39 | crassus | V/120 | 102 | v2/67, f674 | `pendio utens eum suffocavit.` | Confirmed complete |
| E40 | comparatio niciae et crassi | V/127 | 109 | v2/70, f677 | `adjecit.` | Confirmed complete |
| E41 | sertorius | V/164 | 146 | v2/88, f695 | `exosus consenuit.` | Confirmed complete |
| E42 | eumenes | V/194 | 176 | v2/102, f709 | `cum mare inspicere posset.` | Confirmed complete |
| E43 | comparatio sertorii et eumenis | V/197 | 179 | v2/105, f710 | `potemque fecit.` | Confirmed complete |
| E44 | agesilaus | V/262 | 244 | v2/137, f738 | `Leonidas interfecit.` | Confirmed complete |
| E45 | pompeius | V/368 | 350 | v2/188, f789 | `Pompeii sibi allatas Cornelia in Albano humavit.` | Confirmed complete |
| E46 | comparatio agesilai et pompeii | V/374 | 356 | v2/191, f792 | `defectionem ad hostes eorum fecit.` | Confirmed complete |
| E47 | alexander | VI/117 | 109 | v2/244, f843 | `ptus, sanæ mentis jacturam fecisse.` | Confirmed complete |
| E48 | iulius caesar | VI/202 | 194 | v2/285, f884 | `ictui vehementiam addente mortuus est.` | Confirmed complete |
| E49 | phocion | VI/249 | 241 | v2/308, f905 | `et error fuit et Athenarum detrimentum.` | Confirmed complete |
| E50 | cato minor | VI/341 | 333 | v2/363, f948 | `apud Philippos periit.` | Confirmed complete |
| E51 | agis | VII/36 | 28 | v2/378, f959 | `ceteris credidisset.` | Confirmed complete |
| E52 | cleomenes | VII/83 | 75 | v2/402, f983 | `verunt.` | Confirmed complete |
| E53 | tiberius gracchus | VII/110 | 102 | v2/414, f995 | `Vita Scipionis singulatim scriptum.` | Confirmed complete |
| E54 | caius gracchus | VII/132 | 124 | v2/425, f1006 | `nostros moderate ferendi facultatem ei non adimit.` | Confirmed complete |
| E55 | comparatio agidis cleomenis et gracchorum | VII/138 | 130 | v2/428, f1009 | `haud paullo inferiorem fuisse judicaverim.` | Confirmed complete |
| E56 | demosthenes | VII/174 | 166 | v2/446, f1027 | `accepimus conscriptam.` | Confirmed complete |
| E57 | cicero | VII/240 | 232 | v2/478, f1057 | `intulit.` | Confirmed complete |
| E58 | comparatio demosthenis et ciceronis | VII/246 | 238 | v2/481, f1060 | `Antipatri quasi subsannans crudelitatem.` | Confirmed complete |
| E59 | demetrius | VII/311 | 303 | v2/513, f1092 | `tempus est ut Romana in scenam mittatur.` | Confirmed complete |
| E60 | antonius | VII/414 | 406 | v2/563, f1140 | `Quintus ab Antonio ratione stirpis fuit.` | Confirmed complete |
| E61 | comparatio demetrii et antonii | VII/419 | 411 | v2/565, f1142 | `sese vitæ subduxit.` | Confirmed complete |
| E62 | dio | VIII/70 | 62 | v2/596, f1173 | `sumus in Vita Timoleontis prosecuti.` | Confirmed complete |
| E63 | brutus | VIII/132 | 124 | v2/626, f1203 | `ris et mortis modum significat.` | Confirmed complete |
| E64 | comparatio dionis et bruti | VIII/137 | 129 | v2/628, f1205 | `loco relinqui.` | Confirmed complete |
| E65 | artoxerxes | VIII/176 | 168 | v2/647, f1224 | `Ochus, crudelitate et paricidiis omnibus superior.` | Confirmed complete |
| E66 | aratus | VIII/242 | 234 | v2/679, f1254 | `propagatum est.` | Confirmed complete |
| E67 | galba | VIII/275 | 267 | v2/695, f1270 | `reliquit.` | Confirmed complete |
| E68 | otho | VIII/303 | 295 | v2/708, f1281 | `se conjunxerunt.` | Confirmed complete |

## Verification of the prior repairs at current output locations

The read-only ledger `src/facsimile/fixtures/inner_tomi_repair_source_trace.json` contains **25 mask records covering 26 rectangles**. The grouped table below accounts for all 25 records. Every neighborhood was visually checked in the current output; configuration presence alone was not counted as verification. O3 has one record with two rectangles, explaining the 26-rectangle total. Successful furniture removal does not establish that the entire surrounding Life is free of defects.

| Prior IDs | Mask records | Source physical / printed folios | Current physical / reader neighborhood and Latin anchor | Visual assessment |
| --- | ---: | --- | --- | --- |
| W1 | 3 | v1/287-288, f274-275; two records on 287, one on 288 | II/199, r191; `sibi ad-` / `junctis` | Watermark and head absent; local prose survives. |
| O1 | 1 | v1/182, f169 | I/371, r355; `non enim neque is jam exsul,` / `neque nos cives sumus` | Head/169 absent. |
| O1C | 1 | v1/290, f277 | **II/203, r195**; `O ma-` / `ter, ut mecum egisti!` | Head/277 absent; residual gap and right ruleM3/R4 survive. Actual seam is203, not204. |
| W2/G4 | 2 | v1/299, f286 | II/227, r219; `inter` / `se per otium dividere insulam liceret.` | Watermark absent; compact local seam. |
| O1B | 2 | v1/281, f268 | II/187, r179; chapter XXII | Greek head/268 absent; prose neighborhood inspected. |
| O1D/G8B | 2 | v1/298, f285 | II/223, r215; `cum` / `deprecatum ivisset` | Head/285 absent; gap and right dividerM5/R5 survive. |
| G5 | 2 | v1/308, f295 | **II/243, r235**; `neque periit,` / `neque vulnus accepit` | Head/295 absent; gap/right dividerM6/R5 survive. |
| O2 | 1 | v1/550, f537 | III/337, r329; `Haliartii perhibent,` / `habitasse apud ipsos` | Head/537 absent. |
| O3 | 1 (two rectangles) | v2/441, f1022 | **VII/163, r155**; `mœrore suo` / `conficiat, sed et levandus` | Head and right rule absent; compact seam. |
| O4 | 2 | v2/539, f1116 | **VII/365, r357**; `Eo impe-` / `trato, hostes irruentes` | Greek head and rule absent; split-word continuation survives. |
| O6 | 1 | v2/337, f934 | **VI/308, r300**; `nonnulli etiam amici Pompeii` | Greek fragment/right rule absent. |
| O5 | 1 | v1/530, f517 | III/297, r289; `mortem sibi ipse conscivit.`; white 298 | Furniture-only extra body page absent; separator298 fully checked at 180 DPI. |
| G1/O7 | 1 | v1/296, f283 | II/219, r211; `seque ei` / `crediderunt` | Obstructing rule/head absent; lesser copied gap M4 remains. |
| G2/O7 | 1 | v1/304, f291 | **II/235, r227**; `vestem excutiebat` | Actual mask neighborhood checked at 180 DPI; prose survives. Neighbor 236-237 also contain prose. |
| O8B | 1 | v1/306, f293 | II/239, r231; `obside-` / `bantur` | Old line slicing absent at the repaired neighborhood. |
| G6 | 1 | v1/470, f457 | III/175, r167; `strepitu aquæ` / `verba non percipientibus` | Adjacent top lines; old large gap absent. Greek-column scraps remain elsewhere on 173 (F2). |
| O7 | 1 | v2/361, f946 | VI/337, r329; `quoque, inquit, decrevistis virum` | Heavy right rule absent; duplicate D7 survives. |
| G3/O7 | 1 | v2/625, f1202 | VIII/130-132, r122-124; `subli-`, later `partem secutus.` | Local concluding material preserved; no divider-only page. This does not cover the newly found source 624 omission C5. |
| **Total** | **25** | **26 rectangles** | All current neighborhoods visually inspected | All recorded furniture masks visibly remove their targeted furniture at reviewed locations; residual/new findings are separately recorded. |

| Prior duplicate exclusion | Source and printed-folio evidence | Current output seam | Assessment |
| --- | --- | --- | --- |
| D1 | v2/103-104 excluded; f708-709 duplicate; surviving 102 ->105, f709 ->710 | V/195, r177; `magno in` / `pretio habitus` | Confirmed old return to Eumenes absent; expected comparison continuation survives. |
| D2 | v2/369-370 excluded; f952-953 duplicate; surviving 368 ->371, f953 ->954 | VII/20, r12; `unius suffragii numero` / `excellerent qui antiquabant legem` | Confirmed old reset absent; expected Agis continuation survives. Later D8 is a separate repeated block. |
| D3 | v2/339-348 excluded; f926-935 duplicate; surviving 338 ->349, f935 ->936 | VI/312, r304; `ejusdem esse` / `dicens magna mala inferre et depellere` | Confirmed old ten-page reset absent; expected Cato continuation survives. Later D7 is a separate repeated block. |

## Evidence and practical limits

The original full sweeps and native inspections are recorded in both checkpoints. Final continuation evidence adds `final-source-0..3.png` (raw-source columns for remaining checks), `final-output-0..2.png` (exact mask/omission neighborhoods), `endpoint-proof-0..8.png` (terminal output and corresponding source strips), and `endpoint-source-full-0..3.png` (full columns when a terminal sentence lay above the initial diagnostic strip). All these images were inspected. `endpoint-ledger.md` and `endpoint-ledger.json` under `tmp/pdfs/reinspection/` preserve the matched endpoint locators; the 68-row table above is the durable coverage record. Inspection render and comparison scripts read sources and write diagnostics under `tmp/pdfs/reinspection/`; report-authoring scripts additionally write this new report.

Source-header progression was reviewed end to end across v1 physical 14-637 and v2 physical 18-708. Output high-resolution/OCR coverage is complete, but visual adjudication used contact and edge sheets plus targeted native pages rather than reading every full page character by character. The report therefore distinguishes confirmed local omissions from the unmeasured possible extent of clipping, and confirmed endpoints from unproven interior completeness. No additional duplicate block, divider-only page, terminal truncation, QR fragment, or horizontal slice was established beyond the rows above. Faint furniture and individual clipped glyphs can escape this method.

The remaining uncertainty is the **total extent of lateral glyph clipping** outside the explicitly compared neighborhoods and the exhaustive number of surviving divider fragments. No open source-association check from the second checkpoint remains: source 603 watermark, source 375 Greek head, source 480 slice folio, source 213 clipping, source 52 spacing, and the pending source-margin seams were all compared. The source 624 comparison produced C5 and corrected the earlier tentative intact-seam assessment.

## Suggested repair priorities for a later pass

1. **Distinct Latin loss and clipping:** C1, C2, C3, and C5 omit eight distinct source lines; C4/C6 clip Latin glyphs. S1/S2 split text lines across page boundaries. These directly obstruct faithful reading and deserve the first attention in any separately authorized repair pass.
2. **Repeated reading order:** D4-D12 repeat 20 source pages across nine blocks. The diagnostic source joins preserve distinct source sequence, while shared output boundary pages contain neighboring prose. D5/D6/D12 have incomplete second openings and must not be described as two perfectly complete copies.
3. **Furniture inside prose:** H1/H2 and W4-W6 introduce heads, folios, or Google logo material into body text; F1/F2 retain neighboring Greek scraps.
4. **Margin gaps and rules:** M1-M11 and representative R findings interrupt layout. Their local continuity assessments differ, especially C5 at M11. Genuine printed chapter/verse spacing in the non-findings table must remain distinguished from copied source margin.

This report records findings and priorities only. It supplies no mask coordinates, code changes, deletion instructions, or Latin corrections. The intentional physical 8 Google notices, V Praefatio scale transition, white recto separators, signature padding, and genuine sparse complete endings remain non-findings. Historical notes and plans are retained.

Final verification checked all eight category sections, D4-D12, all 68 endpoint IDs against `endings.csv`, source/reader locator arithmetic, and the 25-record mask total. The new report received an explicit whitespace check because `git diff --check` does not include untracked files. `git diff --check` passed for tracked changes; repository status shows the pre-existing user modification to `AGENTS.md` and the retained untracked prompt/checkpoints, plus this report and diagnostics. No Rust test/build was run: this inspection made no code or generated-output change.
