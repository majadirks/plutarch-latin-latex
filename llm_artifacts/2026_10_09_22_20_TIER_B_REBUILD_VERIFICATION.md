# Context

The production reader includes all 20 installed Tier-B events / 43 measured
lines. This is the first rebuild verification pass; reader acceptance and the
full delivery selector remain gated by the delivery review. The source-artifact
acceptance covers the Greek fragments, without implying restored status.

# Fresh compilation

The complete offline facsimile generator ran with callable MiKTeX LuaLaTeX and
took its compile branch. Generated assets were preserved before normal cleanup,
restored for two additional inner passes, and used to regenerate impositions.
All eight PDFs are parseable and newer than the recorded build start. Their final
page counts match the counts used by their imposition wrappers.

| Tomus | Inner reader pages |
| --- | ---: |
| 1 | 416 |
| 2 | 448 |
| 3 | 416 |
| 4 | 128 |
| 5 | 384 |
| 6 | 480 |
| 7 | 448 |
| 8 | 192 |

Immutable copies of all eight readers, provenance, source-test logs, compiler
logs, and the rendered Antonius seams are retained in
`archive/tier_b_omission_repairs_2026_10_09/final-reader-build/`. The 43-file
manifest has SHA-256
`a4c4b5dfbf5a187dd5fbe5c2d9bb8cff7193302eea6b86491780c3b6c976c916`.
Build provenance records source PDF/unit-table/production-file digests, command
exit codes, timestamps, the unchanged full-work scale 1.2023952, the 69 permanent
protection records, and the final Antonius crop [876,1721). Its recognized column
uses scale 1004/845 to retain the complete skewed ending.

# Passing verification

- Antonius source verification: one event / 22 core lines, with complete coverage
  of 38 reviewed regions including annotations, chapter, ending, and predecessor.
- Non-Antonius source verification: 19 events / 21 measured lines across 17 units.
- Historical Tier-A source verification: all initially restored events are checked
  with their historical coverage policy, alongside current repairs in those units.
- Ordinary suite: 418 library tests and nine CLI tests; 75 tests explicitly ignored.
- Formatting and strict all-targets Clippy: clean.
- Archive authentication: all eight `audit_evidence_` tests passed, including every
  baseline unit trace, staged evidence, historical manifests, and exact portable
  projection equality.
- Evidence-free offline snapshot: the ordinary suite passes, and the deliberate
  missing-archive probe fails for the expected missing evidence.

The portable fixture contains 136 authenticated semantic documents, 434,900 bytes.
The new Antonius report preserves all 135 previous projected documents unchanged.
The full delivery selector deliberately waits for a declared partial/complete
state and rebuilt-reader human review. Final baseline comparison, seam locators,
and reader acceptance belong to the delivery verification; this pass does not
declare the plan complete.
