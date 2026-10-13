# Front-matter recto placement plan

## Context

In the facsimile inner documents, the re-typeset front-matter units land on
versos (left-hand, even PDF pages) instead of rectos (right-hand, odd PDF
pages). This affects every emitting tomus, not just Tomus I:

- Tomus I (first tomus of source 0 / v1) places the Tomus title page on PDF page
  6, the Google-books/QR scan on page 8, the Volumen title page on page 10, the
  printer's colophon on page 12, and the Monitum prose on page 14 — every named
  front-matter unit on an even page. Theseus (the first body unit) opens
  correctly on a recto (page 17).
- Tomus 5 (first tomus of source 1 / v2) places its Tomus title page, QR scan,
  Volumen title page, and Praefatio prose on versos. Nicias (the first body
  unit) opens correctly on a recto (page 19).
- Every other emitting tomus carries a Tomus title page and a QR scan, both of
  which also land on versos.

A scholarly reader expects each of these display units to open on a recto, the
way the body Lives already do. The fix must keep the existing flyleaf structure
(two blank leaves = four PDF pages at each end) and must not disturb the separate
transcription build path, which is held byte-for-byte by golden tests.

The requirement is that each front-matter unit begin on an odd PDF page, which —
because flyleaves are whole leaves (two PDF pages each) — is also a true recto
(right-hand page) when the book is bound. "Odd as a PDF page" and "a recto when
folded" are the same condition here, so forcing an odd PDF page satisfies both.

## Current state of the world

