# Tier-B omission losses — the confirmed losses awaiting a human promotion read

Written 2026-10-09. This note enumerates the **Tier-B** omission events from the
inner-tomi audit: real, independently *confirmed* losses of source Latin that the
2026-10-08 repair pass deliberately left untouched. Each one is held back because
enabling a safe fix first needs a human to read the exact source line(s) and the
proposed preservation boundary, then "promote" the event into the repair (Tier-A)
set. This note is the worklist for that review; it changes no code and promotes
nothing on its own.

## Where these come from and why they are held back

The authoritative audit run is `step102-final-all-5`. Its independent review
confirmed **59 unique-loss events / 113 source Latin lines** across 31 units. The
[`repair plan`](2026_10_08_16_00_INNER_TOMI_AUDIT_REPAIRS_PLAN.md) split the 59
into two disjoint eligibility tiers:

| Tier | Membership | Events | Lines | Status |
| --- | --- | ---: | ---: | --- |
| A | 37 confirmed high-confidence events + human-confirmed C1 (Romulus) and C2 (Coriolanus) | 39 | 69 | **All restored** — see the [repair verification](2026_10_09_00_10_INNER_TOMI_AUDIT_REPAIRS_VERIFICATION.md) |
| B | the remaining confirmed **low-confidence** events | 20 | 44 | **Pending** — this note |

"Tier B" is an eligibility label, not a confidence or verdict. Every event below
is `verdict == "confirmed"` and `unique_loss == true`; the line really is absent
from its reader position and does not survive through an established equivalent
scan. "Low confidence" means the detector's machine evidence was weaker (typically
one checklist condition — e.g. context coverage or image registration — did not
pass), not that the loss is doubtful. The repair plan's gate is explicit: **no
repair is enabled for a Tier-B event until a human confirms the exact source lines
and the proposed preservation boundary and promotes the event to Tier A.**
Promotion changes eligibility only — never the historical detector confidence or
the review verdict.

Binding identities (from the frozen run; verify before trusting any locator):

- `candidates.json` SHA-256 `03f31fc68aa8829b90cdf3462578d9978b05d01a5a9c895416333dd4f2c78144`
- `reviews.json` SHA-256 `339ac5a1663bf5ea577a293d1d22aaa412b567a082e66806943c110b126b57ec`
- The durable inventory these records are drawn from is the JSON block at the end
  of the repair plan. The source/reader evidence crops live under
  `archive/inner_tomi_audit_2026_10_08/runs/step102-final-all-5/crops/`
  (Git-ignored; supply the local archive separately on another machine).

### Reading the locators

Pages are **one-based physical pages of the source scan**, not reader folios.
`source_index` `0` = `Ploutarchou_Bioi_v1.pdf` (`v1`), `1` = `v2`. A box
`[x0, y0, x1, y1]` is a fraction of the rendered full source page with `(0,0)` at
top-left, so a small `y0` is near the **top** of the page and a large `y0` is near
the **bottom**. These are *historical baseline* source coordinates for locating
the line in the scan; they are **not** current-build reader repair locators.

## The 20 Tier-B events

Sorted by line count, then unit. "Seam" records where on the source page the
missing line sits — the single most diagnostic fact about each loss (see the
pattern below).

