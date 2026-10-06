# Inner tomi implementation handoff — 2026-10-06, 11:15 checkpoint

## Context

Continue the implementation of [the minimal repairs plan](2026_10_05_INNER_TOMI_MINIMAL_REPAIRS_PLAN.md). The user requested careful execution, approved all measured post-rebuild proposals, and requested this handoff at a verified stopping point because the current session was approaching its context limit. The work is not complete: approved production repairs are installed, but four composition regressions and a fresh final PDF rebuild remain.

Work in `C:\Users\majad\coding\Plutarch\plutarch-latin-latex` on branch **`plan/inner-tomi-minimal-repairs`**. The implementation checkpoint is committed and pushed as **`36a200b` — Install approved margin masks and freeze final-verification checkpoint**. This handoff is a subsequent documentation change. The tracked tree was clean at that checkpoint; the existing untracked `tmp/` directory holds valuable diagnostic evidence. Do not add or delete it wholesale.

The earlier [implementation handoff](2026_10_06_INNER_TOMI_IMPLEMENTATION_HANDOFF.md) describes an earlier stage and has stale counts and pending implementation instructions. Retain it as a historical artifact, but use this note and the current ledger for resumption.

## Current state of the world

Read these before editing:

1. `AGENTS.md` and the user-provided repository instructions. Latin source text must remain faithful; never normalize spelling, punctuation, capitalization, abbreviations, or glyphs downstream.
2. [The current implementation plan](2026_10_05_INNER_TOMI_MINIMAL_REPAIRS_PLAN.md). Its phase and step identifiers remain stable. Conditional whitespace-compaction and packer-change branches are explicitly **not selected**.
3. [The verification note](2026_10_05_INNER_TOMI_MINIMAL_REPAIRS_VERIFICATION.md), which distinguishes first-stage evidence from current-mask verification.
4. [The post-rebuild review](2026_10_06_INNER_TOMI_POST_REBUILD_REVIEW.md) and [the approval record](2026_10_05_INNER_TOMI_REPAIR_RFIS.md).
5. **`src/facsimile/fixtures/inner_tomi_repair_source_trace.json`**, the authoritative numeric ledger. Its `repairs[]` contains **28 records: 26 mask records covering 27 rectangles, plus two duplicate records**. Do not remeasure or widen approved rectangles, and do not regenerate expected data from the implementation under test.
6. `src/facsimile/reflow.rs`, especially existing extraction, source tracing, mask verification, and composition helpers.

### Plan position

| Phase | State at this checkpoint |
| --- | --- |
| Phase 1 | Complete: measurements and human decisions are frozen, including all post-rebuild proposals. |
| Phase 2 | Complete: identity-scoped duplicate exclusions are installed and source-verified. |
| Phase 3 | Every approved mask is installed and pixel-safe. Final copied-height, spacing, and restored-prose composition regressions remain. No row-compaction mechanism is selected. |
| Phase 4 | Marius terminal extraction passed first-stage verification. Final Timoleon whole-line and Timoleon/Brutus furniture-only canvas regressions remain. No packer change is selected. |
| Phase 5 | First-stage all-sixteen-PDF build and inspection completed. Final rebuild and inspection with all current masks remain. |

### Installed repairs

- D1: `skip_known_duplicate_source_page` excludes source 1 pages **103/104** only for `comparatio_sertorii_et_eumenis`, before rendering or pushing either parallel retained vector. Comparison sequence is **102 → 105 → 106**; Eumenes keeps its shared ending. Preserve the join `magno in` / `pretio habitus`.
- D2: the same helper excludes source 1 pages **369/370** only for `agis`. Retained sequence is **363–368 → 371–379**. Preserve `unius suffragii numero` / `excellerent qui antiquabant legem`.
- All original reviewed furniture masks remain installed, including both O3 rectangles and the Marius terminal ornament mask. Older pre-task masks on source 0 pages 56, 233, 235, 240, and 284 remain intact.
- The twelve newly approved full-height right-margin masks are installed on source 0 pages **281, 287, 296, 298, 299, 304, 306, 308, 470** and source 1 pages **361, 539, 625**. Exact normalized and pixel coordinates are in the ledger.
- A separately approved **header-only** rectangle is installed on source 1 page **347**: raster 1787×2866, half-open pixel rectangle `[140,145,1720,205]`. No right-margin mask was approved for page 347 or page 337. Preserve that scope.
- G7/G8 Theseus chapter spacing is genuine and must remain. The W3 Google notice and O8 Praefatio transition are retained. Additional Cato repeated scans were observed but no further deduplication was authorized.

