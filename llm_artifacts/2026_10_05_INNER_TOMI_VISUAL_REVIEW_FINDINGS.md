# Inner tomi visual review findings

## Scope and page references

Reviewed on 2026-10-05: the eight current `build_facsimile/tomus_N_inner.pdf` files, whose build logs record compilation at approximately 15:47–15:48 on that date. All eight documents were reviewed in labeled page contact sheets; suspected flaws and all 68 body-section endings were inspected at higher resolution. Three additional reviewing agents covered I–II, III–IV, and V–VI; the primary reviewer covered VII–VIII and consolidated the findings. The primary reviewer independently confirmed the user-reported Camillus header leak.

**Page numbers in this report are one-based physical pages of the inner PDFs.** Printed reader folios are included where useful. These are not page indices of the imposed printing PDFs, which arrange multiple book pages on each sheet. The user's locator “imposed page 355 (pdf page 371)” is preserved in the finding below; visually, 355 is the reader folio on inner PDF page 371.

| Tomus | Inner PDF pages | Last body page in this build |
| --- | ---: | ---: |
| I | 416 | 398 |
| II | 448 | 425 |
| III | 416 | 411 |
| IV | 128 | 118 |
| V | 384 | 378 |
| VI | 384 | 362 |
| VII | 448 | 423 |
| VIII | 320 | 303 |

The review identifies visible examples, not an exhaustive source-to-output audit of every Latin line. Small or faint artifacts can escape contact-sheet inspection. Exact Latin glyphs, spelling, punctuation, and capitalization remain source material; the short excerpts below identify locations and do not authorize transcription changes. No PDF, generator code, source PDF, or transcription cache was changed.

## Results against the requested issues

| Requested issue | Result |
| --- | --- |
| 1. Large awkward gaps in the middle of prose | **Yes.** Especially severe in II/219–220, II/235–238, and VIII/130–132. Smaller conspicuous interior gaps appear in I/36, I/46, and III/175. |
| 2. Blank pages in the middle of a prose section | **Yes, effectively blank body pages:** II/236–237 and VIII/131 have no prose, although source divider marks and generated headings/folios remain. These differ from entirely white recto separators between sections. |
| 3. Google watermarks | **Yes.** “Digitized by Google” survives inside body prose on II/199 and II/227. Separately, all eight tomi contain the prominent Google Books notice/logo/QR page at physical page 8. |
| 4. Incomplete sentences at the end of prose sections | **No confirmed example among the 68 inspected body-section endings.** Previously reported terminal truncations in IV and VIII have been repaired in this build. There are substantial interior continuity defects, particularly in V. |
| 5. Other obvious flaws | **Yes.** A prose line cut horizontally across a page boundary in II, duplicated/out-of-order prose in V and VII, leaked source headers and page numbers, a furniture-only extra page in III, surviving divider rules/foreign fragments, and an awkward front-matter typography transition in V. |

## Large gaps and effectively blank pages within prose

