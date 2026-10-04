# Facsimile (image-paste) reader — a new build path

**Authoring date:** 2026-10-03

## Context

The transcription pipeline (`plutarch-reader`) re-typesets the Latin column from
AI-vision transcriptions. The Phase-5 cost gate
(`2026_10_03_COST_GATE_FINDINGS.md`) showed that path sits on the owner's $35
ceiling, carries a ~13% per-run transcription-failure rate, and has an unresolved
verse-LaTeX bug — and, more fundamentally, every transcribed character is a place
the stored Latin could silently drift from the page.

This plan adds a **second, independent build path** that sidesteps transcription of
the body entirely: it crops the Latin column out of the source scans, slices it at
inter-line whitespace, and reflows those **image strips** into the Loeb trim format
— like Jefferson gluing passages into his Bible. The reproduction *is* the original
1857 Didot type, pixel-for-pixel, so faithfulness is guaranteed by construction and
the body costs nothing to produce. Only the **front matter** (Monitum, Praefatio,
title pages) still needs vision, because it is full-width prose that must be
re-typeset to the narrow trim measure.

A proof-of-concept spike (`spikes/facsimile_reflow_spike.py`) validated the core
mechanism end to end on a real page (v1 p210, *Pericles*):

- Per-page Latin-column detection via a vertical ink-projection profile cleanly
  separated the column from the Greek and **excluded the marginal reference letters**
  (they sit in the gutter across a whitespace gap from the text margin).
- A horizontal ink-projection profile found every inter-line whitespace band; cuts
  landed between lines, never clipping glyphs.
- The source Latin measure is ~2.66 in; the trim text block is ~3.3 in, so strips
  scale **up ×1.26** — type gets *more* legible, not smaller. The reflowed trim page
  reads like a real book page.
- Source furniture (running head, page number, "Digitized by Google" watermark) was
  removed with top/bottom position zones.

Three source-page archetypes were confirmed by eye and determine the design:

1. **Body page** (e.g. v1 p210): two columns; running head = Latin/Greek Life name +
   folio; marginal section-reference letters in the gutter; inline Roman chapter
   numbers and parenthetical `(n)` section numbers set into the Latin column.
2. **Life-opening page** (e.g. v1 p14, *Theseus*): a one-time full-width **work
   title** ("PLUTARCHI PARALLELA SEU VITÆ PARALLELÆ") on the very first page; the
   **Life title** ("THESEUS.") centered *within* the Latin column; an edition
   cross-reference ("Ed. Wech. pag. 1, 2."); a foot signature ("PLUTARCHUS. I.").
3. **Front-matter page** (e.g. v1 p13, Monitum): single-column, **full-width**,
   mixed Latin + Greek errata — not column-croppable; must be transcribed and
   re-typeset.

**Owner layout decisions (binding on this plan):**
- Single column per trim page (the two-column idea is dropped — the enlarged single
  measure reads well).
- A **typeset Latin running head**, centered at the top of every trim page (the Life
  / Comparatio name) — *not* the pasted source running head.
- **Per-Tomus page numbers** typeset at the bottom of every page, as the current
  reader already does.
- Background speckle normalization is **not** done; light scan grain is acceptable.

**Project conventions (from `CLAUDE.md`), binding on every step:** no git (OneDrive
holds history; `target/` is redirected out of the synced tree via
`.cargo/config.toml`); transcribed/front-matter Latin is verbatim source text, never
normalized; quality gates `cargo fmt --all --check`, `cargo clippy --all-targets --
-D warnings`, and `cargo test` must all pass locally before a step is done. This is a
**new build path that reuses or duplicates existing code — it does not edit the
existing transcription pipeline's behavior.**

## Current state of the world

`plutarch-reader` is one crate: a library plus a `main.rs` binary. Relevant pieces
this plan reuses (and must not regress):

- **`src/render.rs`** — `render_page(pdf, page, dpi, out_dir)` (poppler `pdftoppm`,
  OneDrive-hydration retries), `crop_right_column(png_bytes, left_frac)` and
  `render_latin_column(...)` (the fixed-fraction crop used by transcription),
  `CROP_LEFT_FRAC = 0.48`, `TRANSCRIBE_DPI = 300`, `Pdf::open`, `hash_bytes`,
  `file_hash`. The `image` crate (v0.25, `png` feature) is already a dependency.
