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

For targeted source review, `inventory --source-pages 0:38,0:214,0:215,0:535,1:580,1:562`
restricts the inventory to those physical source pages (source indices are zero
based). `--units` still controls the candidate ranges. The explicit page scope
is authenticated and reported; this operation does not inventory the complete
unit ranges. The same selector on `run` keeps the selected units' mapped reader
pages while comparing the explicitly requested source pages. Such a run is a
targeted control comparison and cannot certify omitted source pages.

Source grouping measures parallel baselines within local vertical bands, using
original component coordinates and a bounded rotation search. The source pixels
remain unchanged. Glyphs, punctuation, faint components and unassigned ink remain
in the inventory. Gutter and grouping estimates remain provisional until source
geometry and ownership are reviewed.

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
Applying a policy preserves one derived child per geometry identity and its
untouched crop. Revised geometry removes superseded derived children while
retaining the original parent components and unreviewed ink. Ancestor reviews
are applied before descendant reviews, so repeated review imports retain a
consistent source inventory.

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
Glyph scale is measured from corresponding anchor-word ink on each original
page, through the shared screened estimator. The page-sized measurement frame
preserves its component-height screen while excluding unrelated typography;
required reference ink remains complete and unchanged. Recorded word boxes bind
this scope to each registration cache entry. Scale, word-box agreement,
reprojection residual, inlier extent and alternate placements govern fit
validity. Each changed source/reader gap or page starts a
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

Prepare the seven disposable full-path cases in a fresh directory:

```powershell
python tools/facsimile_line_audit.py verify --prepare --out tmp/pdfs/facsimile_line_audit/runs/verification/cases-1
```

Preparation creates fourteen source/reader PDFs, reference TeX/PNG assets,
verification snapshots and `case-expectations.json` with builder-known source
boxes and reader placements. Synthetic source policy is accepted solely for
verification targets. Source OCR comes from the actual rendered pixels. The
cases include head/foot and interior losses, complete/redundant controls, a sole
complete copy removed, unique shared-page tail loss and visible PDF overpainting
of a whole line and terminal glyph. Sparse and verse subcases remain explicit.
Prepared inputs are immutable; subsequent verification writes fresh result sets.

Run component coverage against an authenticated capture, or the prepared cases:

```powershell
python tools/facsimile_line_audit.py run --snapshot <snapshot> --run-dir <fresh-run> --units <unit-ids> --stop-after coverage
python tools/facsimile_line_audit.py verify --out tmp/pdfs/facsimile_line_audit/runs/verification/cases_5_1a --stop-after coverage
```

Coverage writes `coverage-ledger.json` and hashed source/visible-reader/missing-ink
images in `coverage-evidence`. Measured references retain every original component
at both grayscale thresholds; unreviewed geometry remains unresolved. Four-pixel deficits and glyph extremity loss prevent
completeness even when almost all aggregate ink survives. Threshold disagreement
and insufficient projected resolution stay explicit. Actual PDF CropBox and
owned body-image extents bound visible support. Authenticated decoded images are
supplementary: their completeness cannot clear a visible PDF deficit.

Reviewed geometry, ownership, ordered localization and valid continuous strip
registration are prerequisites for machine completeness. Separate visible pieces
may satisfy the full reference through an ordered component union; overlapping
support is counted once and insufficient resolution cannot certify the union.
Threshold support is measured across the complete union, including piece boundaries. A complete
logical reference permits redundant source occurrences to be waived. Partial
duplicates preserve the required canonical extent.

An omitted interior gap requires strong owned context on both sides. A reviewed
unit opening/ending uses its mapped reader boundary and one following/preceding
strong context; missing prefixes/suffixes inherit that reviewed boundary. Physical
page edges inside a unit remain interior. Focused OCR at 300 DPI and bounded
grayscale/contrast image searches inspect each implicated extent. Scale and
rotation search bounds are recorded. An ambiguous recovery remains unresolved.
Sparse passages without adequate anchors also remain unresolved. `partial` and
`absent` are candidates with a separate, initially empty human verdict and an
unknown loss stage.

