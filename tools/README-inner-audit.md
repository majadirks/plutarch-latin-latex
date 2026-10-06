# Inner-tomi audit setup

`audit_inner_tomi.py` is an offline diagnostic tool. Its current commands check
prerequisites and capture the inspected baseline. Detection, OCR extraction,
annotation, and evaluation commands belong to later implementation phases.

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

Exit codes are 0 for completed requested processing, 2 for configuration/tool
errors, and 3 for interruption. A successful capture makes no claim that the
readers are free of defects.
