# Inner-tomi audit setup

`audit_inner_tomi.py` is an offline diagnostic tool. Its commands check
prerequisites, capture the inspected baseline, and render benchmark annotations.
OCR extraction and defect detectors belong to later implementation phases.

Use the existing Python 3.10+ interpreter. If `python` does not resolve in a
PowerShell session, use its actual executable path; no reinstall is required.

```powershell
python tools/audit_inner_tomi.py --help
python tools/audit_inner_tomi.py check
```

The check prints interpreter, package, Poppler/Tesseract versions and installed
OCR models. Explicit tool paths are available through `check --pdftoppm PATH
--tesseract PATH`. Dependencies are pinned to versions verified with the existing
Python 3.13.14 interpreter. If imports are missing, install them separately:

```powershell
python -m pip install -r tools/requirements-inner-audit.txt
```

The tracked `inner-audit-baseline-inputs.json` preserves the inspected input
identities. Prepare the working spec deliberately, then capture:

```powershell
New-Item -ItemType Directory -Force tmp/pdfs/inner_tomi_audit/spec
Copy-Item tools/inner-audit-baseline-inputs.json tmp/pdfs/inner_tomi_audit/spec/baseline-inputs.json
python tools/audit_inner_tomi.py capture --repo-root . --work-dir tmp/pdfs/inner_tomi_audit --baseline-spec tmp/pdfs/inner_tomi_audit/spec/baseline-inputs.json
```

Capture verifies originals before copying, verifies each copy, and rehashes
originals before writing the completed manifest. A mismatch stops capture; it
never substitutes a different reader. Existing snapshots, including partial
captures, are preserved. Completed snapshots cannot be overwritten by the tool.
Snapshot input paths preserve the repository-relative tree.

Ordinary diagnostic writes are permitted beneath the working `cache/` and
`runs/` directories. Resolved destinations in snapshots, specs, or live input
locations are rejected, including redirected paths. `resolve_baseline` accepts
an exact baseline ID and verifies every captured input without falling back to
live files. These protections are tool policy, not filesystem access controls
against unrelated programs.

The generated work directory is ignored by Git. Snapshot copies are local;
the branch contains the script, dependencies, pinned spec, and these instructions.
No Latin, production source, or reader is edited by capture.

The tracked `inner-audit-labels.json`, `inner-audit-splits.json`, and
`inner-audit-templates.json` preserve report-derived benchmark data. Copy these
deliberately into the working `spec/` directory as `labels.json`, `splits.json`,
and `templates.json`. Runtime annotation never edits specs. The fixed split
contains 34 development units and 34 validation units. Its 200-page panel was
selected from fresh captured-PDF image occurrences before detector scoring.

```powershell
python tools/audit_inner_tomi.py annotate --baseline baseline_2026_10_06 --labels tmp/pdfs/inner_tomi_audit/spec/labels.json --cases C1 C2 C3 --run-dir tmp/pdfs/inner_tomi_audit/runs/my-gold-review
```

Annotation saves full-resolution source/output crops, contact sheets, an HTML
review page, and a draft in the explicit run directory. It may view validation
gold evidence because it performs no detector scoring. Omit `--cases` to render
all reviewed regions. Pending records render full-page overviews labeled
`overview_not_gold`; they cannot establish localized evaluation hits. Use a new
run directory for changed annotations. Templates contain development source
logos; reader-output images are forbidden as templates.

The case inventory includes all required report groups and 68 endpoint labels.
Eleven findings have manually measured, inspected regions. Duplicate occurrence
locators and contribution endpoints are recorded, but their boxes remain
pending, as do the remaining layout, endpoint, and negative annotations. The
25 prior-mask neighborhoods have separate defect-type negatives rather than
whole-page clean claims. Pending annotations must be measured and reviewed
before final calibration; panel pages also remain unreviewed.

Renders use the full MediaBox and identity/settings-checked caches. Word-box
overlays were inspected on a source page and a generated reader footer. Source
embedded OCR boxes are visibly displaced from some glyphs; gold boxes are
measured from raster ink. All reader inventory pages had standard unrotated
MediaBox/CropBox geometry. Rotated or nonzero-origin
CropBox cases were not exercised; render metadata identifies unusual geometry
as requiring independent overlay inspection.

Exit codes are 0 for completed requested processing, 2 for configuration/tool
errors, and 3 for interruption. A successful capture makes no claim that the
readers are free of defects.
