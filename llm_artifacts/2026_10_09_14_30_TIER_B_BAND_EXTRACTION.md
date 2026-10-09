# Tier-B dropped-line band extraction

Written 2026-10-09. This is the diagnostic output that feeds the Tier-B repair plan.
For each of the 20 promoted Tier-B omissions (see
[`2026_10_09_10_45_TIER_B_PROMOTION_REVIEW_WORKSHEET.md`](2026_10_09_10_45_TIER_B_PROMOTION_REVIEW_WORKSHEET.md)),
it records 22 detector-band measurements for 20 events. The 21 non-Antonius
protection tuples are installable after the repair plan's verification preparation.
The Antonius tuple records a failed single-band trial and is not an installable
standalone repair; see the
[`Antonius findings`](2026_10_09_15_30_ANTONIUS_BLOCK_COMPOSITION_FINDINGS.md).

## How this was produced

The ignored diagnostic `extract_tier_b_source_bands` in the `reflow.rs` test module
runs the real production front-half pipeline on each dropped-line source page, at
`BODY_DPI = 300`, from `build_facsimile/render_cache`:

```
real_source_image → mask_known_source_furniture → preserve_known_body_crop_edge(detect_latin_column)
→ crop_column → line_bands(LINE_MIN_INK)
```

It prints every raw band near the audit box with its ink-span and whether it survives
`render_and_band`, and saves a PNG of each candidate band under
`tmp/pdfs/inner_tomi_audit_repairs/band_extract/`. Each dropped line's band was then
**visually confirmed** against the crop PNG (the printed Latin matches the worksheet's
confirmed reading). Run it with:

```powershell
cargo test --release -p plutarch-reader extract_tier_b_source_bands -- --ignored --nocapture
```

`dimensions` is the cropped Latin-column size `(width, height)` in pixels (the column
height equals the full page height — `crop_column` crops horizontally only). Width and
height do not identify absolute column bounds. The repair plan's Step 0.1 requires the
diagnostic to emit raw and production `[left, right]` bounds, source identities, and
source overlays before constructing ledger records. `observed`
and `protected` are `{top, bottom}` bands in that cropped-column coordinate space.
`retain_known_source_body_bands` matches `observed` against the raw `line_bands` output
by **exact equality** and re-inserts `protected`, so `observed` must be the precise band
the detector emits.

## Which stage drops each line (informational)

For the non-Antonius tuples, `survives_render_and_band` distinguishes two drop
stages. Protection runs after the furniture pass and the Pass-B boundary trims;
restoration still requires source and composition tests:

- `survives=false`: the band is removed at the `render_and_band` stage — the top/bottom
  furniture zone or leading/trailing gap filter in `drop_furniture` removes it, and
  `retain_early_aligned_body_bands` (which rescues eligible aligned bands only in the
  `0.07..0.08` centre fraction) does not restore it. Most head and foot losses are here.
- `survives=true` but still missing in production: dropped later in **Pass B** by
  `drop_interior_running_head` → `drop_trailing_furniture` (a short first/last line,
  `< 0.8 × width`, detached by an above-median gap). Lysander's second line and both
  Pericles verse lines are here.

The 21 non-Antonius tuples use the same ad hoc mechanism as the 47 installed Tier-A
protection records, without heuristic changes. Survival through `render_and_band`
is stage evidence, not proof of final composition. Antonius requires the downstream
trace and method-specific checks in Phase 3 of the repair plan.

## The extraction — 22 measurements for 20 events

Lysander (2 lines, one page) and Pericles (2 lines across two pages) contribute two
entries each; Antonius's 23-line loss is one measured merged band whose standalone
protection failed the reader check. Row numbers identify the original measurements
and remain unchanged. For **clean** bands
`protected = observed`. For bands that `line_bands` **merged with a running-head
fragment** above the Latin line, `protected` is the lower Latin sub-range only (the
Greek/head scrap stays dropped), measured from the band's row ink profile.

