# Facsimile front-matter rework and layout fixes

## Context

The facsimile build path (`src/facsimile/`) reproduces the 1857 Didot *Plutarchi
Vitae* body pixel-for-pixel from the source scans and re-typesets only the front
matter. The current output (`build_facsimile/tomus_*_inner.pdf`) has a set of
front-matter and layout problems the owner has catalogued from a real-eyes read
of the compiled PDFs:

- The front matter carries the wrong pages in the wrong order: a redundant
  "Operum Volumen" half-title is re-typeset, the engraved Didot title page is
  re-typeset as prose instead of shown as the scan it is, the Google
  reproduction-notice page and the printer's colophon are missing, and there are
  two separate "TOMUS I" pages.
- Several body pages overflow their bottom margin, and a few are blank except for
  the running head, interrupting flowing sentences.
- The original running head and source page number occasionally bleed into the
  reproduced body text.
- Each Life/Comparatio opens with a standalone display-title page that the owner
  wants replaced by an inline chapter header.
- The volumes badly exceed the target page budget (Tomus I is 544 pages against a
  400-page target / 500-page cap).

This plan reworks the facsimile front matter to the owner's specified structure
and fixes the layout, furniture, and volume-splitting bugs. The source text is a
public-domain edition reproduced faithfully; the faithfulness invariant in
`CLAUDE.md` applies to the re-typeset prose (Monitum / Praefatio).

## Current state of the world

### Front-matter data (`plutarch.frontmatter.tags.json`)

The committed, hand-editable front-matter sidecar classifies boundary pages and
holds the re-typeset prose. Relevant classified pages today (verified by
rendering each page; v1 = source 0, 643 pp; v2 = source 1, 722 pp):

| source | page | role today | what the scan actually is |
|--------|------|-----------|---------------------------|
| 0 (v1) | 1  | skip        | Google reproduction-notice page (logo + QR + `books.google.com`) |
| 0 (v1) | 2–7 | skip       | cover, marbled endpaper, blank flyleaves |
| 0 (v1) | 8  | front_prose | half-title: ΠΛΟΥΤΑΡΧΟΥ ΣΥΓΓΡΑΜΜΑΤΩΝ / PLUTARCHI OPERUM VOLUMEN PRIMUM |
| 0 (v1) | 9  | skip¹       | printer's **colophon**: "PARIS. — EXCUDEBANT FIRMIN DIDOT FRATRES, VIA JACOB, 56." |
| 0 (v1) | 10 | front_prose | engraved **Volumen title page**: ΠΛΟΥΤΑΡΧΟΥ ΒΙΟΙ / PLUTARCHI VITÆ / … / vignette / PARISIIS … M DCCC LVII |
| 0 (v1) | 11 | skip        | blank |
| 0 (v1) | 12 | front_prose | **MONITUM**, page 1 |
| 0 (v1) | 13 | body²       | **MONITUM**, page 2 — *not currently transcribed* |
| 1 (v2) | 1  | skip        | Google reproduction-notice page |
| 1 (v2) | 10 | front_prose | half-title: PLUTARCHI OPERUM VOLUMEN SECUNDUM |
| 1 (v2) | 12 | front_prose | engraved **Volumen title page** (v2) |
| 1 (v2) | 14 | front_prose | **PRÆFATIO**, part 1 (its own `PRÆFATIO.` heading, "Quum hoc…") |
| 1 (v2) | 15 | body²       | presumed **PRÆFATIO** continuation — *untranscribed; verify before retag (Step 1.0)* |
| 1 (v2) | 16 | body²       | presumed **PRÆFATIO** continuation — *untranscribed; verify before retag (Step 1.0)* |
| 1 (v2) | 17 | front_prose | **PRÆFATIO**, part 2 (its **own** `PRÆFATIO.` heading, "Interpretationem latinam… Xylandri…") |

¹ The colophon at v1 p9 is currently tagged `skip` and has never been transcribed,
so its identity as the colophon is inferred from a render, not from the cache.
Phase 2 embeds it as a scan addressed by the `(0, 9)` constant regardless of role,
but Step 1.0 must confirm by rendering that p9 is in fact the colophon.

² v1 p13 and v2 p15/p16 are currently tagged `body` (a classified record exists;
none is `front_prose` yet). They are *presumed* to be prose continuation of the
preceding Monitum/Praefatio page, but this is unverified — Step 1.0 renders them
before Step 1.1 retags any to `front_prose`, so a plate or blank is not forced into
the re-typeset prose. Note v2 p14 and p17 each carry their **own** `PRÆFATIO.`
heading in the source: p17 opens a distinct section (on Xylander's Latin version),
so its heading is genuine and must be preserved (see Step 1.2).

The v1 p12 Monitum transcription also contains the printer's **signature/catchword
line** `PLUTARCHUS. I` (page furniture at the foot of the page), which renders as
though it were Monitum prose.

`classify()` in `src/pages.rs:28` only classifies boundary pages whose record is
absent (`sidecar.page(si, p).is_none()`, line 43), so **hand edits to existing
page roles survive a re-run**; a `front_prose` page with `result == None` is
(re)transcribed because `work_remains()` (`src/facsimile/frontmatter.rs:32`)
returns true for it.

### Front-matter generation (`src/latex.rs`)

`front_matter_block(sidecar, source, tomus)` (`src/latex.rs:220`) composes, for a
source's first volume only:

1. `title_page()` — a re-typeset title from derived `FrontMatter{title,editor,date}`
   (currently empty) plus a `TOMUS <roman>` line.
2. `front_images()` — every `PageRole::FrontImage` scan for the source, included
   with `\includegraphics[trim=0 0 0 0,clip,width=\textwidth,height=\textheight,keepaspectratio]`.
3. `front_prose()` — every `PageRole::FrontProse` page's `result` blocks.

Separately, `tomus_label(tomus)` (`src/latex.rs:304`) emits a second `TOMUS
<roman>` page. The facsimile inner (`src/facsimile/assemble.rs:50`
`inner_document`) emits, per volume: flyleaves → `front_matter` (on
`is_first_of_source`) → `tomus_label` → `\beginbodynumbering` → units → back
flyleaves → signature padding. `front_matter` is filled by
`fill_front_matter()` (`src/facsimile/mod.rs:333`) from `front_matter_block`.

`Fonts::default()` is Old Standard at 10 pt (`src/latex.rs:16`); the facsimile
preamble (`inner_preamble`, `InnerOpts{running_head:true}`) defines
`\heading{…}` (`\centering\bfseries\large`), `\freshpage` (clear to a recto), and
the centered `\leftmark` running head (`src/latex.rs:163`).

### Per-unit emission (`src/facsimile/assemble.rs`)

`unit_block()` (`src/facsimile/assemble.rs:78`) opens each unit with `\freshpage`,
sets `\markboth` to the Latin title, then emits a **standalone display-title
page** `\heading{LATIN\\ GREEK}` followed by `\clearpage`, then one
`\includegraphics[width=\textwidth]` per composed body image, each followed by
`\par\clearpage`. `PlacedUnit` (`assemble.rs:20`) carries `id`, `latin_title`,
`greek_title`, `image_names` — but **not** `kind`.

### Body composition (`src/facsimile/reflow.rs`)

`compose_pages()` (`reflow.rs:253`) flattens a unit's line-strips (cut-point
intervals) into fixed-width (`tb_w`) canvases, wrapping to a new canvas when
`used_h + sh > budget` with the current canvas non-empty, where `budget =
geom.content_height()` (= `tb_h`, the full text height). `pack_canvas()`
(`reflow.rs:293`) makes a canvas exactly `pad_y + Σ strip heights` tall. A strip
whose own scaled height `sh` exceeds `budget` is placed atomically, producing a
canvas **taller than the text block**. Verified: `facsimile_theseus_7.png` is
1004×**2868** px while a normal page canvas is ~1004×**1420** px (budget ≈ 1429 px
at 300 dpi). Such a strip comes from a page where line-banding merged many lines
into one giant band (dark scan / marginal line-number ticks included in the
column crop).

