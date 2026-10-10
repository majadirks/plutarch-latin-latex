# Tier-B promotion review worksheet

Written 2026-10-09. This is the working checklist for the human read-and-sign-off
that the [repair plan](2026_10_08_16_00_INNER_TOMI_AUDIT_REPAIRS_PLAN.md) requires
before any Tier-B omission can be repaired. The full inventory, pattern analysis,
and approximate readings are in
[`2026_10_09_10_30_TIER_B_OMISSION_LOSSES.md`](2026_10_09_10_30_TIER_B_OMISSION_LOSSES.md);
this file exists to be marked up event-by-event as we go.

## How to use this worksheet

We work top to bottom, one event per turn. For each event:

1. **Read the line.** Open the source crop named below (the `_0` crop is the
   expected source region; `_1`/`_2` show the reader neighborhood at the seam).
   Crops live under
   `archive/inner_tomi_audit_2026_10_08/runs/step102-final-all-5/crops/`.
   The reading printed here is only an aid — confirm against the crop (and, if you
   want certainty, the source PDF page itself). The scan is authoritative; the fix
   preserves the line from the raster and never retypes it.
2. **Fill the three gate checks** (`[ ]` → `[x]`):
   - **Latin confirmed** — the exact line, glyphs, punctuation, and any edge
     word-break are what you see in the scan.
   - **Still missing** — the line is still absent from the current rebuilt
     `tomus_N_inner.pdf` at the matching seam. (I render this for you when we reach
     the event; the baseline coordinates here are historical.)
   - **Boundary safe** — the neighbors that must stay intact/in order are clear, and
     restoring the line will not duplicate text or disturb genuine printed spacing.
3. **Record the decision** on the `Decision:` line — one of **PROMOTE** (move to
   Tier A, eligible for repair), **REJECT** (line is actually present / not a real
   loss — keep excluded), or **DEFER** (needs more evidence; say what).
4. Add anything useful to `Notes:`.

A PROMOTE decision does not change the historical detector confidence or the
original review verdict — it only makes the event repair-eligible. After each
PROMOTE I proceed to confirm-in-build → diagnose → fix → regression-test for that
event before we move on, unless you'd rather sign off several first and batch the
repairs.

Progress: **20 / 20 reviewed — COMPLETE** — 20 promoted, 0 rejected, 0 deferred.
All 20 Tier-B events (44 source lines) promoted to Tier A, eligible for repair.

Review mode from event #4 onward: **confirm-implies-promote** — a human confirmation
that a crop's Latin matches is logged as PROMOTE unless the human says REJECT/DEFER.
Still-missing and boundary are machine-verified for each event before presentation.

---

## Group 1 — Multi-line losses (review first)

### 1. antonius — 23 lines — v2 p562 (head→mid, y0 0.083)

- Event: `miss_antonius_562_20869` · crops: `miss_antonius_562_20869_0.png`, `_1.png`
- Current-build locator: **tomus_7_inner.pdf physical page 274** (printed folio 266),
  the last line of chapter LXXXVI. (The historical "VII/413" does not map to the
  current pagination.)
- Reading: chapter **LXXXVI.2–4** — `asser`[vatam] `dicunt et aureo fuso a Cleopatra
  irritatam et lacessitam brachium arripuisse. (2) Quid verum sit, nemo novit …`
  down to `… Archibius quidam ejus amicus bis mille talentis a Cæsare redemit.`
- Verified against the current build: page 274 jumps straight from `… Alii in seria
  inclusam aspidem asser-` to `LXXXVII. Antonius ex tribus uxoribus septem liberos
  reliquit …`, dropping the 23-line block between. Chapter **LXXXVII and the Life's
  ending are intact** (pages 275–276 continue `-sare interfectus; reliquos Octavia …`
  through the Julio-Claudian genealogy), so this is a single interior loss, not a
  truncated ending.
- Boundary: insert the 23 lines between the present `… aspidem asser-` (preceding
  anchor) and `LXXXVII. Antonius ex tribus uxoribus …` (following anchor); both
  anchors are present and unambiguous. Source extent: top ~40% of the Latin column on
  v2 p562 (box y 0.083–0.479) — a large top-of-page block, so the fix mechanism is
  likely a banding/head-trim issue on this interior page, to be diagnosed at repair
  time.

