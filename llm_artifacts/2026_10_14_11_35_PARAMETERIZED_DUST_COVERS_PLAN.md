# Parameterized Dust-Cover Generator

## Context

The facsimile build (`cargo run --bin facsimile`) produces one printable inner
book block per Tomus (`build_facsimile/tomus_N_inner.pdf`) plus an imposition
wrapper that lays the inner pages onto 11"×17" (or A4) printer sheets with cut,
fold, and sewing guides. It does **not** yet produce a *dust jacket* (the paper
cover that wraps a hardbound book: two end flaps, a back panel, a spine, and a
front panel printed on one wide sheet and folded around the boards).

This plan adds a **standalone Python script** — not a Rust binary — that emits a
single parameterized dust-cover sheet for one Tomus: a LaTeX source file and its
compiled PDF, laid out `[Flap][Back Cover][Spine][Front Cover][Flap]` with a
printed cut rectangle so the operator can trim the jacket to its finished size.

A Python script (rather than extending the Rust crate) is deliberate: cover
geometry is simple arithmetic, the output is a short self-contained LaTeX file,
and the project already keeps its auxiliary tooling in `tools/*.py`. The script
depends on a prior facsimile build only to read that Tomus's *list of Lives* and
its title-page block from the already-generated `tomus_N_inner.tex` — the same
artifact a binder would have in hand when printing a jacket for a finished book.

## Current state of the world

Relevant existing code and artifacts:

- **`src/latex.rs`**
  - `tomus_title_page(tomus, life_latin_titles, exlibris)` (lines ~457–515)
    emits the inner book's **Tomus title page**. Its visible body is a single
    `\begin{center} … \end{center}` block containing, in order:
    - `{\LARGE\bfseries PLUTARCHI}\\[0.55em]` then `{\LARGE\bfseries VITÆ}`
    - the fixed attribution block (`A GUILIELMO XYLANDRO`, `LATINE REDDITÆ,`,
      `ATQUE`, `A THEODORO DŒHNERO`, `RECOGNITÆ, EMENDATÆ`, `ET ORDINATÆ.`)
    - `{\large TOMUS <ROMAN>}`
    - a `\parbox{0.86\textwidth}{\centering …}` listing the Tomus's Lives, each
      as `\mbox{\textsc{\MakeLowercase{<LATIN TITLE>}}}`, joined by
      ` \textbullet\ `.
    - The optional `exlibris_plate(name)` is appended **after** the
      `\end{center}` (so the center block itself never contains `EX LIBRIS`).
  - `exlibris_plate(name)` (lines ~517–522) uppercases the owner name and draws
    a bordered `EX LIBRIS` / name plate.
  - `PaperSize { Letter, A4 }` and `WrapperGeometry` (lines ~641–686) define the
    imposition sheet geometry. The imposition wrapper (`wrapper_sheet_macro`,
    lines ~805–829) is the **reference TikZ idiom** this plan mirrors: a
    `tikzpicture` with `[x=1cm,y=1cm]`, `\useasboundingbox`, `\node[anchor=south
    west,inner sep=0] at (x,y) {…}` placement, and
    `\draw[dotted,thick] (…) rectangle (…)` for cut lines.
  - `Fonts::default()` → `main: "OldStandard"`, `size_pt: 10.0`. The inner
    preamble (`inner_preamble`, lines ~163–213) loads it with
    `\setmainfont{OldStandard}[Extension=.otf, UprightFont=*-Regular,
    BoldFont=*-Bold, ItalicFont=*-Italic, BoldItalicFont=*-BoldItalic]`. The
    `.otf` files are not in the repo; `lualatex`/fontspec resolves them via
    kpathsea from the system TeX installation.

- **`src/facsimile/assemble.rs`** (`inner_document`, lines ~75–83) builds the
  Lives list for the title page by filtering the volume's units to
  `LifeKind::Life` only (Comparationes are excluded), then calls
  `tomus_title_page`. So the title page — and therefore the jacket's front and
  back panels — lists **Lives only**, never Comparationes.

- **`src/bin/facsimile.rs`** is the pattern for CLI design: `clap` with a
  `--paper-size` value-enum (`letter`/`a4`), a validated `--exlibris NAME`
  (rejects blank), `--out`, and `--no-compile`.

