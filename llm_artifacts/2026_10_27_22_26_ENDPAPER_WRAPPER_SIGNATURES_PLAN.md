# Endpaper-wrapper signatures (front + back) for each imposed tomus

## Context

Each imposed tomus is printed as a stack of landscape sheets, folded into
nested 32-page signatures, then sewn and bound. The binder plans to use an
**oversized endpaper** at the front and back of each volume. That endpaper is
larger than the trimmed book block and must be guillotined down to size at a
print shop, but a bare endpaper sheet carries none of the fold line, sew marks,
or cut (trim) border that the rest of the book's sheets carry.

The fix is to give every tomus two extra **one-sheet signatures** — one at the
very front, one at the very back — that are blank except for the same fold line,
sew marks, and cut border the body sheets already have. At binding time each of
these becomes a two-sheet signature: the blank marked sheet we print, wrapped
around the oversized endpaper tucked inside it. Because the blank sheet carries
the cut border, the print shop can guillotine the whole textblock — endpaper
included — to the correct size in one pass.

These wrapper sheets exist only to carry marks and to shepherd the endpaper
through the guillotine. They hold **no book content** and must be **wholly
independent of the book's page numbering**: adding them must not shift, renumber,
or otherwise disturb any facsimile page.

The marks themselves are produced entirely by the Rust imposition code in
`src/latex.rs`. The downstream printing tool `tools/print_sheets.py` is a generic
page-range printer and needs no logic change (see "Current state of the world");
only its README's illustrative page counts shift.

## Current state of the world

### Where the imposition and marks live

`src/latex.rs` generates, per volume, an **imposition wrapper** `.tex`
(`tomus_N.tex`) that arranges the already-rendered inner facsimile PDF
(`tomus_N_inner.pdf`) two book pages per landscape sheet side, in 32-page nested
signature folding order, and draws the fold line, sew marks, and cut border.

Key items in `src/latex.rs`:

- `const PAGES_PER_SIGNATURE: u32 = 32;` (line ~41) — book pages per signature
  (8 sheets × 4 book pages).
- `const SEW_MARGIN_CM / SEW_TAPES / SEW_HALFTAPE_CM` (lines ~36–38) — sew
  geometry.
- `struct Side { left: u32, right: u32, cut: bool, fold: bool }` (line ~720) —
  one imposed **sheet side**: the two book pages on it (`left`, `right`), and
  whether this side draws the cut border (`cut`) and the fold line + sew marks
  (`fold`).
- `fn sheet_sides(num_signatures: u32) -> Vec<Side>` (line ~733) — produces the
  ordered sheet sides for the **body**: for each signature `s` (base `32s`) and
  sheet `i` (1 = outermost of that signature), it pushes the outer side
  (`left = base+34−2i`, `right = base+2i−1`, `cut = (i==1)`, `fold = true`) then
  the inner side (`left = base+2i`, `right = base+33−2i`, `cut = false`,
  `fold = false`). 8 sheets × 2 sides = 16 sides per signature.
- `fn wrapper_document_for_paper(tomus, inner_count, paper_size) -> String`
  (line ~760) — the entry point. It computes
  `signatures = inner_count.div_ceil(32).max(1)`, calls `sheet_sides(signatures)`,
  sets `\innercount` to `inner_count`, emits the sew/fold macros and the
  per-side `\imposedsheetside{left}{right}{cut}{fold}` calls, and returns the
  full `.tex`.
- `fn wrapper_document(tomus, inner_count) -> String` (line ~756) — Letter-paper
  convenience wrapper around `wrapper_document_for_paper`.
- `const WRAPPER_SEWMACRO` (line ~833) — defines `\drawsewmarks`.
- `fn wrapper_sheet_macro(pdf, geometry) -> String` (line ~805) — defines
  `\imposedsheetside`. Crucially it guards content with
  `\ifnum#1>\innercount\else … \includegraphics[page=#1] … \fi`, so **any side
  whose book-page number exceeds `\innercount` draws blank** (just the marks, if
  its `fold`/`cut` flags are set). The fold line is drawn when `#4=1`, the sew
  marks with it; the cut border (`\draw … rectangle …`) when `#3=1`.
- `fn write_imposition_wrappers(n_volumes, inner_page_counts, out_dir, paper_size)`
  (line ~699) — writes one `tomus_N.tex` per volume by calling
  `wrapper_document_for_paper`.

The inner page numbering is baked into `tomus_N_inner.pdf`; the wrapper only
*arranges* those pages. Nothing in the wrapper assigns or renumbers book pages,
so wrapper sides that reference no inner page cannot affect the book's
numbering.

