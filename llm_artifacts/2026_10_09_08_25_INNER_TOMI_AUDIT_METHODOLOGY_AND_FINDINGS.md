# Inner-tomi Python audit: methodology, historical replay, and findings

Written 2026-10-09. This is a retrospective guide to the script designed by
[`2026_10_06_16_05_INNER_TOMI_PYTHON_AUDIT_IMPLEMENTATION_PLAN.md`](2026_10_06_16_05_INNER_TOMI_PYTHON_AUDIT_IMPLEMENTATION_PLAN.md).
It explains the method for an interested reader who need not know text-matching
algorithms, gives commands for inspecting and reproducing the historical work,
and distinguishes the audit's findings from the subsequent repairs.

## What the script does

[`tools/audit_inner_tomi.py`](../tools/audit_inner_tomi.py) asks whether the Latin
in the eight generated inner readers faithfully preserves the selected passages
of the original bilingual scans. An *inner reader* is the PDF in normal reading
order, before pages are rearranged onto sheets for binding. A *unit* is one Life
or comparison section; this audit covers 68 units.

The script is an inspector. It identifies possible omissions, repeated passages,
damaged lines, and unwanted scan material, and saves evidence for review. The
later repair work changes the Rust facsimile generator. The Python audit does
not rewrite Latin, edit the transcription cache, install masks, or repair PDFs.

The central difficulty is that these readers contain pictures of printed Latin,
not a dependable digital transcription. Optical character recognition (OCR)
turns an image into a tentative reading of its words. OCR can misread a faint
letter, split a word, or skip a line that remains perfectly visible. Consequently,
“the OCR did not find it” cannot by itself establish “the reader lost it.” The
method combines passage matching, image comparison, and independent visual
review, preserving disagreements instead of calling them clean pages.

## Methodology, from inputs to reviewed evidence

### Preserve the exact edition and the exact reader build

The audit first captures a fixed set of 21 files: two original source PDFs, eight
reader PDFs, eight corresponding TeX files, the unit map, the existing duplicate
policy trace, and the earlier visual findings report. Copies retain their
original relative paths under `snapshots/baseline_2026_10_06/inputs/`.

Each file has a SHA-256 digest, a fingerprint of its bytes. The script checks the
fingerprints before capture, after copying, and when loading a snapshot. This
answers a practical question: are we inspecting the same pages as the earlier
review, or a later rebuild whose page numbers and defects have changed? A digest
establishes identity, not textual correctness. The snapshot is the historical
audit input; today's `build_facsimile/` is a different build.

Ordinary audit output goes into the working directory's `cache/` and `runs/`
trees. The script rejects output directed into captured snapshots, working
specifications, and protected production inputs. Those are application checks;
the retained archive should still be treated as historical evidence by anyone
using other programs to copy or edit files.

### Establish what each reader page actually contains

The TeX lists the body images in reading order. The script pairs those references
with actual image placements in the compiled PDFs. It counts occurrences, so an
image used twice counts twice. It does not require the original standalone body
PNG files to remain on disk.

This inventory distinguishes body pages from title matter, white separators,
and blank pages added to complete a printing signature. It records both the
physical PDF page and the reader's printed folio. The baseline has 2,912 physical
pages, of which 2,678 contain mapped body images. A blank separator is therefore
not automatically a missing body page.

The source side is built independently. For each unit, the script examines a
broad region of the original Latin column, respecting shared boundary pages and
previously approved duplicate exclusions. It does not adopt the generator's
trimmed crop as its definition of expected Latin: doing that could hide exactly
the line the generator had accidentally removed. Uncertain source boundaries
are retained or marked inconclusive rather than silently narrowed.

### Make tentative readings while keeping the page images authoritative

The script retains words, their page positions, and the OCR provider's confidence.
It keeps embedded source-PDF text observations separate from fresh OCR of the
source raster, because embedded word boxes can be displaced from printed glyphs.
The final profile uses the installed English Tesseract model (`eng`): source
regions at 300 DPI, reader body regions at 180 DPI, and focused retries at 300
DPI. DPI specifies how densely the page is rendered into pixels. Different
page-segmentation settings tell OCR whether to expect a text block, a page, or
scattered text. Their exact settings are preserved with the run.

