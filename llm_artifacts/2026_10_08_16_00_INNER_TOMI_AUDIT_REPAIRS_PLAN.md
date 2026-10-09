# Context

A second, deeper audit of the rebuilt `build_facsimile/tomus_1_inner.pdf`
through `tomus_8_inner.pdf` is synthesized in
[`2026_10_08_14_56_INNER_TOMI_AUDIT_SYNTHESIS_FINDINGS.md`](2026_10_08_14_56_INNER_TOMI_AUDIT_SYNTHESIS_FINDINGS.md),
which combines the human visual reinspection
([`2026_10_06_15_40_INNER_TOMI_REINSPECTION_FINDINGS.md`](2026_10_06_15_40_INNER_TOMI_REINSPECTION_FINDINGS.md))
with the deterministic Python audit. It records finding families beyond the
already-repaired D1–D3 / W1–W3 / O-series set: nine further repeated-scan blocks
(**D4–D12**), distinct Latin omissions (**C1/C2/C3/C5** confirmed by eye, expanded by
the script), three more body watermarks (**W4–W6**), running-head/folio leaks
(**H1, H2**, plus two new ones), lateral clipping and line slices (C4/C6/C7,
S1/S2), and the exploratory margin-gap / divider-rule catalogs (M1–M11, R1–R14).

This plan repairs those finite, confirmed examples with the **same small,
edition-specific mechanisms** the previous two passes established — furniture
masks on the owned source raster and whole-page duplicate exclusion — plus a
targeted diagnosis of the one genuinely new failure mode: **lines of Latin that
disappear** (not just furniture that survives). It deliberately avoids global
threshold retuning, new CLI switches, paid vision/OCR, and any generalized
override framework. The earlier contracts
([source artifact cleanup](2026_10_05_FACSIMILE_SOURCE_ARTIFACT_CLEANUP_PLAN.md),
[inner-tomi minimal repairs](2026_10_05_INNER_TOMI_MINIMAL_REPAIRS_PLAN.md))
supply the implementation patterns; their locators and masks remain valid and
must be preserved.

This document is an implementation contract, not an implementation. All Latin is
source material: preserve every distinct genuine Latin line, its glyphs,
spelling, punctuation, capitalization, ligatures, and reading order. Erasing
source furniture and excluding a proven repeated scan are image/selection
corrections; they are never permission to retype, normalize, reconstruct, or
OCR-replace prose. **Removing a watermark is successful only when both its pixels
and the copied blank space it leaves behind disappear.** A doubtful mask, a
doubtful exclusion, and a doubtful omission "fix" each stay disabled until source
inspection proves it preserves the prose; a retained ambiguous artifact is
acceptable and must be explained.

Work proceeds on branch `plan/inner-tomi-audit-repairs`.

## A note on detector confidence, adjudication, and repair eligibility

The human sweep confirmed four distinct omissions totalling eight source Latin
lines (C1 Romulus, C2 Coriolanus, C3 Phocion, C5 Brutus). The frozen script run
contains 77 omission candidates. Independent adjudication confirms **59 unique
losses / 113 lines across 31 units** and rejects 18 nominations. Of the confirmed
events, 37 have high detector confidence and 22 have low detector confidence.
Candidate-selected precision (0.766) is not a corpus recall rate.

Detector confidence, review verdict, and repair eligibility are separate fields.
Join `candidates.json` and `reviews.json` by `event_id`; include an omission in
the repair inventory only when `type == "missing_occurrence"`,
`unique_loss == true`, and `verdict == "confirmed"`. Never infer a verdict from
confidence. The run has 38 high-confidence omission nominations: one,
`miss_tiberius_gracchus_403_690`, is adjudicated `false_positive` because its
source line already survives. It must remain excluded from repair.

The initial, disjoint eligibility tiers are:

| Tier | Membership | Events | Source lines |
| --- | --- | ---: | ---: |
| A | The 37 confirmed high-confidence events, plus human-confirmed C1 and C2 | 39 | 69 |
| B | The remaining confirmed low-confidence events | 20 | 44 |
| Total | Every confirmed unique-loss event, counted once | 59 | 113 |

- **Tier A:** repair-eligible after source-pixel diagnosis. C1
  (`miss_romulus_38_2595`) and C2 (`miss_marcius_coriolanus_292_10538`) retain
  their original `confidence: low` but receive Tier-A precedence because of the
  human findings. C3 and C5 are already among the 37 high-confidence events.
  Every Tier-A event, including multi-line losses, must be restored before this
  plan is complete; an unsafe or unexplained fix leaves that event unresolved.
- **Tier B:** record the existing confirmed verdict and a repair-specific
  confirming-read status. No repair is enabled until a human confirms the exact
  source lines and proposed preservation boundary and promotes the event to
  Tier A. The 23-line Antonius event on v2/562 starts in this tier. Promotion
  changes eligibility, never detector confidence or historical adjudication.

The authoritative run is
`tmp/pdfs/inner_tomi_audit/runs/step102-final-all-5/`. Bind both files:

- `candidates.json` SHA-256:
  `03f31fc68aa8829b90cdf3462578d9978b05d01a5a9c895416333dd4f2c78144`.
- `reviews.json` SHA-256:
  `339ac5a1663bf5ea577a293d1d22aaa412b567a082e66806943c110b126b57ec`.

The durable omission inventory at the end of this plan preserves all 59 confirmed
event IDs, exact normalized source regions, line counts, confidence, verdict,
initial tier, and candidate/review crop paths and hashes. Step 1.1 imports that
inventory into the diagnostic fixture and validates it against the two frozen
files, failing if neither the `tmp/` originals nor the step 1.0 archive copies
are readable. It does not import OCR as replacement Latin.

**Historical evidence recovery.** Step 1.0 preserves the run, the earlier
run folders referenced by `reviews.json`, `snapshots/baseline_2026_10_06/`,
`runs/step101-development-revised-1/freeze.json`, and the working `spec/` files
in an archival location before any diagnosis or repair. Step 1.1 records that
location and the archive manifest hashes in the diagnostic fixture. The completion note
[`2026_10_08_14_44_INNER_TOMI_AUDIT_STEP_102_COMPLETION.md`](2026_10_08_14_44_INNER_TOMI_AUDIT_STEP_102_COMPLETION.md)
records the exact producer, baseline, freeze, specification, input, and tool
identities. Reproduction requires all of those identities, including the frozen
`eng` OCR profile; changed reader PDFs must not be captured as the old baseline.

When the captured baseline, matching development freeze, working specifications,
and recorded tool versions have been recovered, the original detector command is:

```powershell
python -u tools/audit_inner_tomi.py run --baseline baseline_2026_10_06 --scope all --detectors all --freeze tmp/pdfs/inner_tomi_audit/runs/step101-development-revised-1/freeze.json --lang eng --workers 4 --run-dir tmp/pdfs/inner_tomi_audit/runs/step102-final-all-5
if ($LASTEXITCODE -ne 0) { throw 'Historical audit reproduction failed' }
Get-FileHash tmp/pdfs/inner_tomi_audit/runs/step102-final-all-5/candidates.json -Algorithm SHA256
Get-FileHash tmp/pdfs/inner_tomi_audit/runs/step102-final-all-5/reviews.json -Algorithm SHA256
```

The detector command does not recreate independent adjudications. Restore
`reviews.json` and all evidence it references separately, then verify both file
hashes and every referenced crop hash. A candidate rerun alone cannot establish
confirmed status. If historical files cannot be recovered, use the durable
inventory to identify the source regions, record the missing evidence explicitly,
and obtain a fresh human source/reader comparison and preservation review for
each affected repair. Preserve the old verdict and bind fresh evidence to its
actual inputs; do not claim historical reproduction or enable an unverified fix.
Other repairs whose evidence is available may proceed.

# Current state of the world

- `src/facsimile/reflow.rs`
  - `BODY_DPI = 300`. `SourceFurnitureMask`, `SOURCE_FURNITURE_MASKS` (the frozen
    furniture table from the prior two passes), `apply_source_furniture_mask`, and
    `mask_known_source_furniture(source, page, img)` paint verified normalized
    rectangles white on an **owned** source image inside `render_and_band`, before
    column detection and banding. Raw render-cache rasters stay pristine.
  - `skip_known_duplicate_source_page(unit_id, source, page) -> bool` excludes
    reviewed whole-page repetitions. It currently matches
    `comparatio_sertorii_et_eumenis` v2 103|104 (D1), `agis` v2 369|370 (D2), and
    `cato_minor` v2 339..=348 (D3). It is consulted in `unit_body_strips`'s Pass A
    loop **before** a page is rendered and pushed to the parallel `pages` /
    `source_pages` vectors, so excluded pages never enter metrics, banding, or
    composition. It must return false for every unrelated identity.
  - `unit_body_strips` renders each source page of a unit's range, keeping a
    parallel `source_pages` vector. Pass A bands every kept page via
    `render_and_band`; `line_metrics` summarizes body lines; Pass B applies
    `drop_opening_bands` (first page), `drop_interior_running_head` (interior
    pages), and `drop_shared_tail` (a shared last page). `PageBands` is
    `(GrayImage, Vec<Band>)`.
  - `render_and_band` = `render_fn` → `mask_known_source_furniture` →
    `detect_latin_column` → `crop_column` → `line_bands` → `drop_furniture` →
    `retain_early_aligned_body_bands`.
  - The furniture heuristics that can **drop real body lines** when a page's
    geometry is disturbed: `drop_furniture` (zone + gap-outlier),
    `drop_interior_running_head` → `drop_trailing_furniture` →
    `detached_tail_cluster_start` / `drop_detached_rule`. `detached_tail_cluster_start`
    removes a trailing run of bands after an above-median gap when **every** band
    in the run has ink-span `< BODY_LINE_SPAN_FRAC (0.8) × column width` and at
    least one is `Short`/`Tall`. A body line whose left glyphs are laterally
    clipped is narrower than the measure, so clipping can make real lines look
    like a detached furniture cluster — a plausible shared cause of the
    co-located clipping+omission findings (see Phase 1).
  - `strips_in_reading_order` builds `Strip`s from `cut_points` (gap-midpoint
    interior breaks) and drops strips below `minimum_ink`. `compose_pages` scales
    by one global scale and packs; an ordinary line that fits a full canvas wraps
    whole, a `Tall` (merged) strip may be fill-split at a low-ink row, and a strip
    exceeding a fresh canvas budget is split regardless of classification.
    Composition also removes canvases below its scaled `minimum_ink` floor.
    Trace all of these stages before attributing S2 or a loss to a merged band.
  - Test module (ignored, real-source): `operational_range(id)`,
    `real_source_image(source, page)` (uses `build_facsimile/render_cache` at
    `BODY_DPI`), `traced_real_unit` (asserts **no** skipped pages and indexes
    `page - start_page`), `RetainedSourcePageTrace` with `map_row` (the
    skip/deletion-aware mapping added by the previous pass), `production_body_scale`
    (samples `global_scale_for` over **unmasked** rasters, so masks/exclusions
    leave the global scale unchanged), `assert_scaled_stream`, and
    `verify_real_composition`. `strip_and_canvas_ranges` maps extracted strips
    and composed canvases into one scaled row stream. The existing
    `inner_tomi_known_masks_preserve_latin` and
    `inner_tomi_watermark_cleanup_removes_pixels_and_space` regressions supply
    shared mask and watermark verification. The reviewed numeric ledgers are
    `src/facsimile/fixtures/artifact_source_trace.json`,
    `inner_tomi_repair_source_trace.json`, and `closing_body_bands.json`.
- `src/facsimile/columns.rs`: `detect_latin_column` → `exclude_left_ticks` →
  `exclude_isolated_left_divider`. The divider exclusion is deliberately
  conservative (an 8-connected component ≤ 12 px wide, isolated by ≥ 8 white
  columns, ≥ 3 retained text bands). Do **not** widen these gates to force new
  examples through; a stubborn left rule is handled by a verified local furniture
  mask instead (the established pattern).
- `src/facsimile/lines.rs`: `line_bands`, `drop_furniture`, `cut_points`. The
  interior cut is the midpoint of the inter-band gap, so simply removing an
  interior furniture band still distributes its blank interval into the adjacent
  strips — the reason tail watermarks (below the last band) are cheap to fix but
  interior gaps are not.
- `src/facsimile/units.rs`: `Unit`, `UnitRange`, `load_units`, `ranges`.
  `facsimile_units.json` carries `start_page`, `title_top_frac`, `body_top_frac`,
  `skip_top_bands`, `last_page`, `pinned`. `ranges` derives an interior unit's end
  from the next same-source opening (a `shared_tail` page); the final unit of a
  source uses `last_page`. `body_top_frac` and `skip_top_bands` affect only the
  owning unit's first source page. `title_top_frac` on the following unit supplies
  the preceding unit's shared-tail cut. None of these fields controls an
  interior page's head trim. `detect::merge_pins` preserves `pinned` rows across
  regeneration.
- Builds: `cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf
  --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline`, using Poppler,
  LuaLaTeX, and the offline front-matter caches. Composed PNGs are removed after a
  successful compile; diagnostics must save their own crops under `tmp/`.

## Findings mapped to source identity and mechanism

Confirmed unit source indices and interior ranges (0 = v1, 1 = v2), from
`facsimile_units.json`: all nine D4–D12 duplicate blocks are in v2.

### Repeated scans (D4–D12) — exclude the second occurrence, keep the first

Each block scans the same printed folios twice; the **first** occurrence is
complete and the **second** is equal-or-worse (D5/D6/D12 second openings are
incomplete and/or laterally clipped). The human report's "hypothetical source
join" table already verified, in full Latin-column strips, that dropping the
second block connects the expected prose with no lost distinct source block.

| ID | Unit (v2) | Unit range | First occ. (keep) | Second occ. (exclude) | Join after exclusion | Co-resolves |
| --- | --- | --- | --- | --- | --- | --- |
| D4 | agesilaus | 106–137 | 115–118 | **119–122** | 118→123 | — |
| D5 | alexander | 192–245 | 211–212 | **213–214** | 212→215 | **C4, S1** (both on v2/213) |
| D6 | phocion | 286–309 | 295–296 | **297–298** | 296→299 | R10 |
| D7 | cato_minor | 309–363 | 359–360 | **361–362** | 360→363 | R14 nbhd / D7 note |
| D8 | agis | 363–379 | 375–376 | **377–378** | 376→379 | R9 (first occ.) |
| D9 | cicero | 446–478 | 463–464 | **465–466** | 464→467 | — |
| D10 | antonius | 513–563 | 521–522 | **523–524** | 522→525 | — |
| D11 | aratus | 647–680 | 671–672 | **673–674** | 672→675 | R11 |
| D12 | otho | 696–708 | 699–700 | **701–702** | 700→703 | **C7, R12** (on v2/701) |

Mechanism and fix: identical to D1–D3. Extend `skip_known_duplicate_source_page`
with the second-occurrence source pages. The excluded pages are mid-unit (not
unit boundaries), so no unit-table or pin change is needed. **Caveat to verify in
Phase 1:** a line that exists *only* in the second occurrence must never be
discarded. The reports state every second-occurrence opening line survives in the
first occurrence. C3 (Phocion v2/299) is a separate omission following D6;
C5 (Brutus v2/624) follows the W6 watermark page, not a listed duplicate block.
Both omissions require their own Phase 4 verification.

### Watermarks (W4–W6) and their copied space

| ID | Unit | Source | Position | Notes |
| --- | --- | --- | --- | --- |
| W4 | aemilius_paulus | v1 p321 | bottom margin (split across output 273–274) | heavy right rule R6 runs through it |
| W5 | brutus | v2 p603 | bottom margin (before p604) | right rule R13; seam M10 |
| W6 | brutus | v2 p623 | bottom margin (before p624) | right rule R13; seam M11; **C5 omission on p624 is separate** |

Mechanism and fix: identical to W1/W2. All three are **source-page tails** —
furniture lying *below the last Latin band of a source page*, as opposed to an
**interior gap** between two body lines. The distinction is load-bearing: a tail
is cheap to remove because `cut_points` ends the final strip at the last body
band, so clearing the tail copies **zero** padding rows; an interior gap is not,
because its blank is split into the midpoints of the two adjacent strips (removing
the furniture band alone still distributes the blank). Mask the watermark
rectangle **and** the connecting right-edge rule/scan-noise (as the p233/p235
precedent proved, the watermark otherwise stays merged into the last body band and
its space is copied). The space then disappears with the pixels; Phase 3.2 proves
this through the real extraction path.

### Running-head / folio leaks (H1, H2, two new)

| ID | Unit | Source | Leak |
| --- | --- | --- | --- |
| H1 | philopoemen | v1 p448 | clipped head `…LOPŒMEN.` + folio `435` + Greek scraps |
| H2 | agis | v2 p375 | Greek head fragment `Σ.` (beside R9), in the **retained** first Agis occ. |
| new | caius_marius | v1 p528 | clipped running-head suffix (`IUS` fragment) |
| new | cato_major | v1 p432 | source folio numeral `419` leaked into the body |

Mechanism and fix: identical to the many existing header/folio masks — a small
rectangle on the owned raster above the first body line. H2 lives on a page we
keep (v2/375), so it is a genuine mask, not resolved by de-duplication.

### Lateral clipping and slices (C4/C6/C7, S1/S2)

- **C4, S1 (Alexander v2/213)** and **C7, R12 (Otho v2/701)** are on
  second-occurrence pages and are **auto-resolved by D5/D12 exclusion** — confirm
  in Phase 1, add no separate mask.
- **C6 (Romulus v1/39, with v1/38)** is left-glyph clipping from column
  detection, co-located with omission **C1** and left rule **R3**; see below.
- **S2 (the `comparatio_demosthenis_et_ciceronis` unit, v2/480, output
  VII/244–245)** is a line sliced across a page boundary — a merged/`Tall`
  band fill-split by the packer. Fix by
  restoring separate line bands upstream (the O8B precedent: a margin mask on the
  connecting noise), not by changing the packer.
- The broad `lateral_clipping` screen (1,110 nominations) is a **review queue,
  not a defect list** and is out of scope for repair.

### Distinct Latin omissions (C1–C3, C5, high-confidence blocks) — the novel category

This is the one family with no established fix. Leading mechanism hypothesis, to
confirm or refute per-case in Phase 1:

> A left divider rule (R3-class) disturbs `detect_latin_column` /
> `exclude_isolated_left_divider` on a page, so the Latin crop starts too far
> right and clips left glyphs (the **C6-class clipping**). The clipped lines are
> now narrower than `0.8 × column width`. When such lines sit after an
> above-median **chapter gap** near the page foot, `detached_tail_cluster_start`
> (or `drop_trailing_furniture`) treats them as a detached narrow furniture
> cluster and **drops them** (the **C1-class omission**). One left-edge furniture
> mask that lets column detection find the true left margin would then restore
> full-width bands and resolve clipping and omission **together** — the same
> minimal mechanism already in use, no new structural code.

Co-located evidence supporting the hypothesis: C1 + C6 + R3 all on Romulus
v1/38–39; the losses are repeatedly described as sitting *before a chapter
heading* (C1 before chapter IX; iulius_caesar before chapter XIV; theseus section
VI). Phase 1 must confirm the exact mechanism from pixels before any fix; where a
loss is **not** a clipping/furniture artifact (e.g. a seam line genuinely dropped
by an opening/interior trim, as C3 on interior v2/299 may be), Phase 1 specifies
the narrow alternative appropriate to that stage: a verified mask, an opening
setting for an opening-page loss, or the following unit's title fraction for a
shared-tail loss. Interior-head, strip-filter, and final-canvas losses need a
specific correction to their responsible code if existing masks cannot restore
them. Record an unresolved Tier-A event and amend its implementation step with
exact behavior and tests before coding that correction; recording the limitation
does not satisfy the restoration requirement.

# Desired end state

1. The nine repeated blocks D4–D12 occur **once**, in correct reading order, with
   every distinct Latin line retained from the complete first occurrence and the
   verified source joins intact.
2. Watermarks W4–W6 and their connecting rules disappear **with** their copied
   blank space (no watermark replaced by an equally tall gap).
3. Leaked furniture H1, H2, the caius_marius and cato_major fragments, and the S2
   slice are gone; C4/C7/S1 are gone as a by-product of de-duplication.
