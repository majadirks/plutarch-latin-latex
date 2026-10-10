# Independent facsimile audit

This Python audit compares original source ink with the visible accepted reader
PDFs. It preserves source text, production inputs, and existing repair decisions.
The offline dependency set is `requirements-inner-audit.txt`; the optional model
reviewer has a separate dependency file. No production rebuild is required.

Run from the repository root:

```powershell
python tools/facsimile_line_audit.py check
python tools/facsimile_line_audit.py capture --reader-dir build_facsimile --snapshot-dir tmp/pdfs/facsimile_line_audit/snapshots/accepted_2026_10_10
```

Capture requires both source PDFs, eight reader PDFs/TeX files, the 68-unit table,
repair ledgers, settings, source policy, and executed audit code. Standalone image
assets are optional, but available assets must match decoded PDF pixels. Captures
refuse overwrite and authenticate every copied file. Later processing requires
the same code/tool/language identity; code or policy changes require a fresh capture.

`capture --reuse-snapshot <authenticated-snapshot>` may reuse raw reader geometry
and decoded image identities when the reader/asset bytes and mapper code agree.
It does not reuse logical maps, correspondence or coverage verdicts.

```powershell
python tools/facsimile_line_audit.py inventory --snapshot <snapshot> --run-dir tmp/pdfs/facsimile_line_audit/runs/source_inventory --units theseus,pericles,cato_minor,antonius
```

Inventory renders every inclusive physical source page in the selected units,
including skipped duplicate scans. Per-page checkpoints support interrupted
processing under the exact run identity. Source overlays and untouched crops
remain under the run directory. A measured gutter is a source hypothesis;
unresolved gutters, threshold disagreements and residual objects remain visible.
Selected-ink accounting is distinct from reviewed source ownership and coverage.

Source-only review sheets show the untouched page, annotated inventory and crop
links before reader outcomes. Import source decisions into a fresh policy file:

```powershell
python tools/facsimile_line_audit.py adjudicate --inventory-dir <inventory-run> --source-reviews <reviews.json> --policy-out tmp/pdfs/facsimile_line_audit/runs/source_review/policy-v2.json
```

The review JSON requires `schema_version: 1`, the inventory's `snapshot_id` and
`raw_inventory_id`, and a `decisions` array. Each decision specifies
`occurrence_id`, `crop_sha256`, `classification`, `owner` (or null), `reason` and
`reviewer`. Classifications are `body`, `heading`, `annotation`, `furniture` and
`uncertain`. Optional boolean fields are `required_geometry_reviewed`,
`source_partial`, `unit_start` and `unit_end`; an approved body region requires
one captured unit owner on the same source. Keep uncertain objects unresolved.

Measured `children` specify full-page `box_px`, classification and owner within
their original object's bounds. The original parent retains its components and
unreviewed ink. Optional `page_overrides` specify `source_index`, physical `page`,
`full_crop_sha256`, `gutter_box_px`, `latin_left_px`, reason and reviewer.
Reviews bind the source bytes and crop evidence; stale evidence is rejected.
Reviewed ownership may extend the production table's candidate page interval.

Adjudication preserves the old inventory and captured policy. Adopt the fresh
policy through a new `capture --source-policy <policy-v2.json>` and inventory run;
measured child crops are created in that new run. Verification probes that retain
uncertainty are not approvals of production source ownership or exclusions.

`duplicate-groups.json` and `duplicate-review.html` preserve all numeric duplicate
seeds and show source pairs and canonical coverage polygons. Numeric skips never
grant equivalence. Import measured duplicate reviews with:

```powershell
python tools/facsimile_line_audit.py adjudicate --inventory-dir <inventory-run> --duplicate-reviews <reviews.json> --policy-out tmp/pdfs/facsimile_line_audit/runs/duplicate_review/policy-v3.json
```