For comparison, the script keeps two temporary readings alongside the raw OCR.
The strict reading folds case and surrounding whitespace. The tolerant reading
additionally expands ligatures and the long s, and the aligner can consider
words split or joined at line breaks. It does not fold `u/v` or `i/j` by default.
These are aids for recognizing the same photographed passage despite OCR noise;
they never become replacement Latin in a reader or transcription cache.

### Locate passages by distinctive fragments, then check their order

A useful analogy is finding a passage from a quotation with several misread
letters. Common words give weak clues, while distinctive sequences of letters
and words help identify where the quotation belongs.

The script makes an index of overlapping source passages. Its broad windows
contain 64 words and move forward 16 words at a time; local windows contain 16
words and move forward four. Overlap prevents an arbitrary window boundary from
hiding a match. The index describes passages using short runs of three to five
characters. It gives more weight to uncommon fragments than to fragments found
everywhere. This is character n-gram TF-IDF: *n-gram* means a short consecutive
fragment, and TF-IDF is the weighting scheme that emphasizes distinctive evidence.
The vocabulary and weights come from the source alone. Reader pages are queried
against that index; their wording and the benchmark labels do not train it.

The initial similarity measure, cosine similarity, asks how alike two passages'
weighted fragment profiles are. It is useful for choosing plausible locations,
but it cannot by itself prove that words survive in the right order. The script
therefore aligns the words within a bounded neighborhood, allowing measured OCR
differences and line-break joins, and links neighboring matches into a reading
path through the source. A token is simply a word-sized item in this comparison.

This second stage matters because a page can contain the right words in the
wrong arrangement, and the same phrase can appear naturally in several places.
The script measures whether the context is distinctive and whether the best
placement clearly beats a competing placement. Ambiguous placement weakens the
finding. An ordered-similarity score measures how well the sequence lines up;
character edit similarity measures how many letter changes separate two observed
words. None of these scores is the probability that a defect is real.

### Distinguish repeated scans from ordinary repeated language

Two similar phrases are not necessarily duplicate pages. The script seeks long,
distinctive passages at separate source positions and corroborates repeated-scan
identity with source images or folios. It then checks whether the reader visits
that same logical passage twice. Overlapping search windows are merged so that
one repeated block does not become many findings.

The report records both reader contributions, including their first and last
lines. Those boundaries matter: a reader page can contain the end of a duplicate
and the beginning of unique prose. A duplicate report is not permission to
delete every page touched by the warning.

The second scan can have an incomplete opening while the first scan retains the
whole passage. The script records that separately. Likewise, a naturally
repeated phrase elsewhere in a Life cannot excuse a missing line at its expected
position. Only an established scan-equivalent occurrence can supply that line's
retention evidence.

### Test suspected omissions against context and pixels

For a suspected loss, the script looks for the expected source lines between
matched reader contexts. It asks whether the surrounding prose identifies the
location, whether fresh source OCR supports the supposed missing text, and
whether focused reader OCR recovers it under different settings.

It also compares source and reader images near the seam. *Registration* means
aligning the images despite differences in scale, translation, and small rotation.
After registration, the script searches for the source line's pixels in the
reader neighborhood. Finding the line in the image can defeat an omission
warning even when OCR missed it. Failure to register is recorded as an
observational limitation, not proof of loss.

A `unique_loss` finding means the expected line is absent from its reader
position and does not survive through an established equivalent scan. It is
still a candidate until an independent reviewer inspects the evidence. Events
and lines have different counts: one dropped block can be one event containing
12 lines.

### Inspect furniture, damaged glyphs, space, and endings

The image checks address problems that text alignment alone cannot settle:

| Check | Plain-language explanation |
| --- | --- |
| Watermarks | Compare original-source logo fragments with reader pixels at several scales and contrasts. Correlation measures how closely the pixel patterns agree. A logo present in the source footer does not establish that it survived into the reader. |
| Running heads and folios | Combine capital/numeral patterns, isolation from prose, agreement with the source header, and image evidence. A tiny head fragment requires more than a suspicious single letter. Generated reader headings and folios are exempt. |
| Foreign scraps and divider rules | Measure small groups of connected ink pixels and their positions, shapes, and source correspondence. Legitimate reference letters beside Latin are not automatically Greek-column leakage. |
| Clipping and slicing | Compare source glyphs with reader edges and adjacent pages. Lateral clipping cuts into line beginnings or sides; slicing divides a printed line horizontally across pages. A tight crop alone is not a proven damaged glyph. |
| Copied gaps and blank body pages | Measure white intervals and ink while checking source ownership. Preserve genuine chapter/paragraph spacing, complete sparse endings, recto separators, and printing padding. |
| Section boundaries | Compare the first and final source passages with reader boundary pages. A present ending establishes terminal completeness, independently of any omission or duplicate in the interior. |

