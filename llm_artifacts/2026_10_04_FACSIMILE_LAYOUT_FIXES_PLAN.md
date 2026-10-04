# Facsimile layout fixes: blanks, shared-page boundaries, line metrics

## Context

The facsimile build path (`src/facsimile/`) reproduces the 1857 Didot *Plutarchi
Vitae* body pixel-for-pixel from the source scans and re-typesets only the front
matter. The preceding plan
(`llm_artifacts/2026_10_03_FACSIMILE_FRONTMATTER_AND_LAYOUT_PLAN.md`) reworked the
front matter, added an inline chapter header, capped composed-canvas height,
strengthened furniture removal, balanced the volume split, and added build-dir
cleanup. All of that is implemented and its tests are green, and a full
multi-volume build now compiles end to end (Tomi I–VIII at 416 / 448 / 448 / 128 /
416 / 384 / 448 / 320 pages).

A real-eyes review of that compiled build (`build_facsimile/tomus_*_inner.pdf`)
surfaced layout defects that the page-level reflow and boundary model do not handle
against the **actual** source. This plan fixes them and folds in the items left
open by the 2026-10-03 plan. The 2026-10-03 plan stays on disk as the historical
record of the work it covered; this document supersedes its open items.

The faithfulness invariant in `CLAUDE.md` applies throughout: the reproduced body
is the source pixels, and the re-typeset prose (Monitum / Praefatio) is the
`FrontProse` cache, never silently "improved".

## Current state of the world

### What the 2026-10-03 plan delivered (all green, in `build_facsimile`)

- **Front matter** (`src/latex.rs`, `src/facsimile/frontmatter.rs`): per-tomus
  `tomus_title_page`, `embed_scan`, page-list-driven `front_prose_pages`;
  Rust-cropped QR / Volumen-title / colophon scans; `fill_front_matter`
  (`src/facsimile/mod.rs`) renders scans offline then fills Monitum/Praefatio prose.
- **Inline chapter header** (`src/facsimile/assemble.rs::unit_block`): no standalone
  title page; the unit's first body page carries a large centered **Latin** header
  and no running head; later pages keep the centered running head.
- **Canvas-height cap** (`src/facsimile/reflow.rs`): `compose_pages` takes
  `first_budget` / `budget`; `split_tall_strip` (reflow.rs:399) splits a strip
  **taller than the whole budget** at low-ink rows; `body_budget` (reflow.rs:116)
  and `first_body_budget` (reflow.rs:123) leave a bottom margin and header room
  (`BODY_BOTTOM_SLACK_CM` = 0.3, `HEADER_ALLOWANCE_CM` = 1.2).
- **Furniture** (`src/facsimile/lines.rs`, `reflow.rs`): `TOP_FURNITURE_FRAC` = 0.08
  (lines.rs:23); `drop_interior_running_head` (reflow.rs:233) drops a short, detached
  leading/trailing band on interior pages.
- **Balanced split** (`src/volumes.rs`): nearest-boundary `split_source`;
  `front_overhead` per-tomus overhead model.
- **Build-dir cleanup** (`src/facsimile/mod.rs`): `clean_generated` at start,
  `remove_composed_pngs` after a successful compile; `render_cache/` preserved.

### Uncommitted working-tree edits (tested, not yet compile-verified)

`src/latex.rs::tomus_title_page` has already been changed to (a) wrap the Lives list
(each name an `\mbox`, bullets with ordinary interword spaces, inside
`\parbox{0.86\textwidth}{\centering …}`) so a long tomus no longer clips its Lives,
and (b) push the `EX LIBRIS / MARTINEZ-DIRKS` plate to the bottom of the page
(`\vfill\vfill` between the title block and the plate, `\vspace{1cm}` under it). The
`latex` unit tests were updated and pass, but the on-disk `build_facsimile` was
produced **before** these edits, so they are not yet seen in a compiled PDF. Phase 7
verifies them.

### Defects found in the compiled build (with evidence)

1. **Large blanks interrupting sentences** (e.g. inner pp. 35, 44). The body is
   reflowed by stacking line-strips until the next strip will not fit, then wrapping.
   A strip that is **tall but still ≤ the whole budget** (a merged "giant band" of
   several lines, or a large whitespace interval around a verse / section break)
   arriving on a half-full page cannot fit, so the page closes early (big blank) and
   the tall strip opens the next page. `split_tall_strip` only splits strips taller
   than the **entire** budget, so these mid-size tall strips wrap whole.
   *Measured:* regenerating the `romulus` unit offline, canvases 12 / 26 / 37 are
   610 / 426 / 568 px against a ~1394 px budget, each immediately followed by a
   near-cap canvas.

