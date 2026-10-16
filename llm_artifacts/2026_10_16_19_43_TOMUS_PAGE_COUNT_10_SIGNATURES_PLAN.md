# Retarget Tomus Page Count to 10–11 Signatures

## Context

The facsimile reader splits Plutarch's *Lives* into bound volumes ("tomi"). Each
tomus is sewn from whole 32-page signatures, so a tomus's physical page count is
always a multiple of 32 (`src/signatures.rs`). The volume splitter
(`src/volumes.rs`) decides where one tomus ends and the next begins by balancing
each tomus toward a **target** page count and never letting one exceed a **max**.

Today those are `TARGET_PAGES = 400` and `MAX_PAGES = 500`, which yields **eight
tomi**. The desired physical format is smaller, thinner volumes: a target of **10
signatures (320 pages)** and a hard cap of **11 signatures (352 pages)**. Thinner
targets mean more, smaller tomi — almost certainly **10 to 12** instead of 8.

Growing past eight tomi touches two numeral-aware pieces of code that assume at
most eight volumes:

- The Rust side renders `TOMUS <roman>` labels and title pages with
  `to_roman` (`src/latex.rs`), a general integer→Roman converter with **no upper
  bound** — it already handles IX, X, XI, XII and beyond, so it needs no change.
- The standalone dust-cover generator (`tools/dust_cover.py`) hard-codes the
  numerals `("I" … "VIII")` in a fixed tuple and rejects any Tomus outside 1–8.
  This is the roman-numeral parsing that must change.

The prose in `README.md` and `tools/README-dust-cover.md` names "eight tomi",
`tomus_1 … tomus_8`, and "Tomus number, 1 through 8" in several places; those
must follow the new count.

This plan changes the two page-count constants, generalizes the dust-cover
numeral handling, regenerates the build to learn the real tomi count, and updates
the documentation to match.

## Current state of the world

**`src/signatures.rs`** — `PAGES_PER_SIGNATURE = 32` and
`pad_to_multiple_of_32(pages)` round a volume's page count up to a whole
signature. Both target (320) and cap (352) requested here are exact signature
multiples (10 × 32 and 11 × 32), so the cap coincides with a signature boundary.

**`src/volumes.rs`** — the splitter and its constants:

- `pub const TARGET_PAGES: u32 = 400;`
- `pub const MAX_PAGES: u32 = 500;`
- `split_into_volumes(units, target, max)` groups Lives into pair-groups (a Life,
  its paired Life, and their Comparatio share a `group` number), never splits a
  pair-group, forces a hard break at the v1→v2 source boundary, and within each
  source calls `split_source(group_pages, target, max, source)`.
- `split_source` carries a running `sum` seeded with `overhead_of(true)` (blank
  leaves + front matter) and, before adding the next pair-group to a non-empty
  tomus, closes the tomus if adding would land farther from `target` than closing
  now (`post_dev > pre_dev`) **or** would push `sum + pages` past `max`.
- `volume_physical_pages(content, source, is_first_of_source, is_tomus_one)` adds
  `BLANK_LEAF_PAGES` + `front_overhead(...)` to the content pages and pads to a
  whole signature with `pad_to_multiple_of_32`.
- **Important invariant / risk:** a pair-group is *never* divided. `split_source`
  only applies the `over_max` guard when the current tomus is non-empty, so a
  single pair-group whose own content + minimum overhead exceeds `max` is emitted
  alone in an oversized tomus — the splitter cannot break it. At `max = 500` no
  real pair-group hits this. At the new `max = 352` the headroom is much smaller,
  so this plan must verify no single pair-group overflows 11 signatures.

Constants are consumed in exactly four non-test sites plus tests:

- `src/pipeline.rs:138` and `src/facsimile/mod.rs:381` — the production splits.
- `src/pipeline.rs:665`, `src/facsimile/mod.rs:763`, and
  `src/volumes.rs` tests reference the constants. The `volumes.rs` unit tests that
  pass **explicit literal** target/max arguments (e.g. `250, 400`, `400, 500`,
  `10_000, 300`) are self-contained and unaffected by the constant change; only
  `each_volume_within_max_pages`, which asserts against
  `pad_to_multiple_of_32(MAX_PAGES)`, reads the constant.