| # | Event ID | Unit | Source | Page | Lines | Seam (y0) | Evidence crop prefix |
| --- | --- | --- | --- | ---: | ---: | --- | --- |
| 1 | `miss_antonius_562_20869` | antonius | v2 | 562 | **23** | head→mid (0.083–0.479) | `miss_antonius_562_20869_*` |
| 2 | `miss_lysander_535_1762` | lysander | v1 | 535 | 2 | head (0.076) | `miss_lysander_535_1762_*` |
| 3 | `miss_pericles_214_9084` | pericles | v1 | 214→215 | 2 | **page-spanning** foot(0.863)→head(0.079) | `miss_pericles_214_9084_*` |
| 4 | `miss_aemilius_paulus_326_3162` | aemilius_paulus | v1 | 326 | 1 | head (0.087) | `miss_aemilius_paulus_326_3162_*` |
| 5 | `miss_agesilaus_110_2063` | agesilaus | v2 | 110 | 1 | foot (0.886) | `miss_agesilaus_110_2063_*` |
| 6 | `miss_alcibiades_242_229` | alcibiades | v1 | 242 | 1 | foot (0.865) | `miss_alcibiades_242_229_*` |
| 7 | `miss_artoxerxes_633_2282` | artoxerxes | v2 | 633 | 1 | foot (0.887) | `miss_artoxerxes_633_2282_*` |
| 8 | `miss_brutus_619_10158` | brutus | v2 | 619 | 1 | head (0.063) | `miss_brutus_619_10158_*` |
| 9 | `miss_caius_marius_519_9217` | caius_marius | v1 | 519 | 1 | head (0.078) | `miss_caius_marius_519_9217_*` |
| 10 | `miss_cato_minor_329_8786` | cato_minor | v2 | 329 | 1 | head (0.056) | `miss_cato_minor_329_8786_*` |
| 11 | `miss_cato_minor_352_15055` | cato_minor | v2 | 352 | 1 | foot (0.884) | `miss_cato_minor_352_15055_*` |
| 12 | `miss_crassus_44_1144` | crassus | v2 | 44 | 1 | head (0.063) | `miss_crassus_44_1144_*` |
| 13 | `miss_demosthenes_445_6929` | demosthenes | v2 | 445 | 1 | head (0.063) | `miss_demosthenes_445_6929_*` |
| 14 | `miss_dio_566_426` | dio | v2 | 566 | 1 | foot (0.889) | `miss_dio_566_426_*` |
| 15 | `miss_dio_580_6654` | dio | v2 | 580 | 1 | foot (0.887) | `miss_dio_580_6654_*` |
| 16 | `miss_lucullus_631_13150` | lucullus | v1 | 631 | 1 | head (0.079) | `miss_lucullus_631_13150_*` |
| 17 | `miss_marcellus_377_3280` | marcellus | v1 | 377 | 1 | head (0.081) | `miss_marcellus_377_3280_*` |
| 18 | `miss_nicias_25_2918` | nicias | v2 | 25 | 1 | head (0.067) | `miss_nicias_25_2918_*` |
| 19 | `miss_phocion_291_2223` | phocion | v2 | 291 | 1 | head (0.071) | `miss_phocion_291_2223_*` |
| 20 | `miss_timoleon_318_9879` | timoleon | v1 | 318 | 1 | head (0.086) | `miss_timoleon_318_9879_*` |

Totals: **20 events, 44 source Latin lines.** One 23-line block (Antonius), two
2-line events (Lysander, Pericles), seventeen single-line events.

## The pattern: every Tier-B loss is a page-seam casualty

The seam column is not incidental — it is the whole story. Classifying the 20
events by where the missing line sits on its source page:

- **13 single-line losses at the page head** (y0 ≈ 0.06–0.09): the missing line is
  the **first body line** of a source page — alcibiades is the exception shape;
  these are: aemilius_paulus, brutus, caius_marius, cato_minor/329, crassus,
  demosthenes, lucullus, marcellus, nicias, phocion, timoleon, plus lysander
  (2 lines) and antonius (a 23-line block that *starts* at the head and runs to
  mid-page).
- **6 single-line losses at the page foot** (y0 ≈ 0.86–0.89): the missing line is
  the **last body line** of a source page: agesilaus, alcibiades, artoxerxes,
  cato_minor/352, dio/566, dio/580.
- **1 page-spanning loss** (pericles): a line that straddles the v1 214→215 break,
  present as a foot fragment on 214 and a head fragment on 215.

Not one Tier-B loss sits in the undisturbed interior of a page. This matches the
repair plan's leading mechanism hypothesis exactly: the generator's furniture
trims act at page boundaries, and that is precisely where real body lines get
caught. The responsible code (all in `src/facsimile/`) is:

- **Head losses** → the opening/interior head trims: `drop_opening_bands` (first
  page of a unit), `drop_interior_running_head` → `drop_trailing_furniture`, and
  column detection clipping the top line. A first body line that sits close to a
  running head or a disturbed top margin can be trimmed as furniture.
- **Foot losses** → `drop_shared_tail` (a shared last page) and
  `detached_tail_cluster_start` / `drop_detached_rule` in `drop_trailing_furniture`:
  a last body line after an above-median chapter gap, especially if it is narrow
  (laterally clipped, so `< 0.8 × column width`), gets read as a detached furniture
  cluster and dropped.
- **Page-spanning loss** (pericles) → the line is split by the 214/215 page break
  and each half falls below a per-page retention floor; neither half survives.

This is the same family the Tier-A repairs already fixed (e.g. C1 Romulus was a
head-region loss driven by a left divider rule disturbing column detection). The
difference is only that each Tier-A event had either high machine confidence or a
prior human eyeball; the Tier-B events reached "confirmed" on weaker machine
evidence and were parked behind the promotion gate.

## Approximate transcriptions (locating aid only — the scan is authoritative)

