# Add a `--clean-cache` flag to the facsimile build

## Context

The facsimile build path (`src/facsimile/`) rasterizes every source page with
poppler and caches the resulting PNGs in `<out>/render_cache/` so the width pass and
the compose pass each rasterize a page at most once, and so a *later* build reuses
the rasters instead of re-running poppler over ~1,300 pages. That cache is
deliberately **preserved** across runs: the 2026-10-03 front-matter/layout plan
(Phase 8, `llm_artifacts/2026_10_03_FACSIMILE_FRONTMATTER_AND_LAYOUT_PLAN.md`) removes
the composed strip PNGs after a successful compile but explicitly does **not** delete
`render_cache/`, noting that "if an explicit cache purge is ever wanted, add it behind
a new `--clean-cache` flag on `FacsimileArgs`, defaulting off."

This plan adds exactly that flag. It is wanted for two situations: reclaiming the disk
the raster cache occupies, and forcing a fresh rasterize (for example after changing
the body DPI or when a cache is suspected stale), without the user having to find and
delete the directory by hand.

This change touches only the facsimile binary's argument plumbing and one new
filesystem helper; it does not alter body reflow, detection, or any cache *contents*.

## Current state of the world

- `FacsimileArgs` (`src/facsimile/mod.rs`) holds the parsed CLI inputs: `v1`, `v2`,
  `out`, `units`, `no_compile`, `retag`, `detect_units`, `redetect`, `pages`, `unit`,
  `review_lines`. `restricted()` returns `true` for a `--unit`/`--pages` run.
- `run_facsimile` (`src/facsimile/mod.rs`) creates `<out>`, then (for a non-restricted
  run) calls `clean_generated(&args.out)` to remove the previous run's generated
  artifacts, then computes `let cache = args.out.join("render_cache");` and
  `std::fs::create_dir_all(&cache)?;`. `render_gray` (same file) reads/writes PNGs in
  that directory.
- `clean_generated` (`src/facsimile/mod.rs`) is non-recursive and skips the
  `render_cache/` directory; `remove_composed_pngs` removes only the composed
  body/front PNGs after a successful compile, keeping `render_cache/`. Tests
  `start_cleanup_removes_only_generated` and `post_compile_cleanup_removes_pngs` assert
  a `render_cache/` file survives both.
- The `facsimile` binary (`src/bin/facsimile.rs`) is a `clap` `Parser` whose `Args`
  mirror `FacsimileArgs` field-for-field; `main` copies each field across. Three parse
  tests (`facsimile_run_args_parse`, `facsimile_run_args_parse_pages_and_retag`,
  `detect_units_cli_flags_parse`) assert flag parsing.
- `README.md` documents the facsimile path, its example invocations (lines ~29–43),
  the `Dependencies` table, and a `Files` section describing `render_cache/`-adjacent
  local directories.

There is **no** way today to purge `render_cache/` through the tool.

## Desired end state

- `FacsimileArgs` has a `pub clean_cache: bool` field (default `false`), set by a
  `--clean-cache` CLI flag on the `facsimile` binary.
- When `clean_cache` is set, `run_facsimile` deletes `<out>/render_cache/` (recursively)
  at the start of the run — before the cache directory is (re)created — so the run then
  re-rasterizes every needed page from scratch. The purge happens regardless of whether
  the run is restricted (it is explicit user intent).
- A missing `render_cache/` is not an error: purging when the directory is absent is a
  no-op, so `--clean-cache` is safe on a clean `<out>`.
- With the flag **off** (the default and every existing invocation), behavior is
  byte-for-byte unchanged: the cache is preserved exactly as today.
- The README documents `--clean-cache` in the facsimile examples and explains that it
  forces a fresh rasterize / reclaims the cache's disk.
