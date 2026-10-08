# Inner-tomi audit: full-corpus delivery and verification (step 10.2)

This note records the completed full-corpus run of
[`tools/audit_inner_tomi.py`](../tools/audit_inner_tomi.py), its evaluation,
determinism verification, and input-integrity checks. It closes the
implementation plan
`2026_10_06_16_05_INNER_TOMI_PYTHON_AUDIT_IMPLEMENTATION_PLAN.md` (step 10.2),
building on the frozen development calibration recorded in
`2026_10_08_09_18_INNER_TOMI_VALIDATION_CALIBRATION.md`. Authoring date/time:
2026-10-08 14:44 (−04:00).

The audit **inspects** the eight facsimile inner PDFs against the original
bilingual scans; it does not repair, normalize, or touch the source Latin,
the production PDFs, the LaTeX, or the repair policy. A successful run is not
evidence that the corpus is defect-free.

## Frozen configuration

- Baseline: `baseline_2026_10_06`.
- Producer (script) SHA-256:
  `748b8684b23d74c09da84f3c7a9170eaf185ca008f3b7522519ee40b13eb6227`.
- Working/tracked labels SHA-256:
  `a96a58c6bd924704b7b4e1686cad91851a6f8503880ba182fa836e7354c747c9`.
- Freeze: `runs/step101-development-revised-1/freeze.json`, SHA-256
  `77e6d6be418b169b56ff5b52432decd70d6450971cfeb06bf7bc472c1d166fda`.
- Findings report input:
  `llm_artifacts/2026_10_06_15_40_INNER_TOMI_REINSPECTION_FINDINGS.md`, SHA-256
  `5ac02a65ce32869c42f0638b5d63980ba85cfd32d9d8feeef3371d1ccc1e0474`.

Dependency versions as recorded in the run manifest (not inferred from the
installed environment): Python 3.13.14; NumPy 2.4.3; Pillow 12.1.1;
opencv-python-headless 4.13.0.92; scikit-learn 1.9.0; RapidFuzz 3.14.3;
PyMuPDF 1.27.2; Tesseract v5.5.0.20241111; Poppler `pdftoppm` 25.12.0. OCR
language is the frozen `eng` profile (source DPI/PSM and reader DPI/PSM per the
freeze); the installed `lat` model is not part of the chosen profile.

## Final run and its artifacts

The authoritative full run is `runs/step102-final-all-5/`, invoked as:

```
python -u tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope all --detectors all --freeze tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1/freeze.json --lang eng --workers 4 --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-all-5
```

Earlier run folders `step102-final-all-1` through `-4` are aborted attempts
(three stopped externally mid-run, one a diagnostic launch that stalled before
detection); they are gitignored diagnostic scratch and carry no evidence.

Coverage and completion (manifest `complete: true`, scope `all`,
detectors `all`):

- 68 units; 2,678 body pages requested and completed (requested == completed).
- Reader OCR 2,678/2,678 pages, 0 unreadable; source OCR 1,367/1,367 pages,
  0 unreadable.
- All 12 detectors implemented; `pending_detectors` empty.
- `processing_errors`: 0. Observational limitations (kept separate from tool
  failures): 322 — 311 "source-image registration inconclusive" and 11
  "no assigned source passage; clipping inspection is inconclusive".
- Coverage rows: 2,678, with 0 rows where requested pages ≠ finished pages.

Runtimes and storage:

- First full run: elapsed 6,196.45 s; run storage before final manifest
  308,704,958 bytes; cache 6,528,828,013 bytes.
- Resume rerun (determinism): elapsed 7,657.44 s; run storage 330,343,812
  bytes; cache 6,528,828,013 bytes.
- Reused renders/OCR from the development cache; the full-corpus image
  detection dominates runtime regardless of cache.

Candidate counts: 1,379 total candidates written; 94 grouped core events.

Final artifact SHA-256 (post-resume, the committed reference state):

