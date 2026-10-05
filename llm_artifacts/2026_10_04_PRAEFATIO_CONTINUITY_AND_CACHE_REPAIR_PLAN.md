# Context

The facsimile reader's Tomus V praefatio has three reported defects in `build_facsimile/tomus_5_inner.pdf`: PDF page 13 stops mid-sentence with substantial unused space before a repeated display title on page 14; JSON debris prints across pages 17 and 18; another repeated title appears on page 20 after a nearly empty page 18 and blank page 19. This plan repairs the authoritative cache and composes the praefatio as continuous editorial prose. It does not normalize the transcribed Latin or Greek. This is a diagnosis and implementation contract, not an implemented fix.

# Current state of the world

- `src/facsimile/frontmatter.rs::PRAEFATIO_PAGES` selects source index 1 (the second PDF), one-based source PDF pages 14–17. `SIDECAR` is `plutarch.frontmatter.tags.json`. These source page numbers differ from the output PDF page numbers reported above.
- `src/facsimile/mod.rs::fill_front_matter` calls `src/latex.rs::front_prose_pages` for the first tomus of source 1, currently Tomus V. Each selected page's finalized `result` is rendered separately through `front_prose_page` and `front_prose_page_blocks`. The latter emits `\freshpage`, optional `\markboth{}{}`, the rendered blocks, and `\clearpage` for every source page.
- `src/latex.rs::inner_preamble` defines `\freshpage` with a clear page and a conditional blank page. Its actual condition is `\ifodd\value{page}`; do not infer conventional recto parity from the helper's name. This plan leaves the established section-opening parity convention intact. Repeating this operation at source-page boundaries forces premature page endings and can insert blank pages.
- Visual inspection of source pages 14–17 confirms that page 14 has the opening display title. Pages 15, 16, and 17 have small running heads next to printed roman folios. Cached `result` and `pass_a` for pages 15 and 17 incorrectly include those running heads as `Heading` blocks containing `PRÆFATIO.`. Page 16 correctly omits its running head. The extra headings are cached classification errors, not headings invented by the LaTeX emitter.
- Source page 14 ends `Parisinis testimoniis confirmatas in`; page 15 begins `ordinem recepi`. Source page 15 ends `consentiente ipso`; page 16 begins `Plutarcho M. 261, 7.`. The first paragraph on each of pages 15 and 16 has `continues_previous: false` in both `pass_a` and `result`, despite these sentence continuations. Page 17 begins a genuine new paragraph, `Interpretationem latinam subjicere placuit Xylandri`, followed by the separate closing dateline.
- Page 16's sole paragraph in both `pass_a` and `result` ends with the legitimate `cap. LXVIII, 4 e cod. 1676 dedi καθίσταντο.` followed by `》》》》》》》 ,continues_previous\":true}]}***END OF JSON***】】]`. The source image ends after the Greek word and period; none of that suffix is printed source prose. The cached JSON itself parses successfully because this debris is inside the `text` string.
- All four finalized records identify `claude-sonnet-5` as `result_model`. `src/llm/transcribe.rs::parse_pagedoc` extracts a balanced JSON envelope and deserializes it, but does not check text strings for leaked protocol markers. `src/verify.rs::reconcile_all_in_scope` accepts unflagged Sonnet candidates directly and skips already finalized records. Existing caches therefore retain this defect across builds. The saved raw completion is not available here; the precise model-generation mechanism cannot be established from the cache alone.
- `src/assemble.rs::append_blocks` already joins compatible paragraph or verse continuations for body text. It is private and is called by `assembled_text`. `render_blocks` in `src/latex.rs` does not interpret continuation flags itself. Reuse that assembly rule instead of implementing another joining algorithm.
- The facsimile Monitum uses `front_prose_pages_until` and source scans; the transcription reader uses the separate role-driven `front_prose`. Preserve their existing contracts. `--unit` and `--pages` builds suppress front matter and cannot validate this fix. Full offline builds are documented in `README.md` and compile printing wrappers from actual inner page counts.

