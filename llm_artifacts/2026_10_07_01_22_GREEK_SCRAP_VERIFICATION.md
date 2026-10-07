# Neighboring Greek-column scrap verification

Implemented audit-plan step 7.2 in `tools/audit_inner_tomi.py`.
`find_foreign_scraps` keeps neighboring-column remnants separate from running
heads. It projects original Greek-word regions into the actual body image,
checks marginal connected components, and corroborates the implicated pixels
against the full 300-DPI source/output images. At least three components across
two lines are needed. Single edition reference letters cannot qualify as
Greek-word correspondence.

The shared `match_source_image`, `project_source_box` and
`compare_registered_region` helpers register retained prose using source-only
and output image features. They do not consult production masks, crop settings,
finding IDs or gold coordinates. Source/reader pixel identities, geometry,
comparison settings and OpenCV version bind the cached transform. Failed or
weak registration remains explicit. The same PDF-word adapter supplies head
and body word observations with unchanged raw OCR and unknown embedded-word
confidence.

## Actual verification

- Run: `tmp/pdfs/inner_tomi_audit/runs/step72-pyrrhus`, development baseline
  `baseline_2026_10_06`, `pyrrhus`, scrap detector.
- All 57 requested body pages completed; captured inputs were rehashed before
  and after processing.
- One qualified low-confidence event, `scraps_pyrrhus_173`, matches the report's
  development F2 location: III/173 against v1/470.
- The event contains 20 measured components across 16 component-line groups.
  Individual source/output region correlations range from 0.821952 to
  0.970616. These are image similarities, not recognition probabilities or
  exact counts of complete Greek characters.
- Source 470 registration has 740 inlier feature correspondences and inlier
  fraction 0.783069. The measured transform is stored with its source/reader
  dimensions and source inlier bounds.
- The marginal reader crop and an original-source strip were visually inspected.
  They show corresponding Greek word tails. The localized output strip excludes
  the legitimate Latin-column reference letters beside them.
- No other Pyrrhus page supplied enough corroborated marginal components.
  Per-page registrations and zero/nonzero counts are retained in
  `scrap-debug-pyrrhus.json`.
- The evaluator accepts candidate identities and coverage. F2 and its reference
  negative remain pending gold annotations, so evaluation reports two pending
  annotations and 323 cases not requested, with no claimed scored hits or clean
  negatives. No labels were changed to fit the detector.
- `python -m py_compile tools/audit_inner_tomi.py`, evidence identity checks and
  `git diff --check` passed. No unit tests or Rust checks were added or run.

The Greek raster OCR model is unavailable. The event establishes qualified
image correspondence, not the exact readings of the clipped glyphs. The
validation F1 Theseus example has not been scored. Original Latin, transcription
caches, production crops/masks and generated readers remain unchanged.