- **Tomus → Lives mapping is computed at build time**, not static. `plan_volumes`
  (`src/facsimile/mod.rs` ~357–409) calls `volumes::split_into_volumes(…,
  TARGET_PAGES, MAX_PAGES)`, a page-fill that depends on each unit's *composed*
  body-image count. There is **no committed table** mapping a Tomus number to
  its Lives; the authoritative, per-build answer lives in the generated
  `build_facsimile/tomus_N_inner.tex`. The script therefore reads that file.

- **`tools/`** holds the project's Python tooling. Conventions to follow:
  - `argparse`, `pathlib.Path`, module-level functions, `if __name__ ==
    "__main__": main()`.
  - Subprocess calls take an injectable `runner: Runner = subprocess.run`
    parameter so unit tests can pass a fake (see
    `tools/check_portable_tests.py`), letting the whole script be tested without
    a TeX installation.
  - A companion `tools/test_<name>.py` using the stdlib `unittest`.
  - Optional `tools/requirements-<name>.txt` only if third-party deps are
    needed. **This tool needs none** — standard library only (`argparse`, `re`,
    `subprocess`, `pathlib`, `sys`, `dataclasses`, `math`).

- Python available: 3.13. There is no CI; `python -m unittest` must pass
  locally.

## Desired end state

A new script `tools/dust_cover.py` and its test module `tools/test_dust_cover.py`
exist. Running, e.g.:

```
python tools/dust_cover.py \
  --tomus 1 \
  --total-width 15.5 --total-height 9.5 \
  --spine-width 1.25 --cover-width 5.5 \
  --paper-size tabloid \
  --exlibris "Alex Smith" \
  --out build_facsimile
```

produces:

- `build_facsimile/tomus_1_cover.tex` — a single-page landscape LaTeX document.
- `build_facsimile/tomus_1_cover.pdf` — compiled with `lualatex` (unless
  `--no-compile`).

The sheet (default **11"×17" tabloid**, or **A3** via `--paper-size a3`) carries,
centered on it, a jacket laid out left→right:

```
[   Flap   ][  Back Cover  ][ Spine ][  Front Cover  ][   Flap   ]
```

- **Flaps** — blank, each `flap_width` wide. `flap_width = (total_width − 2 ×
  cover_width − spine_width) / 2`, so the whole jacket is exactly `total_width`
  wide: `2·flap + 2·cover + spine = total_width`. Minimum flap width **1.5"**;
  if the computed flap is below that, the script prints an error and exits
  non-zero without writing output.
- **Front Cover** — the Tomus title-page center block lifted verbatim from
  `tomus_N_inner.tex` (PLUTARCHI VITÆ, the attribution, `TOMUS <ROMAN>`, and the
  Lives list), **without** any `EX LIBRIS` plate, vertically centered in a panel
  `cover_width` wide.
- **Spine** — `PLUTARCHI` and `VITÆ` on their own horizontal lines, then `TOMUS
  <ROMAN>`, centered on the `spine_width` strip; if `--exlibris` is given, an
  `EX LIBRIS` line and the uppercased owner name sit at the foot of the spine.
- **Back Cover** — the Tomus's Lives, each on its own line, centered in a panel
  `cover_width` wide.
- A **cut rectangle** (`\draw[dotted,thick] … rectangle …`) is printed at the
  finished jacket boundary (`total_width × total_height`); light dotted **fold
  guides** mark the four panel seams.

Validation, enforced before any file is written:

- `flap_width ≥ 1.5"` (else error).
- `total_width ≤ printable_width` **and** `total_height ≤ printable_height`,
  where `printable = sheet_dimension − 2 × printable_margin` (default margin
  `0.25"`, overridable with `--printable-margin`). The sheet is used in
  landscape, so for tabloid `printable_width = 17 − 0.5 = 16.5"` and
  `printable_height = 11 − 0.5 = 10.5"`.
- The `TOMUS <ROMAN>` found in the parsed `.tex` matches `--tomus N` (guards
  against pointing at the wrong build or Tomus).

`python -m unittest tools.test_dust_cover` (run from the repo root) passes. The
Rust suite is untouched by this work.

The top-level `README.md` gains a `## Dust covers` section documenting the tool,
its full example invocation, and every command-line argument; a companion
`tools/README-dust-cover.md` holds the longer usage notes.

## Tech Learnings

A few building blocks a junior developer will lean on here:

- **TikZ absolute placement.** `\begin{tikzpicture}[x=1in,y=1in]` makes one TikZ
  unit equal one inch. `\useasboundingbox (0,0) rectangle (W,H);` fixes the
  picture's extent to the full sheet so nothing shifts it. A panel of content is
  dropped with `\node[anchor=south west,inner sep=0] at (x,y)
  {\begin{minipage}[c][h][c]{w}…\end{minipage}};` — `anchor=south west` means
  `(x,y)` is the node's lower-left corner. `\draw[dotted,thick] (x0,y0)
  rectangle (x1,y1);` draws the cut box. This is exactly the idiom
  `wrapper_sheet_macro` in `src/latex.rs` already uses for the imposition sheet;
  read it before writing the generator.

- **A fixed-size, vertically centered box.** `\begin{minipage}[c][<height>][c]{<width>}`
  gives a box of an exact width and height with its content vertically centered
  (`[c]…[c]`). The front/back/spine panels each live in one such minipage so
  their content sits centered regardless of length.

- **Small caps via lowercasing.** The house style renders a title as
  `\textsc{\MakeLowercase{THESEUS}}`: `\MakeLowercase` turns the letters
  lowercase so the font's *true* small-cap glyphs are selected, and the small-cap
  font restores cap height. The Lives titles extracted from the `.tex` are
  already written this way; reuse the captured strings verbatim rather than
  re-casing them.

- **Lifting LaTeX verbatim to avoid duplication.** Rather than re-typing the
  PLUTARCHI/VITÆ/attribution strings in Python (which would duplicate
  `tomus_title_page` in `src/latex.rs` and could silently drift from it), the
  front panel reuses the `\begin{center}…\end{center}` block straight out of the
  generated `.tex`. The only Python-authored cover text is the spine, which the
  inner book has no equivalent of.

- **Injectable subprocess runner.** Passing `runner=subprocess.run` as a default
  parameter and calling `runner([...])` lets a test pass a stub that records the
  command and fakes success, so the LaTeX-generation and validation logic is
  unit-testable on a machine with no `lualatex`.

---

## Phase 1 — Script skeleton, CLI, geometry, and validation

*Suggested implementer: Sonnet* — tightly scoped arithmetic and argument
parsing; every name and formula is pinned below.

### Step 1.1 — Create `tools/dust_cover.py` with the CLI and unit constants

Create `tools/dust_cover.py`. Module docstring: one sentence describing the tool
("Generate a parameterized dust-jacket LaTeX sheet + PDF for one facsimile
Tomus."). Define:

- `MIN_FLAP_IN = 1.5`
- `DEFAULT_PRINTABLE_MARGIN_IN = 0.25`
- A sheet table keyed by paper-size name, storing the sheet's **(long_in,
  short_in)** used in landscape:
  - `"tabloid"` → `(17.0, 11.0)`
  - `"a3"` → `(420.0 / 25.4, 297.0 / 25.4)` (≈ `(16.5354, 11.6929)`)
- `argparse` with these arguments:
  - `--tomus` (int, required)
  - `--total-width` (float, required, inches)
  - `--total-height` (float, required, inches)
  - `--spine-width` (float, required, inches)
  - `--cover-width` (float, required, inches)
  - `--paper-size` (choices `["tabloid", "a3"]`, default `"tabloid"`)
  - `--exlibris` (str, optional; validated non-blank, see Step 1.3)
  - `--printable-margin` (float, default `DEFAULT_PRINTABLE_MARGIN_IN`)
  - `--out` (Path, default `Path("build_facsimile")`) — directory to read
    `tomus_N_inner.tex` from and write the cover into.
  - `--inner-tex` (Path, optional) — explicit override for the inner `.tex`
    path; defaults to `<out>/tomus_<N>_inner.tex`.
  - `--no-compile` (store_true) — write the `.tex` but do not run `lualatex`.
- `main(argv=None)` that parses args and (for now) prints the parsed geometry;
  the generation/compile wiring arrives in later phases.

Define a `DustCoverError(Exception)` used for all user-facing failures;
`main` catches it, prints `f"error: {e}"` to `stderr`, and returns exit code
`2`.

### Step 1.2 — Geometry dataclass and `compute_layout`

Add a `@dataclass(frozen=True) class Layout` with fields: `flap_in`,
`cover_in`, `spine_in`, `total_width_in`, `total_height_in`, and the ordered
panel x-offsets (inches, measured from the jacket's left edge): `back_x`,
`spine_x`, `front_x`, `right_flap_x` (left flap starts at `0`).

Add `compute_layout(total_width, total_height, spine_width, cover_width) ->
Layout`:

- `flap = (total_width - 2 * cover_width - spine_width) / 2`
- If `flap < MIN_FLAP_IN` (use a tiny epsilon, e.g. `flap < MIN_FLAP_IN - 1e-9`),
  raise `DustCoverError` with a message naming the computed flap width and the
  1.5" minimum, e.g. `f"flap width {flap:.3f}in is below the {MIN_FLAP_IN}in
  minimum; reduce spine/cover widths or increase total width"`.
