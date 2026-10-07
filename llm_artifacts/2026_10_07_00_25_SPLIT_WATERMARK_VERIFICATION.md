# Split watermark grouping and exemption verification

Implemented audit-plan step 6.2 in `tools/audit_inner_tomi.py`.
`join_body_edges` extracts actual bottom/top body-image strips at 300 DPI from
adjacent reader pages of the same unit. The diagnostic canvas preserves the
pixel bounds, full-raster dimensions and virtual offsets of both source reader
regions. It excludes generated headings and folios by using the mapped image
placement. Neither reader page is repaired or rewritten.

`merge_logo_events` groups overlapping components and adjacent edge components
with shared occurrence-located source footer evidence. It retains every child
record, confidence condition and evidence crop. Event confidence comes from the
strongest supported child rather than from accumulating weak arcs. Matching the
virtual canvas provides additional image evidence with an explicit join-crossing
flag. Conflicting source assignments are not merged.

Frontmatter exemptions are recorded in `image-exemptions.json` from the reader
map's actual roles and image assets. Notice assets are recognized as
`front_qr_*`; their physical page number is not a detector condition. This
inventory covers the corpus as metadata, without validation reader scoring.

## Actual verification

- Run: `tmp/pdfs/inner_tomi_audit/runs/step62-aemilius`, development baseline
  `baseline_2026_10_06`, `aemilius_paulus`, watermark detector, four workers.
- All 52 requested body pages completed, with captured input hashes checked
  before and after processing.
- W4 is one high-confidence event,
  `watermark_aemilius_paulus_332d6d9e7634`, covering II/273 and II/274 with
  source v1/321 evidence. All six component records remain available.
- Its virtual stitch has a normalized-correlation peak crossing the join. The
  stitched PNG was visually inspected alongside the separate source/output
  crops: the gray logo parts align across the diagnostic join, and the resumed
  Latin remains visible. This is evidence of the retained fragments, not a claim
  that every logo pixel survives or that the prose was repaired.
- The existing localized evaluator counts W4 as a hit: one hit, zero misses,
  324 cases not requested. No benchmark labels were changed.
- Ten low-confidence inconclusive curve events on II/307 remain visible;
  grouping does not discard failed nominations. Candidate-burden calibration
  remains outstanding.
- Eight intentional notice assets across all tomi are explicitly exempted by
  role/asset. No frontmatter notice is a body candidate.
- A direct check with actual mapped images rejected Timoleon II/266 followed by
  Aemilius II/267, and Aemilius II/267 followed by II/269. The checks exercised
  the cross-unit and nonadjacent-image guards without rendering invalid joins.
- `python -m py_compile tools/audit_inner_tomi.py`, the evaluator's evidence
  identity checks and `git diff --check` passed.

No validation output was scored, no unit tests were added, and no Latin cache,
production crop, source PDF or generated reader was changed.
