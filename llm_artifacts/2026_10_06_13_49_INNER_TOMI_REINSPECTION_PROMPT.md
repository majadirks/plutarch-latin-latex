# Inner-tomi re-inspection prompt (2026-10-06 13:49)

This file is a ready-to-use prompt for a fresh Claude session tasked with
visually re-inspecting the rebuilt facsimile readers. It generalizes the defect
classes that earlier passes found so that *other instances of the same class*
are caught this time. Paste everything under "Prompt for Claude" into the new
session.

Background for whoever is dispatching the inspection (not part of the prompt):
the prior work is recorded in
[the visual review findings](2026_10_05_INNER_TOMI_VISUAL_REVIEW_FINDINGS.md),
[the minimal repairs plan](2026_10_05_INNER_TOMI_MINIMAL_REPAIRS_PLAN.md), and
[the Cato Minor duplicate-scan note](2026_10_06_CATO_MINOR_DUPLICATE_SCAN_NOTE.md).
The key lesson driving this prompt: the Cato Minor D3 duplicate was *recorded but
not acted on* during the main repair pass, and D1/D2/D3 were each found
separately — a found-but-unfixed category tends to recur. So this pass is a
deliberate re-sweep of every known defect class across all eight tomi, not just a
recheck of the specific page locators already on file.

---

## Prompt for LLM Agents

You are inspecting the eight rebuilt inner facsimile readers of a Latin-only
scholarly edition of Plutarch's *Lives*, at
`build_facsimile/tomus_N_inner.pdf` (N = I..VIII). The readers are assembled from
scanned source pages (`Ploutarchou_Bioi_v1.pdf` and `Ploutarchou_Bioi_v2.pdf`)
that are cropped, de-furnitured, and reflowed into clean typeset pages. Your job
is to find residual scan/layout defects in this fresh build and report them —
**not** to fix them and **not** to alter any Latin.

### Hard constraints (read first)

- **All Latin is source text. Never alter it.** Do not retype, normalize,
  modernize (u/v, i/j, ligatures, long s), re-punctuate, re-capitalize, expand
  abbreviations, or "correct" the Latin. Your deliverable is a findings report.
  Removing furniture or dropping a duplicate is an image/selection matter for a
  later repair pass, not permission to touch glyphs.
- **Report, don't repair.** Do not edit code, fixtures, the unit table, source
  PDFs, or transcription caches. Save any diagnostic renders under `tmp/` only.
- **Locate passages by recognizable Latin prose, then confirm against the
  printed folio number in the running head.** Do not trust inner-PDF page
  numbers, reader folios, or imposed-sheet page numbers as interchangeable — they
  differ, and old locators shift after every rebuild. Always report *both* the
  physical inner-PDF page (one-based) and the printed reader folio, plus a short
  Latin quotation anchoring the location.
- **A questionable finding is still worth reporting**, flagged as unconfirmed.
  Note explicitly when a defect is faint enough that it might be a false
  positive, and when you could not inspect something at sufficient resolution.

### Method

1. Render every tomus to labeled contact sheets at low DPI to triage, then
   re-render every suspect region and **every body-section ending** at high
   resolution (≥180 DPI) before asserting a finding.
2. For each of the eight categories below, sweep **all eight tomi**, not just the
   pages previously flagged. The specific page locators from the prior review are
   examples of each class — assume more instances exist elsewhere.
3. Produce a findings report structured like
   `2026_10_05_INNER_TOMI_VISUAL_REVIEW_FINDINGS.md`: a table per category with
   ID, location (physical page + reader folio + printed source folio + Latin
   anchor), the visible finding, and an assessment. Mark each finding confirmed
   or unconfirmed. Do not propose implementation; this is a findings record.

### Categories to sweep (each is a known class; find new instances)

**1. Repeated / duplicated / out-of-order scans.** Prior instances: Eumenes↔
comparison (D1), Agis (D2), Cato Minor folios 926–935 appearing twice (D3). The
reliable signal is the **printed running-head folio numbers**: if the folio
sequence in a Life resets, repeats, or runs out of order, a page block was
scanned twice. Scan the folio progression of *every* Life end to end. Where you
suspect a repeat, confirm by matching the Latin prose on the candidate pages and
check that the surviving join seam would be grammatical and lose no distinct
Latin. Report the full occurrence range of each repeat and the folio evidence.

