# Review of the recto starts and body restoration plan

Reviewed on 2026-10-05 against the saved implementation plan, volume-endings audit, and current emitter, detection, range, filtering, metrics, scaling, and compile-test code. This is a review record; it does not revise the implementation plan or implement its changes.

## Assessment

The overall design follows the codebase: a facsimile-specific recto command avoids changing the shared front-matter/transcription parity contract; body numbering after alignment gives matching physical/folio parity; explicit final-unit endpoints and existing pins suit the requested hardcoded repair; range and reflow helpers remain the shared body path. The all-volume audit justifies two endpoint corrections and exclusion of Otho's repeated scans. No Greek OCR work is required by the selected scope.

The plan should be tightened before implementation. Phase 2B contains an unresolved pipeline-design branch, and several verification claims need concrete definitions. Existing identifiers should remain unchanged when applying these corrections.

## Findings

### R1 — Phase 2B's proposed early-filter fallback cannot use metrics as written

The plan says to move `is_aligned_closing_body_line(col, bands, m)` into the initial gap filter if that filter removes the line. But `lines::drop_furniture(bands, page_h)` has neither the column image nor unit metrics. `unit_body_strips` computes metrics after `render_and_band` has already called that filter. Moving the predicate there would require an explicit pipeline/API change, and can create a module dependency cycle if `lines` starts calling `reflow`.

The production-DPI trace has not yet established which stage first removes Lysander's words. The 2.2-gap multiplier and alignment tolerance are proposals inferred from a reduced-size diagnostic, not verified operational values.

Correction: give the trace its own suffixed step, with a required stage report and failing regression before choosing the implementation. If removal occurs in the current trailing reflow filter, keep the narrow fix there and specify exact height/median calculations, zero-gap handling, and sentinel behavior. If earlier removal occurs, explicitly separate zone trimming from gap trimming and schedule any metric-based pass after metric construction; retain one shared implementation. Do not tell an implementer simply to relocate a helper into an incompatible function. A scoped original-raster exception remains an alternative consistent with the user's preference for known-issue fixes.

### R2 — Phase 4's image-identity requirements omit global scale changes

`facsimile::mod::global_scale_for` samples every selected source page and chooses a 95th-percentile column width. Restoring source 0 page 637 and source 1 pages 705–708 changes that sample. If the percentile changes, unrelated units can be resized and repacked even though their source bands are unchanged. The plan permits unrelated changes for restored closing lines but not for this scale change.

Correction: record baseline/rebuilt percentile width and scale before pixel comparisons. Require composed-image equality when scale and packing inputs are equal. If the legitimate added pages change scale, compare unchanged units' ordered unscaled source-band crops, then compare composed output against the expected output at the rebuilt scale. Document resulting pagination/layout changes. Do not freeze scale arbitrarily to satisfy the old comparison.

### R3 — Late endpoint assignment does not validate the full detection path

The override is specified in `units_from_runs` after each run is converted and duplicate-unit validation has already occurred. It sets the emitted endpoint but does not stop scanning Otho's duplicated pages 709–710. If those pages produce a separate recognized Otho run, validation rejects the input before the assignment. This is a possible untested path, not a reproduced failure of the current real detection. The existing source tests also call `detect_openings` directly over full PDF ranges.

Correction: add a real-source, temporary-cache `load_or_detect` regression with no preexisting pins and assert both hardcoded endpoints and successful complete detection. Add synthetic input with a repeated terminal run after the known endpoint. Either use the same endpoint table to bound source scanning before run detection, updating the real-source test helper to follow that path, or explicitly normalize/exclude runs beyond the verified endpoint before duplicate checks. Pick one method in the plan; keep human-pin precedence and unrelated duplicate validation intact. Specify short-PDF and endpoint-bound behavior as part of that choice.

### R4 — Source requests and output-page occupancy do not prove source coverage

