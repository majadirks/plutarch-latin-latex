# Context

The inner-tomi repairs are implemented and pushed through step 5.1 of
`2026_10_08_16_00_INNER_TOMI_AUDIT_REPAIRS_PLAN.md`. The facsimile generator's
repair decisions live in tracked Rust code and do not require the local audit
directories. Its ordinary Rust tests are less portable: some authenticate
historical files under ignored `tmp/` and `archive/` directories, so a checkout
containing Git files alone cannot pass `cargo test`.

Keep the ordinary regression checks meaningful and self-contained by supplying
small, tracked projections of the data those checks consume. Put authentication
of the original historical bytes behind explicit ignored evidence tests. A
portable test passing must mean that the checked-in contracts agree; an evidence
test passing must additionally mean that the historical files were present and
their bytes matched the pinned identities. Do not equate those claims.

# Current state of the world

- `src/facsimile/reflow.rs` includes `audit_repair_tests.rs` inside its existing
  `#[cfg(test)] mod tests`. This work belongs in that test support file and its
  fixtures, not the production generator.
- `audit_repair_ledger()` uses `include_str!` to load the tracked
  `fixtures/inner_tomi_audit_repair_source_trace.json` (about 2.38 MB). It contains
  the immutable 59-event inventory, measured diagnoses, source pixel
  fingerprints, duplicate decisions and references to external evidence. Preserve
  this ledger and the other existing fixtures; the new fixture supplements them.
- Six ordinary tests directly or indirectly read untracked historical files:

  | Existing test | External dependency |
  | --- | --- |
  | `audit_omission_inventory_matches_frozen_review` | Original candidates/reviews and historical archive manifest |
  | `audit_omission_inventory_rejects_false_positive_and_duplicate_events` | The same inventory helper |
  | `audit_omission_diagnosis_covers_all_source_lines` | Baseline manifest, baseline traces and source overlays |
  | `audit_omission_post_dedup_covers_all_source_lines` | Post-dedup traces |
  | `audit_omission_source_signatures_match_proposed_repairs` | Post-mask traces and candidate crop/mask variants |
  | `audit_tier_a_completion_requires_all_lines` | Inventory authentication and passing restoration reports |

- `audit_file_bytes()` tries `original_run` beneath
  `tmp/pdfs/inner_tomi_audit/`, then the archived run. The archive fallback is also
  ignored by Git. `validate_audit_omission_inventory()` mixes event validation
  with file reads and hashes. `validate_tier_a_completion()` mixes completion
  validation with restoration-report reads and hashes.
- The inventory is **Tier A: 39 events / 69 lines; Tier B: 20 events / 44
  lines**. All initial Tier-A cases are restored, no cases are promoted, and the
  separate Cato Minor `Et:` label is restored. The confirmed verdicts and
  confidence values are historical facts. The false positive
  `miss_tiberius_gracchus_403_690` remains excluded.
- No event currently carries a `promoted_tier` field, so the completion
  validator's `promoted_tier == "A"` branch is presently vacuous. The
  "initial/promoted Tier-A" wording below is deliberate future-proofing: there is
  no promoted case to find, so the projection has no promotion field to copy and
  the validators must preserve the (currently empty) promoted path without
  manufacturing data. Do not hunt for a promotion field in the ledger — it does
  not exist yet.
- The production `SOURCE_BODY_BANDS` table contains 47 finite source signatures.
  Source preservation, synthetic packing, crop, fragment and bitmap-fingerprint
  tests already work from tracked data or synthetic images. Preserve these tests.
- The existing ignored source-PDF regressions regenerate rasters using Poppler
  when caches are absent. They are a separate verification layer from historical
  archive authentication. `audit_omission_source_trace()` currently calls the
  inventory validator too, so it must receive portable inputs after the split.
- `assert_tier_a_unit_lines_restored()` and `comparison_s2_line_is_whole()` write
  current reports into `archive/inner_tomi_audit_repairs_2026_10_08/`. Redirect
  these current-run outputs during this work so rerunning source regressions
  cannot overwrite evidence that the new authentication tests treat as frozen.
- Two historical archive manifests have different schemas: the audit archive
  uses `files[].bytes`; the repair baseline uses `files[].size`. Both have
  archive-relative `files[].path` and SHA-256 entries. Post-dedup/post-mask
  manifests are lists of repository-relative `{path, sha256}` objects. Some
  manifest descriptive fields contain the original author's absolute paths;
  those strings are historical metadata, not paths to open on another machine.