| # | unit_id | src | page | dimensions | observed | protected | kind | confirmed line |
| --- | --- | --- | ---: | --- | --- | --- | --- | --- |
| 1 | aemilius_paulus | 0 | 326 | (790, 2982) | 273..304 | 273..304 | clean | `dedisset, strenue pugnarent, gladiisque Romano more ute-` |
| 2 | agesilaus | 1 | 110 | (798, 2834) | 2519..2558 | 2519..2558 | clean (foot) | `bellum cepit, id quod Agesilao pergratum fuit. (2) Ma-` |
| 3 | alcibiades | 0 | 242 | (787, 2981) | 2598..2623 | 2598..2623 | clean (verse) | `Tunc Alcibiades ad me ait balbutiens :` |
| 4 | antonius | 1 | 562 | (815, 2835) | 249..1356 | 249..1356 | **block (23 lines)** | `vatam dicunt … a Cæsare redemit.` |
| 5 | artoxerxes | 1 | 633 | (786, 2826) | 2522..2554 | 2522..2554 | clean (foot) | `ut quidam perhibent, ut alii, a Care quodam, cui præmii` |
| 6 | brutus | 1 | 619 | (805, 2824) | 120..225 | 193..225 | **merged** (Greek `ΟΣ.` above) | `quum leviter extremas partes attigissent paucosque dije-` |
| 7 | caius_marius | 0 | 519 | (818, 2973) | 246..292 | 246..292 | clean | `quominus ea contentio plane erumperet, sociali bello,` |
| 8 | cato_minor | 1 | 329 | (839, 2831) | 40..210 | 180..210 | **merged** (head above) | `rum. Interim Rhodi ad negotium tractandum se com-` |
| 9 | cato_minor | 1 | 352 | (804, 2842) | 2526..2564 | 2526..2564 | clean (foot) | `ducum considerata; quodsi tamen bona quadam fortuna` |
| 10 | crassus | 1 | 44 | (789, 2822) | 191..224 | 191..224 | clean (incl. ch. `V.`) | `V. Ibi tum delitescenti Crasso quotidie alimenta affere-` |
| 11 | demosthenes | 1 | 445 | (827, 2842) | 44..226 | 197..226 | **merged** (Greek `ΕΝΗΣ.` above) | `nunc promissis me movebis. Archia autem irato et mi-` |
| 12 | dio | 1 | 566 | (805, 2838) | 2540..2577 | 2540..2577 | clean (foot) | `satis confirmata tyrannide deficientes Syracusani arripue-` |
| 13 | dio | 1 | 580 | (812, 2835) | 2550..2597 | 2550..2597 | clean (foot; skew → verify, pad bottom if clipped) | `cem navibus advectus est Dionysius, Dionique arma quæ` |
| 14 | lucullus | 0 | 631 | (795, 2978) | 252..287 | 252..287 | clean | `cum gerendum Pompeio dederat, et quam in ærarium` |
| 15 | lysander | 0 | 535 | (796, 2969) | 245..278 | 245..278 | clean (line 1) | `gerere volebant, eos rideri jussit : nam ubi pellis leonina` |
| 16 | lysander | 0 | 535 | (796, 2969) | 306..339 | 306..339 | clean (line 2, Pass-B drop) | `non sufficeret, ibi vulpinam assuendam esse.` |
| 17 | marcellus | 0 | 377 | (808, 2957) | 249..282 | 249..282 | clean | `tes petebant supplices, ut in honestam aliquam militiam` |
| 18 | nicias | 1 | 25 | (812, 2832) | 204..247 | 204..247 | clean | `fatis esse dixerant, uti hoc bellum per novem triennia tra-` |
| 19 | pericles | 0 | 214 | (802, 2988) | 2599..2628 | 2599..2628 | clean (verse 1, foot, Pass-B drop) | `Megaram petentes Simætham meretriculam` |
| 20 | pericles | 0 | 215 | (796, 2974) | 249..272 | 249..272 | clean (verse 2, head, Pass-B drop) | `furto abstulerunt adolescentes ebrii :` |
| 21 | phocion | 1 | 291 | (873, 2836) | 173..239 | 206..239 | tall (whitespace tightened) | `ipsi adversariis Demostheni dicenti, Occident te, o Phocion,` |
| 22 | timoleon | 0 | 318 | (791, 2988) | 263..295 | 263..295 | clean | `largitus, audita deinde re, de qua deliberaretur, suam fere-` |

