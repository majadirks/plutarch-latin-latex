# Context

Each full facsimile Tomus currently has a derived title page with a hard-coded
`EX LIBRIS` / `MARTINEZ-DIRKS` ownership plate. Make that plate optional and
personalizable through `--exlibris "NAME"`, and surround both lines with an
oblong border. Document the argument with a generic owner name.

This is an implementation plan, authored on 2026-10-05 on branch
`plan/optional-exlibris-plate`. Implementation is subsequent work.

# Current state of the world

- `src/bin/facsimile.rs` defines the clap `Args` struct, constructs
  `FacsimileArgs` in `main`, and contains CLI parsing tests.
- `src/facsimile/mod.rs` defines `FacsimileArgs`; its production build loop calls
  `assemble::write_inner(vol, &args.out, &fonts)` for each output volume.
- `src/facsimile/assemble.rs` defines `FacsimileVolume`, `write_inner`, and
  `inner_document`. When `vol.emit_front_matter` is true, `inner_document`
  collects Life titles and calls `latex::tomus_title_page(vol.tomus, &lives)`.
  Restricted `--unit` / `--pages` builds omit this front matter entirely.
- `src/latex.rs::tomus_title_page` emits the bibliographic lines and Life list
  above a flexible gap, then an unconditional centered two-line ownership plate
  near the bottom. The name is hard-coded. Its final `\clearpage` ends the page.
- `src/latex.rs::escape_latex` already escapes user text safely for LaTeX while
  preserving Unicode. Reuse it. The inner preamble does not load TikZ; the
  printing wrapper's TikZ dependency is separate.
- Existing tests `tomus_title_page_renders_fixed_lines_and_small_caps` in
  `src/latex.rs` and `tomus_title_page_lists_only_lives` in
  `src/facsimile/assemble.rs` expect the hard-coded plate. Many assembler tests
  call `inner_document` or `write_inner` directly.
- The assembler test module already provides `fixture_image`, `volume_full`,
  and `compiled_fixture`. The last helper compiles with `compile::compile`,
  checks signature padding, extracts page text, and lists image placements.
  Rendered tests use an explicit `#[ignore]` for LuaLaTeX/Poppler requirements.
- `README.md` has separate facsimile and transcription instructions. The
  transcription binary (`src/main.rs`, `src/pipeline.rs`) uses a different
  title-page path and currently has no such ownership plate.

# Desired end state

- The facsimile command accepts `--exlibris "NAME"`, exposed in help as
  `--exlibris <NAME>`. Omission produces no ownership label, name, or border.
- Supplying a name produces exactly one plate on every emitted Tomus title
  page. `--exlibris "Martinez-Dirks"` displays `MARTINEZ-DIRKS` inside the border.
  Convert the owner name with Rust's Unicode-aware `str::to_uppercase`, then
  pass it through `escape_latex`. Keep spaces and punctuation as supplied.
- Reject explicitly supplied empty or whitespace-only names with a clear clap
  argument error. Use trimming to validate emptiness, without trimming valid
  values. Programmatic `FacsimileArgs` callers receive the same validation
  before any PDF access or output mutation.
- The border is an oblong in the sense of a horizontal rectangle, enclosing
  both `EX LIBRIS` and the owner name. Use standard LaTeX `\fbox` around a
  centered `\parbox`, without introducing a package or drawing dependency.
- The plate retains the existing two-line typography: `EX LIBRIS`, a `0.6em`
  gap, and the owner name in `\large`. The bibliographic block remains in the
  upper portion of the page; ownership stays near the bottom. Omission emits
  no plate markup or placeholder. Exact title coordinates need not match
  between omitted, ordinary, and wrapped-name cases.
- The acceptance samples are `Alex Smith`, `Alexandra Catherine Elizabeth
  Montgomery Smith`, and `Élodie & André`. Each must fit within one title page,
  with readable text and no plate-related overfull boxes or clipping. The long
  multiword sample may wrap. These samples must preserve page counts and body
  starts relative to omission. This is not a guarantee for arbitrarily long
  names or uninterrupted strings; do not add truncation or shrink-to-fit logic.
- Restricted builds still omit the complete title page even when the flag is
  supplied. The option does not affect unit detection or line-review exit modes.
- `README.md` explains omission, uppercase normalization, and restricted-build
  behavior, and demonstrates `--exlibris "Alex Smith"`. Scope the argument to
  the facsimile command; do not add it to the transcription binary.
