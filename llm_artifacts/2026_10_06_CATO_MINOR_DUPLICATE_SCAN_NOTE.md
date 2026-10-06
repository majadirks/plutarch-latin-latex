# Cato Minor duplicate-scan deduplication (D3) — 2026-10-06

## Context

The inner-tomi minimal repairs (see
[the plan](2026_10_05_INNER_TOMI_MINIMAL_REPAIRS_PLAN.md) and
[its verification note](2026_10_05_INNER_TOMI_MINIMAL_REPAIRS_VERIFICATION.md))
deduplicated the Eumenes/comparison (D1) and Agis (D2) repeated scans and froze a
header mask on both occurrences of the Cato Minor folio-934 passage (source 1
pages 337 and 347). That work recorded, but did not act on, an observation that
Cato Minor contained additional repeated scans. This note diagnoses and repairs
that repeat as **D3**, reusing the same identity-scoped duplicate-exclusion
mechanism. It is a selection correction on a proven whole-page duplicate, not a
change to any Latin.

## Finding

In `cato_minor` (source `Ploutarchou_Bioi_v2.pdf`, unit range source pages
309–363), **source pages 339–348 are a second scan of pages 329–338**. The
printed folio numbers in the running head prove it page-for-page:

| Source pages | Printed folios |
| --- | --- |
| 328 | 925 |
| 329–338 (first scan) | 926, 927, 928, 929, 930, 931, 932, 933, 934, 935 |
| 339–348 (second scan) | 926, 927, 928, 929, 930, 931, 932, 933, 934, 935 |
| 349 | 936 |

Per-page text similarity between the two blocks is 0.88–0.997 (the one low
outlier, 332↔342 at 0.61, is OCR/layout noise: both are folio 929, carrying the
identical prose "haec de dividendo imperio et evertenda republica conjuratio.
Erant multi tum ac praeclari viri…"). `cato_major` (source 1 pages 414–434) has
no such repeat.

The join after dropping the duplicate is exact and grammatical: source page 338
(folio 935) ends "…Cato senatui auctor fuit summam rerum Pompeio permittendi,
ejusdem **esse**" and source page 349 (folio 936) begins "**dicens** magna mala
inferre et depellere. (2) Et Pompeius quidem…". No distinct Latin is lost.

## Decision (approved 2026-10-06)

- **Retain the first occurrence (329–338, folios 926–935); exclude the second
  (339–348).** The first scan is the cleaner image — folio 934 at page 337 is
  rule-free, whereas the second occurrence's folio 934 (page 347) carries the
  heavier right-edge rule recorded as O6. This matches the D1/D2 precedent of
  keeping the earlier, cleaner occurrence.
- **Remove the now-redundant page-347 header mask (RFI-13B).** Page 347 is never
  rendered after the exclusion, so its header mask is dead configuration. The
  retained folio-934 page is 337, whose existing O6 header mask is unchanged.

## Implementation

- `src/facsimile/reflow.rs::skip_known_duplicate_source_page` gains
  `("cato_minor", 1, 339..=348)`, identity-scoped to the Cato Minor unit and
  applied in `unit_body_strips` before rendering, exactly as for D1/D2.
- The page-347 entry is removed from `SOURCE_FURNITURE_MASKS`.
- The authoritative ledger
  `src/facsimile/fixtures/inner_tomi_repair_source_trace.json` now records the D3
  `duplicate_page` record (skip pages, the ten occurrence pairs, the retained
  seam, the folio evidence, and the join prose) and no longer carries the
  page-347 mask record. It holds **28 repair records: 25 mask records containing
  26 rectangles, plus three duplicate records**.
- The shared test helper `assert_reviewed_duplicate_sequence` now detects each
  retained page's Latin column on the masked image, so its byte-for-byte source
  comparison aligns with production on retained pages that carry a furniture mask
  (the Cato folio-934 header on 337 and the right-rule margin on 361). The
  retained body bands lie outside those rectangles, so each still compares
  byte-identical to the raw source Latin. This leaves D1/D2 behavior unchanged
  (the mask step is a no-op on their unmasked pages).

## Verification

Workspace-local `CARGO_TARGET_DIR=target`; Poppler and MiKTeX variables as in the
inner-tomi verification note.

- `cargo fmt --all --check`, `cargo clippy --offline --all-targets -- -D warnings`:
  pass.
- `cargo test --offline`: 364 library tests and 9 binary tests pass, 37 library
  tests ignored (the new `cato_minor_repeated_folios_survive_once`). The ordinary
  `known_duplicate_page_exclusions_are_identity_scoped` covers the D3 record.
- `cargo test --offline --release --lib cato_minor_repeated_folios_survive_once -- --ignored --nocapture`:
  pass. Retained sequence is 309–338 then 349–363; every retained page's body
  bands are byte-identical to the raw source. Log: `tmp/check-cato-minor-dedup.log`.
- `eumenes_comparison_prose_is_complete_and_unique`,
  `agis_repeated_passage_survives_once`, and
  `inner_tomi_known_masks_preserve_latin` re-run and pass, confirming the helper
  change is safe and the page-347 mask removal is consistent.
- `cargo build --offline --release --bin facsimile`, `git diff --check`: pass.

## Source landmarks

- First occurrence openings: 329 "…Interim Rhodi ad negotium tractandum se
  com-", 337 "nonnulli etiam amici Pompeii atque domestici rei agerentur…"
  (folio 934), 338 "…excusat Cato, praeturam…" (folio 935).
- The folio-934 duplicate renders are saved under `tmp/pdfs/cato_dedup/`:
  `src_folio934_first_p337-337.png` (retained, clean) and
  `src_folio934_second_p347-347.png` (excluded, right-edge rule).