- **`src/latex.rs`** — `write_inner_volumes(sidecar, plans, out, fonts,
  front_image_closure)` emits each volume's inner `.tex`. `inner_preamble(fonts)`
  (private) holds the page geometry
  (`paperwidth=10.5cm,paperheight=16cm,top=1cm,bottom=1.5cm,inner=1.2cm,outer=0.9cm,footskip=0.8cm`
  → `\textwidth = 8.4 cm`), the Old Standard font setup, and the `folio` page style.
  **The current reader has no running head**: `folio` does `\fancyhf{}` then sets
  *only* `\fancyfoot` (the per-Tomus `\thepage`, roman in front matter, arabic from
  the first body page via `\beginbodynumbering`). Life names appear only as `\heading`
  display titles at each Life start. Reusable macros: `\freshpage` (start recto),
  `\heading` (centered bold large), `\beginbodynumbering`. `write_imposition_wrappers(
  n, inner_page_counts, out)` emits the two-up US-Letter imposition with fold/sew/cut
  marks; `Fonts`, `VolumePlan`, `front_image_name`. `write_inner_volumes` /
  `inner_document` / `life_block` are body-from-transcription-coupled and are **not**
  reused; the facsimile writes its own inner.
- **`src/compile.rs`** — `compile(out, name, passes)`, `lualatex_available()`,
  `INNER_PASSES`, `inner_page_count(out, stem)` (reads the compiled inner page count
  from the lualatex log).
- **`src/volumes.rs`** — `split_into_volumes(units, TARGET_PAGES, MAX_PAGES)` (balances
  units into volumes, forces a hard break at the v1→v2 source boundary),
  `unit_pages(sidecar, lives)`, `UnitPages { id, group, kind, source, pages }`,
  `volume_physical_pages`, `TARGET_PAGES`, `MAX_PAGES`.
- **`src/assemble.rs`** — `detect_lives(sidecar)` derives `LifeBoundary`s **from
  transcribed `Heading` blocks** — i.e. it depends on body transcription, which the
  facsimile path will not have. The facsimile path therefore needs unit boundaries
  from another source (a hand-authored table; see below).
- **`src/tags.rs`** — `LifeBoundary { id, title, kind, group, first_page, last_page }`,
  `LifeKind { Life, Comparatio }`, `Sidecar`, `PageRole`.
- **`src/pages.rs`** — `classify` / `is_boundary` / `BOUNDARY_SCAN = 25` (Haiku
  boundary classification); the contiguous middle of each source is body.
- **`src/pipeline.rs`**, **`src/llm/transcribe.rs`**, **`src/llm/verify.rs`** — the
  transcription build path. The facsimile path reuses **only** transcription of
  front-matter prose (a dozen-odd pages), not of the body.

There is no facsimile build path, no image-analysis module, and no unit-boundary
table independent of transcription.

## Desired end state

A second binary, `facsimile`, that produces the same imposed per-Tomus US-Letter
PDFs as the existing reader, but whose **body pages are images of the original Latin
column** reflowed into the trim text block under a typeset running head and folio,
and whose **front matter** is re-typeset from vision transcription. Concretely:

- `cargo run --bin facsimile -- --v1 … --v2 … --out build_facsimile/` emits, per
  Tomus, `tomus_N_inner.pdf` (trim-size facsimile pages) and the imposed `tomus_N.pdf`
  wrapper, reusing the existing imposition and compile code unchanged.
- **Body** of each Tomus: for every source body page in a unit's range, the Latin
  column is detected, furniture removed, sliced into line-strips, and the strips are
  reflowed (at a single global scale) into trim pages whose text block carries only
  the body type. Each page gets a **typeset centered Latin running head** (the unit's
  Latin name) and a **per-Tomus folio** at the foot.
- **Atomicity**: each Life/Comparatio starts on a fresh page (recto where the signature
  math allows), driven by a hand-authored **unit table**, not by transcription.
