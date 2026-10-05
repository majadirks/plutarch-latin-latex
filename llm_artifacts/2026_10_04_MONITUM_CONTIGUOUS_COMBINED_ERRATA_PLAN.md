# Monitum: consecutive prose and one combined errata page

## Context

The facsimile Monitum is a hybrid: cached Latin prose is re-typeset through
`mutanda esse significavit.`, followed by verbatim source crops of the Greek errata.
The user has inspected `build_facsimile/monitum_check/tomus_1_inner.pdf` and identified
two layout problems: blank PDF pages 15, 17, and 19 interrupt the sequence, and the
short p12 errata tail occupies an entire page despite having just six printed lines.

This plan places the two existing crops together on one page immediately after the
prose, then starts the first body unit on the next page without an intervening blank.
It preserves all cached source text, scan pixels, and crop boundaries.

## Current state of the world

- `src/facsimile/frontmatter.rs` defines `MONITUM_PROSE_PAGES`, the exact
  `MONITUM_PROSE_SENTINEL`, and `MONITUM_IMAGE_PAGES`. p13 is excluded from both
  transcription and reconciliation. These behaviors are already tested.
- `latex::front_prose_pages_until` in `src/latex.rs` selects entire cached blocks
  through the sentinel without modifying their text. Its page framing starts with
  `\freshpage` and ends with `\clearpage`.
- `front_monitum_a.png` contains the p12 errata tail. Its tuned edge fractions are
  `(0.13, 0.769, 0.185, 0.109)` and its current dimensions are 1348 × 363 pixels.
  `front_monitum_b.png` contains the p13 errata, with fractions
  `(0.18, 0.094, 0.14, 0.350)` and dimensions 1359 × 1652 pixels.
- `src/facsimile/assemble.rs::inner_document` emits the prose, then calls
  `latex::embed_scan` separately for each crop. Each call starts with `\freshpage`,
  uses a `center` environment, and ends with `\clearpage`.
- `src/latex.rs::inner_preamble` defines `\freshpage` as
  `\clearpage\ifodd\value{page}\thispagestyle{empty}\hbox{}\clearpage\fi`.
  It explicitly inserts a blank page when the next physical page counter is odd.
  The macro's existing parity convention should be preserved outside this task;
  this plan does not attempt to correct its naming or change global pagination.
- In the verification PDF, the prose is on physical page 14, the crops on pages
  16 and 18, and the first body page on page 20. The intervening blanks are the
  results of the calls before those three content pages.
- `assemble::unit_block` also starts with `\freshpage`. `inner_document` calls
  `\beginbodynumbering` before rendering the units. That numbering macro records
  the current physical page counter; it does not reset the counter. A blank inserted
  afterward can offset body numbering. The first body page explicitly uses
  `\thispagestyle{empty}`, hiding its folio and running head. The next body page
  shows the running head and folio; in the current fixture that visible folio is 3.
- The facsimile trim is 10.5 × 16 cm, with an 8.5 cm text width and approximately
  12.1 cm text height after the configured margins and reserved header band. At full
  text width, the crop heights total
  `8.5 * (363 / 1348 + 1652 / 1359) ≈ 12.62 cm`, before inter-image spacing.
  A shared reduction of approximately 10% leaves room for both crops and a small gap.
- `src/volumes.rs::MONITUM_PAGES` is the unrelated numeric overhead estimate,
  currently 3: one prose page and two separate image pages.
- `README.md` describes two embedded scaled source images. The existing implementation
  plan and its verification record are retained under `llm_artifacts/`.
- A full eight-tomus build was started separately in `build_facsimile/`. Use a separate
  output directory for this plan's verification. Do not overwrite that build's
  outputs or stop its process as part of implementing this plan.

## Desired end state

- Tomus I's Monitum consists of one re-typeset prose page followed immediately by
  one combined errata image page. If prose genuinely spans more than one page, the
  combined page follows the last prose page immediately.
- The combined page contains p12's crop above p13's crop, with a small visible gap.
  Both images preserve their aspect ratios and share one additional scale factor.
  Neither scan is clipped, distorted, re-keyed, or moved to a separate page.