**2. Leaked source running-heads and folio numbers.** This was the most numerous
class. Prior instances: `LUS.`/169 (Camillus), `RIOLANUS.`/275/277 (Coriolanus),
`EON.`/285/295 (Timoleon), `NDER.`/537 (Lysander), and **Greek** fragments
(`ARKIOS`, `ΩΝ.`). Sweep every page for: isolated runs of capital letters that
are not part of a flowing prose line (any length — they are cropped running
heads, so the surviving fragment may be short or long, and usually ends in a
period); isolated multi-digit numbers; and **Greek-letter fragments**
(easy to overlook — look specifically for them). These tend to sit near chapter
breaks and inside whitespace gaps.

Do not filter by fragment length. A leaked running head is a *truncated* all-caps
word whose left edge was cropped off, so the surviving fragment can be any
length — `LUS.` (4), `EON.` (4), `NDER.` (5), and `RIOLANUS.` (9) are all the
same class. The signal is "a run of capital letters (Latin or Greek) that is not
part of the flowing prose line and typically ends in a period," not a specific
letter count. Treat any such isolated all-caps run — short or long — as a
candidate leak, and likewise any isolated multi-digit number sitting apart from
the prose.

Also confirm that masks installed in the prior
repair actually fired on this rebuild — a mask can be present in config yet miss
if a page boundary shifted.

**3. Google watermarks in body prose.** Prior instances: "Digitized by Google"
inside the prose on two Timoleon/Coriolanus pages; only two were confirmed and
the review warned not all pages were checked. Re-scan every whitespace/gap region
across all tomi for gray watermark text, logo, or QR fragments. (The full-page
Google reproduction notice at physical page 8 of each tomus is an intentional
retained front-matter page — note it but do not flag it as a defect.)

**4. Divider-only / effectively-blank body pages.** Prior instances: Timoleon
(G2), Brutus (G3), post-Marius furniture page (O5). These are body pages carrying
a generated heading + folio + a source divider rule but **no prose**.
Distinguish them from legitimate white recto separators between units and from
end-of-signature padding. Confirm none remain.

**5. Large interior gaps from copied source margin/whitespace.** Prior
instances: G1, G4, G6 — multi-line blank intervals mid-sentence, often where a
word is split across the gap (e.g. `ci`/`num`, `subli-`/`partem secutus`,
`strepitu aquæ`/`verba non percipientibus`). For each gap, verify the split word
rejoins correctly and no intervening Latin line was dropped. Distinguish genuine
printed chapter/paragraph spacing (retain) from copied scan margin (defect).

**6. Surviving vertical divider rules.** Prior instance class: O7 — thin, thick,
or broken vertical scan rules running alongside the Latin, sometimes through
otherwise-blank regions. Pay particular attention to **heavy right-edge rules**
(the second Cato scan and VI/358 carried these). Note each, and whether it abuts
Latin pixels.

**7. Raster page break slicing through a text line.** Prior instance: O8B — a
page boundary cutting horizontally through the letters of a line, leaving a
clipped sliver at the top or bottom of the adjacent page. Check the top line and
bottom line of every page for horizontal clipping across the boundary.

**8. Section-ending completeness.** No terminal truncation was confirmed last
time, but IV and VIII had historic truncations and pagination can shift on
rebuild. Re-inspect **every** body-section ending across all eight tomi and
confirm each concludes on a complete sentence, with any split word (e.g. a
trailing `Cæ-`) correctly continued.

### What *not* to flag (intentional, already decided)

- The full-page Google reproduction notice at physical page 8 of each tomus (W3).
- The Praefatio typography/scale transition around V/14–17 (O8).
- Entirely white recto pages at unit boundaries (units start on rectos) and
  final signature padding.
- Sparse pages that contain genuine complete concluding lines.

### Deliverable

A single dated findings file in `llm_artifacts/` (named
`YYYY_MM_DD_HH_MM_...` per the AGENTS.md convention), following the structure of
the 2026-10-05 visual review findings: a scope/method section, one table per
category above with confirmed/unconfirmed status and dual locators (physical
page, reader folio, printed source folio, Latin anchor), and a short suggested
repair-priority list. Preserve the prior findings and plans on disk; this is a
new dated record, not an edit of the old one.