- **Life-opening display title** is typeset from the unit table (not pasted); the
  one-time work title becomes a typeset half-title; source edition cross-refs,
  running heads, folios, marginal letters, and the Google watermark are all excluded
  from the paste.
- **Front matter** (v1 Monitum, v2 Praefatio, derived title pages) is produced by
  reusing the existing transcription of those few `FrontProse` pages plus the
  existing front-matter LaTeX typesetting, prepended to the first Tomus of each source.
  The Monitum and Praefatio transcriptions are **cached** in a committed front-matter
  sidecar (`plutarch.frontmatter.tags.json`) and **never re-run** on a subsequent
  build: a build re-transcribes only when that sidecar is missing, a source file hash
  changed, or `--retag` is passed. That cache is the single source of truth for the
  front-matter text — human proofreading corrections are edited into it, not
  regenerated.
- **Volume splitting** reuses `split_into_volumes` with per-unit **facsimile** page
  counts computed by the reflow step.
- Re-runs are offline and deterministic (rendering + image analysis are pure functions
  of the source PDFs and the unit table); only front-matter transcription touches the
  network, and only on the first build or when the front-matter sidecar is
  missing/stale/`--retag`.

The existing `plutarch-reader` binary and its tests are unchanged.

## Tech Learnings

- **Projection profiles.** A page's structure is read from 1-D sums of "ink"
  (dark pixels) across an axis. The **vertical** profile (ink per column) has deep
  valleys at the margins and the Greek/Latin gutter and plateaus over each text
  column; the Latin left margin is the sustained rise *after* the gutter, which skips
  the sparse marginal letters. The **horizontal** profile (ink per row) is near-zero
  in inter-line gaps and high over glyph rows, giving clean line bands. Thresholds are
  fractions of the profile's max; the spike used gutter-min search in the central
  40–62% of width, a left-margin back-off at 0.20·max, and furniture zones at the top
  6.5% / bottom 5% of page height. Port these to Rust with the `image` crate
  (`image::load_from_memory`, `GrayImage`, pixel iteration, `crop_imm`,
  `DynamicImage::resize` with `FilterType::Lanczos3`, re-encode PNG).
- **Global scale + fixed-width canvases.** Column width drifts a few percent page to
  page (binding curvature, scan). Scaling each page to fill the trim measure would make
  type size breathe visibly. Instead pick **one global scale** for the whole work —
  map a robust column width (e.g. the 95th-percentile detected width, so no line
  overflows the measure) to the trim text-block width — and left-align strips; the
  ragged right is already how the source sets its lines. **Crucially, every composed
  body canvas must have the same pixel width** (the global measure). Because the inner
  `.tex` includes each body image with `\includegraphics[width=\textwidth]`, LaTeX
  rescales each image to `\textwidth`; if canvases had different widths, type size
  would breathe at the LaTeX layer and defeat the global scale. Canvas height is ragged
  (last page of a unit), so include with `width=\textwidth` (height auto) and top-align.
- **Images in a LaTeX text block, and a parameterized preamble.** The facsimile inner
  places one composed body image per trim page as `\includegraphics[width=\textwidth]
  {body-page-N.png}` inside the page frame (one image per page; lualatex paginates
  nothing within a page). The frame — geometry, the `folio` style, `\freshpage` —
  comes from `inner_preamble`, but the **centered Latin running head is new** (the
  current reader has none) and needs reserved head space (`includehead` + a
  `headheight`/`headsep`), which the current `top=1cm` geometry does not provide.
  Rather than duplicate the preamble (CLAUDE.md disfavors duplication), **parameterize
  it**: `inner_preamble(fonts, opts)` where `opts` toggles the running head and the
  head geometry. The existing caller passes `opts` that reproduce today's output
  byte-for-byte (guarded by a golden test); the facsimile passes head-on. The
  `\thepage` folio is unchanged, so per-Tomus page numbers work exactly "as before."