- The first body unit starts immediately after the combined page. Its logical body
  number is 1, and its existing inline heading and `empty` page style are preserved:
  no folio or running head is printed on that opening page. The following body page
  prints folio 2 with the existing running head.
- For the same verification fixture, content occupies PDF pages 14 (prose), 15
  (combined errata), and 16 (first body page), with no blank page between them.
  Existing flyleaves and end-of-document signature padding remain intentional blanks.
- Other front-matter scans, the Praefatio, later body units, restricted runs, and
  other tomi retain their existing page-start behavior.
- `MONITUM_PAGES` is 2; the README describes two crops on one errata page.
- Focused tests, the full Rust suite, clippy with warnings denied, formatting, and
  a compiled and visually inspected scoped PDF all pass.

## Tech Learnings

`\clearpage` ends pending content without forcing a page parity. On an already empty
page after a preceding `\clearpage`, it does not manufacture another blank page.
`\freshpage` adds a parity decision, which is inappropriate between consecutive
parts of the Monitum.

A LaTeX savebox measures real rendered content. Stacking the images into one box
and shrinking that box only if its measured height exceeds a budget preserves both
images' relative scale and makes the stack indivisible for page breaking. This is
more robust than independent maximum-height constraints on the two images.

## Phase 1 — Add one measured, combined scan page — Suggested implementer: Sonnet

### Step 1.1 — Implement and test a shared-scale scan-pair renderer

In `src/latex.rs`, add:

```rust
pub(crate) fn embed_scan_pair(first: &str, second: &str) -> String
```

Keep `embed_scan` unchanged. Reuse the already loaded `graphicx` package; add no
image-processing dependency and create no composite PNG. The helper emits one
page with the following exact layout contract:

- Begin with `\clearpage`, not `\freshpage`, followed by `\thispagestyle{empty}`.
- Use a local TeX group and a local box register (for example `\setbox0`) so this
  helper requires no new global preamble declaration and leaks no register state.
- Set box 0 to a `\vbox` containing `\offinterlineskip`, an `\hbox` with the first
  `\includegraphics[width=\textwidth]`, a `\vskip 2mm`, and an `\hbox` with the second
  image at the same width. Do not wrap this in `center`, whose list spacing consumes
  additional height. Both includes must occur exactly once and in source order.
- Set a local height budget to `\dimexpr\textheight-\baselineskip\relax` (for example
  `\dimen0`). The one-baseline reserve gives paragraph placement some slack and
  keeps the image box below the page-builder limit.
- Compare `\dimexpr\ht0+\dp0\relax` with that budget. When larger, output
  `\resizebox{!}{\dimen0}{\copy0}`; otherwise output `\copy0` without enlargement.
  Put the selected box in `\noindent\makebox[\textwidth][c]{...}` so the shrunken stack
  is horizontally centered. Emit `\par` before ending the local group and finish
  with `\clearpage`.
- Suppress incidental TeX whitespace with `%` at the box and conditional boundaries.
  The centered box must contain only the selected stack, without leading/trailing
  interword spaces from indentation or branch newlines. The images have zero depth;
  the measured vertical box therefore has zero depth too, so unstarred `\resizebox`
  scales the measured height correctly. Keep that invariant explicit in the helper.
- Prefer a single `format!` template following `embed_scan`'s existing convention.
  Escape literal TeX braces correctly. Comments describe the measured box and
  consecutive-page contract, without references to this plan.

Add these tests to the existing `src/latex.rs` test module:

- `embed_scan_pair_preserves_order_and_single_page`: both supplied PNG names occur
  exactly once, first before second, with one opening and one closing `\clearpage`,
  `\thispagestyle{empty}`, and no `\freshpage` or `center` environment.
- `embed_scan_pair_measures_and_shrinks_the_shared_box`: the emitted TeX includes
  the full-width images, 2 mm gap, `\ht0+\dp0` comparison, reserved height budget,
  conditional resize of the entire box, and unscaled copy branch. This is a source
  contract test; compilation in Phase 3 proves real pagination.

#### LLM verification

