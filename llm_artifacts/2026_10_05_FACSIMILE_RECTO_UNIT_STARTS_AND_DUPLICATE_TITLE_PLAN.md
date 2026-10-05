# Context

Facsimile Lives and Comparatio units should open on odd physical book pages (rectos), on the right when the printed signatures are folded and bound. Their body folios should have the same parity. The current emitter generally forces even openings. The requested change must introduce at most one blank at a unit boundary, never two consecutive boundary blanks.

Tomus III also contains a copied source title on a separate page before the generated Comparatio heading. Remove that copied title by correcting its source boundary, preserving the actual Latin body text and the existing generated heading.

This plan supersedes `2026_10_05_FACSIMILE_BODY_BLANK_PAGES_PLAN.md`, which proposed consecutive openings without a parity constraint. Retain that earlier file as a historical record; do not implement its consecutive-start contract or its incompatible page expectations.

The scope also includes restoring Lysander's missing final words from source 0, physical page 551, the omitted concluding source page 637 of Comparatio Cimonis et Luculli, and Otho's omitted source-1 pages 705–708. An audit of all eight volume endings found those two whole-page endpoint truncations; its evidence is recorded in `2026_10_05_FACSIMILE_VOLUME_ENDINGS_AUDIT.md`. The user prefers hardcoded fixes for known endpoints over Greek OCR matching. This plan therefore uses a shared explicit endpoint table and excludes Greek-recognition changes. This is planned work; no generator or cache fix has been applied while authoring this document.

# Current state of the world

## Pagination and imposition

- `src/facsimile/assemble.rs::inner_document` emits four front flyleaf pages, optional front matter, `\\beginbodynumbering`, body units, four back flyleaf pages, and `latex::SIGNATURE_PADDING`.
- Its body loop passes `force_parity_start` to `unit_block`. All units use `\\freshpage` except the first Tomus I unit when front matter is enabled, which uses `\\clearpage` to follow the combined Monitum scans directly.
- `src/latex.rs::inner_preamble` defines `\\freshpage` as `\\clearpage\\ifodd\\value{page}\\thispagestyle{empty}\\hbox{}\\clearpage\\fi`. It fills an odd pending page and leaves an even page for content. Front matter and the transcription reader use this shared macro too.
- Each composed body image already ends with `\\par\\clearpage`. A repeated `\\clearpage` on an empty pending page does not ship another blank; `\\hbox{}` makes a page ship even though it has no visible content.
- `\\beginbodynumbering` records the current physical counter in `\\bodyfirstpage` and displays `physical_page - bodyfirstpage + 1`. It does not reset the physical counter. Aligning the first body page to odd **before** recording this origin makes physical and body-folio parity agree throughout the body.
- `src/latex.rs::sheet_sides` places even physical book pages left and odd pages right. An imposed PDF page is a sheet side containing two book pages; its own PDF page index need not be odd. No imposition permutation change is necessary.
- `src/facsimile/mod.rs::compile_and_impose` reads compiled inner counts before producing wrappers. Its no-compile estimate is approximate. Production validation must compile inners and regenerate wrappers from actual counts.
- Four flyleaf pages at either end and the padding to 32-page signatures intentionally contain consecutive blanks. The no-double-blank requirement applies to blanks introduced at unit openings, including the front-matter/body transition; it does not remove structural endpapers or signature padding.

The existing Tomus III log shows Aristides's final image on physical page 52, blank page 53, Cato's first image on page 54, and Cato's second image on page 55. Thus the preceding blank in this local build is page **53**, rather than 55. Cato's opening corresponds to body folio 46, although the opening-page style suppresses the printed number. Earlier pagination changes mean its new location must be measured after rebuilding, not hardcoded as page 53.

## Diagnosed duplicate title

Read-only inspection rendered `build_facsimile/tomus_3_inner.pdf` pages 96–98 and `Ploutarchou_Bioi_v1.pdf` physical page 434. Diagnostic PNGs and OCR boxes are under `tmp/pdfs/unit_recto_diagnosis/`.

- Inner page 96 shows the source heading `ARISTIDIS CUM CATONE` under Cato's running head, with body folio 88. The log maps it to `facsimile_cato_major_43.png`; it is raster content, not a separate LaTeX heading. Page 97 is parity filler. Page 98 has the intended generated Comparatio heading and its first body image.
- `facsimile_units.json` records `comparatio_aristidis_et_catonis` at source 0, page 434, with `title_top_frac = 0.57787454` and `body_top_frac = 0.6090118`. The title fraction corresponds to the second line, `COMPARATIO.`.
- The source OCR box for `ARISTIDIS` starts at x=220.562 and y=392.381180 on a page 460.320007 wide and 714.479980 high. Its normalized x start is approximately 0.479 and its x end exceeds 0.5. The first title line begins at approximately `392.381180 / 714.479980 = 0.5491843`; the second begins at `412.879780 / 714.479980 = 0.5778745`.
- `src/facsimile/detect.rs::title_box` discards words with `x_min <= 0.5`, losing `ARISTIDIS`. It ignores `CUM` as a stop token. The OCR word `CATΟNE` contains Greek capital omicron, which the existing fold/token-prefix matching does not match to `CATONIS`. Consequently the second line is the first matched title line. This is a geometry/OCR identification defect; changing scholarly Latin spelling is not part of its fix.
- `src/facsimile/units.rs::ranges` shares page 434 between Cato and the Comparatio and propagates that incorrect fraction into Cato's `tail_cut_frac`. `src/facsimile/reflow.rs::drop_shared_tail` correctly retains bands above the supplied cutoff, which includes the first title line. The title has enough ink to survive noise filtering and receives a composed page. Do not mask the defect by dropping short pages or increasing minimum ink thresholds: those changes could discard genuine short final lines.

## Diagnosed missing final Comparatio page

Read-only visual inspection compared Tomus IV inner pages 118–119 with source v1 physical pages 636–638. Diagnostic renders, extracted source text, and page 637 OCR boxes are under `tmp/pdfs/unit_recto_diagnosis/`.