### Keep detection, review, and repair eligibility separate

Detector confidence is a checklist of evidence conditions. “High” requires the
relevant conditions to pass together; “low” can describe a real defect with weaker
machine evidence; “unknown” preserves unresolved observations. Confidence is
not a reviewer verdict. A reviewer can confirm a low-confidence omission or
reject a high-confidence warning after seeing the line already present.

The benchmark is a separately prepared set of known cases and page regions.
It measures whether the script located the right kind of event in the right
place. A whole-page warning cannot substitute for a measured missing line, and
a watermark warning cannot count as a successful omission detection at the same
seam. The metric intersection-over-union (IoU) measures how much two marked
rectangles overlap relative to their combined area; other case types use
line-coverage or endpoint tolerances. These are localization checks, not Latin
corrections.

The units were split into 34 development and 34 validation units before scoring.
Development is the part used to tune the method; held-out validation is the part
initially kept aside to expose failures on unseen reader material. The script
requires a saved development freeze before processing validation readers. A
freeze records settings, tools, source-index identity, inputs, script bytes,
benchmark identities, and the originating development evidence.

The first independent validation exposed mandatory failures involving duplicate
boundaries and faint Brutus watermarks. General revisions addressed them, and
the original predictions and annotation seal were preserved. The revised
validation and final full-corpus run are **post-validation regression results**:
they demonstrate performance on those preserved cases after revision, not a new
independent validation experiment. The
[`calibration note`](2026_10_08_09_18_INNER_TOMI_VALIDATION_CALIBRATION.md)
documents that history.

There are two separate review samples. Reviewers adjudicated all 94 grouped core
events, which measures how often nominated events were real. They also inspected
a fixed panel of 200 body pages, 25 per tomus, selected before scoring, which
measures high-confidence false alarms on those pages. Choosing the most
interesting candidates would not provide the same page-based evidence.

## Inspect the saved historical reports

Run commands from the repository root in PowerShell. The durable local archive
is `archive/inner_tomi_audit_2026_10_08/`; the authoritative final run is
`runs/step102-final-all-5/`. These large evidence trees are Git-ignored and are
not included by cloning the repository. Supply the original local archive
separately if working on another machine. It contains 20,361 evidence files,
2,144,334,160 bytes, plus its manifest. No off-machine backup is established by
the archive note.

Authenticate the archive and open its existing reports without rerunning OCR:

```powershell
python -X utf8 tools/archive_inner_tomi_audit.py --verify-only
$auditArchive = Join-Path (Get-Location) 'archive/inner_tomi_audit_2026_10_08'
$historicalRun = Join-Path $auditArchive 'runs/step102-final-all-5'
Start-Process -FilePath (Join-Path $historicalRun 'index.html')
Start-Process -FilePath (Join-Path $historicalRun 'boundaries.html')
```

The verifier checks the inventory and every recorded file digest, plus pinned
final candidate/review identities. The expected archive manifest SHA-256 is
`9844828f4aaebaa0eae7498994792ae8ede899adefd73629bbc213c9a864f8b4`.
Use `--verify-only` for an existing archive; the command without that flag copies
evidence from the old working tree and writes an archive manifest.

### A reading order for the report files

| File | What to look for |
| --- | --- |
| `run-manifest.json` | Correct baseline, requested units and detectors, `complete: true`, input/configuration identities, and requested/completed page lists. Completion describes processing, not a defect-free reader. |
| `coverage.json` | One row per requested body page, its unit, completion and OCR status. Distinguish the requested subset from the whole corpus inventory. |
| `errors.json` | Processing failures, unreadable OCR and observational limitations. Zero tool failures does not mean all image comparisons succeeded. |
| `index.html` | Candidate cards, confidence conditions, source/reader locators and images. Expand raw observations and provenance when the pictures or placement seem ambiguous. |
| `candidates.json` | Stable event IDs, type/relation, marked regions, line counts, scores, confidence, contribution boundaries, `unique_loss`, and hashed crop references. |
| `report-data.json` / `candidates.csv` | Convenient locators and raw word observations for browsing or sorting. CSV is a summary; the complete evidence remains in JSON and images. |
| `reviews.json` | Independent `confirmed`, `false_positive`, or `uncertain` verdicts and their evidence. Join to candidates by `event_id`; a candidate's `status: candidate` does not encode its later verdict. |
| `evaluation-all.json` | Benchmark hits/misses, negative checks, reviewed precision, fixed-panel results and candidate burden. Read their denominators before interpreting the numbers. |
| `boundaries.html` / `boundaries.json` | Opening/ending comparisons for each unit, including inconclusive endpoint localization. |
| `*-debug-*.json` / `alignment_debug.json` | Why a nomination was made or rejected; alternative matches, image-registration failures and retained watermark probes. |

