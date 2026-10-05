# Facsimile volume endings audit

## Scope and result

Read-only inspection on 2026-10-05 checked the final body page of each of the eight existing inner PDFs against the corresponding source ending. Generated `.tex` image references and LuaLaTeX shipment logs identified the last body page, rather than treating the PDF's final padded page as body content. The last two body pages and source boundary neighborhoods were rendered for inspection. Source continuation text was extracted as supplementary evidence; visual raster inspection determined the endpoint findings.

Two of eight volume endings are truncated by explicit source endpoints: Tomus IV's Comparatio Cimonis et Luculli and Tomus VIII's Otho. The other six final body pages reach their source closing sentences. This is an audit of volume endings, not a complete line-by-line audit of every unit or every interior source page. The previously identified missing words within Lysander's Life remain a separate reflow defect.

All page numbers below are one-based physical PDF indices. v1 means `Ploutarchou_Bioi_v1.pdf`; v2 means `Ploutarchou_Bioi_v2.pdf`.

| Tomus | Final unit | Last inner body page | Actual source closing page | Finding |
| --- | --- | ---: | --- | --- |
| I | Camillus | 397 | v1 195, before Pericles's title | Closing sentence through `dolorem attulerit.` is present. |
| II | Comparatio Pelopidae et Marcelli | 428 | v1 393, before Aristides's title | Closing sentence through `virtus amatur.` is present. |
| III | Comparatio Lysandri et Syllae | 412 | v1 584 | Conclusion through `primam laudem deferamus.` is present. Source 585 starts Cimon and contributes no body to the preceding comparison. |
| IV | Comparatio Cimonis et Luculli | 118 | v1 637 | Truncated at the end of v1 636. Entire concluding Latin column on 637 is missing. |
| V | Comparatio Agesilai et Pompeii | 379 | v2 191 | Conclusion through `defectionem ad hostes eorum fecit.` is present, although source 191 has a Greek running head. Source 192 starts Alexander. |
| VI | Cato Minor | 360 | v2 363, before Agis's titles | Closing sentence through `apud Philippos periit.` is present. |
| VII | Comparatio Demetrii et Antonii | 424 | v2 565 | Conclusion through `sese vita subduxit.` is present, although source 565 has a Greek running head. Source 566 starts Dio. |
| VIII | Otho | 296 | v2 708, first complete occurrence | Truncated at v2 704. Source pages 705–708 are missing. Pages 709–710 repeat 707–708 and must not be added as another body occurrence. |

## Endpoint mechanism and Otho diagnosis

Both source-final units have pinned cache rows with incomplete endpoints: `comparatio_cimonis_et_luculli.last_page = 636` and `otho.last_page = 704`. `units::ranges` honors those endpoints and `unit_body_strips` never requests later pages. Nonterminal volume endings derive their ranges from the following unit's opening; this explains why Greek-headed closing pages can be present in Tomi V and VII even though Latin running-head classification does not recognize them.

Tomus VIII page 296 ends in chapter XII with the source-704 line `qui et peritiam et audaciam secum ad pugnam afferre`. Source 705 continues with `videbantur,` and subsequent pages carry the remainder through chapter XVIII. Source 708 concludes with the line ending `cinæ se conjunxerunt.` following the preceding line's `Cæ-` hyphenation. The printed folios of source 707–708 are 1280–1281. Source 709–710 repeats those folios and the same body text in separate scans. Source 711 is blank; 713 is publishing back matter. The operational endpoint should therefore be 708, not 704, 710, or the source's full 722-page count.

The exact historical detector invocation that produced 704 was not reproduced. Alternating Greek heads and OCR of Latin `OTHO` as visually confusable characters can undermine classification; the proven present failure is the pinned endpoint excluding pages 705–708. A complete Greek matcher is unnecessary for the user's preferred repair: explicit verified endpoints for both source-final units cover the known truncations without changing text recognition.

## Decision for the implementation plan

Use one shared table of edition-specific endpoint overrides keyed by `(source, unit_id)`: `(0, comparatio_cimonis_et_luculli) -> 637` and `(1, otho) -> 708`. Apply it when generating final-unit endpoint metadata and explicitly repair both current pinned cache rows. Preserve human pin precedence. Tests must establish that unrecognized trailing headings do not undo these known endpoints, that blank/back-matter pages remain excluded, and that Otho's duplicated source scans are not emitted twice. Greek OCR matching is outside the selected scope.

## Evidence

Diagnostic files are under `tmp/pdfs/unit_recto_diagnosis/`: `endings_audit.json` maps each final unit to its cache range, last body filename, and log-derived inner page; `end_t*_*.png` renders final output pages; `end_s0-*.png` and `end_s1-*.png` render source boundaries; `end_t*_source.txt` and `otho705-716.txt` contain supplementary source OCR. Existing page-637 and Tomus-IV renders from the earlier diagnosis are in the same directory.

No generator code, source PDFs, unit cache, or existing inner PDFs were changed by this audit.