- Inner page 118 contains `facsimile_comparatio_cimonis_et_luculli_5.png` and ends with `magna clade a Cimone affecti, statim arma contra Græcos`, matching the final Latin line of source page 636 (printed folio 623). The generated `.tex` then emits back flyleaves and signature padding; source page 637 has no placement.
- Source page 637 (printed folio 624) begins its Latin continuation with `tulerunt,` and includes the conclusion through `fuisse.`. The scan confirms substantial body text on this page. Source page 638 is blank apart from scan noise/watermark; it must not be included as body.
- `facsimile_units.json` records `comparatio_cimonis_et_luculli`, source 0, `start_page = 635`, `last_page = 636`, and `pinned = true`. It is the final unit of source 0. Its existing review note verifies the opening, but does not verify the endpoint.
- `src/facsimile/units.rs::ranges` explicitly prefers a final unit's cached `last_page` over the source page count and returns an inclusive, unshared range. `reflow::unit_body_strips` therefore requests pages 635–636 and never requests 637. This omission is established by the cache and control flow, rather than inferred from a furniture threshold or PDF layout.
- Source page 636 has a Latin running head; page 637 has a Greek running head. `detect::page_unit` recognizes the known unit from Latin tokens in `render::running_head`. `smooth` bridges an unrecognized interval only when a later recognized page brackets it. An unrecognized final continuation therefore cannot extend the last run. `units_from_runs` records that recognized run's endpoint as `last_page`; `merge_pins` subsequently preserves the erroneous pinned endpoint verbatim. This is the current mechanism that permits the truncation; no historical detector run was reproduced during diagnosis.

# Desired end state

Every nonempty facsimile unit, including first units in Tomus I, later tomi, and restricted runs, opens on an odd physical page and an odd body folio. The first body page represents folio 1. Opening folios remain visually suppressed, with continuation heads and folios unchanged in style.

After a previous unit ends on even page E, the next unit starts on E+1 without filler. After it ends on odd page E, a single empty verso E+1 precedes the next unit on E+2. Continuation images remain consecutive within each unit. All inserted body-boundary blanks are even, empty of headings, running heads, folios, and images. Repeated alignment calls and empty unit inputs cannot create extra pages.

The complete source display title on page 434 is excluded from Cato's body stream and from the Comparatio's body raster. The generated Comparatio heading remains above its first body image. Cato's final sentence and the Comparatio's opening line are preserved with the source pixels and order intact. No source PDF or Latin transcription cache is edited. A deliberate boundary-cache edit is reviewable and documented.

The imposed output places every unit opening in the right-hand book-page slot. Endpapers and 32-page signature padding retain their contracts. Consecutive structural blanks remain permissible; no unit-boundary operation introduces two consecutive blanks.

Lysander's final body image includes the original raster words `commemoranda habuimus.` exactly once, following the preceding source line. A short, left-aligned closing body line must survive furniture removal even when scan skew makes its preceding whitespace larger than the median gap.

Comparatio Cimonis et Luculli includes the complete Latin body of source pages 635–637, preserving the transition from page 636 to page 637 and the final `fuisse.` exactly once. The range ends at 637 without including page 638, the decorative source end ornament, or library/back-matter pages. Its reviewed endpoint survives redetection through the existing pinned-row mechanism.

Otho includes the complete original body through source-1 page 708 and chapter XVIII. Source 709–710 repeats the concluding scans of 707–708 and must be excluded to avoid duplicating text. A shared edition-specific endpoint table records source 0's final Comparatio at 637 and source 1's Otho at 708; it supplies these values during automatic cache generation without requiring recognition of trailing Greek or misread Latin headings. Current pins are repaired explicitly, and human pin precedence remains intact.

# Phase 1 - Recto openings and body-numbering origin (Suggested implementer: Sonnet)

## 1.1 Implement a single facsimile recto-alignment command

Before changing code or rebuilding, capture the baseline described in 4.1. In `src/facsimile/assemble.rs`, define one private string constant `RECTO_START_DEFINITION` containing:

```tex
\newcommand{\unitrecto}{\clearpage\ifodd\value{page}\else\thispagestyle{empty}\hbox{}\clearpage\fi}
```

Append it to the facsimile document preamble after `latex::inner_preamble` and before `\\begin{document}`. Keep the shared `\\freshpage` definition and its front-matter/transcription call sites intact. The dedicated command restricts this behavior change to facsimile body units and avoids duplicated parity logic.

In `inner_document`, emit `\\unitrecto` immediately before `\\beginbodynumbering` when at least one unit has images. Iterate directly over the units and call `unit_block(unit)`; remove the Tomus I exception and the Boolean parameter. At the start of `unit_block`, return an empty string for an empty `image_names` list. Otherwise begin with `\\unitrecto\n`, followed by the existing marks, inline heading, image placements, and page styles. The first unit repeats alignment after the numbering initialization; the second call must be idempotent. This keeps the standalone unit helper's contract intact without special first-unit emitter logic.

Do not reset `page`, restart numbering for subsequent units, add a standalone title page, or emit an unconditional blank. Update current-contract comments without citing this plan.

Update existing tests `inner_tex_starts_each_unit_fresh_page`, `first_body_keeps_parity_start_without_monitum`, `monitum_errata_and_first_body_use_consecutive_pages`, and `monitum_without_cached_prose_still_uses_consecutive_pages` to the new behavior, using descriptive names and preserving unaffected assertions. Update direct `unit_block` calls. Add named tests:

- `facsimile_units_use_recto_command`: Life and Comparatio openings both use the command, with one shared definition.
- `first_body_alignment_precedes_numbering`: cover Tomus I and later tomi with front matter and restricted output; assert command ordering before `\\beginbodynumbering`.
- `empty_unit_emits_no_heading_or_page_break`: verify empty input produces no output and a following real unit retains its normal opening.

### LLM verification