- `.gitignore` excludes the historical archives, source PDFs, rendered images
  and build outputs. `facsimile_units.json` and
  `plutarch.frontmatter.tags.json` are tracked. `.cargo/config.toml` contains the
  author's build-target path; verification commands below override it without
  changing that unrelated configuration.

# Desired end state

1. `cargo test` passes using Git-tracked inputs and normal test-created temporary
   files, without historical archives, source PDFs, API credentials or preexisting
   rendered caches. Existing optional tests elsewhere in the crate keep their
   established prerequisite behavior; this plan must not introduce another silent
   prerequisite skip into the six audit tests.
2. The six existing test names remain ordinary tests. Their inventory, line
   count, order, signature, completion and negative-control assertions still run.
   They load a small fixture using `include_str!` and do not open evidence paths.
3. Explicit `audit_evidence_...` tests are marked `#[ignore = "requires frozen
   inner-tomi audit evidence archives"]`. They are read-only, never substitute
   regenerated output for historical bytes, and fail clearly if evidence is
   missing or mismatched. They run without the PDFs, Poppler, LuaLaTeX or API
   access when the archives themselves are supplied.
4. One projection definition creates the portable data from authenticated
   historical JSON. Ordinary validators and evidence tests reuse the same
   semantic validation helpers. No independent copies of the validation or
   projection rules are introduced.
5. A fresh-checkout check constructs a snapshot from Git-tracked files, excludes
   the local audit evidence, and runs the full ordinary suite. This succeeds
   before completion is claimed.
6. Documentation distinguishes portable regression checks, source-PDF regression
   checks and historical evidence authentication. It explains that the new
   compact fixture is a projection with provenance, not a replacement for the
   full historical record or a fresh proof of source-pixel preservation.

Production repair tables, Latin, source PDFs, raw caches, transcription caches,
unit rows, thresholds, global scale, packing and generated-reader behavior must
remain unchanged. Preserve all historical plans, notes and archives. Publishing
or uploading the archives is outside this plan; do not invent a download URL.

# Tech Learnings

`include_str!` embeds a tracked fixture into the test executable at compile time.
Reading its JSON therefore does not depend on the working directory or on a
runtime evidence file. `#[ignore]` still compiles a test but excludes it from
ordinary execution; an explicit `--ignored` invocation runs it. A missing file
inside an explicitly requested evidence test is a failure, not an ignored result.

A SHA-256 value attached to a JSON projection identifies the **original file**
from which it was derived. It is not the hash of the projected JSON. Semantic
comparison of a tracked projection and byte authentication of the original are
different operations; preserve both without presenting one as the other.

# Phase 1 - Capture portable inputs and extract reusable validators (Suggested implementer: Opus)

## 1.1 Add the compact fixture and shared projection/validation support

**Files:** `src/facsimile/audit_repair_tests.rs` and new
`src/facsimile/fixtures/inner_tomi_portable_audit_evidence.json`.

Keep the current ordinary-test entry points and their archive-backed behavior in
place for this step, so the populated development checkout remains green while
the new support is introduced.

1. Add `portable_audit_evidence()` using `include_str!` and `serde_json` in the
   same style as `audit_repair_ledger()`. The new JSON has `schema_version: 1` and
   a `documents` object keyed by repository-relative logical evidence path. Each
   value is `{ "source_sha256": "...", "value": { ... } }`. Its SHA is the
   original file's pinned SHA, never a purported digest of the projection.
   Do not put images, base64 bitmaps, OCR text, full manifests or rendered canvas
   arrays in this new fixture. Existing bitmap fixtures stay in place.
2. Add the shared helpers `project_audit_evidence(ledger, documents)` and
   `portable_evidence_document(evidence, reference)`. The latter returns a JSON
   document or an error for a missing path, missing/malformed source digest or
   digest metadata differing from `reference.sha256`. It performs no filesystem
   read. Use `BTreeMap` or sorted object keys for deterministic document ordering.
   References with the same path but different expected hashes must be rejected.
3. Give candidates/reviews a single canonical archive identity through
   `historical_audit_json_reference(ledger, name)`: the path is
   `<ledger.archive.path>/runs/step102-final-all-5/<name>`, with the existing
   `candidates_sha256` or `reviews_sha256`. Do not use an `original_run` fallback
   when constructing the portable fixture or authenticating the archive.