- Panel offsets left→right: left flap at `0`; `back_x = flap`; `spine_x = flap +
  cover`; `front_x = flap + cover + spine`; `right_flap_x = flap + cover + spine
  + cover`.

### Step 1.3 — `validate_exlibris` and printable-area check

- `validate_exlibris(name: str) -> str`: strip-check; if blank, raise
  `DustCoverError("exlibris name must not be blank")`; return the original
  `name` unchanged (preserve the caller's spacing/punctuation, matching
  `validate_exlibris_name` in Rust). Wire it into the `--exlibris` parse path so
  a blank value fails before anything else runs.
- `check_printable(total_width, total_height, paper_size, margin) -> None`:
  look up `(long, short)`; compute `printable_w = long - 2*margin`,
  `printable_h = short - 2*margin`; if `total_width > printable_w + 1e-9` raise
  `DustCoverError` naming both numbers; likewise if `total_height > printable_h +
  1e-9`.

Call `check_printable` and `compute_layout` from `main` right after parsing, so
an impossible request exits before any file I/O.

### LLM verification

- `python tools/dust_cover.py --tomus 1 --total-width 15.5 --total-height 9.5
  --spine-width 1.25 --cover-width 5.5 --no-compile` prints a layout with
  `flap ≈ 1.6in` and exits `0`.
- `--cover-width 6.5` (forcing `flap ≈ -0.4in`) prints the flap-minimum error to
  stderr and exits `2`.
- `--total-width 17` (tabloid printable width is 16.5) prints the printable-width
  error and exits `2`.
- `python -m unittest tools.test_dust_cover` passes the Phase-1 tests below.

### Automated tests (add to `tools/test_dust_cover.py`)

- `flap_width_computed_from_total` — `compute_layout(15.5, 9.5, 1.25, 5.5).flap_in`
  ≈ `1.6`, and `2*flap + 2*cover + spine == total_width` to within 1e-9.
- `flap_below_minimum_raises` — `compute_layout(15.5, 9.5, 1.25, 6.5)` raises
  `DustCoverError` mentioning the minimum.
- `flap_exactly_minimum_allowed` — a total giving flap `== 1.5` does not raise.
- `panel_offsets_follow_flap_back_spine_front_flap_order` — offsets are strictly
  increasing and `right_flap_x + flap == total_width`.
- `total_width_exceeds_printable_raises` / `total_height_exceeds_printable_raises`
  — `check_printable` raises for over-wide / over-tall jackets on `tabloid`.
- `a3_printable_area_smaller_than_tabloid` — the A3 printable width is `< 16.5`.
- `validate_exlibris_rejects_blank` — `""`, `"   "`, `"\t"` raise; `" Alex "`
  returns `" Alex "` unchanged.

### Human verification

None (pure arithmetic/CLI; the compiled artifact is verified in Phase 4).

---

## Phase 2 — Read the Tomus title-page block and Lives from `tomus_N_inner.tex`

*Suggested implementer: Sonnet* — deterministic text parsing with exact anchors
given.

### Step 2.1 — Roman-numeral helpers

Add `int_to_roman(n: int) -> str` and `roman_to_int(s: str) -> int` (standard
subtractive forms, uppercase). These let the script both emit `TOMUS <ROMAN>` on
the spine and cross-check the parsed `.tex`.

### Step 2.2 — `parse_inner_tex`

Add `parse_inner_tex(path: Path, expected_tomus: int) -> InnerTitle` where
`@dataclass(frozen=True) class InnerTitle` has `center_block: str` (the verbatim
LaTeX between the title page's `\begin{center}` and `\end{center}`, exclusive of
those delimiters) and `lives: list[str]` (each captured `\mbox{\textsc{\MakeLowercase{…}}}`
*inner* title string, in document order).

Behavior:

- Read the file text with `encoding="utf-8"` (the title page contains `Æ`, `Œ`,
  `É`); raise `DustCoverError` with the path if the file is missing.
- Locate the title-page section. Anchor on the literal comment line `% Tomus
  title page` emitted by `tomus_title_page`; search forward from it for the
  first `\begin{center}` and its matching `\end{center}`. Capture the text
  between them as `center_block`. If either anchor is absent, raise
  `DustCoverError` explaining the file has no Tomus title page (e.g. a restricted
  single-unit build).
- From `center_block`, verify the volume number: find `{\large TOMUS <ROMAN>}`
  with `re.search(r"\\large\s+TOMUS\s+([IVXLCDM]+)")`; if `roman_to_int` of the
  capture ≠ `expected_tomus`, raise `DustCoverError` reporting both numbers.
- Extract Lives: `re.findall(r"\\mbox\{\\textsc\{\\MakeLowercase\{(.*?)\}\}\}",
  center_block)` (non-greedy). The captured strings are already LaTeX-escaped
  (via `escape_latex` in Rust) and are reused verbatim downstream. If the list
  is empty, raise `DustCoverError` (a Tomus title page always lists ≥ 1 Life).

Because `exlibris_plate` is emitted *after* `\end{center}`, `center_block` never
contains `EX LIBRIS` — the front panel is exlibris-free for free.

### LLM verification

- Run `cargo run --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2
  Ploutarchou_Bioi_v2.pdf --out build_facsimile --no-compile` once (if
  `build_facsimile/tomus_1_inner.tex` is not already present), then in a Python
  REPL: `parse_inner_tex(Path("build_facsimile/tomus_1_inner.tex"), 1).lives`
  returns `["THESEUS", "ROMULUS", "LYCURGUS", "NUMA", "SOLON", "POPLICOLA",
  "THEMISTOCLES", "CAMILLUS"]` (Lives only — no Comparationes), and its
  `center_block` contains `PLUTARCHI` but not `EX LIBRIS`.
- `python -m unittest tools.test_dust_cover` passes the Phase-2 tests.

### Automated tests

Use an inline fixture string mirroring the real emission (copy a minimal but
faithful title-page block into the test — `% Tomus title page`, `\begin{center}`,
the LARGE/large lines, `{\large TOMUS II}`, two `\mbox{\textsc{\MakeLowercase{…}}}`
entries joined by `\textbullet`, `\end{center}`, then a trailing
`exlibris_plate`-style `EX LIBRIS` block) written to a `tempfile`:

- `lives_parsed_from_inner_tex` — returns the two titles in order.
- `center_block_excludes_exlibris_plate` — `"EX LIBRIS" not in center_block`
  even when the fixture appends a plate after `\end{center}`.
- `tomus_mismatch_between_arg_and_tex_raises` — fixture says `TOMUS II`, calling
  with `expected_tomus=1` raises `DustCoverError`.
- `missing_title_page_raises` — a `.tex` without `% Tomus title page` raises.
- `int_to_roman_and_back_roundtrips` — for `n` in `1..=12`,
  `roman_to_int(int_to_roman(n)) == n`.

### Human verification

Eyeball the `lives` list for Tomus 1 against the printed title page in
`build_facsimile/tomus_1_inner.pdf` — they must match exactly, in the same
order.

---

## Phase 3 — Generate the dust-cover LaTeX

*Suggested implementer: Opus* — LaTeX/TikZ layout judgment, matching the Tomus
cover, and vertical-centering details that the plan specifies but that benefit
from care.

### Step 3.1 — Preamble and sheet frame

Add `render_preamble(sheet_long_in, sheet_short_in) -> str` producing an
`article` document preamble:

- `\documentclass{article}`
- `\usepackage[paperwidth=<long>in,paperheight=<short>in,margin=0pt]{geometry}`
  (landscape sheet; width is the long edge).
- `\usepackage{fontspec}` with the **same** `\setmainfont{OldStandard}[Extension=.otf,
  UprightFont=*-Regular, BoldFont=*-Bold, ItalicFont=*-Italic,
  BoldItalicFont=*-BoldItalic]` block as `inner_preamble` in `src/latex.rs`, so
  the jacket's type matches the book. (Do not invent a display name; load by file
  stem exactly as the Rust path does.)
- `\usepackage{tikz}`
- `\usepackage{graphicx}`
- `\pagestyle{empty}` and `\setlength{\parindent}{0pt}`.

