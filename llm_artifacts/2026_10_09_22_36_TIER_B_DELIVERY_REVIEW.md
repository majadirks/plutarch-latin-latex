# Context

All 20 promoted events / 43 measured missing lines are installed. The source
checks pass, and the eight fresh readers embed the verified canvases. This
worksheet provides actual rebuilt page locators for human acceptance. The user
accepted small Greek edge fragments; that acceptance does not imply that every
rebuilt line and seam has been read. Delivery remains `in_progress`.

# Machine verification

The permanent Antonius crop is [876,1721), preserving complete b/c annotations,
the following chapter, and the skewed ending. It uses the original source pixels
with a fingerprint-scoped column fit. The 22 missing lines and 12 remaining p562
body regions are required to stay whole. The three merged heads are excluded at
Brutus [129,158), Cato Minor [114,142), and Demosthenes [127,156). Phocion keeps
its complete first line with a measured width fit. Dio p580 preserves its left
glyph edge and needs no extra bottom padding. Printed verse indentation is
retained in Alcibiades and Pericles. There are 69 permanent protection records.

The authenticated full-work comparison passes all 68 operational units: 48
unrelated units have identical source identities, crops, bands, columns, metrics,
and ordered canvas hashes; 20 affected units preserve every previously
extracted body interval in unique source order. Affected scope includes units
reading repaired shared pages. Production scale is compared by exact f32 bits;
JSON decoding of the historical decimal has no effect on rendering.

Final formatting and strict all-target Clippy checks pass. The ordinary Rust
suite passes 419 library tests and 9 CLI tests; 76 source/evidence tests remain
explicitly ignored in the ordinary run. The authenticated full-work comparison
passes separately in release mode. The portable checker passes in an offline
snapshot without PDFs, raster caches, or archives, including its expected
missing-archive negative probe. Its five Python unit tests also pass.

The comparison and portable-checker changes affect test infrastructure. The
production files and eight reader PDF hashes still match the fresh rebuild
configuration. Model inspection of rendered repaired passages found no new
layout defect; human acceptance remains pending below.

Every locator below is found by matching the passing source-report canvas SHA-256
against the PDF embedded grayscale samples. Physical pages and printed folios
are read from the fresh PDF. Original worksheet locators are historical hints.

# Reader review

Read each repaired passage and its adjacent page breaks against the original
source. Confirm complete letters and punctuation, correct order, printed spacing,
and intact neighboring prose. Check the later unit starts, recto placement,
running folios, contents references, and signature padding in changed tomi.
The final event statuses and full-delivery selector wait for this review.

Review one tomus at a time, in rebuilt physical-page order. Event numbers retain
their original identifiers. Tomus 1 has no Tier-B events to review.