### Signature arithmetic

`src/signatures.rs` holds `PAGES_PER_SIGNATURE = 32` and
`pad_to_multiple_of_32`, used by `src/volumes.rs` to size the **inner** content
to whole 32-page signatures and to enforce the `MAX_PAGES` budget. This plan
does **not** touch inner-content sizing: the wrapper sheets are added by the
imposition wrapper alone and do not change `inner_count`, so
`signatures.rs`/`volumes.rs` are unaffected.

### Existing imposition tests (in `src/latex.rs` `mod tests`)

- `imposition_orders_pages_for_signature` (~1648) — exact `Side` values for
  `sheet_sides(1)` and `sheet_sides(2)`.
- `imposition_places_odd_book_pages_on_right` (~1700) — over `sheet_sides`, every
  `right` is odd, every `left` even, and the union of pages is exactly
  `1..=32*signatures`.
- `fold_and_sew_only_on_outer_sides` (~1715) — `sheet_sides(2)` alternates
  `fold`; `wrapper_document(1,32)` contains `\drawsewmarks` and the fold-line
  draw.
- `cut_border_only_on_first_recto_of_each_signature` (~1725) — over
  `sheet_sides(2)`, `cut` is true only at indices 0 and 16.
- `pages_past_inner_count_blank` (~1732) — `wrapper_document(1,5)` sets
  `\innercount` to 5 and still emits `\imposedsheetside{32}{1}{1}{1}`.
- `wrapper_references_volume_inner_pdf`, `wrapper_letter_retains_existing_sheet_geometry`,
  `wrapper_a4_centers_unchanged_trim_rectangles`,
  `wrapper_paper_size_keeps_signature_order_and_marks`,
  `legacy_wrapper_api_remains_letter`, `write_files_create_inner_and_wrappers`.

All of these assert over `sheet_sides` (the **body**) or use `contains` on the
document, so keeping the wrapper sides out of `sheet_sides` preserves every one
of them.

### The printing tool

`tools/print_sheets.py` prints a page range one duplex sheet per job. Its
validation is entirely generic:

- `parse_page_range` — requires an **odd start page** (front of a sheet), start
  ≤ end, positive.
- `plan_sheets` — pairs consecutive pages into `Sheet { number, first_page,
  last_page }`, sheet `k` = PDF pages `2k−1` and `2k`; an odd final page becomes
  a single-sided last sheet.
- `apply_resume`, `check_range_in_document` (via PyMuPDF page count).

There is **no 32-page / signature assumption anywhere** in `print_sheets.py`.
Each tomus's imposed PDF gains exactly 4 PDF pages (2 wrapper sheets × 2 sides),
the page count stays even, and the odd-start / consecutive-pair validation stays
valid. Therefore `print_sheets.py` needs **no code change**; only
`tools/README-print-sheets.md`'s illustrative sheet counts and prose should
mention the two wrapper sheets.

## Desired end state

- `wrapper_document_for_paper` emits, in order: **one front wrapper sheet
  (2 sides)**, then the full body `sheet_sides(signatures)`, then **one back
  wrapper sheet (2 sides)**.
- Each wrapper sheet carries the fold line, sew marks, and cut border on its
  **outer side** (the side facing the cover board) and is **completely blank on
  its endpaper-facing inner side** (no marks, no content).
- The wrapper's cut border is the **same trim rectangle the body sheets use** —
  `\draw[dotted,thick] (\Lx,\By) rectangle (\Rx,\Ty)`, produced by the existing
  `\imposedsheetside` `#3=1` branch. It is not a new or larger rectangle: the
  whole point is that the guillotine, following this book-trim rectangle, cuts
  the oversized endpaper tucked inside the wrapper down to the book's trim size.
- Wrapper sides reference **no real book page**: both `left` and `right` are the
  sentinel `inner_count + 1`, one past the last real page. The imposition macro
  draws a page only when `#1 ≤ \innercount` (its guard is
  `\ifnum#1>\innercount\else <draw> \fi` — the *else* branch draws), so a page
  number strictly greater than `\innercount` draws nothing. Nothing from
  `tomus_N_inner.pdf` is drawn on the wrappers, and the body sides are
  byte-for-byte unchanged, so no facsimile page is renumbered or shifted.
- For a volume with `S` body signatures, `tomus_N.tex` emits `16*S + 4`
  `\imposedsheetside` calls (was `16*S`) — one call per imposed sheet side — so
  the compiled imposed `tomus_N.pdf` has `16*S + 4` pages: the imposed PDF gains
  **2 physical sheets / 4 PDF pages**, the first two pages being the front
  wrapper and the last two the back wrapper.