# Desired end state

Tomus V emits exactly one praefatio display title, copied exactly from source page 14. Its four source pages flow as a single logical section: sentence continuations join without paragraph breaks, while the paragraph beginning `Interpretationem` and the closing dateline remain distinct. There is one section-opening parity operation and one trailing page break, with no source-page parity restarts. Natural line/page breaks, established flyleaves, section transitions, and signature padding remain valid.

The finalized cache excludes the source running heads and the verified non-source suffix through explicit, reviewable edits. Candidate passes remain historical evidence. A narrow shared validator rejects known protocol contamination in new transcription replies and in finalized praefatio cache text before output; it reports source/page/block context and never strips or rewrites text. An invalid finalized praefatio stops the build rather than silently producing a book without the section. All existing tests and the local formatting, clippy, and test gates pass, and a rebuilt full PDF is visually checked.

# Phase 1 — Repair the authoritative praefatio cache (Suggested implementer: Opus)

## 1.1 Source-check and correct finalized records

Edit only the `result` blocks of `(source: 1, source_page: 15)`, `(1, 16)`, and `(1, 17)` in `plutarch.frontmatter.tags.json`:

- Remove the leading `Heading` block on pages 15 and 17. These are running heads, which `FRONT_PROMPT` already excludes.
- Set the first paragraph's `continues_previous` to `true` on pages 15 and 16. Keep page 17's paragraph and dateline flags false.
- On page 16 remove exactly the suffix after the legitimate terminal period following `καθίσταντο`. Preserve every preceding character, including spelling, accents, Greek, ligatures, punctuation, and whitespace.
- Preserve `pass_a`, `pass_b`, source hashes, image hashes, provenance, statuses, and other records. Keeping the original candidates records what the model returned; generation reads `result`. Do not mislabel a manual repair as an Opus result or invent a new cache field.
- Append a dated cache-repair explanation to this plan identifying these exact edits and their visual source evidence. A cache edit is mandatory; no downstream heading deduplication or suffix trimming is permitted.

Add `praefatio_cache_has_one_display_heading`, `praefatio_cache_marks_sentence_continuations`, and `praefatio_cache_ends_page_sixteen_at_source_period` in the existing `src/facsimile/frontmatter.rs` test module. Load the committed sidecar using a manifest-relative path and check finalized records only; explicitly allow the retained historical candidate to contain its original artifact.

### LLM verification

Inspect `git diff -- plutarch.frontmatter.tags.json` and confirm only the specified finalized blocks changed. Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test praefatio_cache`. Render source pages 14–17 with Poppler and compare the heading placement, joins, and terminal period against the cache.

### Human verification

Compare the cache diff with `Ploutarchou_Bioi_v2.pdf` pages 14–17. Confirm the two removed titles are page furniture, both continuations are correct, and no source characters were removed with the suffix.

# Phase 2 — Compose one continuous praefatio section (Suggested implementer: Sonnet)

## 2.1 Share continuation assembly and use it for listed front prose

In `src/assemble.rs`, make `append_blocks` `pub(crate)` and retain its current implementation and body behavior. In `src/latex.rs::front_prose_pages`, collect the selected finalized documents into one block stream by calling this helper in the caller-supplied page order. Render that stream once through `front_prose_page_blocks(..., true)`. Return an empty string when no blocks were collected, avoiding an empty section/parity page. Reset the running head once at section opening. Keep the final section clear page.

Do not drop repeated headings by string matching: the corrected cache supplies the right structure. Do not change `front_prose`, `front_prose_pages_until`, `\freshpage`, `render_blocks`, or body unit openings. Retain the existing missing-record skip behavior, but clear the incoming first block's continuation flag after any missing record, missing result, or empty document so text is not joined across an omitted source page. Implement this by cloning that document and setting its first block's flag false before calling the shared helper; do not mutate the cache. Track whether the previous listed record supplied a nonempty result; test empty documents as gaps too.

Update the `front_prose_pages` documentation to describe section composition. Add tests in the existing `src/latex.rs` module: `front_prose_pages_flows_as_one_section`, `front_prose_pages_rejoins_continued_paragraphs`, `front_prose_pages_preserves_new_paragraph_and_dateline`, `front_prose_pages_empty_emits_nothing`, and `front_prose_pages_does_not_join_across_missing_page`. Assert one opening `\freshpage`, one trailing `\clearpage`, one head reset, no internal forced breaks, the exact joined strings, and distinct paragraph separators. Include unmodified ligatures and Greek in the fixtures. Update `front_prose_pages_emits_only_listed` to expect one running-head reset and describe the section in its comment; retain its exclusion assertions. Revise the existing `front_prose_pages_until_emits_all_when_sentinel_absent` test to assert that helper's established per-page output directly, since equivalence to continuous `front_prose_pages` is no longer its contract. Preserve the Monitum sentinel tests and body assembly tests.

### LLM verification

Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test front_prose`, and `cargo test assembled_text_rejoins_continued_paragraph`. Inspect synthetic output for exact text preservation and one section boundary pair.