4. Every **Tier-A** omission (initially 39 events / 69 source lines, including
   C1/C2/C3/C5 and every confirmed high-confidence event) is restored: all missing
   Latin lines appear once, in order, with their neighbors and split-word
   continuations intact. A Tier-A event without a safe specified fix stays
   unresolved and prevents declaring this plan complete. Every Tier-B event
   (initially 20 events / 44 lines) has a disposition and stays unrepaired unless
   promoted by a confirming human read; promoted events join the completion gate.
5. Ambiguous dividers/rules that cannot be masked without risking Latin are
   retained with a documented reason (the M/R catalog is not an exhaustive
   cleanup target). Genuine printed chapter/verse spacing (the non-findings table,
   I/28, I/36, I/46, …) is preserved. Confirmed Greek scraps F1 (Theseus v1/17)
   and F2 (Pyrrhus v1/470) receive explicit per-case mask or defer decisions, with
   Latin and legitimate edition-reference letters preserved.
6. Existing successful repairs (D1–D3, W1–W3, the O-series masks), complete
   terminal endings (E01–E68), the global body scale, unit structure, the Google
   notice page, the Praefatio transition, and offline behavior all remain valid.
7. No paid vision/OCR, no new CLI options, no new dependencies, no global
   threshold retuning, no generalized override framework. Raw render caches,
   source PDFs, and transcription caches are untouched.
8. All local Rust checks pass, all eight inner and eight imposed PDFs are rebuilt
   from current source and verified, and a durable verification note records
   source mappings, before/after prose landmarks, before/after copied heights for
   each watermark, and every retained-artifact decision.

# Phase 1 — Diagnose each finding and freeze regression evidence (Suggested implementer: Opus)

No production behavior changes in this phase. Reuse the existing ignored-test
infrastructure (`operational_range`, `real_source_image`, `render_and_band`,
`unit_body_strips`, strip extraction, composition, `RetainedSourcePageTrace`,
`production_body_scale`, `verify_real_composition`); do not add a second bander,
renderer, packer, or a production tracing subsystem. Save full-resolution
source/output crops, raw/retained band intervals, crop edges, strip intervals,
candidate-mask results, hypothetical raw-source exclusion joins, and seam crops under
`tmp/pdfs/inner_tomi_audit_repairs/`.

**Where reviewed coordinates live.** The existing regression harness reads
`repair_ledger()`, i.e.
`src/facsimile/fixtures/inner_tomi_repair_source_trace.json`. That ledger records
installed repairs: its ordinary identity and rectangle tests require every
`duplicate_page` and `mask` entry to match production behavior immediately.

Phase 1 must leave that ledger unchanged. Freeze diagnosis, proposed masks and
exclusions, omission traces, archival provenance, and dispositions in
`src/facsimile/fixtures/inner_tomi_audit_repair_source_trace.json`. In Phase 2,
append each D4–D12 record to the installed ledger in the same step that installs
its predicate match. In Phases 3 and 4, append a mask record in the same step that
installs its rectangles. Never alter existing records. Tests in a step must not
require a later phase's behavior.

Use one owner for each kind of evidence: the diagnostic fixture owns baseline
traces, historical candidate identities, and proposed decisions; the installed
ledger owns enabled rectangles, skip blocks, and frozen repair expectations.
After installation, the diagnostic entry references the installed record by
`finding_ids`, `unit_id`, `source`, and `source_page`; it must not maintain a
second authoritative copy of the installed rectangles. Preserve original
diagnostic evidence as historical measurements. Locate prose by physical PDF
pages and exact source context; OCR locates, it never proves.

## 1.0 Archive the historical audit evidence

Perform the historical-evidence preservation described in the Context section
before any diagnosis or repair. Copy
`tmp/pdfs/inner_tomi_audit/runs/step102-final-all-5/`, every run folder that
`reviews.json` evidence paths reference (at minimum
`runs/step92-development-final-2/`, `runs/step101-validation-first/`, and
`runs/step101-validation-expanded-review/`), `snapshots/baseline_2026_10_06/`,
`runs/step101-development-revised-1/freeze.json`, and the working `spec/`
files into `archive/inner_tomi_audit_2026_10_08/` at the repository root. An
acceptable archival location is any durable directory outside `tmp/` and
outside build outputs; this default keeps the evidence beside the repository
without committing large binary crops — do not stage the archive in git.
Write a manifest inside the archive listing every file's relative path and
SHA-256; step 1.1 imports the archive location and manifest hashes into the
diagnostic fixture. If a referenced evidence file is already missing, list the
gap in the manifest explicitly and follow the fresh-confirming-read path from
the Context section; do not synthesize a replacement.

**LLM verification:** `Get-FileHash` over the archived `candidates.json` and
`reviews.json` matches the two SHA-256 values bound in the Context section; a
fresh hash pass over the archive matches the manifest; `git status --short`
shows no archive content staged.

**Human verification:** Confirm the archival location is durable enough (backed
up, or at least outside anything a build or `tmp/` cleanup touches) and that
any recorded evidence gaps match reality.

## 1.1 Shared support and the repair inventory

In the `reflow.rs` test module, add three ignored entry points:
`audit_duplicate_blocks_source_trace`, `audit_watermark_and_head_source_trace`,
and `audit_omission_source_trace`. Add ordinary
`audit_omission_inventory_matches_frozen_review` to validate the committed
inventory. That test checks internal consistency (event counts, the disjoint
tier partition, unique IDs, the excluded false positive) and verifies the
frozen `candidates.json` and `reviews.json` SHA-256 hashes against the
fixture, reading either the original `tmp/` run path or the step 1.0 archive;
if neither copy is readable it fails with an explicit missing-evidence error
rather than passing silently. (Both files are present on the development
machine and their hashes match the bound values, so the test passes there.)
Reuse
`repair_trace_maps_skipped_pages_and_deleted_rows` for the
existing row-mapping contract. Prefer `RetainedSourcePageTrace`/`map_row` for
units that will skip pages. Never derive their retained index from
`page - start_page`.

Import the durable omission inventory below into the diagnostic fixture, carrying
both audit file hashes, unique event IDs, source regions and source input hashes,
`missing_line_count`, detector confidence, review verdict, initial eligibility,
and candidate/review crop identities. Import the step 1.0 archive location and
its manifest hashes alongside the inventory. Validate a disjoint initial partition of
39 Tier-A events / 69 lines and 20 Tier-B events / 44 lines. Require the complete
59-event / 113-line set; reject duplicate IDs, a missing confirmed event, an
unreviewed nomination, and `miss_tiberius_gracchus_403_690`. Add ordinary
`audit_omission_inventory_rejects_false_positive_and_duplicate_events`, using
one shared validator and small copies of the committed inventory with deliberate
invalid entries. Promotions are additional reviewed fields; they preserve the
frozen initial inventory.

For diagnosis records, use `status: proposed | awaiting_post_dedup_trace |
unresolved | installed | deferred | non_finding`, with a responsible step and
preservation/evidence reason. Tier-A omissions may be proposed, awaiting a trace,
unresolved, or installed; they cannot be deferred to satisfy completion.
Tier-B and optional furniture cases may be deferred. Store production raster
dimensions, original crop edges, original-source band/line intervals, exact
adjacent prose landmarks, and the proposed correction appropriate to the dropping
stage. Never fabricate production pixel coordinates from an OCR bounding box.

Add one test-only `assert_reviewed_source_lines_survive` helper shared by
duplicate, restored-omission, and slice regressions. Its inputs are a reviewed
list of source page identities and half-open original-source line/glyph regions,
the retained page traces, pages, and metrics. It checks each source region is
covered by retained bands and exactly one extracted strip, compares its
unmasked Latin pixels before scaling through `map_row`, verifies source order,
and uses `strip_and_canvas_ranges` plus `assert_scaled_stream` at
`production_body_scale()` to prove preservation in returned canvases.
Assert whole-line canvas ownership when requested for S2; ordinary omission
checks require complete ordered preservation even if an unrelated merged strip
spans canvases. `map_row` maps source rows to retained images; use strip scaling
and canvas ranges for output coordinates, never compare raw pixels directly to
resampled pixels. Include all reviewed body regions from the retained repeated
occurrence to detect missing chapter progression, and seam landmarks to detect
bad joins. Render and compose once per unit and share those results across cases.

Add ordinary `reviewed_source_line_assertion_rejects_missing_and_duplicate_lines`
using a shared synthetic page/line fixture, the real extraction/composition path,
and cases with an absent region and a repeated region. Do not create a second
renderer, bander, packer, or row-stream mapper.

| Finding(s) | Source identity | Phase-1 question to resolve | Likely Phase |
| --- | --- | --- | --- |
| D4–D12 | v2 second-occurrence pages (table above) | Confirm first occ. complete; confirm each join preserves all distinct lines; confirm no unique line only in the second occ. | 2 |
| W4 | v1 p321 | Tail watermark + R6 right rule rectangles; confirm separable from last body band; measure removable tail rows | 3 |
| W5 | v2 p603 | Tail watermark + R13 rectangles; measure removable tail rows | 3 |
| W6 | v2 p623 | Tail watermark + R13 rectangles; measure removable tail rows (C5 is separate, on p624) | 3 |
| H1 | v1 p448 | Head/folio/Greek-scrap rectangle(s) above first body line | 3 |
| H2 | v2 p375 | Greek head fragment rectangle (retained first Agis occ.) | 3 |
| new MARIUS | v1 p528 | Clipped running-head suffix rectangle | 3 |
| new CATO MAJOR | v1 p432 | Leaked folio `419` rectangle | 3 |
| C4, S1 | v2 p213 | Confirm removed by D5 exclusion; add no mask | 2 (verify) |
| C7, R12 | v2 p701 | Confirm removed by D12 exclusion; add no mask | 2 (verify) |
| S2 | v2 p480 | Identify the actual strip classification and cut; specify a safe upstream mask if a merge is proven, otherwise record the mechanism for amendment | 4 |
| C1, C6 | v1 p38–39 | Confirm the clip→narrow→cluster-drop mechanism; specify the left-edge mask that restores full-width bands and both the dropped lines and clipped glyphs | 4 |
| C2 | v1 p291–292 | Account for the two dropped lines after `Numa de-`; classify mechanism; specify minimal fix | 4 |
| C3 | v2 p298–299; retained seam 296→299 after D6 | Diagnose baseline now and rerun in 2.1A; use a safe mask or a specified interior-head correction, never opening fractions | 4 |
| C5 | v2 p623–624 | Account for p624 first line (`L. Erat inter amicos…`); classify mechanism; specify minimal fix (independent of W6) | 4 |
| iulius_caesar block | v2 p252 | High-confidence ~12-line loss before chapter XIV; confirm mechanism and minimal fix | 4 (Tier A) |
| theseus block | v1 p15 | High-confidence ~7-line loss (section VI); confirm mechanism and minimal fix | 4 (Tier A) |
| other confirmed high-confidence events | durable inventory | Diagnose every single- and multi-line loss; specify a safe fix or record unresolved Tier-A work | 4 (Tier A) |
| additional high-confidence blocks | brutus v2/623 (3), demosthenes v2/437 (4) and 443 (2), eumenes v2/91 (2), solon v1/115 (2), tiberius_gracchus v2/414 (2) | Account for every listed source line, not one representative line; Brutus 623's top omission is separate from its W6 tail and C5 on 624 | 4 (Tier A) |
| **20 Tier-B events** incl. antonius v2 p562 (23) | durable inventory, excluding C1/C2 already in Tier A | Record historical confirmed verdict and repair-read status; enable no repair without confirming read and promotion | — |
| F1 | theseus v1 p17 | Diagnose Greek-column slivers; mask safely separable furniture or explicitly defer with preservation reason | 3 / record |
| F2 | pyrrhus v1 p470 | Diagnose Greek-column scraps; preserve Latin and legitimate b/c/d reference letters; mask or explicitly defer | 3 / record |
| M1–M11 | per table | Classify each as copied margin vs genuine spacing; most retain (prior pass did not select row compaction) | 5 (record) |
| R1–R14 | per table | Mask only where safely separable **and** verified; otherwise retain with reason; dedup resolves R9(2nd)/R10/R11/R12 | 3 / record |
| B1–B4 | — | Confirm still absent (no divider-only body pages); record non-finding | 5 (record) |

Fill **Measured repair decisions** with one disposition per finding and one
entry per omission event during 1.1A–1.1C. Record baseline diagnosis separately
from post-dedup confirmation in 2.1A. A reviewed duplicate decision may proceed
to Phase 2 while an omission awaits that checkpoint. An unresolved or guessed
correction cannot proceed to its own production step. Tests read reviewed
expectations from committed evidence; regression runs never regenerate and
accept their own expected values.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy
--all-targets -- -D warnings`; `cargo test audit_omission_inventory`;
`cargo test reviewed_source_line_assertion_rejects_missing_and_duplicate_lines`;
`cargo test repair_trace_maps_skipped_pages_and_deleted_rows`; `cargo test
facsimile::reflow`. Validate the fixture structure, exact initial counts, and
evidence hashes. Each diagnostic entry point must execute assertions; verify the
installed ledger is unchanged in this phase.

**Human verification:** Confirm the inventory covers every synthesis finding and
that the fixture distinguishes original-source coordinates from retained-image
coordinates.

## 1.1A Diagnose the nine duplicate blocks (D4–D12)

Drive `audit_duplicate_blocks_source_trace` over `agesilaus`, `alexander`,
`phocion`, `cato_minor`, `agis`, `cicero`, `antonius`, `aratus`, `otho`. For each
block: compare the two occurrences line by line at both seams; confirm the first
occurrence is complete; inspect the raw-source hypothetical post-exclusion join
(table above), without claiming post-dedup pipeline verification; and explicitly
check that **no line exists only in the second occurrence**. Freeze the body-line
regions covering the retained occurrence and both join landmarks. C3's missing
first line on 299 is recorded for Phase 4, not required to survive at Phase 2.
Record the second-occurrence source pages to exclude and confirm
C4/S1 (v2/213) and C7/R12 (v2/701) lie inside excluded pages. Note any furniture
that stays in the **retained** occurrence and must be handled in Phase 3: H2 (the
Greek head fragment on v2/375) is masked; R9 (the heavy right rules on v2/374–376,
beside Agis prose) is an explicit Phase-1 decision — mask it only if the pixels
prove it separable from Latin, otherwise retain it with a recorded reason.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy
--all-targets -- -D warnings`; `cargo test audit_duplicate_blocks_source_trace -- --ignored
--nocapture`; `cargo test facsimile::reflow`. Inspect saved full-resolution
evidence and selected-test counts.

**Human verification:** For each of D4–D12, compare both occurrences and both
seams; confirm every distinct line has a retained occurrence and that the second
openings of D5/D6/D12 add no unique line.

## 1.1B Diagnose watermarks, head leaks, and the slice

Drive `audit_watermark_and_head_source_trace` over `aemilius_paulus` (v1/321),
`brutus` (v2/603, v2/623), `philopoemen` (v1/448), `agis` (v2/375),
`caius_marius` (v1/528), `cato_major` (v1/432), and
`comparatio_demosthenis_et_ciceronis` (v2/480). For each watermark, confirm it
is a source-page tail below
the last body band; record the watermark rectangle, the connecting right-rule /
scan-noise rectangle, the final true body band, the removable tail interval, the
median ordinary body gap, and the number of furniture-only rows currently copied.
For **W4 (v1/321)** specifically, confirm no v1/321 body line sits *below* the
watermark before treating it as a tail: the human finding notes watermark material
touching the first line of output 274, so verify that line belongs to the next
source page (322), not to a 321 body line beneath the logo.
For each head leak, record the smallest rectangle above the first body line that
clears the furniture without touching the first glyphs. For S2, identify the
merged/`Tall` source band on v2/480 and the packer cut row, and specify the
upstream mask (connecting noise) that would restore separate whole line bands,
if that is the demonstrated cause. Otherwise record the actual failure mechanism
and an unresolved S2 correction. Include F1 (Theseus v1/17) and F2 (Pyrrhus
v1/470) in this entry point: inspect their foreign-column scraps and record one
explicit safe-mask or defer decision for each. Check the entire neighborhood,
including F2's already installed margin mask and legitimate reference letters.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy
--all-targets -- -D warnings`; `cargo test audit_watermark_and_head_source_trace -- --ignored
--nocapture`; `cargo test facsimile::reflow`.

**Human verification:** Inspect each watermark seam (pixels and the blank below
it), each header rectangle against its first body line, and the S2 line at full
resolution. Confirm every proposed rectangle contains furniture alone.

## 1.1C Diagnose the distinct Latin omissions

Drive `audit_omission_source_trace` over all 39 initial Tier-A events from the
durable inventory, including C1/C2/C3/C5 and every multi-line loss. Process each
unit once. Phase 1 records the **baseline pipeline**; cases affected by future
exclusions receive `awaiting_post_dedup_trace` and are finalized in 2.1A.
Do not simulate exclusion by throwing render errors or copy the banding pipeline.

For each event, record exact source line/glyph regions with prose landmarks, raw
bands, detected crop edges, gap structure, and the first stage where any required
region disappears or becomes clipped. Trace source rendering/detection failures,
`line_bands`, `drop_furniture` zone and gap filtering,
`retain_early_aligned_body_bands`, the opening/interior/shared-tail Pass B trims,
`minimum_ink` strip filtering, scaled strips, packing/splitting, text-width
clipping, and final canvas ink filtering. Record every nominated line's
membership in retained bands, extracted strips, the scaled row stream, and
returned canvases; band membership alone cannot establish output preservation.
Reuse existing production functions and test-only trace/stream helpers. A lost
canvas needs a trace of its scaled ink and threshold, not a furniture diagnosis
inferred from an absent output line.

Choose corrections by the exact stage and page role:

- Furniture-driven crop/banding loss: test the smallest source furniture mask
  whose preservation boundary is proven. The C1/C6 left-rule mechanism is a
  hypothesis; verify crop edges, span ratios, classification and dropping stage.
- Opening-page trim: `body_top_frac` or `skip_top_bands` on the owning unit may
  correct its first page; record the exact row and pin.
- Shared-tail trim: `title_top_frac` on the following unit controls this cut.
  Review both units' prose and titles and pin the changed row.
- Interior head/tail, strip-filter, or canvas-filter loss: opening settings have
  no effect. If a safe mask does not restore the line, record `unresolved` and
  amend 4.1 or 4.2 with the responsible function, narrowly bounded behavior,
  ordinary regression names, affected identities, and preservation tests before
  coding a correction. Do not retune global thresholds or add an override system.
- No safe specified fix: keep the source untouched, record unresolved Tier-A
  work, and do not claim this plan complete or downgrade eligibility.

For the 20 initial Tier-B events, preserve the confirmed historical verdict and
an explicit repair-read disposition. Do not design or enable a fix without a
confirming human read and recorded promotion. After promotion, trace all source
lines and include the event in the Tier-A restoration gate.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy
--all-targets -- -D warnings`; `cargo test audit_omission_inventory`;
`cargo test audit_omission_source_trace -- --ignored --nocapture`;
`cargo test facsimile::reflow`. Confirm every Tier-A event has per-line stage
evidence or an explicit unresolved reason, all Tier-B dispositions exist, and no
rejected event entered the inventory.

The baseline diagnosis also runs `audit_candidate_source_trace` with
`-- --ignored --nocapture` and ordinary
`audit_omission_diagnosis_covers_all_source_lines`. The latter validates the
39/69 source-region count, per-line stage evidence, deferred Tier-B dispositions,
and hashes of the diagnostic archive manifest, baseline traces and native
source-line views. The source-dependent commands may use `cargo test --release
--lib` to avoid repeatedly scanning all production rasters in debug mode; each
named ignored command must select and execute at least one test. The packer and
final canvas floor share one production implementation, exposing their boundary
to test tracing without copying the packing algorithm.

**Human verification:** Read every Tier-A source loss and its baseline reader
seam, including complete multi-line blocks. Review proposed masks and boundaries;
confirm the source text and neighbors that a repair must preserve. Review
Tier-B dispositions. Post-dedup repair confirmation occurs in 2.1A.

### Measured repair decisions

The diagnostic fixture owns measured source coordinates and frozen baseline traces.
The D4?D12 production exclusions and their installed ledger records are in place;
the existing D1?D3 records are preserved. The seven step-3.1 furniture cases are installed with original-pixel, crop-edge,
Pass-A-band and full-unit head-trim verification. Omission corrections remain
proposed. Step 2.1 verifies the reviewed retained source-line regions
through extracted strips and composed canvases.