- Doc comments in `src/latex.rs` describe the wrapper sheets as part of the
  current contract (no history narration).
- `tools/README-print-sheets.md` notes that each imposed tomus begins and ends
  with a blank endpaper-wrapper sheet that prints like any other sheet, and its
  illustrative counts are consistent.
- `tools/print_sheets.py` is unchanged.
- `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo test` all pass.

### Concrete emission layout (the contract the implementer must hit)

Using the sentinel page `inner_count + 1` (shown below as `33` for a
one-signature `inner_count = 32` volume) and flags `{cut}{fold}`:

Front wrapper (first physical sheet, printed first):

```
\imposedsheetside{33}{33}{1}{1}   % recto: faces front cover — cut border + fold line + sew marks
\imposedsheetside{33}{33}{0}{0}   % verso: faces the endpaper / body — blank
```

Body: `sheet_sides(signatures)` exactly as today.

Back wrapper (last physical sheet, printed last):

```
\imposedsheetside{33}{33}{0}{0}   % recto: faces the endpaper / body — blank
\imposedsheetside{33}{33}{1}{1}   % verso: faces back cover — cut border + fold line + sew marks
```

Rationale for the back wrapper's side order: the marked side should face the
**outside** of the book (toward the cover board) at the back just as it does at
the front, and the blank side should face the endpaper/body. At the front the
outer face is the sheet's recto (first PDF page); at the back the outer face is
the sheet's verso (last PDF page). This keeps the trim border on the two
outermost faces of the whole textblock, which is exactly what the guillotine
operator sees. This recto/verso orientation is the one detail that only a folded
paper dummy can fully confirm — see the human-verification note in Phase 1.

## Tech Learnings

- **LaTeX `\ifnum`.** `\ifnum#1>\innercount <true> \else <false> \fi` is a
  plain-TeX integer comparison; `\innercount` is set via
  `\newcommand{\innercount}{N}`. In the imposition macro the *true* branch is
  empty and the *false* branch (after `\else`) runs `\includegraphics`, so a page
  is drawn exactly when `#1 ≤ N` and left blank when `#1 > N`. A blank wrapper
  side therefore needs a page number **strictly greater** than `\innercount`;
  `inner_count + 1` is the smallest such value and is always safe (page `0` would
  be ≤ N and wrongly request a nonexistent inner page). Keep the sentinel well
  under TeX's integer ceiling (2^31−1); `inner_count + 1` always is.
- **Imposition / signatures.** A *signature* is a stack of sheets folded
  together and sewn through the common fold. The body uses 32-page signatures (8
  sheets). A *one-sheet signature* is a single folded sheet = 4 book pages; here
  all 4 are blank, so only the two printed sides (recto/verso of the one sheet)
  matter, and they carry marks, not content.
- **Sew marks vs. cut border vs. fold line.** The fold line (dotted vertical at
  the spine `\Mx`) shows where to fold; `\drawsewmarks` places the kettle-stitch
  and tape-station dots along that fold so every sheet in a signature is punched
  consistently; the cut border (`\draw … rectangle (\Lx,\By)(\Rx,\Ty)`) is the
  trim rectangle the guillotine follows.

---

## Phase 1 — Add front and back endpaper-wrapper sheets to the imposition

*Suggested implementer: Opus.* The change is small but demands getting the
blank/marked side orientation and the "no page-number impact" invariant exactly
right, and judgment about which existing tests to extend vs. leave alone.

### Step 1.1 — Build the wrapper sheet sides and splice them around the body

In `src/latex.rs`:

1. Add a small helper returning the two sides of one wrapper sheet, parameterized
   by the inner page count (for the blank sentinel) and by which side carries the
   marks:
   ```rust
   /// One blank endpaper-wrapper sheet: a single folded sheet that carries the
   /// fold line, sew marks, and cut border on the face toward the cover board and
   /// is blank on the face toward the endpaper. `marks_first` selects whether the
   /// marked (outer) side is the sheet's recto (front wrapper) or verso (back
   /// wrapper). Both sides reference book page `inner_count + 1`, one past the
   /// last real page, so the macro's `\ifnum#1>\innercount` guard draws no inner
   /// content on them and the wrappers never affect the book's page numbering.
   fn endpaper_wrapper_sides(inner_count: u32, marks_first: bool) -> [Side; 2] {
       let blank_page = inner_count + 1;
       let marked = Side { left: blank_page, right: blank_page, cut: true, fold: true };
       let blank = Side { left: blank_page, right: blank_page, cut: false, fold: false };
       if marks_first { [marked, blank] } else { [blank, marked] }
   }
   ```
   The sentinel must be **greater** than `\innercount`: the macro draws a page
   only on the `\else` (false) branch, i.e. when `#1 ≤ \innercount`. A sentinel
   of `0` would request nonexistent inner page 0 and fail the lualatex compile.
