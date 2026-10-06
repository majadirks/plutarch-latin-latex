# Post-rebuild inner-tomi review

The frozen repairs are implemented and all eight inner and eight imposed PDFs were rebuilt on 2026-10-06. The following decisions concern findings deliberately left open in the handoff. **The candidate masks below have not been added to production.** They were applied to independent owned-image previews using the existing extraction/composition functions, at the unchanged production scale 1.2023952007293701.

## Findings requiring additional corrections

| Finding | Fresh output evidence | Diagnosis and proposed correction |
| --- | --- | --- |
| G1 | [II/219](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-2-page-219.png), [II/220](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-2-page-220.png) | Source 0 page 296's detected column is x=1980–1990: the scan edge. **The entire Latin column on that source page is omitted.** Masking the isolated far-right edge restores detected crop x=940–1727 and the missing chapters II/III, with the p295→296 and p296→297 continuations. |
| G2 | [II/235](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-2-page-235.png), [II/236](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-2-page-236.png), [II/237](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-2-page-237.png), [II/238](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-2-page-238.png) | Source 0 page 304's detected column is x=1977–1988: another scan edge. **Its Latin column is omitted.** The candidate restores x=917–1709, including `...ad saxum quod-` before page 305's `dam sublime...`. |
| G3 | [VIII/130](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-8-page-130.png), [VIII/131](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-8-page-131.png), [VIII/132](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-8-page-132.png) | Source 1 page 625's detected column is x=1803–1816: a scan edge. **Its Latin is omitted.** The candidate restores x=980–1767, the `subli-`/`dens` continuation from 624, the complete intervening prose, and the continuation to 626's `partem secutus.`. [Candidate canvas 60](../../tmp/pdfs/inner_tomi_repairs/candidate_rules/brutus-candidate-060.png) shows the restored first continuation. |
| O8B | [II/240](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-2-page-240.png), [II/241](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-2-page-241.png) | Source 0 page 306's right scan-edge ink merges two body lines into a tall strip. The actual production cut crosses `segniter negligenterque agere...`. The margin mask restores separate body bands; [candidate canvas 23](../../tmp/pdfs/inner_tomi_repairs/candidate_rules/timoleon-candidate-023.png) holds both lines complete. No splitter change is needed in this preview. |
| G6 | [III/175](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-3-page-175.png) | Source 0 page 470's edge connects the final Latin band to lower margin ink. The copied tail runs through row 2782, separating `...strepitu aquae` from page 471's `verba non percipientibus...`. [Candidate canvas 3](../../tmp/pdfs/inner_tomi_repairs/candidate_rules/pyrrhus-candidate-003.png) restores the continuous sentence with ordinary seam spacing. |
| O7, VI/358 | [VI/358](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-6-page-358.png) | The surviving right rule is source 1 page **361**, not the already-reviewed O6 page 337. The candidate excludes the far-right edge beyond the Latin. Source 359 carries a second occurrence of the same Cato passage without that rule; no additional duplicate-page exclusion is proposed in this review. |
| W1, W2/G4 residual space | Source 0 pages 287/299 still copy through rows 2940/2922 after the approved watermark masks | Far-right scan-edge ink remains, copying 372/354 rows beyond the reviewed last-Latin boundary. Separate right-margin masks are proposed; the approved watermark rectangles remain unchanged. |
| O1B, O1D/G8B, G5 residual space | Upper merged bands on source 0 pages 281/298/308 | Separate far-right scan-edge masks remove the connected upper bands. The existing header rectangles remain unchanged. Page 281 needs a 48-pixel margin rectangle: the first 25-pixel trial left part of its rule. |
| O4 residual space | Source 1 page 539's first retained band spans rows 49–598 despite header removal | The full source raster shows a far-right scan-edge rule connecting the header area and early body. A separate margin mask is proposed. This does not change the confirmed header rectangle or erase its `trato` continuation. |

## Exact proposed margin masks

Coordinates are **half-open full-source pixels at 300 DPI**. Each normalized production rectangle is `[left / width, 0.0, 1.0, 1.0]`. These are new margin masks, not expansions of the approved header/watermark masks.