| Finding | Source decision | Status | Responsible step |
| --- | --- | --- | --- |
| D4 | v2 keep 115, 116, 117, 118; exclude 119, 120, 121, 122; no unique second-scan Latin observed | installed; source-line regression passed | 2.1 |
| D5 | v2 keep 211, 212; exclude 213, 214; no unique second-scan Latin observed | installed; source-line regression passed | 2.1 |
| D6 | v2 keep 295, 296; exclude 297, 298; no unique second-scan Latin observed | installed; source-line regression passed | 2.1 |
| D7 | v2 keep 359, 360; exclude 361, 362; no unique second-scan Latin observed | installed; source-line regression passed | 2.1 |
| D8 | v2 keep 375, 376; exclude 377, 378; no unique second-scan Latin observed | installed; source-line regression passed | 2.1 |
| D9 | v2 keep 463, 464; exclude 465, 466; no unique second-scan Latin observed | installed; source-line regression passed | 2.1 |
| D10 | v2 keep 521, 522; exclude 523, 524; no unique second-scan Latin observed | installed; source-line regression passed | 2.1 |
| D11 | v2 keep 671, 672; exclude 673, 674; no unique second-scan Latin observed | installed; source-line regression passed | 2.1 |
| D12 | v2 keep 699, 700; exclude 701, 702; no unique second-scan Latin observed | installed; source-line regression passed | 2.1 |
| H2 / R9 on 375 | Agis v2/375: right divider begins at x1753; reviewed Latin ends before x1740; candidate x1745..1797 preserves Latin and unmerges the head | installed; source-pixel and head extraction checks passed | 3.1 |
| C3 | Phocion first source line on 299 remains missing; raw 296→299 join reviewed | post-dedup measured; correction specified; unresolved until 4.2 | 2.1A / 4.2 |
| Additional Cato verse label | Cato Minor v2/360 `Et:`, source region [888, 2026, 932, 2048]; 374 ink pixels below the 498-pixel strip floor | unresolved | 1.1C / 4.2 |

The initial omission inventory remains exactly 39 Tier-A events / 69 lines and
20 Tier-B events / 44 lines. The additional Cato verse-label loss is tracked
separately; it does not alter frozen event IDs, historical verdicts, confidence,
or the initial tier partition. Its correction requires a source-preserving
amendment to 4.2 after the complete stage trace.

Agis ends on source 376 at `ceteris credidisset.`; source 379 is the shared
Cleomenes opening and contributes no Agis body band. The 376→379 identity join
is valid without a unit-row change. The source 375 right-rule candidate gives
crop [991, 1743] and separates its body bands; the baseline crop is [991, 1797]
and its leading merged band spans [52, 905], including Greek head furniture.

Supplemental D1–D3 line-region evidence resides in the diagnostic fixture,
with the existing installed records preserved. Existing omissions in those
retained occurrences remain governed by their event-specific eligibility:
`miss_cato_minor_331_9707` is Tier A; `miss_cato_minor_329_8786` is Tier B and
cannot receive a correction without the required confirming human read.

Watermark candidates measured copied endpoints W4: 2881→2572,
W5: 2735→2556, and W6: 2735→2561. Summing actual extracted strip intervals
gives page copied heights W4 2630→2314, W5 2512→2351, and W6 2321→2298.
Actual copied furniture-tail rows beyond the reviewed last Latin are respectively
309→0, 161→0 and 128→0. Endpoint subtraction would overstate W5/W6 tail
removal because the strip floor already discards some intervening intervals.
W6 also restores top-body rows, so its total copied height cannot stand in for
tail removal. Each watermark is a tail, with
no source-page Latin beneath its logo. The next source page supplies the W4/W5
continuation; W6 still has the separate missing first line on 624. The candidate
623 right-rule cleanup also separates the three top lines formerly swallowed by
a merged head band; Phase 4 must verify their restoration independently.

F1 and F2 are deferred: no complete safe boundary has been proven beside Latin
and legitimate reference letters. S2 uses the specifically amended packing
branch in 4.3. The baseline diagnosis covers every initial Tier-A event:
39 events, 69 individually located source-line regions, raw/cropped band
measurements, extraction membership and the final canvas filter. All remain
unresolved until installed repairs pass their source-to-canvas assertions.
The 20 Tier-B events retain their confirmed historical verdict and explicit
pending-human-read disposition; none is promoted or targeted.
The finite body-band correction specification contains 47 observed signatures;
multi-line blocks share one retained interval without duplicating any source row.
Step 2.1A freezes post-exclusion coverage for all 39 Tier-A events / 69 lines
in the diagnostic fixture and hash-bound archive traces. All 39 still require
the finite corrections specified below; no omission is claimed restored by
de-duplication. Final repair dispositions await source-to-canvas regressions.

The separate diagnostic archive is
`archive/inner_tomi_audit_repairs_2026_10_08/baseline`: 4,404 files,
1,657,727,176 bytes. Its manifest SHA-256 is
`8ffd341b1a5c8b821f22698a0089e85f576aede01c86e7b57ecc2e6f457eab50`.
It preserves source/candidate columns, duplicate comparisons, native source-line
views, baseline packed canvases before and after the ink filter, and trace JSON.
Diagnostic fixture references point into this archive, so future pipeline traces
cannot overwrite the baseline. `diagnosis_stage` distinguishes later local
omission/candidate trace output directories. No OneDrive ownership or network
copying was established; these are ordinary local paths.

# Phase 2 — De-duplicate the nine repeated blocks D4–D12 (Suggested implementer: Opus)

## 2.1 Extend the whole-page duplicate exclusion

Apply the step 1.1A decisions. In `reflow.rs`, extend
`skip_known_duplicate_source_page` with the second-occurrence source pages, each
scoped to its unit id and source 1:

- `agesilaus` 119..=122, `alexander` 213|214, `phocion` 297|298,
  `cato_minor` 361|362 (in addition to the existing 339..=348),
  `agis` 377|378 (in addition to 369|370), `cicero` 465|466,
  `antonius` 523|524, `aratus` 673|674, `otho` 701|702.

Preserve the existing D1–D3 matches exactly. Keep the helper a pure identity
predicate (no JSON schema, no CLI config); it must return false for every
unrelated identity. Because the excluded pages are interior, no
`facsimile_units.json` or pin change is required; if Phase 1 unexpectedly found a
boundary effect, stop and amend this step with the specific unit-row correction
before coding.

**Reuse the existing duplicate-regression machinery; do not build parallel
machinery.** `assert_reviewed_duplicate_sequence(id)` in `reflow.rs` already loads
a `duplicate_page` record (`skip_pages`, `retain_sequence`) from the committed
`src/facsimile/fixtures/inner_tomi_repair_source_trace.json` and performs the
skip-aware pixel comparison through `RetainedSourcePageTrace::map_row`. Append the
D4–D12 records to **that** ledger (per "Where reviewed coordinates live" in
Phase 1) and reuse that helper, rather than hand-rolling a per-block test.

Append the D4–D12 installed records together with the predicate changes in this
step; Phase 1 leaves the installed ledger untouched. Each new record includes
reviewed body-line regions for the retained occurrence and join landmarks.
Store the new `reviewed_lines` field as a list of
`{source_page, region: [left, top, right, bottom], role}` objects in full-source,
half-open production pixels; `role` is `retained_body` or `join_landmark`.
Capture missing line-region evidence for existing D1–D3 records in the diagnostic
fixture, keyed by finding identity, rather than editing those frozen records.
The helper reads `reviewed_lines` from each new record or this explicit evidence
reference for an existing record; missing expected regions are a test failure.

The three existing D1–D3 wrappers are `eumenes_comparison_prose_is_complete_and_unique`,
`agis_repeated_passage_survives_once`, and `cato_minor_repeated_folios_survive_once`.
Because D8/D7 add a *second* block to agis and cato_minor, extend those two
existing tests to cover both blocks — do not create a parallel `agis_*`/`cato_minor_*`
test. Add seven thin `assert_reviewed_duplicate_sequence(id)` wrappers:
`agesilaus_second_scan_survives_once`, `alexander_second_scan_survives_once`,
`phocion_second_scan_survives_once`, `cicero_second_scan_survives_once`,
`antonius_second_scan_survives_once`, `aratus_second_scan_survives_once`, and
`otho_second_scan_survives_once`.

**agis and cato_minor need two disjoint skip blocks each.** They already carry a
`duplicate_page` record (agis skip 369–370, retain […368, 371…]; cato_minor skip
339–348, retain [336, 337, 338, 349, 350]), and D8/D7 add a second, non-adjacent
block (agis 377–378; cato_minor 361–362). A single contiguous `retain_sequence`
cannot express two disjoint skips, and `assert_reviewed_duplicate_sequence`'s
`.find` currently takes only the **first** matching record and asserts one
contiguous retained run (`sequence.windows(seam.len()).any(...)`). Resolve this
explicitly: give those units **one `duplicate_page` record per block**. Collect
all records for the unit and build a `BTreeSet<u32>` union of every record's
`skip_pages`. Render the unit once, compute the expected full sequence by
filtering that union from the inclusive unit range, and compare it once.
Then assert each block's skips are absent and its own `retain_sequence` occurs
contiguously in that retained sequence. Never compare the full sequence against
only one block's skip list. Reuse the single render, metrics, traces, strips and
composition for every block; do not merge non-contiguous blocks into one record
or let a first-match `.find` ignore another record. A record's `retain_sequence`
must not include any page in a sibling record's `skip_pages`:
`known_duplicate_page_exclusions_are_identity_scoped` asserts the predicate
returns false for every retained page, so such a record fails it — for agis, the
D8 record's `retain_sequence` uses neighbors like 375–376 and 379, never
377–378.

**Migrate the skip-unaware otho regression.** `otho_final_source_pages_survive_reflow_once`
will break under D12: `traced_real_unit` asserts the request sequence equals the
full `696..=708` range and indexes `pages[page - start_page]`, and
`closing_body_bands.json` holds otho records for **src 1 pages 705–708** — all
*after* the 701–702 skip, so `page - start_page` is off by two. Rewrite the otho
path to be skip-aware: expect 701/702 absent from the rendered sequence and locate
pages 705–708 through the parallel `source_pages` vector / `map_row`, not
`page - start_page`. Audit every `traced_real_unit` and `verify_real_composition`
caller before relying on them: among current callers only **otho** skips
(cimon_lucullus, lysander, cato_major, and comparatio_aristidis_et_catonis are all
source 0 and unaffected), but re-verify after wiring the exclusions.

No new identity-scoping test is needed: `known_duplicate_page_exclusions_are_identity_scoped`
already iterates **every** `duplicate_page` record in the ledger and asserts each
`skip_pages` entry matches only its own `(id, source)` and each `retain_sequence`
entry does not — so appending the D4–D12 records extends its coverage
automatically. Confirm it picks up the new records (its selected-assertion count
rises) rather than adding a parallel test.

The helper's page-sequence and surviving-band comparisons do not by themselves
prove Latin completeness. Extend it to call
`assert_reviewed_source_lines_survive` for every block's retained body regions
and join landmarks, proving coverage once in retained bands and extracted strips,
correct source order, and preservation through the returned composed canvases.
The helper must fail when an expected reviewed region is missing, even if all
remaining pixels match raw source. Assert no strip derives from excluded pages,
including C4/S1 on 213 and C7/R12 on 701.

**C3 staging:** Phocion's source 299 first line is known missing until Phase 4.
At Phase 2, assert the 296→299 source-page adjacency and preservation of the
currently surviving join regions; record the missing first line as unresolved
Tier-A evidence, not as an expectation waived globally. Its exact source
interval is required by `phocion_c3_line_restored` in Phase 4.2, which checks the
complete 296→299 prose join after restoration.

**Additional pending source regions:** D7's retained source 360 contains the
separate `Et:` verse label [888, 2026, 932, 2048], which the baseline strip filter
drops. Keep this explicit pending region in the diagnostic fixture; Phase 2
asserts all currently extracted reviewed D7 regions and does not claim the label
restored. The complete D7 source check must include it after its specified 4.2
correction. Supplemental D3 evidence likewise retains the separately inventoried
331 Tier-A and 329 Tier-B losses, rather than asserting those lines already survive
or waiving their eligibility requirements.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy
--all-targets -- -D warnings`; `cargo test known_duplicate`; `cargo test
facsimile::units`; run all seven named wrappers, the extended
`agis_repeated_passage_survives_once` and `cato_minor_repeated_folios_survive_once`,
and `otho_final_source_pages_survive_reflow_once` with
`-- --ignored --nocapture`; `cargo test facsimile::reflow`. Verify the union
covers both skips per affected unit and every filter selects a test. Confirm
C3 remains explicitly pending without making the Phase 2 regressions depend on
its future restoration.

**Human verification:** Compare each retained occurrence and both source seams;
confirm distinct prose and split words proceed once. Confirm Alexander's
clipping/slice and Otho's repeat are gone. Review C3's pending first-line region
and verify no Phocion body line outside that known omission disappears.

## 2.1A Finalize omission diagnosis against the installed exclusions

Run `audit_omission_source_trace` over all Tier-A events using the actual
`unit_body_strips` predicate installed in 2.1. Recompute full-unit Pass A
`line_metrics`, Pass B trims, extracted strips, scaled stream and composition
for each affected unit once. Save post-dedup evidence alongside the frozen
baseline. Source 299 remains an interior page, but its surrounding metric sample
can change; measure that behavior rather than assuming an unchanged result.

For every event, compare baseline and post-dedup per-line coverage through
returned canvases. If it now survives completely, record the exact cause and
verify it with its named restoration regression without adding a fix. Otherwise
finalize a safe proposed correction or record unresolved Tier-A work and amend
4.1/4.2 with the exact correction and regression names before coding it.
This step may update diagnosis and tests, but must not add omission repairs
or future mask records to the installed ledger. Dependent omission repairs wait
for their own resolved mechanism; unrelated confirmed furniture repairs may
proceed. Refresh this trace after Phase 3's masks before applying Phase 4 fixes.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy
--all-targets -- -D warnings`; `cargo test audit_omission_inventory`;
`cargo test audit_omission_source_trace -- --ignored --nocapture`;
`cargo test facsimile::reflow`. Inspect the actual skip-aware source identities
and changed metrics. Every initial and promoted Tier-A event has post-dedup
coverage evidence or an explicit unresolved status; no Tier-A case is deferred.

**Human verification:** Compare baseline/post-dedup source and reader seam crops,
especially 296→299 and units with two skip blocks. Review the measured mechanism
and preservation boundary of each proposed omission correction.

# Phase 3 — Remove known furniture and its copied space (Suggested implementer: Opus)

## 3.1 Extend the source furniture masks

Apply the step 1.1B decisions. In `reflow.rs`, extend `SOURCE_FURNITURE_MASKS`
with the exact normalized rectangles established in Phase 1:

- **Watermarks:** W4 (v1 p321), W5 (v2 p603), W6 (v2 p623), each with its pale
  watermark rectangle **and** its connecting right-edge rule / scan-noise
  rectangle, so the watermark is not left merged into the last body band.
- **Head/folio leaks:** H1 (v1 p448 — head, folio, and Greek-scrap rectangles as
  needed), H2 (v2 p375 Greek fragment), caius_marius (v1 p528), cato_major
  (v1 p432).
- **Confirmed Greek scraps:** F1 (Theseus v1/17) and F2 (Pyrrhus v1/470), if
  step 1.1B proves safe rectangles. Otherwise preserve each and record an explicit
  defer reason; the exploratory 14-candidate screen remains outside bulk cleanup.
- **Safely separable rules only:** any R-series right rule that Phase 1 proved is
  clear of Latin. Leave ambiguous rules unmasked with a recorded reason. Do not
  widen `exclude_isolated_left_divider`'s gates.

Reuse `apply_source_furniture_mask` and its existing call site in
`render_and_band`; define a rectangle once even if it serves several findings.

The measured head repairs use the smallest successful masks: H1, Marius, and
Cato Major clear their detached folio numerals; existing furniture trims then
remove the remaining head fragments. H2 is resolved by the verified source 375
right-rule mask. For Cato Major source 0/page 432 specifically, clearing the folio
moves the detected crop from [907, 1711] to [907, 1710], cutting original body ink
at x1710. Add `preserve_known_body_crop_edge(source, page, raster, column)` after
column detection in `render_and_band`: when and only when identity is (0, 432),
raster dimensions are (1948, 2980), and the detected column is [907, 1710], return
[907, 1711]. All other inputs return their unchanged column. This preserves the
measured body edge without changing thresholds or the column detector. Add
ordinary `cato_major_folio_cleanup_preserves_rightmost_body_pixels`, proving the
original edge pixel survives and unrelated source/page/dimension/crop identities
are unchanged. The installed mask record uses independently frozen expected
crop [907, 1711] and original body-band intervals with the header removed.
Each entry carries a comment describing the furniture and its preservation
boundary — no plan references, no change narration. Use the smallest safe
rectangles including pale antialiased watermark edges. Never apply a global
grayscale threshold (faint Latin must survive) and never mask the bounding
envelope of a component that includes Latin. Preserve crop height and original
band coordinates through Pass B. Do not touch raw caches, source PDFs,
transcription caches, general thresholds, body scale, or the notice page.

The candidate masks expose three genuine small Latin fragments that the strip
ink floor would otherwise discard. Add `coalesce_known_body_fragments(source,
page, column_dimensions, bands)` immediately after `line_bands` in
`render_and_band`. It merges only the exact consecutive pairs below; all other
bands, identities and dimensions are unchanged. Merging copies original rows,
including the small intervening white gap, without painting any glyph.

| Source/page | Column dimensions | Observed consecutive bands | Retained band |
|---|---|---|---|
| 0/321 | 764 × 2960 | [2529,2533], [2534,2572] | [2528,2572] |
| 1/375 | 752 × 2836 | [2209,2245], [2247,2251] | [2209,2251] |
| 1/375 | 752 × 2836 | [2315,2319], [2321,2357] | [2315,2357] |

Add ordinary `known_body_fragments_preserve_faint_latin_pixels` using the real
band-extraction/composition functions on a synthetic column with faint fragments;
prove original pixels survive, the merged band crosses the unchanged ink floor,
and changed page/source/dimensions or nonmatching band pairs remain unchanged.
Freeze the installed mask's expected Pass-A intervals after these finite joins.
Run this test alongside the Cato crop-edge preservation test in this step.

Append each installed mask record to the existing repair ledger in this same
step; leave future Phase 4 proposals in the diagnostic fixture. Extend existing
ignored `inner_tomi_known_masks_preserve_latin`; it already reads every installed
mask and checks the union of rectangles and every outside pixel. Add assertions
for the new records' independently measured `expected_crop_edges`,
`expected_pass_a_bands`, and reviewed `preserved_latin_regions`, using the
same source-coordinate representation as existing records. Freeze Pass A
expectations here, not an interior page's expected Pass B first line when a
separate Phase 4 omission remains unresolved. Share any reusable Latin-region
assertion with `assert_reviewed_source_lines_survive`; do not add a parallel
`audit_known_masks_preserve_latin` implementation.