- Transcribed Latin, sidecars, source scans, corpus titles, and their existing
  rendering remain unchanged. Uppercase normalization applies to the owner
  name alone. Ownership is a rendering option, never cached source text.

# Phase 1 — Optional bordered plate end to end — Suggested implementer: Sonnet

## 1.1 Implement the argument, pass it to rendering, and update tests

Make the following changes together so the crate compiles at the end of this
step and the CLI option has its documented effect immediately:

LaTeX markup below is literal generated `.tex`: commands have a single
backslash, while `\\` is the intentional LaTeX line-break command. In ordinary
Rust string literals escape each backslash as `\\`; raw Rust string literals
retain the literal LaTeX spelling.

1. In `src/bin/facsimile.rs::Args`, add `exlibris: Option<String>` with
   `#[arg(long, value_name = "NAME", value_parser = parse_exlibris)]` and help describing the optional uppercase
   ownership plate. Do not set a default or a default missing value. Wire
   `args.exlibris` into the `FacsimileArgs` initializer in `main`. Add
   `fn parse_exlibris(name: &str) -> Result<String, String>` using the explicit
   `std::result::Result` spelling to avoid the imported `anyhow::Result` alias.
   It calls the shared validator below, maps its error to a string for clap,
   and returns the original name unchanged on success.
2. In `src/facsimile/mod.rs::FacsimileArgs`, add the documented
   `pub exlibris: Option<String>` field. Update every struct literal, including
   test fixtures, with `exlibris: None` unless testing a supplied name.
   Add `pub fn validate_exlibris_name(name: &str) -> Result<()>` in this module.
   Reject `name.trim().is_empty()` with `exlibris name must not be blank`;
   otherwise succeed. Import and reuse it in the CLI parser. At the start of
   `run_facsimile`, validate a supplied name before PDF access or output
   mutation. Keep this rule in one definition, without repeating the predicate
   in the parser or renderer. Rendering continues to perform case conversion
   and escaping in its own helper.
3. Change the assembler signatures to
   `write_inner(vol: &FacsimileVolume, out_dir: &Path, fonts: &Fonts, exlibris: Option<&str>) -> Result<PathBuf>`
   and
   `inner_document(vol: &FacsimileVolume, fonts: &Fonts, exlibris: Option<&str>) -> String`.
   Pass the option from `write_inner` to `inner_document`; in the production
   build loop pass `args.exlibris.as_deref()` to `write_inner`. Update every
   direct caller in tests and other modules. Use `None` for unrelated fixtures.
   Do not store this run-wide option on every `FacsimileVolume` or change
   `Fonts`/`InnerOpts` to carry ownership information.
4. Change `src/latex.rs::tomus_title_page` to accept a third parameter,
   `exlibris: Option<&str>`. Forward it from the existing guarded title-page
   call in `assemble::inner_document`; preserve the `emit_front_matter` gate.
5. Add one private helper in `src/latex.rs`,
   `fn exlibris_plate(name: &str) -> String`. Normalize and escape the name in
   this helper alone. Have `tomus_title_page` generate the optional fragment
   with `exlibris.map(exlibris_plate).unwrap_or_default()`.
6. The helper emits a local group with `\fboxrule` set to `0.4pt` and
   `\fboxsep` set to `8pt`, enclosing
   `\fbox{\parbox{0.72\textwidth}{\centering EX LIBRIS\\[0.6em]{\large <escaped uppercase name>}\par}}`.
   Wrap that box in the existing centered plate area. Grouping prevents border
   settings from leaking into later pages. The fixed width gives the plate its
   horizontal shape and allows multiword names to wrap at ordinary word
   boundaries. Meet the explicit acceptance samples above; do not promise
   unrestricted overflow prevention. Keep the `1cm` bottom space.
7. Preserve `\null\vfill` before the bibliographic block and
   `\vfill\vfill` after it. Insert the optional fragment from item 5 after
   that gap, then retain the `1cm` bottom space and final `\clearpage` in
   both cases. The helper owns the centered plate environment and all box
   markup, so `None` inserts an empty string. Do not add a phantom, reserved
   plate box, or second box-construction helper. Judge the upper bibliographic
   position visually, without requiring identical coordinates across cases.
8. Rewrite the affected comments and doc comments to describe the optional
   owner plate and current layout, without hard-coded ownership or historical
   refactor narration. Keep the existing bibliographic and Life-list assertions
   in their tests, passing `None`; move ownership expectations into focused
   tests. Remove obsolete assertions that every default document contains
   `MARTINEZ-DIRKS`.