- `cargo test`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo fmt --all --check` all pass.

---

## Phase 1 — Add and document the `--clean-cache` flag
### Suggested implementer: Sonnet

A small, mechanical change: one new field, one CLI flag, one filesystem helper, its
invocation, tests, and README text.

### Step 1.1 — Add the `clean_cache` field and CLI flag

In `src/facsimile/mod.rs`, add to `FacsimileArgs` (after `review_lines`):

```rust
/// Delete `<out>/render_cache/` at the start of the run, forcing every page to be
/// re-rasterized from scratch. Off by default; the raster cache is otherwise
/// preserved across runs.
pub clean_cache: bool,
```

In `src/bin/facsimile.rs`, add a matching `clap` argument to `Args` (after
`review_lines`):

```rust
/// Delete <out>/render_cache/ before building, forcing a fresh rasterize.
#[arg(long)]
clean_cache: bool,
```

and copy it across in `main`: `clean_cache: args.clean_cache,`.

Update the two `FacsimileArgs` struct literals in `src/facsimile/mod.rs` tests to
include the new field (defaulting `clean_cache: false`): the one in
`start_cleanup_skipped_for_restricted` and the one in `run_facsimile_smoke`. (Search
for `review_lines: false,` — each such literal needs `clean_cache: false,` added
alongside it.) The only other `FacsimileArgs` literal is the non-test one in
`src/bin/facsimile.rs` `main`, already handled by the `clean_cache: args.clean_cache,`
copy above.

#### LLM verification
- `cargo build` is clean.
- `grep -n "clean_cache" src/facsimile/mod.rs src/bin/facsimile.rs` shows the field,
  the CLI arg, and the `main` copy.

#### Human verification
- None (argument plumbing).

### Step 1.2 — Purge the cache when the flag is set

In `src/facsimile/mod.rs`, add a helper beside `clean_generated`:

```rust
/// Delete `<out>/render_cache/` entirely (the poppler raster cache). A no-op when it
/// does not exist, so `--clean-cache` is safe on a fresh output directory.
fn clean_render_cache(out: &Path) -> Result<()> {
    let cache = out.join("render_cache");
    if cache.exists() {
        std::fs::remove_dir_all(&cache)
            .with_context(|| format!("removing {}", cache.display()))?;
    }
    Ok(())
}
```

Invoke it in `run_facsimile` immediately before the cache directory is (re)created —
i.e. right before `let cache = args.out.join("render_cache");` /
`std::fs::create_dir_all(&cache)?;`:

```rust
if args.clean_cache {
    clean_render_cache(&args.out)?;
}
```

Place the purge **after** `create_dir_all(&args.out)` (so `<out>` exists) and **after**
the `clean_generated` block, but **before** the cache `create_dir_all`, so the cache is
removed and then immediately recreated empty. Do not gate it on `restricted()`: a
`--clean-cache --unit theseus` run should still purge.

Note the `--detect-units` mode returns early (before any cache code), so
`--clean-cache --detect-units` does **not** purge — that is intended, because
`--detect-units` never reads or writes `render_cache/`. Do not move the purge earlier
to cover that combination.

#### LLM verification
- `cargo build` is clean; `cargo clippy --all-targets -- -D warnings` is clean.
- `grep -n "clean_render_cache" src/facsimile/mod.rs` shows the helper and its call.

#### Human verification
- Run a build once (populating `build_facsimile/render_cache/`), then run again with
  `--clean-cache --no-compile --unit theseus` and confirm from the console/timing that
  the targeted pages are re-rasterized (the cache was emptied and refilled), and that
  `build_facsimile/render_cache/` exists and is non-empty afterward.

### Step 1.3 — Tests

In `src/facsimile/mod.rs` tests:

- `clean_render_cache_removes_the_dir` — create `<out>/render_cache/` with a file in
  it, call `clean_render_cache(out)`, assert the directory no longer exists; call it a
  second time on the now-absent directory and assert it returns `Ok` (idempotent /
  safe when absent).

In `src/bin/facsimile.rs` tests:

- `facsimile_clean_cache_flag_parses` — parse an arg vector including `--clean-cache`
  and assert `args.clean_cache` is `true`; parse one without it and assert it is
  `false` (this also pins the off-by-default behavior).

Keep `start_cleanup_removes_only_generated` and `post_compile_cleanup_removes_pngs`
green (they must still show `render_cache/` surviving a normal, flag-off run).

#### LLM verification
- `cargo test clean_cache` and `cargo test clean_render_cache` pass.
- `cargo test facsimile` and `cargo test --bin facsimile` pass.

#### Human verification
- None.

### Step 1.4 — Update the README

In `README.md`, under **The facsimile path**, add an example invocation to the code
block (alongside the `--unit` / `--pages` examples):

```sh
# Purge the raster cache first, forcing every page to be re-rasterized.
cargo run --bin facsimile -- --v1 … --v2 … --out build_facsimile/ --clean-cache
```

Add a sentence to the surrounding prose (near the unit-table/cache discussion, or in
the `Files` section's `build_facsimile/` note) explaining that `<out>/render_cache/`
holds the reusable poppler rasters and is preserved across runs by default, and that
`--clean-cache` deletes it at the start of a run to reclaim that disk or force a fresh
rasterize. Rewrite the affected sentence(s) in place (per the "Updating existing
markdown documents" rule) rather than appending a note.

#### LLM verification
- `grep -n "clean-cache" README.md` shows the example and the explanation.

#### Human verification
- Read the README section and confirm the flag's purpose and default (off, cache
  preserved) are clear and consistent with the other flag descriptions.

### Step 1.5 — Final green gate

#### LLM verification
- `cargo test`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo fmt --all --check` all pass.

#### Human verification
- None beyond Step 1.2's.

---

## Notes on scope and invariants

- The flag changes only *when* `render_cache/` is deleted; it never changes the cache's
  contents, the composed-PNG cleanup, or the preserved-by-default behavior. With the
  flag off, every existing run is unchanged.
- `clean_generated` / `remove_composed_pngs` remain as they are; `clean_render_cache`
  is the one path that touches the raster cache directory, so the "preserve
  `render_cache/` unless explicitly asked" contract stays expressed in one place.
- This plan stays on disk as the dated record of why `--clean-cache` exists, per the
  `llm_artifacts/` retention rule.