- The facsimile inner document is assembled by
  `src/facsimile/assemble.rs::inner_document`. After the preamble it pushes
  `RECTO_START_DEFINITION` (which defines the LaTeX macro `\unitrecto`), opens
  the document, emits the front flyleaves via `latex::blank_pages(2 *
  latex::FLYLEAVES_PER_END)`, then — when `vol.emit_front_matter` is true —
  emits the front-matter units in this order:
  1. `latex::tomus_title_page(vol.tomus, &lives, exlibris)` — the Tomus title page.
  2. `latex::embed_scan(frontmatter::front_qr_name(vol.source))` — the QR / Google-books scan (every emitting tomus).
  3. `latex::embed_scan(frontmatter::front_volumen_name(vol.source))` — the engraved Volumen title (first tomus of a source only).
  4. `latex::embed_scan(frontmatter::front_colophon_name())` — the printer's colophon (Tomus I only).
  5. `vol.prose` — the re-typeset Monitum (Tomus I) or hybrid Praefatio (v2's first tomus) prose, prepared upstream in `src/facsimile/mod.rs` via `latex::front_prose_pages_until` / `latex::front_praefatio`.
  6. `latex::embed_scan_pair(front_monitum_a_name(), front_monitum_b_name())` — the Monitum corrigenda/errata image pair (Tomus I only).

  It then emits `\unitrecto` before `\beginbodynumbering` and each body unit's
  `\unitrecto`-led block.

- Two LaTeX page-break macros govern parity, both defined in
  `src/latex.rs::inner_preamble` and `src/facsimile/assemble.rs`:
  - `\freshpage` (defined in `inner_preamble`, `src/latex.rs:203`):
    `\clearpage\ifodd\value{page}\thispagestyle{empty}\hbox{}\clearpage\fi`.
    After the `\clearpage`, **if the current page is odd it inserts a blank page
    and clears again**, so the following content lands on an **even** page (a
    verso). This is verso-forcing.
  - `\unitrecto` (defined in `src/facsimile/assemble.rs:19`,
    `RECTO_START_DEFINITION`):
    `\clearpage\ifodd\value{page}\else\thispagestyle{empty}\hbox{}\clearpage\fi`.
    After the `\clearpage`, **if the current page is even it inserts a blank page
    and clears again**, so the following content lands on an **odd** page (a
    recto). This is recto-forcing. It is self-correcting: regardless of how many
    pages the previous unit occupied, the next content starts on a recto.

- Every facsimile front-matter emitter currently opens with `\freshpage`
  (verso-forcing):
  - `tomus_title_page` — `src/latex.rs:490`.
  - `embed_scan` — `src/latex.rs:516`.
  - the prose path — `front_prose_page_blocks` (`src/latex.rs:417`) begins with
    `"% Re-typeset front-matter prose\n\\freshpage\n"`.
  - `embed_scan_pair` (`src/latex.rs:527`) is the exception: it opens with a
    plain `\clearpage`, not `\freshpage`, so the errata pair simply follows the
    prose on the next page.

- `front_prose_page_blocks(blocks, reset_running_head, flexible_spacing)` is
  shared between build paths. The facsimile callers reach it through
  `front_prose_pages` (`src/latex.rs:308`, used by `front_praefatio`) and
  `front_prose_pages_until` (`src/latex.rs:385`, used for the Monitum via
  `src/facsimile/mod.rs:485`). The transcription caller reaches it through
  `front_prose_page` (`src/latex.rs:379`, `reset_running_head: false`). In
  practice each facsimile prose section resolves to a single
  `front_prose_page_blocks` call, hence a single `\freshpage`:
  `MONITUM_PROSE_PAGES` is one page `[(0, 12)]` and stops at
  `MONITUM_PROSE_SENTINEL`; `front_prose_pages` merges its pages into one block
  list before the single call. The test
  `praefatio_hybrid_has_no_internal_parity_breaks` (`src/latex.rs:970`) pins
  that "one parity break per section" contract.

- The transcription inner (`src/latex.rs::inner_document`, the `Sidecar`/
  `VolumePlan` overload) uses its own front-matter emitters — `title_page`
  (`src/latex.rs:230`), `front_images` (`src/latex.rs:266`), `front_prose`
  (`src/latex.rs:289`), and `tomus_label` (`src/latex.rs:553`). These also use
  `\freshpage` and are **out of scope**: the transcription path is held
  byte-for-byte by golden tests (see the comments at `src/latex.rs:48`,
  `src/latex.rs:166`, `src/latex.rs:378`) and must not change.

- Because every front-matter emitter uses the same `\freshpage`, the verso
  problem is identical across tomi; the units a given tomus carries differ by its
  place in the work (every tomus: title + QR; first tomus of a source: + Volumen
  title; Tomus I: + colophon + Monitum; v2's first tomus / Tomus 5: + Praefatio).

- Trace of the current (verso) Tomus I layout, flyleaves = 4 pages (page counter
  at 5 after them):
  - title `\freshpage`: counter 5 (odd) → pad blank 5 → title on **6**; counter 7.
  - QR: 7 odd → pad 7 → QR on **8**; counter 9.
  - Volumen: 9 odd → pad 9 → Volumen on **10**; counter 11.
  - colophon: 11 odd → pad 11 → colophon on **12**; counter 13.
  - Monitum prose: 13 odd → pad 13 → Monitum on **14**; counter 15.
  - errata pair (plain `\clearpage`): errata on **15**; counter 16.
  - body `\unitrecto`: 16 even → pad 16 → Theseus on **17**.

- Trace of the current (verso) Tomus 5 layout (first tomus of source 1), flyleaves
  = 4 pages (counter at 5):
  - title `\freshpage`: 5 odd → pad 5 → title on **6**; counter 7.
  - QR: 7 odd → pad 7 → QR on **8**; counter 9.
  - Volumen: 9 odd → pad 9 → Volumen on **10**; counter 11.
  - Praefatio prose (`front_praefatio`): its opening `front_prose_pages` break is
    `\freshpage`, so the Praefatio opens on verso **12**, then flows across its
    opening prose, two corrigenda image pages, and its closing prose (each image
    and the closing use plain `\clearpage`); the body `\unitrecto` then lands
    Nicias on recto **19**.

- Trace of the current (verso) layout for a plain later tomus (e.g. Tomus 2:
  title + QR only), flyleaves = 4 pages (counter at 5): title on verso **6**, QR
  on verso **8**, first body `\unitrecto` on recto **9**.

- `src/volumes.rs::front_overhead` (`src/volumes.rs:45`) estimates front-matter
  page overhead for imposition planning and is deliberately generous, noting
  `\freshpage` "may round each up to a recto." Imposition uses actual compiled
  counts, so this estimate does not need to change; its per-item rounding is the
  same under recto-forcing.

## Desired end state

- Every named facsimile front-matter unit opens on an odd PDF page (a recto):
  the Tomus title page, the QR / Google-books scan, the Volumen title page, the
  colophon, and the Monitum/Praefatio prose. This holds for every emitting tomus:
  Tomus I (title, QR, Volumen, colophon, Monitum), Tomus 5 (title, QR, Volumen,
  Praefatio), and every plain later tomus (title, QR). The flyleaf count is
  unchanged (two leaves / four PDF pages each end).

- The Monitum corrigenda/errata image pair (`embed_scan_pair`) continues to
  follow the Monitum prose immediately, on the next page (a verso), as a
  continuation of the Monitum unit — it is **not** forced to its own recto.

- The transcription build path is unchanged, byte-for-byte: its title page,
  front images, front prose, and `TOMUS N` label still use `\freshpage` and
  render identically.

- Trace of the new (recto) Tomus I layout, flyleaves = 4 pages:
  - title `\unitrecto`: counter 5 (odd) → title on **5 (recto)**; counter 6.
  - QR: 6 even → pad 6 → QR on **7 (recto)**; counter 8.
  - Volumen: 8 even → pad 8 → Volumen on **9 (recto)**; counter 10.
  - colophon: 10 even → pad 10 → colophon on **11 (recto)**; counter 12.
  - Monitum prose: 12 even → pad 12 → Monitum on **13 (recto)**; counter 14.
  - errata pair (plain `\clearpage`): errata on **14 (verso)**; counter 15.
  - body `\unitrecto`: 15 odd → Theseus on **15 (recto)**.

  Theseus moves from PDF page 17 to PDF page 15; it remains a recto. The
  front-matter blanks that were on pages 5, 7, 9, 11, 13 are gone, replaced by
  blanks on 6, 8, 10, 12.

- Trace of the new (recto) Tomus 5 layout, flyleaves = 4 pages (counter at 5):
  - title `\unitrecto`: counter 5 (odd) → title on **5 (recto)**; counter 6.
  - QR: 6 even → pad 6 → QR on **7 (recto)**; counter 8.
  - Volumen: 8 even → pad 8 → Volumen on **9 (recto)**; counter 10.
  - Praefatio prose: 10 even → pad 10 → Praefatio opens on **11 (recto)**, then
    flows; the body `\unitrecto` then lands Nicias on a recto. Nicias remains a
    recto and shifts earlier (it sheds leading verso blanks); its exact PDF page
    is confirmed from the compiled sample in step 1.3.

- Trace of the new (recto) plain-later-tomus layout (title + QR), flyleaves = 4
  pages (counter at 5): title on recto **5**, QR on recto **7**, first body
  `\unitrecto` on recto **9** (the body page is unchanged from today; only the
  title and QR move from versos 6/8 to rectos 5/7).

- The single recto-forcing idiom, `\unitrecto`, is reused for the front matter;
  no second recto macro is introduced, no `\freshpage` logic is duplicated, and
  the body path is unchanged.

- Focused tests, the full `cargo test`, `cargo fmt --all --check`, and
  `cargo clippy --all-targets -- -D warnings` pass. A compiled Tomus I sample is
  inspected where LuaLaTeX and Poppler are available.

## Tech Learnings

LaTeX's `\value{page}` holds the number that will print on the page currently
being built. The classic recto idiom (`\cleardoublepage` in `openright`
documents) does `\clearpage` then, **if the current page is even**, pads with a
blank and clears again — landing content on an odd recto. `\unitrecto` is
exactly that idiom. `\freshpage` inverts the test (`\ifodd` with no `\else`), so
it pads when the page is odd and lands content on an even verso. Swapping which
branch carries the pad is the entire parity change; it is self-correcting, so a
unit always opens on the intended side regardless of the previous unit's length.

Because the front flyleaves are whole leaves (an even number of PDF pages),
PDF-odd and bound-recto coincide: forcing an odd PDF page is sufficient to put a
unit on a right-hand page when the signature is folded. Changing parity never
reflows Latin or changes the trim page; it only inserts or removes blank padding
pages, which the imposition's actual-compiled-count step absorbs.

## Phase 1 — Open each facsimile front-matter unit on a recto (Suggested implementer: Opus)

### 1.1 Recto-force the title page and embedded scans

In `src/latex.rs`, change the opening page break of the two facsimile-only
emitters from verso-forcing to recto-forcing:

- `tomus_title_page` (`src/latex.rs:447`): replace the `\freshpage` line in its
  emitted block (`src/latex.rs:490`) with `\unitrecto`.
- `embed_scan` (`src/latex.rs:513`): replace the `\freshpage` line in its emitted
  block (`src/latex.rs:516`) with `\unitrecto`.

Leave `embed_scan_pair` unchanged (it keeps its plain `\clearpage`, so the
errata pair follows the Monitum prose on the next verso). Leave `tomus_label`,
`title_page`, `front_images`, and the transcription `front_prose` unchanged —
they belong to the transcription path and keep `\freshpage`.

These two functions are `pub(crate)` and called only from the facsimile
assembler (`src/facsimile/assemble.rs::inner_document`), which defines
`\unitrecto` via `RECTO_START_DEFINITION` before any front-matter emitter runs,
so the macro is always in scope when the emitted `\unitrecto` compiles. Do not
move or redefine `\unitrecto`; reuse the existing definition. Do not add a new
macro or a second copy of the recto logic.

No existing unit test pins `\freshpage` for these two emitters (the current
`tomus_title_page` tests at `src/latex.rs:900`–`src/latex.rs:950`,
`src/latex.rs:1314`, `src/latex.rs:1346` and the `embed_scan` test
`embed_scan_includes_named_png_scaled` at `src/latex.rs:1392` assert content and
`\thispagestyle{empty}`, not the break macro), so there is nothing to rewrite
there — the work is to **add** a direct assertion that `tomus_title_page(...)`
and `embed_scan(...)` each contain `\unitrecto` and do **not** contain
`\freshpage`. Keep `embed_scan_pair_preserves_order_and_single_page`
(`src/latex.rs:1360`) unchanged — it already asserts the pair has no `\freshpage`
and two `\clearpage`s, which still holds.

**LLM verification:** Run `cargo fmt --all`; `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`;
`cargo test tomus_title_page`; `cargo test embed_scan`; `cargo build`; and
`git diff --check`. Confirm the filtered tests execute and pass. Grep the two
functions' output in a scratch harness or read the source to confirm each now
emits `\unitrecto` and neither emits `\freshpage`.

**Human verification:** Read the changed `tomus_title_page` and `embed_scan`
source blocks and confirm the only change is `\freshpage` → `\unitrecto`, with
`embed_scan_pair` untouched.

### 1.2 Recto-force the facsimile prose section

In `src/latex.rs`, add an explicit recto selector to the shared prose renderer so
the facsimile Monitum/Praefatio prose opens on a recto while the transcription
prose keeps `\freshpage` byte-for-byte.

- Add a `recto: bool` parameter to `front_prose_page_blocks` (`src/latex.rs:412`).
  When `recto` is true, emit `\unitrecto` as the opening break; when false, emit
  `\freshpage` exactly as today. Keep the leading comment line
  `% Re-typeset front-matter prose` in both cases.
- Thread `recto` through the callers:
  - `front_prose_page` (`src/latex.rs:379`, transcription) passes `recto: false`.
  - `front_prose_pages` (`src/latex.rs:308`, facsimile Praefatio) passes
    `recto: true`.
  - `front_prose_pages_until` (`src/latex.rs:385`, facsimile Monitum) passes
    `recto: true` on both its emit sites (`src/latex.rs:403` and
    `src/latex.rs:406`).

In practice each facsimile prose section is a single `front_prose_page_blocks`
call (one `\unitrecto`): `MONITUM_PROSE_PAGES` is one page and
`front_prose_pages` merges its pages before the single call. The only path that
emits more than one break is `front_prose_pages_until` when the sentinel is
absent (the warning fallback), which would then recto-force each page — the same
per-page parity behavior it has today with `\freshpage`, and acceptable for that
degraded case.

Update the facsimile prose tests that count `\freshpage`:
- `praefatio_hybrid_has_no_internal_parity_breaks` (`src/latex.rs:970`): change
  the `\freshpage` count-of-1 to a `\unitrecto` count-of-1, and keep the
  assertion that `\freshpage` is absent (move `\unitrecto` out of the
  "must-not-contain" token list and into the single-break assertion). The
  `\clearpage` count stays 4.
- `front_prose_pages_until` stops-at-sentinel test (`src/latex.rs:1428` region):
  change the `\freshpage` count-of-1 to `\unitrecto`.
- `front_prose_pages_until_emits_all_when_sentinel_absent` (`src/latex.rs:1434`):
  change the `\freshpage` count-of-2 to `\unitrecto` count-of-2.
- `front_prose_pages_flows_as_one_section` (`src/latex.rs:1475` region): change
  the `\freshpage` count-of-1 to `\unitrecto`.
- Confirm any transcription-side prose test still expects `\freshpage` and keep
  it unchanged, proving the transcription bytes did not move.

**LLM verification:** Run `cargo fmt --all`; `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`; `cargo test front_prose`;
`cargo test praefatio_hybrid_has_no_internal_parity_breaks`; `cargo build`; and
`git diff --check`. Confirm the filtered tests execute and pass, and that no
transcription golden test changed (run the full `cargo test` and confirm green).

**Human verification:** Read the updated `front_prose_page_blocks` and confirm
the facsimile callers select `\unitrecto` while `front_prose_page` (transcription)
still selects `\freshpage`.

### 1.3 Pin the recto layout with an assembler-level test and verify the compile

In `src/facsimile/assemble.rs`, add a non-compiling source assertion that the
emitted facsimile inner opens every named front-matter unit with `\unitrecto`,
for each tomus shape. Add `front_matter_units_open_on_recto` using the existing
`volume_full` / `unit` test helpers, covering three volumes:

- **Tomus I** (`volume_full(1, 0, true, true, "% MONITUM PROSE MARKER\n",
  vec![unit("theseus", &["x.png"])])`): assert that between the end of the front
  flyleaves and `\beginbodynumbering` the document contains, in order,
  `\unitrecto` immediately preceding each of the title block (`PLVTARCHI`), the QR
  scan (`front_qr_s0.png`), the Volumen scan (`front_volumen_s0.png`), the
  colophon (`front_colophon.png`), and the prose marker; and that the errata pair
  (`% Embedded front-matter scan pair`) is introduced by a plain `\clearpage`,
  not `\unitrecto`.
- **Tomus 5 shape** (`volume_full(5, 1, true, true, "% PRAEFATIO PROSE MARKER\n",
  vec![unit("nicias", &["x.png"])])`): assert `\unitrecto` immediately precedes
  the title block (`PLVTARCHI`), the QR scan (`front_qr_s1.png`), and the Volumen
  scan (`front_volumen_s1.png`); assert there is no colophon and no errata pair.
  The Praefatio prose's own recto break lives inside `vol.prose` and is covered
  by the step 1.2 prose tests, so the marker string here only needs to appear
  after the Volumen scan.
- **Plain later tomus** (`volume_full(2, 0, false, true, "", vec![unit("theseus",
  &["x.png"])])`): assert `\unitrecto` immediately precedes the title block and
  the QR scan (`front_qr_s0.png`), and that there is no Volumen scan, colophon,
  or prose.

For every volume, assert the front-matter region (flyleaves end to
`\beginbodynumbering`) contains no `\freshpage`. Reuse the ordering style of the
existing `facsimile_frontmatter_order` (`src/facsimile/assemble.rs:1031`) test
rather than duplicating its setup.

Two ignored compile tests build volumes with front matter and have hardcoded
page expectations that must be checked:

- `first_body_compile_aligns_before_numbering` (`src/facsimile/assemble.rs:678`)
  hardcodes `let prose_start = if tomus == 1 { 13 } else { 9 };`
  (`src/facsimile/assemble.rs:708`). This test passes its front-matter prose as a
  **literal** string (`FRONT-A`/`FRONT-B`), so that prose does not pass through
  `front_prose_page_blocks` and gains no `\unitrecto`; only the title page, QR,
  Volumen, and colophon move from versos to rectos, pulling the literal prose
  earlier by one page. Update the constant to `if tomus == 1 { 12 } else { 8 }`:
  Tomus I is title(5)+QR(7)+Volumen(9)+colophon(11), so the literal prose lands
  on page 12; Tomus II is title(5)+QR(7) only, so it lands on page 8. Everything
  downstream (`final_prose`, `final_front`, `body`, the `body % 2 == 1` recto
  check, and the boundary-blank branch) is computed from `prose_start` and needs
  no further change. Re-run the test and confirm it passes with the new constant.

- `facsimile_units_compile_on_rectos_with_single_boundary_blanks`
  (`src/facsimile/assemble.rs:622`) builds a Tomus II volume with
  `emit_front_matter = false`, so it has no named front-matter units; its expected
  image pages (`5, 6, 7, 8, 9, 11, 12`) are unaffected. Confirm by reading it and
  leave it unchanged.

`exlibris_compile_preserves_title_page_and_body_starts`
(`src/facsimile/assemble.rs:335`) needs no page-number edit: it detects the
title-page index and the first-body page **dynamically** (by searching the
extracted text / image list) and asserts only the recto invariant
(`body_page % 2 == 1`) plus cross-variant consistency of a baseline it captures
at runtime. Recto-forcing shifts the absolute pages but preserves both the recto
parity and the identical structure across owner variants, so the test should pass
unchanged. Re-run it to confirm; if it fails, treat that as a signal that
something broke the recto invariant or cross-variant consistency rather than as a
cue to hardcode new page numbers.

**LLM verification:** Run `cargo fmt --all`; `cargo fmt --all --check`;
`cargo clippy --all-targets -- -D warnings`;
`cargo test front_matter_units_open_on_recto`;
`cargo test facsimile_frontmatter_order`; `cargo test`; `cargo build`; and
`git diff --check`. Require the new/updated unit tests to execute and pass and
the full suite to stay green. With LuaLaTeX and Poppler available, run
`cargo test exlibris_compile_preserves_title_page_and_body_starts -- --ignored
--nocapture` (which loops over Tomus I and Tomus II, covering the title-page
recto on both),
`cargo test facsimile_units_compile_on_rectos_with_single_boundary_blanks --
--ignored --nocapture`,
`cargo test first_body_compile_aligns_before_numbering -- --ignored --nocapture`
(whose hardcoded `prose_start` was updated above), and the Tomus 5 Praefatio
compile
`cargo test praefatio_hybrid_compile_has_consecutive_content_pages -- --ignored
--nocapture` and `cargo test praefatio_hybrid_compile_fits_maximum_image_budget
-- --ignored --nocapture`; require them to pass. The hybrid-compile tests assert
relative page offsets and a recto body page (`body % 2 == 1`), which still hold
under recto-forcing; if either pins an absolute page that shifted, read the new
value from the failure and update it to the recto layout without weakening the
recto invariant.

**Human verification:** Where LuaLaTeX and Poppler are available, build the real
facsimile and open Tomus I, Tomus 5, and one plain later tomus (e.g. Tomus 2).
Confirm:
- Tomus I: the Tomus title page, QR scan, Volumen title, colophon, and Monitum
  prose each sit on an odd PDF page / right-hand recto; the Monitum errata image
  pair sits on the immediately following verso; Theseus opens on a recto (now PDF
  page 15).
- Tomus 5: the Tomus title page, QR scan, Volumen title, and Praefatio opening
  each sit on a recto; Nicias opens on a recto (shifted earlier from PDF page 19).
- Plain later tomus: the Tomus title page and QR scan sit on rectos; the first
  Life opens on a recto.

In all three, confirm the four-page flyleaf structure at each end is intact (no
leaf added or lost), no Latin reflowed, and the trim page is unchanged.

## Notes for the implementer

- Do not touch `src/main.rs`, `src/pipeline.rs`, or the transcription
  `inner_document`/`title_page`/`front_images`/`front_prose`/`tomus_label`; the
  transcription path must stay byte-for-byte identical.
- Do not introduce a second recto macro or inline a second copy of the
  `\clearpage\ifodd…` logic; reuse `\unitrecto`.
- `src/volumes.rs::front_overhead` does not need editing; imposition uses actual
  compiled page counts and the per-item parity rounding is unchanged. If its doc
  comment's "`\freshpage` may round each up to a recto" wording is updated for
  accuracy, keep it describing present behavior without historical narration.
- Leave all existing `tmp/` files alone.
