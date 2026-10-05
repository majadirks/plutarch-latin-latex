# Review scope

Reviewed the saved cleanup plan against `columns.rs`, `lines.rs`, `reflow.rs`, `metrics.rs`, `mod.rs`, and the metric caller in `praefatio.rs`. Re-rendered Tomus II physical PDF pages 193, 194, and 195 from the file on disk. The review includes the Phase 1 diagnostic evidence. Production cleanup remains unimplemented; the Rust additions are confined to the test module.

# Findings and resolutions

1. **Pixel removal needs a spacing proof, but the verified occurrences are source tails.** An interior masked region could remain inside midpoint strip padding. The source trace establishes that the reported watermarks come from v1 p233 and p235 bottom margins. Right-edge noise retains a furniture-only band on p233 and merges the last body lines with the footer on p235. The measured masks remove that noise as well as pale watermark pixels. Rebanding then ends retained body at rows 2624 and 2622; existing `cut_points` copies zero trailing furniture rows. Step 3.2 therefore verifies exact copied height and neighboring glyph preservation through the existing strip/composition path. Introducing interior-gap compaction for these tails would add machinery without resolving a demonstrated need.

2. **The Coriolanus leak is reproduced.** Physical page 193 of the inspected Tomus II PDF contains `RIOLANUS.` and `271`, followed by `stilis agri ...`. Physical page 194 continues prose from the same source page. The input PDF was last modified at 08:15:11 local time on 2026-10-05. The original review looked at p194 alone and missed the neighboring seam. The revised plan records the user's locator and the observed physical locator and requires correction of source v1 p284, rather than making its correction conditional on reproducing the leak.

3. **Divider recognition lacked an exact continuity definition.** A leaning rule can have no long run in one fixed x column. The revised step 2.1 defines a small local 8-connected component search, explicit envelope and height limits, whitespace separation, and three following text bands. Bounds remain deliberately conservative. They must be checked against production rasters before implementation acceptance; they are not evidence that every example will be detected.

4. **Keep the current PageBands and metric contracts.** The measured source tails need no compaction intervals. Step 3.2 retains the tuple alias, current metric signatures, and Praefatio caller. Full-height coordinates preserve unit boundary fractions. If a later source trace establishes an interior gap, its metadata design must be specified with that evidence before changing the shared page type.

5. **Furniture masks must isolate connected scan noise before rebanding.** On v1 p235 the raw and retained band [2194,3013) includes Latin and footer noise. On p284 a right-edge artifact produces [0,847), including the leaked header and early prose. The proposed safe border masks split those bands without cutting any body band. The p94 horizontal artifact is traced to p235's bottom scan edge; the next Latin line is on p236. Source identity, rather than output pagination, makes the removal reviewable.

6. **The reported dividers do not all meet the conservative whole-column gate.** Source v1 p51 has a 22-pixel-wide lower component; p59's component is 31 pixels wide. Both fail the 12-pixel bound and remain intentionally ambiguous. P56's opening divider is narrow but advancing the full crop would discard meaningful Latin elsewhere on the page. Its measured local rectangle includes the divider and foreign-column fragments without touching the three opening Latin lines. Step 2.1 must retain its gates rather than widen them to force these examples through.

# Existing patterns and duplication

The plan retains the render cache, two-pass band processing, global scale, independent strip copying, composition packer, and ignored real-source helpers. The diagnostic test reuses `operational_range`, `real_source_image`, `render_and_band`, `unit_body_strips`, `strips_in_reading_order`, and `compose_pages`. Candidate experiments live in test code, use original crop edges, and do not write cleaned images into the cache. Later cleanup uses one source-mask table and coordinate conversion helper. Step 3.2 exercises the existing `cut_points` contract rather than adding a second packer or a shared-type migration.

The revised plan avoids generic grayscale whitening, broad threshold tuning, OCR as authoritative text, per-unit copies of page cleanup, and a general-purpose override framework. Source-specific masks are small deliberate exceptions. Further abstraction is unnecessary unless actual implementation exposes repeated logic.

# Verification and remaining implementation gates

The Phase 1 ledger freezes source identities, dimensions, crop edges, raw and retained bands, copied intervals, diagnostic canvas indices, normalized candidate mask coordinates, rebanded results, and divider-component envelopes. Full diagnostic PNGs and JSON remain under `tmp/pdfs/facsimile_artifact_cleanup/`. Poppler renders of I 95/107/112 and II 90/94/102/193/194/195 were matched by visible prose against the source scans, with full-resolution detail crops at erase boundaries.

Local formatting, clippy, the reflow tests (53 passed, 5 ignored), the full ordinary suite, and the ignored source trace passed before the step 1.1 commit. The full ordinary suite passed with 345 library tests and 9 CLI tests, with 20 source-dependent library tests ignored. The explicit source trace passed separately in release and debug modes; the final debug run took 665.49 seconds. Tests use the workspace target directory because the configured external build directory is inaccessible in the sandbox.

Human inspection of the saved erase-region crops remains valuable before Phase 3 enables masks, especially p56's first glyphs and p235's final line versus p236's first line. Phases 2 and 3 must add removal and preservation regressions; Phase 1 asserts diagnostic page availability, valid coordinates, and intact composition row accounting. No production cleanup or rebuilt-reader acceptance is claimed by this review.
