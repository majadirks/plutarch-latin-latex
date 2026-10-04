# Cost-gate findings (Phase 5 of the cost-reduction plan)

**Authoring date:** 2026-10-03
**Companion to:** `2026_10_02_COST_REDUCTION_PLAN.md`

This note records the Phase 5 cost-validation measurements run against a contiguous
body-page sample (`--pages v1:200-260`, 61 pages) on a fresh dryrun sidecar
(`--retag --no-compile`), and the decisions taken from them. It is kept as a dated
record per the project convention; the figures below are a point-in-time
measurement, not a live spec.

## Reference prices used (batch-halved)

Per MTok, input / output, halved for the batched stages:
Haiku $0.50/$2.50, Sonnet $1.50/$7.50, Opus $2.50/$12.50. Verify against current
pricing before reusing.

## The decisive finding: escalation rate, not the Haiku pass, drove cost

The first samples escalated **~100% of pages to Opus** in *both* signal states,
which would blow the budget (~$70 full run). Tallying the stored `VerifyFlag`s
showed the cause was **not** the Haiku signal but the coarse OCR check: Tesseract
was OCRing the **full two-column page** (`render_page`) while Sonnet transcribed
only the **cropped Latin column**, so OCR read ~4,600 non-space chars against
Sonnet's ~2,300 — consistently ~2×, over the 1.6× high bound, flagging nearly every
page.

**Fix applied:** `pipeline::ocr_text` now OCRs the cropped Latin column
(`render_latin_column`) so the coarse check compares like with like. After the fix
the same sample escalated **9 of 54 (~17%)** — Opus is rare again, as intended.

## Measured cost (signal OFF, OCR-fixed)

Sample (54 pages transcribed, 9 escalated):

| Model | input tok | output tok | sample $ |
|---|---|---|---|
| Sonnet | 203,754 | 128,937 | 1.273 |
| Opus | 45,317 | 11,189 | 0.253 |

≈ **$0.0283 per transcribed page**. The work is ~1,250–1,328 transcribed body/
front-prose pages, so the full one-time run is **≈ $35–38** — essentially at the
owner's $35 ceiling, dominated by Sonnet **output** tokens (the transcription text
itself, ≈ $24), which is irreducible without changing the transcription.

Signal ON (first samples, combined tokens) extrapolated to **≈ $70**: Haiku's Latin
on this Didot text diverges from Sonnet on essentially every page, so it escalates
~100% to Opus — confirming the plan's thesis that Haiku is unfit as a stored source
and expensive as a signal here.

**Decision:** `DIVERGENCE_SIGNAL` is set to `false` (plan remedy #1). Sonnet is the
authoritative transcriber; numbering continuity + the coarse OCR check drive the
rare Opus escalation.

## Levers if the full run must come in comfortably under $35

In rough order of saving, without touching transcription faithfulness:
1. Lower `render::TRANSCRIBE_DPI` (300 → ~200): cuts Sonnet **input** image tokens
   (~$7 of the total), saving ≈ $3–5. Human-verify legibility first.
2. `verify::ARBITRATION_EFFORT = "low"`: cuts the ~$6 Opus share.
3. Non-body pages (front/back/skip) are excluded from transcription, so the true
   transcribed-page count is below 1,328 and the real figure trends toward the low
   end of the range.

## Open issues surfaced by the samples (not yet resolved)

1. **~13% of Sonnet batch requests fail per run** — a mix of unparseable JSON and
   "succeeded result with no text block". A few pages recur across runs (e.g. 234,
   236, 240), suggesting page-specific density or content, plus a transient
   component. Handling is correct (the pass is left unset, counted, and warned;
   nothing is fabricated or dropped), and a re-run re-batches the missing passes —
   but the rate is high. **Next diagnostic step:** capture the raw batch message for
   a failing `custom_id` (log `result.message` on parse failure / empty content)
   before deciding whether the cause is `max_tokens` truncation, a refusal, or a
   structured-output edge case. Batch requests are not individually retried the way
   synchronous calls are, so each failure costs a re-run to recover.
2. **lualatex verse-rendering bug** — a `\\`-broken verse quotation in this page
   range trips `\@icentercr` ("Paragraph ended before … was complete"), aborting the
   compile. This is pre-existing in `latex.rs` and independent of the cost-reduction
   work, but it will block the final PDF of any volume containing such verse and must
   be fixed before a full compile.

## Recommendation

Do **not** launch the full run yet: the signal-off estimate sits on the $35 line
(apply a lever to gain headroom), the ~13% failure rate means a full run leaves
pages needing a second pass, and the verse bug would crash the final compile. Decide
on the levers, fix (or triage) the two open issues, then launch.