Add these automated tests in the existing test modules:

- `facsimile_exlibris_flag_defaults_to_none`: a valid baseline CLI without the
  option parses to `None`.
- `facsimile_exlibris_flag_accepts_name`: a single argument `Alex Smith`
  parses to `Some("Alex Smith")`; retain the raw value until rendering.
- `facsimile_exlibris_flag_requires_value`: supplying the flag without a value
  yields a clap error.
- `facsimile_exlibris_flag_rejects_blank_name`: empty, space-only, and tab-only
  arguments fail with the shared validation message; a valid value with leading
  or trailing spaces is accepted unchanged.
- `validate_exlibris_name_rejects_blank`: the shared validator rejects the same
  blank samples and accepts `Alex Smith`.
- `run_facsimile_rejects_blank_exlibris_before_pdf_access`: with nonexistent
  source paths and a blank owner, the returned error is the name-validation
  error and no output directory is created.
- `tomus_title_page_omits_visible_exlibris_by_default`: `None` emits no
  `EX LIBRIS`, `\fbox`, or phantom markup; the normal title-page ending and
  bibliographic lines remain present.
- `tomus_title_page_renders_bordered_uppercase_exlibris`: `Some("Alex Smith")`
  emits a visible `\fbox`, both lines, `ALEX SMITH`, local border settings,
  and the chosen width; no mixed-case owner text remains.
- `exlibris_plate_uppercases_unicode_and_escapes_latex`: test an accented name
  and LaTeX special characters (including backslash, braces, ampersand, percent,
  underscore, hash, dollar, tilde, and caret), using the established escaping
  expectations from `latex_specials_escaped_text_preserved`.
- `tomus_title_page_preserves_requested_legacy_owner`: the supplied name
  `Martinez-Dirks` yields the expected uppercase owner inside the box.
- `inner_document_propagates_exlibris_to_each_tomus`: construct at least two
  emitting volumes, including a later Tomus, and check exactly one visible
  personalized plate in each document. The `None` counterparts have none.
- `restricted_volume_omits_exlibris_even_when_requested`: an
  `emit_front_matter: false` fixture with a supplied name has neither the title
  page nor the plate.
- `write_inner_writes_requested_exlibris`: write a small fixture to a temporary
  directory, read the file back, and confirm the supplied name reaches the
  emitted plate through `write_inner`.

### LLM verification

Run `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test exlibris`,
`cargo test tomus_title_page`, and `cargo build --bins`.
Inspect `cargo run --bin facsimile -- --help` for `--exlibris <NAME>`.
Use `rg -n 'MARTINEZ-DIRKS|exlibris|tomus_title_page|write_inner|inner_document' src`
to verify propagation and ensure the old hard-coded owner remains only in the
explicit regression test. Confirm no normalization touches transcription data.

### Human verification

Read the helper and generated `.tex` for an emitting fixture. Confirm the border
encloses both ownership lines and user characters are escaped. Compare default
and personalized title-page PDFs in Phase 2, checking clean omission and
upper-title positioning. Review the choice of a rectangular oblong.

# Phase 2 — Documentation and rendered acceptance — Suggested implementer: Sonnet

## 2.1 Update the README in place

In `README.md`, extend the existing facsimile command examples with a complete
command using `--exlibris "Alex Smith"`. Explain beside those examples that the
argument adds a bordered `EX LIBRIS` / `ALEX SMITH` plate to each full-build
Tomus title page, omission prints no plate, and names are converted to uppercase.
State that empty or whitespace-only names are rejected.
Explain that restricted `--unit` / `--pages` builds omit the title pages and
therefore the plate. Keep the default full-build command without the flag.
Rewrite the existing front-matter sentence about derived title pages to mention
the optional plate if that helps avoid redundant explanation. Use no
Martinez-Dirks example in the README. Do not add a revision/status block or
renumber existing lists.

### LLM verification

Run `rg -n -i 'exlibris|ex libris|Alex Smith|Martinez-Dirks' README.md`.
Compare the documented option with `cargo run --bin facsimile -- --help`.
Inspect `git diff -- README.md` for focused in-place edits and a complete command.

### Human verification

Read the facsimile instructions as a new user. Confirm they make it clear how
to request a plate, what case appears, and what happens when the option is absent.