4. The projection keeps these exact source fields, copying values without
   rewriting Latin, coordinates, roles, IDs, paths, confidence or verdicts:

   | Document kind | Retained content |
   | --- | --- |
   | Candidates | All `missing_occurrence` candidates with `unique_loss == true`, including those the reviews reject; each retains `event_id`, `unit_id`, `type`, `unique_loss`, `source_regions`, `missing_line_count`, `confidence`, and `crops` |
   | Reviews | Matching review `events` with `event_id`, `verdict`, and `evidence`; omit unrelated fixed-panel pages |
   | Baseline traces | `unit_id`, `metrics`, and `scale`; used for trace identity and metadata agreement, not copied image authentication |
   | Post-dedup traces | `unit_id`, `metrics`, `scale`, and `source_pages` projected to `page` and `retained_index` |
   | Post-mask traces | `unit_id`, and the `pages` needed by the 47 repair signatures; each retains `source`, `source_page`, `raster`, `crop_edges`, `raw_bands`, and the existing `candidate_crops`/`candidate_results` variants projected to `crop_edges` and `raw_bands` |
   | Restoration reports | `unit_id`, `events`, and complete `reviewed_lines`, including reviewed neighbors and the separately tracked Cato label |

   Derive required units/pages from the ledger, not a second hard-coded unit list.
   Deduplicate a document shared by several events. Do not select a convenient
   variant by manufacturing its values from `SOURCE_BODY_BANDS`; copy the actual
   measured variants from authenticated traces. Missing required source fields
   fail projection. Treat legitimately absent optional candidate arrays as empty.
5. Produce the first fixture from the existing hash-verified archives, checking
   every input file's raw bytes against the ledger before projecting it. Add an
   archive-input helper `load_fixture_source_documents(ledger, root)` which
   collects the canonical candidate/review references, unique Tier-A baseline,
   post-dedup and post-mask trace references, and initial/promoted Tier-A and
   additional-source-item restoration references. Read each unique file once,
   check its raw SHA against the pinned reference, and retain its full JSON for
   the shared projector. Missing files/fields or conflicting identities fail.
   Use a shared `read_verified_evidence_bytes(root, reference)` for these reads.
   This helper carries the full containment contract spelled out in 2.1(4), built
   in this step rather than deferred: resolve the reference beneath `root`; reject
   absolute paths, drive prefixes, parent traversal and any resolved path escaping
   the root through a symlink or Windows junction (canonicalize the resolved file
   and confirm it stays under the canonicalized root); normalize relative
   separators consistently; hash the original bytes with the existing `sha2` and
   report the logical path, resolved path and expected/actual digest on mismatch;
   and error clearly on a missing file. The later `AuditEvidenceRoot` delegates to
   this definition rather than reimplementing containment. Building this hardened
   reader is the heaviest part of this step; budget for it accordingly. The first
   exporter obtains its root from `PLUTARCH_AUDIT_EVIDENCE_ROOT`, defaulting to
   `CARGO_MANIFEST_DIR`, with no mutable original-run fallback. Add an
   ignored maintenance test `export_portable_fixture` which calls
   the shared authentication loader and projector, then writes pretty UTF-8 JSON
   with a trailing newline to the explicit `PLUTARCH_PORTABLE_FIXTURE_OUT` path,
   creating that path's parent directory first. Fail when that variable is
   absent. Use a temporary output file for review and copy it into the tracked
   fixture deliberately; do not let ordinary tests or the exporter silently
   overwrite the tracked file. The maintenance test name must match neither the
   `audit_evidence_` filter nor the `portable_audit` filter, because `cargo test`
   filters are substrings: a name containing either substring would be listed by
   the step 2.1 `audit_evidence_ --ignored --list` check or swept up by the
   step 1.1 `portable_audit` run. `export_portable_fixture` contains neither.
6. Aim for less than **512 KiB** of added JSON; **1 MiB is the hard limit** for
   the initial fixture. Measure and record the size. If necessary, remove unused
   fields rather than dropping cases, signatures, original roles or checks. No
   compressed opaque payload and no new dependency is needed.
7. Extract the existing semantic assertions into reusable functions returning
   `Result<()>`: `validate_omission_inventory_data`,
   `validate_omission_diagnosis_data`, `validate_post_dedup_data`,
   `validate_source_signature_data`, and `validate_tier_a_completion_data`.
   They take the ledger plus loaded/projected documents as applicable and do not
   open paths. `validate_tier_a_completion_data` invokes
   `validate_omission_inventory_data` internally first, preserving the current
   nesting where completion validation also re-checks the inventory; do not drop
   that composition when splitting the bodies. Keep compatibility wrappers for the
   existing tests in this step. Separate raw-byte reading/hashing from these
   functions; do not copy entire validator bodies for the two input sources.