Run with `$env:CARGO_TARGET_DIR = "$PWD\target"` when the configured external target
directory is unavailable:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test embed_scan_pair
cargo test embed_scan_includes_named_png_scaled
cargo build
```

All commands must succeed. Inspect the generated template for balanced groups and
correct literal braces; tests cannot establish TeX page-builder behavior.

#### Human verification

Read the helper's emitted template and confirm the two crops form one box, with a
single shared shrink and no page-parity request. PDF verification follows in Phase 3.

## Phase 2 — Connect the consecutive Monitum sequence — Suggested implementer: Sonnet

### Step 2.1 — Assemble one errata page and suppress the first body's parity break

In `src/facsimile/assemble.rs::inner_document`, replace the two Monitum `embed_scan`
calls with one `embed_scan_pair(frontmatter::front_monitum_a_name(),
frontmatter::front_monitum_b_name())`. Keep the existing `emit_front_matter` and
`vol.tomus == 1` guards and placement immediately after `doc.push_str(&vol.prose)`.

Change the private helper signature to:

```rust
fn unit_block(unit: &PlacedUnit, force_parity_start: bool) -> String
```

Its opening string is `\freshpage\n` when the flag is true and `\clearpage\n` when
false. Keep one shared implementation for the remaining heading, marks, images,
and trailing page breaks. Do not duplicate the body renderer.

Enumerate `vol.units` in `inner_document`. Pass false for the first unit only when
`vol.emit_front_matter && vol.tomus == 1`; pass true in every other case. Leave
`\beginbodynumbering` immediately before the unit loop: with the first parity break
suppressed, it records the actual first body page. The false path must also work
when cached prose is missing, since the combined errata page is still emitted.

Update existing direct `unit_block` test calls to pass true, preserving their
existing expectations. Update the helper's doc comment to describe both page-start
choices rather than saying every unit always starts with a parity break.

Two existing assembly tests use an emitting Tomus I fixture even though they are
testing body-only behavior. In `inner_tex_starts_each_unit_fresh_page` and
`inner_tex_includes_one_image_per_page`, make the existing `vol` binding mutable
and set `vol.emit_front_matter = false` before calling `inner_document`. Preserve
their current assertions. This keeps the parity test on the unchanged restricted
path and prevents the new pair's full-width image includes from being counted as
body images. The new tests below cover emitting Tomus I explicitly.

Give `embed_scan_pair` one descriptive emitted marker,
`% Embedded front-matter scan pair`. Extend `tomus_one_embeds_monitum_errata_images`
so it checks that marker once, image order, and one page wrapper. Add:

- `monitum_errata_and_first_body_use_consecutive_pages`: use a Tomus I volume with
  a prose placeholder and two units. Between the prose marker and the first body
  image there is no `\freshpage`; the combined scan block and first body use
  `\clearpage`. The second unit retains `\freshpage`.
- `first_body_keeps_parity_start_without_monitum`: both a non-Tomus-I emitting
  volume and a restricted Tomus I volume retain the first unit's `\freshpage`.
- `monitum_without_cached_prose_still_uses_consecutive_pages`: an emitting Tomus I
  with empty prose still emits the combined image page and suppresses the first
  body's parity break.

Keep the existing checks that other tomi and restricted runs contain neither
Monitum image. Keep `facsimile_frontmatter_order`, running-head checks, and the
flyleaf/signature tests intact.

#### LLM verification

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test assemble
cargo test monitum
cargo build
```

#### Human verification

Inspect a generated Tomus I TeX fragment: prose's closing `\clearpage`, combined
image page, `\beginbodynumbering`, then first body heading, with no intervening
`\freshpage`. Confirm later body units still request their normal parity start.

### Step 2.2 — Update the page estimate and README

In `src/volumes.rs`, set the numeric `MONITUM_PAGES: u32` from 3 to 2 and describe
it as one re-typeset prose page plus one combined errata image page on Tomus I.
Do not rename the constant or change the prose-page list in `frontmatter.rs`.

Add `monitum_front_overhead_counts_two_content_pages` to the volumes tests, asserting
`MONITUM_PAGES == 2` and `front_overhead(0, true, true) == 6` (Tomus title + QR +
Volumen title + colophon + two Monitum pages). Existing expectations expressed
through constants should continue to pass without rewriting their identifiers.