**`src/latex.rs`** — `to_roman(n: u32) -> String` (subtractive table M…I) renders
Tomus numerals; `tomus_label`, `title_page`, and `tomus_title_page` call it. No
upper bound; no change needed. The imposition wrapper names tomi by `i + 1` and
reads real page counts from the lualatex log, so it scales to any tomi count.

**`tools/dust_cover.py`** — numeral handling:

- `TOMUS_ROMANS = ("I", "II", "III", "IV", "V", "VI", "VII", "VIII")`
- `int_to_roman(n)` raises `DustCoverError("tomus must be between 1 and 8")`
  unless `1 <= n <= len(TOMUS_ROMANS)`, else returns `TOMUS_ROMANS[n - 1]`.
- `roman_to_int(s)` does `TOMUS_ROMANS.index(s) + 1`, raising
  `DustCoverError(f"invalid Tomus numeral: {s!r}")` on miss.
- `parse_inner_tex` reads the numeral off the inner `.tex` title page with the
  regex `\\large\s+TOMUS\s+([IVXLCDM]+)` (the character class already admits X,
  L, etc.) and calls `roman_to_int` to compare against `--tomus`.
- `main` takes `--tomus` as a bare `int` with no `choices` bound.

**`tools/test_dust_cover.py`** — `RomanNumeralTests` round-trips `range(1, 9)` and
`test_tomus_outside_eight_volumes_raises` asserts `int_to_roman(9)` raises
"between 1 and 8" and `roman_to_int("IX")` raises "invalid Tomus numeral".

**`tools/audit_inner_tomi.py`** — derives its tomus list dynamically from the
build manifest and the `--tomus` arguments; it does not hard-code the count. Its
only numeric literals (`637`, `708`, `710`) are **source-PDF page bounds**, not
tomi counts, and are unaffected. The audit's `expected_counts.physical_by_tomus`
array in its manifest is generated per build, so it tracks whatever count the
build produces.

**Documentation** — `README.md` (lines ~23, 34–35, 159, 283–317) and
`tools/README-dust-cover.md` name "eight tomi", `tomus_1_inner.pdf` through
`tomus_8_inner.pdf`, and "Tomus number, 1 through 8".

**Frozen-evidence tests** — `src/facsimile/audit_repair_tests.rs:4883` asserts a
**stored baseline fixture** has 8 volumes. This reads a frozen JSON archive
documenting a past build, not live output, and is additionally `#[ignore]`-gated
behind "requires frozen inner-tomi audit evidence archives"; it is a historical
record and should not be re-pointed at the new count. The membership test at
`src/facsimile/mod.rs:763` reads pre-built `.tex` files from
`build_facsimile/praefatio_hybrid_check/` and compares them to a fresh split, so
it requires that build directory to be regenerated after the split changes.

## Desired end state