| ID | Location | Visible finding | Assessment |
| --- | --- | --- | --- |
| G1 | II/219–220, Timoleon; reader folios 211–212 | Page 219 contains two prose lines near the top, followed by a large empty area with a source divider. Page 220 has substantial blank space before prose resumes. The ending `ci` and opening `num` visibly belong to a word split across this interruption. | Severe interruption of continuous prose. |
| G2 | II/235–238, Timoleon; reader folios 227–230 | Page 235 ends with prose; **236 and 237 contain no prose**, just divider marks and generated furniture. Page 238 remains empty through most of its body area before two lines resume near the bottom with `dam sublime se cursu proripuit...`. | Two effectively blank pages inside a Life, plus most of a third wasted page. These are body image pages, not section-start padding. **Also independently reported by the user:** “Timoleon blank pages on pdf pages 236–238 (with some actual text at the foot of pdf page 238)”. |
| G3 | VIII/130–132, Brutus; reader folios 122–124 | Page 130 has four prose lines at the top ending in `subli-`, then a long divider through an empty area. **Page 131 contains no prose**, only the divider, generated BRUTUS heading, and folio. Page 132 has another large top gap and divider fragments before continuing with `partem secutus.` and the concluding paragraph. | A divider-only page interrupts the concluding prose. The visible separation also warrants source comparison to account for the word continuation; this review does not assert the intervening word material was preserved. |
| G4 | II/227, Timoleon; reader folio 219 | Two lines near the top are separated from the continuing prose below by a large empty interval containing a gray Google watermark. | Interior gap and watermark together. |
| G5 | II/245, Timoleon; reader folio 237 | Large top blank region, then a copied source heading fragment **`EON.`** and source folio **295** immediately above continuing prose; a divider fragment also remains in the blank region. | Source furniture is included in the prose flow. **Also reported by the user and independently rechecked at 180 DPI:** “Whitespace and header leak (EON ... 295) on pdf page 245”. |
| G6 | III/175, Pyrrhus; reader folio 167 | A gap of roughly three ordinary line pitches occurs between `...strepitu aquæ` and `verba non percipientibus...`. | Clear gap within a continuous sentence, not paragraph or section spacing. |
| G7 | I/36, Theseus; reader folio 20 | Roughly three line heights of whitespace follow `et mendacia esse.` before chapter XXI; a scan divider remains beside the gap. | Conspicuous chapter-transition gap; less severe than G1–G3. |
| G8 | I/46, Theseus; reader folio 30 | Roughly four line heights of whitespace follow the sentence ending `...Cynoscephalas exstant.` before chapter XXVIII. | Conspicuous chapter-transition gap with retained scan divider. |
| G8B | **II/222**, Timoleon; reader folio **214** | Prose ends around mid-page with `...cum`; a large blank interval follows, containing a thin source divider, then the source head fragment **`EON.`** and source folio **285** near the bottom. | **Reported by the user and independently visually confirmed.** The copied source furniture occupies the whitespace interruption inside the Life. |

The page appearance is consistent with source divider/furniture pixels being retained as part of composed body images. A code-level causal diagnosis was not performed in this review.

## Google watermarks and branding

| ID | Location | Visible finding |
| --- | --- | --- |
| W1 | II/199, Coriolanus; reader folio 191 | Large gray **Digitized by Google** appears below the main prose block, followed by a copied source running-head fragment `RIOLANUS.` and source folio **275**, then another prose line. **Also reported by the user and independently confirmed.** User locator: “Tomus II, pdf page 191”; 191 matches the printed reader folio, while the current physical inner PDF page is 199. Physical page 191 was checked and has no visible watermark. |
| W2 | II/227, Timoleon; reader folio 219 | Large gray **Digitized by Google** occupies the gap between prose blocks. **Also reported by the user:** “pdf page 227 / imposed page 219”; 219 is the printed reader folio. |
| W3 | I–VIII/8 | Full Google Books reproduction notice with prominent logo, URL, and QR code. This is a separately inserted front-matter notice page, not a watermark over prose. Its retention is a separate editorial decision. |

These are confirmed surviving examples in the current PDFs. Earlier cleanup notes describe other watermark occurrences that were removed; those successful cleanups do not establish that all source pages are clean.

## Duplicated and out-of-order prose

### D1 — Tomus V: Eumenes/comparison transition, pages 194–201

This is a substantial reading-order defect:

- **194** concludes Eumenes with the sentence ending `...in Macedoniam redire aut Græcum mare inspicere posset.`
- **195** opens the comparison under the generated heading **SERTORII CUM EUMENE COMPARATIO**, beginning `Hæc sunt quæ de Eumene et Sertorio memoratu digna accepimus.` Its last line breaks at `...partim de reliquis ducibus que-`.
- **196–198** return to Eumenes material instead of continuing that comparison sentence. Page 196 starts `rentes. Deinde impetu facto...`; page 198 includes Eumenes chapter XIX and repeats the ending already printed on 194. These pages nevertheless carry the comparison running head.
- **199** repeats the comparison opening, including the scanned source heading. The comparison proceeds to its complete ending on **201**.

Thus both Eumenes material and the comparison opening occur twice, and the transition from 195 to 196 does not continue the sentence correctly. Complete final sentences do not make this section's interior sequence correct.

### D2 — Tomus VII: Agis, pages 17–24

The passage on **17–20** is repeated on **21–24**. A clear anchor is the opening of pages **17 and 21**, both beginning `non modo luxum, in quo felicitatem vitæ...` and both carrying chapter VIII further down. The same chapter/text progression repeats across the next three pages. Reader folios are 9–12 for the first occurrence and 13–16 for the second.

