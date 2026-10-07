# Inner-tomi audit setup

`audit_inner_tomi.py` is an offline diagnostic tool. Its commands check
prerequisites, capture the inspected baseline, render benchmark annotations,
extract OCR, compare source/reader occurrences and boundaries, and evaluate
localized candidate evidence. Greek scrap and supplemental layout detectors
are reported as pending until implemented.

Use the existing Python 3.10+ interpreter. If `python` does not resolve in a
PowerShell session, use its actual executable path; no reinstall is required.

```powershell
python tools/audit_inner_tomi.py --help
python tools/audit_inner_tomi.py check
```

The check prints interpreter, package, Poppler/Tesseract versions and installed
OCR models. Explicit tool paths are available through `check --pdftoppm PATH
--tesseract PATH`. Dependencies are pinned to versions verified with the existing
Python 3.13.14 interpreter. If imports are missing, install them separately:

```powershell
python -m pip install -r tools/requirements-inner-audit.txt
```

The tracked `inner-audit-baseline-inputs.json` preserves the inspected input
identities. Prepare the working spec deliberately, then capture:

```powershell
New-Item -ItemType Directory -Force tmp/pdfs/inner_tomi_audit/spec
Copy-Item tools/inner-audit-baseline-inputs.json tmp/pdfs/inner_tomi_audit/spec/baseline-inputs.json
python tools/audit_inner_tomi.py capture --repo-root . --work-dir tmp/pdfs/inner_tomi_audit --baseline-spec tmp/pdfs/inner_tomi_audit/spec/baseline-inputs.json
```

Capture verifies originals before copying, verifies each copy, and rehashes
originals before writing the completed manifest. A mismatch stops capture; it
never substitutes a different reader. Existing snapshots, including partial
captures, are preserved. Completed snapshots cannot be overwritten by the tool.
Snapshot input paths preserve the repository-relative tree.

Ordinary diagnostic writes are permitted beneath the working `cache/` and
`runs/` directories. Resolved destinations in snapshots, specs, or live input
locations are rejected, including redirected paths. `resolve_baseline` accepts
an exact baseline ID and verifies every captured input without falling back to
live files. These protections are tool policy, not filesystem access controls
against unrelated programs.

The generated work directory is ignored by Git. Snapshot copies are local;
the branch contains the script, dependencies, pinned spec, and these instructions.
No Latin, production source, or reader is edited by capture.

The tracked `inner-audit-labels.json`, `inner-audit-splits.json`, and
`inner-audit-templates.json` preserve report-derived benchmark data. Copy these
deliberately into the working `spec/` directory as `labels.json`, `splits.json`,
and `templates.json`. Runtime annotation never edits specs. The fixed split
contains 34 development units and 34 validation units. Its 200-page panel was
selected from fresh captured-PDF image occurrences before detector scoring.

```powershell
python tools/audit_inner_tomi.py annotate --baseline baseline_2026_10_06 --labels tmp/pdfs/inner_tomi_audit/spec/labels.json --cases C1 C2 C3 --run-dir tmp/pdfs/inner_tomi_audit/runs/my-gold-review
```

Annotation saves full-resolution source/output crops, contact sheets, an HTML
review page, and a draft in the explicit run directory. It may view validation
gold evidence because it performs no detector scoring. Omit `--cases` to render
all reviewed regions. Pending records render full-page overviews labeled
`overview_not_gold`; they cannot establish localized evaluation hits. Use a new
run directory for changed annotations. Templates contain development source
logos; reader-output images are forbidden as templates.

The case inventory includes all required report groups and 68 endpoint labels.
Eleven findings have manually measured, inspected regions. Duplicate occurrence
locators and contribution endpoints are recorded, but their boxes remain
pending, as do the remaining layout, endpoint, and negative annotations. The
25 prior-mask neighborhoods have separate defect-type negatives rather than
whole-page clean claims. Pending annotations must be measured and reviewed
before final calibration; panel pages also remain unreviewed.

Renders use the full MediaBox and identity/settings-checked caches. Word-box
overlays were inspected on a source page and a generated reader footer. Source
embedded OCR boxes are visibly displaced from some glyphs; gold boxes are
measured from raster ink. All reader inventory pages had standard unrotated
MediaBox/CropBox geometry. Rotated or nonzero-origin
CropBox cases were not exercised; render metadata identifies unusual geometry
as requiring independent overlay inspection.

Evaluate an explicit run directory:

```powershell
python tools/audit_inner_tomi.py evaluate --run-dir tmp/pdfs/inner_tomi_audit/runs/evaluator-localized --split development
```

Each run contains `run-manifest.json` and `candidates.json`. The manifest binds
the baseline/input hashes, the working labels/splits/templates file hashes,
semantic `settings` and their canonical JSON hash, candidate file hash, selected
`units`, `scope`, `detectors`, and explicit requested/completed body-page lists.
Body-page keys use `TOMUS:PHYSICAL_PAGE`, such as `1:68`. `producer` distinguishes
manual evaluator verification from detector output. A changed configuration or
input requires a new run. The evaluator validates page ownership using cached
metadata from captured image occurrences, so changing a candidate's unit name
cannot smuggle a validation page into development.