The mask tests use `reviewed_page_masks` to group all ledger rectangles for each source/page. This matters when both a header record and a margin record apply to the same page. Normalized constants are compared within `4 * f64::EPSILON`, with exact equality of rounded pixel coordinates at the approved raster dimensions. That allowance handles JSON floating-point parsing without accepting a pixel-boundary change.

### Verification already completed

For current code at `36a200b`:

- `cargo fmt --all` and `cargo fmt --all --check`: pass.
- `cargo clippy --offline --all-targets -- -D warnings`: pass.
- `cargo test --offline`: **364 library tests and 9 binary tests pass**, with 32 library tests ignored. Log: `tmp/approved-masks-full-tests.log`.
- `cargo test --offline --release --lib inner_tomi_known_masks_preserve_latin -- --ignored --nocapture`: pass for all approved masks. Log: `tmp/check-approved-inner-tomi-masks.log`.
- `git diff --check`: pass.

The mask source test proves that every pixel outside the union of approved rectangles is byte-identical to the raw source. It does **not** prove that final extraction selected every Latin column or that final composition has the correct spacing; that is the remaining work.

The first-stage source regressions and production composition diagnostic passed before the thirteen additional masks. They must be rerun for final acceptance. The source-mask test above is the exception: it has already been rerun on current code.

**Existing `build_facsimile/` PDFs and `target/release/facsimile.exe` are stale.** They predate the additional masks. Building release library tests does not rebuild the production binary. Use Cargo to rebuild the binary before final generation; do not invoke the old executable directly.

## Desired end state

All four remaining ignored regressions exist and pass against current production extraction/composition. Existing source regressions and the ordinary Rust suite remain green. All eight inner and eight imposed PDFs are freshly generated from current code and inspected, with source Latin restored at the severe-gap locations, whole lines across the Timoleon seam, no furniture-only body canvases, correct imposition, and unchanged retained chapter spacing. The verification note and plan describe final evidence accurately, and the work is committed and pushed.

No further approval interview is necessary for the recorded repairs. The user said **“I approve all.”** Continue the authorized verification and rebuild. A newly discovered need to erase different source content is outside the frozen rectangle set and requires its own evidence and decision.

## Remaining implementation and verification

### 1. Add the four composition regressions

Implement these ignored real-source tests in `src/facsimile/reflow.rs`:

- `inner_tomi_watermark_cleanup_removes_pixels_and_space`
- `inner_tomi_severe_gaps_preserve_all_lines`
- `timoleon_reported_page_break_preserves_whole_lines`
- `timoleon_and_brutus_have_no_furniture_only_body_canvases`

Reuse `RetainedSourcePageTrace`, `map_row`, `retained_page`, existing production extraction/composition, `assert_scaled_stream`, and the shared ledger. Inspect `inner_tomi_post_mask_composition_evidence` and `verify_real_composition` before introducing helpers. Do not clone the bander or packer, introduce a generic whitespace framework, alter thresholds, or change unit boundaries/pins.

The measured production global scale is **1.2023952007293701** (`f32` display 1.2023952). `src/facsimile/mod.rs::global_scale_for` samples raw rendered columns over the same unit ranges independently of masks and duplicate exclusions. Preserve its algorithm and input ranges. Use the actual global scale for acceptance; a unit-local scale cannot prove the reported pagination repair.

Required claims and evidence:

