# plutarch-reader

Generates a Loeb-size (10.5 × 16 cm), Latin-only reader of Plutarch's *Lives* in
the Didot bilingual recension (Dœhner, Paris 1857) from the two Google Books scans,
and imposes it onto US Letter or A4 PDFs, ready to print double-sided, fold into
32-page signatures, sew, and trim to pocket size.

The build (`cargo run --bin facsimile -- …`) makes body pages from the **original
1857 Didot Latin column** itself, cropped from the scans and reflowed into the trim
text block, so the reader reads the verbatim printed type. No AI vision touches the
body; it is offline, deterministic, and costs nothing per body page. The PDFs are
image-heavy and **not** text-searchable. The reader emits per-volume LaTeX with a
shared imposition, signatures, and fold/sew/cut marks; it imposes on US Letter by
default, or on A4 with `--paper-size a4`.

---

## The facsimile build

For a full PDF build on this Windows checkout, run the following from the project
root in PowerShell. MiKTeX must be installed with the `oldstandard` package; the
explicit executable path avoids relying on `PATH`. The command uses the cached
front-matter prose and compiles all ten tomi, including their printing wrappers.

```powershell
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
& $env:PLUTARCH_LUALATEX --version
if ($LASTEXITCODE -ne 0) { throw "LuaLaTeX is unavailable; fix the MiKTeX path or installation before building." }
cargo run --release --bin facsimile --target-dir target -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
if ($LASTEXITCODE -ne 0) { throw "The facsimile build failed; inspect the error above." }
Get-ChildItem build_facsimile/tomus*.pdf
```

The completed build produces `tomus_1_inner.pdf` through `tomus_10_inner.pdf` (book
pages) and `tomus_1.pdf` through `tomus_10.pdf` (imposed US Letter printing PDFs
by default).
If MiKTeX is installed elsewhere, change `PLUTARCH_LUALATEX` accordingly. When
running through Codex's restricted filesystem sandbox, allow the build access to
the MiKTeX installation and its package/font caches. If LuaLaTeX cannot be invoked,
the generator can finish successfully with LaTeX files alone and print
`lualatex not run`; that message means the PDF build has not completed. Confirm
that all sixteen PDFs exist after the run. Re-run the generator after fixing
LuaLaTeX so it derives each printing wrapper from the compiled inner page count.

```sh
# Detect the unit table from the sources and write facsimile_units.json, then exit.
cargo run --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf \
    --out build_facsimile/ --detect-units

# Full build (auto-detects the table on first run if it is absent).
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/

# Impose the same trim-size inner pages on A4 sheets.
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline --paper-size a4

# Full offline build with a personalized ownership plate.
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline --exlibris "Alex Smith"

# Re-detect the table even when the cache is present (keeps pinned hand edits).
cargo run --bin facsimile -- --v1 … --v2 … --out build_facsimile/ --redetect

# Rebuild from source scans and cached front-matter prose without vision calls.
cargo run --bin facsimile -- --v1 … --v2 … --out build_facsimile/ --offline

# A single unit, or a raw source page range, for a cheap test.
cargo run --bin facsimile -- --v1 … --v2 … --out build_facsimile/ --unit theseus --no-compile
cargo run --bin facsimile -- --v1 … --v2 … --out build_facsimile/ --pages v1:14-45 --no-compile

# Purge the raster cache first, forcing every page to be re-rasterized.
cargo run --bin facsimile -- --v1 … --v2 … --out build_facsimile/ --clean-cache
```

For the facsimile command, `--exlibris "Alex Smith"` adds a bordered
`EX LIBRIS` / `ALEX SMITH` plate to each full-build Tomus title page. Names are
converted to uppercase; empty or whitespace-only names are rejected. Omit the
argument to print no ownership plate. Restricted `--unit` / `--pages` builds omit
the title pages and therefore the plate.

Omitting `--paper-size` selects US Letter. `--paper-size a4` changes only the
facsimile printing wrappers: both options keep the 10.5 × 16 cm inner pages and
the same 21 × 16 cm cut border. Each landscape `tomus_N.pdf` page is already
imposed as a two-page spread. Print at actual size (100%, without fit-to-page
scaling), select two-sided printing with **flip on the short edge**, and leave
the printer's booklet and two-pages-per-sheet options off. The short-edge flip
keeps both sides upright when each sheet is folded into a signature. Cut to the
marked trim border after folding and sewing.