Included at `width=\textwidth`, a 2868-px-tall canvas is ~24 cm tall on a 16 cm
page. LaTeX then either overflows it past the bottom margin (the "lower margin is
off" pages — e.g. inner pp. 25, 81, 127) or, when it will not fit the remaining
space, bumps the oversized box to the next page and leaves the current page blank
but for the running head (the "blank page interrupting a sentence" pages — e.g.
inner pp. 23, 37, where a sentence runs p22 → p24 across a blank p23).

### Furniture removal (`src/facsimile/lines.rs`)

`drop_furniture()` (`lines.rs:72`) keeps only bands whose center lies below the
top 6.5 % (`TOP_FURNITURE_FRAC`) and above the bottom 5 % (`BOTTOM_BODY_FRAC`) of
page height, then drops a single leading/trailing band separated by a gap >
2.2× the median (`drop_gap_outliers`). On some interior pages the original
running head + source folio survives this (its center is just below 6.5 %, or its
following gap is not a 2.2× outlier), so a fragment like `IO COMPARATIO. 227`
appears as the first body line. Verified at tomus 1 inner p515 and tomus 7 inner
p18.

### Volume splitting (`src/volumes.rs`)

`split_source()` (`volumes.rs:187`) greedily adds whole pair-groups to a volume
and closes it the moment the running sum reaches `TARGET_PAGES` (400), bounded by
`MAX_PAGES` (500). Verified v1 pair-group body-page counts (from the composed
PNGs): G1 = 98, G2 = 96, G3 = 80, G4 = 101, G5 (Pericles/Fabius/Comp) = 98, …
After G4 the running total (with 16 pp overhead) is ~391 — just under 400 — so the
splitter grabs the whole ~98-page G5 and closes at ~489, which pads to the 544-page
Tomus I. Closing at G4 instead gives ~375 body pp → ~416 physical pp, matching the
owner's "417" expectation. `FRONT_MATTER_PAGES` (8) is charged to the first volume
of each source only; `write_wrappers_from_estimate` (`mod.rs:388`) counts
`image_names.len() + 1` per unit (the `+1` is the standalone title page).

## Desired end state

### Front-matter order (facsimile inner, per tomus)

1. **Tomus Title Page** — *every tomus*. Centered, Old Standard, replacing both the
   old derived `title_page` and the old `tomus_label`. This is the first page with
   content. Lines, centered:
   - `PLUTARCHI VITÆ` (Æ ligature)
   - `SECUNDUM CODICES PARISINOS RECOGNOVIT`
   - `THEODOR DŒHNER` (Œ ligature)
   - `TOMUS <roman>`
   - the Latin names of the **Lives in this tomus only** (no Comparationes), in
     order, separated by bullets. The names come from each unit's corpus
     `latin_title`, which is stored **uppercase** (`THESEUS`, `FABIUS MAXIMUS`,
     `MARCIUS CORIOLANUS`); they are rendered in small caps (see Step 2.2 for the
     exact casing), reading e.g. `Theseus • Romulus • Lycurgus • Numa • Solon •
     Poplicola • Themistocles • Camillus`
   - `EX LIBRIS`
   - `MARTINEZ-DIRKS`
2. **Google QR Page** — *every tomus*. The source reproduction-notice scan (v1 p1
   for v1 tomi, v2 p1 for v2 tomi), embedded directly, scaled to the page.
3. **Volumen Title Page** — *first tomus of each volumen* (`is_first_of_source`).
   The engraved Didot title scan (v1 p10 / v2 p12), Google watermark cropped off,
   scaled to the page.
4. **Colophon** — *Tomus I only* (the first tomus overall). The printer's colophon
   scan (v1 p9), cropped, embedded; placed **after** the Volumen Title Page.
5. **Monitum** (*Tomus I only*, re-typeset from v1 p12–13) **or Praefatio** (*first
   tomus of v2*, re-typeset from the v2 pages Step 1.0 confirms as Praefatio prose —
   expected **p14–p17** inclusive, with p15 and p16 as continuation rather than
   skipped).

The old half-title ("…OPERUM VOLUMEN PRIMUM/SECUNDUM", v1 p8 / v2 p10) is dropped
entirely. There is exactly one "TOMUS" page (the Tomus Title Page). The Monitum
reproduces **both** its pages (v1 p12–13) and the Praefatio **every** page Step 1.0
confirms (expected v2 p14–17), each excluding printer signatures/catchwords such as
the `PLUTARCHUS. I` line.

### Body and per-unit layout

- No composed body canvas exceeds the text block, so no body page overflows its
  bottom margin or is bumped to a blank page. A small, uniform bottom margin is
  visible on full pages.
- The original running head / source folio never appears in reproduced body text.
- Each Life/Comparatio has **no** standalone display-title page. Its first body
  page opens with a large centered chapter header showing the **Latin** title only
  (no Greek), and that first page carries **no** running head; subsequent pages of
  the unit keep the centered Latin running head.

### Volume budget

- The splitter closes each volume at the pair-group boundary nearest the 400-page
  target (bounded by the 500-page cap), so v1 Tomus I ends after group 4
  (Themistocles/Camillus) at ~416 pp and Pericles/Fabius/Comparatio open Tomus II.

### Build directory

After a full compiling run, `build_facsimile/` holds only the `.tex`/`.pdf`
deliverables and the reusable `render_cache/`; the ~3 GB of composed strip PNGs
are gone. A fresh run starts from a clean slate (no orphan PNGs or stale tomus
PDFs from a prior boundary), and a `--no-compile` run keeps the PNGs so a manual
compile still works.

### Tests green

`cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --all
--check` all pass; a regenerated sample is eyeballed.

## Tech Learnings

- **`\includegraphics` and the page box.** An image placed as `\noindent\includegraphics…\par`
  is a single unbreakable box. TeX's `\topskip` makes the box's *top* align with
  the top of the text block when the box is tall, so a box exactly `\textheight`
  tall fits perfectly — but a box *taller* than `\textheight` cannot be split: TeX
  either sets it overfull (it runs past the bottom margin) or, if too little
  vertical space remains on the current page, moves the whole box to the next page
  and leaves the current one nearly empty. Guaranteeing every composed canvas is
  **≤ the text-block height** is therefore the structural fix for both the
  overflow and the blank-page symptoms.
- **graphicx `trim`/`clip` vs. cropping in Rust.** `trim=l b r t,clip` crops in the
  image's own units and is fiddly to keep watermark-tight. This codebase already
  crops and scales images deterministically with the `image` crate
  (`src/facsimile/reflow.rs`, `columns.rs`), so cropping the front scans in Rust to
  a tunable fractional box — producing a watermark-free PNG that is then included
  with `width=\textwidth,height=\textheight,keepaspectratio` — is the idiomatic,
  auditable choice.
- **Ligatures in LuaLaTeX + Old Standard.** `Æ`/`Œ` can be written as literal UTF-8
  in the `.tex` (Old Standard has the glyphs); no input encoding package is needed
  under LuaLaTeX with `fontspec`.
- **Nearest-boundary partitioning.** Replacing "close once the sum reaches the
  target" with "close at whichever side of the next group is nearer the target"
  turns a floor-to-exceed rule into a balanced one, avoiding a whole-group
  overshoot when a volume sits just under target.

---

## Phase 1 — Front-matter data: Monitum completeness and furniture
### Suggested implementer: Opus

The embedded scans (QR, Volumen title, colophon) are addressed in code by
`(source, page)` constants in Phase 2, **not** via `PageRole::FrontImage`, so this
phase only has to make the re-typeset prose correct: the Monitum must reproduce
both its pages and must not contain printer furniture.

### Step 1.0 — Verify the ambiguous pages before retagging (no API cost)

Before changing any role, render these pages at a legible DPI (reuse
`render::render_page`, as a dry-run harness or by eye from the already-built PDFs)
and confirm what each one actually is, because the cache does not yet establish it:

- **v1 p9** is tagged `skip` and has never been transcribed. Confirm it is the
  printer's colophon (`PARIS. — EXCUDEBANT FIRMIN DIDOT FRATRES…`); the `(0, 9)`
  colophon constant in Phase 2 depends on this.
- **v1 p13** is tagged `body`. Confirm it is the Monitum's second page (its first
  words should continue p12's trailing catchword `Κατελθὼν δὲ`).
- **v2 p15 and p16** are tagged `body`. Confirm each is Praefatio prose (not a
  plate, blank, or a non-prose insert). If either is **not** prose, do **not**
  retag it in Step 1.1; instead record what it is and adjust the Praefatio page
  list in Steps 1.1 and 2.5 accordingly (e.g. a Praefatio of `[14, 15, 17]` if p16
  is a blank). The `[14, 15, 16, 17]` span below is the expected case, not a
  guarantee.

This step is a human/real-eyes check with zero network cost; it gates the retag so
the faithfulness invariant is not violated by transcribing a non-prose page as
prose.

### Step 1.1 — Capture the Monitum's second page and strip the signature line

1. In `plutarch.frontmatter.tags.json`, set the v1 (source 0) **page 13** record's
   `role` to `front_prose` and its `result`/`pass_a`/`pass_b` to `null` so
   `work_remains()` schedules it for transcription. (If no record exists for
   source 0 page 13, add one with `role: "front_prose"` and null passes, matching
   the shape of the p12 record.)
2. Set the v1 (source 0) **page 12** record's `result` to `null` as well, so it is
   re-transcribed under the improved prompt (Step 1.2). Leave its `role` as
   `front_prose`.
3. The v2 Praefatio is expected to span **source 1, pages 14 through 17 inclusive**;
   p15 and p16 are currently tagged `body` and are the presumed continuation. **Only
   after Step 1.0 confirms p15/p16 are Praefatio prose**, set each confirmed page's
   `role` to `front_prose` and null its `pass_a`/`pass_b`/`result` so it is
   transcribed. (If Step 1.0 found a page is not prose, leave it out of the page
   list here and in Step 2.5.) Null the `result` of p14 and p17 as well so they are
   re-transcribed under the improved prompt (Step 1.2), stripping any
   signature/catchword furniture. Note p14 and p17 each legitimately carry a
   `PRÆFATIO.` heading in the source; both headings are faithful and are kept (the
   continuation rule in Step 1.2 suppresses a heading only on a page that has none
   of its own).
4. The old half-title (v1 p8 / v2 p10) and the re-typeset Volumen title (v1 p10 /
   v2 p12) `front_prose` records may remain in the cache; the Phase 2 generator
   does not read them. For tidiness, set the v1 p8 / v2 p10 records' `role` to
   `skip`. Do **not** touch v1 p9 (already `skip`), v1 p10, or v2 p12 roles —
   Phase 2 references those pages by constant, and their cache role is irrelevant to
   the new generator.

### Step 1.2 — Teach the front-prose prompt to drop printer furniture

In `src/llm/transcribe.rs`, extend `FRONT_PROMPT` so the re-typeset prose:
- **excludes** running heads, folios, printer's signatures and catchwords (e.g. a
  foot-of-page `PLUTARCHUS. I`), and engraver/printer imprints — these are page
  furniture, not prose;