8. Retain every existing condition: exact eligible inventory, immutable fields,
   false-positive exclusion, 39/69 and 20/44 partition, no deferred Tier A,
   full reviewed line count, ordered nonempty rectangles/landmarks/loss stages,
   exact post-dedup line-region agreement, and measured source-signature matches.
   Completion checks require restored status, a referenced report, all event IDs
   and reviewed lines, and all extra source items. Also require the report's
   `unit_id` to match its owning event/extra item, preventing a wrong-unit report
   from satisfying completion. Preserve the current negative controls.

**New ordinary tests:**

- `portable_audit_evidence_schema_and_size_are_valid`
- `portable_audit_evidence_has_every_required_document`
- `portable_evidence_document_rejects_missing_or_mismatched_reference`
- `portable_audit_projection_is_deterministic`

The deterministic projection test uses small in-memory documents and verifies
equal semantic output for different insertion orders. It does not need archives.

**LLM verification:**

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
rustfmt --edition 2021 src/facsimile/audit_repair_tests.rs
cargo fmt --all
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test portable_audit
cargo test portable_evidence_document
cargo test
cargo build
(Get-Item src/facsimile/fixtures/inner_tomi_portable_audit_evidence.json).Length
git diff --check
```

Run the exporter twice to two distinct temporary files; compare their raw bytes.
Use this exact invocation for each output, changing the output filename for the
second run:

```powershell
$env:PLUTARCH_PORTABLE_FIXTURE_OUT = Join-Path (Get-Location) 'tmp/portable-evidence-1.json'
cargo test --lib facsimile::reflow::tests::export_portable_fixture -- --ignored --exact --nocapture
```

Verified on 2026-10-09: both authenticated exports produced identical raw bytes.
The tracked fixture contains 98 documents and is **448,689 UTF-8 bytes**, below
the 512 KiB target. Every projected original-source digest agrees with the ledger.
The three `portable_audit` tests and the separate `portable_evidence_document`
test passed. The full suite at this step passed 384 library tests and 9 CLI tests,
with 55 library tests ignored; formatting, Clippy, build and diff checks passed.
The fixture and shared support were committed and pushed as `e211ca8`.
`cargo fmt --all` does not traverse the current `include!` test file, so the
explicit `rustfmt` invocation is required.

**Human verification:** Read the fixture's candidate/review row for
`miss_romulus_38_2595`, its four reviewed source lines in the existing ledger,
and the matching restoration projection. Check the extra Cato label remains
separate and that the fixture contains understandable data rather than full
renders. Review the size and provenance distinction.

# Phase 2 - Separate ordinary validation from explicit archive authentication (Suggested implementer: Opus)

## 2.1 Migrate the six tests and add strict evidence tests

**Files:** `src/facsimile/audit_repair_tests.rs`, its new fixture if the shared
projection needs a correction, and test-support portions of
`src/facsimile/reflow.rs` only if a shared helper needs adjustment.

1. Change the six existing ordinary tests to call the shared pure validators with
   `portable_audit_evidence()`. Remove their archive reads and compatibility
   wrappers once all callers have migrated. Keep their existing names and
   existing negative-control cases. A missing portable document is an error;
   there must be no `exists()`-then-return, automatic ignore, environment switch,
   filesystem fallback or archive fetch in an ordinary test.
2. For diagnosis validation, the per-event `reviewed_lines`, `regression_name`,
   `first_loss_stage`, `landmark`, line counts and the `reviewed_source_overlay`
   reference all come from the retained ledger event, not the new fixture; the
   only newly projected diagnosis input is each baseline trace file's `metrics`
   and `scale`. Source-overlay references (read from the ledger) must have a
   nonempty path, a valid 64-hex-digit digest and a valid recorded origin; do not
   pretend that this authenticates PNG bytes. Baseline trace and overlay byte
   checks belong to the evidence tests below. Baseline traces are the case with
   no ledger metadata to agree with: the ledger holds only their `{path, sha256}`
   reference, so the projected baseline `metrics`/`scale` can be checked only for
   internal validity (present, positive scale), not cross-checked against the
   ledger. Post-dedup and post-mask metadata, by contrast, carry inline
   `metrics`/`scale`/`source_pages` in the event, so post-dedup metadata must
   agree with its projected trace, including the retained source-page sequence.
   Source-signature matching must use the projected measured variants through the
   same helper used for a full trace.
3. Update the call in `audit_omission_source_trace()` to the portable inventory
   validator. Source-PDF regressions still require their deliberately supplied
   source PDFs and tools, but must not also require a historical audit archive.
   Redirect the two current-run writers identified in the current-state section
   to `tmp/pdfs/inner_tomi_audit_repairs/restoration-current/` and
   `tmp/pdfs/inner_tomi_audit_repairs/s2-current/` respectively. Do not edit
   `passing_evidence` references, frozen report hashes or historical files.
4. Implement a test-only `AuditEvidenceRoot` with
   `from_environment()`, `resolve(logical_path)`, `read_verified(reference)`, and
   `read_verified_json(reference)`. `PLUTARCH_AUDIT_EVIDENCE_ROOT` names the
   directory **containing** the repository-relative `archive/` tree; default to
   `CARGO_MANIFEST_DIR` when it is unset. This lets another machine place the
   evidence outside its checkout without rewriting the ledger.
   Resolve each file beneath that root. Reject absolute paths, drive prefixes,
   parent traversal and resolved paths escaping the root through a symlink or
   Windows junction. Normalize relative separators consistently. Do not
   interpret manifest `source`, `destination`, or `archive` descriptive absolute
   paths as executable file locations. Validate original manifest bytes before
   interpreting their entries; never rewrite a manifest to relocate evidence.
5. `read_verified` hashes original bytes using existing `sha2`, reports the
   logical and resolved path plus expected/actual digest on mismatch, and errors
   on a missing file. Reuse it for every JSON and PNG reference. Evidence tests
   have no network operation, paid call, Poppler invocation or generator call.
   Remove `audit_file_bytes()` after migrating its callers; archived
   candidates/reviews are the authoritative evidence input, with no mutable
   `original_run` fallback.
6. Add exactly these seven historical tests with the stated ignore reason:

   | Ignored test | Required authentication and shared semantic check |
   | --- | --- |
   | `audit_evidence_historical_inventory_authenticated` | Candidates/reviews raw hashes, historical manifest raw hash, and the full inventory validator |
   | `audit_evidence_baseline_diagnosis_authenticated` | Baseline manifest raw hash, each unique Tier-A baseline trace, and each reviewed source-overlay PNG; run diagnosis validation |
   | `audit_evidence_post_dedup_traces_authenticated` | Pinned post-dedup manifest and all referenced traces; run post-dedup validation |
   | `audit_evidence_post_mask_signatures_authenticated` | Pinned post-mask manifest and all referenced traces; run source-signature validation |
   | `audit_evidence_restoration_reports_authenticated` | Every unique passing report for initial/promoted Tier A and extra source items; run completion validation |
   | `audit_evidence_portable_fixture_matches_archives` | Load all JSON needed by the shared projector from authenticated bytes and require exact semantic equality with the tracked projection, including source hashes and document keys |
   | `audit_evidence_archive_manifests_authenticated` | Walk every entry in both historical archive manifests and both stage manifests, verifying bytes, digest, declared size where present, and pinned file counts/total bytes where present |

   Implement shared manifest handling for the two documented schemas and stage
   list format. Stage entries already contain repository-relative paths;
   historical `files[]` entries are relative to their manifest's archive
   directory. Do not prepend an archive prefix twice. Reject duplicate paths,
   conflicting hashes and escaping paths. Note that identical hashes under
   *different* paths are expected and legal — the post-dedup and post-mask trace
   files are byte-identical (same SHA-256) but live at distinct stage paths — so
   the walker must key uniqueness by path and flag only a repeated path or a path
   whose bytes disagree with a pinned hash, never two paths that share a hash.
   Stream large file hashes rather than retaining all files in memory. Cache each unique read/parsed document within
   a test invocation, not across tests through mutable global state.
7. Add ordinary negative tests using `tempfile::TempDir` and tiny byte files to
   verify the authentication helpers themselves without historical evidence:

   - `evidence_reader_rejects_missing_file`
   - `evidence_reader_rejects_hash_mismatch`
   - `evidence_reader_rejects_escaping_path`
   - `evidence_manifest_supports_archive_and_stage_formats`
   - `evidence_manifest_rejects_duplicate_or_mismatched_entries`
   - `audit_completion_rejects_wrong_unit_and_missing_report_line`

   Pass an explicit root to these helper tests; do not mutate process environment
   variables or the working directory in concurrent Rust tests. Where creating
   a symlink needs unavailable platform permissions, the lexical escape tests
   still run and the limitation is recorded; production root containment must
   still check canonicalized file paths.
8. The existing completion mutation for a null report path must still fail. Add
   mutations for a report's wrong unit and a removed reviewed line in its
   projection. Keep inventory duplicate/false-positive/missing/unreviewed/
   changed-region mutations. These checks must reach the intended validator
   condition, not fail earlier because a fixture record cannot be loaded.

**LLM verification:**

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
rustfmt --edition 2021 src/facsimile/audit_repair_tests.rs
cargo fmt --all
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test audit_omission_
cargo test audit_tier_a_completion_requires_all_lines
cargo test audit_completion_rejects_wrong_unit_and_missing_report_line
cargo test evidence_reader_
cargo test evidence_manifest_
cargo test
cargo build
cargo test --lib audit_evidence_ -- --ignored --list
cargo test --lib audit_evidence_ -- --ignored --nocapture --test-threads=1
git diff --check
```