Extend the existing mask unit tests and shared `masked_tail_fixture` for new
identities and connecting-noise cases. Reuse
`masked_tail_cut_points_exclude_furniture_rows`,
`watermark_cleanup_removes_pixels_and_space`, and
`watermark_cleanup_preserves_unrelated_paragraph_gap` to assert that the
right-rule mask restores a clean final band while every synthetic body pixel
survives; do not build a second watermark fixture or equivalent test harness.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy
--all-targets -- -D warnings`; `cargo test known_source_furniture`; `cargo test
source_furniture_mask`; `cargo test inner_tomi_masks_match_reviewed_ledger`;
`cargo test masked_tail_cut_points_exclude_furniture_rows`; `cargo test
watermark_cleanup`; `cargo test inner_tomi_known_masks_preserve_latin -- --ignored
--nocapture`; re-run prior ignored watermark/head regressions and
`lysander_closing_line_survives_reflow` plus
`cato_comparatio_source_boundary_excludes_duplicate_title` with
`-- --ignored --nocapture`; `cargo test facsimile::reflow`. Check the new
cato_major p432 mask does not disturb its p434 closing-band evidence.

**Human verification:** Inspect each new mask overlaid on the raw source image and
the cleaned seam. Check pale watermark edges, faint Latin, every glyph adjacent to
a masked rule, and that the retained Agis first occurrence keeps H2's neighborhood
Latin. Review any intentionally retained rule explicitly.

## 3.2 Prove the watermark space disappears through the real pipeline

This step enforces the user's explicit requirement that removing a watermark also
removes its copied blank space. Extend the existing ignored
`inner_tomi_watermark_cleanup_removes_pixels_and_space` case table to W4/W5/W6,
preserving W1/W2 coverage. Extract one shared per-case assertion helper from that
test if needed; reuse production extraction, `strip_and_canvas_ranges` and
`assert_scaled_stream`. Each case reads measured endpoints/heights and following
source-line regions from its installed ledger record and runs actual
`unit_body_strips` → strip extraction → `compose_pages` at production global
scale. Assert:

- The page's final retained band ends at the measured last-Latin row (from the
  Phase 1 ledger), not at the former watermark/edge row.
- No extracted strip reaches the furniture-only tail interval.
- The composed seam copies **zero** source-margin padding rows beyond the ordinary
  median body gap (allow only resampling rounding). Sum actual copied strip
  intervals and their intersections with the furniture-only tail; compare the
  independently frozen baseline totals, not endpoint subtraction. Record both
  total copied page height and removed copied tail height. W6's restored top
  lines can increase copied body height independently of the removed tail.
- For **W4 and W5**, the following source page's first true Latin line survives
  intact. For **W6**, do not assert 624's first line here (see the note) — assert
  only the 623 tail/space removal and the integrity of reviewed currently
  surviving neighbors in this phase. Store the separate C5 expected source-line
  region in the diagnosis fixture; it is not a Phase 3 installed expectation.

**W6 and C5 overlap (brutus v2/623 tail, v2/624 first line).** The W6 tail mask is
on 623; C5 is the separately-dropped true first line of 624
(`L. Erat inter amicos Bruti Lucilius quidam, vir fortis.`), which is **not**
restored until Phase 4.2. So a W6 regression that asserts 624's first line survives
would fail at the end of Phase 3. In Phase 3.2 the W6 case asserts only that the
623 tail and its copied space are gone; the "624 first line present" assertion
belongs to the C5 regression in Phase 4.2, where the whole 623→624 seam is
re-verified with both the tail removed and C5 restored. Also note that 624's
*currently*-retained first band is the second line (`Is quum videret…`), so any
later check must anchor on 624's **source** first line, not that band.
Brutus also has a separate confirmed three-line omission at the top of 623
(`miss_brutus_623_11958`). The W6 tail test must not depend on restoring those
top lines in Phase 3; their regression belongs to Phase 4.2.

These are source-page **tails**, so no interior row-compaction is expected. If any
watermark unexpectedly proves to be an interior gap (a surviving band overlaps the
proposed removal interval), **stop**: do not clip a body band and do not invent
spacing metadata — resolve it with a safe pixel mask and rebanding, and amend this
step with the source identity, safe interval, adjacent bands, and measured copied
excess. The `SourceWhitespaceExclusion` row compaction is a design alternative
from the 2026_10_05 minimal-repairs plan that was never implemented — the name
appears in no source file; do not build it unless such a case is established
and reviewed.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy
--all-targets -- -D warnings`; `cargo test inner_tomi_watermark_cleanup_removes_pixels_and_space
-- --ignored --nocapture`; `cargo test facsimile`. Confirm all five W1/W2/W4/W5/W6
cases executed and W6 has no premature assertion for unresolved 623/624 lines.

**Human verification:** Inspect each former watermark seam; confirm the large
copied blank is gone, all Latin survives, and unaffected paragraph/chapter spacing
is unchanged. Whitening followed by an equally tall gap is a failed repair.

# Phase 4 — Restore the omitted Latin lines (Suggested implementer: Opus)

This phase runs after Phase 2 and Phase 3. Re-run
`audit_omission_source_trace` with the installed exclusions and furniture masks
before applying any omission correction, since masks can change columns, bands,
metrics and neighboring seams. Apply only the Tier-A corrections finalized in
2.1A and refreshed here. Each event must have a resolved mechanism and a safe,
specific fix appropriate to its page role.

Reuse masks and existing unit-boundary settings where they affect the diagnosed
stage. Opening settings cannot repair an interior trim. If existing mechanisms
cannot restore an event, keep it unresolved and amend its step with the exact
function/identity scope, behavior and ordinary regression names before coding a
narrow correction. No global threshold retuning, generalized override framework,
new split heuristic or font-scale change is authorized. The default leaves the
packer unchanged; an exceptional S2 correction follows 4.3's amendment gate.
Do not declare the phase or plan complete with any unresolved Tier-A omission.
Install each new mask and its ledger record together; future proposals stay in
the diagnostic fixture.

## 4.1 Restore clipped-and-dropped seams (C1/C6 and same-mechanism losses)

Route each Tier-A event by its refreshed 2.1A diagnosis. This step handles cases
where measured clipping and furniture-driven geometry cause the loss. The
baseline has established separate crop and gap-filter causes for C1/C6 on
v1/38–39 and Caesar v2/252; verify their refreshed signatures before installation. Route all
multi-line events, including Julius Caesar (12), Theseus (7), Brutus 623 (3),
Demosthenes 437 (4) and 443 (2), Eumenes 91 (2), Solon 115 (2), and Tiberius
Gracchus 414 (2), by their actual diagnosis to 4.1 or 4.2. Confirm every source
line; a large loss does not establish a trailing-cluster cause.

The source trace refutes the proposed left-mask mechanism for C1. Romulus
0/38's chapter IX is a full Latin block in raw band [2451,2651];
`drop_furniture` discards it as a detached trailing gap outlier. Its detected
right edge also cuts the ends of the last lines. A left-rule mask spanning this
page would cut upper-page Latin and is therefore prohibited. C6 on 0/39 has
a separate left crop at x1101 that cuts the beginning of the printed word
`educati`. Caesar 1/252 loses the twelve-line raw band [240,744] as a detached
leading gap outlier, and the detected right edge also cuts line endings.

Extend `preserve_known_body_crop_edge` with these exact cases, requiring the
source/page identity, production raster dimensions and observed column:

| Source/page | Raster | Observed column | Corrected column |
|---|---|---|---|
| 0/38 | 1988 × 3000 | [929,1763] | [932,1774] |
| 0/39 | 2004 × 2989 | [1101,1892] | [1085,1892] |
| 1/252 | 1874 × 2831 | [891,1706] | [891,1722] |

The 0/38 left adjustment removes white padding; independently reviewed upper
body glyphs begin at x932, and the lower body ends before x1769. Its genuine
Latin width therefore fits the unchanged 1004-pixel text measure at the unchanged
global scale. The 0/39 correction includes the complete source glyphs. The
1/252 correction includes original line endings and marginal reference ink.
Every unrelated detector result remains unchanged. Add ordinary
`known_body_crop_corrections_preserve_clipped_glyphs`, covering all three exact
cases, original edge pixels and negative identity/dimension/column cases.

Add a closed, edition-specific `retain_known_source_body_bands` helper in
`reflow.rs`. Its finite source/page, column-dimension and observed-band signatures
come from the committed diagnostic fixture's `planned_body_band_repairs`.
It reinserts or extends only reviewed genuine body bands, never an unresolved
mixed head/body band. Call it after Pass-A furniture/early-body filtering and
after Pass-B trims. Pass A has no unit owner restriction; Pass B requires the
record's exact owning unit ID, so shared-tail Latin cannot be resurrected in
the following comparison. Source row coordinates, pixel values, line order,
metrics thresholds and scale stay unchanged. The helper receives raw/Pass-A
band evidence and merges overlapping retained intervals without duplicating rows.
It is a literal finite predicate/table, not a runtime override file or new CLI.
Install only the Romulus 0/38 and Caesar 1/252 entries here; step 4.2 extends
the same definition for the remaining reviewed identities. The protected interval
covers the independently reviewed glyph boundaries even where a few faint edge
rows fell below `line_bands`' unchanged per-row ink threshold.
Add ordinary `known_source_body_bands_survive_furniture_filters` to demonstrate
both gap-outlier failures, complete source-row preservation through extraction
and composition, unchanged unrelated cases, and correct shared-tail ownership.

Add ignored `romulus_c1_c6_lines_restored` and the single table-driven ignored
`audit_tier_a_source_lines_restored`, whose case table covers every other event
assigned here (step 4.2 extends the same table rather than adding a second
test). Both read
independently reviewed source line/glyph regions from committed evidence and call
`assert_reviewed_source_lines_survive` on actual full-unit extraction and
composition. Check Romulus chapter IX and its neighbors, all missing lines, and
C6's left glyph boundaries. Do not use the current retained crop as the expected
left boundary.

Omit `left_rule_mask_restores_full_width_bands_and_tail_lines`: the native source
trace refutes its hypothesized cause. Run the two exact ordinary regressions
above for the measured crop and gap-filter corrections.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy
--all-targets -- -D warnings`; run `cargo test
left_rule_mask_restores_full_width_bands_and_tail_lines` if that mechanism was
confirmed, otherwise run the exact amended ordinary regression;
`cargo test romulus_c1_c6_lines_restored -- --ignored --nocapture`;
`cargo test audit_tier_a_source_lines_restored -- --ignored --nocapture`;
`cargo test facsimile::reflow`. The shared Tier-A regression checks every installed
case; uninstalled cases remain explicitly pending during this step.

**Human verification:** Read Romulus I/68–69 (and each other restored seam) against
v1/38–39: confirm chapter IX's lines are present, complete, and unclipped, and
that no line is duplicated or reordered.

## 4.2 Restore seam omissions not caused by clipping (C2, C3, C5, remaining confirmed events)

For each Tier-A event attributed to another stage, apply its finalized correction:

- **C3 (Phocion v2/299):** page 299 is interior to Phocion (start 286), with or
  without D6. Trace its first line through `drop_furniture`,
  `retain_early_aligned_body_bands`, `drop_interior_running_head`, extraction
  and returned canvases using the post-dedup full-unit metrics. A verified mask
  may restore the geometry. Changing Phocion's `body_top_frac` or
  `skip_top_bands` affects page 286 and cannot repair 299. If the interior-head
  predicate itself drops genuine Latin after safe cleanup, require the specific
  function/identity-scoped correction and ordinary preservation regressions
  specified by an amendment before coding. Record no additional correction only
  when the refreshed trace and regression prove the complete source line survives.
- **C2 (Coriolanus v1/291–292)** and **C5 (Brutus v2/624):** use the correction
  for their measured page role and stage. A source-page mask, an actual opening
  correction, or a following-unit title fraction is applicable only where that
  stage reads it. C5 is independent of W6's 623 tail. Its restoration test checks
  the complete 623→624 source seam, including 624's reviewed true first line.
  Re-run `inner_tomi_watermark_cleanup_removes_pixels_and_space` to ensure W6
  still removes its tail and copied space; its Phase 3 table keeps its original
  tail-focused expectation, and `brutus_c5_line_restored` supplies the complete
  first-line assertion.
- **Additional multi-line losses:** explicitly cover Brutus v2/623's three top
  lines, Demosthenes v2/437's four and v2/443's two, Eumenes v2/91's two, Solon
  v1/115's two, and Tiberius Gracchus v2/414's two, plus Julius Caesar/Theseus
  blocks if assigned here. Each case needs all reviewed source intervals and
  exact line count. The Brutus 623 top loss is distinct from both W6 and C5.
- **All remaining initial or promoted Tier-A events:** install the safe
  specified fix, or keep unresolved and amend this step before a new correction.
  A preservation reason is not restoration and cannot satisfy completion.

Add ignored `phocion_c3_line_restored`, `coriolanus_c2_lines_restored`, and
`brutus_c5_line_restored`, and extend the table of the ignored
`audit_tier_a_source_lines_restored` added in 4.1 with this step's events.
Use the same
`assert_reviewed_source_lines_survive` for these wrappers and every other
single- or multi-line event; render/compose once per unit in the table-driven
test. Store an exact `regression_name` on each diagnosis record: the relevant
named C1/C2/C3/C5 wrapper or `audit_tier_a_source_lines_restored`.

The measured baseline requires no opening or title-fraction changes. Extend
`preserve_known_body_crop_edge` for Brutus source 1/page 623: raster dimensions
1810 × 2843, detected column [982,1771] after W6 cleanup, corrected column
[976,1771]. Native full-width review shows the beginnings of the three restored
top lines extend left of x982. Extend the same ordinary crop-preservation test
with this identity; it leaves the original source glyphs and every unrelated
column unchanged. The protected-band signatures use this corrected 795-pixel
column width. The three lines remain unresolved until this crop and their full
source-to-canvas preservation assertion pass.

Extend
the same `retain_known_source_body_bands` definition from 4.1 with the remaining
finite identities and observed/protected intervals recorded in
`planned_body_band_repairs`. Most losses occur in the top-zone or leading/trailing
gap filters; Coriolanus 0/292 and Solon 0/115 also lose genuine body text in
Pass B. Restrict each Pass-B protection to its exact owning unit. Theseus
0/15's seven chapter-VI lines remain in one genuine body block with an ambiguous
left divider; preserve that block without masking the overlapping rule.
Extend `known_source_body_bands_survive_furniture_filters` with top-zone,
interior-head, shared-tail ownership and multi-line preservation examples.

Install the six independently reviewed external-divider masks from the diagnostic
fixture for source 1/pages 391, 393, 437, 443, 407 and 411. Their pixel left
boundaries are respectively 1790, 1850, 1800, 1800, 1840 and 1830; each rectangle
extends to the right and bottom raster edges. The source raster dimensions and
preserved Latin boundaries are frozen in `candidate_mask_cases`. They separate
mixed running-head/body bands without erasing Latin. Reuse the existing mask
table, installed ledger and source-pixel preservation regression. These masks
restore the nominated lines on most of these pages; protect remaining faint
glyph rows or the 411 top-zone loss with the finite body-band helper.

Extend `coalesce_known_body_fragments` for three further reviewed pairs exposed
by these masks: source 1/page 391, column 782 × 2825, [1192,1195] followed by
[1196,1231] becomes [1192,1231]; page 393, column 789 × 2832,
[928,959] followed by [961,962] becomes [928,962]; page 443, column
807 × 2838, [2153,2179] followed by [2180,2185] becomes [2153,2185].
These contain a printed question-mark fragment and descenders. Extend the same
ordinary faint-fragment test, leaving the global ink floor unchanged.

Restore the additional Cato Minor 1/360 verse label independently of the frozen
39-event/69-line inventory. Its source region [888,2026,932,2048] contains 374
ink pixels, below the unit's 498-pixel strip floor. Add
`is_known_source_verse_label(column, band)`, recognizing only column dimensions
788 × 2838, cut-point strip [2010,2064], and the original 44 × 22 glyph sample at column-relative
[3,2026,47,2048], with grayscale SHA-256
`472ea07f0f6e6d85db2dfffcc4faf9d393da12c60ea1f516ba1c28946ca56f62`.
In `strips_in_reading_order`, retain this exact source fingerprint despite its
ink floor. Never substitute fixture pixels for source pixels. Add ordinary
`known_source_verse_label_survives_ink_floor`, proving complete pixel preservation
through scaling/composition and unchanged filtering for changed sample pixels,
dimensions and coordinates. Include this original-source label in the D7
full-line preservation assertion and final evidence as one additional restored
source item, without changing the initial omission count.

During intermediate steps, the table-driven test asserts every installed event
and reports every still pending event by ID/count without treating it as repaired.
At the end of 4.2, add ordinary `audit_tier_a_completion_requires_all_lines`;
it validates every initial/promoted Tier-A entry is installed, has its exact
reviewed line regions and passing-evidence reference, and has no unresolved
status. This completion check is added only when its expectations can pass;
before then the plan remains incomplete. The final ignored restoration run must
execute every Tier-A case with no pending entries and verify all neighbors,
split-word continuations, order and once-only extraction through composition.
For a loss already restored by an earlier installed repair, record that repair's
identity and require the same restoration assertion; do not add a redundant fix.

If a unit-row field changes, pin the exact row and add ordinary table-driven
`audit_boundary_corrections_survive_redetection` to verify `merge_pins`
preserves every changed row's fields. Reuse the existing detector test helpers;
for a shared-tail title change, verify both adjacent unit boundaries and titles.
Any new behavior in an amended correction also requires its specified ordinary
failure/preservation regressions.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy
--all-targets -- -D warnings`; run the three named C2/C3/C5 tests and
`audit_tier_a_source_lines_restored` with `-- --ignored --nocapture`;
`cargo test inner_tomi_watermark_cleanup_removes_pixels_and_space -- --ignored
--nocapture`; `cargo test audit_tier_a_completion_requires_all_lines`;
`cargo test audit_boundary_corrections_survive_redetection` if boundary fields
changed; `cargo test facsimile::units`; `cargo test facsimile::detect`;
`cargo test facsimile::reflow`. Run every amended ordinary regression by exact
name. A completed 4.2 has zero pending Tier-A events; record promoted event/line
counts in addition to the initial 39/69.

**Human verification:** Read every restored seam and complete multi-line block
against its source. Confirm exact missing Latin, neighbors and split words,
without duplication, reorder or clipping. Read the full Phocion 296→299 and
Brutus 623→624 seams and distinguish Brutus's separate 623 top loss.

## 4.3 Resolve the S2 line slice

Apply the step 1.1B decision for S2 (`comparatio_demosthenis_et_ciceronis`,
v2/480), rechecking its mechanism after the preceding repairs. If source connecting noise
merges lines, add the verified upstream mask that restores separate whole bands
(the O8B precedent), so each composes whole at production scale. If the trace
demonstrates another mechanism, record it and use the amendment gate below;
do not assume that every slice has a safe mask. Leave `compose_pages`,
`lowest_ink_row`, and `split_tall_strip` unchanged in the mask branch. Add
ignored `comparison_s2_line_is_whole` asserting the original source line intervals
each appear intact on one composed canvas. Reuse
`assert_reviewed_source_lines_survive` with whole-line ownership requested,
`strip_and_canvas_ranges`, `assert_scaled_stream`, and the assertion pattern
in `timoleon_reported_page_break_preserves_whole_lines`; add no independent
stream mapper. Check original glyph intervals and composed ownership; resampling
must not hide a split. Install any S2 mask and ledger record together in this
step. Only if source diagnosis proved no upstream mask
is safe may a packer change be considered — and then it requires an ordinary
`merged_strip_wraps_at_complete_line_boundary` plus
`unaffected_ordinary_line_packing_is_unchanged`, and this step must be amended
with the exact responsible function, identity scope, behavior and preservation
boundary before coding. Add both named ordinary regressions only if that branch
is selected; broad packer retuning remains outside scope.

**Measured S2 branch:** The source 480 chapter-IV band [1516, 1647] becomes
strip [1507, 1648] in crop [887, 1704] of raster [1852, 2836]. Skewed Latin glyph
projections overlap; there is no safely erasable connecting furniture. Add a
finite source-pixel predicate for this exact cropped strip: require column
dimensions (817, 2836), strip interval [1507, 1648], and SHA-256 of its unscaled
grayscale bytes equal to
`b9923339da9e250ebc0404036149d63cf9a253aafbb4c2f73957398188f608d0`.
The lossless sample in the diagnostic fixture supplies an ordinary synthetic
regression; production uses the hash to recognize the source, never the sample
to paint or replace text. In `compose_pages`, bypass the Tall fill-split branch
for this recognized strip when it fits a fresh canvas; wrap it whole. Preserve
the existing oversized-strip fallback, `lowest_ink_row`, `split_tall_strip`,
classification, scale, and all unrelated packing. The two ordinary regressions
above must test positive fingerprint recognition, whole cluster ownership,
rejection of changed pixels/geometry, and unchanged ordinary packing. The real
`comparison_s2_line_is_whole` checks every original chapter-IV line through
returned canvases.

**LLM verification:** `cargo fmt --all`; `cargo fmt --all --check`; `cargo clippy
--all-targets -- -D warnings`; `cargo test comparison_s2_line_is_whole -- --ignored --nocapture`;
`cargo test timoleon_reported_page_break_preserves_whole_lines -- --ignored
--nocapture`; run `cargo test merged_strip_wraps_at_complete_line_boundary`
and `cargo test unaffected_ordinary_line_packing_is_unchanged` if the amended
packer branch was selected; `cargo test facsimile::reflow`.