Higher-resolution comparison confirms the same prose on 17 and 21, although the second occurrence includes much more conspicuous right-side divider marks. This is repeated textual content from differing image appearances, not a claim that the two page raster files are byte-identical. The source scan sequence should be checked before deciding which occurrence to retain.

## Other obvious scan and layout flaws

| ID | Location | Visible finding | Qualification |
| --- | --- | --- | --- |
| O1 | **I/371**, Camillus; reader folio **355** | Source running-head fragment **`LUS.`** and source folio **169** intrude between `...non enim neque is jam exsul,` and `neque nos cives sumus...`. | **Reported by the user and independently visually confirmed.** User locator: “imposed page 355 (pdf page 371)”. The generated CAMILLUS head and folio 355 are separate from the leaked source furniture. |
| O1B | **II/187**, Coriolanus; reader folio **179** | A Greek source running-head fragment, reported by the user as **“ARKIOS” in Greek characters**, appears inside a large gap immediately before chapter XXII. | **Reported by the user and independently visually confirmed.** User locator: “pdf page 187, imposed page 179”; 179 is the printed reader folio in the inner PDF. The exact complete Greek source heading is not transcribed here. |
| O1C | **II/204**, Coriolanus; reader folio **196** | Source running-head fragment **`RIOLANUS.`** and damaged source folio **277** appear after the top prose line and before its continuation. A vertical divider also survives at the right. | **Reported by the user and independently visually confirmed.** User locator: “RIOLANUS ... 277” on PDF page 204. |
| O1D | **II/222**, Timoleon; reader folio **214** | Source running-head fragment **`EON.`** and source folio **285** appear near the bottom of a large whitespace interruption. | **Reported by the user and independently visually confirmed.** See G8B for the surrounding layout. |
| O2 | III/337, Lysander; reader folio 329 | Source head fragment `NDER.` at left and source folio **537** at right appear below the body line ending `Haliartii perhibent,`. | Surviving source furniture, additional to the generated heading and reader folio. |
| O3 | VII/167, Demosthenes; reader folio 159 | A partial source running head, ending in Greek letters, appears inside a large gap between the lines ending `...mœrore suo` and beginning `conficiat, sed et levandus...`; a divider remains at the right. | Furniture interrupts a continuous sentence. The fragment's exact text is not established here. |
| O4 | VII/369, Antonius; reader folio 361 | A partial Greek source running head and divider intrude between the split `Eo impe-` and continuation `trato, hostes...`. | Another clear source-page seam contaminated by furniture. |
| O5 | III/298, after Caius Marius; reader folio 290 | No prose: only a black source ornament/rule, tiny fleck, generated C. MARIUS head, and folio. Marius's actual prose concludes on 297; Lysander opens on 299. | Unnecessary furniture-only page at a unit boundary, distinct from an interior blank within an unfinished sentence. |
| O6 | VI/329, Cato Minor; reader folio 321 | Isolated Greek **`ΩΝ.`** above the Latin opening `nonnulli etiam amici Pompeji...`, plus a conspicuous vertical black rule at right. | Likely retained source furniture or neighboring-column fragment; source classification is not proven by this visual review. |
| O7 | Many body pages; examples I/36, I/46, II/219–220, II/236–238, V/195–196, VI/329, VI/358, VII/21, VIII/130–132 | Thin, thick, or broken vertical source divider rules survive alongside the Latin; some extend through large otherwise blank regions. | Obvious scan artifacts. Removing them must preserve all adjacent Latin pixels. G2 and G3 show the most severe layout consequences. |
| O8 | V/14–17, Praefatio | Page 14 ends around mid-page after `...in duarum vitarum exemplis consistam, Ages.`; 15–16 continue in much denser, darker scanned typography, then ordinary typeset prose resumes on 17. | Conspicuous change of text scale/weight and substantial wasted space. It is not a wholly blank page or a confirmed missing-text defect. |
| O8B | **II/240–241**, Timoleon; reader folios **232–233** | The bottom line on 240 is sliced horizontally through its letters; the remaining lower portions appear as a clipped line at the very top of the prose image on 241. A raster page break has cut through a text line instead of falling between lines. | **Reported by the user as PDF 241–242 and independently confirmed at current physical PDF 240–241.** Pages 241–242 and reader folios 241–242 (physical 249–250) were checked; the clear horizontal split is at 240–241. Preserve both locators because the reported boundary differs by one physical page. |