Rewrite the front-matter paragraph in `README.md` in place: the Monitum prose is
followed immediately by one page containing the p12 and p13 errata crops, scaled
together. Preserve the documented p13 exclusion and fully re-typeset Praefatio.
Describe the first body page following directly; do not imply all other front-matter
parity breaks have been removed.

#### LLM verification

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test volumes
cargo build
rg -n 'Monitum|errata' README.md
git diff --check
```

#### Human verification

Read the README paragraph and constant comment and confirm they describe two
Monitum content pages, with the two crops on the second page.

## Phase 3 — Compile and verify actual page adjacency — Suggested implementer: Opus

### Step 3.1 — Regenerate a scoped fixture and inspect the resulting PDF

Use `build_facsimile/monitum_combined_check/` as the output directory and
`build_facsimile/monitum_combined_check_units.json` as a temporary unit table. These
are beneath the ignored build directory. Copy the first Theseus row from the
committed unit table, preserve its fields, and add `last_page: 14`. Write a JSON
array containing that row as UTF-8 without a BOM. Do not edit the committed table.
Run the normal path without `--unit` or `--pages`, which disable front matter:

```powershell
cargo build --bin facsimile
.\target\debug\facsimile.exe --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --units build_facsimile/monitum_combined_check_units.json --out build_facsimile/monitum_combined_check --offline --no-compile
```

Compile `tomus_1_inner.tex` from inside that output directory with installed MiKTeX:

```powershell
& 'C:\Users\majad\AppData\Local\Programs\MiKTeX\miktex\bin\x64\lualatex.exe' -interaction=nonstopmode -halt-on-error tomus_1_inner.tex
```

Render the relevant PDF pages using Poppler and inspect their images. Use `pdfinfo`
to confirm the PDF is valid and `pdftotext` to locate the prose ending and first
body page, but use the rendered page images to verify the Greek errata. Image-only
errata pages naturally have no extracted text; empty text extraction does not
prove a page is blank.

Acceptance checks:

- With the unchanged fixture and preceding front matter, pages 14, 15, and 16 are
  respectively prose, combined errata, and the first Theseus body page. There are
  no intervening blanks. Page 16 retains the empty opening-page style; page 17 is
  the second Theseus body page and prints folio 2 with its running head.
- The combined page contains the six p12 lines above all p13 lines. Check the first
  and last line of each crop, all edge accents, their gap, and the final Comparatio
  entry against the existing standalone crops and source scans.
- The combined page has no source furniture, clipped characters, distortion,
  running head, or folio; the resized text is readable at the actual trim size.
- There is no errata duplication in the prose and no missing-sentinel warning.
- The log has no overfull vertical box or unexpected additional page from the
  combined helper. Existing small horizontal prose overfull boxes were recorded
  in the prior verification and are outside this change's scope.
- The scoped PDF may remain 32 physical pages because signature padding absorbs
  the reduction. Total padded page count alone cannot establish success.

Add an automated rendered regression, not just string assertions. In the existing
`src/facsimile/assemble.rs` test module, add
`monitum_combined_compile_has_adjacent_content_pages`, marked `#[ignore]` because
it requires the source PDFs, cached front matter, Poppler, and MiKTeX. Follow the
crate's existing temporary-directory test convention. Construct its Tomus I
volume using the real sentinel-truncated cached prose, render the front scans via
`render_front_scans`, and use two small synthetic body PNGs (created with the existing
`image` crate) as the Theseus body's two images. Use a 300 × 100 pixel white image
with a black inset rectangle, which fits comfortably below the inline heading when
scaled to text width. Reuse it under two distinct body filenames. Build the volume
through the test module's existing `volume_full` and `unit` helpers; do not copy
their struct-construction logic. Write the inner using `write_inner` and compile it
using `compile::compile(dir.path(), "tomus_1_inner.tex", compile::INNER_PASSES)`.
Read its page count using `compile::inner_page_count(dir.path(), "tomus_1_inner")`
and inspect `1..=page_count` with `render::page_text`:

- Normalize extracted text for matching only, by joining `split_whitespace()` with
  spaces. Do not normalize cached source text or generator output. Require exactly
  one page containing `mutanda esse significavit.`. Find the first later page
  containing `THESEUS`; do not search from the beginning, because the Tomus title
  also names the Life. Require `body_page == prose_end_page + 2`.
