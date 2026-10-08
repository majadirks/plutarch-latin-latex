# Head and folio correspondence verification

Implemented audit-plan step 7.1 in `tools/audit_inner_tomi.py`.
`find_head_candidates` combines observed capital fragments, separate 2-4-digit
folios, isolation, original source-head suffixes, exact source-folio agreement
and source-image components. Three cues and an occurrence-assigned source
transition are required for high confidence. Single-character cases need image
corroboration. Related head/folio components share one event without expanding
the observed clipped heading.

`source_head_references` uses the existing PDF adapter to extract original
full-width header observations and preserves their raw text and unknown embedded
OCR confidence. `head_templates` derives suffix/folio image hypotheses from
original source pixels. The shared `match_templates` helper supports dark head
ink as well as faint watermark ink. Matching must overlap the implicated output
region; a neighboring glyph cannot supply its image evidence.

Roman chapter patterns and multiword prose lines are excluded. A lone OCR word
does not establish prose: the Greek H2 fragment was misread as `by` by the coarse
English OCR. Image-component evidence preserves that uncertainty and finds the
original header fragment. Regional 300-DPI PSM 11 retries are saved alongside
the coarse observations. Installed OCR languages are `eng`, `lat` and `osd`;
no Greek model was available or installed.

## Actual verification

- Final run: `tmp/pdfs/inner_tomi_audit/runs/step71-final`, development baseline
  `baseline_2026_10_06`, units `agis` and `philopoemen`, head detector.
- All 59 body pages completed: 28 Agis and 31 Philopoemen pages.
- Two grouped high-confidence head events, and no other head event in these
  units. H1 includes the clipped word and separate 435 folio at III/123, with
  original v1/448 source regions. H2 localizes the image-supported Greek head
  fragment at VII/29 against original v2/375.
- H2's image correlation is 0.967417. H1's head image correlation is 0.989021;
  its folio agreement is exact. The grouped H1 primary score 1.0 denotes that
  exact agreement, not a probability.
- The existing evaluator reports two hits, zero misses, two pending negative
  annotations and 321 cases not requested. No labels were changed.
- Source/output head and folio crops were visually inspected. They show the
  clipped word, 435, and the retained Greek fragment. The output is not expanded
  into the complete source head.
- Both prescribed sensitivity folds used the same recorded settings and script
  identity from `runs/step71-folds/`. `step71-fold-philopoemen` and
  `step71-fold-agis` each produced one localized hit and no miss; no settings
  changed between the fold runs.
- These folds are limited sensitivity checks: both units participated in
  preceding development calibration. They are not independent or blind
  validation. Validation units remain unopened for detector scoring.
- Captured input hashes were verified before and after processing. The evaluator
  verified evidence identities; `python -m py_compile tools/audit_inner_tomi.py`
  and `git diff --check` passed.

Greek character reading remains uncertain when OCR fails or substitutes a Latin
lookalike; high confidence refers to the source-head leak's presence and location.
The pending negative annotations and the corpus-wide review panel are not
claimed as scored negatives. Greek scraps remain a separate detector task.
No source Latin, transcription cache, production mask or generated reader was
changed; no unit tests or Rust checks were added or run.