- W1/W2: source 0 pages 287/299 previously copied through rows 2940/2922. Current masked zone banding ends at **2575/2573**. Verify full-unit copied endpoints and composed seams; do not substitute these zone measurements for final unit heights.
- G1/G2/G3: detector selection had omitted the Latin columns entirely. Current restored crops are source 0 page 296 **940–1727**, page 304 **917–1709**, and source 1 page 625 **980–1767**. Account for every retained Latin band and pixel, including adjacent joins: page 304 `...ad saxum quod-` → page 305 `dam sublime...`; source 1 page 624 `subli-` → page 625 `dens...` → page 626 `partem secutus`.
- G6: source 0 page 470's former copied tail extended through row 2782. The approved margin removes connecting edge ink. Verify the actual Pyrrhus opening trim and page 470→471 sentence spacing. Raw zone bands include the display title; full-unit expectations differ because opening trims apply.
- O8B: source 0 page 306 previously had a tall merged band 333–441 crossing a canvas seam. The candidate upstream margin correction restored whole-line banding. Verify each actual original line interval survives intact on one composed canvas, as well as composed pixel continuity. Resampling alone must not conceal a line split.
- Timoleon/Brutus: demonstrate that removed rule/edge-only regions yield no furniture-only body strips or canvases while all Latin survives. Explain any blank recto separators separately from body canvases.
- Source 1 page 347: verify the additional header removal in final output. The old VI329 finding belongs to page 347; the original page 337 repair appears around VI308–310. Do not conflate the two occurrences.

The ledger includes independently saved candidate full-unit bands, first/last bands, crop edges, adjacent prose, and preservation boundaries for the new margin records. Keep those frozen. Some candidate crop edges reflect a margin-only preview: applying an existing header mask as well changes the left crop by one pixel on pages 281/298 (full-mask values 1069/949). Diagnose scope differences explicitly; do not overwrite expected data merely to make a test pass.

### 2. Rerun source and Rust checks

Run each new test using the command pattern below, and rerun the existing selected source tests listed in the verification note:

```powershell
cargo test --offline --release --lib <test_name> -- --ignored --nocapture
```

The selected existing tests are:

```text
inner_tomi_duplicate_source_trace
inner_tomi_timoleon_coriolanus_source_trace
inner_tomi_remaining_source_trace
eumenes_comparison_prose_is_complete_and_unique
agis_repeated_passage_survives_once
inner_tomi_known_masks_preserve_latin
marius_terminal_furniture_emits_no_body_canvas
reported_divider_pages_preserve_latin
reported_watermark_pages_preserve_adjacent_lines
fabius_shared_page_header_is_excluded_from_both_units
coriolanus_source_284_header_is_excluded
otho_final_source_pages_survive_reflow_once
cimon_lucullus_final_source_page_survives_reflow
inner_tomi_post_mask_composition_evidence
```

The composition evidence diagnostic passed first-stage for thirteen units and checks their entire composed pixel streams against the concatenated resized retained strips, respecting canvas-width clipping. It took about 229 seconds. Its saved evidence is stale until rerun with current masks. Run Cargo commands sequentially to avoid Windows executable locking.

Before declaring implementation done:

```powershell
cargo fmt --all
cargo fmt --all --check
cargo clippy --offline --all-targets -- -D warnings
cargo test --offline
cargo build --offline --release --bin facsimile
git diff --check
```

### 3. Rebuild and visually verify all sixteen PDFs

Use these PowerShell environment settings for tests and the build:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
$env:PLUTARCH_PDFTOTEXT = 'C:/Users/majad/Downloads/poppler/poppler-25.12.0/Library/bin/pdftotext.exe'
$env:PLUTARCH_PDFTOPPM = 'C:/Users/majad/Downloads/poppler/poppler-25.12.0/Library/bin/pdftoppm.exe'
$env:PLUTARCH_PDFINFO = 'C:/Users/majad/Downloads/poppler/poppler-25.12.0/Library/bin/pdfinfo.exe'
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA/Programs/MiKTeX/miktex/bin/x64/lualatex.exe"