- [x] Latin confirmed  [x] Still missing (verified in current build, tomus_7 p274)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Largest Tier-B loss (23 of 44 lines). Human confirmed the crop's Latin is
  genuine source to restore and the in-block artifacts (left-margin section markers
  etc.) are harmless — restore the whole source block as-is, mask nothing. Fix is a
  banding/head-trim issue on interior page v2 p562, to diagnose at repair time.

### 2. lysander — 2 lines — v1 p535 (head, y0 0.076)

- Event: `miss_lysander_535_1762` · crops: `miss_lysander_535_1762_0.png`, `_1.png`
- Reading (end of ch. VII.5): `gerere volebant, eos rideri jussit : nam ubi pellis
  leonina non sufficeret, ibi vulpinam assuendam esse.`
- Current-build locator: **tomus_3_inner.pdf physical page 305** (printed folio 297).
  Verified: the reader jumps from `(5) Qui Herculis stirpe ortos sine dolo bellum`
  straight to `VIII. Ejus generis est, quod traditum est cum Milesiis …`, dropping
  the two lines. The anecdote's distinctive words (`vulpinam/leonina/assuendam`)
  appear nowhere in the Lysander reader.
- Boundary: insert between `… sine dolo bellum` and `VIII. Ejus generis est …`; clean
  anchors. Source extent: first two body lines of v1 p535 (head, y0 0.076), just
  before chapter VIII — the head-of-interior-page trim pattern.

- [x] Latin confirmed  [x] Still missing (verified, tomus_3 p305)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed the crop reads exactly `gerere volebant, eos rideri jussit :
  nam ubi pellis leonina non sufficeret, ibi vulpinam assuendam esse.` Head-of-page
  trim before chapter VIII; restore both lines as-is.

### 3. pericles — 2 lines, page-spanning — v1 p214→p215 (foot 0.863 → head 0.079)

- Event: `miss_pericles_214_9084` · crops: `miss_pericles_214_9084_0.png`, `_1.png`, `_2.png`
- Reading (Aristophanes *Acharnians* quotation, verse lines 1–2):
  `Megaram petentes Simætham meretriculam` (foot of p214) /
  `furto abstulerunt adolescentes ebrii ;` (head of p215).
- Current-build locator: **tomus_2_inner.pdf physical page 49** (printed folio 41).
  Verified: after `… Aristophanis versuum testimonio utentes :` the reader jumps
  straight to verse line 3 `inde Megarenses ira turgentes gravi,`, dropping verse
  lines 1–2. `Megaram/Simætham/ebrii` appear nowhere in the Pericles reader.
- Boundary: insert the two verse lines between `… testimonio utentes :` and
  `inde Megarenses ira turgentes gravi,`. These are **indented verse** lines in the
  source; restoring pixel-from-source preserves that layout. Page-spanning loss:
  last line of v1 p214 + first line of v1 p215 (the only straddling-seam case).

- [x] Latin confirmed  [x] Still missing (verified, tomus_2 p49)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed both crops — `Megaram petentes Simætham meretriculam` and
  `furto abstulerunt adolescentes ebrii ;`. Page-spanning verse couplet; restore both
  indented lines as-is between `… testimonio utentes :` and `inde Megarenses …`.

---

## Group 2 — Single-line losses at the page head (y0 ≈ 0.06–0.09)

Likely one shared cause (opening / interior-head trim or top-line column clipping).

### 4. aemilius_paulus — v1 p326 (y0 0.087)

- Event: `miss_aemilius_paulus_326_3162` · crop: `miss_aemilius_paulus_326_3162_0.png`
- Reading: `dedisset, strenue pugnarent, gladiisque Romano more ute-`
- Current-build locator: **tomus_2_inner.pdf physical page 282** (folio 274).
  Verified: reader jumps `… rei gerendæ opportunitatem` → `rentur. (5) Nocturnas
  excubias sine hastis …`; split word `ute-`/`rentur` (= *uterentur*) orphaned.
  `strenue pugnarent / Romano more` appear nowhere in the reader. Head line of v1 p326.