**Body.** Each page's Latin column is detected, sliced at inter-line whitespace, and
reflowed (whole source lines repaginated at one **global** scale) into the trim text
block, under a **typeset centered Latin running head** and the per-Tomus folio.
Conservative cleanup excludes isolated dividers and verified source watermarks,
running heads, and scan-edge noise while preserving Latin pixels. Ambiguous
dividers remain; the cached source rasters stay pristine.
Margins are inner / outer 1 cm, 1.5 cm above the running head, and 1.5 cm below the
text. Each nonempty Life and Comparatio **opens on a recto** (an odd physical book
page), with its display title typeset above the first body image; Comparationes
follow their pair. A unit boundary inserts zero or one empty verso, preserving
consecutive pages within each unit. The first body page anchors folio 1 after recto
alignment, so physical-page and body-folio parity agree. Opening folios are hidden.

The reusable poppler rasters in `<out>/render_cache/` are preserved across runs by
default. `--clean-cache` deletes that directory before building to reclaim its disk
space or force a fresh rasterize, including for `--unit` / `--pages` runs; needed
pages then refill the cache. `--detect-units` exits without purging the raster cache.

**The unit table is auto-detected**, not hand-authored. The detector reads each
page's embedded OCR text layer (`pdftotext`), traces the running head to segment the
body into unit runs, and pins each unit's exact start page to its display-title line,
recognizing *which* known unit a page belongs to against a built-in canonical corpus
of the Parallel Lives in Didot order. Greek titles, kind, pair-group, and ordering
come from that corpus (never from unreliable polytonic-Greek OCR); the page OCR only
*identifies* and *locates* units. The result is written to `facsimile_units.json`, a
committed, hand-editable **cache**: `--detect-units` / `--redetect` regenerate it, a
normal build auto-detects when it is absent, and any row marked `"pinned": true` (and
its `skip_top_bands` tweak) survives regeneration unchanged. See
`llm_artifacts/2026_10_03_FACSIMILE_UNIT_AUTODETECT_PLAN.md`.

**Front matter** is the one place the facsimile path uses vision: the v1 Monitum
prose through “mutanda esse significavit.” is re-typeset to the trim measure, and
its Greek errata list follows immediately on one page containing the p12 and p13
source crops, scaled together. The first body page opens on the next recto after
that combined errata page, with zero or one intervening blank. The QR notice,
Volumen-title, and colophon scans retain their
existing page starts. The v2 Praefatio typesets the opening Latin through `Ages.`,
then reproduces all p15–p16 corrigenda as intact printed line images on balanced,
dedicated pages, and resumes typeset Latin and the dateline from p17. The image
run uses one common scale and flows across the source-page boundary without
parity blanks. Derived title pages are typeset too. The prose is transcribed
**once** and cached in `plutarch.frontmatter.tags.json`; later builds make zero
network calls when every required result is present. `--offline` reads a fresh
cache without updating it. An emitting Praefatio requires finalized prose from
v2 p14, p15 and p17 and rejects a missing, stale, or unreviewed mixed-text cache;
unrelated volumes retain the warning-and-skip behavior for unavailable prose.
`--offline` cannot be combined with `--retag` or `--review-lines`. Only
`FrontProse` pages requiring transcription are sent: v1 p13 and v2 p16 are images
and excluded from transcription and reconciliation; v2 p15 is masked to its Latin
prose region for both stages. No body page is sent. Finalized results retain their
historical candidates and are not regenerated merely because masking changed.
A legacy p15 candidate without a finalized prose prefix is rejected before any
API call. Restore a reviewed finalized prefix or deliberately use `--retag` after
preserving a copy: retagging resets the entire front-matter sidecar, including its
corrections and candidate evidence.

---

## Inspecting the facsimile inner PDFs

[`tools/facsimile_line_audit.py`](tools/facsimile_line_audit.py) audits source-line
ink coverage and source, unit, duplicate and reader seams against immutable
captures of the accepted readers. Source-only geometry and ownership reviews,
controlled duplicate equivalence, visible-PDF pixel measurements and neutral
review evidence govern its observations. Unreviewed source regions, weak fits
and small projected components remain explicit uncertainty. Follow
[`tools/README-facsimile-line-audit.md`](tools/README-facsimile-line-audit.md) for
capture, adjudication, calibration, full offline runs, capped optional review,
scope verification and immutable archiving. Processing completion does not
certify complete Latin coverage.