2. In `wrapper_document_for_paper`, replace
   ```rust
   let sides = sheet_sides(signatures);
   ```
   with an assembly that brackets the body with the two wrapper sheets:
   ```rust
   let mut sides = Vec::with_capacity((signatures * 16 + 4) as usize);
   sides.extend(endpaper_wrapper_sides(inner_count, true));   // front: marked recto, blank verso
   sides.extend(sheet_sides(signatures));                     // body, unchanged
   sides.extend(endpaper_wrapper_sides(inner_count, false));  // back: blank recto, marked verso
   ```
   Leave the `for side in &sides { … \imposedsheetside … }` loop untouched — it
   already emits every side in order.
4. Do **not** modify `sheet_sides`. The body arithmetic and all tests over it
   stay exactly as they are.

**Why `sheet_sides` stays untouched:** the wrappers are "wholly independent" of
the book — keeping them out of `sheet_sides` means `imposition_places_odd_book_pages_on_right`
and `cut_border_only_on_first_recto_of_each_signature` (both asserting over the
body) remain true, and no inner page is renumbered.

### Step 1.2 — Update doc comments to the current contract

- Update the module-level comment at the top of `src/latex.rs` (currently
  "...two-up for 32-page-signature binding.") to add, in present tense, that each
  volume's imposed sheets are bracketed by a blank front and back endpaper-wrapper
  sheet carrying the fold line, sew marks, and cut border.
- Update the `wrapper_document_for_paper` doc comment to state that the emitted
  sides are: front wrapper sheet, the body signatures, back wrapper sheet.
- Do **not** narrate the change ("previously…", "now adds…") — describe only the
  present contract, per AGENTS.md.

### Step 1.3 — Tests

Add to `mod tests` in `src/latex.rs` (new tests; do not renumber existing ones):

- `endpaper_wrappers_bracket_the_body`
  Assert that for `wrapper_document(1, 32)` the number of `\imposedsheetside{`
  call occurrences equals `16 + 4 == 20`. (For `inner_count = 32`, `signatures`
  = 1 → 16 body sides + 4 wrapper sides.) Count occurrences of the literal
  `"\\imposedsheetside{"` (the `\newcommand{\imposedsheetside}` definition uses
  `\imposedsheetside}`, so it is not counted).

- `endpaper_wrapper_sides_carry_no_book_pages`
  With sentinel `inner_count + 1 = 33` for `inner_count = 32`, assert
  `endpaper_wrapper_sides(32, true)` == `[Side{33,33,cut:true,fold:true},
  Side{33,33,cut:false,fold:false}]` and `endpaper_wrapper_sides(32, false)` ==
  `[Side{33,33,cut:false,fold:false}, Side{33,33,cut:true,fold:true}]`.

- `endpaper_wrapper_pages_exceed_inner_count_so_they_draw_blank`
  For several `inner_count` values (`0, 5, 32, 33, 128`), assert both sides of
  both `endpaper_wrapper_sides(inner, true)` and `endpaper_wrapper_sides(inner,
  false)` have `left > inner` and `right > inner`. This encodes the invariant
  that wrapper pages are always past the inner count (so they never draw and the
  lualatex compile never requests a nonexistent page).

- `endpaper_wrappers_emit_marked_blank_sides`
  Assert `wrapper_document(1, 32)` contains both `\imposedsheetside{33}{33}{1}{1}`
  (marked wrapper side) and `\imposedsheetside{33}{33}{0}{0}` (blank wrapper
  side).

- `endpaper_wrappers_do_not_renumber_inner_pages`
  Assert `wrapper_document(1, 32)` still contains the unchanged body sides
  `\imposedsheetside{32}{1}{1}{1}` and `\imposedsheetside{2}{31}{0}{0}`, proving
  the body imposition — and thus the facsimile page numbering — is untouched.