Verified on 2026-10-09: the ignored list contains exactly the seven historical
tests, with no ordinary test or exporter, and all seven passed against the local
archives. The focused omission, completion, reader and manifest tests passed.
The full ordinary suite passed 390 library tests and 9 CLI tests, with 62 library
tests ignored. Formatting, Clippy, build and diff checks passed.

The three existing source-PDF regressions passed individually with exact
selection in the release profile. This profile retains the assertions and makes
image processing faster:

```powershell
cargo test --release --lib facsimile::reflow::tests::romulus_c1_c6_lines_restored -- --ignored --exact --nocapture
cargo test --release --lib facsimile::reflow::tests::audit_tier_a_source_lines_restored -- --ignored --exact --nocapture
cargo test --release --lib facsimile::reflow::tests::comparison_s2_line_is_whole -- --ignored --exact --nocapture
```

The Tier-A check covered all 24 units. There are 24 fresh restoration JSON
reports under `tmp/pdfs/inner_tomi_audit_repairs/restoration-current/step-4.2/`
and an S2 `verification.json` under
`tmp/pdfs/inner_tomi_audit_repairs/s2-current/`. SHA-256 comparisons before and
after these checks confirm all 24 historical passing reports are unchanged.
Windows denied symlink creation with privilege error 1314; all lexical escape
checks ran, and the evidence reader still checks canonicalized paths
to contain symlink and junction escapes. The symlink creation branch could not
be exercised on this machine. Steps 2.1 and 3.1 were committed and pushed together
as `4a92e19`, as authorized during implementation.