**Human verification:** Confirm the formerly sliced comparison line is whole on one
page.

# Phase 5 — Rebuild the readers and record the outcome (Suggested implementer: Sonnet)

## 5.1 Run final checks and verify all eight tomi

Use a workspace-local target dir if the configured external target is
inaccessible:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
cargo fmt --all
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
```

Check each command's exit status independently. An ordinary `cargo test` skips
ignored tests and is insufficient source verification; run every new ignored test
individually with `cargo test <name> -- --ignored --nocapture`, and re-run the
prior passes' ignored regressions (the D1–D3, watermark, head, divider, and
endpoint tests) to confirm no regression. A filter selecting zero tests is a
failed verification.

Then rebuild offline with the README's Windows LuaLaTeX setup:

```powershell
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
& $env:PLUTARCH_LUALATEX --version
if ($LASTEXITCODE -ne 0) { throw 'LuaLaTeX unavailable' }
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
if ($LASTEXITCODE -ne 0) { throw 'Facsimile build failed' }
```

Do not invoke `--retag`, `--review-lines`, or paid vision. Raw render caches need
no deletion (masks run on owned images). Verify all eight `tomus_N_inner.pdf` and
eight imposed PDFs compile fresh; "LuaLaTeX not run" is not a successful
verification. Pagination will shift (de-duplication removes 20 source pages), so
locate every repaired passage by source/prose landmark, not old output page
numbers. Inspect: each D4–D12 single retained occurrence and seam; each W4–W6
former watermark seam (pixels and space); H1/H2 and the two new head leaks; the
restored Tier-A omissions (every initial and promoted event, with full blocks);
F1/F2's repaired or deliberately deferred neighborhoods; the S2 line; a
sample of unaffected pages per tomus; the previously repaired D1–D3/W1–W3/O-series
neighborhoods; and the E01–E68 terminal endings. Review imposed sheets around
changed pagination and confirm signatures remain valid; do not require old page
counts.

Create `llm_artifacts/YYYY_MM_DD_HH_MM_INNER_TOMI_AUDIT_REPAIRS_VERIFICATION.md`
(fill the date and 24-hour time with the note’s actual creation time, per the
plan-naming convention) with
exact commands/results, durable source mappings, before/after output locators,
before/after copied heights for W4–W6, the production global scale (recorded and
confirmed **unchanged**, since `global_scale_for` samples unmasked rasters over
ranges that still include the excluded pages), each duplicate decision, each
restored omission with its event ID, exact source-line count and prose landmark,
every retained artifact (ambiguous rules, F1/F2 deferrals, unpromoted Tier-B
events, each M1–M11 copied-margin vs genuine-spacing classification, the B1–B4
absence re-check, W3/Praefatio non-findings), archival evidence
location/hashes, and any environment-dependent check that could not run.
Report initial Tier-A 39/69 and Tier-B 20/44 counts, promotions, repaired counts,
and unresolved counts separately. Require zero unresolved Tier-A events and a
final `audit_tier_a_source_lines_restored` run covering the entire Tier-A set;
a pending case or unavailable required check prevents a completion claim. Update this plan's "Measured repair decisions" in place without renumbering
steps. Preserve all prior plans and findings.

**LLM verification:** All commands and ignored tests above pass. Run `git diff
--check` and `git status --short`; confirm the diff contains only intended
facsimile code/tests, the new fixture, any pinned unit-row corrections, and the
documentation. Confirm source PDFs, raw caches, transcription caches, and
unrelated unit rows were not changed. Report any unavailable check as unverified
rather than passed.

**Human verification:** Open the rebuilt inner PDFs and compare every repaired
passage with the original scans: the nine de-duplicated seams, the three watermark
seams (space gone), the head leaks, the restored omitted lines, and the S2 line.
Spot-check imposed sheets and read the verification note before accepting the
repairs.

# Risks and explicit non-goals

- **Restoring Latin is higher-risk than removing furniture.** A wrong omission
  fix can duplicate or reorder prose. Every Tier-A fix is gated on a Phase 1
  pixel confirmation and a per-case regression that checks the line appears
  **once** and in order through returned canvases. All initial/promoted Tier-A
  losses must be restored; unsafe or unspecified corrections remain unresolved
  and prevent completion. Tier-B events are not repaired without a confirming
  human read and recorded promotion.
- **De-duplication must not drop a unique line.** The second occurrences are
  verified equal-or-worse re-scans whose openings survive in the first occurrence;
  C3 follows D6; C5 follows W6's watermark page. Neither is assumed repaired by
  page exclusion. Both have independent Phase 4 restoration assertions.
- **Watermark space** is removed by masking the tail (watermark + connecting rule)
  so `cut_points` ends at the last true body band; Phase 3.2 proves the copied
  height actually shrinks. No interior row-compaction is introduced unless a
  watermark is proven to be an interior gap and separately reviewed.
- **Out of scope:** the exploratory `lateral_clipping` (1,110), `rule` (68), and
  `foreign_scraps` (14) screens as bulk cleanup; any character-by-character
  collation; the W3 notice page; the Praefatio transition; genuine printed
  chapter/verse spacing; and global threshold/scale/packer retuning.

## Durable omission inventory

This frozen inventory records confirmed audit observations, not repair
coordinates or replacement prose. Source-region boxes retain the exact normalized
audit values and source-PDF input hashes; they locate lines for native inspection
and must not be treated as production band boundaries. Crop paths are relative
to `tmp/pdfs/inner_tomi_audit/`. Candidate crops and independent review evidence
may use different run folders; archive both sets. The initial eligibility tiers
are immutable historical fields. Record later confirming reads and promotions
separately in the diagnostic fixture.

The JSON contains exactly 59 confirmed events / 113 source lines, partitioned
into initial Tier A 39/69 and Tier B 20/44. Step 1.1 imports these records and
adds measured per-line production regions and pipeline traces. A missing crop
requires recovery or a fresh confirming read as specified above.

<details>
<summary>Frozen confirmed-event identities, regions, and evidence hashes</summary>

```json
{"schema_version":1,"baseline_id":"baseline_2026_10_06","candidates_sha256":"03f31fc68aa8829b90cdf3462578d9978b05d01a5a9c895416333dd4f2c78144","reviews_sha256":"339ac5a1663bf5ea577a293d1d22aaa412b567a082e66806943c110b126b57ec","excluded_high_confidence_false_positive":"miss_tiberius_gracchus_403_690",
"events": [
  {"event_id":"miss_aemilius_paulus_326_3162","unit_id":"aemilius_paulus","source_regions":[{"role":"source_pdf","page":326,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.4679552491638291,0.08719088521871741,0.8659716647803383,0.10445386164972141],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_aemilius_paulus_326_3162_0.png","sha256":"4130e902ba17a8728e26f437292e60738ae5158562d6f867797d5b8841418335"},{"path":"runs/step102-final-all-5/crops/miss_aemilius_paulus_326_3162_1.png","sha256":"c6deb3f60828eb03446d37946db012201241c8d4a3a9759f453d1b350e2c428e"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/miss_aemilius_paulus_326_3162_0.png","sha256":"4130e902ba17a8728e26f437292e60738ae5158562d6f867797d5b8841418335"},{"path":"runs/step92-development-final-2/crops/miss_aemilius_paulus_326_3162_1.png","sha256":"c6deb3f60828eb03446d37946db012201241c8d4a3a9759f453d1b350e2c428e"}]},
  {"event_id":"miss_agesilaus_110_1649","unit_id":"agesilaus","source_regions":[{"role":"source_pdf","page":110,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.498844745963067,0.07207059635667035,0.9121360304172613,0.08914570805067591],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_agesilaus_110_1649_0.png","sha256":"a7c70fb207b044bd09982e4b34486076157928589d659cf594eabc98b45b1091"},{"path":"runs/step102-final-all-5/crops/miss_agesilaus_110_1649_1.png","sha256":"eec83faa84279ffb830bce543ba318d1dad9a54309edecb2e0ce189bd1a6c346"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_context_miss_agesilaus_110_1649_0.png","sha256":"f4a1476ed4a6edc49344dd4dc5ab6d1bd4dcefe44209b77826f7b088d30d0c9a"},{"path":"runs/step92-development-final-2/crops/review_context_miss_agesilaus_110_1649_1.png","sha256":"346cdba35ad3f78e866f89ef67713ab0002c9aee4069f0d553e970be109cfb00"}]},
  {"event_id":"miss_agesilaus_110_2063","unit_id":"agesilaus","source_regions":[{"role":"source_pdf","page":110,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.4899558849395718,0.8859686341149446,0.8950486546719703,0.9057015110670648],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_agesilaus_110_2063_0.png","sha256":"27ba4c56cd91d70cd6d6d9419a14cc8a1c35ea3787879390218abc8777988c78"},{"path":"runs/step102-final-all-5/crops/miss_agesilaus_110_2063_1.png","sha256":"9cec742ba63ae70ef418449fe6c00c942e4c94696f209feeb4ca4708b5ed9d7b"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_context_miss_agesilaus_110_2063_0.png","sha256":"7a45c069422bc5563eb3c30796f071c099bc48204412895f099e655445be018e"},{"path":"runs/step92-development-final-2/crops/review_context_miss_agesilaus_110_2063_1.png","sha256":"1686056931565ca63452329649ca1c25192bb8b567a977600a9b4ea86b428f29"}]},
  {"event_id":"miss_alcibiades_242_229","unit_id":"alcibiades","source_regions":[{"role":"source_pdf","page":242,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.5099376489463482,0.8651861900792783,0.7616640612099215,0.8828634713044736],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_alcibiades_242_229_0.png","sha256":"4d82e4b352e2658f49969361e415bd29ee5603dbb19a1c845d9f53715efd98dc"},{"path":"runs/step102-final-all-5/crops/miss_alcibiades_242_229_1.png","sha256":"e4f00893d744cab2db15d26172e1888165b6f85bba57de7b6756ebc2c7ed82c3"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_alcibiades_242_229-native-0.png","sha256":"9f1d54aac7f7a50d5dc41527de92a69e1d31a358638ac7d49a51e5d26883bfac"},{"path":"runs/step101-validation-expanded-review/miss_alcibiades_242_229-native-1.png","sha256":"89424854a7f9367d608be762767f1fbaddc183f3577e8c9b3f54a40b3f396b5d"},{"path":"runs/step101-validation-expanded-review/miss_alcibiades_242_229-native-2.png","sha256":"556c636d4f348ce002d3c03e54f05f22c0fe55bd7d99ba762a78698137eae106"},{"path":"runs/step101-validation-expanded-review/miss_alcibiades_242_229-native-3.png","sha256":"55b73e95c05f5aa6e993e1e50a23d4e3c132fd7759598f98dd11a9041eb7a38e"},{"path":"runs/step101-validation-first/crops/miss_alcibiades_242_229_0.png","sha256":"4d82e4b352e2658f49969361e415bd29ee5603dbb19a1c845d9f53715efd98dc"},{"path":"runs/step101-validation-first/crops/miss_alcibiades_242_229_1.png","sha256":"e4f00893d744cab2db15d26172e1888165b6f85bba57de7b6756ebc2c7ed82c3"}]},
  {"event_id":"miss_alcibiades_251_4338","unit_id":"alcibiades","source_regions":[{"role":"source_pdf","page":251,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.5341224492776611,0.8555064909264487,0.9226540862163817,0.8733243968974308],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_alcibiades_251_4338_0.png","sha256":"ba8188df722c393a88a96b6c0b97494b479f2ee8ffccab46b6b85527caf2c7fb"},{"path":"runs/step102-final-all-5/crops/miss_alcibiades_251_4338_1.png","sha256":"dbe762f2b49a79b1313ec7f3fa36f7b11406ae42e981c1e3a3cc95fac531eb96"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_alcibiades_251_4338-native-0.png","sha256":"bcbb38914fea9f45da486973605c235ae05fffe079b8ef6983de48cc1909af70"},{"path":"runs/step101-validation-expanded-review/miss_alcibiades_251_4338-native-1.png","sha256":"5076a66527b08c33aec2d5ab23ccb7349f4e8b6471ecb595e704dd6fab6c78cc"},{"path":"runs/step101-validation-expanded-review/miss_alcibiades_251_4338-native-2.png","sha256":"f2d96c12a984305ac2a515fa51b63549500319668f3f236ba5f96067bfd5d0c3"},{"path":"runs/step101-validation-expanded-review/miss_alcibiades_251_4338-native-3.png","sha256":"ebcc4be75124d82d1e005056005bb0a88d2c16c91753add6416312836482ac09"},{"path":"runs/step101-validation-first/crops/miss_alcibiades_251_4338_0.png","sha256":"ba8188df722c393a88a96b6c0b97494b479f2ee8ffccab46b6b85527caf2c7fb"},{"path":"runs/step101-validation-first/crops/miss_alcibiades_251_4338_1.png","sha256":"dbe762f2b49a79b1313ec7f3fa36f7b11406ae42e981c1e3a3cc95fac531eb96"}]},
  {"event_id":"miss_alcibiades_263_9839","unit_id":"alcibiades","source_regions":[{"role":"source_pdf","page":263,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.571759306978655,0.8585977160653594,0.8786616680002053,0.8749106840964345],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_alcibiades_263_9839_0.png","sha256":"2d6e8417ca0d9d31fc031e7439d42794515b4bc7093dbd93381c7327d9625820"},{"path":"runs/step102-final-all-5/crops/miss_alcibiades_263_9839_1.png","sha256":"98ece3e003b39edb3385e68b0835b0235bcc61a9244e798ebfc224f15ab89d2d"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_alcibiades_263_9839-native-0.png","sha256":"382ca834315c9447958a03fb4086df4af15b36940da3fcf00e3db4624a456eda"},{"path":"runs/step101-validation-expanded-review/miss_alcibiades_263_9839-native-1.png","sha256":"ec6779bb70190ea19d0394108f6cec1f233943aef28aa211eb966a092e32c691"},{"path":"runs/step101-validation-expanded-review/miss_alcibiades_263_9839-native-2.png","sha256":"392b8e21759dd0c85991a328b1f09c57297b9ccd37364a45f52bca5363949f48"},{"path":"runs/step101-validation-expanded-review/miss_alcibiades_263_9839-native-3.png","sha256":"f127f118acb6397794695b8c09d9728147d7fa8a1a77d0285b5aa6da4c27a97f"},{"path":"runs/step101-validation-first/crops/miss_alcibiades_263_9839_0.png","sha256":"2d6e8417ca0d9d31fc031e7439d42794515b4bc7093dbd93381c7327d9625820"},{"path":"runs/step101-validation-first/crops/miss_alcibiades_263_9839_1.png","sha256":"98ece3e003b39edb3385e68b0835b0235bcc61a9244e798ebfc224f15ab89d2d"}]},
  {"event_id":"miss_alexander_231_16439","unit_id":"alexander","source_regions":[{"role":"source_pdf","page":231,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5284416576985236,0.06587408707919173,0.941948303855864,0.08593853812131239],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_alexander_231_16439_0.png","sha256":"25d4c6292e331f5b40bfe95bede387dc5318aab19bb631246849b41f7300d709"},{"path":"runs/step102-final-all-5/crops/miss_alexander_231_16439_1.png","sha256":"1c234756f03c2a5d9b3341ec42565ab50fd1a829741ffe776433a0eb5729fb7b"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_context_miss_alexander_231_16439_0.png","sha256":"4b0c3ac2dff3deb5c9d6fcab3de08f94aca088d82bb25a2baea507ad9f6617b4"},{"path":"runs/step92-development-final-2/crops/review_context_miss_alexander_231_16439_1.png","sha256":"f8fc0dbd6c5b2dc0739469fce2dd36efd2837d29b9acc05d7b7dd96d655cee5f"}]},
  {"event_id":"miss_antonius_562_20869","unit_id":"antonius","source_regions":[{"role":"source_pdf","page":562,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.4830615272762883,0.08341532353882286,0.9330775439154376,0.4789919609231451],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":23,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_antonius_562_20869_0.png","sha256":"e87a6c284f2ab53499bf2cc0d590776815df6bdb0ac40cba4484aa56cd28e8f7"},{"path":"runs/step102-final-all-5/crops/miss_antonius_562_20869_1.png","sha256":"a1529c7a62ecb81b88ea2877bc5635d45c58e24e86a39fcb8cf8801224ed15ca"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_antonius_562_20869-native-0.png","sha256":"dfee7197a4517d7d68dfe3e5e4d40a6e18c7f4065d577a02912f1f8cbb3fc861"},{"path":"runs/step101-validation-expanded-review/miss_antonius_562_20869-native-1.png","sha256":"7785427086a57291400658e77b21a883c79f0a20adfd762f05f8f07955c13bbe"},{"path":"runs/step101-validation-expanded-review/miss_antonius_562_20869-native-2.png","sha256":"dbbd065674e0c1f3545d1839edb2a555d41fde9f13cd96132b3c6edf020a399b"},{"path":"runs/step101-validation-expanded-review/miss_antonius_562_20869-native-3.png","sha256":"b5c69d40a9fef88fedf56c363539f03ca44a2b21ac6a637d585f67954c7883a7"},{"path":"runs/step101-validation-first/crops/miss_antonius_562_20869_0.png","sha256":"e87a6c284f2ab53499bf2cc0d590776815df6bdb0ac40cba4484aa56cd28e8f7"},{"path":"runs/step101-validation-first/crops/miss_antonius_562_20869_1.png","sha256":"a1529c7a62ecb81b88ea2877bc5635d45c58e24e86a39fcb8cf8801224ed15ca"}]},
  {"event_id":"miss_artoxerxes_633_2282","unit_id":"artoxerxes","source_regions":[{"role":"source_pdf","page":633,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.530053769266641,0.8871780994798188,0.9509384985074305,0.9064751224860704],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_artoxerxes_633_2282_0.png","sha256":"ec6f164de1ebafda275af5792ba56c57dd75b69f4bb6cfedd98b1e966a9a53c5"},{"path":"runs/step102-final-all-5/crops/miss_artoxerxes_633_2282_1.png","sha256":"0354bc428cad7238203893ed7a18a1a7379bcfaf3395eda5d4cf1e9326aef11d"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_artoxerxes_633_2282_0.png","sha256":"c463877c2ba43779730bbc05b4705f1ce5a4683ff20280feaf23a4b04b35bebb"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_artoxerxes_633_2282_1.png","sha256":"a89f3f13d3efffb6e6f513fb9cd398763057ccf4a30691c9f90efd2c78d0cfd2"}]},
  {"event_id":"miss_brutus_619_10158","unit_id":"brutus","source_regions":[{"role":"source_pdf","page":619,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5559367151073668,0.06324651024065285,0.9864616630949534,0.08256490051393053],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_brutus_619_10158_0.png","sha256":"6fccb5506c39a7c84fb0de160a3670f1357a324c8205a77da26afee52fdd3e45"},{"path":"runs/step102-final-all-5/crops/miss_brutus_619_10158_1.png","sha256":"8d60afc73b93f70163c5113d9af030c28867606cf3fd5f50a260c4ad66ae2d12"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_brutus_619_10158-native-0.png","sha256":"f324365c47293c91f02228cd598a77622d69a9302b88e5b3d84b5b9637e0c556"},{"path":"runs/step101-validation-expanded-review/miss_brutus_619_10158-native-1.png","sha256":"f4c3e1a1846b8c7c021ef0b162ab47bdf29a2df2f2015b91cc7117585395c9e9"},{"path":"runs/step101-validation-expanded-review/miss_brutus_619_10158-native-2.png","sha256":"8e8f6afe983d264669f9de474688d4dbeb21cf3dbd4959603f1382911e4aa7d8"},{"path":"runs/step101-validation-expanded-review/miss_brutus_619_10158-native-3.png","sha256":"813f57f9ea27cb323da7d6f46ffe077c234ccc53bbdfdbe087fb7167759339ea"},{"path":"runs/step101-validation-first/crops/miss_brutus_619_10158_0.png","sha256":"6fccb5506c39a7c84fb0de160a3670f1357a324c8205a77da26afee52fdd3e45"},{"path":"runs/step101-validation-first/crops/miss_brutus_619_10158_1.png","sha256":"8d60afc73b93f70163c5113d9af030c28867606cf3fd5f50a260c4ad66ae2d12"}]},
  {"event_id":"miss_brutus_623_11958","unit_id":"brutus","source_regions":[{"role":"source_pdf","page":623,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5276243097803992,0.06963735052950734,0.9741368439414352,0.12439078266078304],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":3,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_brutus_623_11958_0.png","sha256":"e16b2836cac32e3fd060284553f51e33af69f00e4308ec329755615f986b90e9"},{"path":"runs/step102-final-all-5/crops/miss_brutus_623_11958_1.png","sha256":"4a56d101ea412977faf4d138807cf2792447297b421b390d3d7f721ca222d4e7"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_brutus_623_11958-native-0.png","sha256":"788b0a05815894c81948a959792320156caa7a24063b3009b10c83ba2f497266"},{"path":"runs/step101-validation-expanded-review/miss_brutus_623_11958-native-1.png","sha256":"5fd7d4817a09f84761b50689bb39e87767635f3ea05456fe69cd7d4463500c59"},{"path":"runs/step101-validation-expanded-review/miss_brutus_623_11958-native-2.png","sha256":"6d892944815334b7a9db3560728b6ccc0efb2e05750efc4dbb02211d6a4d7b2b"},{"path":"runs/step101-validation-expanded-review/miss_brutus_623_11958-native-3.png","sha256":"6e6e7007ccd2c62cb53e461f35bc7928ec8ceac23e1856dfda8636e1d9d41443"},{"path":"runs/step101-validation-first/crops/miss_brutus_623_11958_0.png","sha256":"e16b2836cac32e3fd060284553f51e33af69f00e4308ec329755615f986b90e9"},{"path":"runs/step101-validation-first/crops/miss_brutus_623_11958_1.png","sha256":"4a56d101ea412977faf4d138807cf2792447297b421b390d3d7f721ca222d4e7"}]},
  {"event_id":"miss_brutus_624_12394","unit_id":"brutus","source_regions":[{"role":"source_pdf","page":624,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.47154020240885064,0.0652380857349025,0.9166666784929846,0.0862386070363925],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_brutus_624_12394_0.png","sha256":"ee5aecaa2a067eaeddb3378fb81df62a6cb13c2a2bb26033dab45593940ee88d"},{"path":"runs/step102-final-all-5/crops/miss_brutus_624_12394_1.png","sha256":"7a0929e9f65c64c6ae172a6a44208e995ed615b8bf09243affb5e0e50e6f2d56"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_brutus_624_12394-native-0.png","sha256":"b562f0952d2d89f1ed5eb9ec4d9e2c1ee0cfbca4741b411eb80e865e81e43bda"},{"path":"runs/step101-validation-expanded-review/miss_brutus_624_12394-native-1.png","sha256":"6e6e7007ccd2c62cb53e461f35bc7928ec8ceac23e1856dfda8636e1d9d41443"},{"path":"runs/step101-validation-expanded-review/miss_brutus_624_12394-native-2.png","sha256":"1c13b7074d9ec3b0b0a7ca41fe207330de59d79c76ce06ff8152020503deb2cf"},{"path":"runs/step101-validation-expanded-review/miss_brutus_624_12394-native-3.png","sha256":"949529a7356088f455491965aee0232387451bb6d71c1ad8fa4f2a630666e6ac"},{"path":"runs/step101-validation-first/crops/miss_brutus_624_12394_0.png","sha256":"ee5aecaa2a067eaeddb3378fb81df62a6cb13c2a2bb26033dab45593940ee88d"},{"path":"runs/step101-validation-first/crops/miss_brutus_624_12394_1.png","sha256":"7a0929e9f65c64c6ae172a6a44208e995ed615b8bf09243affb5e0e50e6f2d56"}]},
  {"event_id":"miss_caius_marius_519_9217","unit_id":"caius_marius","source_regions":[{"role":"source_pdf","page":519,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.5446588702783407,0.07829237967617003,0.9416907133794427,0.09642810690735712],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_caius_marius_519_9217_0.png","sha256":"8c4ffe35b6d1cbc356387a027797635f58343ee367061824127e745ccac8921e"},{"path":"runs/step102-final-all-5/crops/miss_caius_marius_519_9217_1.png","sha256":"11ad1720ba7c1c7c8bab443b05513c6f7312733dfa373b484aa40425406fea4f"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_context_miss_caius_marius_519_9217_0.png","sha256":"95d8f4a0c2c257e9ecc0f54ed7e251b950f5e4e1a776d5d1d1b18e1ad8f33a3d"},{"path":"runs/step92-development-final-2/crops/review_context_miss_caius_marius_519_9217_1.png","sha256":"e5c671e9ada3ede9d397ca1a19d6b63a5418d8e9f857395c4bd3abd3832a3862"}]},
  {"event_id":"miss_cato_minor_327_8321","unit_id":"cato_minor","source_regions":[{"role":"source_pdf","page":327,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5334360358455226,0.8805996242305748,0.9526849812423445,0.9010384827030263],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_cato_minor_327_8321_0.png","sha256":"33f4ada2bf62ad19ea8536a89cbd6e076148000634445ca390e6d1b43e4aa5b1"},{"path":"runs/step102-final-all-5/crops/miss_cato_minor_327_8321_1.png","sha256":"d64e44223d3e576d628247c8c35f4cb35de0448c360461f58ee06a2680f138fb"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_context_miss_cato_minor_327_8321_0.png","sha256":"c621e5c935cd7065bbc068e499a3264c8499dd8e10d455616b36d7df944ddad1"},{"path":"runs/step92-development-final-2/crops/review_context_miss_cato_minor_327_8321_1.png","sha256":"47a3e6ab74cebb5b078fd6c182dd2e906e4db1af5d888d0f29c03831f59c6da1"}]},
  {"event_id":"miss_cato_minor_329_8786","unit_id":"cato_minor","source_regions":[{"role":"source_pdf","page":329,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5472670394086262,0.05642608882645985,0.9622312150938959,0.0774345563758777],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_cato_minor_329_8786_0.png","sha256":"733b73e22d8881c27e51b5c8e9f7d6bfc3efa4f3a19a280abf41e07808df31e1"},{"path":"runs/step102-final-all-5/crops/miss_cato_minor_329_8786_1.png","sha256":"336583c2af47c0e9cb2e548c7bb7405849ed47c4fc0abf83268803cc74974262"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_context_miss_cato_minor_329_8786_0.png","sha256":"d69378e319db0b725de6ce58405a1f95fc5ef49ec705b1560f6ee3be7ce6d315"},{"path":"runs/step92-development-final-2/crops/review_context_miss_cato_minor_329_8786_1.png","sha256":"b06fef9df25a3c52bcb759cfbfa75de2f2920b056861665f5ee4e3c17f58c4f1"}]},
  {"event_id":"miss_cato_minor_331_9707","unit_id":"cato_minor","source_regions":[{"role":"source_pdf","page":331,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5531566980051926,0.05392827795585274,0.9731429255687992,0.07310875874929544],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_cato_minor_331_9707_0.png","sha256":"d4b80094ca48ba989e5f8ec07ffe5fb8099f648e40c43cfa90813f69eedd5550"},{"path":"runs/step102-final-all-5/crops/miss_cato_minor_331_9707_1.png","sha256":"f4b016efae6b380ead99bfa56f4ff935d79f728ab12936c301743e0d5af2812b"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_context_miss_cato_minor_331_9707_0.png","sha256":"83aeead5d7cc8c42345c70f9356d0f4ec1ada51c585d28836baea967a56e9d9c"},{"path":"runs/step92-development-final-2/crops/review_context_miss_cato_minor_331_9707_1.png","sha256":"1dc311b0c04bb29eb41fde1d75ac933fa24a93e52277ba814e9a5a290d92ea50"}]},
  {"event_id":"miss_cato_minor_352_15055","unit_id":"cato_minor","source_regions":[{"role":"source_pdf","page":352,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.4781116622387688,0.8839766563631503,0.9022129066708913,0.9036588016381669],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_cato_minor_352_15055_0.png","sha256":"48953517a9509d1b9d08c84271c18292ded25b45fccf5c4c0e35f68e0cbc3de5"},{"path":"runs/step102-final-all-5/crops/miss_cato_minor_352_15055_1.png","sha256":"39cb690b9d18952fd4b5802a2f45ca3817191e1cc52700c1a4f1b2055868de3b"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_context_miss_cato_minor_352_15055_0.png","sha256":"701be788f6829163da5d564c2daeea75c87aab045bb9a7ade5fa457fc22c9260"},{"path":"runs/step92-development-final-2/crops/review_context_miss_cato_minor_352_15055_1.png","sha256":"ebfaa6b3a3789609bd81bcf3d9f2a499b9848f3b3bf0fce7a5f5d9d4d4162925"}]},
  {"event_id":"miss_cleomenes_391_5043","unit_id":"cleomenes","source_regions":[{"role":"source_pdf","page":391,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5257991285628925,0.06739093701748017,0.9636532160789455,0.08513420802653715],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_cleomenes_391_5043_0.png","sha256":"be2ebdb4a5346fd456fa5411c40850c161e2c8db87e89f78222479f35be6be21"},{"path":"runs/step102-final-all-5/crops/miss_cleomenes_391_5043_1.png","sha256":"7060203992b0b8253412efdefd4ebe28f482fb7a3cfa7c841acc4f3dbf1a7deb"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_context_miss_cleomenes_391_5043_0.png","sha256":"76d8f746a600d3d4738ad12cb68ae15003988151039525d4697a5a1c923aefc0"},{"path":"runs/step92-development-final-2/crops/review_context_miss_cleomenes_391_5043_1.png","sha256":"120ed7d20d412dcc5a52fc6c4d34d2d7050423627240710a9c31dcd4f1292997"}]},
  {"event_id":"miss_cleomenes_393_5916","unit_id":"cleomenes","source_regions":[{"role":"source_pdf","page":393,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5571753185723837,0.06689888873867056,0.969901311398218,0.08592642615463875],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_cleomenes_393_5916_0.png","sha256":"15d34824f9f16f4967c6869e7a215bb59cadc423e9ecbd44199a5d64c7044d8a"},{"path":"runs/step102-final-all-5/crops/miss_cleomenes_393_5916_1.png","sha256":"14580c0b14b6d86edc6e6aa75728d257022a15530164b48df1c1350e2da184a8"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_context_miss_cleomenes_393_5916_0.png","sha256":"1d01187b6f90553a3b7b28243a861d9f3ce77135720d9bcdbf55652beda13125"},{"path":"runs/step92-development-final-2/crops/review_context_miss_cleomenes_393_5916_1.png","sha256":"7c16ac78f7a5310718916ba5acbee8dcd8d1935521a51bbe75cdbea1fa9672de"}]},
  {"event_id":"miss_crassus_44_1144","unit_id":"crassus","source_regions":[{"role":"source_pdf","page":44,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.4980276024843344,0.06255041638479769,0.8978485367520462,0.08152775664261223],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_crassus_44_1144_0.png","sha256":"2ad7fbf5ae16cdbc8cf69af2032d3b972c92c9c797e5bc7b5d1b1397d0acff6d"},{"path":"runs/step102-final-all-5/crops/miss_crassus_44_1144_1.png","sha256":"f2f7ca7192ff373a160c107aff43ce28da8519eba665b69a54bd716c4fb294e4"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_crassus_44_1144-native-0.png","sha256":"27cf4bbcecdb5c9d5c1352e73015192031dc2e57328e912abd646236b96d05ca"},{"path":"runs/step101-validation-expanded-review/miss_crassus_44_1144-native-1.png","sha256":"f10d75253b0bcd71c19f3984ca74629b6f466e4594792b0da35dac81e471d53d"},{"path":"runs/step101-validation-expanded-review/miss_crassus_44_1144-native-2.png","sha256":"4681b302628c8ec9a5c26b7281f8516035d7c27aa6b94fa3669a8b6038e03ac7"},{"path":"runs/step101-validation-expanded-review/miss_crassus_44_1144-native-3.png","sha256":"26494533bcb0597ab561d5b6752efb262d3a662336fe4c9453664ec4d930f637"},{"path":"runs/step101-validation-first/crops/miss_crassus_44_1144_0.png","sha256":"2ad7fbf5ae16cdbc8cf69af2032d3b972c92c9c797e5bc7b5d1b1397d0acff6d"},{"path":"runs/step101-validation-first/crops/miss_crassus_44_1144_1.png","sha256":"f2f7ca7192ff373a160c107aff43ce28da8519eba665b69a54bd716c4fb294e4"}]},
  {"event_id":"miss_crassus_67_11548","unit_id":"crassus","source_regions":[{"role":"source_pdf","page":67,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5639326433463304,0.0708501223606696,0.9716817600622731,0.08921109977534325],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_crassus_67_11548_0.png","sha256":"3893684331bd0812f56bf6072e6a71fe52593604c0746ddc839aa04da2c91ae1"},{"path":"runs/step102-final-all-5/crops/miss_crassus_67_11548_1.png","sha256":"6f7c2e6a0e814e1363da3a47e4daff9e5179810fa9ef9a11bbaa097aa5ef635f"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_crassus_67_11548-native-0.png","sha256":"02a3d331aba415e3b4012f40a6f20e7d219b422b0410967a2eaa3daf65b8bec4"},{"path":"runs/step101-validation-expanded-review/miss_crassus_67_11548-native-1.png","sha256":"bc43f8f0377cb9f9a490df9f406ec2e3f3b0a8186e305dfaa001e9ee4b079083"},{"path":"runs/step101-validation-expanded-review/miss_crassus_67_11548-native-2.png","sha256":"573da823af8ade8363e4a94df15c69a9958e13563255c8502af8b329f845b11d"},{"path":"runs/step101-validation-expanded-review/miss_crassus_67_11548-native-3.png","sha256":"87f057b6206b1155a672706e37cded5fc60ba9555f271968f71469db20d4bd61"},{"path":"runs/step101-validation-first/crops/miss_crassus_67_11548_0.png","sha256":"3893684331bd0812f56bf6072e6a71fe52593604c0746ddc839aa04da2c91ae1"},{"path":"runs/step101-validation-first/crops/miss_crassus_67_11548_1.png","sha256":"6f7c2e6a0e814e1363da3a47e4daff9e5179810fa9ef9a11bbaa097aa5ef635f"}]},
  {"event_id":"miss_demosthenes_437_3468","unit_id":"demosthenes","source_regions":[{"role":"source_pdf","page":437,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5223611505093895,0.07015785695915666,0.9598317140381897,0.1370423953825764],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":4,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_demosthenes_437_3468_0.png","sha256":"0873b4a17caf6ff34694fef2963d14928aab473f8b3cbc52a47e3bf1612d8550"},{"path":"runs/step102-final-all-5/crops/miss_demosthenes_437_3468_1.png","sha256":"d70921982b616493d087a0b416cd56e7ab7055baa68514f862138cde40d95151"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_demosthenes_437_3468_0.png","sha256":"433557d7449d0ba08430fb1595ea5bd0ec4c9fe871f7709f04b278dd3b9618dc"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_demosthenes_437_3468_1.png","sha256":"929f7f29d0f2ad94ba56ddb1d5870abae8d603ed4383ebc7f3b59b90d4633f4f"}]},
  {"event_id":"miss_demosthenes_438_4274","unit_id":"demosthenes","source_regions":[{"role":"source_pdf","page":438,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.4576739965065111,0.8800463817512592,0.8957193703980029,0.9004224200849463],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_demosthenes_438_4274_0.png","sha256":"a46de0a4422fe78311d89015c8cc68b07867c6ec97faeb0c521e93ba9d377751"},{"path":"runs/step102-final-all-5/crops/miss_demosthenes_438_4274_1.png","sha256":"92408149cfcd42e2b08eb260abde2100bad7d5084dc4c4db0e4d7662935a3358"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_demosthenes_438_4274_0.png","sha256":"4ac7c2eeedf14852dc65f5a49932771656db0f00c00f4d7657fa30a248a897ef"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_demosthenes_438_4274_1.png","sha256":"77b3f3d6be99f20802ba12e21b2f2cdef618f6426549c824bdb1b01ae7b17e25"}]},
  {"event_id":"miss_demosthenes_443_6045","unit_id":"demosthenes","source_regions":[{"role":"source_pdf","page":443,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5280035984275591,0.06473154858152068,0.9601174278865454,0.1021550686304865],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":2,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_demosthenes_443_6045_0.png","sha256":"43ba1b146a41d33e278dc43ac82b427a103a6abb22bce40bacc274390e014b3d"},{"path":"runs/step102-final-all-5/crops/miss_demosthenes_443_6045_1.png","sha256":"64f020417d04a3dbd5eb7948e479cd36f3b644f5b7d96ae65fb215e88ffc833f"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_demosthenes_443_6045_0.png","sha256":"78a706656758505bc6d8549fe8e29d671ca8c7d7136f4447b497e469e5ff7bff"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_demosthenes_443_6045_1.png","sha256":"0f9843abc879fca4a90136ee4fe72598db3300ea53a8cc5d6fdebcf4c807b99e"}]},
  {"event_id":"miss_demosthenes_445_6929","unit_id":"demosthenes","source_regions":[{"role":"source_pdf","page":445,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5496021027267806,0.06301653160738169,0.9758997241744172,0.08253526659062038],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_demosthenes_445_6929_0.png","sha256":"c5727f2a9f7dc5ab9a43af0fc9934708bf2e76f532c16fd06fc9d6ecb5b4822f"},{"path":"runs/step102-final-all-5/crops/miss_demosthenes_445_6929_1.png","sha256":"fa8b9cf893401807572827d56a6fec5106f188b5377bae97f5a1dea0c171493f"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_demosthenes_445_6929_0.png","sha256":"63f884288ddd84dccede2b6dbaf35cc67dc9b22eaa71f6ff7e1e375f92e5e795"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_demosthenes_445_6929_1.png","sha256":"e20d1ea017029cdfbc786c11ee7c8f88b3fc8f853ddce0acfb077160caeb0a99"}]},
  {"event_id":"miss_dio_566_426","unit_id":"dio","source_regions":[{"role":"source_pdf","page":566,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.48726490419608276,0.8891614252289759,0.9138988591557085,0.9082826290137436],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_dio_566_426_0.png","sha256":"1410ab136c63db2a096d4e5111e5c825206f37544478deb653f9a2896318e2af"},{"path":"runs/step102-final-all-5/crops/miss_dio_566_426_1.png","sha256":"799aff08d462016343727e84f5295bb7bfe35dc7f0324db91952efa58bf59f5b"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_dio_566_426_0.png","sha256":"d5e8e94ec7f2fe5fe210e067ec78e3afd58c544845defb3ca80432e7996213d3"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_dio_566_426_1.png","sha256":"ccbde3fb76420716ba1bf74dac6757fb6fbe136a7b6d4e86d4e57030f0392320"}]},
  {"event_id":"miss_dio_577_4979","unit_id":"dio","source_regions":[{"role":"source_pdf","page":577,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5544976623390999,0.06446891907696677,0.9641367950177867,0.08421729223792794],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_dio_577_4979_0.png","sha256":"afa61065fbce1fac2231d2a52cdafa157eedf596c4a4d714841fac659f05c44d"},{"path":"runs/step102-final-all-5/crops/miss_dio_577_4979_1.png","sha256":"c6d8a2ff388ba87d16e1e9b2d43c9536ee7b9bc452930eb74bb9cf38f9209964"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_dio_577_4979_0.png","sha256":"6ed864d25dee0ee9b395b2ef22dde202f1a8753acd9ecac28c852dacb79f0558"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_dio_577_4979_1.png","sha256":"15ccfa8b52cd816a9d6f54ad9e8e1c2b2459e8f4d581064ff82f77926465221e"}]},
  {"event_id":"miss_dio_579_5825","unit_id":"dio","source_regions":[{"role":"source_pdf","page":579,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5596539464920939,0.06790143142101404,0.9772313287411046,0.08736918840328442],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_dio_579_5825_0.png","sha256":"93fdca2b8f30eb31b12cd3713aaa3163b1024dfde6d28ff4d4dee0671ef428da"},{"path":"runs/step102-final-all-5/crops/miss_dio_579_5825_1.png","sha256":"592b436075224842f0839a44262cb422741a7d2a917a5fd0bcf5d679902d4ab3"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_dio_579_5825_0.png","sha256":"cdf8ec664d0ddfd8ec43c9ff1ddd113660769a5040a297036ae11d2534e130b5"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_dio_579_5825_1.png","sha256":"9f9df412fd99e8474de5e09584011375e8fa3f3e5ae431df4680113d1e4da36a"}]},
  {"event_id":"miss_dio_580_6654","unit_id":"dio","source_regions":[{"role":"source_pdf","page":580,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.4927211962521019,0.8869995558714361,0.9177091790453524,0.9071486937228963],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_dio_580_6654_0.png","sha256":"8b1ca9d44efd22e5c51caa7d4ec1648c27d5a595457650e3cdddf997dbb6c72d"},{"path":"runs/step102-final-all-5/crops/miss_dio_580_6654_1.png","sha256":"ebd3cb8abb930b0a65d6c7a5845b67ad04641b50eff25864589c2da503888a96"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_dio_580_6654_0.png","sha256":"48f5a60e0352a977ffb39c8fec0ec8169651d040141b455af569f3cf2ec72e47"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_dio_580_6654_1.png","sha256":"3bca4c799e50b9cf16f4577f69dd25ad07d269287bba878654a380a66ffb52ee"}]},
  {"event_id":"miss_eumenes_91_1126","unit_id":"eumenes","source_regions":[{"role":"source_pdf","page":91,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5418208969696188,0.07242067397473279,0.9536400973974996,0.1049728940363293],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":2,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_eumenes_91_1126_0.png","sha256":"7904799956f73ec1dc1c4dac2f007dd1ef21110df8909a097ad34e5fa5883908"},{"path":"runs/step102-final-all-5/crops/miss_eumenes_91_1126_1.png","sha256":"fb7873598b77cca11f71f98f07bc0686ec3c890aa712bde30f4a236b90766d2b"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_eumenes_91_1126-native-0.png","sha256":"ecc73fbf7889d2326069741b40cab21849917f66ef25853be11ed8543c5afa0b"},{"path":"runs/step101-validation-expanded-review/miss_eumenes_91_1126-native-1.png","sha256":"a5a68f3ae1f598794fe5dfb7b62bd8e58cd74c72f73fef11eb10ac526256e9c5"},{"path":"runs/step101-validation-expanded-review/miss_eumenes_91_1126-native-2.png","sha256":"e76dfa5b0a28efdc6b8e3277b5e515875571704d3c265f7328c8d49adbfe9e8f"},{"path":"runs/step101-validation-expanded-review/miss_eumenes_91_1126-native-3.png","sha256":"314cbfe83404667a4d783eb3cb70ca041912a0fd49fd730206b486a460d3cf17"},{"path":"runs/step101-validation-first/crops/miss_eumenes_91_1126_0.png","sha256":"7904799956f73ec1dc1c4dac2f007dd1ef21110df8909a097ad34e5fa5883908"},{"path":"runs/step101-validation-first/crops/miss_eumenes_91_1126_1.png","sha256":"fb7873598b77cca11f71f98f07bc0686ec3c890aa712bde30f4a236b90766d2b"}]},
  {"event_id":"miss_fabius_maximus_228_2889","unit_id":"fabius_maximus","source_regions":[{"role":"source_pdf","page":228,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.4626081316503085,0.08130976686945944,0.8612739198747805,0.09918015558069988],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_fabius_maximus_228_2889_0.png","sha256":"c06ddf6323cb647407a279dea5e8a9b0cd374b81ecd0b9773a7fd0e2e435b8d5"},{"path":"runs/step102-final-all-5/crops/miss_fabius_maximus_228_2889_1.png","sha256":"970e91109763c0d7122549c462587ba7ed67379ed8c96f895ac507b6cec6851f"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_fabius_maximus_228_2889_0.png","sha256":"e094b2712341ef417102c1fc0c50c0dbe0b6d8fc0245c033cbc363823b831102"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_fabius_maximus_228_2889_1.png","sha256":"e514dbb7dc062b9646a4ddb888080baeca37b68742494351acb0bf59b1590716"}]},
  {"event_id":"miss_iulius_caesar_252_2835","unit_id":"iulius_caesar","source_regions":[{"role":"source_pdf","page":252,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.4829227489841553,0.07403069173265703,0.9157422086908199,0.2656726317627415],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":12,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_iulius_caesar_252_2835_0.png","sha256":"e8b79c7498be3019d4222709b665b3afeab08d9edad51077cbeb6975a9e7d93d"},{"path":"runs/step102-final-all-5/crops/miss_iulius_caesar_252_2835_1.png","sha256":"52774885eeacc7ac6900c5cb750c9861645d70cb4973d41a71f692d0c965394e"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_iulius_caesar_252_2835-native-0.png","sha256":"6e6eb0321fc80bc78291a140783388adcc036ffe54971097005b342a17c9de4e"},{"path":"runs/step101-validation-expanded-review/miss_iulius_caesar_252_2835-native-1.png","sha256":"904ba15d5c9390c092d6968bb0018d14f9e835a68c5c6dd716d76b50d18e212b"},{"path":"runs/step101-validation-expanded-review/miss_iulius_caesar_252_2835-native-2.png","sha256":"c463c381a61ca829158c187289da9b58b33dcbcd156be9a56428b6e0fb2f3841"},{"path":"runs/step101-validation-expanded-review/miss_iulius_caesar_252_2835-native-3.png","sha256":"330581e63c4b994272b59f43c459edef4ffaf63f9ac95f0b324d93ca72c6dece"},{"path":"runs/step101-validation-first/crops/miss_iulius_caesar_252_2835_0.png","sha256":"e8b79c7498be3019d4222709b665b3afeab08d9edad51077cbeb6975a9e7d93d"},{"path":"runs/step101-validation-first/crops/miss_iulius_caesar_252_2835_1.png","sha256":"52774885eeacc7ac6900c5cb750c9861645d70cb4973d41a71f692d0c965394e"}]},
  {"event_id":"miss_lucullus_607_2890","unit_id":"lucullus","source_regions":[{"role":"source_pdf","page":607,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.5284839669883896,0.07801535757955988,0.9212007722987652,0.09837695999639644],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_lucullus_607_2890_0.png","sha256":"31b8c76fcc6561fb35dbc79b1042e217525b66a4719d7a10fd1c49eeb40aa250"},{"path":"runs/step102-final-all-5/crops/miss_lucullus_607_2890_1.png","sha256":"9df0aba30c2b9b0dff5748df6a7109af141ea02b7aea8f93ef82f767340ff45b"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_lucullus_607_2890-native-0.png","sha256":"bf84f9f08faf2e15d76567b07ca024d2d2cb3133d687686986c8f608cb167e98"},{"path":"runs/step101-validation-expanded-review/miss_lucullus_607_2890-native-1.png","sha256":"4c2bdce04bbf6058ce000b1f2f013c077075bc901a9b51cb11ae9022502cd432"},{"path":"runs/step101-validation-expanded-review/miss_lucullus_607_2890-native-2.png","sha256":"686d8d4556efb4c254654cd8c4839643da503b99ad184e66ed7d7c16c5dbed35"},{"path":"runs/step101-validation-expanded-review/miss_lucullus_607_2890-native-3.png","sha256":"dcd22bf8e74857de108e522ed64bd56996668ae0f7c13993b9da592c2430eb8c"},{"path":"runs/step101-validation-first/crops/miss_lucullus_607_2890_0.png","sha256":"31b8c76fcc6561fb35dbc79b1042e217525b66a4719d7a10fd1c49eeb40aa250"},{"path":"runs/step101-validation-first/crops/miss_lucullus_607_2890_1.png","sha256":"9df0aba30c2b9b0dff5748df6a7109af141ea02b7aea8f93ef82f767340ff45b"}]},
  {"event_id":"miss_lucullus_631_13150","unit_id":"lucullus","source_regions":[{"role":"source_pdf","page":631,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.5451097613523282,0.07885072227328982,0.9375260297752315,0.09816003402911637],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_lucullus_631_13150_0.png","sha256":"643c37a58f13750ae3b989f676e9248ddb592fa5d0d0b85b7879f18d9373d5fc"},{"path":"runs/step102-final-all-5/crops/miss_lucullus_631_13150_1.png","sha256":"608d60df407c361511167dc7f96a9a74f96f6a7dffd7333491d4ab93eb7def5f"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_lucullus_631_13150-native-0.png","sha256":"f769cd8c5cdef5e178ff48de0fd4bfb5532cb10347b980d61a3da040eec798ea"},{"path":"runs/step101-validation-expanded-review/miss_lucullus_631_13150-native-1.png","sha256":"b2fe6e97562c209712055bb420895e09983fcb76c4f3b7091bb0d3fb39d98027"},{"path":"runs/step101-validation-expanded-review/miss_lucullus_631_13150-native-2.png","sha256":"95fad427273ac2363b2ba47354dee16fbb874ed0868c28696ead5f754b5ad6bc"},{"path":"runs/step101-validation-expanded-review/miss_lucullus_631_13150-native-3.png","sha256":"e23306c7ccaae5f2fb9d070ca798458cdfa1c6d72f4c3b86f9fab68d3ea853c0"},{"path":"runs/step101-validation-first/crops/miss_lucullus_631_13150_0.png","sha256":"643c37a58f13750ae3b989f676e9248ddb592fa5d0d0b85b7879f18d9373d5fc"},{"path":"runs/step101-validation-first/crops/miss_lucullus_631_13150_1.png","sha256":"608d60df407c361511167dc7f96a9a74f96f6a7dffd7333491d4ab93eb7def5f"}]},
  {"event_id":"miss_lycurgus_79_8882","unit_id":"lycurgus","source_regions":[{"role":"source_pdf","page":79,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.5289746274203867,0.09043609031977,0.9176266797441732,0.10653909019049697],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_lycurgus_79_8882_0.png","sha256":"6f7f9aeec2759793972668a414e21529dddb3a5e8c6ede22cbaa9c0ee5727954"},{"path":"runs/step102-final-all-5/crops/miss_lycurgus_79_8882_1.png","sha256":"ff547b9a5ae8410cfa03a8a72e88e6874baf21b833f5ddfa661acacc0dea1cb3"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_lycurgus_79_8882-native-0.png","sha256":"e3be3c9712022eaeddc229152450340ece4aa1fc76f61636d362b781d2002e24"},{"path":"runs/step101-validation-expanded-review/miss_lycurgus_79_8882-native-1.png","sha256":"266629e8059777d367b8e018a3b02d17835af38ecfc853d85b920290860de915"},{"path":"runs/step101-validation-expanded-review/miss_lycurgus_79_8882-native-2.png","sha256":"bdf67d72ae79f49a1680008cb90d6e9f48be004228cb6ffe9297b966c4850613"},{"path":"runs/step101-validation-expanded-review/miss_lycurgus_79_8882-native-3.png","sha256":"99b028e9c71c50de887e70ce1f5d3b0bb61afafc56167e05b873467809104148"},{"path":"runs/step101-validation-first/crops/miss_lycurgus_79_8882_0.png","sha256":"6f7f9aeec2759793972668a414e21529dddb3a5e8c6ede22cbaa9c0ee5727954"},{"path":"runs/step101-validation-first/crops/miss_lycurgus_79_8882_1.png","sha256":"ff547b9a5ae8410cfa03a8a72e88e6874baf21b833f5ddfa661acacc0dea1cb3"}]},
  {"event_id":"miss_lysander_535_1762","unit_id":"lysander","source_regions":[{"role":"source_pdf","page":535,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.5269509867047738,0.07633165570025072,0.9238704531154036,0.11570603849405156],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":2,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_lysander_535_1762_0.png","sha256":"76453436a1ec4c4b06ad6587625aac209d0caaff5d4c45ec9fad36dc18566039"},{"path":"runs/step102-final-all-5/crops/miss_lysander_535_1762_1.png","sha256":"56aa6e4d734958495b921db39e3b41f0ec07d51a3c4550763689ba898b0f27f4"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_lysander_535_1762-native-0.png","sha256":"274efdc3253317484c75934fb6e4664214c59803e3f8d25e3f6f29b948fa5cfa"},{"path":"runs/step101-validation-expanded-review/miss_lysander_535_1762-native-1.png","sha256":"07cfb4074fe447a9e330e3bb1aeeecc2cba855793cb5fe2431451b1ae21d1bff"},{"path":"runs/step101-validation-expanded-review/miss_lysander_535_1762-native-2.png","sha256":"34794ad505b428cbf3a88690e2925417f5a15dcebc415a65624ca6942a6025c5"},{"path":"runs/step101-validation-expanded-review/miss_lysander_535_1762-native-3.png","sha256":"e05a869e675b72ba6ef973958f52d410a635325e644b36c5ff48435db16ea4b6"},{"path":"runs/step101-validation-first/crops/miss_lysander_535_1762_0.png","sha256":"76453436a1ec4c4b06ad6587625aac209d0caaff5d4c45ec9fad36dc18566039"},{"path":"runs/step101-validation-first/crops/miss_lysander_535_1762_1.png","sha256":"56aa6e4d734958495b921db39e3b41f0ec07d51a3c4550763689ba898b0f27f4"}]},
  {"event_id":"miss_marcellus_377_3280","unit_id":"marcellus","source_regions":[{"role":"source_pdf","page":377,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.5473688203357181,0.08109534317956117,0.9519903457465997,0.09740178734758949],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_marcellus_377_3280_0.png","sha256":"c8112bff62639102aea9c2e14b46913c12d358c169fce143bbf858dfa7469130"},{"path":"runs/step102-final-all-5/crops/miss_marcellus_377_3280_1.png","sha256":"69c149e8f1d955a955eaccb42ec32e38afedea08d6221a11fc4b2334447f4e6d"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_marcellus_377_3280-native-0.png","sha256":"07f3bd9c73116a2e5a251a970f1ec9b5200bf53190108369018d1d83c0af8323"},{"path":"runs/step101-validation-expanded-review/miss_marcellus_377_3280-native-1.png","sha256":"d75d3ad9150a7b5310221a22bdb71b80472374cb4a1520094da55146defe10f6"},{"path":"runs/step101-validation-expanded-review/miss_marcellus_377_3280-native-2.png","sha256":"5dbb1910e842de7326a55368e365c2dca13ff402ca27b4e8f4feb723bf99f1ff"},{"path":"runs/step101-validation-expanded-review/miss_marcellus_377_3280-native-3.png","sha256":"e1fef6424688c3ea47b2bcfae60082b407043c56040c7e485675aef99bff6692"},{"path":"runs/step101-validation-first/crops/miss_marcellus_377_3280_0.png","sha256":"c8112bff62639102aea9c2e14b46913c12d358c169fce143bbf858dfa7469130"},{"path":"runs/step101-validation-first/crops/miss_marcellus_377_3280_1.png","sha256":"69c149e8f1d955a955eaccb42ec32e38afedea08d6221a11fc4b2334447f4e6d"}]},
  {"event_id":"miss_marcius_coriolanus_292_10538","unit_id":"marcius_coriolanus","source_regions":[{"role":"source_pdf","page":292,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.4756230065989987,0.09128724682862667,0.86690924032082,0.12569234056889503],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":2,"confidence":"low","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_marcius_coriolanus_292_10538_0.png","sha256":"27770c4d1ede0617d30c56e5fe5572dd16e811d61f0c563dd4bbebd88426a809"},{"path":"runs/step102-final-all-5/crops/miss_marcius_coriolanus_292_10538_1.png","sha256":"d3b309aadf5d04a94931ef2a92306509712238272f9979fab07c8330ab6d7a00"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_marcius_coriolanus_292_10538_0.png","sha256":"c4c5058943c49b0a5833692c7917c7f092e3dba255272ecd973dd62b3bfaf206"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_marcius_coriolanus_292_10538_1.png","sha256":"c945056e670ae5d0c1dba213f5518a2d0efd4984318353d165fdd97fad5900f2"}]},
  {"event_id":"miss_nicias_25_2918","unit_id":"nicias","source_regions":[{"role":"source_pdf","page":25,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5533145310229576,0.06717719673089062,0.9716214548694987,0.08682073266434245],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_nicias_25_2918_0.png","sha256":"fca2887af703334b76ff8c37fce3a51f848b0038760ec6adf8add06a6fc6d731"},{"path":"runs/step102-final-all-5/crops/miss_nicias_25_2918_1.png","sha256":"9f2aceec4240fa6691e625be0f6abfd5d8f864cfbfc544515f0b84512151757f"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_nicias_25_2918_0.png","sha256":"b88330afeff0b1ac55b4854c0222c505b7363a4ad8c3b3d7e32edd7f3d0bc480"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_nicias_25_2918_1.png","sha256":"18c38d2f8781b7af099a515667dac78e61a0853a0812c02826576fac87afb8f9"}]},
  {"event_id":"miss_nicias_25_3357","unit_id":"nicias","source_regions":[{"role":"source_pdf","page":25,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.531656757910626,0.8819957332502864,0.95008427541431,0.900664581693925],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_nicias_25_3357_0.png","sha256":"1024a93238d85ba1bc9ab03759b2ea982e1f1d09c046f92408e8b745cd15935e"},{"path":"runs/step102-final-all-5/crops/miss_nicias_25_3357_1.png","sha256":"7681ae4c3bcba73e55ee6b34fb06302c872812a96488eb6af67bf4964e206b4b"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_nicias_25_3357_0.png","sha256":"60d42c3559803157db73db71b27014c79f9a270935b18ff768ba0073d0faa3be"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_nicias_25_3357_1.png","sha256":"d9f590f401b91ab0f314e94567f0cb1c292b3d53e11186c70060d70b8dd52453"}]},
  {"event_id":"miss_numa_100_7129","unit_id":"numa","source_regions":[{"role":"source_pdf","page":100,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.4802620412050912,0.08459736485084568,0.872979409657266,0.10295533932860644],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_numa_100_7129_0.png","sha256":"71b97f5a92c605a5660dbf404c79c6f8f51f48d240dbc50a36f511dbeaafa48c"},{"path":"runs/step102-final-all-5/crops/miss_numa_100_7129_1.png","sha256":"5feae956041c81292ef8746d619a7f431c2786247f71f267c4e81db4e0c5f023"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_numa_100_7129-native-0.png","sha256":"6bcf8eb65ffd73571bd2256daa54fa82bdc7604650ee4659cde70eb9e044f130"},{"path":"runs/step101-validation-expanded-review/miss_numa_100_7129-native-1.png","sha256":"cfa84fa5ec554cce3330c045e7f87a1e35f51a3b6de2c5d5d7615ecd22b5c2e0"},{"path":"runs/step101-validation-expanded-review/miss_numa_100_7129-native-2.png","sha256":"bf0223b02243bc81fbf92048df8db466d12f63aec2f1209ce59e3fbc447afb90"},{"path":"runs/step101-validation-expanded-review/miss_numa_100_7129-native-3.png","sha256":"ed258c054c4947a524deb16cd1dfea635df84b66c2a15bee4b72647fa4bb460c"},{"path":"runs/step101-validation-first/crops/miss_numa_100_7129_0.png","sha256":"71b97f5a92c605a5660dbf404c79c6f8f51f48d240dbc50a36f511dbeaafa48c"},{"path":"runs/step101-validation-first/crops/miss_numa_100_7129_1.png","sha256":"5feae956041c81292ef8746d619a7f431c2786247f71f267c4e81db4e0c5f023"}]},
  {"event_id":"miss_pericles_200_2252","unit_id":"pericles","source_regions":[{"role":"source_pdf","page":200,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.4665820892535909,0.09040975761979228,0.8624535668711284,0.10726302990888077],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_pericles_200_2252_0.png","sha256":"d39d8f1527a2730537296d7ac947afe2f9dcf145024966582a5b3d2fe6b354c9"},{"path":"runs/step102-final-all-5/crops/miss_pericles_200_2252_1.png","sha256":"cee147732b758128a9cbfc018258257b65340693eec641070e14815693e56199"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_pericles_200_2252_0.png","sha256":"f3af46ef56c919c214c557169d375c4e8564dd6f6dd61281588eca019d0dbf82"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_pericles_200_2252_1.png","sha256":"2b6de6be540c84205fd50e261fbe4abbb747a1b2f8e59a030c16a50493d6e5b9"}]},
  {"event_id":"miss_pericles_214_9084","unit_id":"pericles","source_regions":[{"role":"source_pdf","page":214,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.5120296044460811,0.8625562583767108,0.7916885020454053,0.8801999035874757],"source_index":0,"occurrence_id":"expected"},{"role":"source_pdf","page":215,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.5567639262645425,0.07890446606933,0.8028404072368361,0.09396059240431381],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":2,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_pericles_214_9084_0.png","sha256":"0e71a73b2d6ce2974dc36638436b6837580f49d2d2c2e2eb90a67570cd04d338"},{"path":"runs/step102-final-all-5/crops/miss_pericles_214_9084_1.png","sha256":"fd1ca78a72fdc9276a08b1f4b09fab430bfc77deb0c287876166078f0bce20c3"},{"path":"runs/step102-final-all-5/crops/miss_pericles_214_9084_2.png","sha256":"66a9cccb706dd517c40408436ef503cd17ce45db5f48898460e237d4f06c4dd3"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_pericles_214_9084_0.png","sha256":"f94a4b99ccfba4c1333fe71231718fd29517f2ad0e622ff2f934e267eefed1a1"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_pericles_214_9084_1.png","sha256":"fcf283aad94a3e1fa9863fab835bacf9cc56659a32cdd4561e3ff8a43572bea5"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_pericles_214_9084_2.png","sha256":"caf1d13c2ded8a02178be6314e21624ca97f6251b4071d42ca50572fff88c3f4"}]},
  {"event_id":"miss_phocion_291_2223","unit_id":"phocion","source_regions":[{"role":"source_pdf","page":291,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5272383930014776,0.07097050155560568,0.9441746958547024,0.08780736046515278],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_phocion_291_2223_0.png","sha256":"72f45bf4f32e8807a6ac6642bbc82c001fde28ab8e845e251a866d59c74ab671"},{"path":"runs/step102-final-all-5/crops/miss_phocion_291_2223_1.png","sha256":"12d668070b42abcc690bb1e14e197c830df9f25989544e2919274bc234ce204a"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_phocion_291_2223_0.png","sha256":"80da902a1bc526b1388b29bbfaed5ec03b441055b71cdd622a83c2868f501b36"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_phocion_291_2223_1.png","sha256":"92a65435c59dbbf4c28ff4c2426f0ec1f4e7a96739a7e1a42e64cf1aaf6994b0"}]},
  {"event_id":"miss_phocion_294_3556","unit_id":"phocion","source_regions":[{"role":"source_pdf","page":294,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5113737273030906,0.06832648327095987,0.9066648415904526,0.08773938267205643],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_phocion_294_3556_0.png","sha256":"853eed5a6947b0a51beab4c0b1e3e456a7f79ab8250c4850a4cec6863503f7f9"},{"path":"runs/step102-final-all-5/crops/miss_phocion_294_3556_1.png","sha256":"958ac00d4d063ef99dc9b1f5ab29293d5b7c7cec620574b59151da7d6862f13f"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_phocion_294_3556_0.png","sha256":"b72064304d55a881774c13397fc0c9da91949c007f8f876ef73dd309127cf4a0"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_phocion_294_3556_1.png","sha256":"d7445fc35c4c204676edf2a2b841661445f7a9bbc5b3bc14658f3afbf3bd1bb3"}]},
  {"event_id":"miss_phocion_299_5777","unit_id":"phocion","source_regions":[{"role":"source_pdf","page":299,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.526152454507369,0.05899501673734363,0.9405584960137091,0.07629356448531176],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_phocion_299_5777_0.png","sha256":"490f0b8b5824633cbec118e3888b6cfad5ca054d65eb51d97afb1989de3b2b3c"},{"path":"runs/step102-final-all-5/crops/miss_phocion_299_5777_1.png","sha256":"92eac58cd3c9f616be5692f3af9ff4d57385f9640a3833aa3c293ef55236a863"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_phocion_299_5777_0.png","sha256":"7d9c95567c3620aac9c6e5bb12e455203b7301a34838c044ebbf9b37db69b7df"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_phocion_299_5777_1.png","sha256":"213ed4a1125baf6d772820db0789960b7c50b794caab397ddbe4c44492e48d7a"}]},
  {"event_id":"miss_phocion_301_6648","unit_id":"phocion","source_regions":[{"role":"source_pdf","page":301,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5643569376788652,0.060490844143062814,0.9653163006339459,0.08248050392746156],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_phocion_301_6648_0.png","sha256":"caf0628d059220968bb55ad9b52d7a3ec4321a78e3d9e8ad633be785fe37626c"},{"path":"runs/step102-final-all-5/crops/miss_phocion_301_6648_1.png","sha256":"3a6b4c3a3cdbe0cda411aa443372675af915b09b246ab1221942f9da1f47538e"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_phocion_301_6648_0.png","sha256":"76707f40251fb82455869e3fcf7a0ef8c22657dffcd8526131ed866289f599f7"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_phocion_301_6648_1.png","sha256":"908ccf088aeeea3aa027e8176651fd9f874191201c73a31ac8a5f825f7589d4a"}]},
  {"event_id":"miss_phocion_303_7514","unit_id":"phocion","source_regions":[{"role":"source_pdf","page":303,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5363190808714482,0.06767783650873574,0.9512923664301736,0.085999952473209],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_phocion_303_7514_0.png","sha256":"67046cb60719f4b2b20c9b69062472ca1cd23c1f7750e1efd8a3b7dea68b1876"},{"path":"runs/step102-final-all-5/crops/miss_phocion_303_7514_1.png","sha256":"8df2eb325f7f649624557449de425123128e044e61bb6d75af37f560e5df3d7a"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_phocion_303_7514_0.png","sha256":"b394b020fb5a060e9cd18180a81bdf9fa785f3fb425819e054f59a3ff4a1ab0a"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_phocion_303_7514_1.png","sha256":"bc0d0c558a6619b7372edd3aaaaaad16c9a6af99d703b15d0e886af014c83621"}]},
  {"event_id":"miss_pyrrhus_475_2507","unit_id":"pyrrhus","source_regions":[{"role":"source_pdf","page":475,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.5235691940348858,0.8591226273768746,0.9206121268200098,0.8774022543430346],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_pyrrhus_475_2507_0.png","sha256":"634269d29877f2d66753ace0bf79204df6f3a2b281f15558768246455eef5f5c"},{"path":"runs/step102-final-all-5/crops/miss_pyrrhus_475_2507_1.png","sha256":"fed2395fa28e64a793ef4e8b56181408189626486fc4973d39928da8965fefbe"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_pyrrhus_475_2507_0.png","sha256":"f47a946839230b195415b8e18cf6ed7a6110e562d90fa9828b84df2e6a45776c"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_pyrrhus_475_2507_1.png","sha256":"8f91fae3dffe4066983ffb10e8f94a3fa66632c039e7d7d9680605e15d929227"}]},
  {"event_id":"miss_romulus_36_1670","unit_id":"romulus","source_regions":[{"role":"source_pdf","page":36,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.5092826875453257,0.8697624554524981,0.8907088267652294,0.8867192850674621],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_romulus_36_1670_0.png","sha256":"b0553613d6db6f1d1a63b1d63b794bbf21c3fbb2161c7dd639d624f35627c187"},{"path":"runs/step102-final-all-5/crops/miss_romulus_36_1670_1.png","sha256":"4b23e0fe7373d8577da170acc2c76470ae6db89ca47e1fbe1681ca0eff4c51d3"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_romulus_36_1670_0.png","sha256":"75eba18f93393492c11772afd001ac4b5e085ca4790d54cef6f6c3a8ba8781bd"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_romulus_36_1670_1.png","sha256":"e101cb82a6be0014e7f1230cfd9e41db07a8644592cf1ccf1100a1bc993fdef3"}]},
  {"event_id":"miss_romulus_38_2595","unit_id":"romulus","source_regions":[{"role":"source_pdf","page":38,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.4914372070377711,0.8136524438993168,0.8883979090034788,0.8774067741763696],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":4,"confidence":"low","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_romulus_38_2595_0.png","sha256":"a8fc862b4839cb652e82afb94d335e662fa200a308af6181adf7f7de0d2514cb"},{"path":"runs/step102-final-all-5/crops/miss_romulus_38_2595_1.png","sha256":"fc58b67ff26ac1c5ccc48bce5b3cf4aee2da87cd95b67f07402b5af02df69adc"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_romulus_38_2595_0.png","sha256":"4944d6429266de8767bcc5a1286be34192836010eccd21f233fb90926d444b92"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_romulus_38_2595_1.png","sha256":"f2b4d14a522b0c3ea3b44487887fd35c368a2c242d757e9549d7e5c67f19945a"}]},
  {"event_id":"miss_solon_108_832","unit_id":"solon","source_regions":[{"role":"source_pdf","page":108,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.4908182622665757,0.85583122103822,0.884284479603633,0.8734405047419358],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_solon_108_832_0.png","sha256":"977591b00f0e27fd018d230266248e92ab635be7b30102f393984cfe8f1143d1"},{"path":"runs/step102-final-all-5/crops/miss_solon_108_832_1.png","sha256":"fa3fe54a1b85e5ddf6e1e74ecc53fbaa3046ee10f87592ca9b46c1421d3adf1a"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_solon_108_832-native-0.png","sha256":"48c61b28edbfab07224ba536c394be186f47d8070de14b3aa82d7e133d55f2c8"},{"path":"runs/step101-validation-expanded-review/miss_solon_108_832-native-1.png","sha256":"ae62f955ca808cd23b72903accfe575d9c39c79c8bd68d74ec87de78e64ae038"},{"path":"runs/step101-validation-expanded-review/miss_solon_108_832-native-2.png","sha256":"0b1e8f9bcc58fa6674e1eb8fbb1c64f77243399314d14951ac54220d85c71a96"},{"path":"runs/step101-validation-expanded-review/miss_solon_108_832-native-3.png","sha256":"8741f5e14878af7b9e5949ab92055e436cb406e4b7750564c8a9fd04d66aa5aa"},{"path":"runs/step101-validation-first/crops/miss_solon_108_832_0.png","sha256":"977591b00f0e27fd018d230266248e92ab635be7b30102f393984cfe8f1143d1"},{"path":"runs/step101-validation-first/crops/miss_solon_108_832_1.png","sha256":"fa3fe54a1b85e5ddf6e1e74ecc53fbaa3046ee10f87592ca9b46c1421d3adf1a"}]},
  {"event_id":"miss_solon_115_3532","unit_id":"solon","source_regions":[{"role":"source_pdf","page":115,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.505413938362241,0.07348521779521779,0.8949709445948132,0.1072208683649003],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":2,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_solon_115_3532_0.png","sha256":"1881d178856173102a607eb34d77fa98a9fef71be69256fad0a03fa4cf04bfbd"},{"path":"runs/step102-final-all-5/crops/miss_solon_115_3532_1.png","sha256":"effe923b736a363133c9c8f095afaae09e894c4c126af6cce142f33252a6577c"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_solon_115_3532-native-0.png","sha256":"28e419c33a759f7e70f048932069f1e6b02c3189b79d0658d57d35d62ad390f8"},{"path":"runs/step101-validation-expanded-review/miss_solon_115_3532-native-1.png","sha256":"222faed28434dabba327f9c56792bd0f7ea7ab0da5b089583582f4971907245f"},{"path":"runs/step101-validation-expanded-review/miss_solon_115_3532-native-2.png","sha256":"478a81dfa7975c1333bb91983023476707eef42bcdc51af35cde9f6fb37500c5"},{"path":"runs/step101-validation-expanded-review/miss_solon_115_3532-native-3.png","sha256":"846db4f70d715a381c797de7b02fdbd3df7bb56eb82dd6016723e856dc905e33"},{"path":"runs/step101-validation-first/crops/miss_solon_115_3532_0.png","sha256":"1881d178856173102a607eb34d77fa98a9fef71be69256fad0a03fa4cf04bfbd"},{"path":"runs/step101-validation-first/crops/miss_solon_115_3532_1.png","sha256":"effe923b736a363133c9c8f095afaae09e894c4c126af6cce142f33252a6577c"}]},
  {"event_id":"miss_theseus_15_774","unit_id":"theseus","source_regions":[{"role":"source_pdf","page":15,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.536441129152897,0.7901748445626448,0.9314166535845182,0.8847658984219235],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":7,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_theseus_15_774_0.png","sha256":"4ad82611d22ef37bc2a28b6c6491f42082e94c405f2a0a42707312b337d1f398"},{"path":"runs/step102-final-all-5/crops/miss_theseus_15_774_1.png","sha256":"171df32282c683ac8b4873cf903049e67e4c8e731e47a28c7c35332c2f9a39df"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_theseus_15_774-native-0.png","sha256":"93d3290a96fe0a26e543a05efb2eaaa33c0c4ec4aed02425ee58add37fb6905c"},{"path":"runs/step101-validation-expanded-review/miss_theseus_15_774-native-1.png","sha256":"487f363f47a1b5f348e66387f07a3cbfa8ef30f307a0595cf799e780f6d0aed1"},{"path":"runs/step101-validation-expanded-review/miss_theseus_15_774-native-2.png","sha256":"c407badf4d1e2b1d8562d55eec6a9e814feb69d84724c757563bd87f16500914"},{"path":"runs/step101-validation-expanded-review/miss_theseus_15_774-native-3.png","sha256":"745e0c690c23bbfb361cb393a8e1799bed41a0738d668126abb34e0419d03595"},{"path":"runs/step101-validation-first/crops/miss_theseus_15_774_0.png","sha256":"4ad82611d22ef37bc2a28b6c6491f42082e94c405f2a0a42707312b337d1f398"},{"path":"runs/step101-validation-first/crops/miss_theseus_15_774_1.png","sha256":"171df32282c683ac8b4873cf903049e67e4c8e731e47a28c7c35332c2f9a39df"}]},
  {"event_id":"miss_theseus_25_5427","unit_id":"theseus","source_regions":[{"role":"source_pdf","page":25,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.5547720296391577,0.862420387671722,0.8330217130008781,0.8796850238586802],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_theseus_25_5427_0.png","sha256":"88844c71a29f93f29872d25024860853a6327ca66544f3126ee80b2895bf4da1"},{"path":"runs/step102-final-all-5/crops/miss_theseus_25_5427_1.png","sha256":"a4a5c45b6e37302f9bf3c7fcd188a8e3aac068c9f40fc714b7ad85f55bc081c8"}],"review_evidence":[{"path":"runs/step101-validation-expanded-review/miss_theseus_25_5427-native-0.png","sha256":"249704fdbf5a9137fec1aced4d6effe189f4440b0331f424435284cb7eae8052"},{"path":"runs/step101-validation-expanded-review/miss_theseus_25_5427-native-1.png","sha256":"ee0ae1cfbba3fec6cd5510d9eb9a14db104ddddb35d6463d01e5a8a979e27855"},{"path":"runs/step101-validation-expanded-review/miss_theseus_25_5427-native-2.png","sha256":"dd8f5d1fdd7e9b08c63099f742e1caf7e8a889f620d4bec9e07b1cc0063ad539"},{"path":"runs/step101-validation-expanded-review/miss_theseus_25_5427-native-3.png","sha256":"a220dc138fc8b4b4c89eb9bdd921e4eba5d9e0f545ab5e16aabac445071e0df7"},{"path":"runs/step101-validation-first/crops/miss_theseus_25_5427_0.png","sha256":"88844c71a29f93f29872d25024860853a6327ca66544f3126ee80b2895bf4da1"},{"path":"runs/step101-validation-first/crops/miss_theseus_25_5427_1.png","sha256":"a4a5c45b6e37302f9bf3c7fcd188a8e3aac068c9f40fc714b7ad85f55bc081c8"}]},
  {"event_id":"miss_tiberius_gracchus_407_2016","unit_id":"tiberius_gracchus","source_regions":[{"role":"source_pdf","page":407,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5463649306326864,0.06473292040933538,0.9599434560471738,0.08245110315180193],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_tiberius_gracchus_407_2016_0.png","sha256":"a76175119c7646ba55b548103c57fd8e74d3f625c5795d9ca452893d19fda84c"},{"path":"runs/step102-final-all-5/crops/miss_tiberius_gracchus_407_2016_1.png","sha256":"e334dbb85de1735ea42ee6ea95cff52c463a8ee13f0cca89222eb712992fcb66"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_tiberius_gracchus_407_2016_0.png","sha256":"3acd3d333fd36e9d0db68227684621bc336921ddcfe9896a836523c2b0cfecf6"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_tiberius_gracchus_407_2016_1.png","sha256":"3e9769a9ece889b443ec43fd04487b3f6d0f8ed39b8af4b2f55a2ecc4f0ec4b5"}]},
  {"event_id":"miss_tiberius_gracchus_411_3764","unit_id":"tiberius_gracchus","source_regions":[{"role":"source_pdf","page":411,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.5207898287717209,0.05796350727758995,0.9501418552467726,0.07784375626175322],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_tiberius_gracchus_411_3764_0.png","sha256":"d2069ade37bd86b7228853c8a82a46c48275666eec13ee9f5847adc63e99d989"},{"path":"runs/step102-final-all-5/crops/miss_tiberius_gracchus_411_3764_1.png","sha256":"160357803ad918f8037a7a1a7679156eb271b4d5a06180601e39bb18b4def5b4"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_tiberius_gracchus_411_3764_0.png","sha256":"5f06fca5e09bf73aa44066f27dfffaab109471a802698c87ed739947f98d78ae"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_tiberius_gracchus_411_3764_1.png","sha256":"7ebd11b04338ca914e49d651f2f2f7f4923eca8ff2b13a05d25c003c06a6180e"}]},
  {"event_id":"miss_tiberius_gracchus_414_5143","unit_id":"tiberius_gracchus","source_regions":[{"role":"source_pdf","page":414,"input_sha256":"fc0924f2d8e45fe632fa6c8a97711b33d92ee59f5453ed8bf90e3ec99f8b6684","box":[0.4986037507000138,0.08494472870923508,0.9169909354052825,0.11518504881998431],"source_index":1,"occurrence_id":"expected"}],"missing_line_count":2,"confidence":"high","verdict":"confirmed","initial_tier":"A","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_tiberius_gracchus_414_5143_0.png","sha256":"af34f93b822247bb83eb98cfa0648171d9dbdad41f1792b7d96149b2224d90c6"},{"path":"runs/step102-final-all-5/crops/miss_tiberius_gracchus_414_5143_1.png","sha256":"8d985217aa3a59a13ef43867f08ac15586e8016d20c9abe9eccbb0d3ac8d120f"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_tiberius_gracchus_414_5143_0.png","sha256":"6cfd940331d661b111690db2c63e3fd9e027d447a629da401e59bf566000af14"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_tiberius_gracchus_414_5143_1.png","sha256":"e89bb4e333d6cfd163266847178ba521757e25038f8b05e5abff38dc2ed0ca22"}]},
  {"event_id":"miss_timoleon_318_9879","unit_id":"timoleon","source_regions":[{"role":"source_pdf","page":318,"input_sha256":"e094258e4cae19c5589af1f843f021eab662e086c5071f274118dde4667e5e15","box":[0.46420584871701503,0.08595967497638642,0.8640511667059558,0.10298929058125242],"source_index":0,"occurrence_id":"expected"}],"missing_line_count":1,"confidence":"low","verdict":"confirmed","initial_tier":"B","candidate_crops":[{"path":"runs/step102-final-all-5/crops/miss_timoleon_318_9879_0.png","sha256":"6d0cc9bcb6d914e2dcbeafb0362a48832a36da87179c6d118989e7164398581d"},{"path":"runs/step102-final-all-5/crops/miss_timoleon_318_9879_1.png","sha256":"c6ca91adc910bfd7883af82cfd5f27678fc98dc212f94ce5d5951da7cb79cb67"}],"review_evidence":[{"path":"runs/step92-development-final-2/crops/review_expanded_miss_timoleon_318_9879_0.png","sha256":"c1cd80939b79352505d91cdc40b4fa93eaac819feea89eabca074c6af0699e2e"},{"path":"runs/step92-development-final-2/crops/review_expanded_miss_timoleon_318_9879_1.png","sha256":"698fb2f4a23df76041fc4d0b38490e22dbbb8be32827cb65a3dc8bed540bfc43"}]}
]}
```

</details>
