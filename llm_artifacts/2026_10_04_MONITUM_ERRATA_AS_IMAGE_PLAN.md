# Monitum: re-typeset the prose, embed the Greek errata as images

## Context

On the facsimile build path the body is reproduced as source pixels, but the front
matter is re-typeset from an AI-vision transcription (the Monitum in Tomus I, the
Praefatio in v2's first tomus). The Monitum spans two source pages — v1 p12 and
v1 p13 — and has two very different halves:

1. **Editorial prose** (v1 p12, top): "Qui summæ rei litterariæ et institutioni
   publicæ præsunt…", a clean Latin paragraph with only a few inline Greek words,
   ending exactly at the sentence **"Dœhnerus hæc mutanda esse significavit."**
2. **A corrigenda (errata) list** that begins immediately after that sentence
   ("Thesei cap. 1, lin. 7 legendum γραφὴν pro eo quod est συγγραφήν. …"), runs to the
   foot of p12, and fills all of p13. It is dense, almost entirely polytonic Greek
   fragments interleaved with abbreviations and old-style numerals.

The prose transcribes cleanly and reads well re-typeset. The errata do not: the p13
transcription request (`s_0_13`) returns an unparseable/over-long `PageDoc` even with
the lenient reply parser added by
`llm_artifacts/2026_10_04_FACSIMILE_LAYOUT_FIXES_PLAN.md` (Phase 5), so the Monitum
currently prints only its first page, and a re-typeset errata list would in any case
be error-prone to proofread against the scan.

This plan changes the Monitum to a **hybrid**: keep re-typesetting the prose **up to
and including "…significavit."**, and reproduce the errata (the rest of p12 plus all of
p13) as **scaled source-page images**, exactly like the other embedded front-matter
scans (QR notice, engraved Volumen title, printer's colophon). Reproducing the errata
as verbatim pixels is both more faithful (it is the printed page, not a risky
re-keying of dense Greek) and removes the fragile `s_0_13` transcription entirely. This
supersedes the Monitum-specific goal of that plan's Phase 5 (the lenient `parse_pagedoc`
stays — it is a general improvement — but it no longer needs to rescue `s_0_13`).

The Praefatio (v2) is unchanged by this plan; it stays fully re-typeset.

## Current state of the world

- **Monitum prose is emitted from a page list.** `fill_front_matter`
  (`src/facsimile/mod.rs`) sets `vol.prose = latex::front_prose_pages(&sidecar,
  &frontmatter::MONITUM_PAGES)` for Tomus I, where
  `MONITUM_PAGES: [(usize, u32); 2] = [(0, 12), (0, 13)]`
  (`src/facsimile/frontmatter.rs:137`).
- **`front_prose_pages`** (`src/latex.rs:308`) iterates the given `(source, page)`
  list, looks up each page's cached `result` (`PageDoc`), and renders **all** its
  blocks via `front_prose_page(doc, true)` (a `\freshpage`, a `\markboth{}{}` reset,
  the block stream, a `\clearpage`).
- **Embedded scans** are rendered offline by `render_front_scans`
  (`src/facsimile/frontmatter.rs:208`): `render_and_crop` renders a source page at
  `TRANSCRIBE_DPI`, crops it to a tuned fractional box (`crop_rect`), and writes a
  named PNG (`front_qr_name`, `front_volumen_name`, `front_colophon_name`). The crop
  fractions are named constants (`QR_CROP`, `VOLUMEN_CROP`, `COLOPHON_CROP`) tuned
  after a real-eyes check.
- **Assembly** (`src/facsimile/assemble.rs::inner_document`) emits, for an
  `emit_front_matter` volume: the Tomus title page, then `latex::embed_scan(...)` for
  the QR (every tomus), the Volumen title (first of source), the colophon (Tomus I),
  then `vol.prose`. `embed_scan` (`src/latex.rs:390`) wraps one
  `\includegraphics[width=\textwidth,height=\textheight,keepaspectratio]` in a
  `\freshpage … \clearpage`, i.e. one scaled-to-fit image per page.
- **Transcription scope.** `transcribe_front_matter`
  (`src/facsimile/frontmatter.rs:49`) classifies boundary pages, then transcribes and
  reconciles **every** `FrontProse` page (`in_scope` = the set of `FrontProse`
  `(source, page)`), so p13 is transcribed and `s_0_13` fails each run. `work_remains`
  (`src/facsimile/frontmatter.rs:32`) reports work pending while any `FrontProse` page
  has `result == None`, which is true of p13 forever.
- **Cleanup** already treats any `front_*.png` as a generated artifact
  (`is_generated_artifact`, `remove_composed_pngs` in `src/facsimile/mod.rs`), so new
  `front_monitum_*.png` files need no cleanup changes.
- **Two unrelated `MONITUM_PAGES` constants exist.** The one this plan renames is the
  frontmatter **page list** (`frontmatter.rs:137`, `[(usize, u32); 2]`). A *separate*
  `pub const MONITUM_PAGES: u32 = 2` lives in `src/volumes.rs:30` — the Monitum's
  front-matter **page-overhead** estimate, consumed by `front_overhead`
  (`volumes.rs:43`, which feeds the balanced volume split and the `--no-compile` page
  estimate). These must not be conflated: the volumes constant keeps its name and is
  only *revalued* (Step 1.4), while the frontmatter page list is renamed.
- `README.md` describes the facsimile front matter as "the v1 Monitum, v2 Praefatio,
  and derived title pages are full-width prose re-typeset to the trim measure."

## Desired end state

- The Tomus I Monitum front matter is, in order: the re-typeset **prose** (v1 p12, the
  "Qui summæ…" paragraph through "…significavit." only), then **two embedded images** —
  the errata tail of v1 p12 and the full errata of v1 p13 — each scaled to fit a page
  like the other front scans.
- v1 p13 is **never transcribed**: no `s_0_13` request is issued, `work_remains`
  does not flag it, and a fully-cached build makes zero front-matter API calls.
- The re-typeset prose stops at the block containing "mutanda esse significavit."; the
  errata that the p12 transcription also captured are **not** re-typeset (they appear
  only as the image), so nothing is duplicated and no Greek errata are re-keyed.
- The faithfulness invariant holds: the errata are reproduced as the verbatim source
  pixels, and the prose text is unaltered (only its extent is chosen).
- `cargo test`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo fmt --all --check` pass; a regenerated Tomus I shows the prose then the two
  errata images, with the running heads, folios, signature line, and ornament cropped
  away.

## Tech Learnings

- **`\includegraphics[...,keepaspectratio]` scaling.** With both `width=\textwidth`
  and `height=\textheight` and `keepaspectratio`, LaTeX scales the image to the larger
  factor that still fits both bounds. A tall crop (p13's full errata column) fills the
  page height; a short wide crop (p12's errata tail) fills the width and leaves
  vertical whitespace. Both are acceptable "scaled-to-fit" results; the crop boxes
  control how much whitespace surrounds each.
- **Why a textual sentinel, not a hand-trimmed cache.** The p12 transcription captures
  the whole page (prose + errata start). Truncating the *rendered* prose at a stable
  printed phrase ("mutanda esse significavit.") survives a `--retag` re-transcription,
  whereas deleting blocks from the cached `result` would be silently re-added the next
  time p12 is re-transcribed. Selecting where the re-typeset prose ends is not a
  text edit — the dropped errata are reproduced verbatim as the image — so it does not
  violate the faithfulness gate.

---

## Phase 1 — Prose truncation and p13 transcription exclusion
### Suggested implementer: Opus

The logic-bearing half: end the re-typeset prose at the sentinel, and stop sending p13
to the API.

### Step 1.1 — Truncate the re-typeset Monitum prose at the sentinel

In `src/latex.rs`, add a page-list prose renderer that stops at a sentinel phrase:

```rust
/// Like [`front_prose_pages`], but emits blocks only up to and including the first
/// block whose text contains `sentinel`, then stops (across the whole page list). Used
/// for the facsimile Monitum, whose re-typeset prose ends at "…significavit." and
/// whose remaining errata are reproduced as images instead. If no block contains the
/// sentinel, every block is emitted (a safe degradation) — a warning is the caller's
/// responsibility.
pub(crate) fn front_prose_pages_until(
    sidecar: &Sidecar,
    pages: &[(usize, u32)],
    sentinel: &str,
) -> String
```

`render_blocks` (`src/latex.rs:430`) already takes a `&[Block]` slice, so no
block-by-block loop is needed. Factor `front_prose_page` to render from a blocks slice
(e.g. a `front_prose_page_blocks(blocks: &[Block], reset_running_head: bool)` that
`front_prose_page` calls with `&doc.blocks`), so both the full and truncated renderers
share the `\freshpage` / `\markboth{}{}` / `\clearpage` framing instead of duplicating
it. Then `front_prose_pages_until`, for each listed page, finds the index `idx` of the
first block whose `text` contains `sentinel`:
- if found, emit `front_prose_page_blocks(&doc.blocks[..=idx], true)` and **return**
  (later blocks and later pages are not emitted);
- if not found on that page, emit the whole page and continue to the next.

If the sentinel is **never** found across all listed pages, emit a loud
`eprintln!` warning — otherwise the errata are both re-typeset *and* imaged, a visible
duplication to be caught in Phase 3.

In `src/facsimile/frontmatter.rs`, replace `MONITUM_PAGES` with the prose-only page
list and add the sentinel constant:

```rust
/// The single source page whose prose (through "…significavit.") is re-typeset as the
/// Monitum; the errata that follow are reproduced as images (see MONITUM_IMAGE_PAGES).
pub const MONITUM_PROSE_PAGES: [(usize, u32); 1] = [(0, 12)];
/// The re-typeset Monitum prose ends at the block containing this printed phrase; the
/// earlier "significavit" in the body is mid-paragraph, so this full phrase is unique.
pub const MONITUM_PROSE_SENTINEL: &str = "mutanda esse significavit.";
```

In `src/facsimile/mod.rs::fill_front_matter`, change the Tomus I prose line to:

```rust
latex::front_prose_pages_until(
    &sidecar,
    &frontmatter::MONITUM_PROSE_PAGES,
    frontmatter::MONITUM_PROSE_SENTINEL,
)
```

Leave the Praefatio branch (`front_prose_pages(&sidecar, &PRAEFATIO_PAGES)`) unchanged.

#### LLM verification
- `cargo build` clean.
- `grep -rn "MONITUM_PAGES" src/` shows it **only** in `src/volumes.rs` (the unrelated
  overhead constant, handled in Step 1.4); the frontmatter page list is now
  `MONITUM_PROSE_PAGES` and no longer appears.

#### Human verification
- None (covered by Step 1.3 and Phase 3).

### Step 1.2 — Exclude v1 p13 from front-matter transcription

In `src/facsimile/frontmatter.rs`, add the image-page list and honor it in both the
scope filter and the work check:

```rust
/// Monitum pages reproduced as embedded images rather than transcribed. Excluded from
/// transcription so no request is issued for them and `work_remains` never flags them.
pub const MONITUM_IMAGE_PAGES: [(usize, u32); 1] = [(0, 13)];
```

- In `transcribe_front_matter`, build the `in_scope` set as the `FrontProse` pages
  **minus** `MONITUM_IMAGE_PAGES`, so p13 is never rendered or sent even though it is
  classified `FrontProse`.
- In `work_remains`, skip a `FrontProse` page that is in `MONITUM_IMAGE_PAGES` when
  deciding whether a result is still owed (otherwise p13's permanently-`None` result
  would report work forever and rebuild the vision client every run).

Leave p13's cached record as-is (role `FrontProse`, `result: null`); it is harmless
once excluded. (Optionally, a maintainer may hand-edit its role to `front_image` for
tidiness — a sanctioned cache edit — but the code exclusion is the robust mechanism and
does not depend on it.)

#### LLM verification
- `cargo build` clean.
- `grep -n "MONITUM_IMAGE_PAGES" src/facsimile/frontmatter.rs` shows the const used in
  both `transcribe_front_matter` and `work_remains`.

#### Human verification
- None.

### Step 1.3 — Tests

In `src/latex.rs` tests:
- `front_prose_pages_until_stops_at_sentinel` — a `PageDoc` with a prose paragraph
  block ending "…mutanda esse significavit." followed by two errata blocks renders only
  through the prose block; the errata blocks' text does not appear in the output.
- `front_prose_pages_until_emits_all_when_sentinel_absent` — with no block containing
  the sentinel, all blocks are emitted (degradation path).

In `src/facsimile/frontmatter.rs` tests:
- `work_remains_ignores_monitum_image_page` — a sidecar where the only unfinished
  `FrontProse` page is v1 p13 reports `work_remains == false`.
- `transcribe_excludes_monitum_image_page` — with v1 p12 and v1 p13 both `FrontProse`,
  the stubbed backend is asked for `s_0_12` (and not `s_0_13`); assert no submitted
  custom id contains `_13`. (Mirror the existing `frontmatter_only_transcribes_frontprose`
  stub setup.)

#### LLM verification
- `cargo test front_prose_pages_until` and `cargo test monitum` pass;
  `cargo test frontmatter` passes.

#### Human verification
- None.

### Step 1.4 — Revalue the Monitum front-matter page overhead

The Monitum no longer occupies two re-typeset pages; it is now the prose (≈1 page) plus
two errata images (2 pages) ≈ **3 pages**. Update `pub const MONITUM_PAGES: u32` in
`src/volumes.rs` from `2` to `3` (keep the name and its doc comment, updating the
comment to "the re-typeset Monitum prose page + two embedded errata images, on Tomus I
only"), so `front_overhead` — and thus the balanced volume split and the `--no-compile`
page estimate — accounts for the real page cost. Leave it deliberately generous and
tunable like the sibling overhead constants.

Update any `front_overhead` test expectation in `src/volumes.rs` tests that pins the
Tomus I overhead to the old value (search the tests for a `front_overhead(0, true,
true)` assertion and bump its expected count by one).

#### LLM verification
- `cargo test volumes` passes (with the updated expectation).
- `cargo build` clean.

#### Human verification
- None (the real compile self-corrects page counts; this only sharpens the split /
  estimate).

---

## Phase 2 — Render and embed the errata images
### Suggested implementer: Sonnet

The mechanical half: render the two errata crops offline and place them after the prose
on Tomus I.

### Step 2.1 — Add the Monitum errata scans

In `src/facsimile/frontmatter.rs`, add names and crop constants beside the existing
ones, and render them in `render_front_scans`:

```rust
/// The embedded Monitum errata-tail scan (v1 p12, below "…significavit.") — Tomus I.
pub fn front_monitum_a_name() -> &'static str { "front_monitum_a.png" }
/// The embedded Monitum errata scan (all of v1 p13) — Tomus I.
pub fn front_monitum_b_name() -> &'static str { "front_monitum_b.png" }

/// Crop boxes `(left, top, right, bottom)` tuned after a real-eyes check (Phase 3).
/// A: v1 p12 — drop everything above the errata ("Thesei cap. 1…" begins ~0.75 down)
/// and the "PLUTARCHUS. I" signature / folio at the foot.
const MONITUM_A_CROP: (f32, f32, f32, f32) = (0.12, 0.75, 0.12, 0.05);
/// B: v1 p13 — drop the "ij / MONITUM." running head at top and the ornament/blank
/// at the foot.
const MONITUM_B_CROP: (f32, f32, f32, f32) = (0.12, 0.10, 0.12, 0.34);
```

In `render_front_scans`, under the existing `if is_tomus_one { … }` block (Tomus I is
always source 0, so `pdf` is v1), render both crops:

```rust
render_and_crop(pdf, 12, MONITUM_A_CROP, front_monitum_a_name(), out_dir)?;
render_and_crop(pdf, 13, MONITUM_B_CROP, front_monitum_b_name(), out_dir)?;
```

(The colophon render already in that block stays.)

#### LLM verification
- `cargo build` clean.
- A `--no-compile` Tomus I build (Step 2.3) writes `front_monitum_a.png` and
  `front_monitum_b.png` into `--out`.

#### Human verification
- Open the two PNGs and confirm A is the p12 errata tail (starting at "Thesei cap. 1")
  with no signature line, and B is the p13 errata with no running head or ornament.
  Adjust the crop constants if a furniture remnant survives or text is clipped (tuned
  for real in Phase 3).

### Step 2.2 — Embed the errata images after the prose on Tomus I

In `src/facsimile/assemble.rs::inner_document`, after `doc.push_str(&vol.prose);` and
inside the `emit_front_matter` block, emit the two Monitum errata scans on Tomus I
only, immediately after the prose:

```rust
if vol.tomus == 1 {
    doc.push_str(&latex::embed_scan(frontmatter::front_monitum_a_name()));
    doc.push_str(&latex::embed_scan(frontmatter::front_monitum_b_name()));
}
```

#### LLM verification
- `cargo build` clean.
- `grep -n "front_monitum" src/facsimile/assemble.rs` shows both embeds.

#### Human verification
- None (covered by Phase 3).

### Step 2.3 — Tests

In `src/facsimile/assemble.rs` tests:
- `tomus_one_embeds_monitum_errata_images` — `inner_document` for a Tomus I
  `FacsimileVolume` (with `emit_front_matter` true) contains
  `front_monitum_a.png` and `front_monitum_b.png`, both after the prose placeholder;
  a non-Tomus-I volume contains neither.

#### LLM verification
- `cargo test facsimile` / `cargo test assemble` pass.

#### Human verification
- None.

---

## Phase 3 — Regenerate, tune crops, verify
### Suggested implementer: Opus

### Step 3.1 — Regenerate Tomus I and tune the crop boxes

Run a Tomus I build (a full build, or a scoped one that still emits front matter) and
open `tomus_1_inner.pdf`. Confirm the Monitum reads: the "Qui summæ…" prose through
"…significavit." as re-typeset text, then the p12 errata tail, then the p13 errata, as
images. Tune `MONITUM_A_CROP` / `MONITUM_B_CROP` so each errata image is tight to the
text — no running head, folio, signature line ("PLUTARCHUS. I"), or ornament, and no
clipped first/last errata line — re-rendering until the real-eyes check is satisfied
(same workflow as the existing `*_CROP` constants).

#### LLM verification
- `pdfinfo build_facsimile/tomus_1_inner.pdf` succeeds; the Tomus I page count is
  consistent with "prose (≈1 page) + 2 errata image pages" replacing the former two
  transcribed Monitum pages.

#### Human verification
- Read the regenerated Monitum in `tomus_1_inner.pdf`: prose then two clean errata
  images, no furniture, no clipping.
- Confirm the sentinel matched: no errata line (e.g. "Thesei cap. 1…") appears **both**
  as re-typeset text and in an image. If it does — or if the build logged the
  "sentinel not found" warning from Step 1.1 — the real cached p12 blocks split the
  "mutanda esse significavit." phrase; adjust `MONITUM_PROSE_SENTINEL` to a sub-phrase
  that lies within a single cached block (inspect `plutarch.frontmatter.tags.json`'s
  v1 p12 `result` to choose it).

### Step 3.2 — Final green gate

#### LLM verification
- `cargo test`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo fmt --all --check` all pass.

#### Human verification
- None beyond Step 3.1.

### Step 3.3 — Update the README

In `README.md`, rewrite the facsimile front-matter sentence so it reflects the hybrid
Monitum: the prose through "…significavit." is re-typeset, and the Greek errata list is
reproduced as embedded scaled images alongside the QR / Volumen-title / colophon scans;
the Praefatio remains fully re-typeset. Rewrite the sentence in place (per the
"Updating existing markdown documents" rule), do not append a note.

#### LLM verification
- `grep -ni "errata\|monitum" README.md` shows the updated description.

#### Human verification
- Read the paragraph and confirm it matches the shipped behavior.

---

## Notes on scope and invariants

- Only the **Monitum** changes; the Praefatio stays fully re-typeset.
- The errata are reproduced as verbatim source pixels — strictly more faithful than
  re-keying dense Greek — and the prose text is unaltered (only its extent is chosen at
  the "…significavit." sentinel), so the faithfulness gate holds.
- The lenient `parse_pagedoc` from the 2026-10-04 layout-fixes plan stays; it is a
  general robustness improvement and is simply no longer relied on for `s_0_13`.
- `front_monitum_*.png` match the existing `front_*` cleanup globs, so no change to
  `is_generated_artifact` / `remove_composed_pngs` is needed.
- This plan is retained under `llm_artifacts/` as the dated record of why the Monitum
  is a prose/image hybrid.