Set `$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'`. Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test facsimile::assemble`. Inspect the diff; shared front-matter code and Latin cache text must be unchanged.

### Human verification

Read a generated preamble and body start. Confirm the command fills an even pending page, the body-numbering origin is recorded after alignment, and the first Tomus I unit follows the same recto contract as every other unit.

# Phase 2 - Correct the two-line source-title boundary (Suggested implementer: Opus)

## 2.1 Retain Latin title words that cross the page midpoint

In `src/facsimile/detect.rs::title_box`, change the horizontal eligibility check from `w.x_min > 0.5` to `w.x_max > 0.5`, requiring finite `x_max` as well as the existing finite y coordinate. A word crossing the midpoint can belong to the centered Latin heading; the left Greek title words on the diagnosed page end before the midpoint. Keep the existing top-zone exclusion, line grouping, `title_token`, stop tokens, and shared `token_matches` usage. Do not introduce global OCR substitution or change `verify::fold`.

Add `title_box_two_line_comparatio_crossing_midpoint`, using OCR-derived normalized boxes from page 434. Include the top running head, previous body text, the Greek title to the left, first-line `ARISTIDIS` with x_min below 0.5 and x_max above it, `CUM`, `CATΟNE` with literal Greek omicron, second-line `COMPARATIO.`, and the first body line. Assert the first-line title fraction and existing body fraction within a small float tolerance. Add `title_box_excludes_greek_words_ending_before_midpoint` and retain the existing narrow and two-line title tests. Include a fixture with a left-column word crossing the midpoint to document that geometry alone does not identify it: existing known-unit Latin token matching must still be required.

### LLM verification

Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test title_box`. Verify the regression fails with the former x_min check. Confirm Greek text and source/cache spelling remain untouched.

### Human verification

Compare the regression boxes with source page 434 and confirm both printed title lines belong to the same Latin heading. Inspect the filter change and confirm it addresses the observed geometry rather than suppressing short body content.

## 2.2 Repair the cached cutoff and prove body preservation

Explicitly edit only the `comparatio_aristidis_et_catonis` row in `facsimile_units.json`: set `title_top_frac` to the verified first-line coordinate, approximately 0.5491843, retain `body_top_frac = 0.6090118` and `start_page = 434`, and set `pinned = true`. Add `review_notes` describing source 0/page 434, the two-line Latin title, and the first-line cutoff verified against scan and OCR boxes. Preserve all titles, Greek text, IDs, groups, and other rows. Use a targeted UTF-8 edit; avoid a PowerShell JSON round trip that could change Unicode or reformat the whole table.

Add `shared_tail_excludes_both_lines_of_comparatio_title` in the existing `src/facsimile/reflow.rs` tests. Reuse current band/metric fixtures: place legitimate Cato tail bands above a detached rule, a two-line title at the diagnosed relative positions, and Comparatio body bands below. Assert the retained Cato bands contain every intended final body line and contain neither title line nor following body. Add `opening_fraction_preserves_comparatio_first_body_line` asserting the first Comparatio body band remains and both title bands are excluded. Exercise both cuts through `unit_body_strips` or its existing combined-boundary fixtures, so `ranges` propagation and trimming are checked together rather than duplicating trimming code.

Add an ignored real-source regression `cato_comparatio_source_boundary_excludes_duplicate_title`, requiring the source PDF and Poppler. Load the operational cache, derive ranges, render source page 434 using existing render functions, and inspect retained band row bounds. Require Cato's retained tail to end above the first title line and the Comparatio's retained opening to include its first body line. Save diagnostic crops when explicitly running the test and print their locations for human inspection. Missing tools/source must fail clearly when this test is requested.

Do not delete `facsimile_cato_major_43.png` downstream or discard all one-band pages. Rebuilding must naturally eliminate the title-only canvas because those bands are no longer body input.

### LLM verification

Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test facsimile::units`, `cargo test facsimile::reflow`, and `cargo test cato_comparatio_source_boundary_excludes_duplicate_title -- --ignored --nocapture`. Inspect the JSON diff and verify that only the intended boundary metadata changed.

### Human verification

Read the pinned row and inspect source page 434 beside retained tail/opening crops. Confirm Cato still ends with the sentence containing `inter suae aetatis homines fuit clarissimus.` and the Comparatio starts with `Si vitam alterius totam cum alterius vita`. Preserve exact raster spelling and punctuation; these excerpts identify locations and are not replacement transcription.

# Phase 2B - Restore Lysander's closing line (Suggested implementer: Opus)

## 2B.1A Establish the production-DPI trace before the repair

Execute this diagnostic substep before the repair portion of 2B.1. Save production DPI, column bounds, raw/zone/gap/interior/trailing/minimum-ink band intervals, unit metrics, closing ink bounds, preceding and median gaps, and the first removing stage. On unchanged code the ignored `lysander_closing_line_survives_reflow` must fail because retention fails while source ink is present. Missing prerequisites are not diagnostic evidence. Keep this regression ignored by default so the normal suite stays green. Do not change filtering before the report selects the explicit repair path.

### LLM verification

Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test facsimile::reflow`, and `cargo test lysander_closing_line_survives_reflow -- --ignored --nocapture`. Save its expected retention failure and stage report.

### Human verification

Inspect the production-DPI crop and trace beside source page 551; distinguish the missing words from the detached speck.

## 2B.1 Diagnose the band-removal stage and preserve short aligned body lines

Read-only visual diagnosis confirms that source page 551 contains `commemoranda habuimus.` and current inner page 343 ends after `divitibus coivissent. Hæc ergo sunt quæ de Lysandro`. The latter page embeds `facsimile_lysander_44.png`; the words are already absent from the raster supplied to LaTeX. This is not clipping by the PDF's bottom margin. The missing words' OCR boxes occupy y=616.895188–621.447990 on a 712.799988-high source page, approximately fractions 0.86545–0.87184, safely inside the 95% body zone. The next unit starts on page 552, so a title cutoff on page 552 cannot directly remove this line from page 551.

The source scan has a small detached speck below the closing line. A diagnostic row scan at 1800-pixel page height over the Latin column found the preceding full line at rows 1524–1546, the two-word closing line at 1556–1574 (217-pixel ink span versus about 470 for full lines), and the speck at 1603–1606. The closing gap is 10 rows while nearby body gaps are typically 2–7. These are diagnostic scan coordinates, not hardcoded operational bounds.

The relevant filter chain is `lines::drop_furniture` / `drop_gap_outliers`, then `reflow::drop_interior_running_head` / `drop_trailing_furniture`, then minimum-ink filtering and packing. The general gap filter can remove the detached speck. The subsequent trailing-furniture rule treats a narrow final band separated by **any above-median gap** as furniture, which fits the genuine closing line. This is the leading diagnosis from source geometry and code; reproduce at the actual build DPI and identify the first removing stage before editing its predicate. Do not claim a proven per-stage trace from the diagnostic row scan alone.