## 2.2 Compile and inspect title pages, then run the final local gates

Add the ignored rendered test
`exlibris_compile_preserves_title_page_and_body_starts` to the existing test
module in `src/facsimile/assemble.rs`, using
`#[ignore = "requires MiKTeX/LuaLaTeX and Poppler"]`. Reuse `volume_full`,
`fixture_image`, and `compiled_fixture`, along with `write_inner`; do not build
a separate standalone generator or expose crate-private title-page helpers.

Use emitting volumes for Tomus I and Tomus II with the same small synthetic
body fixture. For each volume, generate omitted, ordinary-name, long-name, and
accented-name variants using the three exact acceptance samples in Desired end
state. Create each variant in a separate temporary directory. Populate every
image referenced by the fixture (QR, and for Tomus I the Volumen, colophon,
and Monitum pair) using `fixture_image` and the existing `frontmatter` filename
helpers. Use empty prose and no Praefatio assets so the test requires neither
source PDFs nor transcription caches.

Pass each variant through `compiled_fixture`. Compare extracted pages and image
placements with that volume's omitted baseline: equal page counts, one
bibliographic title page, unchanged first-body physical page, and unchanged
body folios and recto alignment. Normalize extracted whitespace for text
assertions, since PDF extraction may separate wrapped words differently.
Omitted PDF text must contain no `EX LIBRIS`; named PDF text must contain
exactly one `EX LIBRIS` and the complete uppercase sample name. Reject
plate-related overfull boxes in the compile logs. The ordinary plate must be
wider than it is tall; the multiword sample may wrap but must remain readable
and fit on the same title page.

For visual inspection, render the title pages with the existing Poppler render
helpers. If adding a preview helper is necessary, have the rendered test reuse
that same fixture setup rather than duplicate volume construction or compilation.
Preserve rendered page PNGs, their corresponding `.tex`, and compile logs under
`tmp/exlibris_preview/` in the workspace before temporary test directories are
dropped. Use subdirectories identifying the Tomus and name variant so repeated
filenames do not overwrite other samples. Both the LLM and human can inspect
these artifacts. This directory is a generated preview, not a historical plan
or note.

After the fixture checks, run one full offline build with the existing source
scans and front-matter sidecar to confirm real CLI integration:

```powershell
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/exlibris_acceptance/ --offline --exlibris "Alex Smith"
```

Use the README's `PLUTARCH_LUALATEX` setup when needed. This is a full build
because restricted builds intentionally cannot exercise the title page. Do not
use `--retag`, alter the transcription cache, or trigger paid vision calls to
validate a cosmetic rendering option. If required local inputs are absent,
record the real-build limitation; the synthetic rendered test still provides
title-page verification when LuaLaTeX and Poppler are available. If those tools
are unavailable, report rendered verification as incomplete.

Render representative first and later Tomus title pages from the inner PDFs.
Confirm the named plate is centered near the bottom, the border is wider than it
is tall for the ordinary name, and both text lines have comfortable padding.
Inspect the omitted and personalized fixture previews too. Confirm the default
page has no ownership text or border, the bibliographic block stays in the upper
portion, and named plates sit near the bottom. Verify readable wrapping in the
long multiword sample and correct glyphs and ampersand in the accented sample.
Inspect LaTeX logs for errors and plate-related overfull boxes. For these
acceptance samples, preserve page starts, body recto alignment, folios, and
signature padding; the option must not add a title page or change page counts.

### LLM verification

Run `cargo fmt --all`, `cargo fmt --all --check`,
`cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo build --bins`.
Run `cargo test exlibris_compile_preserves_title_page_and_body_starts -- --ignored --nocapture`
with LuaLaTeX and Poppler configured. Inspect the saved fixture previews and
their compile logs. Check that the full build produced inner and imposed PDFs;
a successful
generator exit with `lualatex not run` is insufficient for rendered acceptance.
Confirm the rendered fixture comparisons passed for page counts, body starts,
and extracted ownership text. Run `git diff --check` and inspect the final diff
for unintended cache
or source-text changes. Report any unavailable render verification explicitly.

### Human verification

Open first and later Tomus title pages from the fixture previews and real build.
Confirm the oblong
style is pleasing, both lines fit, omission is clean, and the bibliographic
title remains in its expected position. Inspect the long-name sample for readable
wrapping and the accented-name sample for correct glyphs. Confirm the README
example reproduces the shown personalized title page.