- On that opening body page, require no extracted line consisting only of decimal
  digits, consistent with its empty page style. On `body_page + 1`, require the
  running head `THESEUS` and an extracted line whose trimmed contents are exactly
  `2`. This verifies numbering without incorrectly requiring a visible opening folio.
- Require `render::pdftotext_available()` before extracting page text. Missing text
  extraction must fail clearly: `page_text` deliberately returns an empty string
  when the tool is unavailable or fails. Assert the expected sentinel and title
  matches, rather than treating missing extracted text as an acceptable blank.
- Invoke Poppler `pdfimages -list <inner_pdf>` from the test with
  `std::process::Command`, requiring successful exit. Use `PLUTARCH_PDFIMAGES` when
  set, otherwise `pdfimages` on PATH. This override is local to the integration
  test; do not introduce a production subprocess abstraction or change rendering
  modules for this one check. Read the two crop dimensions with `image::image_dimensions`.
  Parse whitespace-separated output rows whose first two fields are numeric page
  and image indices; for rows of type `image`, compare the next width/height fields
  with each crop's dimensions. Require each distinct crop dimension pair exactly
  once in the PDF, with both rows on `prose_end_page + 1`. Ignore heading/separator
  rows and image rows of unrelated dimensions. This establishes that both actual
  source crops are embedded on the intervening page.
- Render the intervening page with `render_page` at 100 DPI and use a coarse
  grayscale ink count to require substantial dark content in both its upper and
  lower halves. Count pixels darker than 128 and require at least 1% in each half.
  This is a supplementary placement/blankness check; by itself it cannot prove
  both crops are present, since p13 alone can fill both halves.
- Render into a `renders/` child of the test's temporary directory. The source
  cache is loaded read-only via `frontmatter::load_cached_front_matter` with both
  source paths, then the prose is obtained through `front_prose_pages_until` and
  the existing Monitum constants. Use `Pdf::open` for v1 and call
  `render_front_scans(0, true, true, &pdf, dir.path())` for the real scans.
- Panic with page numbers, paths, and subprocess output when a check fails. Do not
  silently skip the ignored test when dependencies are missing. The implementer
  must explicitly run it once in the configured environment.

If actual TeX layout needs refinement, adjust the helper's reserved height or box
placement and repeat compilation and visual review. Do not change the source crop
constants or remove lines to force a fit. If the shared reduction proves too small
to read, report that finding rather than silently sacrificing source content.
Document the final measured fit and page positions in a new dated verification
note under `llm_artifacts/`; retain the prior plan and verification note.

#### LLM verification

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test monitum_combined_compile_has_adjacent_content_pages -- --ignored --nocapture
pdfinfo build_facsimile/monitum_combined_check/tomus_1_inner.pdf
git diff --check
```

Set `PLUTARCH_LUALATEX` to the installed executable for the automated test, and set
the existing Poppler overrides if those tools are absent from PATH. `pdfimages`
ships with the same Poppler installation; set the test's `PLUTARCH_PDFIMAGES`
override if necessary. Run the test from the repository root so it can locate the
source PDFs and cache. The ordinary suite continues to ignore this external-tool
test, matching the existing ignored compile tests in `src/latex.rs`.

#### Human verification

Open the scoped PDF at actual trim size. Read the prose ending, inspect both scan
sections on the following page, and turn directly to the first body page. Confirm
readable Greek, complete lines, no blank transitions, and visible body folio 2 on
the second body page. Confirm the first body page keeps its empty style. Compare
with the user-reviewed `monitum_check` PDF to verify the intended improvement.

### Step 3.2 — Final green gate

Run the final checks after the last Rust/layout edit:

#### LLM verification

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
git diff --check
```

All must pass. Record the explicitly executed ignored regression separately from
the ordinary suite. Confirm the transcription cache, unit table, crop constants,
global `\freshpage` definition, and unrelated scan helper are unchanged. Do not
overwrite the separate full-build output as part of this gate.

#### Human verification

Read the final README description and the verification note alongside the scoped
PDF. Confirm the documented two-page Monitum and immediate body transition match
the delivered behavior.
