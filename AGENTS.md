

# The transcribed Latin is source text, never to be "improved"

The Latin transcribed from the source PDFs by vision is **source text to be
reproduced faithfully**. Never
silently "improve" it: do not normalize or modernize spelling and orthography
(e.g. u/v, i/j, ligatures, the long s), expand or introduce abbreviations,
"correct" apparent errors, re-punctuate, or regularize capitalization. The
cached transcription is the single source of truth; any change to it is a
deliberate, reviewable edit **to that cache**, not an on-the-fly cleanup in the
generator.

**Why:** this is a scholarly reader of a specific printed edition. A reader
comparing it against the original must find the same text; invisible
normalization destroys that trust and cannot be audited after the fact. If the
source is genuinely wrong, record the correction in the transcription cache with
a note rather than patching it downstream — and when in doubt, preserve exactly
what the page says. This is the project's analog of the faithfulness gate: it is
a correctness invariant, not a style preference.

# Renumbering in documents

Renumbering items in documents (section numbers, test numbers, list numbers, etc.) is a **last resort**. Before renumbering, consider:

1. **Is renumbering necessary at all?** Gaps in numbering are often perfectly acceptable, especially in long lists (e.g. tests).
2. **Can a fractional or suffixed number be used instead?** Inserting `6.7.1` or `6.7b` between `6.7` and `6.8` avoids touching any existing numbers.

Only renumber when the numbering itself carries meaning that would be genuinely broken by gaps or fractional numbers.

Note that creating numbered lists is acceptable in general, but renumbering an existing list should be avoided.

# Updating existing markdown documents

When explicitly asked to update an existing markdown file, **change the
existing text in place** in preference to adding an "Update", "Revision
note", or "Status" block (or strikethrough-plus-correction markup) on top of
the old text. Rewrite each affected sentence/section so the document reads as
if it had been written today.

**Why:** older versions are preserved git history if anyone needs
them; a document with layers of update notes forces every future reader to
mentally diff the layers, while a self-consistent document is simply read.
Readability beats an in-document audit trail.

This coexists with the renumbering rule above: rewrite *text* in place, but
*numbered identifiers* (step numbers, test numbers) still keep their numbers —
append or suffix-insert new items rather than renumbering.

# Code comments describe the current state, not the history

The same principle applies to code comments. A comment should help a reader
understand the code **as it is now** — not narrate how it used to be or what a
refactor changed. When you write or modify code, the comments should describe
the present state; do not add comments that explain a previous design, note that
the code "used to be" duplicated/inlined/split differently, or say that a given
construct "replaces" / "collapses" / "now generates instead of" an older one.

**Why:** older versions are preserved in git history. A comment like
"these five structs were ~85% identical before the macro" or "this collapses the
four copies into one" is noise to every future reader — it describes a diff, not
the code in front of them. Describe what the macro/helper/abstraction *is and
does*, and let the file history carry the before/after.

Offending example (a module doc that narrates the refactor):

```rust
//! Those five structs were ~85% textually identical … `boundary_model!`
//! generates all of that from one declaration so the five kinds can no longer
//! drift …
```

Better — describe the current contract:

```rust
//! `boundary_model!` generates a census-boundary model (struct + CRUD) from one
//! declaration; the SQL-string builders here keep the column layout in one
//! place. Only `ZctaBoundary::states` is hand-written.
```

This does not forbid explaining *why* the current code is shaped a certain way
(a non-obvious invariant, a safety constraint, a performance trade-off) — that
describes the present and is valuable. Drop only the historical before/after
narration.

# Code comments don't reference the implementation plan

When implementing from a plan in `llm_artifacts/`, do not leave comments that
cite the plan — no phase/step/section identifiers (`Phase 4c`, `§2.2.1`,
`step 6.3`), no "as required by the plan", no plan filenames. This covers code
comments, doc comments, test section banners, and the like.