- **Why a hand-authored unit table.** `detect_lives` reads transcribed headings, which
  the facsimile body won't have. There are ~46 Lives + ~19 Comparationes ≈ a few dozen
  units in a fixed, well-documented order; a hand table of `(id, latin_title,
  greek_title, kind, group, source, start_page)` is a one-time, auditable, fully
  deterministic input. End pages derive as "next unit's start − 1" (last unit ends at
  the source's last body page).
- **Hybrid faithfulness.** Body = verbatim pixels (no gate needed). Front matter =
  transcription, which *is* under the faithfulness gate — so the Monitum/Praefatio
  reuse the existing verbatim-preserving prompts and are human-proofread, exactly as
  the transcription path intends.

---

## Phase 1: Scaffolding — binary, unit table, render reuse

**Suggested implementer: Opus.** Sets the module boundaries and the data model the
rest of the plan builds on; requires judgment about what to reuse vs duplicate.

### Step 1.1 — New binary and module skeleton

- Add `src/bin/facsimile.rs` (a `clap`-derived CLI mirroring `main.rs`: `--v1 --v2
  --out --no-compile`, plus `--pages`/`--unit <id>` for cheap single-unit test runs).
  It calls a new `plutarch_reader::facsimile::run_facsimile(args)`.
- Add `src/facsimile/mod.rs` (declared `pub mod facsimile;` in `lib.rs`) with
  submodules created empty-but-compiling: `units`, `columns`, `lines`, `reflow`,
  `assemble`. `run_facsimile` for now loads the unit table and renders the first page
  of the first unit to prove the wiring, then returns.
- Reuse `render::render_page` and `render::Pdf`; **do not** modify `render.rs`.

**Tests:** `facsimile_run_args_parse` (CLI parsing); `run_facsimile_smoke` (with
`--no-compile` and a stub unit range, exits Ok without network).
**LLM verification:** `cargo build --bins`; `cargo test facsimile`.
**Human verification:** None.

### Step 1.2 — Unit table format and loader

- Define `facsimile::units::Unit { id: String, latin_title: String, greek_title:
  String, kind: LifeKind, group: u32, source: usize, start_page: u32, skip_top_bands:
  Option<u32> }` and `UnitRange { unit: Unit, end_page: u32 }`. `skip_top_bands` is a
  per-unit override for how many leading line-bands to drop on the unit's **first**
  source page (work title / Life title / edition cross-ref); `None` falls back to the
  heuristic in Step 3.2.
- `pub fn load_units(path: &Path) -> Result<Vec<Unit>>` reading a hand-authored
  `facsimile_units.json` — **JSON, not TOML**, so no new dependency is added
  (`serde_json` is already in `Cargo.toml`, and the project already hand-edits JSON
  sidecars). Validated: strictly increasing `start_page` within a source, v1 units
  before v2, non-empty titles. The `group` integers must match the pairing
  `split_into_volumes` enforces (a Life pair and its Comparatio share one group so the
  splitter never bisects them).
- `pub fn ranges(units: &[Unit], source_last_body_page: &[u32]) -> Vec<UnitRange>`
  deriving `end_page = next unit's start_page − 1` (last unit per source → that
  source's last body page).
- Seed `facsimile_units.json` with the first few real units (Theseus @ v1 p14,
  Romulus, Comparatio Thesei et Romuli, …) — enough for a dry run; the full table is
  a human deliverable completed during Phase 5 verification.

**Tests:** `load_units_parses_and_validates`; `load_units_rejects_unsorted`;
`ranges_derives_end_pages`; `ranges_last_unit_runs_to_source_end`.
**LLM verification:** `cargo test units`.
**Human verification:** Confirm the seeded unit rows match the real source pages
(spot-check Theseus @ p14 renders the Theseus opening).

---

## Phase 2: Deterministic image analysis (column + lines)

**Suggested implementer: Opus.** Image algorithms ported from the spike; correctness
hinges on threshold judgment and robust edge cases.

### Step 2.1 — Latin-column detection

In `src/facsimile/columns.rs`, port the spike's vertical-projection logic to Rust
over a `GrayImage`:

- `pub struct Column { pub left: u32, pub right: u32 }`.
- `pub fn ink_profile_cols(img: &GrayImage, thr: u8) -> Vec<u32>` (dark-pixel count
  per column) and a small smoothing helper.
- `pub fn detect_latin_column(img: &GrayImage) -> Result<Column>`: gutter = min-ink
  column in the central 40–62% of width; left edge = sustained rise past the gutter
  (back off left of a firmly-dense column while ink > 0.20·max) so marginal letters
  are excluded; right edge = inward from the page's right until ink > 0.18·max.
- `pub fn crop_column(img: &GrayImage, c: &Column) -> GrayImage`.

**Tests (synthetic `GrayImage`s, no PDF):**
`detect_column_finds_right_block_of_two`; `detect_column_excludes_sparse_left_marks`
(a few isolated dark cells in the gutter are not included);
`detect_column_handles_width_drift` (shifted column still detected).
**LLM verification:** `cargo test columns`.
**Human verification:** Crop v1 p210 and p14 and confirm the Latin column is tight,
Greek excluded, marginal letters gone.

### Step 2.2 — Line-band detection and furniture removal

In `src/facsimile/lines.rs`:

- `pub struct Band { pub top: u32, pub bottom: u32 }`.
- `pub fn line_bands(col: &GrayImage, min_ink: u32) -> Vec<Band>` (contiguous runs of
  texty rows via the horizontal profile).
- `pub fn drop_furniture(bands: &[Band], page_h: u32) -> Vec<Band>`: keep only bands
  whose center is within the body zone (top 6.5% / bottom 5% excluded), then drop a
  leading/trailing band separated from the body by an abnormally large gap (running
  head / watermark belt-and-suspenders).
- `pub fn cut_points(bands: &[Band]) -> Vec<u32>` (band tops/bottoms and gap
  midpoints) so a reflow break never bisects a line.

**Tests (synthetic):** `line_bands_splits_on_gaps`;
`drop_furniture_removes_top_and_bottom_zones`; `cut_points_lie_in_gaps`
(every cut is in a zero-ink row).
**LLM verification:** `cargo test lines`.
**Human verification:** Overlay cut lines on the v1 p210 column (as the spike's
`out_col_cuts.png`) and confirm each lands in a gap.

---

## Phase 3: Reflow and body-page composition

**Suggested implementer: Opus.** The reflow packing, the global-scale decision, and
the opening-page title/furniture handling all require judgment.

### Step 3.1 — Global scale and strip extraction

In `src/facsimile/reflow.rs`:

- `pub struct TrimGeom { … dpi }` whose text-block dimensions are derived from the
  **same geometry `inner_preamble` uses** — `paperwidth=10.5cm,paperheight=16cm,
  inner=1.2cm,outer=0.9cm` → `\textwidth = 8.4 cm`; `top`, `bottom=1.5cm`,
  `footskip=0.8cm`, plus the **new head space** reserved for the running head (see
  Step 4.1) → the body text-block height. Expose text-block width/height in px at the
  body DPI (default 300 → `tb_w ≈ 992 px`). Do not invent symmetric 1 cm margins (the
  spike's approximation); pin to the real geometry so the composed image fills
  `\textwidth` at exactly the intended scale.
- `pub fn global_scale(col_widths: &[u32], tb_w_px: u32) -> f32`: map the
  95th-percentile column width to the trim text-block width, so no line overflows.
- `pub struct Strip { pub page_img_index: usize, pub band: Band }`; a unit's body is a
  `Vec<Strip>` in reading order across its pages (Step 3.2 fills page images).

**Tests:** `global_scale_maps_p95_to_measure`;
`global_scale_never_overflows_widest` (widest line ≤ measure after scaling).
**LLM verification:** `cargo test reflow`.
**Human verification:** None.

### Step 3.2 — Compose per-trim-page body images

- `pub fn unit_body_strips(ranges, render_fn) -> Result<Vec<(GrayImage, Vec<Band>)>>`:
  for each source page in the unit range, render (reuse `render_page`), detect the
  column, crop, band, and drop furniture. On the unit's **first page** additionally
  skip the leading title/edition-cross-ref bands: drop `skip_top_bands` leading bands
  when the unit table sets it, else fall back to the heuristic "drop bands above the
  first body-width line." The title is typeset from the table in Phase 4, so these are
  never pasted. (The heuristic is human-verified; `skip_top_bands` is the escape hatch
  for openings it gets wrong.)
- `pub fn compose_pages(strips, scale, geom) -> Vec<GrayImage>`: pack scaled strips
  top-to-bottom, breaking only at cut points, onto **fixed-width** canvases — every
  canvas is exactly `tb_w` px wide (so `\includegraphics[width=\textwidth]` keeps type
  size uniform across pages); strips left-aligned, inter-line spacing preserved, canvas
  height ragged on a unit's last page. Returns one body image per trim page.
- Write each body image to `--out` as `facsimile_<unit>_<n>.png`; the count per unit
  is the unit's **facsimile page count** (fed to volume splitting in Phase 4).
- A failed render/detection on a page is **counted and reported**, never silently
  dropped; the page is skipped with a warning so the run surfaces it (mirrors the
  transcription path's no-silent-drop rule).

**Tests (synthetic strips):** `compose_breaks_only_at_cut_points`;
`compose_fills_page_then_wraps`; `compose_canvases_share_fixed_width`;
`first_page_excludes_title_and_crossref_bands`;
`skip_top_bands_override_drops_exact_count`.
**LLM verification:** `cargo test reflow`.
**Human verification (critical):** Build the body images for Theseus (v1 p14→) and
Pericles (v1 p210 region) and eyeball: titles excluded from paste, no furniture, no
clipped lines, consistent type size across pages.

---

## Phase 4: Page frame, volume split, imposition

**Suggested implementer: Opus.** Must parameterize and reuse the existing LaTeX page
frame and the volume/imposition machinery without regressing the transcription path.

### Step 4.1 — Facsimile inner `.tex` with a new running head and the reused folio

- **The current reader has no running head** (the `folio` style sets only
  `\fancyfoot`), so the centered Latin running head is **new**. It also needs reserved
  head space, which the current `top=1cm` geometry does not provide. **Parameterize
  the existing preamble** rather than duplicate it (CLAUDE.md disfavors duplication):
  change `inner_preamble(fonts)` → `inner_preamble(fonts, opts)` where
  `opts: InnerOpts { running_head: bool }`. When `running_head` is true the preamble
  (a) adds `includehead` and a `headheight`/`headsep` to the `geometry` options, and
  (b) adds `\fancyhead[C]{...}` to the `folio` style driven by a `\markboth`/`\markright`
  set at each unit's start; the folio foot is unchanged. The existing
  `inner_document` caller passes `InnerOpts { running_head: false }`, which must
  reproduce today's output **byte-for-byte** (golden test). The running head is
  suppressed on title/flyleaf pages via `\thispagestyle{empty}` as those pages already do.
- `facsimile::assemble::write_inner(volume_plan, unit_page_images, out, fonts)` emits
  `tomus_N_inner.tex` using the parameterized preamble with `running_head: true`: for
  each unit, set the running head (via `\markboth`) and emit the reused `\heading`
  display title (Latin + Greek from the table) on a `\freshpage`, then one
  `\includegraphics[width=\textwidth]` page per composed body image. Reuse
  `\beginbodynumbering` so the body folio restarts at 1 per Tomus exactly as today.
- **The facsimile inner must honor the same structural contract the imposition
  assumes**, or the reused fold/sew/cut placement drifts: `sheet_sides` maps odd pages
  to rectos, so the inner must (a) open and close with the end **flyleaves**
  (`FLYLEAVES_PER_END`, an even leaf count → preserves verso/recto parity), (b) start
  each unit/title on a recto via `\freshpage`, and (c) **pad to a whole 32-page
  signature** with a blank final verso at document end. Reuse the existing pieces for
  this — `latex::FLYLEAVES_PER_END` (already `pub`), and make `latex::blank_pages` and
  the `SIGNATURE_PADDING` constant `pub(crate)` so the facsimile inner emits the same
  flyleaves and the same end padding as `inner_document`, rather than re-deriving them.

**Tests:** `inner_preamble_running_head_off_unchanged` (golden: existing
`inner_document` output is byte-identical to the pre-change output);
`inner_preamble_running_head_on_reserves_head_space`;
`inner_tex_sets_running_head_per_unit`;
`inner_tex_includes_one_image_per_page`; `inner_tex_starts_each_unit_fresh_page`;
`facsimile_inner_emits_flyleaves_and_signature_padding` (the facsimile inner carries
the same `FLYLEAVES_PER_END` flyleaves and end-padding structure as `inner_document`).
**LLM verification:** `cargo test latex facsimile`.
**Human verification:** Read a generated `tomus_1_inner.tex`; confirm the running head
is the Latin Life name and the per-Tomus folio is present; diff the existing reader's
`tomus_1_inner.tex` against a pre-change copy to confirm it is unchanged.

### Step 4.2 — Volume splitting and imposition reuse

- Build `UnitPages { id, group, kind, source, pages }` for each unit using the
  **composed facsimile page count** from Phase 3, then reuse
  `volumes::split_into_volumes(TARGET_PAGES, MAX_PAGES)` and the existing
  `VolumePlan` construction (v1/v2 never share a volume).
- Compile each inner (reuse `compile::compile` / `inner_page_count`), then reuse
  `latex::write_imposition_wrappers` and compile the two-up wrappers — unchanged.
- With `--no-compile`, write wrappers from the computed page counts and print the
  manual compile commands (mirror `pipeline.rs::write_wrappers_from_estimate`).

**Tests:** `facsimile_units_split_into_volumes` (computed counts → balanced volumes,
v1→v2 boundary respected); `facsimile_imposition_wrappers_written`.
**LLM verification:** `cargo test facsimile volumes`; `cargo build --bins`.
**Human verification (critical):** With lualatex available, compile one Tomus and open
`tomus_1.pdf`: body is the original type, enlarged and legible; running head + folio
correct; fold/sew/cut marks present; each Life starts fresh.

---

## Phase 5: Front-matter hybrid, unit table, end-to-end

**Suggested implementer: Opus.** Integrates the one remaining vision dependency and
the full-work run; needs the complete hand-authored unit table and real-eyes proofing.

### Step 5.1 — Front-matter via cached transcription + LaTeX

- There is no front-matter-only `Scope` in `pipeline::run`, and `run` always
  transcribes the body too, so **do not** call `pipeline::run`. Instead the facsimile
  front-matter step reuses the stage functions directly: it loads/initializes a
  dedicated committed sidecar `plutarch.frontmatter.tags.json`, runs `pages::classify`
  over the boundary regions, then `transcribe_all` / `reconcile_all` with an
  `in_scope` closure that admits **only `FrontProse`/`FrontImage`** pages (body pages
  are never requested). It builds its own `Client` / `ApiVision` / `ApiBatchBackend`
  (as `main.rs` does), lazily, only when front-matter work remains.
- **Caching (required).** `plutarch.frontmatter.tags.json` is a committed cache
  (OneDrive history, hand-editable) reused across every build with the same semantics
  as `plutarch.tags.json`: on load, if present and not stale (`Sidecar::is_source_stale`
  over the source file hashes) and the `FrontProse` `result`s are filled, the build
  makes **zero** network calls. The Monitum and Praefatio are therefore transcribed
  **once**; later builds read the cache. Re-transcription happens only when the
  sidecar is missing, a source hash changed, or `--retag` is passed. That cache is the
  single source of truth — human proofreading corrections are edited into it (per the
  CLAUDE.md faithfulness rule), never regenerated over.
- Reuse `latex::title_page` / `front_images` / `front_prose` (via a thin wrapper, or by
  reading the same `FrontProse` `result`s) to typeset the front matter to the trim
  measure, prepended to the first Tomus of each source. `front_matter` bib strings
  (title/editor/date) are hand-filled in the sidecar (the pipeline leaves them
  `Default`). No change to the transcription prompts or the faithfulness gate.

**Tests:** `frontmatter_only_transcribes_frontprose` (an `in_scope` that rejects body
pages → zero body requests through a stub `BatchBackend`);
`frontmatter_offline_when_sidecar_fresh` (a complete, non-stale front-matter sidecar →
no vision/backend is constructed, zero calls).
**LLM verification:** `cargo test facsimile`.
**Human verification:** Proofread the typeset Monitum and Praefatio against the source
pages (p13 etc.); confirm verbatim fidelity and that Greek fragments are intact; run a
second build and confirm it makes no network calls (cached).

### Step 5.2 — Unit table (auto-detected)

The unit table is **not hand-authored**. It is auto-detected from the source PDFs'
embedded OCR layer plus a built-in canonical corpus of the Parallel Lives, as
specified and implemented in `2026_10_03_FACSIMILE_UNIT_AUTODETECT_PLAN.md`.
`facsimile_units.json` is a committed, hand-editable **cache**: `--detect-units`
(or `--redetect`) regenerates it, a normal build auto-detects when it is absent, and
rows marked `"pinned": true` survive regeneration. Each row's `(id, latin_title,
greek_title, kind, group, source, start_page)` comes from the corpus and detected
openings; the final unit of each source also carries a detected `last_page` that
excludes the back-matter/library pages. See that plan for the detector, corpus, and
per-step verification.

### Step 5.3 — README

Update `README.md` **in place** (rewrite affected text so the document reads as if
written today — no "update"/"revision" blocks) to document the project's **two build
paths**:

- The **facsimile** binary (`cargo run --bin facsimile -- …`): body pages are the
  original 1857 Didot Latin column, cropped and reflowed (repaginated whole source
  lines at a single global scale) into the trim text block under a **typeset centered
  Latin running head** and the per-Tomus folio; each Life/Comparatio starts fresh,
  driven by the hand-authored `facsimile_units.json` table (with `skip_top_bands`
  overrides); front matter (Monitum, Praefatio, title pages) is transcribed **once**
  and cached in `plutarch.frontmatter.tags.json` (re-run only on `--retag`/stale); the
  imposition, signatures, fold/sew/cut marks, and two-stage compile are the same as the
  transcription path.
- How the two paths differ and when to use each: facsimile = verbatim pixels,
  ~zero body cost, non-searchable, image-heavy PDFs; transcription = re-typeset,
  selectable text, per the cost gate. Point to
  `2026_10_03_FACSIMILE_READER_PLAN.md` and `2026_10_03_COST_GATE_FINDINGS.md`.
- The facsimile dependencies (poppler + lualatex; **no** API key needed except for
  the one-time front-matter transcription) and the `facsimile_units.json` format.

Add the facsimile binary to the "Running" and "Files" sections and note that
`facsimile_units.json` and `plutarch.frontmatter.tags.json` are committed, hand-editable
caches (OneDrive history), like `plutarch.tags.json`.

**LLM verification:** none (docs).
**Human verification:** Follow the README's facsimile instructions on a fresh checkout
and confirm a `--unit theseus --no-compile` run and a full Tomus compile both work as
written.

---

## Cross-cutting requirements

- Every Rust-touching step ends green on `cargo fmt --all --check`, `cargo clippy
  --all-targets -- -D warnings`, and the relevant `cargo test`; full `cargo test` is
  green before a phase is declared done. No CI.
- **The existing transcription binary and its output are not changed.** Shared code is
  reused by *parameterizing one definition* (with a golden "output unchanged" test) in
  preference to duplication; duplication is a last resort per CLAUDE.md. The existing
  transcription path's behavior and emitted `.tex` must stay byte-identical.
- Body reproduction is verbatim pixels: the column crop must not clip Latin (human-
  verified), source furniture (running head, folio, edition refs, marginal letters,
  watermark) and source titles are excluded, and the typeset running head / folio /
  display title are the only generated text on a body page. Front-matter Latin stays
  verbatim under the existing gate.
- No page is silently dropped: a failed render/detection is counted and warned.
- Re-runs are offline and deterministic except front-matter transcription, which runs
  only when `plutarch.frontmatter.tags.json` is missing/stale or `--retag` is passed;
  the cached Monitum/Praefatio transcriptions are never re-run on an ordinary build.
- Composed body-page PNGs are ~300-DPI full-text-block images, so inner PDFs are
  image-heavy and larger than the transcription path's text PDFs — expected, acceptable.
- No code comment references this plan, its phases, or `llm_artifacts/`.
- Imposition, signature folding, fold/sew/cut marks, and the two-stage compile are
  reused unchanged.