In an ignored real-source regression `lysander_closing_line_survives_reflow`, load the existing unit range and render actual source page 551 at the production DPI. In the `src/facsimile/reflow.rs` test module, inspect the raw column bands, zone/gap-filtered bands, interior-filtered bands, and final packing input. Map the known OCR line rectangle to raster rows with a one-median-line tolerance and prove that the rectangle has real ink. Record which stage first discards it. Use the real range's metrics so the test exercises the production decision. Assert source retention and scaled fidelity using the shared 2B.2 contract. A temporary diagnostic print may be used during implementation; permanent assertions must prove preservation, not merely print band counts.

Implement the general fix in `src/facsimile/reflow.rs::drop_trailing_furniture`: a normal-height, left-aligned short body line is not furniture merely because it is narrow and its preceding gap exceeds the median. Add a private `is_aligned_closing_body_line(col, bands, m)` helper:

- Require valid metrics, at least one preceding full-measure band, final-band height at least half the median and at most the existing tall-band threshold, and its preceding gap at most `2.2 * median_gap` (with a minimum tolerance of one pixel).
- Compute the median leftmost ink coordinate of preceding full-measure, normal-height bands. Regard the final band as aligned if its leftmost ink differs by at most half a median line height, with a minimum tolerance of one pixel. This permits scan skew without equating centered signatures with body lines.
- If that predicate holds, preserve the final band through the narrow/outlier branch. Keep the existing detached-rule cleanup, which still removes genuine thin horizontal separators. Do not protect a thin rule or a distant watermark with this predicate.
- Factor the pixel scan into `src/facsimile/metrics.rs::band_ink_bounds`, returning `Option<(u32, u32)>` with an exclusive right edge; make existing `band_ink_span` derive its result from that helper, preserving its current empty-band behavior. Reuse the helper for alignment instead of copying the span scanner.

Execute diagnostic substep 2B.1A before changing filtering. If the first loss occurs in `drop_trailing_furniture`, keep the repair there. If `lines::drop_gap_outliers` removes the line earlier, separate `lines::drop_furniture` into `pub(crate) body_zone_bands(bands, page_h)` and the existing gap pass, retaining their composition for existing callers. Add `pub(crate) drop_gap_outliers_with_protected_tail(bands, protect_tail: bool)`; ordinary callers delegate with false. In the reflow path construct metrics from zone-trimmed columns, then evaluate the shared alignment helper and apply that gap pass before interior/trailing filtering. Protection suppresses only the trailing pop, never leading removal. Keep coordinates unchanged; `lines` must not depend on `reflow` or metrics. Recheck existing metrics/filter regressions because this branch changes metric sampling. If zone trimming, minimum ink or packing causes the first loss, amend this repair contract with that measured cause before implementing it; do not apply an ineffective downstream guard.

The geometry helper's signature is `(col: &GrayImage, bands: &[Band], m: &LineMetrics) -> bool`. Sentinel metrics, fewer than two bands, no usable reference bands, or an empty final ink interval return false. Reference bands must have full-measure width and normal height under existing thresholds. A closing band may be width-class `Narrow`; exclude short/tall height cases rather than requiring `BandClass::Normal`. Make existing `metrics::median_u32` `pub(crate)` and reuse it for reference left edges; preserve the existing float gap median and add no third median implementation. Treat overlap as zero gap, use `max(1.0, 2.2 * median_gap)` as the closing-gap ceiling, and `max(1, m.height_med / 2)` pixels as alignment tolerance. Accept these proposed values only after the real production-DPI regression passes. If they fail, record measurements and amend this contract before proceeding; do not widen global thresholds blindly.

Add focused synthetic tests `trailing_furniture_preserves_narrow_left_aligned_closing_line_with_skewed_gap`, `trailing_furniture_still_removes_detached_centered_signature`, and `trailing_furniture_still_removes_thin_separator`. Exercise normal-height body text, a narrow closing line with a 10-row gap against approximately 5-row median gaps, and centered/nonnormal furniture with otherwise comparable widths. Retain the existing `shared_tail_frac_keeps_short_attached_last_line`, detached-rule, sentinel, speck, and packing tests. Add `band_ink_bounds_preserves_span_contract` for populated and empty bands.

Prefer this original-raster repair. The user's permission to generate a hardcoded string remains a fallback if source-band preservation cannot be made reliable; it is not the primary design because the complete original pixels are available. Any fallback must be narrowly keyed to unit `lysander`, source 0, page 551, and must detect/prevent duplication when the raster words survive. Record its exact literal, placement, and provenance in a reviewable cache/resource; do not silently manufacture Latin text in a generic generator cleanup. Revise this step to specify that fallback fully before implementing it.

### LLM verification

Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test facsimile::metrics`, `cargo test facsimile::lines`, `cargo test facsimile::reflow`, and `cargo test lysander_closing_line_survives_reflow -- --ignored --nocapture`. Prove the regression fails on unchanged code and passes with the fix. Save the final retained source crop and composed tail for visual inspection. Do not infer raster word presence from `pdftotext` of the generated facsimile.

### Human verification

Compare source page 551 with the retained tail and composed last page. Read the complete closing sentence through `commemoranda habuimus.`; confirm both words appear once with the original punctuation and glyphs, and no speck/signature/watermark is copied into the reader. Inspect genuine short closing lines and separator boundaries in other units affected by the shared filter.

## 2B.2 Share exhaustive source-coverage and scaled-stream assertions

Define one `SourcePageTrace` record and shared assertion helpers in the existing `reflow.rs` test module, reused by Cato, Lysander, Cimon/Lucullus and Otho. Record requests, successful renders, successful Latin-column detection, and retained source-coordinate bands separately. Fail on any render/detection error. Require returned `PageBands` count to equal the inclusive range length and the successful-page ledger to equal that range before mapping vector indices to source pages. A render request alone proves no coverage.

Save reviewed production-DPI expectations in `src/facsimile/fixtures/closing_body_bands.json`: source index, physical page, DPI, column bounds, every expected body-band interval and explicit excluded furniture intervals. Generate candidates from raw line bands and visually review the entire source column before freezing the ledger; never derive expectations from filtered output. Cover every body line on pages 551, 637 and 705–708, plus Cato's final body line and excluded title lines. Require each expected band retained once in order and its unscaled crop byte-identical to the source-column crop. Require excluded furniture absent. First/middle/last landmarks supplement this exhaustive check.

For independent composition verification, build expected intervals with `lines::cut_points` on retained bands after independently asserting every reviewed body interval and furniture exclusion; call existing `scale_strip` once per interval at the actual scale. Reviewed low-ink noise intervals can affect surrounding whitespace cut points but must contribute no strip. Calculate their ink independently and apply the documented minimum-ink floor; every expected body band must exceed it. This preserves the production whitespace contract without treating noise as body text. Use full column width, exclusive bottom row, existing rounded dimensions and Lanczos3. Do not call `compose_pages` to build the oracle. Place each expected strip at horizontal offset `geom.pad_x` on white rows of width `geom.tb_w`, with documented canvas clipping, and concatenate rows in source order. Decode actual canvases, remove exactly their leading `geom.pad_y` rows and concatenate remaining rows in canvas order; `pack_canvas` has no bottom inset. Require equal dimensions and decoded bytes of the two row streams. This checks splitting and wrapping without guessed output offsets or duplicate packing logic. Unscaled source retention and resized fidelity are separate assertions; original dark pixel values do not survive Lanczos unchanged. Use no arbitrary pixel tolerance after identical transforms. Diagnose any expected strip rejected by minimum ink rather than removing it from the oracle. Keep tracing in test code, without a production provenance API.

Add `composed_scaled_stream_preserves_unique_body_patterns_once` and `composed_scaled_stream_excludes_control_furniture_pattern` with unique ink patterns, noninteger scale, ordinary nonsplittable lines, a small canvas forcing wrapping, and a separate tall-strip splitting case. Require the complete expected stream and absence of the furniture control. All real-source restoration tests must reuse these successful-page, exhaustive-band and scaled-stream assertions; representative lines and nonempty output canvases alone are insufficient.

### LLM verification

Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test composed_scaled_stream`, and `cargo test facsimile::reflow`. Run affected ignored source tests when their repairs are available. Confirm an omitted, duplicated or reordered expected band fails the assertion.

