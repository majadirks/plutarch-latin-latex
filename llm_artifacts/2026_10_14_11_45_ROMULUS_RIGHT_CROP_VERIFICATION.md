# Tomus I Romulus III right crop verification (2026-10-14)

The reviewed source is `Ploutarchou_Bioi_v1.pdf` PDF p. 34, 300 DPI raster
`1982×3008`. The source has dark terminal-glyph pixels through x=1782 on
Romulus III (2)–(4); x=1783–1899 is clear across the Latin body. The guarded
crop is now `[945,1790)`, width 845, leaving seven clear source columns after
the last glyph. The earlier detector bounds `[950,1774)`, source shear,
furniture mask, and excision `[1892,2135)` remain in place. The ordinary
production scale would round the 845-column strip to 1016 px, so the
fingerprint-guarded scale cap fits it into the 1004 px text measure. The raw
terminal boxes and ink counts are recorded in
`2026_10_14_10_30_ROMULUS_RIGHT_CROP_MEASUREMENT.md`.

## Checks

- `romulus_p34_right_crop_guard_rejects_unreviewed_rasters` passed; the real
  raster's positive guard passed in `romulus_chapter_break_deskewed`.
- `romulus_p34_wide_crop_fits_content_width` passed. An altered same-size
  image retains its requested scale.
- `romulus_p34_right_glyphs_restored` passed. Every reviewed terminal region
  has ink beyond the old x=1773 edge, and its exact grayscale pixels survive
  shear, row excision, crop, one retained strip, and one composed canvas in
  source order. An unaffected preceding line anchors that order. The test
  also checks that the former 828-column candidate loses source ink.
- `inner_tomi_shears_preserve_latin` passed with all eight added terminal
  regions in the repair ledger. `romulus_chapter_break_gap_normal` passed.
- `cargo build`, `cargo fmt`, `cargo fmt --all --check`,
  `cargo clippy --all-targets -- -D warnings`, and `cargo test` passed after
  the final Rust and fixture edits.

The offline command completed successfully and produced eight volumes:

```powershell
cargo run --release --bin facsimile -- --v1 Ploutarchou_Bioi_v1.pdf --v2 Ploutarchou_Bioi_v2.pdf --out build_facsimile/ --offline
```

Its local log is `tmp/romulus_phase3_offline_build.log`. The rebuilt Tomus I
inner PDF has 416 pages, unchanged from the prior verification; the other
known-skew volumes still have 416 pages (Tomus III) and 480 pages (Tomus VI).
The affected printed folios 43–44 are PDF pp. 57–58. Rendered neighboring
PDF pp. 56 and 59 were also inspected at 200 DPI. The local preview files are
`tmp/romulus_reader-056.png` through `tmp/romulus_reader-059.png`.

Source p. 34 and reader pp. 57–58 were compared at readable zoom. Folio 43
shows the chapter III heading at its normal gap and the complete section (2)
endings through `patrimonio`. Folio 44 carries `re-`, `poneret,`, `ex`, and
`eum` with clear right margins, followed once by the continuation from the
next source page. The left gutter, page break, and adjoining folios render
normally. The source pixel checks establish fidelity at the repaired edge;
the project owner's visual sign-off remains the human review step.