Page numbers are one-based physical PDF pages unless explicitly labeled as
folios. `VII/413` means physical page 413 of the historical seventh reader;
`v2/562` means physical page 562 of the second source scan. JSON `source_index`
is zero-based: `0` is v1 and `1` is v2. A box `[x0, y0, x1, y1]` uses fractions
of the rendered full page, with `(0,0)` at the top left. The report additionally
shows observed/inferred printed folios and their provenance. Page numbers from
this baseline must not be used as current-build repair locators.

For a concrete example, search `index.html` for `miss_romulus_38_2595`. It marks
four source lines on v1/38 and the expected seam on historical reader I/68.
`unique_loss: true` distinguishes those lines from an incomplete redundant copy.
The candidate is low confidence because its context-coverage condition failed,
despite other conditions passing. Its independent review confirms the omission;
the earlier human C1 finding later gives it Tier-A repair eligibility. Its
score `0.7188` is a correspondence measurement, not a 71.88% probability of loss.
The later rebuilt reader locator is T1/70, recorded in the repair verification.

“Hit” means a benchmark case was localized with the expected type; “miss” means
that localization requirement was not met. Neither is a count of all corpus
defects. “Not requested” is outside the selected processing/assessment;
“pending annotation” lacks completed independent benchmark regions. A negative
violation is a high-confidence warning where the reviewed benchmark expects
that defect type to be absent. Unreviewed events are not treated as negatives.

## Reproduce detection from the historical artifacts

### Prerequisites and two important dependencies

The detector needs Python 3.10+, the packages in
[`requirements-inner-audit.txt`](../tools/requirements-inner-audit.txt), Poppler
`pdftoppm`, and Tesseract with the `eng` model. The archival verifier uses
`hashlib.file_digest`, so use Python 3.11+ for that helper; the historical
interpreter was 3.13.14. Install missing packages separately, then check:

```powershell
python -m pip install -r tools/requirements-inner-audit.txt
python -X utf8 tools/audit_inner_tomi.py check
```

The frozen run recorded NumPy 2.4.3, Pillow 12.1.1, OpenCV 4.13.0.92,
scikit-learn 1.9.0, RapidFuzz 3.14.3, PyMuPDF 1.27.2, Tesseract
5.5.0.20241111 and Poppler 25.12.0. Frozen detection verifies recorded version
and source-index identities. Changed tools are not automatically an equivalent
reproduction. `run` and `check` accept explicit `--pdftoppm` and `--tesseract`
executable paths when those programs are not on `PATH`. Detection is offline
and requires no Rust build, LuaLaTeX, API key, or paid model call.

First, use the **preserved producer**, not today's working script. The final
freeze pins script SHA-256
`748b8684b23d74c09da84f3c7a9170eaf185ca008f3b7522519ee40b13eb6227`.
The archive retains those exact bytes at
`runs/step101-validation-revised-1/producing-script.py`. The working
`tools/audit_inner_tomi.py` has a different digest as of this note and cannot
satisfy that freeze. Do not weaken the identity checks to make it fit.

Second, the archived revised-development directory contains **the freeze alone**.
The freeze refers to an absolute `development_run_dir` under the original
`tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1/` and authenticates
three files there: `run-manifest.json`, `candidates.json`, and
`evaluation-development.json`. Those three files still exist and match the
freeze on the original checkout at the time of this note, but are not included
in that archived directory. Thus the archive suffices to inspect the final
reports and run a development diagnostic, but **does not by itself suffice for
a frozen all-unit rerun**. Obtain byte-identical copies of these three files
from the original working artifacts when moving the audit to another machine.

### Prepare a separate replay workspace

