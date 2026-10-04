# Facsimile shared-page boundaries: locate titles by OCR bounding box

## Context

The facsimile path reproduces the 1857 Didot Latin column pixel-for-pixel: it
crops the Latin column out of each source scan, slices it at inter-line
whitespace into line-bands, and reflows the bands into the trim text block
(`src/facsimile/reflow.rs`). Each Life/Comparatio ("unit") gets a generated,
typeset display title (`\heading{…}`) and opens on a fresh page
(`src/facsimile/assemble.rs`). The source, however, does **not** start each unit
on a fresh page: a unit's text opens partway down a source page it **shares**
with the end of the previous unit, divided by the previous unit's last lines, a
decorative separator rule, and the next unit's **printed** display title. The
reflow must split that one shared source page into "belongs to the previous
unit" (above the title) and "belongs to the next unit" (below the title),
dropping the separator and the printed title so only the generated title shows.

The current split (added in `2026_10_03_FACSIMILE_UNIT_AUTODETECT_PLAN.md` and
tuned in `2026_10_04_FACSIMILE_LAYOUT_FIXES_PLAN.md`) locates the printed title
on the shared page with an **image heuristic**: it scans the first 6 line-bands
(for a unit's opening page) or the last 6 line-bands (for a shared tail page) for
a band that is **narrow** (ink span < 0.6× the median line width) and **isolated**
(set off by an above-median gap). That heuristic fails on real boundaries, and
the failures are now visible in the compiled book. This plan replaces the
heuristic with a **deterministic boundary located from the OCR text layer's word
bounding boxes**, recorded in the committed, hand-editable unit table.

## Current state of the world

### The failures (confirmed against `build_facsimile/tomus_1_inner.pdf` and the raw scans)

Three defects, all one root cause — the printed display title on a shared source
page is not where the 6-band scan looks, or is not "narrow":

1. **Theseus → Romulus (inner p. 56).** Source v1 p. 33 carries ~13 lines of
   Theseus's tail, a `— — —` separator, the centered `ROMULUS.` title, then
   Romulus ch. I. `pdftotext -bbox` puts `ROMULUS.` at **yMin 278.94 of page
   height 714.24 ≈ 39 % down the column** — far outside the first-6-bands window.
   So `find_title_band` returns `None`, `drop_opening_bands` falls back to "keep
   from the first full-measure line," and Romulus's first composed image
   (`facsimile_romulus_1.png`) contains **Theseus's tail + the separator + the
   embedded `ROMULUS.` title + Romulus's body**, under the *generated*
   `\heading{ROMULUS}`. Symmetrically, Theseus's shared tail on p. 33 finds no
   title in its last-6-bands window, so `drop_shared_tail` drops the **whole**
   page and Theseus silently **loses its last lines** ("…appellamus.").

2. **Romulus → Comparatio (inner pp. 105–106).** Source v1 p. 56 carries
   Romulus's tail, the separator, the **two-line, full-width** title
   `COMPARATIO THESEI CUM / ROMULO.`, then the comparatio's ch. I. The first
   title line spans nearly the whole measure, so `is_narrow` is false and the
   title is not recognized even when in-window. Result: the embedded title stays
   on Romulus's last page (inner p. 105), and the comparatio's opening sections
   are **missing** from `facsimile_comparatio_thesei_et_romuli_1.png`, which
   begins mid-text instead of at "Habes quæ de Romulo et Theseo…"
   (`pdftotext -bbox` puts that title at yMin ≈ 550.7 / 719.5 ≈ 76 %, its body at
   yMin ≈ 595.2 ≈ 83 %).

3. **Blank page mid-unit (inner p. 115).** The comparatio's **last** composed
   image (`…_comparatio_…_8.png`) is near-blank — a few stray specks. These are
   fragments of the inter-unit separator rule (and OCR-column noise) that
   `drop_shared_tail` kept *above* where it located the next unit's (Lycurgus)
   title on the shared tail page; the fragments wrap onto their own trailing
   canvas, which lualatex renders as a blank folio with a speck. (The prior plan
   fixed *large* mid-unit blanks from tall-strip packing; this residual-speck
   blank is a different, boundary-driven cause.)

### Relevant code

- `src/render.rs`
  - `page_text(pdf, page) -> Result<String>` shells `pdftotext -enc UTF-8 -f p -l p pdf -`.
  - `running_head`, `title_token`, `page_count`, `render_page`. `pdftotext_bin()`
    resolves the binary (`PLUTARCH_PDFTOTEXT`).
- `src/facsimile/corpus.rs`
  - `match_page(page_tokens: &[String]) -> Option<usize>` resolves folded caps
    tokens to a canonical unit index; `corpus()` is the canonical table.
- `src/facsimile/detect.rs`
  - `tokens(s)`, `title_line_unit(page_text)` (first short ALL-CAPS line that the
    corpus resolves — pins `start_page` today, with **no vertical position**),
    `detect_openings(range, page_text_fn)`, `units_from_runs`, `load_or_detect`
    (writes `facsimile_units.json`, merging `pinned` rows).
- `src/facsimile/units.rs`
  - `struct Unit { id, latin_title, greek_title, kind, group, source, start_page,
    skip_top_bands: Option<u32>, last_page: Option<u32>, pinned: bool,
    review_notes: Option<String> }`.
  - `struct UnitRange { unit: Unit, end_page: u32, shared_tail: bool }`.
  - `ranges(units, source_last_body_page) -> Vec<UnitRange>` (sets `shared_tail`
    and `end_page = next.start_page` for consecutive same-source units).
- `src/facsimile/columns.rs`
  - `detect_latin_column`, `crop_column` — **crops to the full page height**
    (`crop_imm(img, left, 0, width, h)`), so a fraction of page height maps
    directly to a row of the cropped column.
- `src/facsimile/reflow.rs`
  - `unit_body_strips(range, render_fn)` — Pass A bands every page; Pass B applies
    `drop_opening_bands` (first page), `drop_interior_running_head` (interior),
    `drop_shared_tail` (shared last page).
  - `drop_opening_bands(col, bands, skip_top_bands, m)`,
    `drop_shared_tail(col, bands, m)`, `find_title_band` (scans `0..6`),
    `find_title_band_from` (scans `len-6..len`), `scan_title_band`
    (`is_narrow` + isolated-gap).
  - `compose_pages(...)` packs strips onto fixed-width canvases; `pack_canvas`.
- `src/facsimile/mod.rs`
  - `run_facsimile`, `select_ranges` (calls `units::ranges`), `render_gray`.

### The cached table

`facsimile_units.json` is committed and hand-editable; `--detect-units` /
`--redetect` regenerate it and preserve any row with `"pinned": true`. It
currently carries **no** vertical boundary information.

## Desired end state

The reflow splits every shared source page at the exact line the source's
printed display title begins on, located from the OCR word boxes and recorded in
the unit table, so that:

- **The previous unit keeps its full tail** (down to its last body line) and
  **nothing below the separator** — no lost lines, no kept separator, no printed
  title.
- **The next unit keeps its full body from its first line** — no inherited
  previous-unit tail, no separator, no printed title; only the generated
  `\heading{…}` title shows.
- A unit's composed images contain **no near-blank trailing page** and no stray
  speck pages.

Concretely, after a `--redetect` and full build:

- `facsimile_units.json` gains two optional fields per unit — `title_top_frac`
  and `body_top_frac` — populated for every unit that opens on a shared page.
- Inner p. 56 (Romulus's first page) shows the generated `\heading{ROMULUS}`
  immediately above "I. Ingens Romæ nomen…", with **no** Theseus tail and **no**
  embedded `ROMULUS.`; Theseus's last page ends with "…appellamus."
- The embedded `COMPARATIO THESEI CUM ROMULO.` no longer appears on Romulus's
  last page; the comparatio's first page opens at "Habes quæ de Romulo et
  Theseo…".
- There is no blank/speck folio inside the Comparatio (former inner p. 115).
- `cargo test`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo fmt --all --check` are clean.

### Approach chosen, and the alternative rejected

**Chosen — locate the boundary from `pdftotext -bbox` and store it in the unit
table.** The detector already OCRs each page and resolves its unit against the
corpus; extending it to record *where* the title sits (a fraction of page
height) is deterministic, offline at build time, auditable, and lands in the
same committed, `pinned`-able cache a human already edits. It is width- and
position-agnostic: a title at 39 % or 76 %, narrow or full-width, one line or
two, is handled identically.

**Rejected — keep improving the image heuristic** (scan the whole column;
recognize wide titles; special-case the separator). It stays fragile (every new
edition quirk is a new heuristic), cannot distinguish a genuinely short
paragraph-final line from a title without more cues, and offers no audit trail or
hand-pin when it guesses wrong. The image heuristic is **retained only as a
fallback** when a unit carries no recorded fractions (e.g. an un-redetected
cache), so the build never hard-fails on a missing box.

## Tech Learnings

- **`pdftotext -bbox`** emits XHTML: one `<page width="W" height="H">` per page,
  then `<word xMin="…" yMin="…" xMax="…" yMax="…">text</word>` per word, in
  reading order. Coordinates are in PDF points on the page's media box, origin
  top-left, y increasing downward — so `yMin / H` is the fraction of page height
  at the top of a word, directly comparable to `row / img.height()` of a
  full-height column crop. (`-bbox-layout` adds `<block>/<line>` grouping; this
  plan groups words into lines itself by `yMin` proximity, to avoid depending on
  poppler's layout analysis.)
- **The Latin column is the right half.** On these bilingual pages Greek is left,
  Latin right; the Latin display title and body words have `xMin > ~0.5·W`
  (verified: ROMULUS xMin 315/478; comparatio title xMin 244/482, body 257).
  Filtering to the right half isolates the Latin column without rendering.
- **The running head collides by text.** A page's running head repeats the unit
  name (e.g. `COMPARATIO THESEI CUM ROMULO`) at the very top, so the title must
  be matched *below* a top furniture zone (reuse the ~8–10 % band that
  `lines::TOP_FURNITURE_FRAC` already treats as furniture), not by text alone.
- **Fraction → row.** Because `crop_column` keeps full page height, a recorded
  `frac` maps to column row `(frac * col.height()).round()`. Comparisons use band
  **centers** with a small tolerance so a fraction landing inside an inter-line
  gap never clips the adjacent line.

---

## Phase 1 — Parse `pdftotext -bbox` and locate the title/body rows

**Suggested implementer: Opus** (OCR parsing, line grouping, and corpus matching
with coordinate and column filters; several judgment calls on tolerances).

### Step 1.1 — `page_words` in `src/render.rs`

Add a bounding-box reader beside `page_text`:

```rust
/// One OCR word with its page-fraction box (origin top-left, y down).
#[derive(Debug, Clone, PartialEq)]
pub struct Word {
    pub text: String,
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

/// One page's OCR words as fractions of page width/height, via
/// `pdftotext -bbox`. Empty vec when the binary is absent or the page has no
/// text layer (mirrors `page_text`'s soft-fail).
pub fn page_words(pdf: &Path, page: u32) -> Result<Vec<Word>>;
```

Shell `pdftotext -bbox -enc UTF-8 -f p -l p pdf -`, parse the XHTML with a
minimal hand-rolled scan (not a full XML dep): read `width`/`height` from the
first `<page …>` tag, then each `<word xMin=… yMin=… xMax=… yMax=…>TEXT</word>`,
dividing x by width and y by height. Unescape the five XML entities
(`&amp; &lt; &gt; &quot; &apos;`) in `TEXT`. Soft-fail (`Ok(vec![])`) on a
missing binary or non-zero exit, exactly like `page_text`.

**LLM verification.** New unit tests in `render.rs`:
- `page_words_parses_bbox_fractions` — feed a canned two-word XHTML string to a
  pure parser helper (factor the string-parsing out of the process call, e.g.
  `fn parse_bbox(xml: &str) -> Vec<Word>`), assert texts, and that `y_min` ≈
  `yMin/height` within 1e-4.
- `parse_bbox_unescapes_entities` — a word `R&amp;R` parses to `R&R`.
- `cargo test -p plutarch-reader render::`.

**Human verification.** Run
`PLUTARCH_PDFTOTEXT=… pdftotext -bbox -f 33 -l 33 Ploutarchou_Bioi_v1.pdf -` and
confirm the `ROMULUS.` word box matches the parser's fractions (~0.39 height).

### Step 1.2 — `title_box` in `src/facsimile/detect.rs`

Add a coordinate-aware title locator that, given a page's `Vec<Word>` and the
corpus unit index the page opens, returns the title's top and the body's top as
page-height fractions:

```rust
/// The vertical fractions `(title_top, body_top)` of `unit`'s printed display
/// title and its first body line on this page, from the OCR word boxes. `None`
/// when no title line below the top furniture zone resolves to `unit`.
pub fn title_box(words: &[Word], unit: usize) -> Option<(f32, f32)>;
```

Algorithm:
1. Keep words with `x_min > 0.5` (Latin column) and `y_min >
   TITLE_TOP_ZONE_FRAC` (skip the running head; set `TITLE_TOP_ZONE_FRAC = 0.10`).
2. Group the kept words into lines by `y_min` proximity (same line when the
   gap between sorted `y_min`s is under ~0.6 of the local line pitch; a simple
   fixed epsilon such as 0.012 of page height is adequate and tested).
3. Fold each line to caps title tokens (reuse `title_token` + `tokens`). Scan
   top-down for the first line whose tokens **intersect** `corpus()[unit]`'s
   folded `latin_title` tokens. This is the title's first line; keep consuming
   immediately-following lines that also intersect (the second line of a wide
   two-line Comparatio title). Record `title_top = first title line's y_min`.
4. `body_top = y_min` of the first line *after* the title block (any case). If
   none follows on this page, return `None` (degenerate; fall back).

Matching by **intersection with the known unit's** tokens (not a fresh
`match_page`) is what makes the two-line `COMPARATIO THESEI` / `ROMULO` title
resolvable even though neither line alone uniquely resolves.

**LLM verification.** Unit tests in `detect.rs` with canned `Word` vectors:
- `title_box_narrow_midpage` — a running-head line at y 0.06, Theseus-tail lines,
  a `ROMULUS` line at y 0.39, body below; asserts `title_top` ≈ 0.39 and the
  running head is ignored.
- `title_box_wide_two_line` — `COMPARATIO THESEI` at 0.765 and `ROMULO` at 0.787
  as one title block, body `Habes…` at 0.827; asserts `title_top` ≈ 0.765,
  `body_top` ≈ 0.827.
- `title_box_skips_left_greek_column` — the same title words duplicated at
  `x_min` 0.1 (Greek side) must be ignored.
- `title_box_none_when_absent` — a page of body-only lines returns `None`.
- `cargo test -p plutarch-reader facsimile::detect`.

**Human verification.** None (covered by the Phase 4 real-build eyeball).

---

## Phase 2 — Record the fractions in the unit table

**Suggested implementer: Opus** (threads an injected bbox source through
detection and widens the cache schema; must preserve the `page_text`-only
fallback and `pinned` semantics).

### Step 2.1 — Widen `Unit`

> **Drop `Eq`, keep `PartialEq`.** `Unit` (`units.rs:16`), `UnitRange`
> (`units.rs:54`), and `Run` (`detect.rs:119`) currently derive `Eq`; `f32`
> is not `Eq`, so every struct gaining a fraction field must change its derive
> from `PartialEq, Eq` to just `PartialEq`. This is safe: none of the three is
> used as a `HashSet`/`HashMap`/`BTreeSet` key or needs `Hash` (the only set in
> the facsimile code is `HashSet<(usize, u32)>` in `frontmatter.rs`), and
> `assert_eq!`/`dedup` require only `PartialEq`. `Run` keeps `Copy` (an
> `Option<f32>` is `Copy`). Do this derive change in the same step as the field
> addition so each step still compiles.

In `src/facsimile/units.rs` add, after `skip_top_bands`:

```rust
/// Page-height fraction at the top of this unit's printed display title on its
/// `start_page`, from the OCR boxes. Used to trim the previous unit's shared
/// tail (keep bands above it) and this unit's own opening. `None` on units that
/// open at the top of a non-shared page, or an un-redetected cache.
#[serde(default, skip_serializing_if = "Option::is_none")]
pub title_top_frac: Option<f32>,
/// Page-height fraction at the top of this unit's first body line on its
/// `start_page`. Used to trim this unit's opening page (keep bands from here
/// down). `None` as above.
#[serde(default, skip_serializing_if = "Option::is_none")]
pub body_top_frac: Option<f32>,
```

Update every `Unit { … }` literal (tests in `units.rs`, `mod.rs`,
`detect.rs`, and `select_ranges`'s synthetic `--pages` unit) to set both to
`None`. `pinned` rows must keep these fields verbatim across `--redetect`
(already true — `merge_pins` clones the whole pinned `Unit`).

### Step 2.2 — Populate during detection

`detect_openings` currently takes `page_text_fn: Fn(u32)->Result<String>`. Add a
parallel `page_words_fn: Fn(u32)->Result<Vec<Word>>`. After a run's `start_page`
is pinned, call `title_box(&page_words_fn(start_page)?, run.unit)` and carry the
`(title_top, body_top)` onto the `Run` (add two `Option<f32>` fields to `Run`;
drop its `Eq` derive per Step 2.1). `units_from_runs` copies them into the new
`Unit` fields.

Thread the real source through `load_or_detect` and `run_facsimile`
(`src/facsimile/mod.rs`): add a `page_words` closure beside the existing
`page_text` closure, both over `source_paths`. Keep detection soft on a missing
text layer: if `page_words` is empty, leave the fractions `None` (fallback
covers it).

**Update every `detect_openings` caller and test** in the same step so the crate
compiles: `load_or_detect` (both source loops), and the `detect.rs` tests that
pass a single closure — `start_page_pinned_by_display_title`,
`start_page_falls_back_to_run_when_no_title`, the two `load_or_detect_*` tests,
and the `full_source` real-PDF tests (`detect_source` helper). Tests that do not
exercise boxes pass a `|_| Ok(vec![])` words closure.

**First-unit and v2 layout are already handled** (verified via `pdftotext
-bbox`): on v1 p. 14 the work banner `PLUTARCHI`/`VITÆ` and `THESEUS.` all sit in
the right (Latin) half, and v2 p. 18 (`NICIAS`) confirms Greek-left/Latin-right
there too. So `x_min > 0.5` plus the top-zone filter isolates the title on a
first unit's banner page without special-casing: `VITÆ` is skipped by token
intersection (it is not a `theseus` token) and `PLUTARCHI` by the top zone, so
`title_box` returns the `THESEUS.` line and its body below — the generated
heading then replaces the embedded banner.

**LLM verification.**
- `detect_openings` test extended: canned `page_words_fn` for the Romulus start
  page yields `title_top_frac` ≈ 0.39 on the Romulus unit.
- `units_from_runs_carries_fracs` — runs with fractions produce units carrying
  them.
- A round-trip test in `units.rs`: a `Unit` with both fracs set serializes and
  parses back unchanged; a minimal row without them parses with `None`.
- `load_or_detect` test: the written JSON for the Theseus/Romulus/Comparatio
  fixture contains `title_top_frac` on Romulus and the Comparatio.
- `cargo test -p plutarch-reader facsimile::`.

**Human verification.** After the Phase 4 `--redetect`, open
`facsimile_units.json` and confirm `romulus`, `comparatio_thesei_et_romuli`,
`lycurgus`, … carry plausible `title_top_frac`/`body_top_frac` (0 < f < 1,
`title_top_frac < body_top_frac`).

---

## Phase 3 — Split shared pages by fraction in the reflow

**Suggested implementer: Opus** (reworks the boundary trims and their fallback,
the core correctness change; needs care with the fraction→row mapping and the
separator drop).

### Step 3.1 — Carry the fractions onto `UnitRange`

In `src/facsimile/units.rs` add to `UnitRange`:

```rust
/// This unit's own `body_top_frac` (its opening page's first body line). Drives
/// `drop_opening_bands`.
pub open_body_frac: Option<f32>,
/// The *next* unit's `title_top_frac` when `shared_tail` is set — the line this
/// unit's shared tail is cut at. `None` otherwise. Drives `drop_shared_tail`.
pub tail_cut_frac: Option<f32>,
```

In `ranges`, set `open_body_frac = unit.body_top_frac`, and when the next unit is
in the same source, `tail_cut_frac = next.title_top_frac` (else `None`). Update
`ranges` tests and the synthetic `--pages`/`--unit` ranges accordingly.
(`UnitRange` loses its `Eq` derive here per Step 2.1; its `assert_eq!` tests keep
working under `PartialEq`.)

Add one shared mapping helper so the two trims cannot drift in sign:

```rust
/// The column row a page-height fraction maps to, less a half-median-line
/// tolerance so a fraction landing in an inter-line gap never clips the
/// adjacent line. `None` on the sentinel metrics (no usable line height).
fn frac_row(frac: f32, col_h: u32, m: &LineMetrics) -> Option<u32>;
```

### Step 3.2 — Fraction-driven opening trim

Rewrite `drop_opening_bands` (`src/facsimile/reflow.rs`) to prefer, in order:
1. `skip_top_bands` override (unchanged — exact-count manual escape hatch).
2. **`open_body_frac`**: with `row = frac_row(frac, col.height(), m)`, **keep**
   every band whose center `>= row` (drop everything above the body's first
   line — the previous tail, separator, and printed title, regardless of their
   width or depth).
3. Existing image heuristic (`find_title_band` → full-measure fallback) when no
   frac is present (or `frac_row` is `None` under sentinel metrics).

`drop_opening_bands` must receive `open_body_frac`; thread it from
`unit_body_strips` (which has the `UnitRange`). Its existing tests
(`first_page_excludes_title_and_crossref_bands`,
`skip_top_bands_override_drops_exact_count`) gain the new `None` argument and
stay green on the fallback path.

### Step 3.3 — Fraction-driven shared-tail trim

Rewrite `drop_shared_tail` to prefer:
1. **`tail_cut_frac`**: with `row = frac_row(frac, col.height(), m)`, **keep**
   every band whose center `< row` (drops the printed title and everything
   below). Then drop a **trailing separator**: if the last kept band is
   `Short`/`Narrow` (via `classify_band`) and detached by an above-median gap,
   drop it — reuse the trailing-band test already in `drop_interior_running_head`
   by extracting it into a shared
   `fn drop_trailing_furniture(col, bands, m) -> Vec<Band>`.
2. Existing `find_title_band_from` heuristic when no frac is present (or
   `frac_row` is `None`).

Thread `tail_cut_frac` into `drop_shared_tail` from `unit_body_strips`. Its
existing tests (`bottom_trim_keeps_previous_tail`, `single_page_unit_trims_both_ends`)
gain the new `None` argument and stay green on the fallback path.

**LLM verification.** New/updated tests in `reflow.rs`, using the existing
synthetic-column fixtures (`page_and_metrics`):
- `opening_frac_drops_prev_tail_separator_title` — a page `[prev tail ×3]
  [separator][TITLE][body ×3]` with `open_body_frac` at the body's top keeps
  exactly the 3 body bands, regardless of the title's width.
- `opening_frac_handles_wide_title` — same with a **full-measure** title band; the
  frac path still keeps only the body (the old `find_title_band` would fail here).
- `shared_tail_frac_keeps_prev_body_drops_separator` — a page `[prev body ×3]
  [separator][TITLE][next body]` with `tail_cut_frac` at the title keeps the 3
  prev-body bands and drops the trailing separator.
- `opening_falls_back_to_heuristic_without_frac` — `open_body_frac = None`
  reproduces the current `drop_opening_bands` result on the existing fixture
  (guards the fallback).
- Keep `top_trim_drops_previous_tail_and_title`, `bottom_trim_keeps_previous_tail`,
  `single_page_unit_trims_both_ends` green (they exercise the fallback).
- `cargo test -p plutarch-reader facsimile::reflow`.

**Human verification.** Deferred to Phase 4 (the real boundaries).

---

## Phase 4 — Suppress residual near-blank canvases and specks

**Suggested implementer: Sonnet** (localized, well-specified safety net in
`compose_pages`).

Defence in depth for issue 3, independent of the boundary fix: a stray speck
(marginal tick, separator fragment, OCR noise) that survives furniture and
boundary trims must never become a page.

### Step 4.1 — Drop negligible-ink strips

In `strips_in_reading_order` (or just before packing in `compose_pages`), drop
any strip whose **dark-pixel count** — summed over the band's rows from
`lines::ink_profile_rows` (not `band_ink_span`, which is a width) — is below
`max(SPECK_MIN_INK, SPECK_INK_FRAC * metrics.height_med * metrics.width_med)`
with `SPECK_INK_FRAC` ≈ 0.02 and `SPECK_MIN_INK` a small floor — so a few-pixel
speck is dropped while any real glyph line (orders of magnitude more ink)
survives. Skip the filter under sentinel metrics.

### Step 4.2 — Never emit a near-blank canvas

As a final guard, drop any composed canvas whose total dark-pixel count is below
the same floor before returning from `compose_pages`.

**LLM verification.** Tests in `reflow.rs`:
- `compose_drops_speck_trailing_canvas` — real body bands followed by one
  3-dark-pixel speck band produce canvases that are **all** non-blank (the speck
  neither forms nor rides a trailing canvas).
- `compose_keeps_short_real_last_line` — a legitimate short final line (one word,
  hundreds of dark pixels) is **kept**.
- Existing `compose_*` tests stay green.
- `cargo test -p plutarch-reader facsimile::reflow`.

**Human verification.** None beyond Phase 5.

---

## Phase 5 — Regenerate, verify the boundaries, and the full suite

**Suggested implementer: Sonnet** (mechanical regen + real-eyes check).

### Step 5.1 — Repopulate the cache and rebuild

```sh
cargo run --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf \
    --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --redetect
```

`--redetect` repopulates `title_top_frac`/`body_top_frac` for every unit while
preserving any `pinned` row. (If an individual boundary still looks wrong after
this, the fix is to hand-edit that unit's fractions — or `skip_top_bands` — and
set `"pinned": true`, not to re-tune a heuristic.)

### Step 5.2 — Checks

**LLM verification.**
- `cargo test` (full suite) green.
- `cargo clippy --all-targets -- -D warnings` clean.
- `cargo fmt --all --check` clean.
- `grep` the regenerated `facsimile_units.json` for `title_top_frac` on
  `romulus`, `comparatio_thesei_et_romuli`, and a v2 shared boundary.

**Human verification (the project's primary feedback loop).** Render and eyeball
`build_facsimile/tomus_1_inner.pdf`:
- **Theseus → Romulus**: Theseus's last page ends with "…appellamus."; Romulus's
  first page shows the generated `\heading{ROMULUS}` directly above "I. Ingens
  Romæ nomen…", with no Theseus tail and no embedded `ROMULUS.`.
- **Romulus → Comparatio**: no embedded `COMPARATIO THESEI CUM ROMULO.` on
  Romulus's last page; the comparatio opens at "Habes quæ de Romulo et Theseo…".
- **No blank/speck folio** inside the Comparatio (the former p. 115).
- Spot-check two or three other shared boundaries (e.g. Lycurgus → Numa, and a
  v2 pair) for the same clean split.

---

## Notes for the implementer

- **Do not touch the source text.** This plan only changes *which* image bands
  are kept and *where* a page is cut; the kept pixels are the verbatim 1857 type.
  No band's content is altered.
- **Fraction→row uses the cropped column's own height**, which equals the full
  page height (`crop_column` keeps `0..h`). Do not re-scale by the body DPI;
  the fraction is dimensionless.
- **Tolerances** are expressed in the existing median-line-height units already
  available as `LineMetrics::height_med`, so they travel with the global scale.
- **The fallback path must stay** so an un-redetected cache (or a page with no
  text layer) still builds, just with the old heuristic for that one boundary.