[Tomus 2](#tomus-2) · [Tomus 3](#tomus-3) · [Tomus 4](#tomus-4) · [Tomus 5](#tomus-5) · [Tomus 6](#tomus-6) · [Tomus 7](#tomus-7) · [Tomus 8](#tomus-8)

## Tomus 2

Open `tomus_2_inner.pdf`. The table and entries below use this reader.

| Worksheet event | Unit / source pages | Lines | Rebuilt reader physical pages / folios | Neighbor pages to inspect | Human acceptance |
| --- | --- | ---: | --- | --- | --- |
| 3 | pericles — v1 p214, v1 p215 | 2 | T2 p49 / f41, T2 p50 / f42 | T2 p49, T2 p50 | Pending |
| 16 | alcibiades — v1 p242 | 1 | T2 p108 / f100 | T2 p108 | Pending |
| 15 | timoleon — v1 p318 | 1 | T2 p265 / f257 | T2 p265 | Pending |
| 4 | aemilius_paulus — v1 p326 | 1 | T2 p282 / f274 | T2 p282 | Pending |
| 12 | marcellus — v1 p377 | 1 | T2 p389 / f381 | T2 p389 | Pending |

### Event 3: miss_pericles_214_9084

Historical locator: tomus_2_inner.pdf physical page 49 (printed folio 41). Verified: after `… Aristophanis versuum testimonio utentes :` the reader jumps straight to verse line 3 `inde Megarenses ira turgentes gravi,`, dropping verse lines 1–2. `Megaram/Simætham/ebrii` appear nowhere in the Pericles reader.

Rebuilt locators: tomus 2, physical 49, folio 41, tomus 2, physical 50, folio 42.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

### Event 16: miss_alcibiades_242_229

Historical locator: tomus_2_inner.pdf physical page 108 (folio 100), Alcibiades ch. 1 lisp passage. Verified: reader jumps `his verbis quibus Theorum cavillatur :` → verse line 2 `« Cernin' Theolum? caput habet colacis. » Bene Alcibiades profecto illud balbutiit.`, dropping verse line 1. Foot line of v1 p242.

Rebuilt locators: tomus 2, physical 108, folio 100.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

### Event 15: miss_timoleon_318_9879

Historical locator: tomus_2_inner.pdf physical page 265 (folio 257). Verified: reader reads broken `… laudationibus acclamationibusque` / `bat sententiam. (5) Ea approbata ministri bigas a theatro`; split word `fere-`/`bat` (= *ferebat*) orphaned. `deliberaretur / audita deinde` absent. Head line of v1 p318.

Rebuilt locators: tomus 2, physical 265, folio 257.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

### Event 4: miss_aemilius_paulus_326_3162

Historical locator: tomus_2_inner.pdf physical page 282 (folio 274). Verified: reader jumps `… rei gerendæ opportunitatem` → `rentur. (5) Nocturnas excubias sine hastis …`; split word `ute-`/`rentur` (= *uterentur*) orphaned. `strenue pugnarent / Romano more` appear nowhere in the reader. Head line of v1 p326.

Rebuilt locators: tomus 2, physical 282, folio 274.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

### Event 12: miss_marcellus_377_3280

Historical locator: tomus_2_inner.pdf physical page 389 (folio 381). Verified: reader reads broken `(5) Hi ergo ad Marcelli pedes tunc se confertim provolven-` / `ascriberentur, multis adhibitis vociferationi lacrimis,`; split word `provolven-`/`tes` (= *provolventes*) orphaned. `petebant supplices / honestam aliquam` absent. Head line of v1 p377.

Rebuilt locators: tomus 2, physical 389, folio 381.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

## Tomus 3

Open `tomus_3_inner.pdf`. The table and entries below use this reader.

| Worksheet event | Unit / source pages | Lines | Rebuilt reader physical pages / folios | Neighbor pages to inspect | Human acceptance |
| --- | --- | ---: | --- | --- | --- |
| 7 | caius_marius — v1 p519 | 1 | T3 p272 / f264 | T3 p272 | Pending |
| 2 | lysander — v1 p535 | 2 | T3 p305 / f297 | T3 p305 | Pending |

### Event 7: miss_caius_marius_519_9217

Historical locator: tomus_3_inner.pdf physical page 272 (folio 264). Verified: reader jumps `… Acriter quidem Sylla repugnavit, sed` → `quod subito exarserat, impeditum est. (6) Nam bellicosis-`; full line dropped (sentence reads broken without it). `contentio plane erumperet / sociali bello` absent. Head line of v1 p519.

Rebuilt locators: tomus 3, physical 272, folio 264.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

### Event 2: miss_lysander_535_1762

Historical locator: tomus_3_inner.pdf physical page 305 (printed folio 297). Verified: the reader jumps from `(5) Qui Herculis stirpe ortos sine dolo bellum` straight to `VIII. Ejus generis est, quod traditum est cum Milesiis …`, dropping the two lines. The anecdote's distinctive words (`vulpinam/leonina/assuendam`) appear nowhere in the Lysander reader.

Rebuilt locators: tomus 3, physical 305, folio 297.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

## Tomus 4

Open `tomus_4_inner.pdf`. The table and entries below use this reader.

| Worksheet event | Unit / source pages | Lines | Rebuilt reader physical pages / folios | Neighbor pages to inspect | Human acceptance |
| --- | --- | ---: | --- | --- | --- |
| 11 | lucullus — v1 p631 | 1 | T4 p104 / f96 | T4 p104 | Pending |

### Event 11: miss_lucullus_631_13150

Historical locator: tomus_4_inner.pdf physical page 104 (folio 96). Verified: reader reads broken `… quam ad bellum Pirati-` / `publicum intulerat, et seorsum dati …` (nonsense `Piratipublicum`); split words `Pirati-`/`cum` (= *Piraticum*) and `ærarium`/`publicum`. `Pompeio dederat` absent. Head line of v1 p631.

Rebuilt locators: tomus 4, physical 104, folio 96.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

## Tomus 5

Open `tomus_5_inner.pdf`. The table and entries below use this reader.

| Worksheet event | Unit / source pages | Lines | Rebuilt reader physical pages / folios | Neighbor pages to inspect | Human acceptance |
| --- | --- | ---: | --- | --- | --- |
| 13 | nicias — v2 p25 | 1 | T5 p33 / f15 | T5 p32, T5 p33 | Pending |
| 9 | crassus — v2 p44 | 1 | T5 p72 / f54 | T5 p72 | Pending |
| 5 | agesilaus — v2 p110 | 1 | T5 p209 / f191 | T5 p209 | Pending |

### Event 13: miss_nicias_25_2918

Historical locator: tomus_5_inner.pdf, seam at page boundary 32→33 (folios 14→15). Verified: p32 ends `… exsecrantes eos qui in` and p33 opens orphaned `heretur, communicatis toto de negotio sermonibus, pacem`; the dropped line (= `… eos qui in` + missing + `traheretur`) fell at the page break. `novem triennia / fatis esse dixerant` absent. Head line of v2 p25.

Rebuilt locators: tomus 5, physical 33, folio 15.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

### Event 9: miss_crassus_44_1144

Historical locator: tomus_5_inner.pdf physical page 72 (folio 54). Verified: reader jumps `… ad fontem elidit.` → `bat villicus, neminem ipse cernens vel cognoscens, sed ipsis`; split word `affere-`/`bat` (= *afferebat*) orphaned. `delitescenti Crasso / quotidie alimenta` absent. Head line of v2 p44 — carries the chapter-V heading `V.`, so the reader also loses that chapter number.

Rebuilt locators: tomus 5, physical 72, folio 54.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

### Event 5: miss_agesilaus_110_2063

Historical locator: tomus_5_inner.pdf physical page 209 (folio 191). Verified: reader jumps `… satis sibi virium esse,` → `gnum enim aliquid se effecturum sperabat, indignumque`; split word `Ma-`/`gnum` (= *Magnum*) orphaned. `Agesilao pergratum / bellum cepit` appear nowhere. Foot line of v2 p110.

Rebuilt locators: tomus 5, physical 209, folio 191.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

## Tomus 6

Open `tomus_6_inner.pdf`. The table and entries below use this reader.

| Worksheet event | Unit / source pages | Lines | Rebuilt reader physical pages / folios | Neighbor pages to inspect | Human acceptance |
| --- | --- | ---: | --- | --- | --- |
| 14 | phocion — v2 p291 | 1 | T6 p209 / f201 | T6 p209 | Pending |
| 8 | cato_minor — v2 p329 | 1 | T6 p284 / f276 | T6 p283, T6 p284 | Pending |
| 18 | cato_minor — v2 p352 | 1 | T6 p312 / f304 | T6 p312 | Pending |

### Event 14: miss_phocion_291_2223

Historical locator: tomus_6_inner.pdf physical page 209 (folio 201). Verified: reader jumps `… (7) Ex oratoribus` → `Athenienses, ubi insanierint; Te vero, inquit, ubi sapue-`; the anecdote's setup line is dropped (only the punchline survives). Distinctive `Occident te` absent everywhere. Full line, no split word. Head line of v2 p291.

Rebuilt locators: tomus 6, physical 209, folio 201.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

### Event 8: miss_cato_minor_329_8786

Historical locator: tomus_6_inner.pdf, seam at page boundary 283→284 (folios 275→276). Verified: p283 ends `… Veneris Paphiæ concessu-`, p284 begins `parans, responsa exspectabat. (2) …`; the reader reads broken `concessu-parans`. Split words `concessu-`/`rum` and `com-`/`parans`. `Interim Rhodi / negotium tractandum` absent. Head line of v2 p329.

Rebuilt locators: tomus 6, physical 284, folio 276.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

### Event 18: miss_cato_minor_352_15055

Historical locator: tomus_6_inner.pdf physical page 312 (folio 304). Verified: reader jumps `… imperitia et temeritate` → `eveniret, ut Cæsar opprimeretur, se quidem Romæ non man-`; full line dropped. `ducum considerata / bona quadam fortuna` absent. Foot line of v2 p352.

Rebuilt locators: tomus 6, physical 312, folio 304.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

## Tomus 7

Open `tomus_7_inner.pdf`. The table and entries below use this reader.

| Worksheet event | Unit / source pages | Lines | Rebuilt reader physical pages / folios | Neighbor pages to inspect | Human acceptance |
| --- | --- | ---: | --- | --- | --- |
| 10 | demosthenes — v2 p445 | 1 | T7 p41 / f33 | T7 p41 | Pending |
| 1 | antonius — v2 p562 | 22 | T7 p277 / f269 | T7 p276, T7 p278 | Pending |
| 19 | dio — v2 p566 | 1 | T7 p288 / f280 | T7 p288, T7 p289 | Pending |
| 20 | dio — v2 p580 | 1 | T7 p317 / f309 | T7 p317 | Pending |
| 6 | brutus — v2 p619 | 1 | T7 p395 / f387 | T7 p395, T7 p396 | Pending |

### Event 10: miss_demosthenes_445_6929

Historical locator: tomus_7_inner.pdf physical page 41 (folio 33), Demosthenes' death scene. Verified: reader jumps `… neque ante personatus, neque` → `nitari incipiente, Nunc, ait, ex Macedonico tripode loque-`; split word `mi-`/`nitari` (= *minitari*) orphaned. `promissis me movebis` absent. Head line of v2 p445.

Rebuilt locators: tomus 7, physical 41, folio 33.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

### Event 1: miss_antonius_562_20869

Historical locator: tomus_7_inner.pdf physical page 274 (printed folio 266), the last line of chapter LXXXVI. (The historical "VII/413" does not map to the current pagination.)

Rebuilt locators: tomus 7, physical 277, folio 269.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

### Event 19: miss_dio_566_426

Historical locator: tomus_7_inner.pdf physical page 284 (folio 276), Dio ch. III. Verified: reader reads broken `… Hermocratis Syracusani duxit filiam. Hanc nondum` / `runt, corpus ejus scelerata et fœda libidine ita violarunt,`; split word `arripue-`/`runt` (= *arripuerunt*) orphaned. Foot line of v2 p566.

Rebuilt locators: tomus 7, physical 288, folio 280.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

### Event 20: miss_dio_580_6654

Historical locator: tomus_7_inner.pdf physical page 313 (folio 305), Dio ch. XXIX end. Verified: reader reads broken `… Septima die in ar-` / `apud Synalum deposuerat, plaustris allata, quæ inter cives`; orphaned `ar-`. `navibus advectus est Dionysius` absent. Foot line of v2 p580.

Rebuilt locators: tomus 7, physical 317, folio 309.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

### Event 6: miss_brutus_619_10158

Rebuilt reader locator: tomus 7, physical page 395, printed folio 387. Inspect physical pages 395–396 for the adjoining seam.

Pre-repair baseline locator: physical page 391, printed folio 383. At that location in the baseline reader, the text jumps `… præterito Cæsaris lævo cornu,` → `cissent, circumducto suo cornu castra Cæsaris adorti sunt.`; final split word `…d-`/`cissent` is orphaned. The omitted head line comes from v2 p619. These baseline page numbers do not locate the restored line in the rebuilt reader.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: The user confirmed on 2026-10-10 that the missing text beginning `quum leviter…` is restored on physical page 395 / printed folio 387. Full text and punctuation, the adjoining seam, and layout acceptance remain pending.

## Tomus 8

Open `tomus_8_inner.pdf`. The table and entries below use this reader.

| Worksheet event | Unit / source pages | Lines | Rebuilt reader physical pages / folios | Neighbor pages to inspect | Human acceptance |
| --- | --- | ---: | --- | --- | --- |
| 17 | artoxerxes — v2 p633 | 1 | T8 p19 / f11 | T8 p19 | Pending |

### Event 17: miss_artoxerxes_633_2282

Historical locator: tomus_8_inner.pdf physical page 19 (folio 11). Verified: reader jumps `… (2) Cadit ibi Cyrus, ictus a rege ipso,` → `hujus facinoris loco rex dederit, ut in bello semper aureum`; full line dropped (who killed Cyrus — a Carian). `Care quodam / quidam perhibent` absent. Foot line of v2 p633.

Rebuilt locators: tomus 8, physical 19, folio 11.

- [ ] Complete printed source text and punctuation confirmed.
- [ ] Source order, neighboring text, and every touched page break confirmed.
- [ ] Layout and source artifacts accepted.

Human decision: Pending.

# Evidence

The exact PDF identities, embedded image hashes, and page renders are recorded
in `archive/tier_b_omission_repairs_2026_10_09/delivery-review/pdf-seam-locators.json`.
The full-work comparison and source/code configuration are retained in that
archive alongside the current unit snapshots and authenticated baseline references.
All 101 archived review files authenticate against the manifest whose SHA-256 is
`8611aee21c1cf2ede91744071d012d777a4a3006c7d8fe89978c1a2183bef65c`.
The eight reader PDFs and compilation provenance remain in the immutable
`final-reader-build/` archive. The installation and rebuild verification notes
record their manifest hashes. Historical plans, failed trials, promotion evidence,
and the pre-installation baseline are retained.
