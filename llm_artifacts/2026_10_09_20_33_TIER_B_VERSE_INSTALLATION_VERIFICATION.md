# Context

Three measured source-body protection records cover the Alcibiades verse on v1
p242 and the two Pericles verse lines across v1 p214/p215. They preserve original
raster pixels, indentation, spelling, and punctuation. No transcription changes
are involved.

# Installed measurements

| Unit | Source/page | Production column | Column dimensions | Observed and protected rows |
| --- | --- | --- | --- | --- |
| alcibiades | v1/242 | [946,1733) | (787,2981) | [2598,2623) |
| pericles | v1/214 | [922,1724) | (802,2988) | [2599,2628) |
| pericles | v1/215 | [1037,1833) | (796,2974) | [249,272) |

The candidate source test `tier_b_verse_source_lines_restored` executed one
ignored test and verified two selected events / three measured lines. Its shared
verifiers checked original pixels, unique retained bands and strips, source
order, whole-line canvas coverage, historical neighboring regions, and the body
ledger against actual production crops. Pericles p214 is on canvas 41 and p215
is on canvas 42; their measured stream intervals meet in source order. Alcibiades's
new verse is on canvas 2 with its preceding and following text.

The saved canvases were inspected for printed verse indentation and complete
glyphs. The preserved source layout remains subject to the rebuilt-reader human
review required before restored status.

# Evidence and state

Fresh reports and relevant canvases are retained under
`archive/tier_b_omission_repairs_2026_10_09/installed-verse/alcibiades/` and
`installed-verse/pericles/`. Their exact report paths and SHA-256 digests are
recorded in `inner_tomi_audit_repair_source_trace.json`. Both events are
`installed`; rebuilt-reader acceptance is pending. The accumulated installation
covers 15 promoted events / 17 measured lines, with 64 production protection
records including the original 47 Tier-A records. Delivery remains `in_progress`.

The authenticated portable export adds two reports, yielding 131 documents and
908,555 bytes. All 129 previous semantic documents compare unchanged. Historical
evidence and the immutable full-work baseline are retained without modification.

Verification includes `cargo fmt --all`, `cargo fmt --all --check`, strict
all-targets Clippy, the two `known_source_body_band` tests, the source-based verse
test, the authenticated portable export, and the full ordinary `cargo test`
suite, including staged-delivery and portable-audit regressions.