These readings were taken by eye from the archived source crops to help a reviewer
find and recognize each line. They are **not** a transcription of record: the
printed scan is the single source of truth, and any fix preserves the line from the
source raster — it is never retyped from this table. Expect scan/reading noise
(e.g. `æ`/`œ` ligatures, long-s, and the edition's `u/v`, `i/j`). Edge hyphens mark
a word continuing onto the adjacent line.

| Event | Reading (approximate) |
| --- | --- |
| antonius (23 ll.) | `asser`[vatam] `dicunt et aureo fuso a Cleopatra irritatam et lacessitam brachium arripuisse. (2) Quid verum sit, nemo novit …` through the Life's end: `Mortua est Cleopatra, annos nata undequadraginta … Archibius quidam ejus amicus bis mille talentis a Cæsare redemit.` |
| lysander | `gerere volebant, eos rideri jussit : nam ubi pellis leonina non sufficeret, ibi vulpinam assuendam esse.` |
| pericles (2 ll., 214→215) | `Megaram petentes Simætham meretriculam` / `furto abstulerunt adolescentes ebrii :` |
| aemilius_paulus | `dedisset, strenue pugnarent, gladiisque Romano more ute-` |
| agesilaus | `bellum cepit, id quod Agesilao pergratum fuit. (2) Ma-` |
| alcibiades | `Tunc Alcibiades ad me ait balbutiens :` |
| artoxerxes | `ut quidam perhibent, ut alii, a Care quodam, cui præmii` |
| brutus | `quum leviter extremas partes attigissent paucosque d[u]-` |
| caius_marius | `quominus ea contentio plane erumperet, sociali bello` |
| cato_minor (p329) | `rum. Interim Rhodi ad negotium tractandum se com-` |
| cato_minor (p352) | `ducum considerata; quodsi tamen bona quadam fortuna` |
| crassus | `V. Ibi tum delitescenti Crasso quotidie alimenta affere-` |
| demosthenes | `nunc promissis me movebis. Archia autem irato et mi-` |
| dio (p566) | `satis confirmata tyrannide deficientes Syracusani arripue-` |
| dio (p580) | `…arcem ducto muro ab urbe disjunxit. Septima die in ar-cem navibus advectus est Dionysius.` |
| lucullus | `cum gerendum Pompeio dederat, et quam in ærarium` |
| marcellus | `tes petebant supplices, ut in honestam aliquam militiam` |
| nicias | `fatis esse dixerant, uti hoc bellum per novem triennia tra-` |
| phocion | `ipsi adversariis Demostheni dicenti, Occident te, o Phocion,` |
| timoleon | `largitus, audita deinde re, de qua deliberaretur, suam fere-` |

Every reading is a grammatically continuous line fragment ending (or beginning) in a
hyphenated word-break at the page edge — consistent with a dropped first/last body
line rather than OCR noise or a false alarm.

## What a promotion read must establish, per event

For each event the reviewer opens the two evidence crops (`_0` the expected source
region, `_1`+ the reader neighborhood where it should appear) from the archive,
or re-renders the source box from the pinned source PDF, and confirms:

1. **The exact missing Latin** — the glyphs, spelling, punctuation, capitalization,
   ligatures, and any split-word continuation across the seam. This text is source
   material: it will be *preserved* from the source raster, never retyped,
   normalized, or OCR-replaced.
2. **The loss is real in the current build** — the baseline coordinates are
   historical; confirm the line is still absent from today's rebuilt reader at the
   corresponding position, not already restored by an unrelated change.
3. **A safe preservation boundary** — which neighboring lines must stay intact and
   in order, so a fix restores the line without duplicating it or disturbing
   genuine printed chapter/paragraph spacing.

Only after those three are signed off does the event move to Tier A and become
eligible for the same minimal, edition-specific repair mechanism (a verified
furniture mask on the owned source raster, or a trim-boundary correction) used for
the Tier-A set.

## Suggested review order

1. **The 23-line Antonius block** (`miss_antonius_562_20869`) first — it is by far
   the largest loss (half of all 44 Tier-B lines) and the repair plan calls it out
   by name as the headline Tier-B item. It starts at the page head and runs to
   mid-page, so it is almost certainly an opening-trim / column-detection casualty
   rather than a foot artifact.
2. **The two 2-line events** (Lysander head, Pericles page-spanning) next — Pericles
   is the only split-across-pages case and deserves its own look.
3. **The 17 single-line events**, batched by seam: the 13 head-region losses likely
   share one opening/head-trim cause, and the 6 foot-region losses likely share one
   trailing-furniture cause — so confirming a couple of each may clarify the rest.
