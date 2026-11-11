# Collation (back-mark) stripes on every signature spine

## Context

The imposition wrapper (`src/latex.rs`, "Imposition wrapper" section) already
prints three bindery aids on each imposed sheet side: a dotted fold line down the
spine, sew marks along the fold, and a cut border on the first recto of each
signature. This plan adds a fourth aid: a **collation mark** (binder's "back
mark"). Each gathered unit — the front endpaper wrapper, every body signature,
and the back endpaper wrapper — gets one thick stripe straddling the spine fold,
positioned a little lower per unit. When the units are folded and gathered in
order, the stripes line up into a continuous diagonal running down the spine; a
missing, doubled, out-of-order, or flipped unit shows up instantly as a break or
reversal in the diagonal.

The marks live entirely in the spine/waste area and never appear in the finished
book, so it does not matter that the topmost stripes sit above the trim and some
of each stripe is bound into the gutter — the aid exists only for the binder.

## Current state of the world

Relevant code, all in `src/latex.rs`:

- `const SEW_MARGIN_CM`, `SEW_TAPES`, `SEW_HALFTAPE_CM` (lines ~41–43) hold sew
  geometry as string constants in centimetres.
- `struct Side { left, right, cut, fold, sew }` (line ~735) models one imposed
  sheet side. `cut`/`fold`/`sew` are independent booleans.
- `push_signature(sides, base, sheets)` (line ~750) pushes one signature's sheet
  sides. Only sheet `i == 1`'s outer/recto side gets `cut: true`; every side gets
  `sew: true`; only outer sides get `fold: true`.
- `sheet_sides(inner_count)` (line ~778) builds the body: as many full 16-page
  signatures as fit, then a truncated final signature (whole sheets, floored at
  two).
- `endpaper_wrapper_sides(inner_count, marks_first)` (line ~803) returns the two
  sides of one blank endpaper wrapper; the **marked** side has
  `cut: true, fold: true, sew: true` and the blank side has all three `false`.
- `wrapper_document_for_paper(tomus, inner_count, paper_size)` (line ~834) builds
  the ordered `sides` vector (front wrapper, body, back wrapper), emits the
  preamble defs (`\Lx`/`\Mx`/`\Rx`/`\By`/`\Ty`, the sew defs), the sew macro
  (`WRAPPER_SEWMACRO`, line ~912) and the per-side macro
  (`wrapper_sheet_macro`, line ~882), then one `\imposedsheetside{..}{..}{..}{..}{..}`
  call per side.
- `wrapper_sheet_macro` defines `\imposedsheetside` with **5** parameters:
  `#1` left page, `#2` right page, `#3` cut flag, `#4` fold flag, `#5` sew flag.
  It draws the fold line when `#4=1`, calls `\drawsewmarks` when `#5=1`, and draws
  the cut border when `#3=1`.
- `WrapperGeometry` (line ~661) carries `bounding_height` (the sheet's top edge in
  the tikz coordinate system: the page runs `(0,0)`–`(bounding_width,
  bounding_height)`) and `ty` (`\Ty`, the **top cutline**, below the sheet top).
  Letter: `bounding_height = "21.59"`, `ty = "18.795"`. A4: `"21"` / `"18.5"`.

Key invariant this plan relies on: **exactly one side per gathered unit has
`cut == true`**, and those cut sides appear in binding order (front wrapper →
sig 1 → … → sig N → back wrapper). The count of cut sides equals the number of
collation steps.

The sew macro computes its vertical span in TeX from the defs:
`\yb = \By + \sewmargin` (bottom kettle stitch), `\yt = \Ty - \sewmargin` (top
kettle stitch). The **top sew mark is at `\yt`**.

Tests live in the `#[cfg(test)] mod tests` block at the bottom of the file
(imposition tests around lines ~1724–2060).

## Desired end state

The emitted wrapper `.tex` defines a `\drawcollationmark{step}` macro and a
`\collationtotal` count, and each `\imposedsheetside` call carries a **sixth**
argument: the collation step (`0` = no mark; `1..=\collationtotal` = this unit's
position in the gathered order). The macro draws, for cut-bearing sides only, a
filled stripe straddling the spine fold `\Mx`, in a vertical band that runs from
the sheet top (`\collationtop`, above the top cutline) down to a small gap above
the top sew mark (`\yt`). The band is divided into `\collationtotal` equal,
top-anchored slots; step *k* fills slot *k*, so step 1 touches the sheet top and
the last step's lower edge sits `COLLATION_GAP_CM` above `\yt`.

Concretely, for a one-signature volume (`inner_count = 16`) the emitted calls
include:

```
\imposedsheetside{17}{17}{1}{1}{1}{1}   % front wrapper marked side  -> step 1
\imposedsheetside{16}{1}{1}{1}{1}{2}    % signature 1 first recto    -> step 2
\imposedsheetside{2}{15}{0}{0}{1}{0}    % inner body side, no mark   -> step 0
\imposedsheetside{17}{17}{1}{1}{1}{3}   % back wrapper marked side   -> step 3
```

and `\newcommand{\collationtotal}{3}`. A compiled multi-signature wrapper shows
the spine stripe stepping steadily downward from sheet to sheet, always clear of
the sew marks, so a gathered block reads as a clean diagonal on the spine.

`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test` all pass.

## Tech Learnings

- **pgfmath in TikZ.** `\pgfmathsetmacro{\name}{expr}` evaluates a fixed-point
  arithmetic expression and stores the result in `\name` for use in coordinates.
  The sew macro already uses it; the collation macro follows the same idiom.
  Values here are centimetres in the 0–30 range, well within pgfmath's range.
- **`\fill (x0,y0) rectangle (x1,y1);`** draws a solid filled rectangle — the
  stripe. `\Mx` is the spine fold's x; the stripe straddles it by
  `±\collationhalf`.
- **Why tie the mark to `cut`.** The cut border is drawn once per gathered unit,
  on that unit's outermost recto — exactly the sheet whose fold becomes the
  visible outside of the gathered spine. Reusing that invariant means the
  collation step is simply "the ordinal of this side among cut-bearing sides,"
  so no new `Side` field and no new boundary detection is needed.

---

## Phase 1 — Emit collation marks

Suggested implementer: Sonnet

All changes are in `src/latex.rs`. The plan pins the exact constants, macro text,
emission logic, and test strings.

### Step 1.1 — Add the collation geometry constants

Immediately below the sew constants (after `SEW_HALFTAPE_CM`, ~line 43), add:

```rust
/// Collation-mark ("back mark") geometry, in centimetres. Each gathered unit —
/// the two endpaper wrappers and every body signature — gets one thick stripe
/// straddling the spine fold, stepped down per unit so a gathered block shows a
/// diagonal staircase on the spine. The band of stripes runs from the sheet top
/// (above the top cutline, in the waste) down to `COLLATION_GAP_CM` above the top
/// sew mark, so the marks never touch the sewing holes and never survive into the
/// trimmed book.
const COLLATION_HALFWIDTH_CM: &str = "0.30"; // half of a 6 mm stripe across the fold
const COLLATION_GAP_CM: &str = "0.50"; // clearance kept above the top sew mark
```

**LLM verification:** `cargo build` still compiles (constants are unused until
Step 1.3 — that is fine within this step; the next steps consume them, so do not
land Step 1.1 alone if clippy's `dead_code` is a concern — run clippy after
Step 1.4).

**Human verification:** None.

### Step 1.2 — Add the `\drawcollationmark` macro constant

Below `WRAPPER_SEWMACRO` (~line 924), add a sibling constant. The macro takes the
step index and fills slot *k* of a top-anchored band:

```rust
const WRAPPER_COLLATIONMACRO: &str = "\
\\newcommand{\\drawcollationmark}[1]{%\n\
\x20 \\pgfmathsetmacro{\\colbot}{\\Ty-\\sewmargin+\\collationgap}%\n\
\x20 \\pgfmathsetmacro{\\colslot}{(\\collationtop-\\colbot)/\\collationtotal}%\n\
\x20 \\pgfmathsetmacro{\\colyhi}{\\collationtop-(#1-1)*\\colslot}%\n\
\x20 \\pgfmathsetmacro{\\colylo}{\\collationtop-#1*\\colslot}%\n\
\x20 \\fill (\\Mx-\\collationhalf,\\colylo) rectangle (\\Mx+\\collationhalf,\\colyhi);%\n\
}\n";
```

Notes for the implementer:
- `\colbot` recomputes the top sew mark (`\Ty-\sewmargin`, i.e. `\yt`) plus the
  gap, rather than depending on `\drawsewmarks` having run; both macros are
  independent.
- `\collationtop`, `\collationhalf`, `\collationgap`, and `\collationtotal` are
  doc-level defs added in Step 1.3. The macro only needs them to exist before
  `\begin{document}`.

**LLM verification:** Covered by the string-content test in Step 1.5.

**Human verification:** None.

### Step 1.3 — Define the collation values and register the macro

In `wrapper_document_for_paper`, after the sew defs are pushed and
`WRAPPER_SEWMACRO` is appended (~line 865, before `wrapper_sheet_macro` is
appended), compute the count of gathered units and push the defs + macro:

```rust
let collation_total = sides.iter().filter(|s| s.cut).count();
doc.push_str(&format!(
    "\\def\\collationtop{{{}}}\n\
     \\def\\collationhalf{{{COLLATION_HALFWIDTH_CM}}}\n\
     \\def\\collationgap{{{COLLATION_GAP_CM}}}\n\
     \\newcommand{{\\collationtotal}}{{{collation_total}}}\n",
    geometry.bounding_height
));
doc.push_str(WRAPPER_COLLATIONMACRO);
```

`sides` is already fully built at this point (front wrapper + body + back
wrapper), so the filter sees every gathered unit. `collation_total` is always
≥ 2 (the two endpaper wrappers), so `\colslot`'s division never divides by zero.

**LLM verification:** Step 1.6 asserts `\collationtop` and `\collationtotal`.

**Human verification:** None.

### Step 1.4 — Bump `\imposedsheetside` to six arguments and emit the step

In `wrapper_sheet_macro` (~line 882): change the parameter count from `[5]` to
`[6]`, update the leading comment to name `#6` as the collation step, and add a
`#6` block **after** the sew block (`\ifnum#5=1 … \fi`) and before the cut-border
block:

```
\x20   \\ifnum#6>0
\x20     \\drawcollationmark{#6}
\x20   \\fi
```

Update the comment line to:

```
% #1 left book page, #2 right book page, #3 cut flag, #4 fold flag, #5 sew flag, #6 collation step
```

In the emission loop in `wrapper_document_for_paper` (~line 868), track a running
step that advances once per cut side and emit it as the sixth argument:

```rust
let mut collation_step = 0u32;
for side in &sides {
    let step = if side.cut {
        collation_step += 1;
        collation_step
    } else {
        0
    };
    doc.push_str(&format!(
        "\\imposedsheetside{{{}}}{{{}}}{{{}}}{{{}}}{{{}}}{{{}}}\n",
        side.left,
        side.right,
        u8::from(side.cut),
        u8::from(side.fold),
        u8::from(side.sew),
        step,
    ));
}
```

Because cut sides occur in binding order, the steps come out
`1, 2, …, collation_total` across front wrapper → signatures → back wrapper.

**LLM verification:**
- `cargo build`.
- `cargo clippy --all-targets -- -D warnings` (confirms the Step 1.1 constants
  are now used).
- `cargo fmt --all` then `cargo fmt --all --check`.

**Human verification:** None yet (behavior is checked after the tests and the
compile in Step 1.7).

### Step 1.5 — Update the one exact-match test

Only `front_wrapper_is_first_sheet_back_wrapper_is_last` (~line 1951) asserts
full-line equality and must gain the sixth argument. For `inner_count = 16`,
`collation_total = 3`, so the marked wrapper sides carry steps 1 and 3 and the
blank sides carry 0:

```rust
assert_eq!(calls[0], "\\imposedsheetside{17}{17}{1}{1}{1}{1}");
assert_eq!(calls[1], "\\imposedsheetside{17}{17}{0}{0}{0}{0}");
let n = calls.len();
assert_eq!(calls[n - 2], "\\imposedsheetside{17}{17}{0}{0}{0}{0}");
assert_eq!(calls[n - 1], "\\imposedsheetside{17}{17}{1}{1}{1}{3}");
```

Every other imposition test uses `.contains(...)` on a five-field prefix (which
is still a substring of the six-field call) or counts `\imposedsheetside{`
occurrences, so they pass unchanged. Do **not** edit them.

**LLM verification:** `cargo test front_wrapper_is_first_sheet_back_wrapper_is_last`.

**Human verification:** None.

### Step 1.6 — Add collation tests

Add these to the imposition `mod tests` block:

- `collation_step_increments_once_per_gathered_unit` — for `wrapper_document(1, 16)`
  assert the doc contains all of:
  `\imposedsheetside{17}{17}{1}{1}{1}{1}` (front wrapper → 1),
  `\imposedsheetside{16}{1}{1}{1}{1}{2}` (signature 1 first recto → 2),
  `\imposedsheetside{17}{17}{1}{1}{1}{3}` (back wrapper → 3),
  and `\imposedsheetside{2}{15}{0}{0}{1}{0}` (an inner body side → no mark).

- `collation_total_counts_gathered_units` — assert
  `wrapper_document(1, 16)` contains `\newcommand{\collationtotal}{3}` (1 body
  signature + 2 wrappers) and `wrapper_document(1, 33)` contains
  `\newcommand{\collationtotal}{5}` (2 full + 1 truncated signature + 2 wrappers).

- `collation_macro_defines_band_above_top_sew_mark` — assert
  `wrapper_document(1, 16)` contains the macro header
  `\newcommand{\drawcollationmark}[1]`, the call guard `\ifnum#6>0`, and the band
  floor `\pgfmathsetmacro{\colbot}{\Ty-\sewmargin+\collationgap}` (documents that
  the band bottom is held above the top sew mark `\yt`).

- `collation_top_tracks_paper_size` — assert `\def\collationtop{21.59}` for
  `wrapper_document_for_paper(1, 32, PaperSize::Letter)` and `\def\collationtop{21}`
  for `PaperSize::A4` (the stripe band starts at the sheet top, above `\Ty`).

**LLM verification:** `cargo test collation` runs the four new tests; then
`cargo test` for the full suite.

**Human verification:** None.

### Step 1.7 — Document the new mark and eyeball a compiled wrapper

Update `README.md`'s mark description (the sentence near line ~385: "Fold lines
are drawn on each outer/recto sheet side only; sew marks on every sheet side
(both faces); a cut border on the first recto of each signature.") to add the
collation mark — rewrite the sentence in place so it reads as one current
description, e.g. append: "; and a collation stripe on each gathered unit's spine
(the two endpaper wrappers and every signature), stepped down per unit so the
gathered block shows a diagonal down the spine, clear of the sew marks." Follow
the in-place rewrite rule in AGENTS.md — do not add a revision note.

Then regenerate and compile one multi-signature wrapper to confirm the visual.

**LLM verification:**
- `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test` all green.
- Generate a wrapper with several signatures (e.g. a volume whose inner count
  spans 4–5 signatures) and `grep -n "{6}\|collation" tomus_*.tex` to confirm the
  sixth arg steps `1,2,3,…` across the cut sides and `0` elsewhere.

**Human verification:** Compile the generated multi-signature wrapper to PDF with
lualatex and look at the spine column: confirm (a) each cut sheet's stripe sits a
little lower than the previous one, (b) the first stripe begins at the very top of
the sheet, above the cut border, (c) the last stripe stays clear above the top sew
dot, and (d) laying the printed sheets in order shows the stripes forming a clean
diagonal down the spine. This is the real-eyes check that the diagonal reads
correctly and does not collide with the sew marks.
