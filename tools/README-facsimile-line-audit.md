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
