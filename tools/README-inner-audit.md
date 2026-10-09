# Inner-tomi audit setup

## Regression and evidence verification

There are three verification layers. They answer different questions:

| Layer | What a pass establishes | Inputs |
| --- | --- | --- |
| `cargo test` | Tracked and synthetic regression contracts agree, including the omission inventory, diagnoses, measured repair signatures and completion checks | Git-tracked files and test-created temporary data |
| Named ignored source-PDF tests | Current extraction and composition preserve the reviewed original source pixels and line boundaries | Exact source scans and Poppler; rasters are rendered when caches are absent |
| Seven ignored `audit_evidence_` tests | Historical files match their pinned byte identities, and shared semantic validators agree | Separately supplied frozen archives |

Ordinary tests require no scans, historical archives, API credentials, or rendered
caches. Other optional tests retain their existing prerequisite behavior. The
normal generator still requires the exact source scans and its rendering tools;
a full PDF build additionally requires LuaLaTeX.

From the repository root in PowerShell:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
python -m unittest discover -s tools -p test_check_portable_tests.py
python tools/check_portable_tests.py --offline --log tmp/portable-tests.log
```

The Python tool requires Python 3.10+, Git and Cargo, with no audit/OCR/image
packages. It locates the repository from its own file and can be invoked from
another working directory. `--log` is required and its parent directory is
created automatically. Omit `--offline` on a first run if Cargo needs to fetch
dependencies. New files must be staged: `git ls-files --cached -z` selects paths,
and their current working-tree contents are copied. This is a prospective
tracked-files snapshot, not authentication of a committed revision.

The snapshot excludes `tmp/`, `archive/`, build outputs, source PDFs, rendered
caches and secret data. Child processes receive no inherited API-key or
secret-path variables; the tool gives Cargo an owned target directory outside
the copied checkout and points the evidence root at an empty directory. It checks
the required ordinary names are listed and actually pass in the full suite.
It then runs exactly one historical inventory test and requires a
missing-evidence failure. Unexpected success, zero selected tests and unrelated
failures make the tool fail. Cleanup removes its owned temporary directory;
original scans, archives and caches stay in place. Complete command output and
exit statuses are available in the log.

### Current source-PDF checks

Supply `Ploutarchou_Bioi_v1.pdf` and `Ploutarchou_Bioi_v2.pdf` in the repository
root and make Poppler available. Run these tests individually. The release
profile makes their image processing faster while retaining their assertions:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
cargo test --release --lib facsimile::reflow::tests::romulus_c1_c6_lines_restored -- --ignored --exact --nocapture
cargo test --release --lib facsimile::reflow::tests::audit_tier_a_source_lines_restored -- --ignored --exact --nocapture
cargo test --release --lib facsimile::reflow::tests::comparison_s2_line_is_whole -- --ignored --exact --nocapture
```

Fresh restoration reports and their canvas images go to
`tmp/pdfs/inner_tomi_audit_repairs/restoration-current/step-4.2/`.
The comparison check writes
`tmp/pdfs/inner_tomi_audit_repairs/s2-current/verification.json` and canvas images.
These tests do not require the historical audit archives and do not overwrite
frozen restoration reports.

### Historical archive authentication

All seven tests have the ignore reason
`requires frozen inner-tomi audit evidence archives`. They read and authenticate
historical JSON and PNG bytes, reuse the ordinary semantic validators, and never
invoke the generator, Poppler, LuaLaTeX, a network operation or a paid API call.
They fail with the logical and resolved path when required evidence is missing,
and show expected/actual digests on hash mismatch.

| Test | Authentication |
| --- | --- |
| `audit_evidence_historical_inventory_authenticated` | Candidates, reviews and original audit manifest; full inventory validation |
| `audit_evidence_baseline_diagnosis_authenticated` | Baseline manifest, unique Tier-A traces and reviewed source-overlay PNGs; diagnosis validation |
| `audit_evidence_post_dedup_traces_authenticated` | Post-dedup manifest and traces; line-region and metadata agreement |
| `audit_evidence_post_mask_signatures_authenticated` | Post-mask manifest and traces; all 47 measured source signatures |
| `audit_evidence_restoration_reports_authenticated` | Initial/promoted Tier-A and extra-source-item passing reports; completion validation |
| `audit_evidence_portable_fixture_matches_archives` | All original JSON inputs; exact semantic equality with the tracked projection, including its paths and source digests |
| `audit_evidence_archive_manifests_authenticated` | Every entry in both historical manifests and both stage manifests; hashes, declared sizes and pinned counts/totals |