Duplicate review JSON binds the same schema, snapshot and raw-inventory IDs and
contains `duplicate_groups`. Each group specifies `owner`, `logical_sequence`,
`canonical_occurrence_id`, `canonical_crop_sha256`, `source_sha256`, reason,
reviewer and `evidence_sha256` equal to the canonical crop hash. Each `region_maps`
entry specifies occurrence/crop identity, original full-page `source_box_px`, a
2-by-3 `to_canonical` affine matrix, `covered_polygon` in canonical crop pixels,
finite `max_residual_px`, and evidence hash equal to its untouched source crop.
Canonical maps must cover the complete required reference span. Multiple maps
may describe one physical occurrence; a partial fragment cannot claim `complete`.
An unreviewed or source-partial reference stays unresolved. Import records the
physical pages so new inventories include reviewed duplicate regions outside
production candidate intervals. Production skip decisions remain unchanged.

Inventory also writes `logical-lines.json` and `logical-review.html`. Logical IDs
bind canonical reference identity and geometric maps, so a classification/verdict
change preserves them. Each region retains the whole reference box, threshold
component observations, physical maps, per-occurrence source-span fractions,
ownership review status and source-partial uncertainty. Candidate owners support
localization without becoming reviewed ownership. Physical, logical, redundant,
annotation, source-partial and unresolved counts are reported separately; a
partial alternate never shrinks the required canonical span. Source-span coverage
describes source geometry and does not certify visible reader coverage.

Run the available localization stage with:

```powershell
python tools/facsimile_line_audit.py run --snapshot <snapshot> --run-dir <fresh-run> --units theseus,pericles,cato_minor,antonius,comparatio_sertorii_et_eumenis --stop-after locate
```

This stage writes the inventory, logical regions, source-only token bundle/index,
reader map/OCR/tokens, ordered locations and local-strip correspondences. Raw OCR
and ambiguous words remain visible; tokenless objects stay in the image queue.
An empty source vocabulary is recorded as `no_source_anchors`. The existing token
matcher consumes canonical source sequences with no historical duplicate aliases.
Bounded local alignment refines reader tokens around exact anchors. Conflicting
or backward refinements remain ambiguous observations. Post-binding tokenless
counts come from the token bundle; logical records mark OCR assignment as pending.

Local image fits use the shared SIFT/RANSAC primitive with explicit pixel regions.
Scale, word-box agreement, reprojection residual, inlier extent and alternate
placements govern fit validity. Each changed source/reader gap or page starts a
separate strip. A context fit outside the line's bounded projection is marked
`changed_gap_context_only`; it cannot certify that line's ink. Source-region IDs
depend on source detector settings, independently of reader matching settings.
Completed-stage resume requires the same code, tool, language, policy and settings
identities and authenticates saved record hashes. Localization completion does
not claim that coverage or seam processing has completed.

`correspondences.json` stores token pairs and registrations in shared registries.
Each line references their IDs and retains its own projection validity/evidence
type. `expand_correspondence_line` restores the same observation shape for the
coverage evaluator. These registries preserve inlier details without repeating
them for every source object. Reviewed duplicate sequences use the earliest
physical member for ordering while retaining the complete canonical scan as the
fixed reference; a later reference cannot move a passage past unique prose.

Tool resolution uses the command option, `PLUTARCH_PDFTOPPM` or
`PLUTARCH_TESSERACT`, then PATH. English language data uses `--tessdata-dir`,
`PLUTARCH_TESSDATA_DIR`, or the directory reported by Tesseract. Missing usable
OCR is an explicit limitation; raw inventory may proceed but cannot claim
completed OCR localization. Credentials are never part of capture identity.

Ordinary output is confined to the work directory's `cache` and `runs`.
Snapshots become read-only after their manifest is committed. Source/production
inputs, credentials, and historical archives are protected. Verification builders
receive separate permission for fresh, allowlisted PDFs inside their exact owned
directory beneath `runs/verification`; ordinary processing cannot create PDFs.

Exit codes are 0 for completed requested processing (including findings and
uncertainty), 2 for invalid inputs/configuration, 3 for interrupted/incomplete
processing, and 4 for failed explicitly requested calibration. Commands for later
stages report unavailability until those stages are implemented.