### Notes and risks

- **agesilaus p110 already has a `SOURCE_BODY_BANDS` entry** from Tier-A (a *head*
  band `observed 211..248 → protected 211..249`). Entry #2 here is the separate *foot*
  band on the same page; multiple entries per page are already supported (e.g. page 233
  has three), and the filter keys on `(source, page, dimensions)` and the observed band,
  so there is no conflict.
- **Antonius (#4) is the one high-risk entry.** `line_bands` returns the whole 23-line
  conclusion as a **single 1107-px band** (`249..1356`), not 23 separate bands — the
  inter-line rows on v2 p562 stay above `LINE_MIN_INK`, so the detector never
  split them. A standalone protection trial retained the band but failed the reader
  check. The composer packs raster strips and cuts at page boundaries; it does not
  reconstruct individual lines. The downstream loss stage and safe method remain
  open. Phase 3 requires a full trace and evaluates whole-block recognition, faithful
  crop adjustment, and glyph-safe explicit bands. Keep this tuple as diagnostic
  provenance; do not install it alone as a completed repair.
- **Merged entries (#6, #8, #11)** deliberately set `protected` below the Greek/Latin
  running-head fragment that `line_bands` merged into the band, so the fix restores the
  Latin line only and does **not** re-introduce furniture. Sub-ranges were measured from
  the band's row ink profile (bottom-most inked group → band bottom).
- **dio p580 (#13)** is the skewed-crop case flagged in the worksheet; `observed`/
  `protected` are the detected band, but verification should confirm no bottom clip and
  measure an extension of `protected` if a descender is lost (per the human directive
  to prefer extra space over a mangled line). Keep `observed` fixed, update retained
  ledger intervals and neighboring glyph evidence, regenerate output, and repeat the
  source/composition and final delivery checks after any adjustment.

### Head-fragment exclusion candidates

The repair plan proposes brutus `[129,158]`, cato_minor p329 `[114,142]`, and
demosthenes `[127,156]` for `excluded`. Their measured row-group derivation was not
recorded in this original extraction. Treat them as candidates: remeasure source ink
groups in Step 0.1, save overlays tying each interval to furniture above the Latin,
and record confirmed bounds before installing exclusion assertions. Do not present
these candidate coordinates as verified measurements. Phocion p291 tightens upper
whitespace; it has no reviewed head fragment to exclude.

### Ready-to-install non-Antonius entries

After the repair plan's Phase 0, append these 21 tuples to `SOURCE_BODY_BANDS` in
`src/facsimile/reflow.rs`. Clean entries have `protected = observed`; three head
sub-ranges and the Phocion whitespace tightening use `protected.top > observed.top`.
The failed Antonius tuple remains in table row 4 as diagnostic provenance.

```rust
// aemilius_paulus — ute[rentur]
SourceBodyBand { source: 0, page: 326, unit_id: "aemilius_paulus", dimensions: (790, 2982),
    observed: Band { top: 273, bottom: 304 }, protected: Band { top: 273, bottom: 304 } },
// agesilaus — foot "(2) Ma[gnum]" (separate from the existing head entry on p110)
SourceBodyBand { source: 1, page: 110, unit_id: "agesilaus", dimensions: (798, 2834),
    observed: Band { top: 2519, bottom: 2558 }, protected: Band { top: 2519, bottom: 2558 } },
// alcibiades — verse "Tunc Alcibiades ad me ait balbutiens:"
SourceBodyBand { source: 0, page: 242, unit_id: "alcibiades", dimensions: (787, 2981),
    observed: Band { top: 2598, bottom: 2623 }, protected: Band { top: 2598, bottom: 2623 } },
// artoxerxes — foot
SourceBodyBand { source: 1, page: 633, unit_id: "artoxerxes", dimensions: (786, 2826),
    observed: Band { top: 2522, bottom: 2554 }, protected: Band { top: 2522, bottom: 2554 } },
// brutus — Latin sub-range below Greek "ΟΣ."
SourceBodyBand { source: 1, page: 619, unit_id: "brutus", dimensions: (805, 2824),
    observed: Band { top: 120, bottom: 225 }, protected: Band { top: 193, bottom: 225 } },
// caius_marius
SourceBodyBand { source: 0, page: 519, unit_id: "caius_marius", dimensions: (818, 2973),
    observed: Band { top: 246, bottom: 292 }, protected: Band { top: 246, bottom: 292 } },
// cato_minor p329 — Latin sub-range below head fragment
SourceBodyBand { source: 1, page: 329, unit_id: "cato_minor", dimensions: (839, 2831),
    observed: Band { top: 40, bottom: 210 }, protected: Band { top: 180, bottom: 210 } },
// cato_minor p352 — foot
SourceBodyBand { source: 1, page: 352, unit_id: "cato_minor", dimensions: (804, 2842),
    observed: Band { top: 2526, bottom: 2564 }, protected: Band { top: 2526, bottom: 2564 } },
// crassus — includes chapter heading "V."
SourceBodyBand { source: 1, page: 44, unit_id: "crassus", dimensions: (789, 2822),
    observed: Band { top: 191, bottom: 224 }, protected: Band { top: 191, bottom: 224 } },
// demosthenes — Latin sub-range below Greek "ΕΝΗΣ."
SourceBodyBand { source: 1, page: 445, unit_id: "demosthenes", dimensions: (827, 2842),
    observed: Band { top: 44, bottom: 226 }, protected: Band { top: 197, bottom: 226 } },
// dio p566 — foot
SourceBodyBand { source: 1, page: 566, unit_id: "dio", dimensions: (805, 2838),
    observed: Band { top: 2540, bottom: 2577 }, protected: Band { top: 2540, bottom: 2577 } },
// dio p580 — foot (skew: verify / pad bottom if clipped)
SourceBodyBand { source: 1, page: 580, unit_id: "dio", dimensions: (812, 2835),
    observed: Band { top: 2550, bottom: 2597 }, protected: Band { top: 2550, bottom: 2597 } },
// lucullus
SourceBodyBand { source: 0, page: 631, unit_id: "lucullus", dimensions: (795, 2978),
    observed: Band { top: 252, bottom: 287 }, protected: Band { top: 252, bottom: 287 } },
// lysander line 1
SourceBodyBand { source: 0, page: 535, unit_id: "lysander", dimensions: (796, 2969),
    observed: Band { top: 245, bottom: 278 }, protected: Band { top: 245, bottom: 278 } },
// lysander line 2
SourceBodyBand { source: 0, page: 535, unit_id: "lysander", dimensions: (796, 2969),
    observed: Band { top: 306, bottom: 339 }, protected: Band { top: 306, bottom: 339 } },
// marcellus
SourceBodyBand { source: 0, page: 377, unit_id: "marcellus", dimensions: (808, 2957),
    observed: Band { top: 249, bottom: 282 }, protected: Band { top: 249, bottom: 282 } },
// nicias
SourceBodyBand { source: 1, page: 25, unit_id: "nicias", dimensions: (812, 2832),
    observed: Band { top: 204, bottom: 247 }, protected: Band { top: 204, bottom: 247 } },
// pericles verse 1 (foot of p214)
SourceBodyBand { source: 0, page: 214, unit_id: "pericles", dimensions: (802, 2988),
    observed: Band { top: 2599, bottom: 2628 }, protected: Band { top: 2599, bottom: 2628 } },
// pericles verse 2 (head of p215)
SourceBodyBand { source: 0, page: 215, unit_id: "pericles", dimensions: (796, 2974),
    observed: Band { top: 249, bottom: 272 }, protected: Band { top: 249, bottom: 272 } },
// phocion — tall band, Latin sub-range
SourceBodyBand { source: 1, page: 291, unit_id: "phocion", dimensions: (873, 2836),
    observed: Band { top: 173, bottom: 239 }, protected: Band { top: 206, bottom: 239 } },
// timoleon
SourceBodyBand { source: 0, page: 318, unit_id: "timoleon", dimensions: (791, 2988),
    observed: Band { top: 263, bottom: 295 }, protected: Band { top: 263, bottom: 295 } },
```
