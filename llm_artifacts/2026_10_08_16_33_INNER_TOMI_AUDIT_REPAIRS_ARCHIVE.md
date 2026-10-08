# Historical evidence preservation

Step 1.0 of `2026_10_08_16_00_INNER_TOMI_AUDIT_REPAIRS_PLAN.md` preserves
the original audit evidence in the repository-local, Git-ignored directory
`archive/inner_tomi_audit_2026_10_08/`. This directory is outside `tmp/` and
build outputs. No backup service or off-machine durability has been verified.

The archive includes the complete final run, every run folder referenced by
the historical reviews, `snapshots/baseline_2026_10_06/`, the development
freeze, and the working specifications. It contains 20,361 evidence files
(2,144,334,160 bytes), plus its own manifest. No referenced evidence is missing.
The baseline's nested `manifest.json` is included in the file inventory.

The manifest SHA-256 is
`9844828f4aaebaa0eae7498994792ae8ede899adefd73629bbc213c9a864f8b4`.
Each manifest entry records relative path, byte length, and SHA-256. The archive
copy was checked against the original files, and a fresh hash pass over the
complete archive matched the manifest. Historical review evidence hashes were
checked against `reviews.json`.

The archived frozen audit hashes match the implementation contract:

- `runs/step102-final-all-5/candidates.json`:
  `03f31fc68aa8829b90cdf3462578d9978b05d01a5a9c895416333dd4f2c78144`.
- `runs/step102-final-all-5/reviews.json`:
  `339ac5a1663bf5ea577a293d1d22aaa412b567a082e66806943c110b126b57ec`.

Commands completed successfully:

```powershell
python -X utf8 tools/archive_inner_tomi_audit.py
python -X utf8 tools/archive_inner_tomi_audit.py --verify-only
Get-FileHash archive/inner_tomi_audit_2026_10_08/runs/step102-final-all-5/candidates.json -Algorithm SHA256
Get-FileHash archive/inner_tomi_audit_2026_10_08/runs/step102-final-all-5/reviews.json -Algorithm SHA256
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The clean baseline passed 364 active library tests and nine binary tests;
37 library tests were ignored. No production facsimile behavior changed.
The installed repair ledger, source PDFs, render caches, and transcription
caches were not edited. Git status showed no archive content staged.

The repository, `tmp/pdfs/inner_tomi_audit`, and archive path components are
ordinary directories on local `C:\`, without junctions or symbolic links.
No network storage or OneDrive involvement was established. Commands use
the approved execution path because the sandbox execution helper fails
during setup.