| Source/page | Raster | Proposed pixel rectangle | Candidate detected Latin crop | Purpose |
| --- | --- | --- | --- | --- |
| 0/281 | 1948×2974 | [1900, 0, 1948, 2974] | 1068–1859 | O1B upper-band rule |
| 0/287 | 1900×2975 | [1875, 0, 1900, 2975] | 1073–1829 | W1 copied tail |
| 0/296 | 1990×2984 | [1965, 0, 1990, 2984] | 940–1727 | G1 omitted prose |
| 0/298 | 1990×2982 | [1965, 0, 1990, 2982] | 948–1741 | O1D/G8B upper-band rule |
| 0/299 | 1900×2963 | [1875, 0, 1900, 2963] | 1039–1823 | W2/G4 copied tail |
| 0/304 | 1988×2985 | [1963, 0, 1988, 2985] | 917–1709 | G2 omitted prose |
| 0/306 | 2001×2988 | [1976, 0, 2001, 2988] | 941–1726 | O8B merged body lines |
| 0/308 | 1998×2982 | [1973, 0, 1998, 2982] | 949–1734 | G5 upper-band rule |
| 0/470 | 2016×3023 | [1991, 0, 2016, 3023] | 972–1805 | G6 copied tail |
| 1/361 | 1810×2848 | [1785, 0, 1810, 2848] | 1000–1773 | O7, VI/358 right rule |
| 1/539 | 1824×2838 | [1799, 0, 1824, 2838] | 971–1771 | O4 connected scan edge |
| 1/625 | 1816×2842 | [1791, 0, 1816, 2842] | 980–1767 | G3 omitted prose |

The proposed left edges are beyond the candidate Latin crop's right edge on every page, with a visible white margin separating the type and scan edge. The source is never retyped. Full source images with a gray boundary marker, independent cleaned-column crops, and candidate canvases are retained under `tmp/pdfs/inner_tomi_repairs/candidate_rules/`. The authoritative mask/duplicate ledger will be amended only after human confirmation.

The candidate unit canvas counts are Timoleon 50 (current 51), Coriolanus 48 (48), Brutus 62 (62), Pyrrhus 57 (57), Cato minor 112 (112), and Antonius 102 (102). Counts alone are not preservation evidence: the restored Latin and complete seams are the acceptance criteria.

## Recommended retentions and findings already resolved

- **G7, I/36:** retain the chapter XXI gap. Comparing the rebuilt page with source 23 shows the same genuine space above `XXI. Solvens ex Creta Delum appulit...`. The nearby rule is distinct from the chapter gap and is not erased by this spacing disposition.
- **G8, I/46:** retain the chapter XXVIII gap. Source 28 has genuine space between `...et Cynocephalas extant.` and `XXVIII. Haec memoratu digna...`, corresponding to the rebuilt page. Do not compact that source typography.
- **O7, V/195–196:** the former divider interruption from the repeated comparison scans is gone after D1. Fresh V/195 contains a single comparison opening and the continuous `magno in pretio habitus` join.
- **O7, VII/21:** the reported repeated-occurrence divider is gone after D2. The fresh page lies in the retained continuation; no additional Agis erasure is proposed.
- **O5:** Marius's complete terminal sentence is on fresh III/297. III/298 is now an ordinary empty recto-start separator before Lysander, without source ornament, body image, running head, or folio.
- **W3/O8:** retain the reproduction notice and Praefatio transition as previously confirmed.

## Additional Cato header occurrence uncovered by final locator verification

The frozen O6 source **337** rectangle is implemented correctly. Its source page contributes to
fresh **VI/308–310**, rather than the original finding's VI/329 locator. **VI/329 still leaks
the Greek header from source page 347**, which repeats the same folio-934 passage. Both
occurrences begin with `nonnulli etiam amici Pompeii...`. This additional scan occurrence is
not covered by the confirmed page-337 rectangle; that rectangle and its no-right-rule decision
remain unchanged.

Source 1 page **347** is **1787×2866**. Header/folio ink occupies rows **154–193**;
the first body ink begins at **219**. The separately measured proposed header rectangle is
**[140, 145, 1720, 205)**, normalized `[140/1787, 145/2866, 1720/1787, 205/2866]`.
Its bottom edge is 14 pixels above the first body ink. Every pixel outside the rectangle is
unchanged in the independent preview. **No right-edge mask is proposed for page 347.**

Evidence: [fresh VI/329](../../tmp/pdfs/inner_tomi_repairs/rebuilt/tomus-6-page-329.png),
[source box](../../tmp/pdfs/inner_tomi_repairs/rebuilt/cato-source-347-header-box.png),
[header-only candidate](../../tmp/pdfs/inner_tomi_repairs/rebuilt/cato-source-347-header-candidate.png).
This is a separate uncovered occurrence, not a remeasurement or expansion of the confirmed page-337 mask.

## Review decision

Pending human review: approve the twelve exact margin masks, the separately measured page-347 header mask, and the G7/G8 spacing retentions, or identify particular rows that should remain unchanged. After confirmation, freeze the decisions in the ledger, extend the existing mask tests to handle multiple reviewed records for one source page, add source-pixel/continuation/whole-line regressions, rebuild all sixteen PDFs again, and verify the new locators. The current sixteen PDFs contain the confirmed first-stage repairs and the unresolved defects illustrated above.