| Artifact | SHA-256 |
| --- | --- |
| `run-manifest.json` | `5ea74c84cd323988eb8b11076f56504161929ae163ae6b4025ada29c7c6063a6` |
| `candidates.json` | `03f31fc68aa8829b90cdf3462578d9978b05d01a5a9c895416333dd4f2c78144` |
| `candidates.csv` | `bbf3a980f3efbc3d872836869c5948f89c022290af0368a992fd773427f75d49` |
| `coverage.json` | `74088eecb4d8c082c04a2dd5bc262c9c803cee4c9b21c14702053fecd22fd6b0` |
| `errors.json` | `14ac9ed9e675027ac8046bf58136334941d90acc9978d1a7f5e3a455602cb16d` |
| `evaluation-all.json` | `aec7eebf17cda6baf6fdf307b5ceeea622b7ac9dd9c707fef920d65375aac08c` |
| `reviews.json` | `339ac5a1663bf5ea577a293d1d22aaa412b567a082e66806943c110b126b57ec` |
| `index.html` | `41ddc06e96cf7f7ae578353f11ce8b9c5151e66eb3f2d2f7d0d81fe435ba7361` |
| `boundaries.html` | `affacbad3c9660de12ff61b4d746f201abd8b76db41d0fea3146a018ce616349` |

The pre-resume `run-manifest.json` was `b3e6478b…`; only runtime fields differ
between pre- and post-resume. `candidates.json` is byte-identical across the two
runs (see determinism below). `coverage.json` and `errors.json` are byte-stable
across the resume.

## Reviews, evaluation and gates

Independent review verdicts were transferred from the revised development and
validation 10.1 runs by
`runs/step101-validation-gold-preparation/merge-final-reviews.py`, which binds
each source run's `candidates_sha256`, transfers a verdict only when the final
candidate's identity (`unit_id`/`type`/`source_regions`/`output_regions`/
`crop_sha256`/`contributions`/`incomplete_second_opening`) is identical, and
re-hashes every transferred crop. Result: **94 core events transferred, 200
distinct fixed panel pages, 0 pending events** — every final-run core candidate
matched a revised-run identity exactly, so no new adjudication was required.
`reviews.json` SHA-256 `339ac5a1…`.

Evaluation (`evaluate --split all --reviews reviews.json`, no `--freeze`, no
`--enforce-gates`):

- Counts: 51 hit, 65 miss, 9 not-requested, 0 pending-annotation,
  0 negative-violation, 200 negative-no-high-candidate (clean negative pages).
- Mandatory localized correct-type hits — all present: D4–D12; C1, C2, C3, C5;
  W4, W5, W6; H1, H2.
- Supported/qualified localized hits: S1, S2, C4, C6. C7 extent remains
  unresolved (recorded as a miss); this is the single known qualified case.
- Core burden: 94 grouped core events / 2,678 body pages = **3.510 per 100
  pages** (≤ 5 gate; equal to the 10.1 combined figure).
- Fixed panel: 200/200 pages reviewed; **0 high-confidence false-positive
  pages** and **0 high-confidence uncertain pages** (0.0 per 100 each);
  complete panel review.
- Candidate-selected reviewed precision over the 94 core events (this is a
  candidate sample, not a representative page rate): duplicate 9 confirmed /
  0 FP (1.000); watermark 3 / 0 (1.000); head 4 / 1 (0.800); missing_occurrence
  59 / 18 (0.766); order none adjudicated. The previously unlabeled /
  low-confidence-per-type cohort is retained separately by design and is not
  folded into this core precision.

Hit/miss/not-requested by detector type:

- Hits (51): duplicate 9, missing_occurrence 4, watermark 3, head 2,
  foreign_scraps 1, lateral_clipping 2, slicing 2, copied_gap 2,
  terminal_loss 26.
- Misses (65), all explicitly permitted exploratory-layout or endpoint
  localization misses: terminal_loss 42, rule 12, copied_gap 9,
  foreign_scraps 1, lateral_clipping 1.
- Not-requested (9): watermark 8, blank_body 1.