**Human verification:** Read one passing ordinary inventory/completion result
and one passing evidence-authentication result. Confirm the output descriptions
make different claims and that a missing archive produces an actionable failure.
Check the evidence-root directory layout and the source-regression output paths.

# Phase 3 - Prove the tracked-files checkout contract (Suggested implementer: Sonnet)

## 3.1 Add a repeatable fresh-checkout verification command

**Files:** new `tools/check_portable_tests.py` and
`tools/test_check_portable_tests.py`.

Use Python standard-library modules and the existing Git/Cargo executables; do
not add audit/OCR/image dependencies or create a new production CLI flag.

1. The tool runs from any directory and locates the repository relative to its
   own tracked location. Enumerate paths with `git ls-files --cached -z`, then
   copy the current working-tree contents of those paths into a newly owned
   temporary directory. This tests the prospective tracked source state before
   committing. New files must be staged before they participate; report a clear
   error if the portable fixture is not in the tracked-path set. Print that this
   is a tracked-files snapshot of the working tree, not an authentication of a
   committed revision.
2. Do not copy `tmp/`, `archive/`, `target/`, `build_facsimile/`, source PDFs or
   rendered caches, even when an old scratch artifact is tracked. Do not copy
   user secrets. Validate all destinations remain beneath the owned temporary
   directory. Do not follow source links to files outside the checkout; reject
   such a tracked link with a clear diagnostic. Use one Python filesystem
   implementation end-to-end for copying/cleanup, not shell-built recursive
   deletion. Delete only the tool's own verified temporary directory; leave the
   user's original directories intact.
3. Run `cargo test --lib -- --list` and `cargo test` with the snapshot as the
   working directory. Set `CARGO_TARGET_DIR` to a tool-owned target directory
   outside the copied checkout (or an explicit `--target-dir`), overriding the
   author's committed target path. Remove inherited API-key/secret-path
   variables from the child environment and set
   `PLUTARCH_AUDIT_EVIDENCE_ROOT` to an empty child directory. Never invoke the
   ignored source tests, exporter or live LLM tests as part of ordinary testing.
   Cargo may fetch Rust dependencies normally; optionally accept `--offline`
   for an already populated Cargo dependency cache, without requiring it on
   another user's first run.