### Human verification

Review frozen band ledgers beside complete Latin columns, including furniture exclusions. Compare reconstructed scaled streams with final canvases.

# Phase 2C - Restore the final source page of Comparatio Cimonis et Luculli (Suggested implementer: Sonnet)

## 2C.1 Correct the pinned source endpoint and verify inclusive range coverage

Make a targeted UTF-8 edit in `facsimile_units.json` to the `comparatio_cimonis_et_luculli` row: change `last_page` from 636 to **637**, retain `pinned = true`, and rewrite `review_notes` to cover both the verified opening on page 635 and the endpoint verified against page 637's concluding Latin text, with page 638 confirmed blank. Preserve its existing opening fractions, all titles, group, source, start page, and other row data. This is a second deliberate cache-row edit in this plan, alongside the Cato/Comparatio title cutoff correction.

Use the existing explicit endpoint and pin mechanisms; do not delete `last_page`, fall back to the PDF's complete page count, increase every unit endpoint, infer body coverage from composed-page counts, or append a transcription of page 637. No range algorithm change is needed. In `src/facsimile/detect.rs`, add one private constant `KNOWN_FINAL_BODY_PAGES: &[(usize, &str, u32)]` containing `(0, "comparatio_cimonis_et_luculli", 637)` and `(1, "otho", 708)`, and a private lookup helper `known_final_body_page(source, unit_id) -> Option<u32>`. In `units_from_runs`, when assigning each source's final unit endpoint, prefer the matching table value over the recognized run's endpoint. Apply this rule only to the final unit whose source and ID both match. Use one lookup implementation for both rows; keep `merge_pins` afterward so human overrides retain precedence. The table describes verified boundaries in this edition and must not be extrapolated to other source editions. This fixes automatic generation of the two known endpoints while avoiding Greek OCR and unbounded extension through blank/library pages.

Add `known_final_body_pages_override_unrecognized_terminal_runs` in the detector tests: inject runs ending at 636 and 704 for the two known final units and require generated metadata to end at 637 and 708. Add `known_final_body_pages_do_not_override_other_units_or_sources`; verify the lookup returns `None` for unknown IDs and mismatched sources. Add `known_final_body_pages_preserves_wrong_source_validation` requiring the existing wrong-source error from `units_from_runs`. Extend `load_or_detect_preserves_pinned_final_endpoint` to prove a deliberately different pinned endpoint still takes precedence, while the repaired operational pins retain 637/708. The operational cache changes in 2C.1 and 2D.1 are essential because the existing incorrect pins would otherwise override the new table.

Bound detection before converting runs: add `detection_last_page(source, pdf_page_count) -> u32`, taking the lesser of actual count and this source's table endpoint, or the count without a table entry. Use it in `load_or_detect` before `detect_openings`, smoothing and duplicate validation. Update real-source `detect_source` and its callers to pass the source index and use this bound. Duplicate scans 709–710 then cannot create a repeated Otho run before endpoint assignment. Retain final metadata assignment and subsequent pin merging. At cached-return and merged-save boundaries validate explicit endpoints against actual source PDF counts: `1 <= start_page <= last_page <= count`. Fail with source/unit/count context before writing invalid metadata. Never silently clip a known endpoint into a short PDF. Matching-unit stub PDFs need at least 637/708 pages; lookup fixtures need no PDF. Human pins retain precedence within actual bounds; existing wrong-source and in-bound duplicate errors remain intact.

Add `known_final_body_pages_bounds_detection_before_duplicate_terminal_run`, with synthetic recognition beyond the cap, asserting no OCR request past it and successful conversion, while retaining rejection of duplicates within it. Add `known_final_body_pages_rejects_endpoint_beyond_pdf_count`. Add ignored `known_final_body_pages_real_sources_load_or_detect_without_pins` using both real sources, normal OCR callbacks, absent temporary cache and no pins; require successful complete detection ending at 637/708. Never write the operational cache in these tests.

Add `ranges_honors_explicit_final_source_endpoint` to the existing test module in `src/facsimile/units.rs`. Build a synthetic source-0 final unit with `start_page = 635`, `last_page = Some(637)`, followed by a source-1 unit. Supply a source-0 PDF count greater than 638. Assert `end_page = 637`, `shared_tail = false`, `tail_cut_frac = None`, and opening metadata preserved. Test a one-source input as well, since restricted builds must use the same endpoint contract.