- `src/volumes.rs` has `TARGET_PAGES = 320` and `MAX_PAGES = 352`, with doc
  comments stating these are 10 and 11 whole 32-page signatures.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo fmt --all --check` are green.
- A full offline facsimile build produces **N tomi** (expected 10–12), every one
  of which is at most 11 signatures (352 book pages) after padding, and no single
  pair-group overflows the cap. N is recorded in the plan's verification notes and
  used to update the docs.
- `tools/dust_cover.py` accepts and renders Tomus numbers across the full produced
  range (and comfortably beyond), parsing and emitting Roman numerals
  algorithmically rather than from an 8-entry tuple, with an explicit, documented
  upper bound. `tools/test_dust_cover.py` covers the widened range.
- `README.md` and `tools/README-dust-cover.md` describe the new signature target,
  the real tomi count N, the `tomus_1 … tomus_N` range, and the dust-cover
  `--tomus` bound — rewritten in place (no "update note" blocks), per AGENTS.md.

## Tech Learnings

- **Signatures and padding.** A signature is one folded sheet-stack of 32 book
  pages. `pad_to_multiple_of_32` uses `u32::div_ceil` — integer division that
  rounds up — so `pad_to_multiple_of_32(321)` is `352`. Because the new `max`
  (352) is itself `11 × 32`, a tomus whose pre-padding content + overhead is `≤
  352` pads to **exactly** `≤ 352`; the cap and the signature grid line up, which
  is why 352 (not, say, 350) is the right number to encode.
- **Subtractive Roman numerals.** The classic algorithm walks a descending table
  of (value, symbol) pairs — `(1000,"M") … (9,"IX") (5,"V") (4,"IV") (1,"I")` —
  emitting each symbol while the remaining value is ≥ its weight. `src/latex.rs`
  already does this; Phase 2 mirrors it in Python so the dust cover is not capped
  at a hand-written list. Parsing back (`roman_to_int`) can reuse the same table,
  or — simplest and sufficient here — generate the numeral for each candidate
  integer in range and match. Both directions must agree, which the round-trip
  test enforces.
- **Why `choices=range(...)` is awkward in argparse.** `--tomus` stays a plain
  `int`; bounds are enforced in `int_to_roman` so the error message is a single
  `DustCoverError`, consistent with the rest of the tool's validation.

---

## Phase 0 — Pre-flight: predict the new split without recompiling

*Suggested implementer: Opus.* Judgment call on the overflow outcome, but no code
change. This phase runs **first** so the expensive Phase 3 rebuild is only started
once the tomi count and the cap are known to be sound.

The split is a pure function of each unit's composed trim-page count and the two
constants. Composing is the slow part (your memory note
`facsimile-compose-tests-are-slow`: compose is >10 min; a full build is
hour-plus), but the per-unit trim counts **do not change** when only
`TARGET_PAGES`/`MAX_PAGES` change — they are already recorded in the most recent
full build log as lines of the form `  <id>: <k> source page(s) -> <m> trim
page(s)` (emitted at `src/facsimile/mod.rs:205`). So the new split can be computed
in seconds from an existing log, with no recompose.

### Step 0.1 — Compute the predicted split from existing per-unit counts

Find the most recent complete build log that contains the `-> <m> trim page(s)`
lines (e.g. one of the `tmp/*-full-build.log` files, or regenerate just the log by
locating the latest run; do **not** recompose for this step). Parse each unit's
trim count with the regex already used by the ignored QA test
(`src/facsimile/mod.rs:745`):
`^  ([a-z_]+): \d+ source page\(s\) -> (\d+) trim page\(s\)`.

Build `UnitPages` for each unit by joining those counts to the unit metadata in
`facsimile_units.json` (`id`, `group`, `kind`, `source`), then call
`volumes::split_into_volumes(&units, 320, 352)` and, for each resulting tomus,
`volumes::volume_physical_pages(content, source, is_first_of_source,
is_tomus_one)`. Do this with a short throwaway Rust test or a `tmp/` binary that
imports the crate — the pure split logic needs no build artifacts. Report:

- **N** — the predicted number of tomi (expect 10–12).
- Each tomus's padded physical page count, confirming every one is `≤ 352`.
- **Overflow check:** whether any single pair-group's content + minimum overhead
  already exceeds 352. Because a pair-group is never divided (`split_source` only
  applies the `over_max` guard to a non-empty tomus), such a group would be emitted
  alone in an over-cap tomus. If one exists, **stop and surface it to the user** —
  that Life-pair alone needs more than 11 signatures — rather than raising the cap.

If the overflow check is clean, proceed. If not, the plan pauses here for a
decision before any code or rebuild.

**LLM verification:**
- The throwaway harness prints N and the per-tomus padded counts; every count is a
  multiple of 32 and `≤ 352`.
- No pair-group is reported as over-cap (or, if one is, the run stops for a user
  decision).

**Human verification:** Read the predicted N and per-tomus sizes; sanity-check
that the count and distribution look plausible (some tomi well under the 320
target is expected — target is a goal, not a floor). Confirm the prediction before
the Phase 3 rebuild is kicked off.

---

## Phase 1 — Retarget the page-count constants

*Suggested implementer: Sonnet.* Tightly scoped: two constants, their doc
comments, and one constant-referencing test; the exact names and values are
pinned below.

### Step 1.1 — Change `TARGET_PAGES` and `MAX_PAGES`

In `src/volumes.rs`, change the two constants and their doc comments:

```rust
/// Target physical pages per volume: ten whole 32-page signatures.
pub const TARGET_PAGES: u32 = 320;
/// Hard cap on physical pages per volume: eleven whole 32-page signatures.
pub const MAX_PAGES: u32 = 352;
```

Do not change `split_source`, `volume_physical_pages`, `front_overhead`, or any
test that passes **literal** target/max arguments — those literals are
intentional fixtures of the balancing algorithm and must keep their values
(renumbering/rewriting them would break what they pin down). The only test that
reads the constant, `each_volume_within_max_pages`, continues to assert against
`pad_to_multiple_of_32(MAX_PAGES)` and needs no edit: its `regular_units`
content pads well under 352.

**LLM verification:**
- `cargo build`
- `cargo test --lib volumes::` — all `volumes` unit tests pass.
- `grep -n "TARGET_PAGES\|MAX_PAGES" src/volumes.rs` shows `320` / `352`.

**Human verification:** None (constant change; behavior is verified end-to-end in
Phase 3).

### Step 1.2 — Confirm the whole suite still compiles and passes

Run the full fast suite (excluding the slow compose/build-artifact tests, which
Phase 3 handles).

**LLM verification:**
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- `cargo test` — green except any test explicitly gated on a stale
  `build_facsimile/` artifact directory (see Phase 3) or `#[ignore]`.

**Human verification:** None.

---

## Phase 2 — Generalize dust-cover Roman numerals beyond eight tomi

*Suggested implementer: Sonnet.* Mechanical and well-specified: replace the fixed
tuple with algorithmic conversion bounded by a named constant, and widen the
tests.

### Step 2.1 — Algorithmic `int_to_roman` / `roman_to_int` with an explicit bound

In `tools/dust_cover.py`, replace the fixed `TOMUS_ROMANS` tuple and the two
converters with a bounded algorithmic form. Define a module constant for the
upper bound, generous enough to never need touching again as the tomi count
settles (the real count will be 10–12):

```python
MAX_TOMUS = 20

_ROMAN_TABLE = (
    (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
    (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
    (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
)


def int_to_roman(n: int) -> str:
    if not 1 <= n <= MAX_TOMUS:
        raise DustCoverError(f"tomus must be between 1 and {MAX_TOMUS}")
    out = []
    for value, symbol in _ROMAN_TABLE:
        while n >= value:
            out.append(symbol)
            n -= value
    return "".join(out)


def roman_to_int(s: str) -> int:
    for n in range(1, MAX_TOMUS + 1):
        if int_to_roman(n) == s:
            return n
    raise DustCoverError(f"invalid Tomus numeral: {s!r}")
```

The `parse_inner_tex` regex `([IVXLCDM]+)` already admits X/L and larger numerals,
so it needs no change. No other call site changes.

**LLM verification:**
- `python -c "from tools.dust_cover import int_to_roman, roman_to_int; print([int_to_roman(n) for n in range(1,13)]); print(roman_to_int('XII'))"`
  prints `['I','II',…,'XII']` then `12`.
- `grep -n "TOMUS_ROMANS" tools/dust_cover.py` returns nothing.

**Human verification:** None.

### Step 2.2 — Widen the dust-cover tests

In `tools/test_dust_cover.py`, update `RomanNumeralTests`:

- Change the round-trip loop from `range(1, 9)` to `range(1, dust_cover.MAX_TOMUS + 1)`
  (import `MAX_TOMUS` or reference it through the module), asserting
  `roman_to_int(int_to_roman(n)) == n` and spot-checking `int_to_roman(12) == "XII"`.
- Replace `test_tomus_outside_eight_volumes_raises` with
  `test_tomus_outside_valid_range_raises`: assert `int_to_roman(MAX_TOMUS + 1)`
  raises matching the new message `f"between 1 and {MAX_TOMUS}"`, `int_to_roman(0)`
  raises, and `roman_to_int("nonsense")` raises "invalid Tomus numeral". (Renaming
  a test is permitted — it is not renumbering a numbered list.)

Leave `TITLE_FIXTURE` (Tomus II) and the other test classes unchanged.

**LLM verification:**
- `python -m unittest tools.test_dust_cover -v` — all pass.

**Human verification:** None.

---

## Phase 3 — Regenerate the build and verify the real tomi count and cap

*Suggested implementer: Opus.* Requires running the long offline build, reading
the real split, and judging whether any pair-group overflows the 11-signature cap
— a correctness call, not a mechanical edit. Run long commands inside a Monitor
(see memory `long-runs-use-monitor`); the full facsimile build and the
compose-domain tests are slow.

### Step 3.1 — Full offline facsimile build

Run the full build from the project root (PowerShell), per `README.md`:

```powershell
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
cargo run --release --bin facsimile --target-dir target -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
Get-ChildItem build_facsimile/tomus*.pdf
```

Record the printed `Volumes: N` line and the list of `tomus_1 … tomus_N`.

**LLM verification:**
- The build exits 0 and emits `tomus_1_inner.pdf … tomus_N_inner.pdf` and
  `tomus_1.pdf … tomus_N.pdf`.
- For each `tomus_k_inner.pdf`, confirm its page count is a multiple of 32 and
  `≤ 352` (11 signatures). Read the page counts from the lualatex logs or
  `pdfinfo`; equivalently assert the imposition wrapper placed `≤ 11` signatures.
- **Cap check:** if any inner PDF exceeds 352 pages, a single pair-group
  overflowed the cap (the splitter cannot divide a pair-group). Do **not** paper
  over it by raising `MAX_PAGES`; surface it to the user as a binding constraint —
  that Life-pair alone needs more than 11 signatures — and stop for a decision.

**Human verification:** Open two or three `tomus_k.pdf` imposition PDFs and the
corresponding `tomus_k_inner.pdf`; confirm the Tomus title page shows the correct
Roman numeral (including X, XI, XII where applicable), the Lives list is right,
and the final signature's trailing blanks look reasonable. Confirm the total tomi
count is in the expected 10–12 range, matching Phase 0's prediction. Because the
target (320) and cap (352) are only one signature apart and pair-groups are lumpy,
**expect several tomi well below the 320 target** — target is a balancing goal,
not a floor, and most tomi will hold a single pair-group that cannot be paired with
a neighbor without crossing the 352 cap. A spread of sizes is correct, not a sign
of an unbalanced split.

### Step 3.2 — Run the full suite; optionally refresh the ignored QA test

**LLM verification:**
- `cargo test` — full suite green. No default-run test asserts a fixed production
  tomi count (`pipeline.rs:665` and `facsimile/mod.rs:883`/`913` use small
  synthetic fixtures with `>= 2` / `== 1`), so the retarget does not change the
  default suite's expectations.
- `cargo clippy --all-targets -- -D warnings` and `cargo fmt --all --check`.
- The frozen-baseline test at `audit_repair_tests.rs:4883` asserts a stored
  8-volume archive; it is `#[ignore]`-gated and reads a frozen JSON record of a
  past build, so it is unaffected and must **not** be re-pointed at N — it is a
  dated historical record.

The membership test `praefatio_full_build_matches_current_volume_estimate`
(`src/facsimile/mod.rs:742`) is likewise `#[ignore]`-gated ("requires complete
offline QA output and its UTF-8 build log") and is **not** part of a default
`cargo test` run. It reads `tmp/praefatio/full-build.log` and
`build_facsimile/praefatio_hybrid_check/` and compares their per-tomus unit
membership to a fresh split, so after the retarget its stored inputs no longer
match. Refreshing it is optional QA, not a gate on this plan: if you choose to
exercise it, regenerate those dedicated inputs (the QA build that populates
`praefatio_hybrid_check/` and the matching log) and run it with
`cargo test -- --ignored praefatio_full_build_matches_current_volume_estimate`;
otherwise leave it ignored.

**Human verification:** None beyond Step 3.1's PDF read.

### Step 3.3 — Smoke-test a dust cover on a high-numbered tomus

Generate a cover for the highest-numbered tomus produced (e.g. Tomus X/XI/XII) to
prove the generalized numeral path works end-to-end against a real inner `.tex`:

```powershell
python tools/dust_cover.py --tomus N --total-width 15.5 --total-height 9.5 --spine-width 1.25 --cover-width 5.5 --paper-size tabloid --out build_facsimile --no-compile
```

**LLM verification:** The command exits 0 and writes
`build_facsimile/tomus_N_cover.tex`; `parse_inner_tex` matched the numeral without
raising.

**Human verification:** If lualatex is available, drop `--no-compile`, open
`tomus_N_cover.pdf`, and confirm the spine reads `TOMUS <roman>` with the correct
numeral and the back lists that tomus's Lives.

---

## Phase 4 — Update the documentation to the new format

*Suggested implementer: Opus.* Requires substituting the real count N discovered
in Phase 3 and rewriting prose in place (AGENTS.md forbids "update note" blocks),
which benefits from judgment about wording.

**Scope — live docs only.** Edit only `README.md` and `tools/README-dust-cover.md`.
Many files under `llm_artifacts/` and `archive/` also say "eight tomi" / `tomus_8`
/ "1 through 8", but those are dated historical records; AGENTS.md requires leaving
them untouched (older state lives in those artifacts deliberately, not just in git
history). Do not sweep-replace across the repo.

### Step 4.1 — `README.md`

Rewrite in place (no revision blocks), substituting the real N from Phase 3:

- The intro / build section (~lines 23, 34–35): "compiles all eight tomi" →
  "compiles all N tomi"; "`tomus_1_inner.pdf` through `tomus_8_inner.pdf`" and
  "`tomus_1.pdf` through `tomus_8.pdf`" → `… through tomus_N…`.
- The audit line (~159): "inspects the eight …" → "inspects the N …".
- The **Dust covers** section (~283–317): change "Tomus number, 1 through 8" to
  "1 through N" (or "1 through MAX_TOMUS" if you prefer to document the tool's full
  supported range rather than the current build's count — state whichever you
  choose consistently).
- If the README states the target/cap page counts anywhere, update them to "10
  signatures (320 pages) … 11 signatures (352 pages)". (As of this plan the README
  does not print `400`/`500`, but confirm with a grep.)

**LLM verification:**
- `grep -niE "eight|tomus_8|1 through 8| 8 tomi" README.md` returns nothing stale.
- `grep -nE "signature|320|352" README.md` shows the new figures where relevant.

**Human verification:** Read the edited README sections; confirm the counts and
ranges read correctly and no "eight" survives.

### Step 4.2 — `tools/README-dust-cover.md`

Rewrite in place: the `--tomus` row "Tomus number, 1 through 8" → the new bound
(matching the choice made in Step 4.1), and any other "1 through 8" phrasing.

**LLM verification:**
- `grep -niE "1 through 8|eight" tools/README-dust-cover.md` returns nothing.

**Human verification:** Read the table row; confirm it matches
`tools/dust_cover.py`'s actual bound (`MAX_TOMUS`) and the README.

### Step 4.3 — Clear the backlog item (optional)

The parameterized dust-cover work is already shipped; this plan only widens its
numeral range. If the "Parameterized dust covers" line in `BACKLOG.md` is now fully
satisfied, leave it to the user to decide whether to strike it — do not remove
backlog items unprompted.

**LLM verification:** None.

**Human verification:** None.

---

## Open questions / decisions for the user

1. **Dust-cover `--tomus` bound wording in the docs:** document the tool's full
   supported range (`MAX_TOMUS`, 20) or the current build's real count (N)? The
   code supports 20; the build produces N. The plan assumes you prefer the docs to
   name the real count N for the `tomus_1 … tomus_N` ranges but may mention the
   tool accepts up to `MAX_TOMUS`. Confirm.
2. **Pair-group overflow (Step 3.1 cap check):** if a single Life-pair exceeds 11
   signatures on its own, the splitter cannot divide it. The plan stops for your
   decision rather than silently raising the cap. Confirm that is the behavior you
   want.