4. Check the list contains all six original ordinary names and the new portable
   tests, rather than accepting a filtered invocation that ran zero tests.
   Require the full test command's exit status to be zero. Keep stdout/stderr
   available in a user-specified log file, creating the log file's parent
   directory first (`tmp/` is not guaranteed to exist — `.gitignore` lists only
   specific `/tmp/...` subdirectories, not a top-level `tmp/`), and return nonzero
   on any failed command. Do not swallow failure in cleanup or print a success
   after failure.
5. After the ordinary suite passes, explicitly invoke
   `cargo test --lib facsimile::reflow::tests::audit_evidence_historical_inventory_authenticated -- --ignored --exact --nocapture`.
   Require one test to execute and fail with a
   missing-evidence diagnostic. This is an expected negative probe, not a suite
   failure; unexpected success, zero selected tests or an unrelated failure
   makes the portability tool fail. This proves opt-in evidence checks do not
   quietly pass without evidence. Do not modify real archive files to probe it.
6. Add standard-library `unittest` tests named
   `test_snapshot_excludes_local_evidence`,
   `test_snapshot_requires_tracked_portable_fixture`,
   `test_failed_cargo_command_is_reported`, and
   `test_evidence_probe_requires_one_missing_evidence_failure`. Use a temporary
   tiny Git repository or injectable subprocess runner; these Python tests need
   not compile Rust repeatedly. Do not duplicate the Rust evidence validator in
   the Python tool.

**LLM verification:**

```powershell
python -m unittest discover -s tools -p test_check_portable_tests.py
# Stage the intended new files so the snapshot includes them; do not stage
# unrelated user changes.
python tools/check_portable_tests.py --offline --log tmp/portable-tests.log
git diff --check
git status --short
```

Verified on 2026-10-09: all four standard-library Python tests passed. The
tracked-files snapshot copied 127 files, compiled and listed all ten required
ordinary audit/portable names, and passed the full ordinary suite: 390 library
tests and 9 CLI tests, with 62 library tests ignored. The exact historical
inventory probe executed one test and failed with exit 101 and the expected
`missing historical evidence: logical archive/...` diagnostic. The tool accepted
that explicit negative probe and exited successfully.

The snapshot excludes evidence, source scans, rendered caches, build outputs and
secret data; it uses an empty evidence root and its own Cargo target directory.
Its secret-data exclusion preserves the tracked `src/secrets.rs` loader module,
with a Python regression assertion distinguishing that module from secret data
files. No unrelated ordinary-test dependency on untracked inputs was exposed,
and no additional prerequisite skip was introduced. The complete suite output
and negative probe are retained in the local portability logs.

**Human verification:** Run the command and read the final log. Confirm it does
not remove, rename or modify the original archives, source scans or caches.

# Phase 4 - Document the verification layers and finish checks (Suggested implementer: Sonnet)

## 4.1 Document commands, fixture maintenance and verification limits

**Files:** `README.md`, `tools/README-inner-audit.md`, and this plan's existing
verification text as needed. Preserve the previous repair plan and verification
note as dated records; do not rewrite their historical successful results.

1. Add a concise ordinary-test/fresh-checkout explanation beside the current
   README audit setup text, with a link to detailed tool instructions. Update
   affected existing sentences in place; do not overlay a status or revision
   block. State that the normal generator requires the exact source scans and
   tools, while these ordinary tests do not require those scans or the archives.
2. Document the three verification layers in `tools/README-inner-audit.md`:

   - `cargo test`: tracked/synthetic regression contracts.
   - Existing named ignored source-PDF tests: current source-pixel preservation,
     with explicitly supplied scans/Poppler and newly created temporary reports.
   - `cargo test --lib audit_evidence_ -- --ignored --nocapture --test-threads=1`:
     the seven read-only historical authentication checks.

   List all seven evidence names and the exact `--list` command. Do not recommend
   an unfiltered `cargo test -- --ignored`; the crate also has ignored live API
   tests and unrelated expensive checks.
3. Show PowerShell examples setting `CARGO_TARGET_DIR` to a portable location and
   `PLUTARCH_AUDIT_EVIDENCE_ROOT` to a directory containing
   `archive/inner_tomi_audit_2026_10_08/` and
   `archive/inner_tomi_audit_repairs_2026_10_08/`. List the expected manifests,
   stage directories and restoration reports. Explain that the archives remain
   local and ignored, must be supplied separately, and cannot be reconstructed
   byte-for-byte by running the latest generator. No public retrieval location
   has been established; report that limitation directly.