- `front_wrapper_is_first_sheet_back_wrapper_is_last`
  Build `wrapper_document(1, 32)`, collect the ordered `\imposedsheetside{…}`
  call lines (e.g. `doc.lines().filter(|l| l.starts_with("\\imposedsheetside{"))`,
  which excludes the `\newcommand{\imposedsheetside}` definition because that
  renders as `\imposedsheetside}`), and assert on the ends of that ordered list
  (sentinel `33` for the `inner_count = 32` document):
  - `calls[0] == "\\imposedsheetside{33}{33}{1}{1}"` (front wrapper marked recto),
  - `calls[1] == "\\imposedsheetside{33}{33}{0}{0}"` (front wrapper blank verso),
  - `calls[calls.len()-2] == "\\imposedsheetside{33}{33}{0}{0}"` (back wrapper
    blank recto),
  - `calls[calls.len()-1] == "\\imposedsheetside{33}{33}{1}{1}"` (back wrapper
    marked verso).
  This pins the exact bracketing order in one test.

Confirm the existing tests still pass unchanged:
`imposition_orders_pages_for_signature`, `imposition_places_odd_book_pages_on_right`,
`fold_and_sew_only_on_outer_sides`, `cut_border_only_on_first_recto_of_each_signature`,
`pages_past_inner_count_blank`, `wrapper_paper_size_keeps_signature_order_and_marks`,
`legacy_wrapper_api_remains_letter`.

### LLM verification

- `cargo test latex` (or `cargo test imposition`, `cargo test wrapper`,
  `cargo test endpaper`) — the new tests pass and no existing imposition test
  regresses.
- `cargo test` — full suite green before declaring done.
- `cargo fmt --all --check` and `cargo clippy --all-targets -- -D warnings` —
  clean. (Run `cargo fmt` to apply, not just check.)
- `grep -n "imposedsheetside{" ` on a freshly generated `tomus_1.tex` (or inspect
  the string in a `--nocapture` test) shows the first side `{0}{0}{1}{1}`, the
  body sides unchanged, and the final side `{0}{0}{1}{1}`.

### Human verification

- Build a tomus and compile its wrapper with `lualatex`; confirm the **first**
  printed sheet and the **last** printed sheet are blank except for the fold
  line, sew-station dots, and the rectangular cut border, and that every body
  page's facsimile content and page numbers are exactly where they were before.
- Fold a paper dummy of the front and back wrapper sheets as they will be bound,
  nest a stand-in oversized endpaper inside each, and confirm the marked side
  faces outward (toward the cover) and the blank side faces the endpaper at
  **both** front and back — i.e. the back wrapper's recto/verso orientation
  (Step 1.1) is correct. This physical check is the authority on the side order.

---

## Phase 2 — Update the printing README (no code change)

*Suggested implementer: Sonnet.* Mechanical doc edit; the facts are pinned.

### Step 2.1 — Note the wrapper sheets in `tools/README-print-sheets.md`

- Do **not** change `tools/print_sheets.py`. Its generic validation already
  handles the two extra sheets (page count stays even; odd-start / consecutive-
  pair pairing stays valid). State nowhere that a code change was needed.
- In the prose that explains sheet numbering (and the examples that say e.g.
  "a whole Tomus range (16 sheets, pages 145–160)"), add that each imposed tomus
  now begins and ends with a **blank endpaper-wrapper sheet** carrying the fold
  line, sew marks, and cut border, and that these print like any other sheet (so
  a whole-tomus range includes them — its first and last sheets are the
  wrappers). Adjust any stated sheet count that claimed to be the body-only count
  so it is consistent with the two added sheets, or rephrase it to avoid a stale
  absolute count. Rewrite affected sentences in place (no "Update:" block), per
  AGENTS.md.
- Keep the example page ranges internally consistent: if an example range is
  described as a whole tomus, make its sheet count and page span agree after the
  +2 sheets.

### LLM verification

- `grep -n "sheet" tools/README-print-sheets.md` — the wrapper sheets are
  described and no stale "N sheets" count contradicts the new structure.
- Confirm `tools/print_sheets.py` has **no** diff (`git diff --stat` shows only
  the README under `tools/`).

### Human verification

- Read the README top to bottom; confirm it reads as if written today (no update
  scars) and that someone printing a whole tomus understands the first and last
  sheets are the endpaper wrappers.

---

## Out of scope / explicitly unchanged

- `src/signatures.rs` and `src/volumes.rs` — inner-content sizing and the
  `MAX_PAGES` budget are unaffected; the wrappers are added by the imposition
  wrapper only and do not change `inner_count`.
- `tools/print_sheets.py` — no validation or logic change; see Phase 2.
- The inner facsimile rendering (`src/facsimile/*`) and the book's page
  numbering — untouched.