### Step 3.2 — Panel builders

Each returns the LaTeX for a `minipage` of fixed width and the full jacket
height, vertically centered, to be dropped into a TikZ node:

- `panel_box(width_in, height_in, body) -> str` →
  `\begin{minipage}[c][<height>in][c]{<width>in}\centering <body>\end{minipage}`.
- `front_cover_body(center_block, cover_width_in) -> str` → the verbatim
  `center_block` from `parse_inner_tex`, wrapped so its relative lengths resolve
  against the **panel**, not the sheet page. This matters: the captured block
  contains `\parbox{0.86\textwidth}` for the Lives, and inside a `minipage`
  `\textwidth` is *not* redefined — it stays the page text width (≈17in on a
  tabloid sheet), so an unguarded `0.86\textwidth` parbox would be ~14.6in wide
  and overflow the panel badly. Wrap the block in a group that first narrows
  `\textwidth` (and `\hsize`/`\columnwidth` for safety) to the panel width:
  `{\setlength{\textwidth}{<cover_width>in}\setlength{\columnwidth}{<cover_width>in}\hsize=<cover_width>in <center_block>}`.
  The enclosing `panel_box` still supplies the panel width, height, and vertical
  centering.
- `back_cover_body(lives) -> str` → the Lives, each on its own line, in the same
  small-caps house style and joined by `\\[0.8em]`:
  `\\[0.8em]`-join of `\textsc{\MakeLowercase{<life>}}` for each captured life.
  (Reuse the captured, already-escaped strings; do not re-escape.)