Set `PLUTARCH_AUDIT_EVIDENCE_ROOT` to a directory **containing** the
repository-relative `archive/` tree. It defaults to `CARGO_MANIFEST_DIR`, so local
archives in the checkout work without an override. For archives stored elsewhere:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
$env:PLUTARCH_AUDIT_EVIDENCE_ROOT = 'D:\Plutarch-evidence'
cargo test --lib audit_evidence_ -- --ignored --list
cargo test --lib audit_evidence_ -- --ignored --nocapture --test-threads=1
```

The list must contain exactly the seven names above. Select these checks
explicitly: the crate also has ignored live API and unrelated expensive tests.

The expected tree beneath that evidence root includes:

```text
archive/
  inner_tomi_audit_2026_10_08/
    manifest.json
    runs/step102-final-all-5/candidates.json
    runs/step102-final-all-5/reviews.json
    ... all 20,361 manifest entries
  inner_tomi_audit_repairs_2026_10_08/
    baseline/
      manifest.json
      omissions-baseline/<unit>/trace.json
      reviewed-source-lines/<event>.png
      ... all 4,404 baseline manifest entries
    post-dedup/
      manifest.json
      <unit>/trace.json
    post-masks/
      manifest.json
      <unit>/trace.json
    restoration/step-4.2/<unit>.json
```

Both stage manifests name 24 trace files. Passing reports cover all required
units, including the report containing the separate Cato Minor `Et:` region.
Historical `files[].path` entries are relative to their manifest's archive
directory; stage-list paths are repository-relative. Absolute `source`,
`destination` and `archive` descriptions inside manifests are retained metadata,
not file locations to open. The reader rejects absolute references, drive
prefixes, parent traversal and resolved escapes through symlinks or junctions.
Duplicate manifest paths are errors; identical digests at different stage paths
are legal. Large-file hashes are streamed.

The archives remain local and Git-ignored and must be supplied separately. No
public retrieval location has been established. Running the latest generator
cannot reconstruct their original bytes. An unavailable historical archive is a
prerequisite failure for these explicitly selected tests; it does not prevent
the ordinary suite from passing.

### Portable fixture maintenance and limits

`src/facsimile/fixtures/inner_tomi_portable_audit_evidence.json` is a compact,
tracked projection with provenance. Its 98 documents retain candidates (including
rejected unique-loss nominations), matching reviews, required trace measurements
and complete restoration report lines, including neighbors and the Cato label.
The initial exported file is 448,689 UTF-8 bytes. It supplements the full ledger
and historical archives. It contains no image payloads or rendered canvases.

Each `source_sha256` identifies the original historical file, not the projected
JSON. The fields and digest metadata are copied historical expectations. A green
ordinary suite confirms those contracts agree; authenticating original JSON/PNG
bytes requires the historical tests. Ordinary results alone are not a new
69-line source-preservation claim or human acceptance of the generated readers.

To refresh the projection deliberately, authenticate the supplied archives and
export to an explicit temporary review file:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
$env:PLUTARCH_AUDIT_EVIDENCE_ROOT = 'D:\Plutarch-evidence'
$env:PLUTARCH_PORTABLE_FIXTURE_OUT = Join-Path (Get-Location) 'tmp/portable-evidence-review.json'
cargo test --lib facsimile::reflow::tests::export_portable_fixture -- --ignored --exact --nocapture
```

The exporter authenticates every original input before using the shared
projection definition. It creates the output parent directory, requires the
explicit output variable, and refuses the tracked fixture as an output target.
It uses the canonical archived candidates/reviews without a mutable run fallback.
Ordinary tests never update the fixture or bless new historical hashes.

Review the temporary JSON semantically: check candidate/review
`miss_romulus_38_2595`, its four reviewed ledger lines and matching restoration
projection; confirm the extra Cato label remains separate. Check provenance,
original roles, measured crop/mask variants, complete cases and fixture size
(target below 512 KiB; hard limit 1 MiB). Run the exporter twice to distinct
temporary files and compare their raw bytes. After review, deliberately copy the
chosen export into the tracked fixture and verify the complete comparison:

```powershell
Copy-Item -LiteralPath tmp/portable-evidence-review.json -Destination src/facsimile/fixtures/inner_tomi_portable_audit_evidence.json
cargo test portable_audit
cargo test portable_evidence_document
cargo test --lib facsimile::reflow::tests::audit_evidence_portable_fixture_matches_archives -- --ignored --exact --nocapture
cargo test --lib audit_evidence_ -- --ignored --nocapture --test-threads=1
```

Keep the ledger's historical hashes, original archives and dated plans/notes.
Any deliberate change to historical expectations requires separate review.
Rust changes in the included audit test file require an explicit
`rustfmt --edition 2021 src/facsimile/audit_repair_tests.rs` pass in addition to
`cargo fmt --all`, since Cargo's formatter does not traverse that `include!` file.

## Diagnostic audit setup

`audit_inner_tomi.py` is an offline diagnostic tool. Its commands check
prerequisites, capture the inspected baseline, render benchmark annotations,
extract OCR, compare source/reader occurrences, boundaries, furniture and
layout, and evaluate localized candidate evidence. Every detector run saves a
static local evidence report, JSON/CSV candidates and coverage/error records.

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
All 187 development annotations have independently measured, inspected regions,
including duplicate contribution endpoints, layout, endpoints and negatives.
All 138 validation cases also have independently inspected source/reader
regions. The 25 prior-mask neighborhoods have separate defect-type negatives
rather than whole-page clean claims. The Lucullus divider annotation records
the independently verified absence of that rule in the current reader; its
copied-gap case remains positive. Aratus duplicate opening boxes bound the
printed first word `a`, while the raw OCR observation `ase` remains preserved.
The first frozen validation evidence and subsequent annotation corrections
are retained separately. Fixed panel image review and candidate adjudication
are recorded separately from these benchmark labels.

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
The semantic settings include the chosen OCR language and the fixed
source/reader/retry DPI and page-segmentation modes. Changing `--lang` cannot
resume an existing run or satisfy a freeze made with another OCR profile.
Body-page keys use `TOMUS:PHYSICAL_PAGE`, such as `1:68`. `producer` distinguishes
manual evaluator verification from detector output. A changed configuration or
input requires a new run. The evaluator validates page ownership using cached
metadata from captured image occurrences, so changing a candidate's unit name
cannot smuggle a validation page into development.

Candidate JSON has `schema_version: 1` and a `candidates` list. Each event needs
an `event_id`, `unit_id`, `type`, machine-readable `relation`, source/output
regions with captured input hashes and normalized boxes, `confidence`,
`confidence_reasons`, `status`, `evidence_provenance`, and existing `crops` with
work-relative paths and SHA-256 hashes. Local omission nominations use
`relation: missing_expected_position`; a localized positive also requires
`unique_loss: true`. Retained or redundant material remains visible as
inconclusive evidence with `unique_loss: false`. Duplicates use
`relation: repeated_occurrence`, two nonoverlapping `contributions` with measured
start/end page boxes, and the incomplete-second-opening assessment. Other
relations are defined in the script's shared `RELATIONS` table. Complete endpoint
observations use `type: terminal_loss` and `relation: complete_endpoint`.

Duplicate boundary refinement retains single-letter Latin prepositions and
short terminal words. When OCR fails to match the terminal word, the locally
corroborated prose line supplies its measured end; an already matched terminal
word keeps its box. Opening-line coverage remains recorded when a later copy
has too little opening prose for a complete edge match. Missing redundant
prefixes therefore remain distinguishable from unique source loss.

The evaluator matches types and actual expected source/output positions; it
never matches Latin anchors. Missing spans require at least 80% source coverage
with at most one extra source line. Duplicate contribution endpoints must be
within one measured reader line. Tiny image events use the predeclared IoU
rule. A full-page warning or another lexical occurrence cannot substitute for
a localized hit. Results distinguish hits, misses, pending annotations,
not-requested cases, and reviewed negative regressions. Unmatched events remain
unreviewed. Candidate burden includes every grouped core event; precision and
panel false-positive rate remain unset until their review samples exist.

