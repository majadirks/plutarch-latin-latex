# plutarch-reader

Generates a Loeb-size (10.5 × 16 cm), Latin-only reader of Plutarch's *Lives* in
the Didot bilingual recension (Dœhner, Paris 1857) from the two Google Books scans,
and imposes it onto US Letter PDFs, or A4 PDFs on the facsimile path, ready to
print double-sided, fold into 32-page signatures, sew, and trim to pocket size.

There are **two build paths**, sharing the same imposition, signatures, fold/sew/cut
marks, and two-stage compile:

- **facsimile** (`cargo run --bin facsimile -- …`) — body pages are the **original
  1857 Didot Latin column** itself, cropped from the scans and reflowed into the trim
  text block, so the reader reads the verbatim printed type. No AI vision touches the
  body; it is offline, deterministic, and costs nothing per body page. The PDFs are
  image-heavy and **not** text-searchable.
- **transcription** (`cargo run -- …`) — the Latin column is **re-typeset** from an AI
  vision transcription, producing clean, selectable text at the cost of a one-time
  paid run (see the cost gate below).

Use **facsimile** when you want the verbatim pixels of the 1857 edition at near-zero
body cost and don't need to search or copy the text; use **transcription** when you
want re-typeset, selectable Latin and the budget allows. Both emit per-volume LaTeX.
The transcription path imposes on US Letter; the facsimile path uses US Letter by
default and accepts A4 with `--paper-size a4`.

---

## The facsimile path

For a full PDF build on this Windows checkout, run the following from the project
root in PowerShell. MiKTeX must be installed with the `oldstandard` package; the
explicit executable path avoids relying on `PATH`. The command uses the cached
front-matter prose and compiles all eight tomi, including their printing wrappers.

```powershell
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
& $env:PLUTARCH_LUALATEX --version
if ($LASTEXITCODE -ne 0) { throw "LuaLaTeX is unavailable; fix the MiKTeX path or installation before building." }
cargo run --release --bin facsimile --target-dir target -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
if ($LASTEXITCODE -ne 0) { throw "The facsimile build failed; inspect the error above." }
Get-ChildItem build_facsimile/tomus*.pdf
```