- `spine_body(roman, exlibris) -> str` → stacked, centered:
  - `{\large\bfseries PLUTARCHI}\\[0.4em]{\large\bfseries VITÆ}` then
    `\\[1.2em]{\large TOMUS <roman>}`.
  - If `exlibris` is not `None`: append a `\vfill` then an `EX LIBRIS` line and
    `{\large <UPPERCASED NAME>}` so the plate sits at the **foot** of the spine.
    Uppercase the name with `str.upper()` **first, then** LaTeX-escape it — i.e.
    `escape_latex(name.upper())` — matching the Rust `exlibris_plate`'s
    `escape_latex(&name.to_uppercase())` order exactly (see Step 3.3). When
    `exlibris is None`, the spine has no owner line.

  Keep the spine text horizontal (as specified) and do **not** rotate. The
  words are unbreakable (`PLUTARCHI` is one token), so a narrow spine cannot
  "wrap" them — an un-scaled `\large PLUTARCHI` (~1.4in) overflows a 1.25in
  spine. Guarantee fit by scaling the whole stacked spine block to the spine
  width: build the stack in a `\parbox`/`\shortstack` and wrap it in
  `\resizebox{<spine_width - 2·pad>in}{!}{…}` (use a small horizontal pad, e.g.
  `0.1in`, so the text does not touch the fold). `\resizebox` scales the stack
  uniformly, so `PLUTARCHI`, `VITÆ`, and `TOMUS N` keep consistent relative
  sizes and the longest line determines the scale. The optional `EX LIBRIS`
  foot block is a **separate** `\resizebox`ed stack pinned to the bottom of the
  spine panel (via the `minipage`'s vertical space), so it scales independently
  and does not inflate with the title.

### Step 3.3 — `escape_latex` helper

Add a minimal `escape_latex(s: str) -> str` covering the characters the Rust
`escape_latex` handles for owner names (`\ { } % _ # $ ~ ^ &`, and map `~`/`^`
to `\textasciitilde{}`/`\textasciicircum{}`). This is only applied to the
`--exlibris` name (the Lives/center-block text is already escaped by the Rust
emitter and must not be double-escaped).

### Step 3.4 — `render_cover` assembling the sheet

Add `render_cover(layout: Layout, sheet_long_in, sheet_short_in, title:
InnerTitle, roman: str, exlibris: Optional[str]) -> str`:

- Compute the jacket's lower-left origin that centers it on the sheet:
  `ox = (sheet_long_in - layout.total_width_in) / 2`,
  `oy = (sheet_short_in - layout.total_height_in) / 2`.
- Open `render_preamble`, `\begin{document}`, then one
  `\begin{tikzpicture}[x=1in,y=1in]`:
  - `\useasboundingbox (0,0) rectangle (<sheet_long>,<sheet_short>);`
  - Place panels with `\node[anchor=south west,inner sep=0] at (<ox+panel_x>,
    <oy>) {<panel_box(width, total_height, body)>};` for back, spine, front.
    (Flaps are blank — no node needed, but include a comment.)
  - **Cut rectangle:** `\draw[dotted,thick] (<ox>,<oy>) rectangle
    (<ox+total_width>,<oy+total_height>);`.
  - **Fold guides:** a lighter `\draw[dotted] (x,<oy>) -- (x,<oy+total_height>);`
    at each interior seam x: `ox+flap`, `ox+flap+cover`, `ox+flap+cover+spine`,
    `ox+flap+cover+spine+cover`.
  - Close `tikzpicture`, `\end{document}`.
- Format all coordinates to a fixed precision (e.g. `f"{v:.4f}"`).

### LLM verification

- In a REPL, `render_cover(...)` for Tomus 1 returns a string that:
  - contains exactly one `\draw[dotted,thick]` rectangle,
  - contains three `\node[anchor=south west` placements (back, spine, front),
  - contains `PLUTARCHI` and `VITÆ` and `TOMUS I`,
  - lists each Life from `title.lives` once,
  - contains `EX LIBRIS` **only** when `exlibris` is passed, and the front-cover
    node's body (the `center_block`) never contains `EX LIBRIS`.
- `python -m unittest tools.test_dust_cover` passes the Phase-3 tests.

### Automated tests

Drive the builders with a small hand-made `Layout` + `InnerTitle`:

- `layout_order_flap_back_spine_front_flap` — in `render_cover` output, the
  `at (x,…)` coordinate of the back node < spine node < front node, and all lie
  within the cut rectangle.
- `cut_rectangle_matches_total_size` — the rectangle's width/height equal
  `total_width`/`total_height` (parse the two corner coordinates).
- `front_cover_reproduces_title_block_without_exlibris` — front node body equals
  the `center_block`; output has no `EX LIBRIS` when `exlibris=None`.
- `spine_lists_plutarchi_vitae_and_tomus` — spine body has `PLUTARCHI`, `VITÆ`,
  `TOMUS III` (for roman `III`), each once.
- `spine_exlibris_uppercased_at_foot` — with `exlibris="alex smith"`, spine body
  contains `ALEX SMITH` and `EX LIBRIS`; without it, neither appears anywhere.
- `back_cover_lists_each_life_on_its_own_line` — back body has one
  `\textsc{\MakeLowercase{…}}` per life, separated by `\\[0.8em]`.
- `exlibris_name_uppercased_then_escaped` — `exlibris="a & b_c"` yields
  `A \& B\_C` in the spine body (uppercase applied before escaping, mirroring
  Rust).
- `fold_guides_at_four_seams` — output has four interior fold `\draw[dotted]`
  lines at the expected x coordinates.

### Human verification

Generate and compile a real cover (Phase 4) and look at the PDF: the front panel
should read like the book's Tomus title page, the spine should stack PLUTARCHI /
VITÆ / TOMUS N legibly, the back should list the Lives, and the dotted cut box
should enclose the whole jacket within the sheet margins.

---

## Phase 4 — Compile wiring, end-to-end `main`, and docs

*Suggested implementer: Sonnet* — mechanical wiring mirroring the facsimile CLI
and the `tools/` subprocess idiom.

### Step 4.1 — Write and (optionally) compile

Add:

- `write_cover_tex(out_dir: Path, tomus: int, tex: str) -> Path` → writes
  `out_dir/tomus_<N>_cover.tex` with `encoding="utf-8"` (create `out_dir` if
  absent), returns the path.
- `compile_cover(tex_path: Path, runner=subprocess.run) -> Path` → runs
  `lualatex --interaction=nonstopmode <stem>.tex` with `cwd=tex_path.parent`
  (so relative graphics/font resolution matches the facsimile build), returns
  the `.pdf` path. Raise `DustCoverError` if `runner` reports a non-zero return
  code, surfacing the last lines of the `.log` if present. Accept the injectable
  `runner` so tests pass a stub.

### Step 4.2 — Wire `main`

`main` sequence: parse args → `validate_exlibris` (if given) →
`check_printable` → `compute_layout` → resolve inner-tex path (`--inner-tex` or
`<out>/tomus_<N>_inner.tex`) → `parse_inner_tex` → `int_to_roman(tomus)` →
`render_cover` → `write_cover_tex` → unless `--no-compile`, `compile_cover`.
Print the written `.tex` (and `.pdf`) paths. Catch `DustCoverError` → stderr +
exit `2`. Return `0` on success.

### Step 4.3 — Documentation

Two documents are updated:

1. **`tools/README-dust-cover.md`** (new): a short usage section (the example
   invocation from "Desired end state"), the argument table, the flap-minimum and
   printable-area rules, and a note that the tool reads `tomus_N_inner.tex` from a
   prior facsimile build.

2. **Top-level `README.md`** (edit): add a new `## Dust covers` section, placed
   after the `## Inspecting the facsimile inner PDFs` section (it, too, is a
   `tools/` utility that consumes the facsimile build output). The section must:
   - State that `tools/dust_cover.py` generates a parameterized dust jacket
     (`.tex` + `.pdf`) for one Tomus, reading that Tomus's Lives and title-page
     block from `build_facsimile/tomus_N_inner.tex`, so a facsimile build must
     exist first.
   - Show the full example invocation (the one from "Desired end state") in a
     PowerShell/`sh` fenced block consistent with the existing facsimile
     examples.
   - Document every command-line argument in a table: `--tomus`,
     `--total-width`, `--total-height`, `--spine-width`, `--cover-width`,
     `--paper-size` (`tabloid` default / `a3`), `--exlibris`,
     `--printable-margin`, `--out`, `--inner-tex`, `--no-compile` — each with its
     meaning, unit (inches), required/optional status, and default.
   - Restate the two hard rules in prose: the jacket width is `2·flap + 2·cover
     + spine`, flaps shrink/grow to fit with a **1.5" minimum** (error below it),
     and both `total-width` and `total-height` must fit the printable sheet
     (sheet minus `2 × printable-margin`), else the tool errors out.
   - Link the tool and its README the way the existing "Inspecting the facsimile
     inner PDFs" section links its scripts
     (`[tools/dust_cover.py](tools/dust_cover.py)`,
     `[tools/README-dust-cover.md](tools/README-dust-cover.md)`).

Neither document may reference this plan file (per AGENTS.md).

### LLM verification

- `python tools/dust_cover.py --tomus 1 --total-width 15.5 --total-height 9.5
  --spine-width 1.25 --cover-width 5.5 --out build_facsimile --no-compile`
  writes `build_facsimile/tomus_1_cover.tex` and exits `0`.
- Drop `--no-compile` (with a TeX install present) → `build_facsimile/tomus_1_cover.pdf`
  is produced.
- `python -m unittest tools.test_dust_cover` — full module passes.
- `grep -n "Dust covers" README.md` and `grep -n "dust_cover.py" README.md`
  both match; the new section documents every CLI argument listed in Step 4.3.

### Automated tests

- `write_cover_tex_creates_file` — writes to a `tempfile` dir and the file
  exists with expected name `tomus_1_cover.tex`.
- `no_compile_writes_tex_only` — `main([... , "--no-compile"])` with a fake inner
  `.tex` fixture returns `0` and produces only a `.tex` (stub `runner` asserted
  **not** called).
- `compile_invokes_lualatex` — `compile_cover(path, runner=stub)` calls the stub
  with a command whose first token is `lualatex` and `cwd` is the parent dir;
  stub returns rc 0.
- `compile_failure_raises` — stub returns rc 1 → `DustCoverError`.
- `main_exits_2_on_impossible_geometry` — over-wide jacket → `main` returns `2`.

### Human verification

Print `tomus_1_cover.pdf` at 100% scale on an 11"×17" sheet; confirm the cut box
measures `total_width × total_height`, the spine width matches `--spine-width`,
and folding on the four guide lines wraps a real Tomus cleanly with ≥1.5" flaps.

---

## Out of scope / notes for the implementer

- **No Rust changes.** This tool is Python-only and reads the Rust build's
  output; it does not modify the crate. Keep the Rust suite untouched.
- **No bleed/registration marks** beyond the cut rectangle and fold guides. If
  full-bleed printing is wanted later, that is a separate change (and would use
  the `--paper-size … bleed` style rather than the margin model here).
- **Comparationes are intentionally excluded** from both panels, because the
  Tomus title page lists Lives only; the back cover mirrors the front.
- **Tomus VI caveat:** per existing project notes some Tomi are still being
  finalized; the tool works off whatever `tomus_N_inner.tex` currently exists and
  makes no assumptions about which Tomi are "done".