2. **A Life/Comparatio starts mid-page with the previous unit's tail above it**
   (e.g. inner pp. 60 Romulus, 112 Comparatio). Confirmed from the raw scan: **source
   v1 page 33** carries Theseus's ending (top quarter), an ornament, the source's own
   `ROMULUS.` display title, then Romulus ch. I — all on one page. The unit table
   uses **page-level** boundaries (`romulus.start_page = 33`, every unit's
   `skip_top_bands = None`), so `ranges` (units.rs:106) gives Theseus `end_page = 32`
   and Romulus `start_page = 33`. Result: Romulus's first composed image swallows
   Theseus's tail **and** the printed `ROMULUS.` title (the "second header pasted from
   the PDF"), while Theseus is **truncated** (it loses the lines physically on p33).
   `drop_opening_bands` (reflow.rs:275) with `skip_top_bands = None` keeps everything
   from the first full-measure line, and the previous unit's tail lines *are*
   full-measure, so nothing is dropped. This is pre-existing (it is in the composed
   image content, independent of the inline-header change) and is pervasive because
   Lives run on throughout the edition.

3. **Bled-through source furniture** (e.g. a stray signature numeral "2" on an
   interior page). `drop_interior_running_head` did not catch a short detached
   numeral sitting below the body.

4. **Monitum second page missing.** The 2026-10-03 plan nulled v1 p12–13 and v2
   p14–17 so they would re-transcribe. The build's batch re-transcription succeeded
   for five pages but **`s_0_13` (Monitum p2, v1 p13 — the dense errata list with
   heavy inline Greek) returned an unparseable `PageDoc`** ("parsing transcription
   reply as a PageDoc"), so its pass was left unset and the Monitum currently prints
   only its first page. The record is still `role: front_prose`, `result: null`, so
   `work_remains` will retry it, but a plain retry may fail the same way.

### Items left open by the 2026-10-03 plan (folded in here)

- **Old Step 4.3** — reduce giant bands at the *source* by excluding the far-left
  marginal line-number tick column in `detect_latin_column` / `crop_column`
  (columns.rs:60/113). Deferred then as belt-and-suspenders; it is the root cause of
  defect #1's merged bands, so it belongs here.
- **Old Phase 7** — crop-box / header-allowance tuning after a real-eyes check. The
  QR (`0.04` all sides), Volumen-title (`0.08/0.03/0.08/0.07`), and colophon
  (`0.10/0.33/0.10/0.47`) crops and `HEADER_ALLOWANCE_CM`/`TOP_FURNITURE_FRAC`
  rendered acceptably in this build; kept as a final tuning step.
- **Old Step 1.3** — re-run front-matter transcription. Done except `s_0_13`
  (defect #4 above).

### Key code references

- `src/facsimile/reflow.rs`: `compose_pages` (341), `split_tall_strip` (399),
  `drop_opening_bands` (275), `band_ink_span` (292), `drop_interior_running_head`
  (233), `median_gap` (258), `unit_body_strips` (171), `render_and_band` (208),
  `TrimGeom::body_budget` (116) / `first_body_budget` (123), `BODY_LINE_SPAN_FRAC`
  (31), `LINE_MIN_INK` (22).
- `src/facsimile/lines.rs`: `line_bands` (50), `drop_furniture` (75),
  `ink_profile_rows` (33, `pub(crate)`), `cut_points` (131), `TOP_FURNITURE_FRAC`
  (23), `GAP_OUTLIER_FACTOR` (29).
- `src/facsimile/columns.rs`: `detect_latin_column` (60), `crop_column` (113),
  `ink_profile_cols` (27), `INK_THRESHOLD` (14).
- `src/facsimile/units.rs`: `Unit` (with `skip_top_bands`, `last_page`, `pinned`),
  `UnitRange`, `ranges` (106).
- `src/facsimile/detect.rs`: `units_from_runs` (218) builds each `Unit` with
  `skip_top_bands: None`; `load_or_detect` (325).
- `src/llm/mod.rs`: `trait Vision` (354) with the single method
  `classify(&self, png_base64) -> Result<(PageRole, Usage)>`; `ApiVision` (360).

## Desired end state

- **No large mid-unit blank.** A tall strip is split at a low-ink row to **fill the
  current page** before the page wraps; a single tight line still wraps whole; every
  composed canvas is still ≤ its budget and keeps a visible bottom margin. A unit's
  text flows continuously with no page left half-empty mid-sentence.
- **Clean unit boundaries on shared pages.** When a unit begins partway down a source
  page, its first image begins at its **own** first line (the previous unit's tail
  and the source's printed display title are dropped), and the **previous** unit
  keeps the lines that physically sit at the top of that shared page. No duplicated
  source title appears, and no unit is truncated.
- **Line metrics.** A per-run line-metrics summary (robust typical line **height**
  and **ink-span width**, with dispersion) classifies each band as normal / tall
  (merged) / short / narrow, and this classification drives the fill-split decision,
  the title-band detection, and the furniture drop. Outlier bands can optionally be
  flagged for a Claude Vision review (behind a flag) whose decisions are recorded in
  the unit table; the normal build stays offline and deterministic.
- **No bled-through furniture** (running heads, folios, signatures) in reproduced
  body text.
- **Monitum reproduces both pages.** The p13 re-transcription is robust (lenient
  reply parsing and/or a larger token ceiling), so the dense errata page parses.
- **Fewer giant bands** because the marginal tick column is excluded from the crop.
- **Title page** Lives list wraps and the `EX LIBRIS` plate sits at the bottom
  (verify the already-applied edits).
- `cargo test`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo fmt --all --check` all pass; a regenerated build is eyeballed on the
  previously-broken pages.

## Tech Learnings

- **Why tall boxes leave blanks.** `\includegraphics` is one unbreakable box aligned
  to the top of the text block. The reflow packs scaled strips onto a fixed-width
  canvas until the next strip overflows the height budget, then starts a new canvas.
  A strip that is tall but ≤ budget fits *a* canvas, so it is never split — but if the
  current canvas is already partly full it cannot fit there, forcing an early wrap and
  a blank. Splitting such a strip *to fill the remaining space* (at an internal
  low-ink row, so no glyph row is bisected) removes the blank while keeping each
  canvas ≤ budget.
- **Robust line statistics.** Body line heights and widths are near-constant (one
  font, one leading, one measure), so the **median** line height/width with the
  **median absolute deviation (MAD)** gives a tight central estimate that a few merged
  or partial bands cannot skew (unlike the mean/standard deviation). An outlier is
  then "height > median + k·MAD" (merged) or "ink-span < f·median width"
  (title / short line). This is the standard robust-outlier approach.
- **Shared-page boundaries as a line split.** Two units sharing a source page are
  divided by a *line* (the second unit's printed title), not a page edge. Modelling
  the boundary as "the previous unit keeps the bands above the title; the next unit
  keeps the bands below it" splits one page's bands between two units, which the
  page-level `end_page = next.start − 1` rule cannot express.
- **Keeping the build offline.** The body path is deterministic and network-free by
  design. A Claude Vision review therefore belongs at **authoring/detection** time
  (behind a flag), writing its conclusions into the committed `facsimile_units.json`
  so every subsequent build reuses them with zero network calls — mirroring how the
  front-matter sidecar caches transcription.

---

## Phase 1 — Line-metrics foundation
### Suggested implementer: Opus

A small, reusable metrics layer the later phases key off. It computes robust typical
line dimensions for a unit's pages and classifies each band, so the fill-split
decision (Phase 2), title detection (Phase 3), and furniture drop (Phase 5) share one
definition of "tall" / "short" / "narrow" instead of each inventing a threshold.

### Step 1.0 — Restructure `unit_body_strips` into two passes that return the metrics

The metrics summarize **all** of a unit's bands, but `drop_opening_bands`
(reflow.rs:275) currently runs **inside** `unit_body_strips`'s per-page loop at
`idx == 0`, before the later pages exist — and Phase 3 moves the *last*-page bottom-trim
into that same loop. Neither trim can run until the metrics exist, and the metrics
cannot exist until every page is banded. So `unit_body_strips` (reflow.rs:171) must be
restructured into two passes, and this restructure is a prerequisite for Phases 1–3:

1. **Pass A — band every page.** For each source page call `render_and_band` (which
   already does `drop_furniture` + `drop_interior_running_head`) and collect
   `Vec<(GrayImage, Vec<Band>)>`. Do **not** apply `drop_opening_bands` here.
2. **Compute `LineMetrics`** over the Pass-A bands (so the medians are taken from
   body lines with page furniture already removed — see Step 1.1).
3. **Pass B — apply the boundary trims** using the metrics: the first page's top-trim
   (`drop_opening_bands`, extended in Phase 3.2) and, when the unit shares its last page
   with the next unit, that page's bottom-trim (Phase 3.3). When the unit's first page
   *is* also its shared last page, apply both to that one page (Phase 3).

Change the return type so the metrics flow to the caller:
`unit_body_strips(...) -> Result<(Vec<(GrayImage, Vec<Band>)>, LineMetrics)>`. In
`run_facsimile` (mod.rs, the compose loop ~line 126) destructure the pair and pass the
`LineMetrics` into `compose_pages` (Phase 2). A unit that fails every page returns the
sentinel metrics (`height_med == 0`); callers then fall back to today's behavior.

### Step 1.1 — Add a `LineMetrics` type and band classification

In `src/facsimile/reflow.rs` (or a new `src/facsimile/metrics.rs` module declared in
`src/facsimile/mod.rs` if reflow grows unwieldy — prefer a new module, since lines.rs
and reflow.rs both need it):

- `pub struct LineMetrics { pub height_med: u32, pub height_mad: u32, pub width_med: u32 }`.
- `pub fn line_metrics(pages: &[(GrayImage, Vec<Band>)]) -> LineMetrics` — over every
  band of every page, collect band heights (`bottom - top`) and ink-span widths
  (reuse `band_ink_span`, which must be made `pub(crate)` or moved here), and take the
  median of each and the MAD of the heights. Guard against empty input (return a
  sentinel with `height_med = 0`, callers fall back to their current behavior).
- `pub enum BandClass { Normal, Tall, Short, Narrow }` and
  `pub fn classify_band(col: &GrayImage, band: &Band, m: &LineMetrics) -> BandClass`:
  - `Tall` when `height > height_med + TALL_MAD_K * height_mad` **and**
    `height > height_med * TALL_MIN_FACTOR` (both, so a tight distribution with tiny
    MAD does not over-flag);
  - `Short` when `height < height_med * SHORT_FACTOR`;
  - `Narrow` when `band_ink_span < width_med * NARROW_FACTOR`;
  - else `Normal`. (`Narrow` and the height classes can co-occur; expose the width
    test separately as `is_narrow` so callers combine as needed.)
- Named constants with doc comments marking them tunable in Phase 7.3:
  `TALL_MAD_K ≈ 3.0`, `TALL_MIN_FACTOR ≈ 1.6`, `SHORT_FACTOR ≈ 0.5`,
  `NARROW_FACTOR ≈ 0.6`.

### Step 1.2 — Tests
- `line_metrics_medians_ignore_outliers` — a set of ~20 uniform bands plus one giant
  and one tiny band yields `height_med` ≈ the uniform height (outliers do not move the
  median).
- `classify_band_flags_tall_short_narrow` — a synthetic column with a uniform body, a
  merged double-height band, a half-height band, and a narrow centered band classifies
  each as `Tall` / `Short` / `Narrow` respectively and the body as `Normal`.
- `unit_body_strips_returns_metrics` — the restructured `unit_body_strips` returns a
  non-sentinel `LineMetrics` for a multi-page unit, and the first page's opening bands
  are still dropped (the two-pass order preserves the existing first-page trim). Keep
  `first_page_excludes_title_and_crossref_bands` / `skip_top_bands_override_drops_exact_count`
  green against the new (Pass-B) call path.

### LLM verification
- `cargo test metrics` (or `cargo test line_metrics`) passes; `cargo build` clean.

### Human verification
- None (pure computation; exercised visually via later phases).

---

## Phase 2 — Greedy-fill compose (fix the blanks)
### Suggested implementer: Opus

### Step 2.1 — Split a too-tall strip to fill the current canvas before wrapping

**Pixel basis (pin this before coding).** Two bases are in play:

- **Original px** — the band heights/widths on the source column image, and the
  `LineMetrics` (`height_med`, `width_med`) computed from them in Phase 1.
- **Trim/scaled px** — the composed-canvas space: `budget` / `first_budget`
  (`body_budget` is `content_height` in trim px), `used_h`, `remaining`, and every
  scaled strip height (`original_height × scale`). This is the `budget` basis.

The **Tall** verdict is a *ratio* (band height vs `height_med`), so classify the
**original** band against the **original** metrics *before* scaling — the verdict is
identical in either basis, and this avoids rescaling the metrics. Everything about
**space and cutting** is in trim/scaled px: compute `MIN_PIECE_PX` from the scaled
typical line, `MIN_PIECE_PX = round(MIN_PIECE_FACTOR × height_med × scale)` with
`MIN_PIECE_FACTOR ≈ 0.5` (≈ half a trim line; falls back to a small absolute floor when
`height_med == 0`), and search `lowest_ink_row` on the **scaled** strip image over the
trim-px window `[MIN_PIECE_PX, remaining]`.

Rewrite the packing loop in `compose_pages` (reflow.rs:341) so that, for each scaled
strip (in reading order), with the strip's original band available for the Tall
check:

1. Let `remaining = b - used_h` (where `b` is `first_budget` while `canvases` is empty,
   else `budget`).
2. If the strip fits (`height <= remaining`): place it whole; `used_h += height`.
3. Else, the strip does not fit the remaining space:
   - If the strip is **splittable here** — i.e. `remaining >= MIN_PIECE_PX` **and** a
     low-ink cut row exists within `[MIN_PIECE_PX, remaining]` of the strip (reuse the
     `ink_profile_rows` search from `split_tall_strip`, factored into a shared
     `lowest_ink_row(strip, lo, hi) -> Option<u32>` helper) — cut a piece that fills
     `remaining` at that row, place it, close the canvas (`used_h = 0`), and continue
     with the **remainder** of the same strip.
   - Else (the strip is a single tight line, or `remaining` is too small to bother):
     if the current canvas is non-empty, close it and retry the whole strip on a fresh
     canvas; if it is empty and the strip still exceeds `b`, fall back to
     `split_tall_strip(strip, b)` (the Phase 4.1 whole-budget split) so the invariant
     holds.

This subsumes `split_tall_strip`'s role (a strip taller than a whole budget is split
across several fills) while also filling partial canvases, so **no page closes early
with a large blank**. `MIN_PIECE_PX` is derived per-call from `height_med × scale` (as
above), so a fill piece is never a sliver and the floor tracks the real line height.
The invariant is unchanged: **every returned canvas has height ≤ its budget.**

`compose_pages` gains a `metrics: &LineMetrics` parameter (passed from
`unit_body_strips` via `run_facsimile`, Step 1.0); `scale` is already a parameter. Use
the metrics only to avoid splitting a `Normal`/`Short` strip that merely does not fit
(those wrap whole); a strip classified `Tall` is the one that gets fill-split. A
`Normal` strip never triggers an early wrap unless the canvas is nearly full (then the
blank is at most one line tall), so gating the split on `Tall` keeps single lines
intact. When `metrics` is the sentinel (`height_med == 0`), skip the `Tall` gate and
fall back to the whole-budget `split_tall_strip` behavior so a unit with no usable
bands still composes.

### Step 2.2 — Tests (reflow)
- `greedy_fill_splits_tall_strip_to_fill_page` — a half-full canvas followed by a tall
  (merged) strip yields a **full** first canvas (≥ budget − one line) rather than a
  half-empty one; the remainder flows onto the next canvas.
- `greedy_fill_keeps_single_line_whole` — a normal-height strip that does not fit the
  remaining space wraps whole (is not cut); no canvas exceeds budget.
- `compose_never_exceeds_budget` (keep) and `oversized_strip_split_at_low_ink` (keep /
  adapt to the shared `lowest_ink_row`).
- `no_large_blank_before_tall_strip` — assert the last canvas before a tall strip is
  filled to within one median line height of budget.
- Update every existing `compose_*` call site (the `compose_canvases_share_fixed_width`
  / `compose_fills_page_then_wraps` / `compose_breaks_only_at_cut_points` /
  `compose_insets_content_with_safety_margins` / `full_page_keeps_bottom_slack` tests
  and `run_facsimile`) for the new `metrics` parameter; pass the sentinel
  `LineMetrics` where a test wants today's unconditional wrap behavior.

### LLM verification
- `cargo test reflow` passes; `cargo build` clean.
- Regenerate the `romulus` unit offline (`--unit romulus --no-compile`) and confirm no
  canvas is < ~0.7 × budget except the unit's final (ragged) canvas.

### Human verification
- Compile a tomus (or the `romulus` unit) and confirm former-blank inner pages (around
  old pp. 35, 44) are now full, with the sentence contiguous.

---

## Phase 3 — Shared-page unit boundaries (Issue #2, Option A)
### Suggested implementer: Opus

A unit that begins partway down a source page must start at its own first line, and
the previous unit must keep the lines above the new unit's title on that shared page.
Both halves key off detecting the **source display-title band** on the shared page.

### Step 3.1 — Detect the source display-title band

Add to reflow (or metrics) `fn find_title_band(col: &GrayImage, bands: &[Band], m: &LineMetrics) -> Option<usize>`
returning the index of the unit's printed display title among the leading bands. A
display title (`ROMULUS.`, `COMPARATIO THESEI ET ROMULI`) is distinctive: it is
**narrow** (centered, ink-span well under the measure) and **isolated** (above-median
gaps on one or both sides), and often **taller/heavier** than a body line. Score the
first few bands (say the leading `TITLE_SCAN_BANDS ≈ 6`) by (narrow ink-span) + (large
adjoining gap) and return the best-scoring band that clears a threshold; return `None`
when no band looks like a title (a clean top-of-page start). Keep the heuristic
conservative and documented as tunable.

### Step 3.2 — Top-trim the next unit at its title

In `drop_opening_bands` (reflow.rs:275), when `skip_top_bands` is `None`, replace the
"first full-measure line" rule with: if `find_title_band` returns `Some(i)`, drop all
bands **up to and including** `i` (the previous unit's tail + the printed title); else
fall back to the current full-measure-line heuristic. The explicit `skip_top_bands`
override still wins when set (so a human can pin a hard count). `drop_opening_bands`
gains a `metrics: &LineMetrics` parameter; it runs in Pass B of the restructured
`unit_body_strips` (Step 1.0), after the metrics are computed.

### Step 3.3 — Bottom-trim the previous unit on a shared last page

Make the previous unit keep the shared page's top. Changes:

- `ranges` (units.rs:106): when the next unit is in the same source, set the current
  unit's `end_page = next.start_page` (the shared page, **inclusive**) instead of
  `next.start_page − 1`, and add a `UnitRange.shared_tail: bool` set `true` for that
  case (the shared page is then simply the unit's `end_page`, so no extra field is
  needed to locate it; the next unit's title is found by image analysis). Update the
  two other `UnitRange` construction sites for the new field — the `--pages` synthetic
  range in `select_ranges` (mod.rs) and the `unit(...)` test helper / literals in the
  `units` and `mod` test modules — defaulting `shared_tail: false`.
- `unit_body_strips` (Pass B, Step 1.0): on the unit's **last** page, when
  `shared_tail` is set, run `find_title_band` and keep only the bands **above** the
  detected title (drop from the title down — the next unit's content). If
  `find_title_band` is `None` on that page (the next unit actually started at the page
  top, no shared tail), keep nothing from that page (equivalent to the old
  `end_page = next.start − 1`).

**When the unit's first page *is* its shared last page** (a single-page unit whose own
title sits mid-page and whose successor also opens on that page): apply **both** trims
to that one page — top-trim at the unit's own title (`drop_opening_bands`) and
bottom-trim at the next unit's title — keeping only the bands **between** the two
titles. Implement this by making Pass B, for each page, drop leading bands up to the
unit's own opening (first page only) and trailing bands from the next unit's title
(shared last page only), composing the two on a page that is both. `find_title_band`
scans only the leading region, so the "next unit's title" on a shared last page is
found by scanning forward from the end (or by a `find_title_band_from(bands, start)`
that scans a trailing window); specify a trailing-scan helper rather than reusing the
leading-only scan.

The two trims are symmetric and share the title-detection logic, so the shared page's
bands are partitioned cleanly: previous unit = above the title, next unit =
title-exclusive and below. No band appears in two units' output, and no unit is
truncated.

> **Shared pages are rendered twice** — once as the previous unit's last page and once
> as the next unit's first page. This is harmless: `render_gray` caches the
> rasterization (mod.rs:265), so only the banding repeats, and the two trims keep
> disjoint band ranges.

> **Note on `last_page` units.** The final unit of each source uses `last_page` /
> source end, not a following unit, so `shared_tail` is `false`; leave that path
> unchanged.

### Step 3.4 — Tests
- `find_title_band_picks_centered_isolated_band` — a synthetic shared page (several
  full-measure tail lines, a wide gap, a narrow centered title band, then body lines)
  returns the title band's index.
- `find_title_band_none_on_clean_page` — a page whose first band is already a
  full-measure body line returns `None`.
- `top_trim_drops_previous_tail_and_title` — `drop_opening_bands` on the shared page
  keeps only the bands below the title.
- `bottom_trim_keeps_previous_tail` — the previous unit's last (shared) page keeps only
  the bands above the title.
- `single_page_unit_trims_both_ends` — a one-page unit whose page holds [prev tail][own
  title][own body][next title][next body] keeps only [own body] (both trims on one
  page).
- `ranges_shares_boundary_page_in_source` (units.rs) — for consecutive same-source
  units, the earlier unit's `end_page` equals the later unit's `start_page` and
  `shared_tail` is set; across the v1→v2 boundary it is not.
- Keep `ranges_derives_end_pages` / `ranges_last_unit_runs_to_source_end` green,
  updating expectations to the inclusive shared `end_page`.

### LLM verification
- `cargo test facsimile` / `cargo test units` pass; `cargo build` clean.

### Human verification
- Compile Tomus I; confirm Romulus (old p60) and the Comparatio (old p112) each begin
  on their own fresh page at their own first line, with **no** duplicated source title
  and **no** Theseus/Romulus tail above them, and that Theseus/Romulus retain their
  final lines (nothing truncated).

---

## Phase 4 — Band hygiene: giant bands at the source + interior furniture
### Suggested implementer: Opus

### Step 4.1 — Exclude the marginal tick column from the crop (old Step 4.3)

Giant merged bands partly come from the far-left marginal line-number tick column being
pulled into the Latin crop, bridging inter-line gaps with ink. In
`detect_latin_column` / `crop_column` (columns.rs:60/113), trim a thin left margin
(e.g. drop columns whose ink is sparse, isolated ticks left of the dense text block —
reuse the `ink_profile_cols` profile already computed) so the crop starts at the text
body, not the tick column. Keep it conservative (never eat the first glyph column);
gate the trim on the tick column being clearly detached from the body by a whitespace
gap, mirroring how the gutter is found. This reduces (not eliminates) giant bands;
Phase 2 still guarantees correctness when one survives.

> **Scale sensitivity — verify before committing.** `detect_latin_column`'s returned
> width feeds the 95th-percentile column width → `global_scale` → the one global type
> size for the whole work (`global_scale_for`, mod.rs). The trim must change only the
> *left edge past the detached ticks*, not the dense text block's own left/right
> detection, so the column **width** (and thus the global scale) is effectively
> unchanged. After implementing, confirm `global_scale_for` over a sample of pages
> returns essentially the same scale as before the trim (add a logging check or a
> targeted test), so body type size does not shift across the whole book.

### Step 4.2 — Drop a bled trailing signature / folio using line metrics

Extend `drop_interior_running_head` (reflow.rs:233) so the **trailing** drop also
removes a band that is `Short`/`Narrow` (per `classify_band`) and detached below the
body — a bare signature numeral like the stray "2" — not only an above-median-gap
band. Keep the leading-head rule as is. Stay conservative: require both the
metric-class outlier **and** detachment, so a legitimately short last body line
survives.

### Step 4.3 — Tests
- `crop_excludes_left_tick_column` (columns) — a synthetic page with a sparse tick
  column left of the dense block crops to the dense block, excluding the ticks; the
  existing `detect_column_excludes_sparse_left_marks` stays green.
- `drop_trailing_signature_numeral` (reflow) — a short narrow detached numeral band at
  the foot is dropped; a full-measure last line at the same position is kept.
- `left_tick_trim_preserves_column_width` (columns) — on a synthetic page with a
  detached tick column, the detected column **width** (right − left) is within a small
  tolerance of the width detected without the ticks, so `global_scale` is stable.

### LLM verification
- `cargo test columns` / `cargo test reflow` pass; `cargo build` clean.

### Human verification
- Render a page that previously showed a stray signature (e.g. old inner p59): the
  numeral is gone and the body starts/ends clean.

---

## Phase 5 — Monitum p2 re-transcription robustness
### Suggested implementer: Opus

### Step 5.1 — Make the front-prose reply parse robustly

`s_0_13` (v1 p13, the dense Greek errata list) returned an unparseable `PageDoc`. In
`src/llm/transcribe.rs::parse_pagedoc`, before `serde_json::from_str`, strip a
leading/trailing Markdown code fence and any prose outside the outermost `{ … }`
(extract the first balanced JSON object), so a model reply wrapped in ```json … ``` or
prefixed with commentary still parses. Do **not** loosen the schema or alter accepted
text.

**Apply this globally** (to `parse_pagedoc` itself, used by both the body path —
`transcribe_all` and `verify::reconcile_all` — and the front-prose path), not a
front-only wrapper: a fenced/pre-ambled reply is a reply-envelope issue, not a
content-mode issue, and making the body path equally robust is strictly beneficial and
avoids a parallel code path. The `parse_pagedoc_rejects_malformed` test still pins that
genuinely non-JSON input errors.

`MAX_TOKENS` (transcribe.rs:34) is a single constant shared by every request. If the
dense page plausibly truncates, bump the one constant modestly (a uniform, simpler
change than threading a per-mode ceiling); note the small cost increase applies to all
pages. Prefer the lenient-parse fix first and only raise `MAX_TOKENS` if a re-run shows
the reply was actually truncated (not just fenced).

### Step 5.2 — Re-run and confirm

The v1 p13 record is still `role: front_prose`, `result: null`, so a build (or a
focused front-matter run) retries exactly it. Confirm p13 now parses and the Monitum
reads continuously across p12→p13 with no furniture (`PLUTARCHUS. I`) — the Phase 1.2
prompt from the prior plan already instructs furniture exclusion.

### Step 5.3 — Tests
- `parse_pagedoc_strips_code_fence` — a reply wrapped in a ```json fence parses to the
  expected blocks.
- `parse_pagedoc_extracts_object_amid_prose` — a reply with leading commentary before
  the JSON object parses.
- Keep `parse_pagedoc_rejects_malformed` green (genuinely non-JSON still errors).

### LLM verification
- `cargo test transcribe` passes.
- `grep -n "PLUTARCHUS" plutarch.frontmatter.tags.json` returns nothing in a Monitum
  `result`; the v1 p13 record has a non-null `result`.

### Human verification
- Read the regenerated Monitum: both pages present, continuous, furniture-free.

---

## Phase 6 — Optional Claude Vision line review (authoring-time, flagged)
### Suggested implementer: Opus

A deterministic offline build is the default. This phase adds an **opt-in** review that
sends outlier bands/pages to Claude Vision to resolve the cases the heuristics are
least sure about (is a leading band a title or a real line? is a tall band merged
lines that need a specific split? is a short band furniture?), and records the answers
in `facsimile_units.json` so later builds reuse them with zero network calls.

### Step 6.1 — Per-unit review annotations in the unit table

Add optional, defaulted fields to `Unit` (units.rs), skipped when empty so existing
tables parse unchanged and diffs stay minimal:
- `skip_top_bands` already exists (title top-trim count); the review may set it.
- `review_notes: Option<String>` (free-text provenance, e.g. "vision: title at band 4").

Keep these `#[serde(default, skip_serializing_if = …)]`, honoring the hand-edit /
`pinned` contract (a reviewed value the detector must not clobber).

### Step 6.2 — A flagged review pass

Add `--review-lines` to `FacsimileArgs` / the `facsimile` bin (default off). When set,
after composing (or in a dedicated pass over `unit_body_strips` output), for each unit
collect bands that `classify_band` marks `Tall` or that `find_title_band` is unsure
about, render just those band crops (reuse `render::render_page` at `TRANSCRIBE_DPI`
then crop in-memory), and submit them to a new `Vision` method
`review_line(&self, png_base64) -> Result<(LineVerdict, Usage)>`. Thread a
`UsageTracker` and report cost, mirroring `fill_front_matter`. Write the conclusions
back into the unit table (`skip_top_bands`, `review_notes`) and re-save, so a subsequent
normal build is deterministic. A missing API key warns and leaves the table unchanged
(as `fill_front_matter` does).

**Scope the verdict to what a consumer uses.** The deterministic path already splits
merged bands at low-ink rows (Phase 2) and detects titles (Phase 3); the review exists
only to *correct the boundary decisions* those heuristics get wrong. So define
`enum LineVerdict { Title, Furniture, Body }` — each with a consumer:
- `Title` on a leading band → record `skip_top_bands = index + 1` for the unit (so the
  top-trim is pinned regardless of the heuristic);
- `Furniture` on a leading/trailing band → fold into the pinned `skip_top_bands` (lead)
  or note it for the trailing drop;
- `Body` → leave the heuristic's decision as is.

Do **not** add a `MergedLines(n)` verdict: no code consumes a stored merge count (the
greedy fill splits deterministically), so it would be dead data. Record provenance in
`review_notes`.

**Extend every `Vision` impl.** Adding `review_line` to `trait Vision` (mod.rs:354)
requires implementing it on `ApiVision` (mod.rs:360) **and** on every test stub that
implements `Vision` — notably `CountingVision` in `frontmatter.rs`'s tests (and any
other). Give the stubs a trivial canned verdict so existing tests keep compiling.

Keep the normal (no-flag) build path completely unchanged and network-free.

### Step 6.3 — Tests
- `unit_table_roundtrips_review_fields` (units) — a `Unit` with `review_notes` /
  `skip_top_bands` round-trips through JSON; a table without them still parses.
- `review_targets_only_outlier_bands` — with a stub `LineMetrics`/classifier, the
  review selects only `Tall`/uncertain bands, not `Normal` body lines.
- `review_offline_without_flag` — a run without `--review-lines` constructs no Vision
  client and makes no calls (assert via a counting stub, mirroring
  `frontmatter_offline_when_sidecar_fresh`).

### LLM verification
- `cargo test facsimile` / `cargo test units` pass; `cargo build` clean.

### Human verification
- Run `--review-lines` on one pair-group with credentials; confirm the unit table gains
  sensible `skip_top_bands` / `review_notes` and a following normal build makes no API
  calls and renders those boundaries correctly.

---

## Phase 7 — Title-page verification + regenerate, verify, tune
### Suggested implementer: Opus

### Step 7.1 — Verify the already-applied title-page edits

Compile a tomus and confirm the Lives list **wraps** within the measure (no clipping,
no mid-name hyphenation) for a long-list tomus, and that `EX LIBRIS / MARTINEZ-DIRKS`
sits at the bottom, clearly separated from the title block. Adjust `0.86\textwidth`,
the `\vfill\vfill` ratio, or `\vspace{1cm}` if the real-eyes check wants it.

### Step 7.2 — Full regeneration and eyeball

Run the full facsimile build with the real sources, let it compile, and record each
tomus's page count. Re-examine the specific pages this plan targets: former blanks
(old pp. 35, 44), the Theseus→Romulus boundary (old p60), the Romulus→Comparatio
boundary (old p112), and a former signature-bleed page (old p59). Confirm the volume
boundaries remain within target/cap (the Phase 6 split from the prior plan is
unchanged here).

### Step 7.3 — Tune the metric / crop / layout constants

From the compiled PDFs, adjust the Phase 1 metric factors (`TALL_MAD_K`,
`TALL_MIN_FACTOR`, `SHORT_FACTOR`, `NARROW_FACTOR`), the Phase 3 title-score
thresholds, `MIN_PIECE_PX`, and (if needed) the crop boxes / `HEADER_ALLOWANCE_CM` /
`TOP_FURNITURE_FRAC`, re-running affected units offline to check.

### Step 7.4 — Final green gate

`cargo test`, `cargo clippy --all-targets -- -D warnings`, and
`cargo fmt --all --check` all pass.

### LLM verification
- Full `cargo test` green; clippy and fmt clean.
- `pdfinfo` on each regenerated `tomus_*_inner.pdf` shows balanced counts within the
  cap.

### Human verification
- End-to-end read of Tomus I: correct front matter (wrapped Lives, bottom plate, both
  Monitum pages), no blanks, clean unit starts (no duplicated titles, no truncation),
  no furniture bleed.

---

## Notes on scope and invariants

- The transcription (non-facsimile) build path is unchanged; its golden
  `front_matter_block_unchanged_for_transcription` test must stay green.
- All `plutarch.frontmatter.tags.json` and `facsimile_units.json` edits are deliberate,
  reviewable cache edits (the sidecars are the single source of truth and are
  hand-editable), per the faithfulness invariant in `CLAUDE.md`.
- The normal (no-flag) build stays fully offline and deterministic; Claude Vision is
  used only under `--review-lines`, and its conclusions are cached in the committed
  unit table.
- The 2026-10-03 plan stays on disk as the record of the work it covered; this plan
  carries its open items forward.