- Boundary: insert between `… opportunitatem` and `rentur. (5) Nocturnas …`.
- [x] Latin confirmed  [x] Still missing (verified, tomus_2 p282)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop reads `dedisset, strenue pugnarent, gladiisque Romano
  more ute-`. First confirm-implies-promote event.

### 5. agesilaus — v2 p110 (y0 0.886 — see note)

- Event: `miss_agesilaus_110_2063` · crop: `miss_agesilaus_110_2063_0.png`
- Reading: `bellum cepit, id quod Agesilao pergratum fuit. (2) Ma-`
- Current-build locator: **tomus_5_inner.pdf physical page 209** (folio 191).
  Verified: reader jumps `… satis sibi virium esse,` → `gnum enim aliquid se
  effecturum sperabat, indignumque`; split word `Ma-`/`gnum` (= *Magnum*) orphaned.
  `Agesilao pergratum / bellum cepit` appear nowhere. **Foot** line of v2 p110.
- Boundary: insert between `… satis sibi virium esse,` and `gnum enim aliquid …`.
- [x] Latin confirmed  [x] Still missing (verified, tomus_5 p209)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop reads `bellum cepit, id quod Agesilao pergratum fuit.
  (2) Ma`. Foot-of-page loss on v2 p110.

### 6. brutus — v2 p619 (y0 0.063)

- Event: `miss_brutus_619_10158` · crop: `miss_brutus_619_10158_0.png`
- Reading (human-corrected): `quum leviter extremas partes attigissent paucosque
  dije-` (split word `dije-`/`cissent` = *dijecissent*). Earlier auto-reading `d[u]-`
  was wrong.
- Current-build locator: **tomus_7_inner.pdf physical page 391** (folio 383).
  Verified: reader jumps `… præterito Cæsaris lævo cornu,` → `cissent, circumducto
  suo cornu castra Cæsaris adorti sunt.`; final split word `…d-`/`cissent`
  orphaned. Head line of v2 p619.
- Boundary: insert between `… lævo cornu,` and `cissent, circumducto …`.
- [x] Latin confirmed  [x] Still missing (verified, tomus_7 p391)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed and corrected the reading to `… paucosque dije-`
  (*dijecissent*). Head line of v2 p619, Philippi battle narrative.

### 7. caius_marius — v1 p519 (y0 0.078)

- Event: `miss_caius_marius_519_9217` · crop: `miss_caius_marius_519_9217_0.png`
- Reading: `quominus ea contentio plane erumperet, sociali bello`
- Current-build locator: **tomus_3_inner.pdf physical page 272** (folio 264).
  Verified: reader jumps `… Acriter quidem Sylla repugnavit, sed` → `quod subito
  exarserat, impeditum est. (6) Nam bellicosis-`; full line dropped (sentence reads
  broken without it). `contentio plane erumperet / sociali bello` absent. Head line
  of v1 p519.
- Boundary: insert between `… Sylla repugnavit, sed` and `quod subito exarserat …`;
  full line, no split word.
- [x] Latin confirmed  [x] Still missing (verified, tomus_3 p272)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop reads `quominus ea contentio plane erumperet, sociali
  bello,`. Head line of v1 p519.

### 8. cato_minor (p329) — v2 p329 (y0 0.056)

- Event: `miss_cato_minor_329_8786` · crop: `miss_cato_minor_329_8786_0.png`
- Reading: `rum. Interim Rhodi ad negotium tractandum se com-`
- Current-build locator: **tomus_6_inner.pdf**, seam at page boundary **283→284**
  (folios 275→276). Verified: p283 ends `… Veneris Paphiæ concessu-`, p284 begins
  `parans, responsa exspectabat. (2) …`; the reader reads broken `concessu-parans`.
  Split words `concessu-`/`rum` and `com-`/`parans`. `Interim Rhodi / negotium
  tractandum` absent. Head line of v2 p329.
- Boundary: insert between `… concessu-` (foot of reader p283) and `parans, responsa
  …` (head of reader p284).