Add `load_or_detect_preserves_pinned_final_endpoint` in `src/facsimile/detect.rs` using the existing injected OCR/stub PDF/tempfile patterns. Supply an existing pinned final source-0 row with `last_page = Some(637)` and detector input whose recognized final run ends at 636 while subsequent pages are unrecognized. Require the redetected output and saved JSON to retain 637 and the reviewed metadata. Do not run redetection against the operational cache as a test.

Add `unit_body_strips_requests_inclusive_final_page` in `src/facsimile/reflow.rs` using the existing synthetic bilingual-page fixtures and a render closure that records requests. Derive the range through `units::ranges` with the explicit endpoint, require render calls for 635, 636, and 637 exactly once and no call for 638, and require visibly inked body bands from the final page to survive. Ensure the last page is treated as an interior continuation, not as a shared title boundary. This tests source coverage independently of page packing.

### LLM verification

Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test ranges_honors_explicit_final_source_endpoint`, `cargo test load_or_detect_preserves_pinned_final_endpoint`, `cargo test unit_body_strips_requests_inclusive_final_page`, `cargo test facsimile::detect`, `cargo test known_final_body_pages`, `cargo test known_final_body_pages_real_sources_load_or_detect_without_pins -- --ignored --nocapture`, and the focused `cargo test facsimile::units` suite. Inspect the targeted JSON diff: the only new correction is this row's endpoint and provenance; the separately planned Cato boundary edit remains permitted.

### Human verification

Compare source pages 636–638 with the revised cache note. Confirm 637 contains the continuation beginning `tulerunt,` and the conclusion, while 638 contains no body text. Read the endpoint diff and confirm the value is a physical source PDF page index, not printed folio 624 or an output-page number.

## 2C.2 Prove the real concluding body survives reflow and composition

Add `cimon_lucullus_final_source_page_survives_reflow`, ignored with an explicit source-PDF/Poppler requirement, in the existing `src/facsimile/reflow.rs` test module. Load the operational unit table read-only, derive the real range, and require the source-0 final unit's endpoint to equal 637. Render at the production DPI, using the normal Latin-column detection, furniture filtering, real unit metrics, scale, and composition path. Assert page 637 contributes retained body bands, including its first Latin line, a middle body line, and its closing line. Locate these using saved/parsed OCR boxes from `render::page_words` and verify ink in their source raster regions; OCR identifies locations, not replacement text. In particular, the last body band containing `fuisse.` must survive the short-line protection from Phase 2B. The detector's failure to recognize the Greek running head does not authorize discarding the Latin column below it.

Compare the page-637 retained body bands to the source scan, then verify those strips survive composition exactly once in source order. Follow existing strip/pixel fixtures and shared render helpers; do not add a second source extraction pipeline. Capture the retained final-page crop and composed tail for inspection. Exclude the detached decorative ornament below the text using the existing furniture/rule filters, and inspect its absence in the output. Do not infer completeness from total output page count or a nonempty final canvas.

The all-volume build in 4.1 must rebuild Tomus IV and its wrapper with this endpoint. Source-page content may span several trim pages, and the original output page 118 may move or become fuller; do not prescribe a fixed new page count or append one blank/one image by hand. Require every source body line from 637 in the final generated Comparatio stream, including the continuation across the source-page boundary and the conclusion.

### LLM verification

Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test facsimile::reflow`, and `cargo test cimon_lucullus_final_source_page_survives_reflow -- --ignored --nocapture`. Show that the real endpoint assertion fails with `last_page = 636` and passes after the cache repair. Require actual retained-band and composed-pixel checks; report missing prerequisites rather than silently skipping the explicitly requested test.

### Human verification

Inspect the diagnostic final-page crop and composed Comparatio ending beside source page 637. Read through the join `contra Græcos` / `tulerunt,` and the final sentence through `fuisse.`. Confirm the full Latin column appears once and the Greek column, source ornament, watermark, and following blank page do not enter the reader.

# Phase 2D - Restore Otho's ending without duplicated scans (Suggested implementer: Sonnet)

## 2D.1 Repair Otho's endpoint and verify the four restored pages

The all-volume endings audit found Tomus VIII's last body page 296 ends at the last line of source-1 page 704, midway through chapter XII. The pinned `otho` row has `last_page = 704`; the renderer never requests 705–708. Read-only comparison confirms those four pages carry the remaining body through chapter XVIII and the conclusion on 708. Source 709–710 repeats the scans/body of 707–708. Use the first complete sequence ending at 708 to restore the text exactly once. This is a second source-terminal endpoint failure, alongside Cimon/Lucullus; the other six audited volume endings contain their closing sentences.

Make a targeted UTF-8 edit to the `otho` row in `facsimile_units.json`: set `last_page = 708`, retain `pinned = true`, preserve its opening at 696 and existing opening fractions, and rewrite `review_notes` to document the opening, complete conclusion on 708, and deliberate exclusion of duplicate scans 709–710 and subsequent blank/back matter. Preserve every other field and row except the two previously authorized boundary edits. The plan's complete cache-change scope is now three rows: Cato's following Comparatio cutoff, the Cimon/Lucullus endpoint, and Otho's endpoint.

Reuse `KNOWN_FINAL_BODY_PAGES` from 2C.1 and the same lookup logic. No extra Otho-only renderer branch, Greek-head matcher, text insertion, or new endpoint abstraction is needed. Parameterize the endpoint/pin/render-request fixtures from 2C.1 to cover source 1's Otho as well as source 0's Comparatio. For Otho, assert range 696–708 inclusive, `shared_tail = false`, `tail_cut_frac = None`, and no render calls for 709 or 710. Retain the tests that unknown source/unit keys use ordinary endpoints and pins override the table.

Add the ignored real-source regression `otho_final_source_pages_survive_reflow_once` in `src/facsimile/reflow.rs` tests, requiring source PDFs and Poppler. Load the real unit cache read-only, require endpoint 708, and trace the production rendering/reflow/composition route with requests recorded. Require pages 705–708 requested once each and 709–710 never requested. Verify representative retained body bands from each restored page, the continuation `afferre` / `videbantur,`, chapter XVIII, and the final body band on 708. Verify the retained source strips reach the composed stream once in order using the same test helper approach as 2C.2. Require body through the final hyphenated `Cæ-` / `cinæ se conjunxerunt.`; preserve the actual scanned glyphs and source line breaks. Exclude the library stamp, horizontal ornament, source footer, and watermark below the body. Do not append a transcription of the missing chapters.