Endpoint comparisons: terminal_loss cases total 68 (26 localized + 42
algorithmic localization misses), i.e. one independent endpoint comparison per
unit, all 68 retained. The 42 misses are algorithmic localization limitations
(predominantly inconclusive source-image registration), **not** findings of
truncation: all 68 units' terminal passages were confirmed visually complete.
A matched endpoint establishes only that the ending is present; it does not
establish interior completeness.

## What the audit detected (actionable findings)

Confirmed, localized core findings in this run (candidate-selected precision
sample, with source/reader crops in `index.html`): 9 duplicated passages,
59 unique missing source occurrences, 4 surviving running-heads, 3 Google
watermark fragments. Localized layout findings include Greek-column scraps,
copied page-margin gaps and divider rules, lateral clipping and lines sliced
across output pages. The prose comparison separates **unique source loss** from
**redundant-copy loss**, so a missing redundant-copy opening is not counted as
unique loss. These located findings — not the determinism checks — are the
material for synthesizing with the earlier reinspection findings and driving
build-code changes; the audit itself changes nothing.

## HTML integrity

`index.html` references 8,006 distinct crop files and `boundaries.html` 299,
all resolving. All 7,420 candidate crop records are present and byte-match their
recorded SHA-256. 8,177 crop files on disk are all referenced (no orphans); the
references beyond candidate crops are boundary crops and retained rejected
watermark probes, shown in the debug sections. Browser visual-layout QA is a
human step and was not performed by the implementing session; the structural
link/hash verification above passed in full.

## Determinism verification

- **Resume rerun** (`--resume`, same identity, same folder): candidate records
  `identical: true`. The normalized comparison and the raw file bytes both
  match (`candidates.json` SHA `03f31fc6…` before and after). All identities,
  regions, scores, confidence predicates, raw OCR, provenance and crop hashes
  are retained in the comparison; only timestamp/duration manifest fields
  differ. Result in `resume-comparison.json` (SHA
  `2b9b6e02169da1a2912750c47e1f316645e98e2186f9616b8df446467cf6b3da`).
- **Worker 1-vs-4 comparison** on the `agesilaus`+`agis` subset (92 pages,
  separate folders `step102-final-workers1` / `step102-final-workers4`): both
  manifests complete with 2 units / 92 pages and 0 processing errors; 52
  candidates each; normalized comparison `identical: true` (raw bytes differ
  only by the normalized run-dir path prefix). Result in
  `step102-final-workers-comparison.json` (SHA
  `80ae5eba3fc2a80416a45c3d000425d1866165d97f23262a74defd8972b87ad1`).
  Subprocess workers open their own PyMuPDF documents; PyMuPDF objects are not
  shared across processes.

## Input integrity

Every live and captured input was rehashed against
`snapshots/baseline_2026_10_06/manifest.json`: 21 declared inputs (2 source
PDFs, 8 reader PDFs, 8 reader TEX files, the unit map, the repair/duplicate
policy trace, and the findings report). **21/21 live hashes match and 21/21
captured snapshot hashes match** (42 checks), the findings report live hash
matches, and the final run manifest's 21 `input_hashes` match the baseline with
0 mismatches. All inspected inputs are unchanged.

## Historical evidence and limitations

The original 9.2 freeze and `runs/step101-validation-first/` remain permanent
historical evidence; the first independent validation scored its gold at 7 hits,
46 misses, 82 passing negatives with D10/D11/D12/W5/W6 as mandatory failures.
Subsequent results — including this all-unit run — are **post-validation
regression against frozen calibration, not a fresh independent validation**; no
independent-validation claim is made for them. The independent annotation seal
is `664c8ec4303adf05e095b5236570ae0cf9afae738adbf73e1931279e022b94a2`, and the
post-validation D11 printed-word annotation correction is preserved.

Plainly stated limitations: known-case regression hits do not establish
corpus-wide recall; low confidence or an inconclusive observation is not
automatic assurance of correctness; a matched endpoint does not prove interior
completeness; and candidate-selected precision is not a representative page-level
false-positive rate. Exploratory layout and endpoint-localization misses require
image review and are reported explicitly rather than hidden. The source Latin
remains untouched.