`unit_body_strips` logs and skips render/column failures, then can return `Ok` with fewer pages. Recording a request for 705–708 proves an attempt, not successful retained content. Its returned `Vec<PageBands>` contains no source-page numbers, so indexing it as though every requested page survived can misattribute bands. The plan asks for first/middle/last-line checks but also claims complete restored-page coverage.

Correction: real-source tests must require successful renders/detection for every expected page, assert the returned page count equals the inclusive range length, and preserve a verified source-page/index mapping. Check every expected body band on each restored page in order, with explicit furniture exclusions; representative line checks supplement this coverage check. Put any shared tracing/assertion helper in one test module definition reused by the Cato, Lysander, Cimon/Lucullus, and Otho tests. Keep this test support out of production APIs unless runtime provenance is separately required.

### R5 — Original-pixel preservation is underspecified after resizing

`scale_strip` uses Lanczos3 resizing, so literal source dark-pixel values do not persist in the composed raster. The plan repeatedly requires original pixels to survive composition without defining the reference scale, rounding, crop, or comparison. Tests that invoke the same composition routine twice are not independent evidence of inclusion.

Correction: separately prove source-band retention using unscaled crop coordinates, then create expected scaled strips with the established `scale_strip` contract and check those strips in the output at defined offsets/order. For small fixtures, use unique ink patterns and exercise wrapping with ordinary nonsplittable lines; require absent control patterns to remain absent. Define any real-scan tolerance and output provenance needed for split strips. Do not invent a second production image-transform implementation merely for testing.

### R6 — Several fixture contracts and test commands need clarification

The plan says unknown IDs and mismatched sources should retain run-derived endpoints, but `units_from_runs` accepts corpus indices and explicitly rejects a corpus unit in the wrong source. That assertion belongs to the lookup-helper test; the integration test must preserve the existing wrong-source error. Name both tests accordingly.

`first_body_compile_aligns_before_numbering` has no exact synthetic prose/page inputs forcing the two ending parities. Specify the generated scan filenames and explicit one-page/two-page prose blocks, assert the final content's observed parity, then assert first-body numbering and blank count. Merely inspecting whatever parity the fixture happened to produce does not establish both branches were exercised.

Run `cargo test facsimile::detect` and `cargo test known_final_body_pages` in 2C.1 as well as the named range/pin tests. Currently those newly added detector tests are not explicitly driven until Phase 2D. Keep a full suite before completion. Spell out any renamed real Monitum compile-test name so the verification command remains actionable.

### R7 — Shared helper extraction needs one clear home

The plan correctly proposes extracting `band_ink_bounds` rather than copying the scanner from `band_ink_span`. However, it does not specify where the new median-left-edge calculation lives or how it reuses the existing median implementation. `metrics::median_u32` is currently private; `reflow` has a separate float gap median for a different contract.

Correction: use the existing integer median through an appropriately scoped shared helper, or specify a deliberate compatible helper once. Avoid a third median implementation. Define the alignment helper's exact signature and the criterion for normal-height reference bands; `BandClass::Narrow` is a width class, so rejecting every non-`Normal` reference or closing band can defeat the intended protection. Preserve `Short`/`Tall` exclusions and sentinel behavior explicitly.

## Scope and priority

Resolve R1, R2, R4, and R5 before implementation so restoration and fidelity checks have executable contracts. R3 needs an end-to-end detector regression before claiming the table prevents recurrence; if the late assignment is sufficient on actual inputs, report that evidence, but keep the synthetic repeated-run case as a defined requirement or explicitly narrow the claim. R6 and R7 are smaller implementation-contract corrections.

No evidence found that the recto macro itself requires two consecutive boundary blanks or an imposition permutation change. The structural flyleaf/padding exception is explicit, the first-body origin ordering is correct, and the restricted 2/3/2-image fixture's page/folio expectations are consistent. The three targeted cache edits, preserved Latin source text, and retention of historical plans/notes agree with repository instructions.