Keep the archive unchanged. The following setup copies its specs and captured
inputs into a new working directory, saves the historical producer and settings,
and, if available, verifies and copies the three development dependencies. It
creates a replay freeze with only `development_run_dir` relocated. That copy has
a different file digest from the historical freeze; its semantic settings and
authenticated evidence identities remain unchanged. It is a replay convenience,
not a replacement historical seal.

The commands refuse an existing replay directory. On another machine, change
`$env:PLUTARCH_REPLAY_DEVELOPMENT` to the directory containing the supplied three
original development files. The setup verifies their digests before accepting
them. Without those files, the setup still enables the development-only command
below and explains why the frozen full run is unavailable.

```powershell
$env:PLUTARCH_REPLAY_DEVELOPMENT = Join-Path (Get-Location) 'tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1'
@'
import hashlib, json, os, shutil
from pathlib import Path

archive = Path('archive/inner_tomi_audit_2026_10_08').resolve()
work = Path('tmp/pdfs/inner_tomi_audit/replay_methodology_2026_10_09').resolve()
if work.exists():
    raise SystemExit('Replay directory exists; use a new name and matching commands.')
producer = archive / 'runs/step101-validation-revised-1/producing-script.py'
expected = '748b8684b23d74c09da84f3c7a9170eaf185ca008f3b7522519ee40b13eb6227'
def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()
if digest(producer) != expected:
    raise SystemExit('Historical producer digest differs.')
freeze = json.loads((archive / 'runs/step101-development-revised-1/freeze.json').read_text(encoding='utf-8'))
shutil.copytree(archive / 'snapshots', work / 'snapshots')
shutil.copytree(archive / 'spec', work / 'spec')
bootstrap = work / 'runs/replay-bootstrap'
bootstrap.mkdir(parents=True)
shutil.copy2(producer, bootstrap / 'producing-script.py')
(bootstrap / 'settings.json').write_text(json.dumps(freeze['settings'], indent=2) + '\n', encoding='utf-8')
origin = Path(os.environ['PLUTARCH_REPLAY_DEVELOPMENT']).resolve()
dependencies = [('run-manifest.json', 'run_manifest_sha256'),
                ('candidates.json', 'candidates_sha256'),
                ('evaluation-development.json', 'evaluation_sha256')]
if not all((origin / name).is_file() for name, _ in dependencies):
    print('Snapshot ready for development diagnostics; full replay needs the three original development files.')
else:
    for name, key in dependencies:
        if digest(origin / name) != freeze[key]:
            raise SystemExit('Development evidence digest differs: ' + name)
    destination = work / 'runs/step101-development-revised-1'
    destination.mkdir()
    for name, _ in dependencies:
        shutil.copy2(origin / name, destination / name)
    freeze['development_run_dir'] = str(destination)
    (bootstrap / 'freeze-replay.json').write_text(json.dumps(freeze, indent=2) + '\n', encoding='utf-8')
    print('Frozen replay prerequisites copied and verified.')
'@ | python -X utf8 -
if ($LASTEXITCODE -ne 0) { throw 'Replay setup failed.' }
$replayWork = 'tmp/pdfs/inner_tomi_audit/replay_methodology_2026_10_09'
$replayProducer = "$replayWork/runs/replay-bootstrap/producing-script.py"
$replaySettings = "$replayWork/runs/replay-bootstrap/settings.json"
$replayFreeze = "$replayWork/runs/replay-bootstrap/freeze-replay.json"
python -X utf8 $replayProducer check
```

The copied snapshot occupies 1,156,797,006 bytes, about 1.16 GB. Leave additional
room for renders/OCR: the historical shared cache reached about 6.53 GB and the
final run about 330 MB. The archived tree does not include that entire shared
cache. New replay caches can therefore require fresh rendering and OCR.

### A focused diagnostic, then an optional full replay

Start with a development unit if testing setup or if the three freeze dependencies
are unavailable. This inspects Romulus's boundary pages, not its whole interior:

```powershell
python -u -X utf8 $replayProducer run --repo-root . --work-dir $replayWork --baseline baseline_2026_10_06 --scope development --units romulus --detectors boundaries --config $replaySettings --lang eng --workers 1 --run-dir "$replayWork/runs/romulus-boundaries-1"
```

For the historical full scope, the relocated freeze and all its dependencies are
required. This command reads the archived baseline copies, not today's readers:

```powershell
python -u -X utf8 $replayProducer run --repo-root . --work-dir $replayWork --baseline baseline_2026_10_06 --scope all --detectors all --config $replaySettings --freeze $replayFreeze --lang eng --workers 4 --run-dir "$replayWork/runs/all-replay-1"
```

The first historical full run took 6,196.45 seconds, about 103 minutes, with
development renders/OCR already cached. Its resume took 7,657.44 seconds, about
128 minutes: cached OCR does not eliminate image-detection work. These are
historical measurements, not timing promises for a new machine.

To resume, repeat the same invocation and add `--resume`. Keep the baseline,
settings, script and selection unchanged. Changed configurations require a new
run directory. Preserve interrupted evidence; remove a stale lock only after
checking that its owning process has stopped. A success exit code means the
requested processing finished. Codes are `0` completed, `2` configuration/tool
error, `3` interrupted/incomplete, and `4` explicitly enforced calibration
failure. None means “all Latin is correct.”

Evaluate and regenerate reports in the replay tree:

```powershell
python -X utf8 $replayProducer evaluate --repo-root . --work-dir $replayWork --run-dir "$replayWork/runs/all-replay-1" --split all
python -X utf8 $replayProducer report --repo-root . --work-dir $replayWork --run-dir "$replayWork/runs/all-replay-1"
```

Evaluation without `--reviews` supplies benchmark results, not reviewed precision
or fixed-panel verdicts. Historical `reviews.json` binds the exact historical
candidate digest and evidence paths. Do not attach it to a new run by assumption:
transfer verdicts only after verifying identical event regions, contribution
boundaries, and crop digests and recording the new candidate identity. The
historical completion note describes the original checked transfer. Reading the
archived evaluations directly is sufficient when reproducing their recorded
statistics rather than conducting another review.

`report` regenerates presentation, not detector scores, and may recreate OCR
cache entries when needed. Run it in the replay tree. Do not run `report`,
`evaluate`, or `run --resume` against the archived final directory, since those
commands can write diagnostic artifacts there. Do not run a new `capture`
against current readers to recreate `baseline_2026_10_06`: changed input hashes
must be rejected, not substituted.

The setup block was executed for this guide in the separate replay directory.
The preserved producer's prerequisite check passed with the historical package
and external-tool versions. Its snapshot loader verified all 21 input digests,
and its freeze/scope checks accepted the copied specifications, saved settings,
relocated development evidence and all 68 units. Archive authentication verified
all 20,361 files. Full-detection timings and findings below are the preserved
historical measurements; these setup checks do not constitute another complete
detector run.

## Ultimate findings of the historical audit

The authoritative outcome is the frozen `step102-final-all-5` run, together with
its independent candidate reviews. The
[`full-corpus completion note`](2026_10_08_14_44_INNER_TOMI_AUDIT_STEP_102_COMPLETION.md)
records execution and identities; the
[`synthesis findings`](2026_10_08_14_56_INNER_TOMI_AUDIT_SYNTHESIS_FINDINGS.md)
compares this outcome with the earlier visual sweep.

Coverage was complete: 68 units and all 2,678 requested body pages, reader OCR
on 2,678 pages and source OCR on 1,367 pages, zero unreadable OCR pages, and zero
processing failures. There were still 322 explicit observational limitations:
311 inconclusive source-image registrations and 11 clipping inspections without
an assigned source passage. Processing completeness and observation certainty
are different claims.

| Category | Nominated core events | Confirmed | Rejected | Finding |
| --- | ---: | ---: | ---: | --- |
| Missing source occurrences | 77 | 59 | 18 | 113 unique source Latin lines across 31 units; 37 confirmed events high confidence, 22 low confidence. |
| Repeated passages | 9 | 9 | 0 | All D4–D12 blocks corroborated: 20 repeated source pages in nine blocks. |
| Watermark fragments | 3 | 3 | 0 | All W4–W6 body remnants corroborated. |
| Running heads/folios | 5 | 4 | 1 | H1/H2 plus Marius's head suffix and the Cato Major source folio. |
| Total | 94 | 75 | 19 | All core events adjudicated; no pending core verdicts. No order events were adjudicated. |

The largest confirmed omission is the low-confidence Antonius event on v2/562,
historical VII/413, containing 23 lines. Caesar's v2/252 event contains 12 lines;
Theseus's v1/15 event contains seven. The earlier visual sweep had established
four omissions containing eight lines. The audit substantially expanded that
inventory, while confirming all nine repeated-scan blocks and three body
watermarks found visually.