- [x] Latin confirmed  [x] Still missing (verified, tomus_6 p283→284)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop reads `rum. Interim Rhodi ad negotium tractandum se
  com-`. Head line of v2 p329; loss lands at reader page break 283→284.

### 9. crassus — v2 p44 (y0 0.063)

- Event: `miss_crassus_44_1144` · crop: `miss_crassus_44_1144_0.png`
- Reading: `V. Ibi tum delitescenti Crasso quotidie alimenta affere-`
- Current-build locator: **tomus_5_inner.pdf physical page 72** (folio 54).
  Verified: reader jumps `… ad fontem elidit.` → `bat villicus, neminem ipse cernens
  vel cognoscens, sed ipsis`; split word `affere-`/`bat` (= *afferebat*) orphaned.
  `delitescenti Crasso / quotidie alimenta` absent. Head line of v2 p44 — **carries
  the chapter-V heading** `V.`, so the reader also loses that chapter number.
- Boundary: insert between `… ad fontem elidit.` and `bat villicus, neminem …`;
  restores chapter heading `V.` plus the line.
- [x] Latin confirmed  [x] Still missing (verified, tomus_5 p72)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop. Head line of v2 p44; also restores chapter heading `V.`

### 10. demosthenes — v2 p445 (y0 0.063)

- Event: `miss_demosthenes_445_6929` · crop: `miss_demosthenes_445_6929_0.png`
- Reading: `nunc promissis me movebis. Archia autem irato et mi-`
- Current-build locator: **tomus_7_inner.pdf physical page 41** (folio 33),
  Demosthenes' death scene. Verified: reader jumps `… neque ante personatus, neque`
  → `nitari incipiente, Nunc, ait, ex Macedonico tripode loque-`; split word
  `mi-`/`nitari` (= *minitari*) orphaned. `promissis me movebis` absent. Head line
  of v2 p445.
- Boundary: insert between `… neque ante personatus, neque` and `nitari incipiente, …`.
- [x] Latin confirmed  [x] Still missing (verified, tomus_7 p41)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop. Head line of v2 p445, Demosthenes' death scene.

### 11. lucullus — v1 p631 (y0 0.079)

- Event: `miss_lucullus_631_13150` · crop: `miss_lucullus_631_13150_0.png`
- Reading: `cum gerendum Pompeio dederat, et quam in ærarium`
- Current-build locator: **tomus_4_inner.pdf physical page 104** (folio 96).
  Verified: reader reads broken `… quam ad bellum Pirati-` / `publicum intulerat, et
  seorsum dati …` (nonsense `Piratipublicum`); split words `Pirati-`/`cum` (=
  *Piraticum*) and `ærarium`/`publicum`. `Pompeio dederat` absent. Head line of v1 p631.
- Boundary: insert between `… quam ad bellum Pirati-` and `publicum intulerat …`.
- [x] Latin confirmed  [x] Still missing (verified, tomus_4 p104)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop. Head line of v1 p631.

### 12. marcellus — v1 p377 (y0 0.081)

- Event: `miss_marcellus_377_3280` · crop: `miss_marcellus_377_3280_0.png`
- Reading: `tes petebant supplices, ut in honestam aliquam militiam`
- Current-build locator: **tomus_2_inner.pdf physical page 389** (folio 381).
  Verified: reader reads broken `(5) Hi ergo ad Marcelli pedes tunc se confertim
  provolven-` / `ascriberentur, multis adhibitis vociferationi lacrimis,`; split word
  `provolven-`/`tes` (= *provolventes*) orphaned. `petebant supplices / honestam
  aliquam` absent. Head line of v1 p377.
- Boundary: insert between `… se confertim provolven-` and `ascriberentur, multis …`.
- [x] Latin confirmed  [x] Still missing (verified, tomus_2 p389)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop. Head line of v1 p377.

### 13. nicias — v2 p25 (y0 0.067)

- Event: `miss_nicias_25_2918` · crop: `miss_nicias_25_2918_0.png`
- Reading: `fatis esse dixerant, uti hoc bellum per novem triennia tra-`
- Current-build locator: **tomus_5_inner.pdf**, seam at page boundary **32→33**
  (folios 14→15). Verified: p32 ends `… exsecrantes eos qui in` and p33 opens
  orphaned `heretur, communicatis toto de negotio sermonibus, pacem`; the dropped
  line (= `… eos qui in` + missing + `traheretur`) fell at the page break. `novem
  triennia / fatis esse dixerant` absent. Head line of v2 p25.