### Human verification

Read the generated praefatio LaTeX. Confirm `in ordinem recepi` and `ipso Plutarcho` are within their continuing paragraph, while `Interpretationem` and the dateline have paragraph boundaries. Confirm the Monitum helper still terminates its prose before the embedded errata.

# Phase 3 — Reject protocol artifacts without rewriting source text (Suggested implementer: Opus)

## 3.1 Add one validator at ingestion and finalized praefatio use

Add `pub(crate) fn validate_pagedoc_text(doc: &PageDoc) -> anyhow::Result<()>` in `src/llm/transcribe.rs`. Check each block for the exact `***END OF JSON***` marker and for a `continues_previous` field spelling immediately followed by a JSON-style colon and boolean value, allowing the observed literal backslash before the quote and whitespace. Implement the latter with one small regex definition reused for all blocks. Match structural syntax, not the bare word, generic braces, quotes, or unusual Unicode punctuation. Return a descriptive error with the zero-based block index and offending marker; never modify the string. These are narrow protocol signatures, not a general claim to detect every hallucination.

Call the validator after deserialization in `parse_pagedoc`. Existing ingestion/reconciliation error behavior then handles a rejected reply without receiving sanitized prose. Add tests `parse_pagedoc_rejects_end_of_json_inside_text`, `parse_pagedoc_rejects_leaked_continuation_field`, `parse_pagedoc_preserves_source_punctuation`, and `parse_pagedoc_accepts_external_json_end_marker`. The external marker test verifies valid prose inside a valid envelope remains accepted when commentary follows outside the JSON object. Preserve all existing envelope tests.

In `src/facsimile/frontmatter.rs`, add `pub(crate) fn validate_praefatio_results(sidecar: &Sidecar) -> anyhow::Result<()>`, iterating available finalized results for `PRAEFATIO_PAGES` and calling the same validator with source/page context. Do not validate historical candidate passes. In `src/facsimile/mod.rs::fill_front_matter`, after successfully obtaining the sidecar and before assigning volume prose, call it with `?` when an emitting volume is the first of source 1. Keep this outside the existing load-error catch that skips unavailable prose, so contaminated finalized text causes a build error. Preserve existing missing/stale-cache handling. Do not rewrite the cache, trigger retranscription, or silently fall back to a candidate.

Add `praefatio_validation_rejects_contaminated_final_result` and `praefatio_validation_ignores_historical_passes` to frontmatter tests, using synthetic sidecars. Extract `assign_front_prose(volumes, sidecar)` in `src/facsimile/mod.rs`, returning `Result<()>`, from the existing assignment loop. Validate before mutating any volume, then assign prose. Test `assign_front_prose_rejects_poisoned_praefatio_without_mutating_volumes` with synthetic volumes and sidecars. This helper runs before inner output is written and has no network dependencies. Retain existing offline cache-loading and lazy-client tests; do not introduce a second loading path.