Supply independent candidate and fixed-panel adjudications with `--reviews`:

```powershell
python tools/audit_inner_tomi.py evaluate --run-dir tmp/pdfs/inner_tomi_audit/runs/my-development-run --split development --reviews tmp/pdfs/inner_tomi_audit/runs/my-development-run/reviews.json
```

The review JSON contains `schema_version: 1`, `baseline_id`, the actual
`candidates_sha256`, `sample_description`, `events` and `pages`. Describe how
the candidate sample was selected; benchmark hits alone do not establish
candidate precision. Each event record contains `event_id`, `verdict`
(`confirmed`, `false_positive` or `uncertain`), `reason` and `evidence`.
Each page record contains `page_id` in `TOMUS:PHYSICAL_PAGE` form,
`status: reviewed`, `reason` and `evidence`. Evidence is a nonempty list of
objects with work-relative `path` and actual `sha256`, pointing to inspected
files under `cache/` or `runs/`. The evaluator checks these identities.

Pages must belong to the fixed panel and completed scope. Every core event
touching a reviewed page requires an explicit verdict, including low and
unknown events. A page cannot be declared negative while its candidates remain
unreviewed. Per-class precision uses confirmed events divided by confirmed
plus false-positive events; uncertain events are listed separately. Panel
rates count distinct pages with a high-confidence false-positive or uncertain
core event per 100 fully reviewed pages. Candidate-selected precision and
the fixed-page rates describe separate samples and have separate denominators.

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

Expected prose comes from generous original-source regions, with observed
column borders and separate head, footer and neighboring-column observations.
Greek quotations inside the Latin column remain expected text. Scan equivalence
requires exact matched passages and independently observed matching printed
folios; unmatched material remains independently expected. Equivalent context
anchors can bound a local omission without treating a natural repeat elsewhere
as coverage. Bounded alignments between anchors also account for retained text
in verified scan-equivalent reader occurrences. A unique whole-line loss must
still satisfy the whole-line absence fraction after that coverage is accounted
for. The evidence records expected Latin tokens, local seam deletions, retained
tokens in verified occurrences and still-uncovered tokens for each source line.
Redundant local losses remain inconclusive rather than becoming unique omissions.
Omission confidence includes source-raster agreement, informative
contexts, regional reader OCR retries and a localized search for retained
original line pixels. An OCR deletion with visibly retained line pixels stays
inconclusive. These observations never edit production Latin or repair a page.

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

Source-tail corroboration compares original footer logos across nearby raster
scales and rotations, retaining the same high correlation threshold. Completed
matches and negative comparisons are cached against source pixels, template
identities, comparison settings, helper code and OpenCV version. An original
footer match establishes source context; reader glyph support is still required
for a grouped output logo finding.

Grouped logo nominations require at least one glyph supported by refined
reader pixels or a virtual stitch. A logo in the original footer alone cannot
establish its retention in the reader. This check happens after component
grouping, so every weak component of a supported split event remains attached.
Unsupported groups retain all source/output crops, component scores and probe
decisions in `watermark-debug-<unit>.json`, `report-data.json` and the HTML's
uncorroborated-logo-probe section. They are reported separately from accepted
grouped core candidates; low and unknown accepted findings remain in the
candidate JSON/CSV and normal report sections.

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
excluded. An isolated fragment also remains ordinary prose when its ink matches
the registered source image at an expected body-token box. This distinguishes
short final lines from surviving running heads without depending on the
fragment's OCR reading. These exclusions retain their source registration,
projected boxes and body-token evidence in `head-debug-<unit>.json`.
Alternate regional OCR remains available beside the image evidence.
Source folio agreement, suffix similarity, isolation and image correlation are
separate cues; at least three cues and a source transition are needed for high
confidence. A single glyph requires image corroboration. The report preserves
raw OCR and Greek/Latin uncertainty rather than expanding a clipped heading.

`--detectors scraps` checks marginal components against original Greek-column
words. A shared source-image registration maps retained prose to the source
without reading production crop policy; implicated scraps are compared at
300 DPI. At least three corroborated components on two lines are required.
Single reference letters cannot supply Greek-word correspondence. Missing
Greek OCR leaves a qualified image-supported candidate and an explicit reading
limitation. Registration failures and per-page component counts remain in
`scrap-debug-<unit>.json` rather than being treated as clean pages.