[`tools/audit_inner_tomi.py`](tools/audit_inner_tomi.py) inspects the ten
facsimile inner PDFs against the original bilingual scans and reports suspected
**duplicated passages, unique missing source occurrences, displaced or reversed
passages, and incomplete starts or endings**. Image checks look for **surviving
running heads and folios, Google watermark fragments, neighboring Greek-column
scraps, copied page-margin gaps and divider rules, prose-free (blank) body
regions, lateral glyph clipping, and lines sliced across output pages**. Every
finding carries localized source/reader evidence and its confidence conditions.
The audits write diagnostic files and leave the Latin,
the production PDFs, the LaTeX, and the repair policy unchanged — they never
repair or normalize the source Latin.

Run commands from the repository root in PowerShell with a Python 3.10+
interpreter. Poppler (`pdftoppm`) and Tesseract must be available; the
prerequisite check prints their versions, installed OCR models, and Python
package versions. If imports are missing, install the pinned dependencies.

```powershell
python tools/audit_inner_tomi.py check
python -m pip install -r tools/requirements-inner-audit.txt
```

The audit works from a captured, hash-verified baseline. Prepare its working
specs and capture the pinned inspected inputs as described in
[`tools/README-inner-audit.md`](tools/README-inner-audit.md). The tracked
baseline spec identifies the exact source scans, reader PDFs, LaTeX, unit table,
and repair-policy trace; if any of these inputs change, capture stops with a
mismatch and a new baseline needs an explicit specification. Diagnostic
snapshots, caches, and runs live under the ignored `tmp/pdfs/inner_tomi_audit/`
directory and are not committed.