All 68 unit endings were visually confirmed complete. The script localized 26
endpoint benchmark cases and missed localization on 42; those misses are not
42 truncated endings. The complete ending of a Life does not establish that
every interior line is present or appears once.

The file contains 1,379 candidate/endpoint records overall, not 1,379 established
defects. Supplemental screens include 1,110 lateral-clipping nominations, 68
rules, 14 foreign-scrap nominations, eight copied gaps and 32 slicing records.
These are review queues, not exhaustive adjudicated inventories. Human-confirmed
C4/C6/C7 clipping and S1/S2 slicing remain relevant; S1/S2 were localized, while
C7's full extent remained unresolved. Blank-body screening did not establish a
residual furniture-only body-page defect. Intentional front-matter notices,
genuine printed spacing and complete sparse endings remain non-findings.

The core burden was 94 events over 2,678 pages, or 3.510 per 100 pages; this
measures review workload, including false alarms. Candidate-selected precision
was 9/9 for duplicates, 3/3 for watermarks, 4/5 for heads and 59/77 for omissions.
For omissions, about 77% of the nominated events were confirmed; it does not
mean that the method found 77% of every omission in the corpus. That latter
question is recall and would require a complete independent defect inventory.

The fixed 200-page panel had zero high-confidence false-positive pages and zero
high-confidence uncertain pages. This applies to that panel and confidence
category; a confirmed high-confidence head false positive outside the panel
remains in the inventory. It is not a claim of zero false alarms everywhere.
Known-case evaluation reported 51 hits, 65 misses and nine not requested, with
zero pending annotations and negative violations. The misses include exploratory
layout checks and the 42 endpoint-localization limitations.

Re-running with `--resume` produced byte-identical candidates, coverage and
errors. A one-worker versus four-worker comparison on Agesilaus and Agis matched
after normalizing the run-directory path prefix. All 21 live and 21 captured
input digests matched at delivery. These checks establish reproducibility and
input integrity, not corpus-wide completeness or an absence of faint defects.

The final candidate digest is
`03f31fc68aa8829b90cdf3462578d9978b05d01a5a9c895416333dd4f2c78144`;
the independent reviews digest is
`339ac5a1663bf5ea577a293d1d22aaa412b567a082e66806943c110b126b57ec`.

## What happened after the audit

The later
[`repair plan`](2026_10_08_16_00_INNER_TOMI_AUDIT_REPAIRS_PLAN.md)
assigned the 37 confirmed high-confidence omissions, plus independently
human-confirmed low-confidence C1 and C2, to Tier A: 39 events and 69 lines.
The remaining 20 confirmed low-confidence events, containing 44 lines, became
Tier B, requiring a confirming human read of the exact lines and preservation
boundary before promotion. These eligibility tiers did not alter historical
confidence or review verdicts. The 23-line Antonius block remains in Tier B.

The
[`2026-10-09 repair verification`](2026_10_09_00_10_INNER_TOMI_AUDIT_REPAIRS_VERIFICATION.md)
records all 39 Tier-A events and all 69 reviewed lines restored, zero unresolved
Tier-A events, and no Tier-B promotions. A separate Cato Minor verse label and
C6's clipped words were preserved without changing the omission totals. D4–D12
second scans were excluded within their owning units; W4–W6 copied watermark
pixels and tails were removed; the reviewed head/folio cases and S2 whole-line
packing were addressed. Ambiguous F1/F2 scraps and rule neighborhoods without a
proven safe boundary remained deferred, and genuine source spacing was preserved.

The repaired readers compiled offline and contained 2,639 body images across
2,880 physical inner pages. The verification matched the compiled images to
verified canvases, compared all 69 restored source-line regions, and inspected
all 68 terminal reader views. Original source pixels were preserved; Latin was
not retyped, normalized, corrected, or reconstructed. The current locators and
repair evidence are in
`archive/inner_tomi_audit_repairs_2026_10_08/final/` and the verification note,
separate from the frozen pre-repair audit archive.

The audit thus supplies a much larger measured defect inventory than the visual
sweep alone, and the subsequent repair evidence establishes completion of the
finite Tier-A scope. Twenty Tier-B events remain pending the specified human
confirmation. Neither result claims an exhaustive character-by-character
collation of the corpus.
