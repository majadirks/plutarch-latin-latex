# plutarch-reader

Generates a Loeb-size (10.5 × 16 cm), Latin-only reader of Plutarch's *Lives* in
the Didot bilingual recension (Dœhner, Paris 1857) from the two Google Books scans,
and imposes it onto US Letter PDFs ready to print double-sided, fold into 32-page
signatures, sew, and trim to pocket size.

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
want re-typeset, selectable Latin and the budget allows. Both emit per-volume LaTeX
and imposed US Letter PDFs.

---

## The facsimile path

```sh
# Detect the unit table from the sources and write facsimile_units.json, then exit.
cargo run --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf \
    --out build_facsimile/ --detect-units

# Full build (auto-detects the table on first run if it is absent).
cargo run --bin facsimile -- --v1 … --v2 … --out build_facsimile/

# Re-detect the table even when the cache is present (keeps pinned hand edits).
cargo run --bin facsimile -- --v1 … --v2 … --out build_facsimile/ --redetect

# A single unit, or a raw source page range, for a cheap test.
cargo run --bin facsimile -- --v1 … --v2 … --out build_facsimile/ --unit theseus --no-compile
cargo run --bin facsimile -- --v1 … --v2 … --out build_facsimile/ --pages v1:14-45 --no-compile

# Purge the raster cache first, forcing every page to be re-rasterized.
cargo run --bin facsimile -- --v1 … --v2 … --out build_facsimile/ --clean-cache
```

**Body.** Each page's Latin column is detected, sliced at inter-line whitespace, and
reflowed (whole source lines repaginated at one **global** scale) into the trim text
block, under a **typeset centered Latin running head** and the per-Tomus folio.
Margins are inner / outer 1 cm, 1.5 cm above the running head, and 1.5 cm below the
text. Each Life and Comparatio **opens fresh**, with its display title typeset from
the unit table; Comparationes follow their pair.

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

**Front matter** is the one place the facsimile path uses vision: the v1 Monitum, v2
Praefatio, and derived title pages are full-width prose re-typeset to the trim
measure. They are transcribed **once** and cached in `plutarch.frontmatter.tags.json`
(same semantics as the body cache below); later builds read the cache and make zero
network calls. `--retag` or a changed source re-runs the transcription. Only
`FrontProse` pages are ever sent — no body page is. If no API key is configured on a
first run, front-matter transcription is skipped with a warning and the body build
still completes.

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
   Letter wrapper carrying fold lines, sew marks, and cut borders.

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
- `tomus_N.tex` — the US Letter imposition wrapper.

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