Production-DPI review also establishes that the fixed 8% top zone removes the first body bands on pages 706 ([205,236)) and 707 ([203,234)). Before the Otho regression can pass, preserve these source rows in the reflow path using a geometry-based top-zone rescue: retain a normal-height, full-measure band centered between 7% and 8% of page height when its left ink aligns with normal full-measure body lines. Keep the shared furniture API and its 8% running-head behavior unchanged. Use existing metrics and ink-bound helpers; do not key production behavior to Otho or page numbers. Add a synthetic regression for a full-measure aligned early body line and a contrasting short running head. Record and visually inspect every additional restored band in the all-volume comparison.

Add Otho's restored tail to the all-volume build and pixel/source-coverage audit in 4.1. The new body-page count is measured after composition; do not assume four source pages produce four trim pages, and do not rely on total signature count to prove completeness. Save retained/composed ending crops and inspect them against source 705–708.

### LLM verification

Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test known_final_body_pages`, `cargo test load_or_detect_preserves_pinned_final_endpoint`, `cargo test unit_body_strips_requests_inclusive_final_page`, `cargo test facsimile::reflow`, and `cargo test otho_final_source_pages_survive_reflow_once -- --ignored --nocapture`. The real endpoint assertion must fail with the old cached value 704. Inspect the three-row JSON diff and prove source requests stop at 708. Do not redetect or modify operational caches as a test side effect.

### Human verification

Compare Otho's restored conclusion with source 705–708, reading the join after `afferre`, the remaining chapters, and the final sentence. Compare source 707–708 with 709–710 to confirm the duplicated text occurs once in the reader. Inspect the output below the last line for unwanted stamp/ornament/footer content. Confirm the shared hardcoded endpoint table covers precisely the two known terminal units.

# Phase 3 - Compiled pagination regressions (Suggested implementer: Sonnet)

## 3.1 Exercise both parities, repeated calls, and the front-matter transition

In `src/facsimile/assemble.rs` tests, follow the existing `monitum_combined_compile_has_adjacent_content_pages` pattern: `tempfile`, visibly inked `image::GrayImage` fixtures, `write_inner`, `compile::compile` with `INNER_PASSES`, `compile::inner_page_count`, `render::page_text`, `render::render_page`, and `pdfimages -list` honoring `PLUTARCH_PDFIMAGES`. Extract a small shared test helper for repeated image-list parsing if needed; do not create a parallel compiler/render framework.

Add `facsimile_units_compile_on_rectos_with_single_boundary_blanks`, ignored with a clear MiKTeX/Poppler requirement. Use a restricted volume with four initial flyleaf pages and three units (Life, Life, Comparatio) containing 2, 3, and 2 unique-dimension images. Require physical image pages `[5, 6, 7, 8, 9, 11, 12]`, with titles on 5, 7, 11. There is no blank between units one and two; page 10 is the sole filler before unit three. Continuation folios are 2, 4, 5, and 8; inferred opening folios are 1, 3, 7. Opening pages show no numeric folio. Render page 10 and assert no dark pixels under a documented white-page tolerance, and require no text or images there. Require each fixture image exactly once, a 32-page inner, and no gaps inside units.

Add `recto_alignment_compile_is_idempotent`: inject repeated `\\unitrecto` calls on both odd and even pending pages in a small fixture. Require an even pending page produces exactly one blank and subsequent calls produce none. Include an empty unit between nonempty units to prove it produces neither a title-only page nor a second filler.

Add `first_body_compile_aligns_before_numbering`: test synthetic front matter with final content on each parity, using `vol.prose` and small generated scan fixtures for the names required by the ordinary front-matter helpers. Inspect the actual final content page rather than guessing from the front-matter page count. If its physical page is even, the body follows directly; if odd, exactly one empty even page intervenes. Require the first body page odd and its continuation folio 2. Cover Tomus I, a later tomus, and restricted output across this and the preceding fixture. No source PDF or vision call is needed for synthetic fixtures.

Make the first-body fixture a four-case table: source 0, Tomus I and later Tomus II with front matter enabled, each with one/two explicit prose pages and two unique body images. Supply 300×100 inked PNGs under `front_qr_name(0)`, `front_volumen_name(0)`, `front_colophon_name()`, `front_monitum_a_name()` and `front_monitum_b_name()` as required. One-page prose is `\thispagestyle{empty}\noindent FRONT-A\par\clearpage`; two-page prose appends the same block with `FRONT-B`, with no parity commands. For Tomus II assert prose page 9 or pages 9–10: final front page 9/filler 10/body 11, or final front page 10/direct body 11. For Tomus I assert prose page 13 or pages 13–14, then combined scans on page 14 or 15: direct body 15, or filler 16/body 17. Assert observed sentinel/scan pages before transition checks so both parity branches are forced. Both Monitum crops share the specified scan page, continuation folio is 2, and predicted fillers contain no text/images/ink. Restricted output remains covered by the preceding 2/3/2 fixture.

Update the real-source ignored Monitum compile test: preserve prose-to-combined-scan adjacency and both crops on the same page. Replace unconditional scan-to-body adjacency with `body_page = smallest odd page greater than combined_scan_page`; permit zero or one blank and render any permitted filler to confirm emptiness. Keep continuation folio 2. Rename it `monitum_combined_compile_keeps_scans_adjacent_and_body_on_recto`.

In the existing `src/latex.rs` imposition tests, add `imposition_places_odd_book_pages_on_right` over multiple signatures: every `Side.right` is odd and every `Side.left` is even; each page occurs exactly once. Reuse `sheet_sides` directly. Do not change its mapping.

### LLM verification

Set `$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"` and confirm `--version` succeeds. Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test facsimile::assemble`, and `cargo test imposition`. Run each newly named compile test with `-- --ignored --nocapture`, plus `cargo test monitum_combined_compile_keeps_scans_adjacent_and_body_on_recto -- --ignored --nocapture`. Requested ignored tests must actually run; report missing prerequisites instead of claiming success.

### Human verification

Inspect fixture openings and the one permitted blank. Check that repeated commands do not make another page, the first body folio is correctly anchored, and the Monitum scans remain together. Confirm odd inner pages appear in the wrapper's right slot.

# Phase 4 - Rebuild and audit the real books (Suggested implementer: Opus)

## 4.1 Capture baseline before changes; rebuild and verify all units