- Boundary: insert between `… exsecrantes eos qui in` (foot of reader p32) and
  `heretur, communicatis …` (head of reader p33).
- [x] Latin confirmed  [x] Still missing (verified, tomus_5 p32→33)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop. Head line of v2 p25; loss lands at reader page break
  32→33.

### 14. phocion — v2 p291 (y0 0.071)

- Event: `miss_phocion_291_2223` · crop: `miss_phocion_291_2223_0.png`
- Reading: `ipsi adversariis Demostheni dicenti, Occident te, o Phocion,`
- Current-build locator: **tomus_6_inner.pdf physical page 209** (folio 201).
  Verified: reader jumps `… (7) Ex oratoribus` → `Athenienses, ubi insanierint; Te
  vero, inquit, ubi sapue-`; the anecdote's setup line is dropped (only the punchline
  survives). Distinctive `Occident te` absent everywhere. Full line, no split word.
  Head line of v2 p291.
- Boundary: insert between `(7) Ex oratoribus` and `Athenienses, ubi insanierint;`.
- [x] Latin confirmed  [x] Still missing (verified, tomus_6 p209)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop. Head line of v2 p291; anecdote setup, punchline survives.

### 15. timoleon — v1 p318 (y0 0.086)

- Event: `miss_timoleon_318_9879` · crop: `miss_timoleon_318_9879_0.png`
- Reading: `largitus, audita deinde re, de qua deliberaretur, suam fere-`
- Current-build locator: **tomus_2_inner.pdf physical page 265** (folio 257).
  Verified: reader reads broken `… laudationibus acclamationibusque` / `bat
  sententiam. (5) Ea approbata ministri bigas a theatro`; split word `fere-`/`bat`
  (= *ferebat*) orphaned. `deliberaretur / audita deinde` absent. Head line of v1 p318.
- Boundary: insert between `… acclamationibusque` and `bat sententiam. (5) …`.
- [x] Latin confirmed  [x] Still missing (verified, tomus_2 p265)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop. Head line of v1 p318. Completes the head-group single-liners.

---

## Group 3 — Single-line losses at the page foot (y0 ≈ 0.86–0.89)

Likely one shared cause (shared-tail / trailing-furniture drop).

### 16. alcibiades — v1 p242 (y0 0.865)

- Event: `miss_alcibiades_242_229` · crop: `miss_alcibiades_242_229_0.png`
- Reading (Aristophanes *Wasps* verse, line 1): `Tunc Alcibiades ad me ait balbutiens :`
- Current-build locator: **tomus_2_inner.pdf physical page 108** (folio 100),
  Alcibiades ch. 1 lisp passage. Verified: reader jumps `his verbis quibus Theorum
  cavillatur :` → verse line 2 `« Cernin' Theolum? caput habet colacis. » Bene
  Alcibiades profecto illud balbutiit.`, dropping verse line 1. Foot line of v1 p242.
- Boundary: insert the **indented verse** line between `… Theorum cavillatur :` and
  `« Cernin' Theolum? …`. Restoring pixel-from-source preserves the verse layout.
- [x] Latin confirmed  [x] Still missing (verified, tomus_2 p108)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop (shown in chat). Foot line of v1 p242; first verse line
  of the Aristophanes lisp couplet, second line survives.

### 17. artoxerxes — v2 p633 (y0 0.887)

- Event: `miss_artoxerxes_633_2282` · crop: `miss_artoxerxes_633_2282_0.png`
- Reading: `ut quidam perhibent, ut alii, a Care quodam, cui præmii`
- Current-build locator: **tomus_8_inner.pdf physical page 19** (folio 11).
  Verified: reader jumps `… (2) Cadit ibi Cyrus, ictus a rege ipso,` → `hujus
  facinoris loco rex dederit, ut in bello semper aureum`; full line dropped (who
  killed Cyrus — a Carian). `Care quodam / quidam perhibent` absent. Foot line of
  v2 p633.
