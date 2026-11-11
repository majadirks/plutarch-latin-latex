# Handoff Note: Collation (Back-Mark) Spine Stripes

A portable summary of the decisions and impacted files from implementing
collation marks in this project, written so an LLM can reproduce the feature in a
similar imposition/binding project. The full step-by-step plan is in
`2026_11_11_06_22_COLLATION_SPINE_MARKS_PLAN.md`.

## What was built
A binder's **collation mark**: one thick stripe straddling each signature's spine
fold, stepped down a bit per gathered unit so that when signatures are folded and
gathered in order, the stripes line up into a continuous diagonal down the spine.
Any missing/doubled/out-of-order/flipped unit breaks the diagonal. The marks live
only in the spine/waste area and never survive into the trimmed book.

## Key design decisions (portable across projects)

1. **Reuse the existing "cut border" invariant instead of adding a
   signature-index field.** The imposition already drew a cut border exactly once
   per gathered unit, on that unit's outermost recto — which is precisely the
   spine that should carry the mark. So the collation step is just *the ordinal of
   a side among cut-bearing sides*. This meant **no new field on the per-side
   struct**, no new boundary detection, and the steps come out `1..N` in binding
   order for free. Look for the analogous "once-per-signature" flag in the target
   project and piggyback on it.

2. **Endpaper/wrapper sheets participate** as the first and last steps (not just
   body signatures). Total steps = count of cut-bearing sides. Confirm with the
   user whether their equivalent wrapper sheets should be in the diagonal.

3. **Geometry is emergent, not drawn diagonally.** On any single flat sheet the
   mark is just a horizontal filled stripe at the fold; the diagonal only appears
   across the *thickness* of the gathered block (signature order = spine depth).
   So the per-sheet code only needs a vertical position as a function of step
   index.

4. **Band placement:** a vertical band from the sheet top (above the top cutline,
   in the waste) down to a small gap above the **top sew mark**. Divide into
   `total` equal, top-anchored slots; step *k* fills slot *k*. This satisfies
   "first mark starts at page top" and "last mark stays above the first sew hole."
   The gap-above-sew constant prevents collision with the sewing holes.

5. **Compute the band in the typesetting language (TeX/pgfmath), driven by named
   constants**, mirroring how the existing sew-mark geometry was already
   parameterized — rather than precomputing positions in the host language. Keeps
   one geometry source of truth and matches surrounding idioms.

6. **Stripe height ≈ one slot** so adjacent units tile vertically into a solid
   diagonal band rather than disconnected ticks. Stripe width is a small fixed
   straddle of the fold line (6 mm here). Both are tunable constants.

## Impacted files (this project — map to equivalents)

- **`src/latex.rs`** (the only code file) — all changes here:
  - Added two cm string constants (stripe half-width, clearance above the top sew
    mark), next to the existing sew constants.
  - Added a `\drawcollationmark{step}` TeX macro constant, next to the existing
    sew macro.
  - In the wrapper-document builder: counted gathered units
    (`sides.iter().filter(|s| s.cut).count()`), emitted doc-level defs
    (`\collationtop` = sheet top, stripe width, gap, `\collationtotal`), and
    registered the macro.
  - Bumped the per-side macro `\imposedsheetside` from 5 to 6 parameters; the 6th
    is the step (0 = none), guarded by `\ifnum#6>0 \drawcollationmark{#6}\fi`.
  - In the emission loop: a running counter incremented on cut sides, emitted as
    the 6th arg.
  - Tests: updated the **one** exact-line-equality test for the new arity/steps;
    added 4 tests (step increments once per unit in order; total counts
    wrappers+signatures; band floor held above top sew mark; band top tracks paper
    size). All other imposition tests used substring `.contains` on a 5-field
    prefix, which is still a substring of the 6-field call, so they needed no
    change — **prefer substring assertions over full-line equality** to minimize
    churn when adding a field.

- **`README.md`** — extended the one-sentence description of the printed marks
  (fold/sew/cut) to include the collation stripe. (Project convention: rewrite in
  place, no "revision note" block.)

- **`llm_artifacts/<dated>_COLLATION_SPINE_MARKS_PLAN.md`** — the implementation
  plan (Context / Current state / Desired end state / phased steps with per-step
  LLM + human verification). Kept as a durable record.

## Verification approach worth repeating
- Unit tests assert the emitted step sequence and the geometry strings — fast, no
  PDF needed.
- For the visual check: generate a multi-signature wrapper, set the inner page
  count to 0 so no body PDF is required, compile the **marks-only** sheets, and
  render a few cut-side pages to PNG. Confirm (a) step 1 above the top cutline,
  (b) monotonic descent, (c) last step clears the top sew mark. The full "gather
  and see the diagonal" check is left to a human with a real printed block.

## Watch-outs
- The stripe straddles the same fold x-coordinate as the fold line, so it covers a
  short stretch of the fold dashes in the band region (intentional; flag to the
  user if they want it offset).
- Division by `total` in the macro: ensure `total ≥ 1` always (here it's ≥ 2
  because the two wrappers always count).
- If the target project's "once-per-unit" marker is on a *verso* rather than a
  recto, verify that face still becomes the visible spine exterior after folding;
  otherwise choose the outermost sheet's outer face explicitly.