The ordinary Rust regression suite runs from tracked fixtures and synthetic or
test-created inputs. It does not require source scans, historical audit archives,
API credentials, or preexisting rendered caches. The normal generator and the
diagnostic audit above require the exact scans and their tools. To verify the
ordinary suite in a temporary snapshot of tracked working-tree files, run:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
cargo test
python tools/check_portable_tests.py --offline --log tmp/portable-tests.log
```

Omit `--offline` if Cargo needs to download dependencies. Stage intended new files
before creating the snapshot. The tool excludes local evidence and scans, runs
the full ordinary suite, and requires an explicitly selected historical test to
fail clearly without its evidence. See the
[verification and fixture-maintenance instructions](tools/README-inner-audit.md#regression-and-evidence-verification)
for the separate source-PDF checks and seven historical authentication tests.

For an initial development inspection after capture:

```powershell
python tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope development --detectors all --workers 4 --run-dir tmp/pdfs/inner_tomi_audit/runs/my-development-run
python tools/audit_inner_tomi.py evaluate --run-dir tmp/pdfs/inner_tomi_audit/runs/my-development-run --split development
python tools/audit_inner_tomi.py report --run-dir tmp/pdfs/inner_tomi_audit/runs/my-development-run
Start-Process tmp/pdfs/inner_tomi_audit/runs/my-development-run/index.html
```

A validation or full-corpus request requires a complete development **freeze**
that matches the script, settings, specs, inputs, source index, and tool
versions. Creating the freeze requires independent candidate adjudications and
the fixed development page review, supplied through `evaluate --reviews … `
with `--freeze`; the setup guide describes the review JSON and the exact
commands. The frozen full-corpus run of record was produced with the development
freeze like so, writing into a new run folder:

```powershell
python tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope all --detectors all --freeze tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1/freeze.json --lang eng --workers 4 --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-all
python tools/audit_inner_tomi.py evaluate --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-all --split all --reviews tmp/pdfs/inner_tomi_audit/runs/step102-final-all/reviews.json
python tools/audit_inner_tomi.py report --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-all
```

Add `--resume` when repeating the same run identity so valid cached renders,
OCR, and comparisons are reused; it also serves as a determinism rerun, whose
candidate records must be identical to the first. `--units` and `--tomus` narrow
the selected scope, and the reports show the actual requested/completed pages.
Use a new run folder for any changed settings.

Open a run's `index.html` in a browser to inspect findings with their confidence
reasons, scores, physical-page and printed-folio locators, source/reader crop
pairs, related events, and raw OCR; `boundaries.html` holds the per-unit
start/ending comparisons. Running `report` after `evaluate` folds the saved
evaluation into the HTML without rerunning detection. Keep each run's `crops/`
folder beside its HTML. Machine-readable results are in `candidates.json` and
`candidates.csv`; `evaluation-<split>.json` carries localized hit/miss and
negative-example results. `coverage.json`, `errors.json`, and `run-manifest.json`
record the completed scope, any processing failures or inconclusive
observations, the input/configuration identities, runtime, and storage.
Per-detector debug JSON retains failed comparisons and rejected logo probes for
further inspection.

The prose comparison builds its text index from **original-source observations
only**, then maps reader OCR to source occurrences using overlapping passage
windows, ordered anchors, and bounded alignment (a character n-gram TF-IDF index
that is transformed, never refit, on reader windows). Repeated scans and
multiple reader visits are tracked separately, so a missing redundant copy is
not counted as unique source loss. Source-raster agreement, reader OCR retries,
and retained-line image searches corroborate omission candidates. Endings are
compared against the actual terminal source prose; a matching endpoint does not
establish interior completeness.

Furniture and clipping checks compare original-source and reader pixels with
scale-aware template matching, source-image registration, connected components,
and measured glyph boundaries; the Google watermark search correlates source
logo fragments against reader pages. Virtual joins of adjacent reader edges help
inspect fragments sliced across a page boundary while retaining both original
coordinate systems. Generated headings, intentional notices, genuine source
spacing, and reviewed repair neighborhoods provide local negative examples. A
fixed development/validation split keeps reader output from fitting the source
index or the logo templates; evaluation uses independently measured positions,
with a separate fixed-page false-positive panel and a candidate-selected
precision sample. Similarity scores are not probabilities, and known-defect
regression hits do not establish corpus-wide recall. Low-confidence and
inconclusive observations stay visible; exploratory layout and endpoint
localization limitations are reported explicitly and require image review. See
the dated calibration note
[`llm_artifacts/2026_10_08_09_18_INNER_TOMI_VALIDATION_CALIBRATION.md`](llm_artifacts/2026_10_08_09_18_INNER_TOMI_VALIDATION_CALIBRATION.md)
and the full-corpus completion note
[`llm_artifacts/2026_10_08_14_44_INNER_TOMI_AUDIT_STEP_102_COMPLETION.md`](llm_artifacts/2026_10_08_14_44_INNER_TOMI_AUDIT_STEP_102_COMPLETION.md)
for the verified counts, identities, and remaining limitations.

---

## Dust covers

[`tools/dust_cover.py`](tools/dust_cover.py) generates a parameterized dust
jacket (`.tex` and compiled `.pdf`) for one Tomus. It reads the Lives and title
page block from `build_facsimile/tomus_N_inner.tex`, so run the facsimile build
first. Covers run 1 through 10, one per Tomus the build produced; the tool itself
accepts up to Tomus 20. The sheet runs left to right as flap, back cover, spine,
front cover, flap. The front reproduces the inner title block, and the back lists
its Lives.

From the repository root in PowerShell:

```powershell
python tools/dust_cover.py --tomus 1 --total-width 15.5 --total-height 9.5 --spine-width 1.25 --cover-width 5.5 --paper-size tabloid --exlibris "Alex Smith" --out build_facsimile
```

| Argument | Meaning | Required | Default |
|----------|---------|----------|---------|
| `--tomus` | Tomus number — any Tomus the build produced | Yes | — |
| `--total-width` | Finished jacket width, inches | Yes | — |
| `--total-height` | Finished jacket height, inches | Yes | — |
| `--spine-width` | Spine width, inches | Yes | — |
| `--cover-width` | Width of each front/back cover, inches | Yes | — |
| `--paper-size` | Landscape sheet: `tabloid` (11 × 17 in) or `a3` | No | `tabloid` |
| `--exlibris` | Owner name printed at the spine foot | No | None |
| `--printable-margin` | Unprintable margin on each sheet edge, inches | No | `0.25` |
| `--out` | Directory containing the inner `.tex` and receiving cover output | No | `build_facsimile` |
| `--inner-tex` | Explicit path to the inner `.tex` | No | `<out>/tomus_N_inner.tex` |
| `--no-compile` | Write the cover `.tex` without running `lualatex` | No | Compile PDF |

The width is `2 × flap + 2 × cover + spine`; the two flaps expand or shrink
equally and must each be at least 1.5 inches. The jacket width and height must
fit the printable sheet (sheet size minus twice `--printable-margin`). Short
fold ticks at the four seams sit outside the cut rectangle, so the height check
also reserves 0.12 inches above and below the jacket for those marks. The tool
errors before writing output when these dimensions do not fit. See
[`tools/README-dust-cover.md`](tools/README-dust-cover.md) for usage notes.

---

## Printing sheets

[`tools/print_sheets.py`](tools/print_sheets.py) prints a page range of an
imposed `build_facsimile/tomus_N.pdf` to the Windows default printer as a series
of separate two-page jobs, one job per physical sheet, each duplex
flip-on-short-edge, monochrome, and at actual size. Printing one sheet per job
means a jam ruins only that sheet, and a damaged signature can be reprinted on
its own. It needs [SumatraPDF](https://www.sumatrapdfreader.org/) and PyMuPDF
(`python -m pip install -r tools/requirements-print-sheets.txt`).

From the repository root in PowerShell:

```powershell
python tools/print_sheets.py build_facsimile/tomus_1.pdf --pages 145-160
```

The range must start on an odd page (the front of a sheet). See
[`tools/README-print-sheets.md`](tools/README-print-sheets.md) for the full
argument table, SumatraPDF setup, exit codes, and driver caveats.

---

## Dependencies

| Tool | Purpose | Override env var |
|------|---------|------------------|
| Rust / Cargo | build the tool | — |
| poppler `pdftoppm`, `pdfinfo` | rasterize source pages, read page counts | `PLUTARCH_PDFTOPPM`, `PLUTARCH_PDFINFO` |
| poppler `pdftotext` | read the embedded OCR layer for facsimile unit detection | `PLUTARCH_PDFTOTEXT` |
| MiKTeX `lualatex` + the **Old Standard** font (`oldstandard` package) | compile the LaTeX | `PLUTARCH_LUALATEX` |
| Tesseract with the `lat` language model | OCR cross-check for the inner-PDF audit | `PLUTARCH_TESSERACT` |

Each tool is found on `PATH` by default; point the matching env var at an explicit
binary if it is elsewhere. Old Standard is loaded **by file name** (the
`OldStandard-*.otf` files), so it need only be installed in the TeX tree — run
`mpm --install=oldstandard` if lualatex cannot find it.

An Anthropic API key is needed **only** for the one-time front-matter transcription —
the single place the build uses AI vision. The body build needs no key. Put the key
in `secrets.json` in the project root:

```json
{ "anthropic_api_key": "sk-ant-…" }
```

See `secrets.example.json`. This file is **local only** — it is not shared.

## Output and the two-stage compile

For each volume the build writes, into `--out`:

- `tomus_N_inner.tex` — the trim-size inner book pages.
- `tomus_N.tex` — the imposition wrapper: US Letter by default, A4 when
  `--paper-size a4` is given.

With lualatex available they compile both: the inner once (it self-pads to a whole
32-page signature), reads the real page count from the lualatex log, then emits and
compiles the wrapper, which pulls inner pages two-up via
`\includegraphics[page=N]{tomus_N_inner.pdf}` in signature-folding order. Without
lualatex (or with `--no-compile`) it writes the wrappers from an estimated page count
and prints the manual compile commands. Fold lines are drawn on each outer/recto
sheet side only; sew marks on every sheet side (both faces); a cut border on the
first recto of each signature.

## Files

- `facsimile_units.json` — the auto-detected, committed, hand-editable unit table for
  the facsimile path (`--detect-units` / `--redetect`; `pinned` rows preserved).
- `plutarch.frontmatter.tags.json` — the committed, hand-editable cache of the
  re-typeset front matter.
- `build_facsimile/`, `target/`, `secrets.json` — local, not shared.
- `spikes/` — throwaway layout and detection experiments kept as a record.
- `llm_artifacts/` — the implementation plans and notes, kept as the dated record of
  why the code is shaped as it is.

The committed caches above are kept in the project tree and hand-corrected in
place; git carries their history.