## Prose endings and intentional blanks

No incomplete final sentence was confirmed at the inspected body endpoints:

| Tomus | Physical pages inspected as body-section endpoints |
| --- | --- |
| I | 56, 105, 113, 164, 203, 213, 259, 291, 298, 340, 398 |
| II | 61, 102, 106, 160, 208, 215, 267, 320, 323, 374, 419, 425 |
| III | 51, 95, 104, 135, 167, 171, 229, 297, 342, 403, 411 |
| IV | 39, 112, 118 |
| V | 66, 120, 127, 164, 194, 201, 266, 372, 378 |
| VI | 117, 202, 249, 362 |
| VII | 40, 87, 114, 136, 142, 178, 244, 250, 315, 418, 423 |
| VIII | 70, 132, 137, 176, 242, 275, 303 |

IV/118 currently ends `utrumque fuisse.` VIII/303 includes Otho's complete repaired ending, with `Cæ-` on the preceding page continued as `cinæ se conjunxerunt.` The older [volume endings audit](2026_10_05_FACSIMILE_VOLUME_ENDINGS_AUDIT.md) records real earlier truncations, but its IV/VIII findings do not describe these current PDFs. Other old page locators have also shifted after rebuilding.

Entirely white pages occur at unit boundaries because the generated documents start units on rectos. Examples include III/96, 136, 168, 172, 230, 404; IV/40; V/128, 202; VI/118, 250; VII/88, 316; and VIII/138, 276. Front/end flyleaves and final signature padding also remain. These are structurally different from the prose-free **body image pages** in G2 and G3.

Sparse pages that contain genuine concluding lines are not classified as missing-text defects: III/95 has two final lines, III/342 three, IV/118 three, and VIII/303 two. Their economical packing could be reviewed separately, but their endings are complete.

## Suggested repair priority

1. Resolve **D1 and D2** by checking the source scan sequence and accounting for each occurrence of genuine Latin. Reading order and duplication are correctness issues even where individual section endings are complete. Also repair **O8B** so every raster page boundary falls between complete text lines.
2. Resolve **G2 and G3**, then G1, with source-to-output line comparison. Divider-only pages must disappear without losing the adjoining words or lines.
3. Remove confirmed body watermarks **W1–W2** and leaked furniture **O1–O6**, including suffixed findings **O1B–O1D**, through narrowly bounded, reviewable image treatment that preserves Latin pixels.
4. Review the remaining divider artifacts and spacing examples; distinguish genuine printed paragraph/chapter spacing from copied source-margin space.
5. Make an explicit editorial decision about the Google notice page **W3** and the Praefatio typography transition **O8**.

This is a findings record, not an implementation plan. The priorities do not authorize silent alteration of source text.

## Local visual evidence

Temporary review renders remain under:

- `tmp/pdfs/inner_review/`: current all-volume 45 DPI pages and VII/VIII contact sheets; higher-resolution `user_t1_371.png`, `user_t2_187.png`, `user_t2_191.png`, `user_t2_204.png`, `user_t2_222.png`, `user_t2_near-199.png`–`user_t2_near-203.png`, `user_t2_cut_240.png`, `user_t2_split-241.png`–`user_t2_split-242.png`, `user_t2_cut_249.png`–`user_t2_cut_250.png`, `detail_t7_17.png`, `detail_t7_21.png`, `detail_t7_167.png`, `detail_t7_369.png`, `detail_t8_130.png`–`detail_t8_132.png`, and the VII/VIII `end_t*.png` endpoint renders.
- `tmp/pdfs/review12/`: enlarged I/36 and I/46, II/199 and II/227 watermarks, II/219–220, II/235–238, and II/245. A further 180 DPI confirmation of II/245 is `tmp/pdfs/inner_review/user_t2_245.png`.
- `tmp/pdfs/review34/`: enlarged III/175, III/298, III/337, and section endpoints.
- `tmp/pdfs/review56/`: enlarged V/194–201 comparison transition, Praefatio, VI/329 artifact, and section endpoints.

These ignored temporary paths support local inspection; the durable locators are the tomus/page references in this report. The PDFs and historical notes were left intact.
