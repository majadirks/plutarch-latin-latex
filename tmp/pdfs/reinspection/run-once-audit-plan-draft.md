# Inner-tomi Python audit implementation plan

## Context

Write a **run-once, offline Python diagnostic script** for the current facsimile readers. It should nominate likely missing, repeated, or out-of-order Latin, Google watermark remnants, and leaked source heads/folios, with source/output crops for visual review. Additional image checks cover the related layout findings when practical. Calibration and revision use [the completed reinspection report](2026_10_06_15_40_INNER_TOMI_REINSPECTION_FINDINGS.md), including its confirmed defects and non-findings.

This is an inspection tool, not a repair tool. Never modify Latin, transcription caches, source PDFs, generated readers, unit definitions, or production masks/crops. Diagnostic matching may use tolerant text views while preserving original OCR and source images. The user explicitly requires **no unit tests** and does not want a complex software architecture. Verification consists of actual runs, inspected evidence, and comparison against the findings report. This user instruction overrides the repository's general implementation-plan testing guidance for this task.

## Current state of the world

The project is one Rust crate, `plutarch-reader`. The facsimile body consists of original scanned images, not searchable Latin text. Extracting inner I/68 through `pdftotext` returns the generated ROMULUS heading and reader folio52, but no body prose. Source PDFs contain imperfect embedded OCR; the reader images require raster OCR.

| Existing input / contract | Use in this script |
| --- | --- |
| `Ploutarchou_Bioi_v1.pdf`, `Ploutarchou_Bioi_v2.pdf` | Read-only bilingual source scans and embedded OCR. Expected Latin comes from the original columns independently of the generator's retained crop. |
| `build_facsimile/tomus_N_inner.pdf` and `.tex`, N=1..8 | Reader pages and ordered unit/body-image references. Inspect these, not imposed printing PDFs. |
| `facsimile_units.json` | 68 ordered units; source indices0/1, one-based source pages, opening/title fractions, and final source endpoints637/708. |
| `src/facsimile/units.rs::{validate,ranges}` | The next unit's opening page is a shared inclusive tail, cut above its title. It is not necessarily the preceding unit's actual last prose page. |
| `src/facsimile/assemble.rs::{inner_document,unit_block}` | Recto starts, body numbering, ordered `facsimile_<unit>_<ordinal>.png` references, and suppressed folios on unit openings. |
| `src/facsimile/fixtures/inner_tomi_repair_source_trace.json` | Read approved duplicate exclusions from `duplicate_page` records using `source`, `unit_id`, and **`skip_pages`**. Do not read only the representative `source_page`. D1/D2/D3 skip2/2/10 pages respectively. Other ledger masks do not authorize dropping expected Latin. |
| Production crop/band settings and test-only `RetainedSourcePageTrace` | Do not define expected source prose from these. There is no complete production source-to-output pixel manifest to assume. |
| `tmp/pdfs/reinspection/{body_map.csv,endings.csv,endpoint-ledger.json}` | Useful inspected-build reference metadata; generate fresh reader mapping and use these only for verification. |
| Existing `duplicates.cjs`, `gap-map.cjs`, `ocr-all.cjs`, and `map.ps1` diagnostics | Reuse their successful ideas: source similarity, output word matching, Tesseract, and ordered PDF-image/TeX mapping. Their filenames alone do not validate a reusable OCR cache. |

The report records nine additional duplicate blocksD4-D12, four distinct omissionsC1/C2/C3/C5 totaling eight source lines, watermarksW4-W6, headsH1/H2, horizontal slicesS1/S2, lateral clipping, copied margins, and rules. All68 section endings were complete. Several large gaps reproduce genuine printed chapter spacing.

The inspected build has2,912 physical pages and2,678 body-image pages. Physical counts by tomus are416/448/416/128/384/352/448/320; body counts are376/411/395/109/354/331/409/293. Current reader offsets are16 inI,18 inV,8 elsewhere. These are verification values, not future pagination assumptions. Source scans themselves contain duplicates, so matching reader prose to raw source order alone can accept duplicated reading material.

The user confirms that **`python` runs in their PowerShell session**. The agent's PowerShell session has not resolved it; this is an execution-session discrepancy, not evidence that Python is absent from the machine. Start with `Get-Command python -All` and `python --version` in the session actually running the script. Reuse that interpreter or its explicit path. Do not require reinstalling Python or the optional `py` launcher. Poppler and Tesseract are available in the agent environment. No Python code/dependency setup has been executed for this plan.

## Desired end state

Create one script, **`tools/audit_inner_tomi.py`**, and a short dependency file, **`tools/requirements-inner-audit.txt`**. Keep helper functions in that script and share extraction, coordinates, comparison, alignment, image matching, and reporting routines. No package tree, test suite, plugin system, registry, generalized model framework, or new Rust integration is needed.

The script produces JSON/CSV candidate lists, coverage information, evidence crops, and a local HTML review page. It can rerun calibration using saved renders/OCR without repeating all extraction. It never automatically confirms or repairs a defect. It records similarity/evidence scores rather than invented probabilities.

Completion means the core report findings are nominated with localized evidence, negatives are adjudicated, all requested pages/units are accounted for, and the final limitations are stated honestly. A run's successful exit is processing completion, not proof the readers are clean. Implement only the script, dependencies, and necessary diagnostic data; preserve all historical artifacts. The steps below retain their existing phase/step identifiers; Phase1B provides benchmark preparation before detector development.

## Practical implementation contract

### Files, inputs, and output protection

Use the existing Python interpreter after checking compatibility. Initial dependency choices are NumPy, Pillow, `opencv-python-headless`, scikit-learn, RapidFuzz, and PyMuPDF. Choose versions that work on that interpreter and record those actual versions in the dependency file/run metadata. No pytest, Ruff requirement, pip-tools lock infrastructure, automatic installer, or cloud service is needed. An optional virtual environment is a convenience, not a prerequisite imposed on the user.

Keep all generated diagnostic data beneath a work directory:

```text
tmp/pdfs/inner_tomi_audit/
  spec/                     # manually reviewed baseline, labels, splits, templates
  snapshots/baseline_2026_10_06/
    manifest.json
    inputs/                 # captured source PDFs, readers/TeX, units, policy, report
  cache/                    # renders/OCR with input/settings metadata
  runs/<run_name>/           # candidates, crops, coverage, HTML, evaluation
```

The work directory is a container, **not a blanket writable root**. `capture` may create a new snapshot and verify its copies, but may not overwrite a completed one. Ordinary processing writes only beneath `cache/` or `runs/`; it may not change `spec/`, captured inputs, live PDFs, `src/`, `build_facsimile/`, or transcription caches. Resolve paths and existing symlinks/junctions before checking actual writable locations. A read-only snapshot and writable cache/run directories can be siblings; do not reject that legitimate common parent. Add the work directory to `.gitignore` if needed without changing existing rules.