- transcribes the body prose faithfully and, when a page carries only the tail of
  the previous page's section with **no heading of its own**, **continues across the
  page break** without inventing a heading (so the Monitum's p12→p13 and the
  Praefatio's p14→p15→p16 join cleanly). This rule keys off what the page actually
  shows: a page that *does* bear a heading in the source keeps it. v2 p17 opens a
  distinct section under its own `PRÆFATIO.` heading, so that heading is transcribed,
  not suppressed — the rule removes only headings the page does not have, never a
  heading the source prints.

This is consistent with the faithfulness invariant: furniture is not part of the
editorial prose, and excluding it mirrors how the body path drops running
heads/folios, while a heading the page genuinely prints is prose and is kept. Keep
the existing block-schema contract unchanged.

### Step 1.3 — Re-run front-matter transcription (cheap)

Run the facsimile build far enough to invoke `run_front_matter` (or a focused
harness) so the nulled pages (v1 p12, p13; v2 p14–17) are re-transcribed and
written back to `plutarch.frontmatter.tags.json`. Only these few pages hit the
API. Confirm the Praefatio reads continuously across p14→p17 with no dropped page
and no furniture.

### LLM verification
- `cargo test transcribe` and `cargo test frontmatter` pass.
- `grep -n "PLUTARCHUS" plutarch.frontmatter.tags.json` returns nothing inside a
  Monitum `result` text block.
- The source 0 page 13 record and each source 1 Praefatio page confirmed in
  Step 1.0 (expected p14–p17) exist with `role: "front_prose"` and a non-null
  `result`; any page Step 1.0 found to be non-prose is left as-is and excluded from
  the Praefatio page list.

### Human verification
- Read the re-typeset Monitum in a regenerated Tomus I: it runs across both source
  pages, reads continuously, and shows no `PLUTARCHUS. I` line or other furniture.
- Confirm the Praefatio (v2) still reads correctly.

---

## Phase 2 — New per-tomus front-matter assembly
### Suggested implementer: Opus

Replace the old `front_matter_block` + `tomus_label` emission in the **facsimile**
path with the owner's structure. Keep the transcription path
(`latex::inner_document`) compiling; its golden front-matter test
(`front_matter_block_unchanged_for_transcription`) must stay green, so **do not
change `front_matter_block`'s existing bytes** — add new builders instead and wire
them into the facsimile assemble only.

### Step 2.1 — Carry the data the Tomus Title Page needs

1. Add `kind: LifeKind` to `PlacedUnit` (`src/facsimile/assemble.rs:20`) and
   populate it in `run_facsimile` (`src/facsimile/mod.rs` compose loop, where
   `PlacedUnit` is built ~line 149) from `range.unit.kind`.
2. The Tomus Title Page lists this tomus's **Lives only** (kind == `Life`), in
   order, by Latin title. A `FacsimileVolume` already owns its `units`
   (`assemble.rs:33`); derive the list there.

### Step 2.2 — Add the front-matter builders (shared helpers in `latex.rs`)

Add to `src/latex.rs` (reusing `to_roman`, `escape_latex`):

- `pub(crate) fn tomus_title_page(tomus: usize, life_latin_titles: &[String]) -> String`
  — a `\freshpage`, `\thispagestyle{empty}`, centered block with the fixed lines
  (`PLUTARCHI VITÆ`, `SECUNDUM CODICES PARISINOS RECOGNOVIT`, `THEODOR DŒHNER`,
  `TOMUS <roman>`), then the Life list, then `EX LIBRIS` / `MARTINEZ-DIRKS`.
  Literal `Æ`/`Œ` are fine under LuaLaTeX + Old Standard. End with `\clearpage`.
  **Life-name rendering (pinned):** the corpus `latin_title` is already uppercase
  (`THESEUS`, `C. MARIUS`, `TIB. GRACCHUS`, `T. QUINCTIUS FLAMININUS`, `CATO MAJOR`).
  Do **not** re-case it by hand (naive title-casing mangles the abbreviated praenomina).
  Emit each name as `\textsc{\MakeLowercase{<escape_latex(title)>}}` — lower-casing
  feeds true small caps while the font restores the caps height, so `C. MARIUS`
  renders as small-cap "C.\ Marius" without manual case surgery — and join the names
  with `\,\textbullet\,`. Keep this transform in one helper. (If the owner's Phase 7
  real-eyes check prefers full caps instead of small caps, drop the
  `\MakeLowercase` in that one place.)
- `pub(crate) fn embed_scan(name: &str) -> String` — a `\freshpage`,
  `\thispagestyle{empty}`, centered `\includegraphics[width=\textwidth,
  height=\textheight,keepaspectratio]{<name>}`, `\clearpage`. (The PNG is already
  cropped in Rust, Step 2.4, so no `trim` is needed.)
- `pub(crate) fn front_prose_pages(sidecar: &Sidecar, pages: &[(usize, u32)]) -> String`
  — emit the `result` blocks of **exactly the listed `(source, page)` records**,
  in the given order, each opened with `\freshpage` and `\markboth{}{}` (so the
  prose pages carry no stale running head under the facsimile preamble's
  `running_head: true`). This is **page-list-driven, not role-driven**: the
  existing role-driven `front_prose(sidecar, source)` (`latex.rs:288`) would emit
  *every* `FrontProse` record for the source — which for v1 still includes the
  half-title at p8 (until Step 1.1 retags it `skip`) and the re-typeset Volumen
  title at p10, and for v2 the half-title at p10 and Volumen title at p12 — none of
  which belong in the Monitum/Praefatio, so it must **not** be used for the facsimile
  front prose. (The colophon at v1 p9 is `skip`, so role-driven emission would *not*
  pick it up; it is embedded as a scan in Step 2.5.) Refactor so `front_prose` (transcription
  path) and `front_prose_pages` (facsimile path) share one block-rendering helper
  rather than duplicating the `\freshpage` + `render_blocks` + `\clearpage` loop.

### Step 2.3 — Define the embedded-scan sources and names

Add, in `src/facsimile/frontmatter.rs`, both the `(source, page)` constants and
the matching file-name helper functions (so the Step 2.5 call sites resolve),
independent of `PageRole`:

- QR page: `(source, 1)` for the tomus's own source → `front_qr_name(source) ->
  "front_qr_s{source}.png"`
- Volumen title page: `(0, 10)` for v1, `(1, 12)` for v2 → `front_volumen_name(source)
  -> "front_volumen_s{source}.png"`; `front_volumen_page(source)` returns `10`/`12`
- Colophon: `(0, 9)` → `front_colophon_name() -> "front_colophon.png"`

### Step 2.4 — Crop the scans in Rust (tunable boxes)

Add a helper that renders a source page to PNG (reuse `render::render_page` at
`TRANSCRIBE_DPI`, as `render_front_images` does) and crops it to a fractional box
`(left, top, right, bottom)` removed, via the `image` crate, writing the named
PNG into the output dir. Replace `render_front_images` (`frontmatter.rs:136`) with
a function that renders exactly the three named scans a given tomus needs.

Default crop fractions (chosen from the rendered scans; **tunable, flagged for the
owner's real-eyes check**):

| scan | left | top | right | bottom | why |
|------|------|-----|-------|--------|-----|
| QR page | 0.04 | 0.04 | 0.04 | 0.04 | trim scan edges; keep the whole notice |
| Volumen title | 0.08 | 0.03 | 0.08 | 0.07 | bottom 7 % removes the faint "Digitized by Google"; sides tidy the wide margins |
| Colophon | 0.10 | 0.33 | 0.10 | 0.47 | crop to the central colophon band (ornament + imprint line); drops the watermark |

Expose these as named constants with a comment that they are tuned after a
real-eyes PDF check.

### Step 2.5 — Rewire the facsimile inner document

**Split of responsibilities.** `inner_document(vol, fonts)` has no `Sidecar`, so it
cannot read prose. The prose (Monitum/Praefatio) is therefore **prepared upstream
in `fill_front_matter`** (which holds the sidecar) via `front_prose_pages` and
stored on the volume; `inner_document` only *places* front matter using the
volume's own flags. Repurpose the `FacsimileVolume.front_matter` field into a
clearly named `prose: String` carrying the prepared prose string (and update that
field's doc comment — it currently says "prepended when this is the first volume of
its source… until Phase 5 fills it," which no longer describes it); the Tomus Title
Page and the embedded-scan includes are generated inside `inner_document` from
`vol` fields alone.

**Restricted runs.** `fill_front_matter` runs only when `!args.restricted()`
(`mod.rs:161`), so a `--unit`/`--pages` run renders **no** front scans. The new
`inner_document` must therefore not unconditionally emit `embed_scan(…)` — that
would reference `front_qr_s*.png` files that were never written and break the
compile. Add a bool `emit_front_matter: bool` to `FacsimileVolume`, set `true` for
a normal build and `false` for the restricted single volume (in `plan_volumes`,
`mod.rs`). `inner_document` emits the whole new front-matter block (Tomus Title
Page + scans + prose) **only when `vol.emit_front_matter`**; a restricted volume
emits none of it (and no `tomus_label` either), matching today's "restricted runs
carry no front matter" contract. The existing emit guard
`if vol.is_first_of_source && !vol.front_matter.is_empty()` (`assemble.rs:57`) is
**removed** and replaced by this flag plus the per-piece conditions below.

In `src/facsimile/assemble.rs::inner_document` (line 50):

1. After the front flyleaves, when `vol.emit_front_matter`, emit in order:
   - `tomus_title_page(vol.tomus, &lives)` — **every** emitting volume (replaces
     `tomus_label`); `lives` = the Latin titles of `vol.units` with
     `kind == LifeKind::Life`, in order;
   - `embed_scan(front_qr_name(vol.source))` — **every** emitting volume;
   - if `vol.is_first_of_source`: `embed_scan(front_volumen_name(vol.source))`;
   - if `vol.tomus == 1`: `embed_scan(front_colophon_name())`;
   - the prepared `vol.prose` string (empty unless this volume gets the Monitum or
     Praefatio — see the upstream rule below).
2. **Remove** the `latex::tomus_label(vol.tomus)` call (line 61) entirely (it is
   not emitted even for restricted volumes). Keep `\beginbodynumbering` immediately
   before the units.
3. In `fill_front_matter` (`src/facsimile/mod.rs:333`), which currently iterates
   **only** `is_first_of_source` volumes (`mod.rs:356`), change the loop to iterate
   **all** volumes (all have `emit_front_matter == true`, since `fill_front_matter`
   is skipped entirely for restricted runs), and for each:
   - render the QR scan it needs (`front_qr_name(vol.source)` from source page 1);
   - if `vol.is_first_of_source`, also render the Volumen-title scan
     (`front_volumen_name(vol.source)` from `front_volumen_page(vol.source)`);
   - if `vol.tomus == 1`, also render the colophon scan (`front_colophon_name()`
     from v1 p9);
   - set `vol.prose`: if `vol.tomus == 1`,
     `front_prose_pages(sidecar, &[(0,12),(0,13)])` (Monitum); else if
     `vol.is_first_of_source && vol.source == 1`,
     `front_prose_pages(sidecar, &PRAEFATIO_PAGES)` (Praefatio), where
     `PRAEFATIO_PAGES` is the `(source, page)` list Step 1.0 confirmed (expected
     `[(1,14),(1,15),(1,16),(1,17)]`); else leave it empty.
   (Rendering the same QR PNG once per tomus of a source simply overwrites an
   identical file — harmless; optionally render each named scan once.)
4. A build with no API credentials (or no front-matter sidecar) must still produce
   the Tomus Title Page + embedded scans — those need no network. Only the prose
   degrades gracefully to empty, matching the current warn-and-continue behavior
   (`mod.rs:349`). This means the scan rendering **must not sit behind the
   `run_front_matter` early return** (`mod.rs:352–354`): restructure
   `fill_front_matter` so it renders the scans for every volume first (needs only
   the `Pdf`s and `--out`), then attempts the prose transcription and, on success,
   fills `vol.prose`; a transcription failure warns and leaves `prose` empty but the
   scans are already on disk.

### Step 2.6 — Tests

In `src/facsimile/assemble.rs` tests and `src/latex.rs` tests:

- `tomus_title_page_lists_only_lives` — the generated page contains each Life's
  Latin title and the `MARTINEZ-DIRKS` line, and contains **no** Comparatio title.
- `tomus_title_page_has_single_tomus_line` — exactly one `TOMUS` occurrence; the
  old `tomus_label` is gone (`inner_document` contains no second `TOMUS` page).
- `facsimile_frontmatter_order` — in a first-of-source Tomus I document, the Tomus
  Title Page precedes the QR scan, which precedes the Volumen title scan, which
  precedes the colophon scan, which precedes the Monitum prose.
- `qr_scan_on_every_tomus` — a non-first-of-source emitting volume still embeds its
  source's QR scan and does **not** embed the Volumen title / colophon / prose.
- `restricted_volume_emits_no_front_matter` — a volume with
  `emit_front_matter == false` emits no Tomus Title Page, no `embed_scan`, no
  `tomus_label`, and no prose (so a `--unit`/`--pages` inner references no
  `front_*.png`).
- `volumen_title_only_first_of_source` / `colophon_only_tomus_one` — placement
  guards hold.
- `crop_box_removes_fraction` — the Rust crop helper on a synthetic image returns
  the expected sub-rectangle dimensions.
- `front_prose_pages_emits_only_listed` — with a synthetic sidecar whose source 0
  has `front_prose` records at p8, p10, p12, p13, `front_prose_pages(sc,
  &[(0,12),(0,13)])` emits only the p12/p13 prose and **neither** the p8 (half-title)
  nor p10 (Volumen title) text, and each emitted page carries `\markboth{}{}`.
- Keep `front_matter_block_unchanged_for_transcription` (`latex.rs:821`) green
  (the shared block-rendering refactor must not change its bytes).

### LLM verification
- `cargo test facsimile` and `cargo test latex` pass; `cargo build` clean.
- `grep -n "TOMUS" <generated tomus_1_inner.tex>` shows one title occurrence.

### Human verification
- Compile Tomus I and Tomus II inners; confirm the front-matter order and that the
  Tomus Title Page reads as specified (correct Æ/Œ, correct Life list for that
  tomus, `EX LIBRIS / MARTINEZ-DIRKS`).
- Confirm the QR, Volumen-title, and colophon scans are watermark-free and sized
  to the page; tune the crop constants if the owner sees watermark remnants or
  over-cropping.

---

## Phase 3 — Inline chapter header; drop the standalone title page
### Suggested implementer: Sonnet

### Step 3.1 — Replace the standalone title page with an inline header

In `src/facsimile/assemble.rs::unit_block` (line 78):

1. Open the unit with `\freshpage` and set `\markboth{LATIN}{LATIN}` as today.
2. Do **not** emit a standalone `\heading{…}\clearpage` title page.
3. On the unit's **first** body page only: suppress the running head
   (`\thispagestyle{empty}`) and emit the large centered chapter header with the
   **Latin title only** (no Greek) *above* the first image, e.g.
   `\thispagestyle{empty}\heading{LATIN}\noindent\includegraphics[width=\textwidth]{first}\par\clearpage`.
   The first body image may need to be composed slightly shorter so header + image
   fit the page (see Phase 4, which already caps canvas height — compose the first
   page's budget as `content_height` minus the header's vertical allowance).
4. Subsequent images are emitted as today (`\noindent\includegraphics…\par\clearpage`)
   and keep the Latin running head from `\markboth`.
5. Drop `greek_title` from `PlacedUnit` if nothing else uses it (verify no other
   reader), per the owner's "Latin titles are sufficient" instruction; otherwise
   leave the field and simply stop emitting it.

### Step 3.2 — Reduce the first-page compose budget for the header

> **Implementation order:** this step builds on the capped-budget compose loop
> introduced in **Phase 4**. Implement Phase 4 before Phase 3 (or land both
> `compose_pages` edits together), since the first-page reduction is only
> meaningful once `compose_pages` enforces a per-canvas budget. The reduction is a
> *correctness* requirement, not cosmetics: a full-height first canvas plus a
> `\heading` above it overflows the page exactly the way Phase 4 fixes.

In `src/facsimile/reflow.rs`, have `compose_pages` take a per-call `first_budget`
(applied to the **first emitted canvas** of the unit) equal to `content_height() -
header_allowance_px`, where `header_allowance_px` corresponds to the `\heading`
block height (≈ one `\large` line + `\bigskip`); all later canvases use the full
capped budget from Phase 4. The caller in `run_facsimile` passes the reduced
`first_budget` for every unit (since every unit now opens with an inline header).

**Units.** `header_allowance_px` must be in the **same pixel basis as
`content_height()`** — pixels at `TrimGeom::dpi` (`BODY_DPI`), i.e.
`px = cm / 2.54 × dpi` — because the composed canvas is included at
`width=\textwidth` with no height constraint, so its on-page height is its pixel
height scaled by `\textwidth / tb_w`. Reducing the first canvas's pixel budget by
`header_allowance_px` (same px scale) frees exactly that much vertical space on the
page for the header. Derive it from a header height in cm (one `\large` line +
`\bigskip`, perhaps ~1.2 cm) converted at `BODY_DPI`, and expose it as a named
constant. `header_allowance_px` is tuned in Phase 7.

### Step 3.3 — Tests

- `unit_block_has_no_standalone_title_page` — `unit_block` output contains no
  page whose only content is the heading; the heading appears on the same page as
  the first image.
- `unit_first_page_suppresses_running_head` — the first page carries
  `\thispagestyle{empty}`; later pages do not.
- `unit_header_is_latin_only` — the header contains the Latin title and no Greek.

### LLM verification
- `cargo test facsimile` passes; `cargo build` clean.

### Human verification
- Compile a tomus and confirm each Life/Comparatio opens with a large Latin header
  atop its first page of reproduced text, with no running head on that page and no
  separate title page (old inner p16 behavior gone).

---

## Phase 4 — Cap composed canvas height (overflow + blank pages)
### Suggested implementer: Opus

### Step 4.1 — Never emit a canvas taller than the page budget

In `src/facsimile/reflow.rs::compose_pages` (line 253): when a single scaled strip
`sh` exceeds `budget`, split it into `budget`-tall sub-strips before packing. Cut
preferentially at an internal minimal-ink row within the strip (reuse the
row-ink profile from `lines::line_bands`/`ink_profile_rows`, exposing a helper if
needed) so a split never bisects a glyph row; fall back to a hard cut at `budget`
when no low-ink row exists in range. The invariant: **every returned canvas has
height ≤ its budget.**

This step also introduces the per-canvas budget parameterization that Phase 3.2
depends on: `compose_pages` takes `first_budget` (the unit's first canvas) and
`budget` (all later canvases). When implemented before Phase 3, pass
`first_budget == budget` so behavior is unchanged until the header lands.

### Step 4.2 — Leave a visible bottom margin on full pages

Reduce the pack budget slightly below `tb_h` so a full page does not fill to the
exact text-block edge. Use a **bottom-only** slack — a new `BODY_BOTTOM_SLACK_CM ≈
0.3` subtracted from the *height budget* only — **not** `SAFETY_Y_CM`, which is
applied symmetrically (it also insets the top in `pack_canvas`/`content_height`,
`reflow.rs:85,97`) and would push type down from the top. This keeps the type size
uniform (width still maps to the full measure) while guaranteeing a consistent
lower margin.

### Step 4.3 — Investigate the giant-band source (belt-and-suspenders)

The 2868-px strip came from line-banding merging lines into one band (dark scan /
marginal line-number ticks pulled into the column crop). Tighten
`columns::detect_latin_column` or add a margin-trim so the far-left tick column is
excluded, reducing the incidence of giant bands. This is secondary to Step 4.1,
which already guarantees correctness regardless of band quality.

### Step 4.4 — Tests

- `compose_never_exceeds_budget` — with an input strip taller than `budget`, every
  returned canvas has height ≤ budget.
- `oversized_strip_split_at_low_ink` — a tall strip with an internal whitespace row
  is split at that row, not mid-glyph.
- `full_page_keeps_bottom_slack` — a fully packed canvas height ≤ budget with the
  bottom slack applied.
- Keep the existing `compose_*` tests green (adjust expected canvas counts only
  where the budget constant changed, documenting the new budget).

### LLM verification
- `cargo test reflow` / `cargo test facsimile` pass; `cargo build` clean.
- Regenerate Tomus I; `pdfinfo` page count drops (no bumped blank pages).

### Human verification
- Render former-problem inner pages (around old pp. 23, 25, 37, 81, 127): no page
  overflows its bottom margin, and no blank page interrupts a flowing sentence
  (the p22→p24 sentence is now contiguous).

---

## Phase 5 — Stop running-head / folio bleed into body
### Suggested implementer: Opus

### Step 5.1 — Strengthen interior-page furniture removal

The goal is to drop a top band that is a bled-through running head even when its
center sits just below the current 6.5 % zone, without eating a real first body
line. Two complementary measures, placed where each has the data it needs:

**(a) Image-free zone bump, in `lines.rs`.** In `drop_furniture`
(`lines.rs:72`) — which receives only `(bands, page_h)` — raise `TOP_FURNITURE_FRAC`
modestly (e.g. to ~0.08) if a real-eyes check confirms it does not clip the first
body line on normal pages. This is the only change that belongs in `drop_furniture`,
because the ink-span test below needs the column image, which `drop_furniture` does
not have.

**(b) Ink-span interior-head drop, in `reflow.rs`.** The "short detached band is a
running head" test needs `band_ink_span(col, band)` (`reflow.rs:211`, private to
`reflow`) and the `BODY_LINE_SPAN_FRAC` measure, so it operates on the column image
and therefore lives in `reflow`, **not** `lines.rs`. Add a helper
`drop_interior_running_head(col: &GrayImage, bands: &[Band]) -> Vec<Band>` that
drops a **leading** band which is both (i) **short** — ink span well under
`BODY_LINE_SPAN_FRAC × col.width()` — and (ii) separated from the next band by an
above-median gap, and likewise a **trailing** numeric-only/short band at the foot (a
bare folio). Call it in `render_and_band` (`reflow.rs:178`) **after**
`drop_furniture` returns, so it sees the already-zone-filtered bands.

Keep the change conservative: it must not eat the first real body line. Gate the
interior-head drop on **both** a short ink span **and** an above-median detaching
gap (a running head is detached from the body; a legitimately short first line or an
original short sub-head is not — short alone is not enough).

**Interaction with the first-page drop.** `drop_furniture` (and now
`drop_interior_running_head`) run on **every** page in `render_and_band`, while the
unit's *first* page additionally has its leading title / cross-reference bands
removed later by `drop_opening_bands` (`reflow.rs:194`), applied in
`unit_body_strips` (`reflow.rs:155`) to the first page only. Because
`drop_interior_running_head` runs inside `render_and_band`, it executes before
`drop_opening_bands` on every page including the first. Verify the two compose
cleanly on a first page: the interior-head rule should target the detached
running-head bleed, and `drop_opening_bands` the (attached, full-ish-measure) title
bands, so they do not fight over the same leading band. If a first-page title band
could match the short-and-detached test, prefer running `drop_interior_running_head`
only on interior pages (idx > 0) and leaving the first page's leading furniture to
`drop_opening_bands`.

### Step 5.2 — Tests

- `drop_interior_running_head_removes_low_head` (in `reflow` tests) — a short band
  whose span is under the body measure, above an above-median gap, is dropped
  (models old inner p515 / tomus 7 p18).
- `drop_interior_running_head_keeps_full_measure_first_line` (in `reflow` tests) — a
  full-measure first body line at the same height is **kept** (no false positive).
- If `TOP_FURNITURE_FRAC` is bumped, add/adjust a `drop_furniture` test pinning that
  a normal first body line just below the old zone survives the new zone.
- Keep existing `drop_furniture_*` tests green.

### LLM verification
- `cargo test lines` / `cargo test facsimile` pass; `cargo build` clean.

### Human verification
- Render old inner p515 (Tomus I) and old inner p18 (Tomus 7): the stray
  `…COMPARATIO. 227` / source-folio fragment is gone and the body text starts
  clean.

---

## Phase 6 — Balanced volume splitting
### Suggested implementer: Opus

### Step 6.1 — Close each volume at the boundary nearest the target

In `src/volumes.rs::split_source` (line 187): before adding the next group, if the
current volume is non-empty, compare the pre-add deviation `|sum − target|` with
the post-add deviation `|sum + pages − target|`; if adding the group would land
**farther** from the target than closing now (and the current volume is not empty),
close the current volume first. Retain the hard `MAX` guard (never let a volume
exceed `max`). This replaces the floor-to-exceed `sum >= target` close so a volume
sitting just under target does not grab a whole oversized group.

### Step 6.2 — Reflect the new per-tomus overhead

The front-matter overhead is no longer "8 pages on the first volume of a source
only": every tomus now carries a Tomus Title Page + a QR page, the first tomus of
each volumen adds a Volumen title page, and Tomus I adds a colophon + Monitum (and
v2's first tomus the Praefatio). Because the overhead depends on both the source and
whether the volume is the first of its source, the two functions that compute it
need enough information to tell those cases apart — today neither does.

**New overhead constants** (replace `FRONT_MATTER_PAGES`; `\freshpage` may round each
up to a recto, so these are deliberately generous estimates, tuned in Phase 7):

- `FRONT_PER_TOMUS_PAGES = 2` — Tomus Title Page + QR page, on **every** volume.
- `VOLUMEN_TITLE_PAGES = 1` — engraved Volumen title, on the **first tomus of each
  source**.
- `COLOPHON_PAGES = 1` and `MONITUM_PAGES = 2` — on **Tomus I only** (source 0,
  first volume).
- `PRAEFATIO_PAGES_EST = 4` — on **v2's first tomus** (source 1, first volume);
  adjust to the count Step 1.0 confirmed.

**Shared helper.** Add `fn front_overhead(source: usize, is_first_of_source: bool,
is_tomus_one: bool) -> u32` returning `FRONT_PER_TOMUS_PAGES` plus the conditional
pieces (`VOLUMEN_TITLE_PAGES` when `is_first_of_source`; `COLOPHON_PAGES +
MONITUM_PAGES` when `is_tomus_one`; `PRAEFATIO_PAGES_EST` when `is_first_of_source &&
source == 1`). Use it from both:

- `volume_physical_pages` (`volumes.rs:85`): change its signature to
  `volume_physical_pages(content_pages, source, is_first_of_source, is_tomus_one)`
  and replace the `FRONT_MATTER_PAGES`-vs-0 branch with `front_overhead(…)`.
- `split_source` (`volumes.rs:187`): it is called per source (`volumes.rs:114`), so
  pass the `src` index in, and inside it treat the **first** volume it closes for
  `source == 0` as `is_tomus_one`. Seed and reset `overhead` from `front_overhead`
  (the first volume uses `is_first_of_source = true`, later volumes `false`) instead
  of the `FRONT_MATTER_PAGES`/`BLANK_LEAF_PAGES` literals.

At the `split_source` call site (`split_into_volumes`, `volumes.rs:114`), pass
`src`; at the `volume_physical_pages` call site (`write_wrappers_from_estimate`,
below) pass `v.source`, `v.is_first_of_source`, and `v.tomus == 1`.

Also update `write_wrappers_from_estimate` (`mod.rs:388`) to stop adding `+1` per
unit (the standalone title page removed in Phase 3) — count `image_names.len()` per
unit — and to call the new `volume_physical_pages` signature. Keep everything padded
to the 32-page signature via `pad_to_multiple_of_32`.

### Step 6.3 — Tests

- `split_closes_at_nearest_boundary` — a source whose groups are 98/96/80/101/98
  (the measured v1 figures) with target 400, max 500 splits so group 5 opens the
  second volume (the first volume ends at group 4).
- `split_respects_max_when_nearest_would_exceed` — if the nearest-boundary choice
  would exceed `max`, the volume closes earlier.
- Keep `split_never_breaks_a_pair_group`, `each_volume_within_max_pages`,
  `volumes_are_roughly_balanced`, `split_forces_break_at_v1_v2_boundary`, and the
  overhead test green (update `volume_physical_pages` expectations for the new
  overhead constant, documenting the new numbers).

### LLM verification
- `cargo test volumes` passes; `cargo build` clean.

### Human verification
- Regenerate the full build; confirm v1 Tomus I ends with Themistocles/Camillus
  (~416 pp) and Pericles opens Tomus II, and that every tomus is ≤ 500 pp and near
  400.

---

## Phase 7 — Regenerate, verify, and tune
### Suggested implementer: Opus

### Step 7.1 — Full regeneration and eyeball

Run the full facsimile build (`cargo run --bin facsimile …` with the real sources),
let it compile the inners, and record each tomus's page count. Confirm against the
target/cap and the expected boundaries.

### Step 7.2 — Tune crop boxes and header allowance

From the compiled PDFs, adjust the Phase 2 crop constants and the Phase 3 header
allowance if the owner's real-eyes check flags watermark remnants, over-cropping,
or a cramped first page.

### Step 7.3 — Final green gate

`cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --all
--check` all pass.

### LLM verification
- Full `cargo test` green; clippy and fmt clean.
- `pdfinfo` on each regenerated `tomus_*_inner.pdf` shows balanced counts within
  the cap.

### Human verification
- End-to-end read of Tomus I front matter and the first few Lives: correct
  front-matter order and content, no overflow/blank/furniture bugs, inline Latin
  headers, and sensible volume boundaries.

---

## Phase 8 — Build-directory cleanup
### Suggested implementer: Sonnet

The build directory accumulates ~3 GB of composed strip PNGs per run under
`args.out`, which also churns OneDrive. Two cleanups keep it tidy, scoped so the
expensive raster cache and any manual-compile workflow are preserved.

### Step 8.1 — Clean generated output at the start of a run

Early in `run_facsimile` (`src/facsimile/mod.rs:75`), after `args.out` is ensured
to exist and **before** composing, delete the previous run's *generated* artifacts
in `args.out`: files matching `facsimile_*.png`, `front_*.png`,
`tomus_*_inner.tex`, `tomus_*_inner.pdf`, and `tomus_*.{tex,pdf}` (the imposition
wrappers). This is a correctness measure: after the Phase 6 boundary shift or any
unit-table change, a removed unit's orphan PNGs and a now-nonexistent
`tomus_N.pdf` must not linger. Skip this for restricted (`--pages` / `--unit`)
runs, or scope it to the units being rebuilt, so a single-unit test run does not
wipe a full build.

**Do not** delete `args.out/render_cache/`: it is the reusable poppler
rasterization cache (`render_gray`, `mod.rs:265`) and purging it forces a full
re-rasterize. If an explicit cache purge is ever wanted, add it behind a new
`--clean-cache` flag on `FacsimileArgs`, defaulting off.

### Step 8.2 — Remove composed PNGs after a successful compile

In `compile_and_impose` (`src/facsimile/mod.rs:365`), after the inner PDFs have
compiled and the imposition wrappers are written and compiled, delete the composed
body PNGs (`facsimile_*.png`) and front-scan PNGs (`front_*.png`) — they are now
embedded in the inner PDFs and nothing downstream reads them (the wrappers
`\includepdf` the inner PDFs). This runs **only** on the compile branch
(`!args.no_compile && lualatex_available()`); the `write_wrappers_from_estimate`
branch (`mod.rs:388`) must leave every PNG in place so the printed manual-compile
instructions still work. Keep `render_cache/` and the `.tex`/`.pdf` deliverables.

### Step 8.3 — Tests

- `start_cleanup_removes_only_generated` — a temp `out` dir seeded with
  `facsimile_x_1.png`, `front_qr_s0.png`, `tomus_1.pdf`, a `render_cache/` file,
  and an unrelated `notes.txt` has the generated files removed and `render_cache/`
  + `notes.txt` preserved.
- `start_cleanup_skipped_for_restricted` — a `--unit`/`--pages` run does not wipe
  sibling units' PNGs.
- `post_compile_cleanup_removes_pngs` (gated on poppler/lualatex like
  `run_facsimile_smoke`, else skipped) — after a compiling run, no
  `facsimile_*.png`/`front_*.png` remain but the inner/wrapper PDFs do.

### LLM verification
- `cargo test facsimile` passes; `cargo build` clean.

### Human verification
- Run a full build and confirm `build_facsimile` holds the `.tex`/`.pdf`
  deliverables and `render_cache/` but no composed PNGs; run a `--no-compile`
  build and confirm the PNGs remain for a manual compile.

---

## Notes on scope and invariants

- The transcription (non-facsimile) build path is left behaviorally unchanged; its
  golden front-matter test pins `front_matter_block`'s bytes, so new front-matter
  builders are additive and wired only into the facsimile assemble.
- All edits to `plutarch.frontmatter.tags.json` are deliberate, reviewable cache
  edits (the sidecar is the single source of truth and is hand-editable), per the
  faithfulness invariant in `CLAUDE.md`.
- `facsimile_units.json` is not edited by this plan; the volume boundary shift in
  Phase 6 is an algorithm change, not a hand pin.
