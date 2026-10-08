# Independent validation annotation evidence

This note records the held-out annotation review of all 138 validation cases
completed before opening the first validation detector candidate/debug results.
The original frozen producer, working specs and source/reader inputs were
unchanged during that review. Annotation-only image rendering and independent measurements do not
fit the detector or establish its performance.

## Evidence and identities

The original-source/reader overview run is
`tmp/pdfs/inner_tomi_audit/runs/step101-validation-gold-overviews/`.
Independent measurements, raw observations, image review ledger and proposed
labels are in `runs/step101-validation-gold-preparation/`. The localized
annotation-only run `runs/step101-validation-gold-final/` completed all 138
cases at 300 DPI and includes `index.html`, contact sheets, native crops,
`annotation-draft.json` and a complete `annotation-manifest.json`.

The proposed labels SHA-256 is
`664c8ec4303adf05e095b5236570ae0cf9afae738adbf73e1931279e022b94a2`.
`independent-evidence-inventory.json` records hashes and sizes for 699 PNG
evidence files, the reviewed case list, completed image ledger and the fact
that validation predictions had not been opened. All 325 proposed cases have
reviewed annotation status. All 187 development case objects and the four
previously reviewed validation cases C5/W5/W6/S2 are unchanged.

The first validation run used the original labels bound to the original
freeze. The sealed proposal remains in the ignored diagnostic work tree. Its
separate retrospective score applies the unchanged localized matcher to the
actual first-run candidates and these independently prepared annotations,
identifies both hashes, and preserves the original pending-label evaluation. Installing reviewed labels requires
new run identities and a fresh development freeze under the current tool's
strict spec binding.

## Mandatory core evidence

D9-D12 each have independently measured source regions and two reader
contributions with start/end word boxes and line heights. Native boundary
crops and source/reader page contacts were inspected for every contribution.
D9 covers Cicero's repeated block; D10 covers Antonius; D11 covers Aratus;
D12 covers Otho. D11's independent reader OCR observes `ase` across the
printed `a se`; the raw observation remains unchanged. D12's second
contribution starts at `rimum temporis`, after a two-line prefix retained in
its first contribution. The source breaks the word `plurimum` as `plu-` /
`rimum`. Missing redundant prefix lines do not establish unique source loss.

C5 original source 2/624 starts with the complete Latin line introducing
Lucilius. Reader VIII/128 resumes the following `Is quum videret` line after
the preceding source page's ending. Full native reader pages VIII/127-129
were inspected: the introduction is absent, and later Lucilius mentions do
not supply it. The independently retained image
`C5-readers-127-129.png` and raw 300-DPI reader observations document the
check. This is a single unique line loss, distinct from the nearby copied
gap, logo and margin rule.

W5/W6 retain faint gray upper Google glyph fragments on reader VIII/87 and
VIII/128. Native source footer logos and actual reader fragments were
inspected side by side. S2's source line beginning `IV. Exsilii vero calamitas`
corresponds to upper glyph fragments at VII/244's bottom and the lower
remainder at VII/245's top. C7's later clipped Otho repeat opening has a
localized inspected region; its total lateral extent and precise missing
glyph count remain unconfirmed.

## Layout, endings and negative examples

All four validation copied gaps, six rule cases and the neighboring Greek
fragments F1 have independently inspected native source/reader images.
Genuine chapter/verse spacing and the ordinary Caesar chapter gap remain
negative examples. Five prior-mask negatives are measured around the actual
historical header/rule neighborhoods; frozen mask coordinates serve only as
annotation locators. Detectors do not consume those repair records or gold
coordinates. Adjacent Latin is visibly retained.

All 34 validation endings were compared against their actual source closing
prose and found visibly complete. Sparse ending pages therefore remain
negative examples for divider/empty-prose claims. Generated heading regions
on all 34 validation units were inspected as local exemptions, without
declaring the rest of each page free of defects. Three intentional Google
Books notice pages remain nonbody exclusions. Source Otho 709/710 contain
repeated ending scans; the selected reader retains the ending once.

## Independently confirmed annotation corrections

R8's original report describes a retained divider at Lucullus IV/48. Full
300-DPI reader imagery and native strips along both seam margins show no
vertical rule there. An independent whole-body narrow-stroke measurement
finds zero vertical components. The original v1/603 does have an outer bar;
v1/604 supplies continuation context. The proposed R8 label is negative and
retains the original report row and a correction explanation in provenance.
M9 remains a copied-gap positive. `R8-annotation-discrepancy.json` preserves
the native reader image hash, measurement and explicit independently
confirmed annotation-error status. No validation detector prediction was
used to make this correction. The pinned findings report and snapshots
remain untouched.

R11 source 674 and R13 sources 604/624 are contextual continuation pages
without the reported stroke. Their original locator records remain in
context provenance; positive source regions bound the actual visible bars.
Annotation helpers also excluded printed-number suffixes from E21/E68's
terminal prose boxes using a general shortest-window tie rule and rejected
a short unrelated reference when locating the Caesar chapter marker. Native
images confirm these measurements; no detector threshold or predicate was
changed.

## Fixed page review

All 107 validation pages in the preselected 200-page panel were viewed on 54
full-page contact sheets. `runs/step101-validation-panel-images/panel-manifest.json`
records original input/render/PNG hashes and completed reader-image review.
The independent page-image review was completed before candidate
adjudication. The separately retained first-run reviews subsequently adjudicate
all touching core events; revised runs require their own evidence binding.
The 93 development panel reviews remain in the immutable development run.
Candidate-selected precision and known-case recall are separate measurements.