**Why:** a developer reading the code later has not read the plan — it may be
months stale or already deleted — so a bare `§3.3.5` is pure noise that points
at a document they can't see and that no longer describes the system. Describe
what the code or test actually does; a descriptive test name
(`refresh_users_clears_list_when_no_selection`) already carries the intent. The
plan's traceability belongs in the plan under `llm_artifacts/`, not the source.

# Place "only" (and other modifiers) next to what they modify

In prose, comments, and doc comments, put a limiting modifier — **only**, **just**,
**even**, **also** — immediately before the word or phrase it actually restricts,
not reflexively after the verb. The reflex placement after the verb often attaches
the modifier to the wrong thing.

- Write `` `key` is used only for error context``, not `` `key` is only used for
  error context`` (the restriction is on *what it's used for*, not on the act of
  using it).
- Write "this runs only on the owner pool", not "this only runs on the owner pool".

# Writing implementation plans

This codebase captures non-trivial work in markdown plans under `llm_artifacts/`. A plan is a contract between the planning LLM and the implementing LLM (or a junior developer) — the implementer must be able to pick it up cold, work through it across multiple sessions, and verify each step's correctness without re-deriving design decisions.

**File location and naming.** Plans live in `llm_artifacts/` with filenames following `YYYY_MM_DD_PLAN_NAME.md` — underscore-separated date followed by a descriptive name in UPPER_SNAKE_CASE (e.g. `2026_05_28_LOGIN_SCREEN_PLAN.md`). The date is the plan's authoring date.

**Required opening sections.** Every plan begins with three sections and a fourth optional section, in order:

1. **Context** — what part of the system this plan touches and why the work matters now.
2. **Current state of the world** — what exists today: relevant files, types, and decisions already made.
3. **Desired end state** — what the codebase looks like after the plan is fully implemented, concrete enough that a junior developer can recognize "done" when they see it.
4. (Optional) **Tech Learnings** — Relevant features of Rust, LaTeX, or other slices of the tech stack that would be helpful to a junior developer's learning. Explain each feature and how they relate to the plan. This section should be relatively brief, and may be omitted if the plan is not likely to use any interesting or useful features. (It is left to Claude's discretion to determine what would be interesting or useful to a junior developer.)

These sections exist so a reader who has never seen the work can build a complete mental model before touching code.

**Audience.** Plans are written for an implementing LLM first, but explain concepts at the point where they appear. Assume the implementing LLM cannot be relied on to infer unstated context, fill gaps from architectural taste, or notice an implication the plan didn't spell out. Write each step explicitly — the exact files to touch, the exact function/type/field names, the exact behavior expected — rather than trusting the implementer to derive them from the surrounding description. A junior developer reading the plan straight through should be able to follow what's being asked and why — they should not need to chase a separate primer to understand a domain term or framework concept the plan relies on.

**Step-by-step structure.** The implementation portion is a sequence of small ordered steps, each sized to complete in a single LLM session and to leave the codebase compiling with all previously-passing tests still passing. Group steps into numbered phases (`Phase 1`, `Phase 2`, …). If the plan is revised, new phases should be inserted later without renumbering (eg `Phase 1B`). Within a phase, steps are numbered (`1.1`, `1.2`, `1B.1`, `2.1`…). Creating numbered lists is encouraged; **renumbering existing items is not** — see "Renumbering in documents" above for the rationale and the suffix-or-fractional workaround.

**Each phase names a suggested implementer.** Every phase heading must state which model should implement it — `Suggested implementer: Sonnet` or `Suggested implementer: Opus` — chosen by the phase's complexity and how much inference/judgment it demands. Route phases that are tightly scoped, mechanical, and well-specified (the plan already pins down the exact files, names, and behavior) to Sonnet. Route phases that are more complex, more ambiguous, or require weighing design tradeoffs, inferring unstated intent, or synthesizing across a less-certain part of the codebase to Opus. This is a suggestion for whoever kicks off the implementing session, not a hard gate.