### LLM verification

Run `cargo fmt --all`, `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test parse_pagedoc`, `cargo test praefatio_validation`, and `cargo test assign_front_prose`. Inspect error output for source 1, source page 16, and the block index. Confirm the repaired committed results pass while retained candidate evidence does not prevent an offline build.

### Human verification

Read a representative rejection message and the cache correction guidance. Confirm the message makes the location actionable and the validator never silently alters scholarly text.

# Phase 4 — Rebuild and verify the complete book (Suggested implementer: Opus)

## 4.1 Run local gates and inspect rebuilt PDFs

Run `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `cargo build`. Then use the full offline build command from `README.md` with a separate output directory such as `build_praefatio_check`, preserving the reported original PDF for comparison:

```powershell
$env:PLUTARCH_LUALATEX = "$env:LOCALAPPDATA\Programs\MiKTeX\miktex\bin\x64\lualatex.exe"
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_praefatio_check --offline
if ($LASTEXITCODE -ne 0) { throw "Facsimile build failed" }
```

Require actual compiled inner and imposed PDFs, not merely a successful LaTeX-only fallback. Do not use `--unit`, `--pages`, `--retag`, or `--redetect` for this validation. Run `pdftotext -layout` on the rebuilt Tomus V inner PDF and search its praefatio for one `PRÆFATIO.` title and absence of the protocol signatures. Locate the section by its text because pagination will change. Render every praefatio page and the adjoining section pages using `pdftoppm`; inspect each image. Check that the former page-13 join now flows naturally, there is no debris-only page or parity blank inside the praefatio, the final paragraph/dateline remain distinct, and the first body unit opens correctly. Inspect Tomus I's Monitum and embedded errata as a regression check. Inspect the regenerated Tomus V printing wrapper for correct actual page count, complete content, and signature padding; do not hard-code the old count.

### LLM verification

Record gate results, build output locations, the actual praefatio page span, title count, text searches, and visual inspection findings in this plan. Verify offline cache-loading and lazy-client tests prove no client is invoked; the facsimile prose path does not print a usage report and `git diff` contains no unintended cache edits. Generated outputs remain local artifacts; retain this plan as the historical record.

### Human verification

Read the entire rebuilt praefatio alongside source pages 14–17, focusing on the two joins, the single opening title, the clean source-page-16 ending, and the closing dateline. Inspect the transition into the first Life and a representative imposed sheet. Blank flyleaves and signature padding are intentional; accept no blank page inserted within the praefatio.

# Implementation record

2026-10-04, step 1.1: visually verified source pages 14–17. Removed finalized running-head blocks from pages 15 and 17, marked the page-15 and page-16 paragraph continuations, and removed the non-source suffix after the terminal period on page 16. Original candidate passes remain intact. Cache edits preserve UTF-8 and all preceding text. Review corrections specify the changed head-reset assertion, empty-document gaps, and a pure prose-assignment test seam. Rust checks use --target-dir target because the configured external target directory is inaccessible in the sandbox.

2026-10-04, step 2.1: listed prose now shares append_blocks continuation assembly and renders one section. Missing records, missing results, and empty documents break continuation. Updated both affected existing tests and added five section/continuity tests. Formatting, clippy, front-prose tests, and the body continuation regression passed using the workspace target directory.

2026-10-04, step 3.1: shared validation rejects the exact END OF JSON marker and leaked continuation-field syntax with ordinary or backslash-containing quotes, whitespace, either boolean, and a trailing word boundary. The regex compiles once per validation call and is reused across blocks. Pure assign_front_prose validates before mutating volumes; its tests cover rejection and real finalized-cache composition without network dependencies. Ten parser tests, two validator tests, two assignment tests, offline loading, lazy-client coverage, formatting, and clippy passed.