Before implementation, create `tmp/pdfs/recto_units_verification/baseline/` and preserve all eight inner `.tex` files, logs, page counts, `pdfimages -list` outputs, unit cache, and boundary renders for Tomus III (Cato opening, pages 94–99, and Lysander's ending on pages 342–343), plus Tomus IV's ending on pages 117–119 and Tomus VIII's ending on pages 295–296. Preserve source-page renders and OCR boxes for v1 pages 434, 551, and 635–638 and v2 pages 704–711. Full builds call `clean_generated`; capture evidence before they overwrite outputs. Record baseline column-width samples, 95th-percentile widths and resulting `global_scale_for` scales for each source/trim geometry, using temporary diagnostics removed before completion. Record SHA-256 manifests for source PDFs, transcription/front-matter caches, and existing render-cache entries. Use `pdfimages -png` to preserve decoded baseline body rasters and correlate their extraction indices to `.tex` image order and log shipment pages. PNG file byte equality is not required; compare decoded pixel bytes and dimensions. If baseline files are missing, establish them with unchanged code before applying the plan.

After all changes and regressions pass, rebuild with existing offline resources:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'target'
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
& $env:PLUTARCH_LUALATEX --version
if ($LASTEXITCODE -ne 0) { throw "LuaLaTeX is unavailable." }
cargo run --release --bin facsimile --target-dir target -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
if ($LASTEXITCODE -ne 0) { throw "Facsimile build failed." }
```

Do not clear caches, retag, run vision review, or redetect all units as part of this build. Require successful PDF compilation and fresh imposed PDFs, rather than accepting the no-compiler estimate branch.

Save rebuilt manifests under `tmp/pdfs/recto_units_verification/rebuilt/`. Audit every expected unit in every volume using generated image references, wrapped LuaLaTeX shipment logs, and PDF image listings. Distinguish physical PDF indices from body folios and imposed sheet-side indices. For each nonempty unit require an odd opening, one image per intended body page, and consecutive continuation pages. Between units require next opening at E+1 for even final page E or E+2 for odd E. Render every intervening boundary page and require it blank. Require the front-matter/body transition has at most one blank and the body-numbering origin equals the first body image's page. Reject missing, duplicate, or unexpected body placements.

Inspect the corrected Cato/Comparatio transition by content and filename mapping; old pages 96 and 98 will move. Confirm no source title-only page remains and the generated title appears exactly once above Comparatio body content. Render Cato's final body page and the first Comparatio body page, checking both against source page 434. Check other two-line title boundaries for regressions from the broader word eligibility check. Preserve all genuine short final pages.

Compare decoded body images against baseline. Record rebuilt width samples, percentile widths and scales before comparing images. Equal scale and retained packing inputs require decoded composed-image equality and unchanged order. Restored pages can legitimately change global scale and repack unrelated units: require their unchanged ordered unscaled source-band crops to equal baseline, then verify composed streams at rebuilt scale using 2B.2 and document resulting layout/pagination changes. Do not freeze scale to satisfy old hashes. The intended exceptions are Cato's corrected tail, Lysander's restored closing line, Comparatio Cimonis et Luculli's restored concluding source page, and Otho's restored source pages 705–708. Cato's retained first title line disappears, normally eliminating the final title-only canvas. Lysander gains the missing original source line in its final canvas. The final Comparatio in Tomus IV gains the original Latin body of source page 637; its formerly short last canvas may become fuller and new canvases may follow it. Compare the ordered retained body bands/source pixels for those units and inspect their final sentences to prove no body text was lost; do not require unchanged composed image hashes where the intended correction changes packing. Inspect the rebuilt Lysander tail by content rather than old page 343, which can move. Require the complete `commemoranda habuimus.` line exactly once. For the Tomus IV Comparatio, require the full source-page-637 body once in order, the join from `contra Græcos` to `tulerunt,`, and the final `fuisse.` before back flyleaves; require no source-page-638 contribution. For Otho, require restored source-1 pages 705–708 in order through chapter XVIII and the final sentence, and require no contributions from duplicate scans 709–710. When scale and packing inputs match baseline, other Comparatio body images and unrelated units must remain pixel-identical unless the general aligned-body protection restores another demonstrably genuine closing line; identify and visually review every such restoration against its source. Any unexplained pixel change must be investigated before completion. Source PDFs and Latin caches must match baseline hashes. Boundary-cache changes must be confined to the three explicitly reviewed rows; existing render-cache entries must remain intact.

Verify all inner counts divisible by 32 and each freshly generated wrapper's `\\innercount` equals its compiled inner count. Confirm every unit opening maps to the right book-page slot in the wrapper and inspect representative imposed sheets around Cato and its Comparatio. Document that the imposed sheet-side PDF number itself is not a book-page parity assertion.

Update the relevant facsimile layout description in `README.md` in place to state recto starts with zero or one boundary blank and matching body-folio parity. Save commands, versions, test results, cache diffs/hashes, old/new opening locations, all-volume audit results, pixel comparisons, visual findings, and any limitations in `llm_artifacts/2026_10_05_FACSIMILE_RECTO_UNIT_STARTS_AND_DUPLICATE_TITLE_VERIFICATION.md`. Retain both plan and verification note.

### LLM verification

Run `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo build`, and `git diff --check`. Inspect `git status --short` and the complete scoped diff. Require the actual compiled all-volume parity/adjacency audit and source/cache comparisons described above. Tests or total page counts alone cannot prove that a raster title was removed or that no Latin body line was clipped.

### Human verification

Open the rebuilt Tomus III with odd pages on the right in two-page reading view. Inspect Cato's opening and its final transition into the Comparatio; confirm no copied title page remains, at most one blank separates units, and the legitimate final and opening lines are intact. Read Lysander's final sentence alongside source page 551 and confirm `commemoranda habuimus.` survives exactly once. Open Tomus IV and compare the complete Comparatio ending with source pages 636–637, including the restored continuation and final sentence. Confirm the final body content precedes the back flyleaves and the source ornament/blank page 638 is excluded. Read Tomus VIII through Otho's complete conclusion alongside source-1 pages 705–708 and confirm the repeated source scans do not duplicate body text. Inspect representative Life and Comparatio openings in other tomi and the corresponding imposed sheets, then verify the fold/binding interpretation. Existing endpapers and final signature blanks remain intentional.