Candidate JSON has `schema_version: 1` and a `candidates` list. Each event needs
an `event_id`, `unit_id`, `type`, machine-readable `relation`, source/output
regions with captured input hashes and normalized boxes, `confidence`,
`confidence_reasons`, `status`, `evidence_provenance`, and existing `crops` with
work-relative paths and SHA-256 hashes. Omissions use
`relation: missing_expected_position` and `unique_loss: true`. Duplicates use
`relation: repeated_occurrence`, two nonoverlapping `contributions` with measured
start/end page boxes, and the incomplete-second-opening assessment. Other
relations are defined in the script's shared `RELATIONS` table. Complete endpoint
observations use `type: terminal_loss` and `relation: complete_endpoint`.

The evaluator matches types and actual expected source/output positions; it
never matches Latin anchors. Missing spans require at least 80% source coverage
with at most one extra source line. Duplicate contribution endpoints must be
within one measured reader line. Tiny image events use the predeclared IoU
rule. A full-page warning or another lexical occurrence cannot substitute for
a localized hit. Results distinguish hits, misses, pending annotations,
not-requested cases, and reviewed negative regressions. Unmatched events remain
unreviewed. Candidate burden includes every grouped core event; precision and
panel false-positive rate remain unset until their review samples exist.

`run` extracts OCR and nominates duplicate scans, reader order changes, and
missing occurrences. It also compares starts and terminal prose with the same
bounded occurrence aligner. `--detectors boundaries` processes the first and
last two body pages of each selected unit; its coverage lists those pages
explicitly and makes no claim about interior completeness. Use `--detectors
prose` to include interior prose checks. Boundary evidence is saved in
`boundaries.json` and `boundaries.html`, with raw OCR, source/reader crops,
occurrence IDs, alignment operations, retry evidence, and separate start/terminal
results. Complete endpoint observations use `relation: complete_endpoint`;
source-supported terminal-loss nominations require reader retries and remain
low confidence until visual adjudication. OCR disagreement remains inconclusive.

```powershell
python tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope development --detectors boundaries --run-dir tmp/pdfs/inner_tomi_audit/runs/my-development-boundaries
```

Development requests cannot expand into validation units through
`--units` or `--tomus`. Validation/all requests require an explicit complete
development freeze with matching settings, specs, inputs, tool/model versions,
and source-index identity. Metadata inventory and annotation-only gold curation
may cover the corpus before this freeze. Detector dispatch receives unit IDs
and settings; it does not load label content.

`--detectors watermark` searches mapped body images using the reviewed
development source logos. Full logos, Google text, letter groups and shallow
horizontal fragments retain source coordinates and pixel identities in
`watermark-templates.json`. Matching uses grayscale and local contrast,
consistent source/reader glyph-height measurements, and faint-ink support;
scores are normalized correlations. Implicated regions are refined at 300 DPI.
High confidence requires stable views, at least two observed glyphs and a
distinct second fragment or matched source-tail context. Tiny fragments and
failed retries remain visible with their confidence conditions.

`join_body_edges` creates diagnostic bottom/top strips from adjacent images of
the same unit, with source page coordinates for both halves. `merge_logo_events`
preserves every component while combining overlapping and corroborated edge
fragments into a logical event. Virtual stitches add evidence; the original
reader images remain intact. `image-exemptions.json` identifies frontmatter
notices and other frontmatter images from mapped roles and assets, without a
fixed physical-page exemption. Logos touching body prose remain eligible.

`--workers 4` can parallelize pure image searches. PDF inventory, rendering
metadata and evidence extraction stay in the main process. Matching caches bind
rendered pixels, source-template identities, comparison settings, OpenCV version
and helper identity. No validation reader contributes a template or fitted
source-index statistic.

`--detectors head` compares isolated capital fragments and separate source
folios with original header words, occurrence-assigned source transitions and
source image suffixes. Roman chapter markers and multiword prose lines are
excluded. Alternate regional OCR remains available beside the image evidence.
Source folio agreement, suffix similarity, isolation and image correlation are
separate cues; at least three cues and a source transition are needed for high
confidence. A single glyph requires image corroboration. The report preserves
raw OCR and Greek/Latin uncertainty rather than expanding a clipped heading.

`evaluate --freeze` requires a complete final detector run over all development
units/types and their actual body pages, reviewed development cases and
negatives, recorded thresholds/index/tool identities, and a strong localized
development positive for each core class. Manual or partial verification cannot
create a freeze. `--enforce-gates` returns exit 4 for calibration failures; these
are diagnostic checks, not user approval requests. Exit 3 reports incomplete
requested processing. A frozen evaluation is preserved; revisions use new runs.

Exit codes are 0 for completed requested processing, 2 for configuration/tool
errors, 3 for interruption/incomplete processing, and 4 for an explicitly
requested calibration gate failure. A successful capture makes no claim that the
readers are free of defects.