Verification copies the immutable builder inputs into fresh captures bound to
current executed code/settings, then runs ordinary OCR, localization and coverage.
Each case writes `case-result.json` in its new `results/<snapshot-id-prefix>`
directory. Aggregate records live in `results/<report-hash>/verification-results.json`
under the prepared root. Known losses that remain unresolved and incomplete
retained controls fail verification with exit 4. `--stop-after coverage` scores
coverage; the default full invocation includes seams, neutral tasks and reports.

The scorer checks at least 80% required-source intersection, limits extra source
line height, requires the independently expected reader page and gap/painted
extent, and checks opening/ending rules. Known partial losses require a persistent
connected deficit at both thresholds in the expected reader pixels. A generic
page warning cannot pass this geometry check. Source-page and reader-body seam
counts are checked against the independent builder placements. The sparse
manual-localization subcase is excluded from automatic recall.

Fix the real calibration panel before scoring detector revisions:

```powershell
python tools/facsimile_line_audit.py panel --snapshot <corpus-snapshot>
```

`panel.json` lives under `runs/calibration/<reader-input-identity>` unless
`--run-dir` specifies another audit directory. It contains 25 distinct body pages
per tomus, selected by sorting SHA-256(input identity, tomus, page): 200 pages in
total. Its identity uses source PDFs, reader PDFs/TeX and the unit table, excluding
code, settings and source-policy versions. Repeating the command for those same
inputs requires the same saved panel. Six restored source-page controls and the
numeric partial-duplicate opening pages are listed and counted separately; their
presence in the panel does not approve ownership or equivalence. Previously
observed pages and post-revision results remain regression evidence.

Keep all earlier predictions and crops. Record misses, false alarms, processing
failures and uncertainty against their actual page and required-region
denominators. Freeze calibration identities after the seven cases and reviewed
retained controls pass; pending ownership, equivalence and weak fits remain
explicit uncertainty and cannot be promoted by a successful process exit.

Generate a full audit or regenerate derived reports from an authenticated run:

```powershell
python tools/facsimile_line_audit.py run --snapshot <snapshot> --run-dir <fresh-run> --units <unit-ids>
python tools/facsimile_line_audit.py report --run-dir <completed-run>
python tools/facsimile_line_audit.py verify --out tmp/pdfs/facsimile_line_audit/runs/verification/cases_5_1a
```

Full runs write `seam-inventory.json`, `review-tasks.json`, `coverage-report.html`,
`report-manifest.json` and `loss-review-template.json`. The seam inventory includes
every adjacent physical source page, unit opening/ending/transition, potential
shared-page boundary, reviewed duplicate bypass and adjacent reader body
occurrence, including apparently complete anchors. Physical duplicate copies
remain visible; unresolved or partial logical coverage cannot waive them.
Three owned body lines supply context on each side, with uncertain ink and
annotations retained separately. Sparse context stays within its unit. Source
ownership uncertainty and complete anchors have distinct fields.

Each overlapping neutral task contains at most six region IDs and images whose
longest side is at most 1,400 pixels. Untouched crop hashes and resize transforms
are retained alongside labeled display copies. Reader evidence uses the same
visible PDF raster as coverage, including both reader-page sides and mapped unit
endpoint overviews. Unlocalized regions stay explicit and inconclusive. Machine
verdicts and repair labels are excluded from task payloads. Reports show scope,
source/reader exclusions, uncertainty denominators and all evidence links.

For human loss review, copy `loss-review-template.json` to a fresh file, retain
the identity and crop-hash fields, select the decisions being reviewed, and fill
in `verdict` (`confirmed_loss`, `retained` or `uncertain`), `reviewer` and `reason`:

```powershell
python tools/facsimile_line_audit.py adjudicate --run-dir <completed-run> --loss-reviews <filled-template.json> --reviews-out <new-review-version.json>
```

Review import authenticates snapshot/runtime, ledger, task set and every bound
source/visible-reader crop. It rejects stale evidence, duplicate decisions and
loss confirmations without reader evidence. Outputs are separately versioned;
machine verdicts and source policy remain immutable. `report` requires executed
code to match the capture and reproduces derived views for the same ledger.

The optional reviewer uses the separately pinned official Python SDK:

```powershell
python -m pip install -r tools/requirements-facsimile-seam-review.txt
python tools/facsimile_seam_review.py --tasks <run>/review-tasks.json --out <run>/haiku-observations --max-calls 10 --dry-run
python tools/facsimile_seam_review.py --tasks <run>/review-tasks.json --out <run>/haiku-observations --offline --resume
```

SDK 0.84.0, the `claude-haiku-4-5` default, zero SDK retries, a 120-second timeout
and 2,000 output tokens are explicit. SDK loading and repository-root
`secrets.json` access occur when the first live submission is authorized by a
positive `--max-calls`. Credentials use the existing `anthropic_api_key` field;
they are never recorded or hashed. Dry-run/offline modes load neither the SDK
client nor the key and submit nothing. Live calls require a positive cap and use
the same command without `--dry-run`/`--offline`. `--model` selects another model
explicitly. API, SDK and vision support were checked against the
[official Python SDK guide](https://platform.claude.com/docs/en/api/sdks/python),
[vision guide](https://platform.claude.com/docs/en/build-with-claude/vision) and
[model lifecycle table](https://platform.claude.com/docs/en/about-claude/model-deprecations).

The cap counts uncached task submissions. Completed observations retain the raw
response, requested/returned model, usage and validated line/crop identities.
`--resume` authenticates them and processes eligible pending tasks within the cap.
Fully unlocalized tasks remain inconclusive. Neutral requests show immutable
region IDs, source references/alternates and visible-reader images; machine
coverage verdicts and repair labels are excluded. Client/model results cannot
modify source policy, machine coverage or human decisions.

Request state is atomically recorded as `in_flight` before submission. An orphaned
request without a durable response becomes `delivery_unknown` and is never
silently resubmitted. A preserved valid response can be recovered locally; an
invalid delivered response remains `response_invalid`. Explicit unknown retry
requires all three flags and an unused attempt ID:

```powershell
python tools/facsimile_seam_review.py --tasks <run>/review-tasks.json --out <run>/haiku-observations --resume --max-calls 1 --task-ids <task-id> --retry-unknown --attempt-id retry-1
```

The prior attempt and its state remain archived. Retry selections are checked
before live delivery, and inspection of retry arguments submits nothing. SDK
errors record their type/status without credentials or sensitive exception text.
Saved JSON can exercise the response validator without a live request:

```powershell
python tools/facsimile_seam_review.py --tasks <run>/review-tasks.json --out <run>/offline-replay --offline --task-ids <single-task-id> --replay-response <saved-response.json>
```

Replay output is labeled `replay` and never becomes a live completed observation.
The response must contain the exact `task_id` and one entry per requested ID in
`lines`, each with `line_id`, `verdict`, `object_type`, `reader_crop_ids` and
`reason`. Missing/unknown IDs and unrelated support are rejected. Regions without
localized reader extent require `unclear`. Reviewer exits are 0 for completed
requested inspection/capped work, 2 for invalid inputs, and 3 for interrupted or
failed delivery/response validation. `review-summary.json` keeps pending, cached,
unknown and inconclusive counts separate from submitted requests.

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
processing, and 4 for failed explicitly requested calibration. Optional trace
stages report unavailability until implemented.

## Scope verification and evidence archives

Two real horizontal-split acceptance controls are fixed in
`facsimile-real-acceptance-controls.json`, separately from synthetic cases and
the random panel. They bind intact source rows to Tomus I pages 212–213 and
Tomus II pages 20–21 in the current input PDFs. Their success criterion is an
explicit glyph-cut finding at both reader edges. Generic uncertainty or combined
ink completeness without that warning does not pass. Header leakage is outside
this source-Latin coverage task.

```powershell
python tools/facsimile_line_delivery.py controls --run-dir <run-with-authenticated-capture> --out tmp/pdfs/facsimile_line_audit/runs/real-controls.json
```

This mode independently measures the fixed source boxes against visible reader
edges, fitting intact neighboring words with the shared strict registration
primitive. A shared glyph-cut predicate requires supported upper/lower portions
of the same source components on consecutive pages at both captured thresholds.
It records its executed code, fixed-control hash, input/render identities,
transforms, component measurements and source/reader crops. Output JSON, HTML
and evidence directories must be fresh. Component boxes are relative to the
required source crop; each control's source box uses full-source coordinates.
`--controls <file>` selects an explicitly
versioned alternative control set; changed source/reader identities are rejected.

The two fixed real controls pass this targeted mode; two intact neighboring rows
produce no glyph-cut warnings. Targeted measurement supplies source geometry and
page pairs; it does not establish automatic full-corpus localization. The
comparison run may still be incomplete, which is recorded separately. This mode
preserves the machine ledger and existing comparison caches. Header, source text
and production repair policy are unchanged.

The companion general seam command selects reviewed source regions and adjacent
page pairs from a completed run's actual correspondence neighborhoods, without
using fixed control page pairs:

```powershell
python tools/facsimile_line_delivery.py glyph-cuts --run-dir <completed-run> --out tmp/pdfs/facsimile_line_audit/runs/general-glyph-cuts.json
```

It applies the same registration and glyph-cut functions as control mode and
writes separate JSON/HTML evidence. Completed seam runs also attach the matching
reader-body seam IDs. Machine ink status remains alongside the supplemental
glyph observation; a complete ink status cannot hide a detected glyph cut in
this companion report. The command authenticates comparison records and records
its own executed code. No comparison records or cached verdicts are overwritten.

The report counts reviewed source regions, eligible page-pair measurements,
unresolved registrations and regions outside this reviewed scope. Unreviewed
geometry, unavailable two-page neighborhoods and failed fits remain uncertified.
No glyph-cut finding is not a completeness certificate. The two fixed controls
remain targeted validation; automatic corpus localization of those same defects
is a separate check when their source geometry and neighborhoods are available.

Scope and archive modes read authenticated completed comparison records.
The delivery tool records its own code identity and the captured comparison
identities; it does not participate in comparison caches. Scope reporting independently counts
requested unit/source ranges and reader occurrences, checks actual processing
counts, authenticates source/measurement/task crops and verifies report links.
Large source-page and correspondence arrays are read incrementally.

```powershell
python tools/facsimile_line_delivery.py scope --run-dir <completed-run> --panel <fixed-panel-directory>/panel.json --out tmp/pdfs/facsimile_line_audit/runs/full-scope.json
```

The output requires a fresh filename. An optional panel must already have been
fixed before detector scoring; delivery cannot create a retrospective panel.
It records associated source-region observations for each of the 200 pages,
unreviewed source counts and unlocalized uncertainty. Ground-truth misses and
false alarms remain null until independently reviewed; OCR of a panel page is
distinct from source coverage or independent panel validation. Calibration
approval and complete Latin certification remain explicitly false in this
diagnostic report.

Archive an authenticated run and its capture into a fresh direct child of the
work directory's snapshots folder:

```powershell
python tools/facsimile_line_delivery.py archive --run-dir <completed-run> --archive-dir tmp/pdfs/facsimile_line_audit/snapshots/archive_2026_10_10 --case-root tmp/pdfs/facsimile_line_audit/runs/verification/cases_5_1a --calibration-dir tmp/pdfs/facsimile_line_audit/runs/calibration
python tools/facsimile_line_delivery.py verify-archive --archive-dir tmp/pdfs/facsimile_line_audit/snapshots/archive_2026_10_10
```

The optional supplementary directories must be existing audit-run directories.
The archive copies original bytes, including source/reader PDFs, into its exact
fresh snapshot root; it creates no new PDF content. Source and copy digests are
checked before the final manifest makes the archive read-only under the shared
output policy. Every archived file's relative path, byte count and SHA-256 is
recorded. Verification checks the exact file inventory and all hashes. Existing
archives are never overwritten. Reports keep their relative evidence links;
operational paths in captured records retain their historical provenance.

Delivery exits are 0 for successful requested verification/copying, 2 for
invalid, incomplete or changed evidence, and 4 for unmet real-control criteria.
Preserve incomplete diagnostic runs and
all earlier predictions. The archive's identities authenticate evidence without
approving calibration, source ownership or proposed repairs.