Use SHA256 for input identity. For this single-use tool, a cache file plus small metadata JSON is sufficient: PDF hash, page, MediaBox policy, DPI/crop, OCR version/language/model/PSM, and comparison settings. Reuse an entry only when these match; rerun an incomplete or failed entry. Schedule each cache item once, write completed data through a temporary file, and prevent simultaneous invocations from writing the same run. No elaborate distributed locking system is required. Do not delete earlier runs or old reinspection diagnostics.

`run_name` identifies the baseline/scope/configuration; a changed configuration gets a new run folder. `--resume` resumes that explicit folder after verifying its manifest, rather than guessing the latest run. Hash inputs before/after the run. Exit0 means requested processing completed,2 means configuration/tool error,3 means incomplete processing,4 means an explicit calibration gate failed. Findings themselves are results, not execution errors.

### One PDF adapter and an explicit coordinate policy

Use PyMuPDF for embedded words, image **occurrences**, and their placement rectangles; use Poppler for rasterization and Tesseract for TSV OCR. Do not create competing extraction backends in individual detectors. PyMuPDF calls remain in the main process. Up to4 workers can supervise separate Poppler/Tesseract subprocesses or pure NumPy/OpenCV tasks, but must never share/call PyMuPDF document/page objects from threads. If parallel PDF extraction is later necessary, separate processes must open their own documents ([PyMuPDF concurrency guidance](https://pymupdf.readthedocs.io/en/latest/recipes-multiprocessing.html)). For this run-once script, sequential PDF inventory is the default.

Render full **MediaBox** pages, matching the repository renderer:

```text
pdftoppm -png -f PAGE -l PAGE -r DPI -singlefile INPUT OUTPUT_PREFIX
```

Do not add `-cropbox` implicitly. Save MediaBox/CropBox origins/extents, rotation, actual raster dimensions, and `render_box_policy="mediabox"`. Regional retries crop the full raster; convert TSV boxes back by adding the saved crop offset.

All diagnostic boxes use the displayed MediaBox raster, normalized[0,1], top-left origin. Convert PyMuPDF unrotated points to original PDF coordinates using the inverse of `page.transformation_matrix`. For MediaBox `[mx0,my0,mx1,my1]`, set `u=(x-mx0)/(mx1-mx0)`, `v=(my1-y)/(my1-my0)`; apply clockwise rotation0 `(u,v)`,90 `(1-v,u)`,180 `(1-u,1-v)`,270 `(v,1-u)`. Transform all four box corners. Pixel bounds use floor(left/top), ceil(right/bottom), and actual image dimensions. Record clipping rather than silently changing a region. Unit opening/title fractions already refer to the production MediaBox raster; do not reinterpret them as CropBox fractions. See [PyMuPDF coordinates](https://pymupdf.readthedocs.io/en/latest/page.html).

Verify this adapter by overlaying extracted boxes on actual renders, including a rotated/nonzero-CropBox example if encountered. If those cases are absent from the corpus, do not claim they were exercised; retain an explicit unsupported-coordinate diagnostic rather than guessing. No automated unit-test fixtures are required.

### Small diagnostic data formats

Use ordinary dictionaries/JSON with `schema_version:1`. Validate required fields, IDs, source indices, page bounds, box bounds, and input hashes in one helper; no schema framework is needed.

| Data file / record | Required content |
| --- | --- |
| `spec/baseline-inputs.json` | `baseline_id`, report path/hash, and `inputs` records with role, repository-relative path, SHA256, source index/tomus as applicable; expected page/unit counts and MediaBox policy. Populate from the pinned table below. |
| Snapshot `manifest.json` | Baseline ID, spec hash, verified input hashes, original paths, snapshot-relative paths under `inputs/`, copied report identity, and `capture_complete:true`. Preserve the original relative tree. |
| `spec/labels.json` | Report/baseline identity and cases: `case_id`, report ID, unit ID, defect type, label positive/negative/complete_endpoint, source/output regions, expected relation, annotation status reviewed/pending, and annotation provenance. Missing counts are4/2/1/1; duplicate labels contain both source occurrences and output contribution boundaries. Negative labels are per defect type, not whole-page claims. |
| `spec/splits.json` | Disjoint development/validation unit lists covering68 units, fixed review-panel pages, seed/selection rule, and the two small head-fold definitions. No detected scores belong here. |
| `spec/templates.json` | Development source-PDF hash/page/normalized crop, fragment type, owning unit, and visual provenance. No held-out output image or expected output location may become a template. |
| Page/word/line observations | Input hash, physical page, unit/tomus, normalized box, raw OCR text/provider, optional confidence, and raw-to-comparison offset mapping. Embedded OCR confidence is unknown, not100. Reader/source folios carry printed/inferred/unknown provenance. |
| Expected occurrence | Source unit/physical word interval and unique occurrence ID; only explicitly established scan-duplicate intervals share a logical occurrence ID. Natural repeated wording stays separate. |
| Candidate | Type, stable event ID, source/output regions and occurrence IDs, raw OCR excerpts, primary/component scores, confidence high/low/unknown with reasons, status candidate/inconclusive, linked events, locators/provenance, and crop paths. |
| Run/evaluation JSON | Inputs/settings/versions/scope, requested and completed coverage, errors, candidates, case hits/misses, reviewed TP/FP/uncertain counts, actual denominators, and freeze/revision identity. |

`capture --baseline-spec PATH` reads that explicit file. Runtime `--baseline baseline_2026_10_06` resolves **only** `WORK_DIR/snapshots/baseline_2026_10_06/manifest.json`; accept IDs matching `[A-Za-z0-9_]+`, reject traversal, and never fall back to live PDFs. `--run-dir` is an explicit run directory, not a timestamp lookup. Source/profile geometry overrides, if needed, belong in manually reviewed spec data with source identity and provenance; they must not encode known missing reader passages. Ordinary detector functions do not read labels or finding anchors.

### Commands and processing stages

The following are implementation targets, not commands executed while writing the plan. Replace quoted run-directory examples with actual folders created by the script. Dependency installation is done separately if imports are missing; runtime is offline.

```powershell
Get-Command python -All
python --version
# If this session needs an explicit executable, use its actual path instead.
python -m pip install -r tools/requirements-inner-audit.txt
python tools/audit_inner_tomi.py capture --repo-root . --work-dir tmp/pdfs/inner_tomi_audit --baseline-spec tmp/pdfs/inner_tomi_audit/spec/baseline-inputs.json
python tools/audit_inner_tomi.py annotate --repo-root . --baseline baseline_2026_10_06 --labels tmp/pdfs/inner_tomi_audit/spec/labels.json
python tools/audit_inner_tomi.py run --repo-root . --baseline baseline_2026_10_06 --scope development --detectors core --run-dir tmp/pdfs/inner_tomi_audit/runs/development-core --resume
python tools/audit_inner_tomi.py evaluate --run-dir tmp/pdfs/inner_tomi_audit/runs/development-core --split development
# After development calibration with all requested detectors:
python tools/audit_inner_tomi.py evaluate --run-dir tmp/pdfs/inner_tomi_audit/runs/development-final --split development --freeze
python tools/audit_inner_tomi.py run --repo-root . --baseline baseline_2026_10_06 --scope validation --detectors all --freeze tmp/pdfs/inner_tomi_audit/runs/development-final/freeze.json --run-dir tmp/pdfs/inner_tomi_audit/runs/validation-first
python tools/audit_inner_tomi.py evaluate --run-dir tmp/pdfs/inner_tomi_audit/runs/validation-first --split validation --enforce-gates
python tools/audit_inner_tomi.py run --repo-root . --baseline baseline_2026_10_06 --scope all --detectors all --freeze tmp/pdfs/inner_tomi_audit/runs/development-final/freeze.json --run-dir tmp/pdfs/inner_tomi_audit/runs/all-final --resume
```

Expose only practical options: repository/work directories, baseline/spec, explicit run directory, scope, unit/tomus subset, detectors core/prose/furniture/layout/all, worker count, config file, external-tool paths, and resume. `annotate` renders supplied gold regions/contact sheets and drafts annotation data in its own run folder; it does not run detectors or edit spec files. The implementer deliberately updates reviewed specs.

Implement a minimal evaluator before detector calibration. Early evaluations consider only the units/detectors requested in that run and list other cases as not requested. A development freeze requires the final development run to cover all development cases for the requested final detector scope, with reviewed negatives and recorded thresholds. Validation/all processing requires this freeze. First validation uses frozen thresholds/templates/reference index settings; later changes are recorded as post-validation revisions in new run folders. Metadata inventory and source-reference extraction can cover the whole corpus early; validation reader OCR/alignment/image detections cannot. Annotation-only gold curation does not count as detector validation feedback. Labels are publicly based on the report, so describe this as controlled validation rather than a blind experiment.

### Evidence, confidence, and acceptance metrics

Similarity scores are not probabilities. Separate broad nomination from high-confidence evidence. Store every nomination with its failed/satisfied confidence conditions; never downgrade all candidates to make negatives pass.

| Type | Initial high-confidence requirements, configurable before freeze |
| --- | --- |
| Missing occurrence | At least6 unmatched tokens or a whole source line; both contexts have>=8 informative matched tokens and similarity>=0.85; competing-placement margin>=0.10; source embedded/raster agreement>=0.80; output regional retry preserves absence at that expected position. Another genuine occurrence cannot supply its coverage. One-sided cases remain low without image corroboration. |
| Duplicate/order | Similarity>=0.90 and cosine>=0.80 across>=40 informative tokens, actual nonoverlapping contributions, and repeated folio or source-image corroboration. Record a logical repeated visit/reversal, not overlapping-window artifacts. |
| Watermark | Correlation>=0.80 on a nonconstant template with>=2 glyph-widths of support, stable location across grayscale/contrast views, and a second distinctive fragment or matched source-tail corroboration. A tiny isolated arc remains low. |
| Head/folio | Outside generated-text exemptions and at least3 cues: capital/number pattern, isolation, source-head suffix match>=0.80, exact source-folio agreement, or matched source-head image component. A single-character head requires image/source corroboration. |
| Gap/rule/scraps | A gap needs>=3 line pitches and source correspondence>=0.85 plus copied-margin/excess-spacing evidence. A rule needs>=4 line heights, width<=0.3 line height, and collinear broken segments. Scraps need>=3 marginal Greek-like components on>=2 lines with Greek-column correspondence; reference letters alone fail. |
| Slicing/clipping/blank/end loss | Full300-DPI source/output comparison must corroborate slicing or lateral loss; lateral high confidence needs>=3 affected line beginnings. A blank-body claim needs mapped body status, no matched prose after retry, and no unexplained text-like ink. End loss needs terminal-source evidence and failed expected-position alignment after retry; punctuation alone is insufficient. |

Pattern/isolation cues have fixed initial definitions:1-20 Latin/Greek capitals or2-4 digits;<=3 isolated OCR words apart from prose by>=0.5 median line pitch. Source-observation agreement is the fraction of tokens matched at character similarity>=0.80 in an ordered comparison. A crop/source match is an image-matcher result, not a visual assertion fabricated by the script. Missing evidence or contradictory observations yields unknown/inconclusive. Confidence predicates and thresholds are frozen with the configuration before validation. At least one strong development positive per core detector must remain high, preventing the all-low workaround.

Within a type, rank omissions by mean context similarity, duplicates by mean(cosine,ordered similarity), logos by valid correlation, heads by fraction of satisfied cues, and layout events by normalized observed support. Sort by defect severity, confidence high/low/unknown, primary score, then stable event ID. Link related findings at one seam rather than substituting a gap/logo warning for an omission.

Measure **candidate burden** as grouped output core events per100 processed body pages, counting candidates whether reviewed or not. Measure **reviewed precision** as `TP/(TP+FP)` over a clearly identified reviewed sample; uncertain events are a separate count. For a **false-positive-per-page** figure, use the fixed fully reviewed page panel below, not top-ranked candidates as a substitute denominator. Count each grouped high-confidence core event intersecting that panel once, and divide FP by the actual number of fully reviewed panel body pages. Top50 and low-score samples can supplement precision review but cannot define a representative page rate. No metric treats unreviewed events as negative or extrapolates a small benchmark to the full corpus.

## Frozen baseline and report-derived cases

### Input identities

Populate `spec/baseline-inputs.json` from these SHA256 values. The report path is `llm_artifacts/2026_10_06_15_40_INNER_TOMI_REINSPECTION_FINDINGS.md`. If a live file has changed, locate a byte-identical preserved copy; do not silently substitute a rebuilt reader. The snapshot captures copies before any later repairs/rebuilds. A different build needs independently revised labels and is outside this run-once baseline's acceptance claim.

| Input | SHA256 |
| --- | --- |
| `Ploutarchou_Bioi_v1.pdf` | `e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15` |
| `Ploutarchou_Bioi_v2.pdf` | `fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684` |
| `facsimile_units.json` | `7098d39e6c1a83c0de4333e17a09bbebed59c2bf19a33b0339d79cfe74e11949` |
| `src/facsimile/fixtures/inner_tomi_repair_source_trace.json` | `17fd5fbe3cb0f1d5b94d454bd7fa56181f782b98d4ce0fba42fd24d77c1a8b43` |
| Findings report | `5ac02a65ce32869c42f0638b5d63980ba85cfd32d9d8feeef3371d1ccc1e0474` |
| `build_facsimile/tomus_1_inner.pdf` | `c30fe260631bf2b32101d95cd18157a2eed6920d8e130355d6329c12cf89f5c2` |
| `build_facsimile/tomus_2_inner.pdf` | `30f7af8dcda3f95095ad4329390a49278de4ada35b7a6c7cb640dc223df96595` |
| `build_facsimile/tomus_3_inner.pdf` | `9b65a94b8ce89d0182b37831123bec6e4a3fe38e9c25becedd37b3853ae0d58c` |
| `build_facsimile/tomus_4_inner.pdf` | `a571fdef1fae482979bc399c01e57255273b681a36ec0fa8dabc662fd08c254a` |
| `build_facsimile/tomus_5_inner.pdf` | `0ad7342398a96cf7a937aff1c5cb7c9d0f22e3c7417dc1b4ffaf4698d6ae5e06` |
| `build_facsimile/tomus_6_inner.pdf` | `7f0b74c72d25bf800c265bcd94b0e3da885dc33b65bba8513fe3bdd0f918ee66` |
| `build_facsimile/tomus_7_inner.pdf` | `4430eff76cc7d7790a2656a3e4948f3914dacc69f94dd629fec1b5d9268ad7eb` |
| `build_facsimile/tomus_8_inner.pdf` | `2f571a1a74af15c9a9b22bfc0e0625c1dc735679a1fb8b4f014c11bac03b7c12` |
| `build_facsimile/tomus_1_inner.tex` | `83c5e127b3dbe8e1c56bc5ac8a157ab6e28b13ec3bf68e22c0556cee8e272116` |
| `build_facsimile/tomus_2_inner.tex` | `f6ce4f7148d226b123dd6b2bfdd402463ad4cfe027d2df5b9b31d8bcdffc348e` |
| `build_facsimile/tomus_3_inner.tex` | `c46d0909b2d462133a53ac0b1c44bcf231abdd8ededa36602e0d5a2fcdf34b30` |
| `build_facsimile/tomus_4_inner.tex` | `c83c15bd6583facac5d0e6625ed9f2c1c430c96b412c153aab3c35d4b86e2ea3` |
| `build_facsimile/tomus_5_inner.tex` | `8f9b4f3f751f3760aae32a68d62e5846e1746fe40f3d23caab96042640df2ba3` |
| `build_facsimile/tomus_6_inner.tex` | `e58e5a2455e179074b05ecb4382e5cdce044416939939e9c54b8b0a2e96548d0` |
| `build_facsimile/tomus_7_inner.tex` | `48941e64c5abec004c040c772f8329de7e4a3495d6221a05dd65db88d7774e0e` |
| `build_facsimile/tomus_8_inner.tex` | `97f10093242a92c5c7980fad09b3d2d45ae0af38d3d35dcc8d4a3f233e7016bd` |

### Cases and splits

Create reviewed regions from the **final report**, not tentative checkpoints. Copy short Latin anchors faithfully as diagnostic provenance; never replace cached transcription with them. Gold regions are manually measured on raw crops, not generated from detector predictions. Pending regions remain pending. A full-page warning cannot count as a localized finding.

| Cases | Required assessment |
| --- | --- |
| D4-D12 | Nine blocks; both source occurrences and both output contributions, using the report's corrected ranges. D4 has4 repeated source pages; the others2 each. Shared output boundary pages are contribution ranges, not deletion instructions. |
| D5/D6/D12 incomplete second openings | Distinguish redundant-copy omissions from unique loss; first copies retain those openings. |
| C1/C2/C3/C5 | Four unique losses of4/2/1/1 source lines. C5 is Brutus VIII/128 r120, sourcev2/624 f1201; do not replace it with a W6/gap warning. |
| W4-W6 | Three body watermark events; W4 spansII/273-274, whileW5/W6 are faint Brutus remnants. |
| H1/H2 | Philopoemen III/123 head/435 and Agis VII/29 single GreekΣ fragment. H2 cannot require a two-character minimum. |
| C4/C6/C7, F1/F2, S1/S2 | Lateral clipping, Greek scraps, and horizontal slicing have distinct labels. C7's total extent remains unconfirmed. |
| M1-M11, R1-R13 | Representative gap/rule regions. Source chapter spacing is a negative; rules can remain positive at those same pages. |
| E01-E68 | Complete terminal prose correspondence, independently of interior losses/duplicates. |
| Required negatives | Genuine-spacing table; D1-D3 repaired joins; W1/W2 repaired logos; all25-mask neighborhoods; generated headings/folios, chapters/reference letters; whiteIII/298; sparse endings; intentional notices; repaired O8B; Otho709-710 outside selected body. |

Mandatory development unit IDs: `agesilaus`, `alexander`, `phocion`, `cato_minor`, `agis`, `romulus`, `marcius_coriolanus`, `aemilius_paulus`, `timoleon`, `philopoemen`. Mandatory validation IDs: `cicero`, `antonius`, `aratus`, `otho`, `brutus`, `comparatio_demosthenis_et_ciceronis`. Assign remaining units by the first byte of SHA256(`audit-split-v1|baseline_2026_10_06|UNIT_ID`): even=development, odd=validation. Persist the lists before scoring. A case inherits its unit's split; do not reassign failures.

Develop watermark matching onW4 and source-logo templatesv1/287,299,321. Keep BrutusW5/W6/C5/M10/M11/R13 together for validation. S1 is development; S2 is validation. H1/H2 allow two small leave-one-head-unit-out sensitivity runs, explicitly acknowledging that these units participate in other development work; these are not independent validation experiments.

Before Phase10, baseline detector runs and matching checks use only development units. Metadata counts across all68 units and the complete original source-reference corpus are permitted early. All68 endpoint matching, allM/R checks, and held-out output assertions are final checks after freeze, not earlier passing requirements. Source-only reference data may contain validation prose; reader output must not fit vocabulary/IDF or affect pre-freeze detector tuning.

Select the review panel before scoring: within each tomus, sort actual body pages by SHA256(`audit-panel-v1|baseline_2026_10_06|TOMUS|PHYSICAL_PAGE`) and keep the first25. Persist200 pages with inherited unit splits. Review the development subset during tuning and the remainder after validation opens. Do not choose panel pages from candidate ranks or replace inconvenient pages.

## Phase 1 - Environment and read-only capture - Suggested implementer: Sonnet

### 1.1 Establish the single script and interpreter

Create `tools/audit_inner_tomi.py` with simple argument dispatch and shared helpers, plus `tools/requirements-inner-audit.txt`. Validate the user's working interpreter/imports and Poppler/Tesseract paths/models. Use argument-list subprocess calls with `shell=False`, timeouts, and hidden Windows subprocesses. No automatic downloads inside the script. Print actionable errors and progress at least every completed batch/30seconds. Do not start a Rust build.

**LLM verification:** Run `python --version`, import required packages, and invoke the script's help/prerequisite check. Record actual versions and any session-specific path problem; do not claim missing machine-wide Python. Check syntax with `python -m py_compile tools/audit_inner_tomi.py`.

**Human verification:** Read prerequisite output and commands; confirm the existing Python is reused and the tool offers diagnostics rather than repairs.

### 1.2 Capture the inspected inputs and check write boundaries

Populate baseline-spec data from the pinned hashes; implement `hash_file`, `resolve_baseline`, `safe_write_path`, and `capture_inputs`. Copy inputs/report into a new snapshot, verify hashes, and save the manifest. Keep original relative paths. Ordinary run writes use cache/run siblings and cannot enter snapshot/spec/live input paths. Reuse matching completed diagnostic entries; no package/cache framework is needed.

**LLM verification:** Run `capture`, inspect every copy/hash, then create a harmless diagnostic file in a sibling run folder. Verify the same helper rejects a proposed snapshot/live-input destination without attempting to overwrite it. Rehash originals and check exact ID resolution.

**Human verification:** Inspect snapshot/run paths and the manifest. A snapshot can be read by a normal run despite sharing the work-directory parent; no copied input is treated as writable output.

## Phase 1B - Benchmark preparation before detectors - Suggested implementer: Opus

### 1B.1 Prepare gold cases, splits, and crops

Implement minimal `render_page` and `annotate_regions` helpers now; later stages reuse them. Populate small baseline/labels/splits/template-spec JSONs with required fields. Measure independent source/output gold regions from the final report and raw images. Persist unit splits and the200-page panel before tuning. Gold curation may view report/raw validation evidence but cannot use validation detector scores or predicted boxes.

**LLM verification:** Render a source and reader gold crop, inspect normalized boxes, and check the case inventory covers every required positive/negative group and68 endings. Verify development/validation unit sets are disjoint and cover68 units, panel counts are25 per tomus, and pending annotations are explicit.

**Human verification:** Compare labeled crops with the report, especially shared duplicate boundary pages and C5 versusW6. Confirm negatives are per defect type.

### 1B.2 Make simple evaluation available before calibration

Implement `evaluate_cases` and scope checks before the detector stages that invoke them. Match supplied candidate JSON to reviewed regions/types/relations using the localization rules in9.2. Early evaluations cover requested development detectors/units; list others as not requested. Detector functions receive only selected unit IDs/settings, never labels or finding anchors. Validation runs require a frozen development configuration; direct unit requests cannot expand protected scope silently.

**LLM verification:** Evaluate a manually prepared development candidate record with its real reviewed evidence, then a deliberately overbroad/page-only record; only the localized correct-type event counts. Check that a premature Brutus detector request is refused, while metadata inventory/gold annotation remains possible. No synthetic unit-test suite is required.

**Human verification:** Read the evaluation's hit/miss/not-requested entries and the protected-scope explanation. Confirm configuration freezes are diagnostics, not user approval gates.

## Phase 2 - Reader inventory and independent source prose - Suggested implementer: Opus

### 2.1 Map actual reader images to units and folios

Implement `parse_tex_refs`, `enumerate_image_occurrences`, and `map_reader_pages`. Match actual ordered TeX image references to PDF image occurrences, not unique image objects; ignore comments/macro definitions and soft masks. Body PNG files may be absent. Check counts and unit/title transitions; mark ambiguity/incomplete inventory rather than guessing. Record body image placements, frontmatter, white separators, and padding. Read printed reader folios from embedded generated footer text; infer suppressed openings from consistent adjacent numbering with provenance.

**LLM verification:** Compare fresh counts to2,678 body pages/68 units and each tomus's report counts. Inspect first/last body pages and V numbering. Check placements against the existing `body_map.csv` without treating that file as live detector truth.

**Human verification:** View mapped openings/endings and distinguish physical PDF pages from reader folios, including suppressed folios and V's18-page baseline offset.

### 2.2 Extract expected Latin independently of production crop loss

Implement `partition_source_units`, `read_approved_exclusions`, and `extract_source_regions`. Honor shared-tail/title/open-body fractions and final endpoints. Read complete unit-scoped `skip_pages` lists. Start with a broad right-column/gutter regionx0=0.44,x1=1.0, then use source-word/image-line consensus and expand by character-width margin. Preserve uncertain top/bottom/left Latin rather than copying production trimmed bands. Keep source head/footer observations separately. Ambiguity widens the region/reduces confidence, not silently drops text.

**LLM verification:** Inspect development source pagesv1/38,292 andv2/299: omitted reader lines must still occur in expected-source observations. Check all14 approved skipped pages and shared-page ownership. Defer C5 reader correspondence to validation; complete source-reference extraction itself is allowed.

**Human verification:** Compare expected columns to the original scans, especially chapterIX and first source lines. Verify that source furniture removal has not eaten expected Latin.

## Phase 3 - OCR and tolerant comparison - Suggested implementer: Sonnet

### 3.1 Collect coordinates and OCR observations

Use the shared PDF/MediaBox adapter and render helper; source300 DPI, output180 DPI, candidate-region retries300 DPI. Tesseract TSV preserves boxes/confidence; begin source-column PSM6 and output-page PSM3, with regional alternate-PSM retries. Use installed English OCR initially; compare Latin/Greek models only if available and explicitly recorded. Preserve embedded and raster source observations separately. Blank OCR on inked body is unreadable/inconclusive, not clean. Reuse caches only with matching metadata, not old plaintext filenames alone. Keep PDF calls sequential and external/image tasks bounded.

**LLM verification:** Run development OCR and overlay words/lines on source/reader images. Check MediaBox origins/rotation, regional offsets, raw confidence, and source/output correspondence. Inspect retry/failure logs and interrupted-run resume behavior. Record unexercised unusual-coordinate cases honestly.

**Human verification:** Check boxes surround visible words and Greek/head recognition limitations are disclosed. OCR is visibly labeled diagnostic, not authoritative Latin.

### 3.2 Define source-only similarity indices and anchor information

Implement `comparison_views`, `passage_windows`, `fit_source_index`, and `anchor_information`. Preserve raw text and word-offset maps. Strict view folds case/whitespace; tolerant matching can expandæ/œ and long-s, retaining alternate line-hyphen joins. Do not normalize output Latin or foldu/v/i/j by default. Keep short words in alignment.

Use source-window character TF-IDF3-5 grams, token2-4-shingle overlap, and ordered fuzzy matching. Initial long windows are64 tokens/stride16; local windows16/stride4; include trailing tokens. Fit strict/tolerant vectorizers only on source-reference windows: `analyzer="char"`, `ngram_range=(3,5)`, `lowercase=False`, `min_df=1`, `max_df=1.0`, `sublinear_tf=True`, `smooth_idf=True`, `norm="l2"`. Reader windows are transform-only. Persist vocabulary/IDF/source hashes/settings; no validation-reader statistics or labels affect fit. See [TF-IDF options](https://scikit-learn.org/stable/modules/generated/sklearn.feature_extraction.text.TfidfVectorizer.html).

A token is informative if it belongs to a2-4-token shingle found at<=10% of distinct source positions in that unit, with a minimum allowance2. Anchor uniqueness is1 divided by the number of distinct context-correct source positions scoring>=0.80; overlapping windows of one interval count once. Collapse only established duplicate aliases, never natural repeated wording. Placement margin is best minus second-best distinct position score; no competing position gives margin1.0.

**LLM verification:** Inspect raw/strict/tolerant views for real ligatures and split words. Compare the saved source index before/after output transformation: it must not refit/change. Inspect informative contexts and ambiguity on repeated phrasing.

**Human verification:** Confirm evidence displays raw OCR/pixels and explains tolerant matches without suggesting spelling changes.

## Phase 4 - Duplicate scans and reader order - Suggested implementer: Opus

### 4.1 Detect source duplicate blocks

Implement `find_source_duplicates` and `merge_duplicate_blocks`. Compare distinct source passages within each unit, including adjacent and distant pages. Exclude only identical/overlapping self-windows of the same physical passage, before retrieving top8 distinct candidate positions. Start nomination at cosine>=0.55, ordered similarity>=0.75, and40 informative matched tokens (or60% of tokens for a unit shorter than40). Group consecutive matching pages; retain physical/printed folios.

Established scan equivalence needs high duplicate evidence, distinct physical scans, and matching folio or independent source-image corroboration. Alias only matched intervals; unmatched material remains expected independently. Uncertain equivalence can nominate a duplicate but cannot excuse missing coverage. Natural identical wording at genuinely different printed positions is not scan equivalence.

**LLM verification:** Evaluate developmentD4-D8 using the already available evaluator. Inspect source-pair scores, folios, grouped ranges, adjacent-page eligibility, and incomplete second openings. Revise retrieval/grouping when a known block is missed; do not add its page number to scoring logic.

**Human verification:** Read paired scans and chapter progression. Confirm one block event per repeated sequence and no whole-output-page deletion recommendation.

### 4.2 Match multiple reader occurrences and preserve restarts

Implement `find_reader_occurrences`, `chain_matches`, and `assign_occurrence_coverage`, using one shared `align_tokens` helper. Begin its exact-token mode here and extend that same helper in5.1. Every natural source occurrence has a separate ID/count1. Approved skipped scans count0. Established duplicate aliases share one logical expectation, but extra reader visits still generate repetition events.

Initial deterministic chain rules:

- Refine retrieval hits to maximal matched anchors of>=8 tokens with source/output intervals and cosineC/ordered scoreO. Deduplicate overlapping retrieval hits for the same position, preferring reward `min(source_length,output_length)*(0.5*C+0.5*O)`, then longer span, physical offsets, and ID. Distinct occurrences survive.
- Process increasing reader positions; a path cannot consume the same reader token twice. Forward logical-source edges cost `0.04*min(source_gap,400)+0.02*min(output_gap,400)`.
- Allow backward/repeated logical visits with restart cost `2.0+0.5*min(backward_tokens/64,4)+0.02*min(output_gap,400)`. Record restart/order events explicitly. Established aliases use canonical first-occurrence positions for order while retaining actual source regions.
- Keep8 alternative paths per reader-start group, maximizing anchor rewards minus edge costs. Ties prefer fewer ambiguous equivalences, fewer skipped source tokens, then lexicographic anchor IDs. Record unmatched starts/tails and competing occurrence assignments.
- One reader interval can cover only one independent source occurrence. Multiple raw scan aliases may refer to that same logical occurrence; unrelated repeated phrases may not. Uncertain equivalence stays ambiguous rather than forcing clean coverage.

All costs/settings are saved and may be calibrated on development data. A source line missing only from a redundant second scan does not become unique loss when the first supplies it. Repeated reader material can also arise without duplicate source scans; repeated visits to a single source occurrence must still be nominated.

**LLM verification:** Inspect development reset/boundary crops and `alignment_debug.json`. Verify both reader occurrences are retained, shared contribution pages are represented, and D1-D3 approved exclusions do not create missing-prose warnings. Review naturally repeated contexts for separate assignments.

**Human verification:** Compare reset starts/ends with the source and distinguish a surviving later repeat from a successfully removed earlier block.

## Phase 5 - Missing occurrences and endings - Suggested implementer: Opus

### 5.1 Refine local alignment and nominate omissions

Extend the same `align_tokens` helper with a bounded custom token dynamic program. RapidFuzz supplies normalized character Levenshtein similarity, not the custom token/join alignment itself ([distance API](https://rapidfuzz.github.io/RapidFuzz/Usage/distance/Levenshtein.html)). For source/output token lengthsm/n, initialize `D[0,j]=j`, `D[i,0]=i`. Deletion/insertion cost1.0; substitution is0 for strict equality,0.10 for tolerant equality, otherwise `min(1.8,2*(1-sim))`, with sim=max(strict,tolerant character similarity).

```text
D[i,j] = min(
  D[i-1,j] + 1.0,
  D[i,j-1] + 1.0,
  D[i-1,j-1] + substitution(source[i-1],output[j-1]),
  D[i-2,j-1] + 0.15 + substitution(join(source[i-2:i]),output[j-1]),
  D[i-1,j-2] + 0.15 + substitution(source[i-1],join(output[j-2:j]))
)
```

Join/split transitions exist only for observed discretionary line-hyphen/split hypotheses, not arbitrary adjacent words. Coverage requires strict/tolerant equality or fuzzy similarity>=0.75; a cheaper low-similarity substitution remains unmatched. Ties prefer strict diagonal, tolerant diagonal, permitted join/split, fuzzy diagonal, source deletion, output insertion, then original word IDs. Ordered similarity is `max(0,1-cost/max(m,n,1))`. Preserve operations and boxes. Cap local matrices at400x400; split with additional anchors or mark a localized span inconclusive, never truncate it silently. Reconcile local results so different gaps cannot consume one reader span twice.

Nominate uncovered expected occurrences with matched surrounding context and source text/ink evidence. Initial bracket nomination:>=8 informative tokens each side, ordered score>=0.80, placement margin>=0.10; weaker localized cases remain low/unknown. Search the whole reader unit for the same phrase, but classify hits by expected occurrence, context, order, and already-consumed tokens. **Another natural occurrence does not satisfy this one.** A displaced instance creates order/missing-expected-position evidence, not a claim that its text is absent everywhere. Only a context-correct match or established same-logical-occurrence alias supplies coverage.

Retry implicated source/reader regions at300 DPI/alternate PSM. Preserve conflicting observations as inconclusive. A one-sided unit-boundary loss can still be nominated with lower confidence. Count unique missing lines separately from displaced or redundant-copy losses. Never replace a missing-line candidate with a gap/logo warning.

**LLM verification:** EvaluateC1/C2/C3 development omissions with source unmatched regions and neighboring reader crops. Inspect alignment operations and whether repeat/elsewhere matches consume the correct occurrence. Check OCR retries before increasing confidence. Record exact misses and causes; revise methods rather than labels.

**Human verification:** Read missing source lines and reader joins side by side. Confirm four/two/one distinct lines are absent at the documented development joins and no output Latin is edited.

### 5.2 Match starts and terminal prose

Implement `compare_boundaries` with the same occurrence alignment. Determine actual terminal prose inside source partitions, not from a following start page or a period alone. Report matched terminal correspondence separately from interior completeness. Sparse complete pages and cross-page split words are legitimate. Check development endpoints now; defer all68 and Brutus/Otho/other validation endpoints to Phase10.

**LLM verification:** Compare development terminal/source regions and the report ledger. Inspect gaps where prose ends before the next source page. Report unknown/inconclusive OCR honestly; do not call it terminal loss without source support.

**Human verification:** Review development sparse endings and complete endings in units with interior defects. Inspect Otho's split continuation after validation opens.

## Phase 6 - Google watermark remnants - Suggested implementer: Opus

### 6.1 Build source templates and search reader images

Implement `make_templates`, `match_templates`, and shared peak grouping. Use reviewed development source logosv1/287,299,321; full logo, text, and distinctive letter-group fragments. Preserve grayscale/faint pixels; reject constant/near-empty templates and nonfinite scores. Match grayscale and local-contrast views using `cv2.matchTemplate(...,TM_CCOEFF_NORMED)` ([OpenCV reference](https://docs.opencv.org/4.x/df/dfb/group__imgproc__object.html)).

Initial scale bank uses source/output line-height ratio times0.75..1.30 in0.05 steps, fallback0.3..1.5; rotation-3..3 degrees. Start nomination correlation0.60, then calibrate. Search each allowed body image coarsely and refine peaks/source-seam regions at300 DPI, including regions touching prose. Tiny fragments need source-seam corroboration and remain low; a single ordinary Latin curve cannot earn high confidence from correlation alone.

**LLM verification:** EvaluateW4 and repairedW1/W2 neighborhoods, inspect peak/correlation/source evidence, and measure false matches in development negatives. No Brutus template or validation detection is used.

**Human verification:** Compare surviving logo fragments and ordinary letters; confirm scores are similarities rather than percentages of certainty.

### 6.2 Handle split logos and exemptions

Implement `join_body_edges` and `merge_logo_events`. Stitch diagnostic bottom/top strips from adjacent pages of the same unit, preserving both coordinates and avoiding generated headings. Use partial templates and actual image-placement edges; virtual stitching is evidence, not a repaired image. Exempt intentional notices by frontmatter role/asset rather than permanently hard-coding physical8.

**LLM verification:** InspectW4 as one event spanning two pages and confirm all intentional notice roles are excluded from body candidates. Verify that only same-unit adjacent images are stitched and body logos touching Latin remain eligible.

**Human verification:** Open separate and stitched crops, checking both locations and the retained uncertainty of incomplete fragments.

## Phase 7 - Running heads and Greek scraps - Suggested implementer: Opus

### 7.1 Combine patterns with geometry and source correspondence

Implement `find_head_candidates` with1-20 Latin/Greek capital fragments, optional period, and separate2-4-digit folios. Do not require fragment+number on one OCR line. Keep Greek/Latin lookalike uncertainty. Match source-head suffixes and source folios; measure isolation/seam context. Exclude generated title/footer placements and chapter/reference patterns. Retry small regions with alternate PSM/Greek model when available; H2 can use source-image correspondence even if OCR fails. Group nearby components within3 median line heights without inventing the complete head.

**LLM verification:** InspectH1/H2 and generated/chapter/reference negatives. Run the two small head sensitivity folds without tuning on the held-out fold result; state their limited independence. Check confidence conditions and single-character eligibility.

**Human verification:** Compare the actual leaked fragments/435/Σ with raw source heads; confirm no guessed expansion is presented as observed text.

### 7.2 Keep neighboring-column scraps separate

Implement `find_foreign_scraps` using marginal Greek observations/image components and source Greek-column correspondence. F1/F2 are scraps, not full head leaks. Reference letters alone do not qualify. Missing Greek OCR produces a qualified image-supported candidate or limitation, not silent detector success.

**LLM verification:** Inspect development scrap cases and reference-letter negatives selected by the fixed split. Defer any validation-unit scrap checks. Record recognition/model limitations.

**Human verification:** Compare marginal scraps with legitimate edition reference letters and distinguish those labels from running heads.

## Phase 8 - Supplemental layout candidates - Suggested implementer: Opus

### 8.1 Nominate rules, blank-body candidates, and copied gaps

Reuse shared body boxes/image metrics and text correspondence. `find_rules_and_gaps` identifies long/broken vertical components, possible prose-free body images, and gaps>=3 median line pitches. Compare aligned source spacing before calling a gap copied margin. Genuine chapter/verse gaps remain negative even when a rule survives. An inked OCR-unreadable page is inconclusive, not blank. These are candidate diagnostics, not mask measurements or erase instructions.

**LLM verification:** Check only developmentM/R regions and repaired blank-page negatives now. Compare genuineI/36,46,61 spacing with copied margins in development Timoleon/Coriolanus units. M10/M11/R13 Brutus and other validation-unit examples stay deferred.

**Human verification:** Read source and reader gaps/rules side by side. Confirm white separators and sparse complete conclusions are not defects.

### 8.2 Nominate horizontal and lateral clipping separately

Implement `find_slicing_and_clipping`, reusing source alignment and image matching. Check actual body-image boundary glyph components, complementary adjacent-page fragments, and repeated left-edge cuts with source ink beyond the crop. Confirm suspect cuts on full300-DPI regions; a diagnostic strip's cropped second line is not a page slice. Rules crossing edges are not sliced text. Preserve unmeasured extent/uncertainty.

**LLM verification:** Inspect developmentS1/C4/C6 with full source/reader crops and report evidence; deferS2/C7 to validation. Record source-line correlation, affected regions, and failures without inferring exact missing glyph counts from OCR.

**Human verification:** Distinguish horizontal fragments from missing left glyphs and verify body bounds exclude generated titles/folios.

## Phase 9 - Reporting and development calibration - Suggested implementer: Opus

### 9.1 Produce a reviewable report and finish annotations

Implement `write_report` using the evaluator already introduced in1B, not a second evaluator. Save candidate JSON/CSV, coverage/error logs, and static local HTML with source/output crops, raw OCR, locators/provenance, component scores, confidence conditions, related events, and reasons. Escape text/paths and prevent executable CSV-cell interpretations. Keep low/unknown findings visible. Finish any pending reviewed gold regions before their evaluation; do not infer gold from predictions.

**LLM verification:** Open the development HTML and verify crop links, dual/source locators, unknown-folio labeling, and complete requested-scope coverage. Compare report candidates to JSON/evaluation and inspect an omission, duplicate, faint furniture case, and negative gap.

**Human verification:** Assess whether the evidence explains each warning without knowledge of the code. Check the development hit/miss list and unresolved annotations.

### 9.2 Revise detection on development evidence and freeze it

Investigate misses in order: mapping/coordinates, expected-source extraction, OCR, retrieval, occurrence alignment, then thresholds. Save successive configs/results; do not hard-code finding IDs/pages/anchors in detector branches. Existing gold corrections require independent image evidence and a documented annotation error, not a convenience relabeling. Group events consistently.

Evaluation localization rules: an omission must match the annotated expected occurrence/source lines and output seam, covering>=80% of the labeled source span with no more than one extra source line; oversized page/unit warnings do not count. A duplicate needs both source/reader occurrences and contribution endpoints within one reader line of annotated boundaries, accounting for incomplete repeated openings. Tiny image events need boxIoU>=0.1 or the predeclared center-distance<=one line-height rule plus correct type/source correspondence; choose that rule before scoring. Unreviewed events remain unreviewed. Natural repeated/moved text must not gain false coverage from a lexical hit elsewhere.

Use development positives/negatives to revise predicates and nomination settings. Report candidate burden, per-class reviewed precision, and panel false-positive counts with actual denominators. Keep at least one strong positive per core class high; frozen confidence cannot be changed by the evaluator. `evaluate --freeze` saves thresholds, templates, source-index/profile identities, tool/model versions, split hash, reviewed development metrics, and known misses under that explicit run. Freeze only a complete final development run; no marker belongs in snapshot inputs.

**LLM verification:** Evaluate each development core/layout/end case in scope, inspect all missed/overbroad matches and adjudicated negatives, and compare revisions. Review all candidates on development panel pages before computing a per-page false-positive rate. Check the freeze JSON binds actual settings and that no validation output has been scored.

**Human verification:** Review remaining false positives and the changes made to address them. Check that threshold adjustments preserve strong positives and do not hide candidates or genuine source spacing.

## Phase 10 - Controlled validation and final inspection run - Suggested implementer: Opus

### 10.1 Run held-out units using the frozen configuration

Process validationD9-D12, C5/W5/W6/M10/M11/R13, S2/C7, and other fixed validation units for the first time. Require the development freeze and matching semantic settings; retain first results. Source-reference vocabulary may include their original text, but reader output/labels never fitted the index. If methods need revision afterward, record it as post-validation regression work, not fresh independent validation.

Final requirements:

| Area | Gate |
| --- | --- |
| Core positives | All nineD4-D12 blocks, all fourC1/C2/C3/C5 unique losses, all threeW4-W6 watermarks, and bothH1/H2 heads are nominated with correct-type localized evidence. No generic page warning substitutes for a hit. |
| Core distinctions | Eight missing source lines remain distinct from redundant-copy openings, displaced text, and other natural repeated occurrences. ApprovedD1-D3 exclusions do not create unique-loss findings. |
| Negatives | No high-confidence head/logo on reviewed generated/repaired exemptions; no high-confidence copied-margin label on genuine spacing; no divider-only claim on current prose/sparse-ending/separator examples. |
| Supplemental layout | Process every annotatedM/R/F/S/C clipping region; nominateS1/S2 andC4/C6 when image correspondence supports them. Record localized low-confidence outcomes and any unresolved misses, particularlyC7 extent, rather than inventing assurance. These exploratory layout limitations do not erase core success, but must be explicit. |
| Endings | Compare all68 terminal source/output passages; resolve any purported terminal loss against the complete report endpoints and distinguish inconclusive OCR from confirmed truncation. |
| Coverage/integrity | All2,678 body pages/68 units accounted for, with actual completed scope and no hidden tool failures; source/reader/policy/snapshot hashes unchanged. |
| Confidence/burden | Strong development positives per core class remain high. Candidate burden<=5 grouped output core candidates per100 body pages; excess requires a documented scoring/grouping revision, not truncation. Fully review all200 fixed panel pages before the final FP rate; high-confidence core FP<=1 per100 fully reviewed panel pages. |

Also review top50 previously unlabeled core events and up to10 low-confidence events per core type as a separate candidate-selected sample; label its precision and uncertain counts without presenting it as a representative page rate. Known core positives are mandatory regression targets, not proof of corpus-wide recall. Supplemental layout may remain exploratory, but neither core misses nor unexplained coverage failures count as completion.

**LLM verification:** Run/evaluate validation with frozen settings; inspect every missed core case and negative regression, confidence predicates, and actual panel counts. Complete the deferredC5 source/reader first-line assertion, S2/C7 evidence, and all68 endpoint evaluation. Preserve first-validation metrics before revisions.

**Human verification:** Review held-out single-line loss, faint logos, new duplicate blocks, S2, and validation negatives. Check that any revision is general and remains clearly labeled post-validation.

### 10.2 Finish the full run and deliver evidence

Run all units with the chosen final settings into an explicit new/resumable run folder; evaluate all cases. Inspect the fixed panel and supplemental candidate sample. Reuse valid renders/OCR; do not build a permanent testing framework. Record actual runtime/storage, dependency versions, candidate counts, core hit/miss/negative results, layout limitations, and unchanged input hashes. Save a new dated calibration/completion note under `llm_artifacts/` and retain all historical files.

**LLM verification:** Rerun the explicit final directory with `--resume`; compare normalized candidate identities/regions/scores, excluding timestamps/durations. If using multiple workers, compare the same development subset with workers1/4 using separate run records; do not share PyMuPDF objects. Run `python -m py_compile tools/audit_inner_tomi.py`, inspect every requested coverage row, rehash inputs, and run `git diff --check` / `git status --short`. No unit tests, lint framework, or Rust build are required for this script-only change.

**Human verification:** Open the final HTML, check crop links and highest-priority warnings, and read the calibration note's limitations. Confirm the tool performed inspection only and that the existing PowerShell Python invocation works.

## Implementation discipline

This plan is for the single diagnostic script; it does not start implementation, install dependencies, or alter source Latin. Follow small steps, keep useful progress messages during OCR, and preserve concise continuation notes with actual run/config IDs and unresolved cases. Changes to costs/thresholds require another evidence-backed calibration run, not a new software architecture. Keep existing phase/step identifiers if the plan is revised again.

Script comments describe current helper behavior and invariants, without plan-phase references. Diagnostic annotation provenance may cite the findings report. Respect the user's explicit decision to omit unit tests: use the actual inspected corpus, visual comparisons, and calibration results as verification. Do not add a test suite indirectly under another name.