- Boundary: insert between `… ictus a rege ipso,` and `hujus facinoris loco rex
  dederit …`; full line, no split word.
- [x] Latin confirmed  [x] Still missing (verified, tomus_8 p19)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop (shown in chat). Foot line of v2 p633.

### 18. cato_minor (p352) — v2 p352 (y0 0.884)

- Event: `miss_cato_minor_352_15055` · crop: `miss_cato_minor_352_15055_0.png`
- Reading: `ducum considerata; quodsi tamen bona quadam fortuna`
- Current-build locator: **tomus_6_inner.pdf physical page 312** (folio 304).
  Verified: reader jumps `… imperitia et temeritate` → `eveniret, ut Cæsar
  opprimeretur, se quidem Romæ non man-`; full line dropped. `ducum considerata /
  bona quadam fortuna` absent. Foot line of v2 p352.
- Boundary: insert between `… imperitia et temeritate` and `eveniret, ut Cæsar
  opprimeretur …`; full line, no split word.
- [x] Latin confirmed  [x] Still missing (verified, tomus_6 p312)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop (shown in chat). Foot line of v2 p352.

### 19. dio (p566) — v2 p566 (y0 0.889)

- Event: `miss_dio_566_426` · crop: `miss_dio_566_426_0.png`
- Reading: `satis confirmata tyrannide deficientes Syracusani arripue-`
- Current-build locator: **tomus_7_inner.pdf physical page 284** (folio 276), Dio
  ch. III. Verified: reader reads broken `… Hermocratis Syracusani duxit filiam.
  Hanc nondum` / `runt, corpus ejus scelerata et fœda libidine ita violarunt,`;
  split word `arripue-`/`runt` (= *arripuerunt*) orphaned. Foot line of v2 p566.
- Boundary: insert between `… Hanc nondum` and `runt, corpus ejus scelerata …`.
- [x] Latin confirmed  [x] Still missing (verified, tomus_7 p284)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09)
- Notes: Human confirmed crop (shown in chat). Foot line of v2 p566.

### 20. dio (p580) — v2 p580 (y0 0.887)

- Event: `miss_dio_580_6654` · crop: `miss_dio_580_6654_0.png` (audit crop is skewed;
  a clean crop was re-rendered from source v2 p580 foot for the confirming read)
- Reading (the single dropped line, from the clean re-render): `cem navibus advectus
  est Dionysius, Dionique arma quæ` (bridges `… Septima die in ar-` and `apud Synalum
  deposuerat …`). The line above it, `… ab urbe dejunxit. Septima die in ar-`, is the
  *preceding* context and is present in the reader.
- Current-build locator: **tomus_7_inner.pdf physical page 313** (folio 305), Dio
  ch. XXIX end. Verified: reader reads broken `… Septima die in ar-` / `apud Synalum
  deposuerat, plaustris allata, quæ inter cives`; orphaned `ar-`. `navibus advectus
  est Dionysius` absent. Foot line of v2 p580.
- Boundary: insert between `… Septima die in ar-` and `apud Synalum deposuerat …`.
- [x] Latin confirmed  [x] Still missing (verified, tomus_7 p313)  [x] Boundary safe
- Decision: `PROMOTE` (human sign-off 2026-10-09, on a clean re-rendered crop)
- Notes: Audit crop was skewed/partial; confirmed on a clean crop re-rendered from
  source v2 p580 foot. Fix directive for this event: when restoring, bring in extra
  vertical source space (accepting the risk of minor duplication of an adjacent line)
  rather than tight-cropping and risking this line being clipped or mangled. (One-off
  note for this skewed crop, not a general rule.)

---

## Running tally

| Decision | Count | Events |
| --- | ---: | --- |
| PROMOTE | 20 | antonius, lysander, pericles, aemilius_paulus, agesilaus, brutus, caius_marius, cato_minor (p329), crassus, demosthenes, lucullus, marcellus, nicias, phocion, timoleon, alcibiades, artoxerxes, cato_minor (p352), dio (p566), dio (p580) |
| REJECT | 0 | |
| DEFER | 0 | |
| (unreviewed) | 0 | |
