# Automatic real-control checkpoint

## Acceptance behavior

The delivery command accepts `controls --observations <general-glyph-cuts.json>`
to score neighborhoods selected by an authenticated, completed comparison run.
The observation file must match the snapshot, runtime, comparison-record digest
and executed observation code. Source/reader input identities are checked even
when no automatic neighborhood is available. The fixed source-control rectangle
is remeasured in full; a reviewed subregion never reduces that reference.
Expected reader page pairs are used only for final scoring. Missing, ambiguous
or incorrect neighborhoods fail with exit 4 without searching the known pair.

The source-only context attempt is bounded and complete. Two source pages were
reviewed independently of reader survival: volume I pages 105 and 200. Mixed
Greek/Latin parent regions remain uncertain; twelve contained Latin spans have
reviewed geometry and ownership. All other source ink remains inventoried.
The policy is `source-controls/policy-real-splits-context-v5.json`, and capture
`calibration_real_splits_context_v5` has snapshot identity
`b1589e68718b51cda4fab9c3ff3f8a8c8b26ad266beff427504c2b0ec0a92cdf`.
These source reviews do not certify either full acceptance-control rectangle.

Normal comparison processes both source pages, 477 source-region records and
63 reader pages, and completes coverage/seam reporting. It still supplies no
eligible automatic neighborhood for either defect: the ordered anchor evidence
needed around the damaged lines is unavailable. Automatic acceptance therefore
scores **0/2**, not passed. No further manual context tuning is authorized by
this bounded experiment's completion criterion.

## Verification

Evidence paths below are relative to
`tmp/pdfs/facsimile_line_audit/runs/calibration/`:

- `real_split_context_normal_pipeline/`: completed normal comparison.
- `real_split_context_general_3.json`: automatic companion observations.
- `real_split_automatic_acceptance_2.json`: two recorded automatic misses,
  exit 4, with no fixed-pair search.
- `real_split_targeted_final.json`: both targeted defects detected, exit 0.
- `glyph_cuts_automatic_scorer_probe.json`: general observations of the
  independently completed real calibration run.
- `intact_automatic_neighborhood_result.json`: actual reviewed volume II
  page 562 region automatically selected on reader pages 278–279. The intact
  row has no persistent supported glyph cut. This deliberately unmet positive
  control exits 4, proving the nonempty automatic selection/measurement path
  does not force a warning.

Python compilation and the source-backed real probes pass. Source text, accepted
readers, production Rust and repair ledgers are unchanged. The session's Rust
formatting, strict Clippy and full-suite gates passed before this independent
Python change. The preexisting user `BACKLOG.md` edit is excluded from commits.

## Corpus execution failure and next work

`runs/corpus_7_2_candidate` stopped with an OpenCV SIFT allocation failure after
its last printed registration counter, **280,801/358,744**. It inventoried all
1,315 requested source pages but produced no final full coverage ledger or seam
report. Its manifest remains incomplete; its capture, raw caches and failure log
are retained. The counter counts source-region correspondence records, not
confirmed Latin lines or successful strip fits.

The next engineering change should defer registrations on physical source pages
with no reviewed geometry. Preserve every region and mark deferred image evidence
unresolved. Register all regions on physical source pages containing reviewed
geometry, including unreviewed neighbors: their shared continuous fits can help
measure reviewed lines on the same source page. This preserves every fit that
the current coverage scorer can use to certify a line. Source masks, glyph/ink
thresholds and Latin remain unchanged. A changed comparison implementation
requires a fresh capture and all seven controlled-case regressions before the
corpus retry; previous predictions remain historical evidence.

The deliverable remains an auditable script, ledger, evidence/report bundle and
explicit unresolved scope. Neither targeted success nor a successful processing
exit approves calibration or certifies complete readable Latin.