cargo run --offline --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
```

The full first-stage build succeeded with MiKTeX LuaHBTeX 1.24.0 / MiKTeX 25.12. In this environment, MiKTeX installation/cache access required `sandbox_permissions: require_escalated`; a sandboxed attempt reported `lualatex not run`. Request tool escalation for the authorized build when necessary, and inspect exit status/logs and resulting PDF freshness. Git writes/push also required tool escalation. Do not treat cached PDFs as evidence of a successful new compilation.

First-stage physical PDF counts, for reference only:

| Tomus | Inner pages | Imposed pages |
| --- | ---: | ---: |
| I | 416 | 208 |
| II | 448 | 224 |
| III | 416 | 208 |
| IV | 128 | 64 |
| V | 384 | 192 |
| VI | 384 | 192 |
| VII | 448 | 224 |
| VIII | 320 | 160 |

Do not assume final counts or locators stay unchanged. Locate passages through source landmarks and fresh composition traces. Confirm wrapper `innercount` values against `pdfinfo`; inspect `\imposedsheetside{...}{...}{...}{...}` mappings for each inner page exactly once and correct 32-page signature order. Render affected inner pages and imposed sheets, unchanged samples, and relevant IV/VIII endings. Inspect horizontal clipping at the unchanged global scale.

First-stage locator anchors include D1 join on V195, D2 join on VII20, Marius ending on III297 with an unnumbered blank recto separator III298, G1 around II219–220, G2 around II235–238, O8B around II240–241, G6 around III175, and G3 around VIII130–132. These are historical physical pages, not guaranteed final locations.

### 4. Record acceptance and commit

Update the plan, verification note, and relevant decisions in place. Keep existing numbered identifiers and retain historical artifacts. State which evidence is final and which is first-stage. Code comments must describe present behavior and must not cite plan phase/step identifiers. Commit and push verified work on the existing branch. Do not declare the original plan complete until the four tests and fresh PDF acceptance are finished.

## Local evidence and operational details

| Path | Contents and freshness |
| --- | --- |
| `tmp/pdfs/inner_tomi_repairs/masks/` | Current raw/masked page images and `measurements.json`; all approved masks are applied. Zone band measurements are not final unit composition. |
| `tmp/pdfs/inner_tomi_repairs/post_mask/composition.json` and PNGs | First-stage thirteen-unit production traces and canvases; rerun diagnostic for current evidence. |
| `tmp/pdfs/inner_tomi_repairs/candidate_rules/` | Independent approved margin previews. `tmp/preview-rule-candidates.log` holds saved candidate full-unit bands used in ledger expectations. |
| `tmp/pdfs/inner_tomi_repairs/rebuilt/` | First-stage rendered output and source-347 header evidence. |
| `tmp/inner-tomi-rebuild-compiled.log` | Successful first-stage all-sixteen-PDF build. `tmp/inner-tomi-rebuild.log` is the earlier uncompiled attempt. |
| `tmp/tomus-N-rebuilt-text.txt` | First-stage extracted PDF text; do not use as final output. |
| `tmp/approved-masks-full-tests.log` | Current ordinary test checkpoint. |
| `tmp/check-approved-inner-tomi-masks.log` | Current exact outside-mask pixel preservation proof. |

Source indices are **0 = `Ploutarchou_Bioi_v1.pdf`**, **1 = `Ploutarchou_Bioi_v2.pdf`**. Source pages and inner PDF pages are one-based physical pages; printed folios are different coordinates. Approved mask bounds are normalized half-open rectangles on the full 300-DPI source raster.

Use `Get-Content -Encoding utf8` for artifacts and `rg` for searches. Preserve raw render caches and transcription caches. Stay offline: no paid vision, `--retag`, `--review-lines`, new dependencies, or new CLI flags. Temporary diagnostic executables may be stale; inspect and rebuild before trusting them. In PowerShell, parenthesize arithmetic inside arrays when constructing page tuples; an earlier locator script produced an off-by-one error with unparenthesized expressions, which was corrected in the saved ledger locators.

The next useful action is to inspect the current composition helpers and frozen band expectations, then implement the four remaining tests. The repairs and approvals are already available; do not restart diagnosis.