**Per-step verification.** Every step includes two verification subsections:

- **LLM verification** — commands the implementing LLM can run and inspect (`cargo test ...`, `cargo build`, `grep -n ...`). Confirms the step did what it claimed.
- **Human verification** — what a human should look at to (a) confirm things only a human can confirm (real-eyes confirmation that the generated LaTeX source is well-formed, that the compiled PDF renders correctly, that the Latin text lays out as intended and matches the source page), and (b) understand what the LLM did so they can provide feedback or redirect if needed (sanity-reading sample output, running the generator locally, eyeballing a generated `.tex` file or PDF). "None" is acceptable for purely mechanical changes (e.g., a type-system rename whose only observable effect is "still compiles"), but lean toward listing *something* for any step that changes runtime behavior — the human's read is the project's primary feedback loop.

**Automated tests are part of the plan.** Steps that add or change behavior must specify the unit, integration, or end-to-end tests that prove it. Spell test names out in the plan (`login_with_email_succeeds`, `cookie_clearing_header_has_max_age_zero`, …) so the implementing LLM has an explicit checklist and the reviewer can confirm coverage without reading the diff.


**A formatting pass is mandatory too, alongside clippy.** A step that touches Rust
is not done until `cargo fmt --all --check` exits clean — run `cargo fmt` to apply
it, not just to check. Plans must list the fmt pass in the verification steps of
any step that changes Rust, right next to the clippy and test commands; a green
clippy run over unformatted code is not a finished step. (`cargo clippy --fix` in
particular leaves behind over-long lines and unsorted imports that only
`cargo fmt` cleans up.)

There is no CI, so `cargo fmt --all --check`, `cargo clippy --all-targets -- -D
warnings`, and `cargo test` must all pass locally before a change is considered
done — nothing downstream will catch a skipped pass for you.

**Keep the Rust test suite green.** `plutarch-reader` is a single Rust crate that extracts the Latin text from the bilingual source PDFs and generates LaTeX. Plans must drive its tests in their verification steps, and the implementer must keep them green before declaring done:

- `cargo test` runs the full suite. Scope the inner loop with a name filter — `cargo test <name_filter>` — and reserve the full run for before you declare the work done.
- `cargo build` confirms the crate compiles; running the binary and inspecting the emitted `.tex` (and, where relevant, the compiled PDF) confirms behavior a unit test can't.

A change need only run the suites its layer touches, but any change that alters generated output should keep the full `cargo test` green and, where practical, include a step that regenerates and eyeballs a sample document.

**Offer a review after saving a plan.** Once an implementation plan has been generated and saved to `llm_artifacts/`, prompt the user: "Would you like me to review the plan for gaps, inconsistencies, and ambiguities?" If the response is affirmative, do so — re-read the saved plan (and the sources it draws on) and report any missing requirements, internal contradictions, or under-specified steps before implementation begins. The review must also check that the plan is **consistent with extant code patterns** (it follows the idioms, helpers, and conventions the surrounding code already uses, rather than inventing a parallel approach) and that it **avoids duplicated code** (shared logic is extracted into one definition and reused, not copied per call site or per level) — call out any place the plan would diverge from an existing pattern or introduce a near-duplicate of logic that already exists.

**Plans and notes are kept as historical records.** Everything under `llm_artifacts/` — implementation plans and standalone notes alike — is retained after the work is done; do **not** delete or propose deleting a plan/note just because it has been fully implemented or its findings have been resolved. A completed plan documents *why* the code is shaped the way it is, and a note about a problem that has since been fixed still records that the problem existed and how it was handled — both are valuable to a future reader even when no longer "live". Leave them in place as a dated record. (This is the one deliberate exception to the "older versions live in git history" reasoning applied elsewhere: these artifacts are the durable narrative, not the source, so they stay on disk rather than being pruned.)