4. Document fixture maintenance using the exact ignored exporter name, an explicit
   `PLUTARCH_PORTABLE_FIXTURE_OUT` temporary JSON path, semantic review, deliberate
   replacement of the tracked fixture, and the complete evidence comparison.
   Current tests do not auto-update the fixture or bless new historical hashes.
   Explain that fields and source digest metadata in the projection are copied
   historical expectations; authenticating their original bytes requires the
   evidence tests. Ordinary green results alone are not a new 69-line source
   preservation claim or human acceptance of the readers.
5. Perform final checks on the populated checkout and with the tracked-files
   tool. Save the observed results, actual fixture byte size, ordinary/ignored
   test counts and any platform limitation in this plan's verification text.
   Zero selected tests, a missing required check or a failed fresh snapshot blocks
   completion. A historical archive missing on a different user's machine is an
   explicitly documented prerequisite for evidence tests, not an ordinary-suite
   failure.

**LLM verification:**

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
rustfmt --edition 2021 src/facsimile/audit_repair_tests.rs
cargo fmt --all
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
python -m unittest discover -s tools -p test_check_portable_tests.py
python tools/check_portable_tests.py --offline --log tmp/portable-tests-final.log
cargo test --lib audit_evidence_ -- --ignored --list
cargo test --lib audit_evidence_ -- --ignored --nocapture --test-threads=1
git diff --check
git status --short
```

Final verification on 2026-10-09 passed the explicit included-file rustfmt pass,
Cargo formatting/check, Clippy with warnings denied, full ordinary suite, build,
four Python tests, final tracked-files snapshot, seven-name ignored list, all
seven historical authentication tests, and diff checks. Both populated and
evidence-free checkouts passed **390 library tests plus 9 CLI tests**, with
**62 library tests ignored**. The snapshot also required the single expected
missing-evidence failure. The final fixture remains **448,689 bytes / 98
documents**. The Windows symlink-creation limitation is recorded under 2.1;
lexical checks and canonicalized containment remain active.

The final manifest walker authenticated these original entries:

| Manifest | Files | Total authenticated bytes |
| --- | ---: | ---: |
| Historical audit | 20,361 | 2,144,334,160 |
| Repair baseline | 4,404 | 1,657,727,176 |
| Post-dedup | 24 | 5,356,261 |
| Post-mask | 24 | 5,374,068 |

Diff inspection confirms that production code outside test support, all existing
tracked source/Latin caches, `facsimile_units.json`, ledger historical hashes and
unrelated Cargo configuration remain unchanged. Every entry in the four manifests
passed byte authentication, and all 24 passing reports retained their original hashes
after source regressions. Generated logs and review exports remain local beneath
`tmp/pdfs/inner_tomi_audit_repairs/portable-verification/` and do not enter the
implementation commits. A full sixteen-PDF rebuild is not required because
runtime/generated-output behavior does not change; the three named real-source
regressions in 2.1 verified the adjusted test-output destinations. Human acceptance
of the generated readers remains separate from these automated results.

**Human verification:** Follow the portable checkout instructions without the
archives, then the explicit evidence instructions with the archives. Confirm the
documentation accurately distinguishes a passed ordinary suite, unavailable
historical evidence and passed source-PDF checks.

# Risks and boundaries

- Moving all six tests behind `#[ignore]` would lose ordinary semantic coverage;
  that does not satisfy this plan. Keep their shared semantic and negative checks
  ordinary and use the compact tracked inputs.
- Regenerating a fixture from mutable current output would erase its independent
  historical expectations. Authentication must precede projection; fixture
  replacement is deliberate and reviewable.
- A compact JSON projection cannot authenticate a discarded PNG or an entire
  original JSON file. Pinned source digest metadata preserves provenance, and
  opt-in tests authenticate original bytes. Do not present the metadata as that
  byte check.
- Current source regression reports must not overwrite the frozen historical
  reports. Keep those histories, identities and final-build evidence in place.
- Preserve exact source Latin and all existing numbered event/step identifiers.
  No spelling correction, Unicode normalization, OCR replacement, confidence
  change, Tier-B promotion or production repair retuning belongs in this work.
- This plan makes the test input boundary portable. Removing the existing
  personal target-dir setting, distributing source PDFs or hosting archives can
  be separate work; the commands here override the setting and state external
  input requirements honestly.