`--detectors spacing` inspects long or broken vertical rules, possible
prose-free body images, and gaps of at least three median line pitches.
Full-resolution original line images resolve source-page assignments around
large gaps. Genuine spacing within a printed source page remains negative;
copied-margin candidates retain the two neighboring prose correspondences.
Adjacent-page gaps are considered within a unit, excluding sparse unit endings.
Unreadable ink remains inconclusive after regional OCR rather than becoming a
blank-page assertion. `spacing-debug-<unit>.json` retains inspected interior gaps,
including negative source-spacing comparisons. These measurements support
inspection and do not specify repair masks.

`--detectors clipping` checks original Latin glyph ink at the actual body-image
edges. Lateral candidates compare line beginnings with retained ink and the
white region outside the left boundary. Horizontal candidates require glyph
components crossing a top or bottom boundary; complementary fragments on
adjacent pages of the same unit are grouped. The comparisons use full 300-DPI
images, so the edge of a diagnostic crop cannot establish a slice. The report
retains individual affected-word regions beside overview crops, measured
correlations and unmeasured extent. Adjacent lateral observations join only when they map to
the same physical source page and column boundary, on consecutive physical reader
pages and consecutive body ordinals. Distinct source scans remain separate.
Failed registration or absent source assignments remain explicit in
`clipping-debug-<unit>.json`.

Embedded word boxes serve as locators for clipping comparisons. A small glyph
context includes raster ink just outside those boxes, while the original boxes
remain in evidence. Two-letter body words and Roman chapter markers remain
eligible for pixel-supported slicing checks; their ordinary intact appearance
does not establish a running-head or slicing finding.

`index.html` shows every recorded candidate and endpoint observation, including
low/unknown findings, with source/output crops, physical-page and body-folio
locators, printed source folios when observed, confidence conditions, scores,
related events and raw OCR/provenance. Folio provenance distinguishes printed,
inferred and unknown values. Raw word observations can extend beyond a crop;
their complete boxes remain visible in the evidence details. Scores are
similarities, not probabilities. CSV cells are escaped against spreadsheet
formula interpretation.

`coverage.json` accounts for each requested body page and distinguishes observed
reader OCR from inconclusive OCR on unreadable ink. `errors.json` preserves
processing failures, OCR limitations and inconclusive image registration.
Corpus inventory and completed requested scope are displayed separately.
The evaluator writes `evaluation-<split>.json`; pending benchmark annotations
require independent image review. Reports do not change detector
confidence or convert unreviewed candidates into negatives.

An existing run can regenerate its report and read its original body OCR:

```powershell
python tools/audit_inner_tomi.py report --run-dir tmp/pdfs/inner_tomi_audit/runs/my-development-run
```

Run this after evaluation to show the saved hit/miss counts in the HTML.
Candidate/input identities and evidence crop hashes are checked before report
generation. Re-rendering a report does not rerun detector scoring.

`evaluate --freeze` requires a complete final detector run over all development
units/types and their actual body pages, reviewed development cases and
negatives, recorded thresholds/index/tool identities, complete fixed development
panel review and independent candidate adjudications supplied with `--reviews`.
Each of missing occurrence, duplicate, watermark and head needs a strong
localized high-confidence benchmark positive and an independently confirmed
reviewed event. The final run must have been produced by the current script.
Manual or partial verification cannot
create a freeze. `--enforce-gates` returns exit 4 for calibration failures; these
are diagnostic checks, not user approval requests. Exit 3 reports incomplete
requested processing. A frozen evaluation is preserved; revisions use new runs.

Exit codes are 0 for completed requested processing, 2 for configuration/tool
errors, 3 for interruption/incomplete processing, and 4 for an explicitly
requested calibration gate failure. A successful capture makes no claim that the
readers are free of defects.

Run manifests stay incomplete until reporting, final input verification and
runtime/storage recording succeed. They record elapsed runtime and measured
run/cache storage. A failed invocation saves a timestamped `failure-*.json` in
its diagnostic run directory when that destination is safe. Interrupted runs and their evidence are retained. Resume
uses hash-validated completed cache entries; a stale lock requires checking
that its owner has stopped before removing that specific lock file.