The completed build produces `tomus_1_inner.pdf` through `tomus_8_inner.pdf` (book
pages) and `tomus_1.pdf` through `tomus_8.pdf` (imposed US Letter printing PDFs
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
the same 21 × 16 cm cut border. Print the landscape `tomus_N.pdf` sheets at
actual size (100%, without fit-to-page scaling) and flip on the short edge when
duplex printing. Cut to the marked trim border after folding and sewing.

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

[`tools/audit_inner_tomi.py`](tools/audit_inner_tomi.py) inspects the eight
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

## The transcription path

Transcribes **only the Latin column** of each page with AI vision, verifies the
result across models, and re-typesets it as selectable text.

1. **Classifies** every source page (front matter, body, or skip). The unambiguous
   middle of each PDF is marked body with no API call; only the front/back boundary
   regions are examined, and near-uniform (blank/dark) thumbnails are skipped
   deterministically.
2. **Transcribes** each body and front-matter-prose page into a structured per-page
   document (headings, Roman chapter numbers, parenthetical section numbers,
   paragraph-continuation flags, verse), reproducing the printed Latin **verbatim**
   (no normalization of u/v, i/j, ligatures, the long s, spelling, or punctuation).
   **Sonnet is the sole authoritative transcriber;** body pages are cropped to the
   **Latin column** first (front-matter prose is sent full-width). A second **Haiku**
   pass is run only as a cheap **divergence signal** — its reading is compared
   against Sonnet's to decide whether a page needs Opus, and is never stored as the
   result. The output schema is compact (only `kind` and `text` are required), and
   every pass is submitted through the **Message Batches API in chunks** for a flat
   50% discount. The Haiku signal is behind a toggle (`DIVERGENCE_SIGNAL`); turning
   it off is the first cost lever.
3. **Verifies and reconciles (free, local).** For each page the Sonnet and Haiku
   passes are compared under **orthography folding** (u/v, i/j, the long s,
   ligatures, case, spacing, and punctuation are ignored), so only a **substantive**
   wording difference — not spelling noise — counts as a disagreement. Section-number
   continuity and a **coarse** Tesseract-`lat` char-count check (a dropped or garbled
   page) supplement that signal. A page with no flags is accepted as-is with Sonnet's
   text; only flagged pages escalate to **Opus**, which re-decides the page from the
   image at **reduced effort** (`ARBITRATION_EFFORT`), guarded so an implausibly short
   Opus reply keeps the Sonnet text. Reliability rests on Sonnet-quality Latin plus
   cross-model divergence detection plus a human spot-check — **not** on Haiku's own
   readings.
4. **Assembles** pages into Lives and Comparationes, splits them into balanced
   output volumes on pair-group boundaries (a hard break is forced at the v1→v2
   source boundary), and emits each volume's inner `.tex` and imposition wrapper.
5. **Compiles** two stages with lualatex: the trim-size inner, then the two-up US
   Letter wrapper carrying fold lines, sew marks, and cut borders. The facsimile
   command alone can select A4 for that wrapper.

Transcription and escalation run as asynchronous batches: each is submitted, then
polled to completion (most finish within an hour; the cap is 24 h), with a progress
bar over the chunks. Each chunk's batch id is persisted in the sidecar, so an
interrupted run **re-attaches** to a batch already in flight instead of resubmitting
and paying twice. At the end of any run it prints a **usage report**: per model
(Haiku, Sonnet, Opus) the call count and tokens spent — total, and median / max /
min per page. A fully-cached offline run reports zero calls.

```sh
# Full build (transcribes everything on the first run, then is offline).
cargo run -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build/

# Cheap test: the first 4 body pages of v1 (separate dryrun sidecar).
cargo run -- --v1 … --v2 … --out build/ --dry-run 4

# A specific source page range; re-call the API; or write .tex without lualatex.
cargo run -- --v1 … --v2 … --out build/ --pages v1:40-60
cargo run -- --v1 … --v2 … --out build/ --retag
cargo run -- --v1 … --v2 … --out build/ --no-compile
```

The first full run populates `plutarch.tags.json`; subsequent runs read it and make
**zero** network calls (instant). `--retag`, a changed source file, or a missing
sidecar triggers re-tagging. A crash or Ctrl-C resumes — the sidecar is saved after
every page. Restricted runs (`--dry-run`, `--pages`) write a separate
`plutarch.dryrun.tags.json` so they never pollute the committed cache.

### Cost and how to sanity-check it

The transcription path is a one-time transcription with a tight budget (the facsimile
path only pays this for the front matter). Before the full run, measure on a
contiguous body-page sample against a **fresh** dryrun sidecar, e.g.:

```sh
cargo run -- --v1 … --v2 … --out build/ --pages v1:200-260 --retag
```

Read the per-model token totals from the printed usage report and apply the current
reference prices by hand, **halved** for the batched Sonnet / Haiku / Opus stages
(classification is synchronous and full price): at the time of writing, per MTok
input/output, Haiku $1/$5, Sonnet $3/$15, Opus $5/$25 — verify these against current
pricing before trusting the estimate. Extrapolate to the full ≈ 1,328-page work. If
that exceeds budget, the cost levers in order are: set `DIVERGENCE_SIGNAL = false`
(drops the Haiku pass — the largest single saving), lower `ARBITRATION_EFFORT` to
`"low"`, then raise `SIMILARITY_THRESHOLD` so only large disagreements escalate. See
`llm_artifacts/2026_10_03_COST_GATE_FINDINGS.md`.

---

## Dependencies

| Tool | Purpose | Override env var |
|------|---------|------------------|
| Rust / Cargo | build the tool | — |
| poppler `pdftoppm`, `pdfinfo` | rasterize source pages, read page counts | `PLUTARCH_PDFTOPPM`, `PLUTARCH_PDFINFO` |
| poppler `pdftotext` | read the embedded OCR layer for facsimile unit detection | `PLUTARCH_PDFTOTEXT` |
| MiKTeX `lualatex` + the **Old Standard** font (`oldstandard` package) | compile the LaTeX | `PLUTARCH_LUALATEX` |
| Tesseract with the `lat` language model | OCR cross-check (transcription path) | `PLUTARCH_TESSERACT` |

Each tool is found on `PATH` by default; point the matching env var at an explicit
binary if it is elsewhere. Old Standard is loaded **by file name** (the
`OldStandard-*.otf` files), so it need only be installed in the TeX tree — run
`mpm --install=oldstandard` if lualatex cannot find it.

An Anthropic API key is needed **only** for AI vision: the whole transcription path,
and the one-time front-matter transcription of the facsimile path. The facsimile body
build needs no key. Put the key in `secrets.json` in the project root:

```json
{ "anthropic_api_key": "sk-ant-…" }
```

See `secrets.example.json`. This file is **local only** — it is not shared.

## Output and the two-stage compile

For each volume both paths write, into `--out`:

- `tomus_N_inner.tex` — the trim-size inner book pages.
- `tomus_N.tex` — the imposition wrapper: US Letter for transcription and, by
  default, facsimile; A4 when facsimile uses `--paper-size a4`.

With lualatex available they compile both: the inner once (it self-pads to a whole
32-page signature), reads the real page count from the lualatex log, then emits and
compiles the wrapper, which pulls inner pages two-up via
`\includegraphics[page=N]{tomus_N_inner.pdf}` in signature-folding order. Without
lualatex (or with `--no-compile`) it writes the wrappers from an estimated page count
and prints the manual compile commands. Fold lines and sew marks are drawn on each
outer/recto sheet side only; a cut border on the first recto of each signature.

## Files

- `facsimile_units.json` — the auto-detected, committed, hand-editable unit table for
  the facsimile path (`--detect-units` / `--redetect`; `pinned` rows preserved).
- `plutarch.frontmatter.tags.json` — the committed, hand-editable cache of the
  re-typeset front matter (shared intent with `plutarch.tags.json`).
- `plutarch.tags.json` — the committed transcription cache. Hand-editable: correct a
  page `role` or a transcription directly and re-run.
- `build/`, `build_facsimile/`, `target/`, `secrets.json` — local, not shared.
- `spikes/` — throwaway layout and detection experiments kept as a record.
- `llm_artifacts/` — the implementation plans and notes, kept as the dated record of
  why the code is shaped as it is.

There is no git; the project lives on OneDrive, which provides file history. The
committed caches above are kept in the project tree and hand-corrected in place.
