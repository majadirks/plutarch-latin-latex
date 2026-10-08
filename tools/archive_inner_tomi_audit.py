"""Preserve and verify the frozen audit inputs before facsimile repairs."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil


ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "tmp/pdfs/inner_tomi_audit"
ARCHIVE = ROOT / "archive/inner_tomi_audit_2026_10_08"
RUN = Path("runs/step102-final-all-5")
EXPECTED = {
    "candidates.json": "03f31fc68aa8829b90cdf3462578d9978b05d01a5a9c895416333dd4f2c78144",
    "reviews.json": "339ac5a1663bf5ea577a293d1d22aaa412b567a082e66806943c110b126b57ec",
}


def digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--verify-only", action="store_true")
    args = parser.parse_args()
    if args.verify_only:
        manifest_path = ARCHIVE / "manifest.json"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        expected_paths = {item["path"] for item in manifest["files"]}
        actual_paths = {path.relative_to(ARCHIVE).as_posix() for path in ARCHIVE.rglob("*")
                        if path.is_file() and path != manifest_path}
        if expected_paths != actual_paths:
            raise RuntimeError("Archive file inventory differs from manifest")
        for item in manifest["files"]:
            if digest(ARCHIVE / item["path"]) != item["sha256"]:
                raise RuntimeError(f"Archive hash differs: {item['path']}")
        for name, expected in EXPECTED.items():
            if digest(ARCHIVE / RUN / name) != expected:
                raise RuntimeError(f"Frozen archived audit hash mismatch: {name}")
        print(json.dumps({"verified_files": len(expected_paths),
                          "manifest_sha256": digest(manifest_path),
                          "missing_evidence": manifest["missing_evidence"]}, indent=2))
        return
    for name, expected in EXPECTED.items():
        if digest(AUDIT / RUN / name) != expected:
            raise RuntimeError(f"Frozen audit hash mismatch: {name}")
    reviews = json.loads((AUDIT / RUN / "reviews.json").read_text(encoding="utf-8"))
    evidence = [item for event in reviews["events"] for item in event["evidence"]]
    folders = {RUN, Path("snapshots/baseline_2026_10_06"), Path("spec")}
    for item in evidence:
        relative = Path(item["path"])
        if relative.is_absolute() or ".." in relative.parts:
            raise RuntimeError(f"Evidence path escapes audit: {relative}")
        if relative.parts[0] != "runs":
            raise RuntimeError(f"Unexpected evidence location: {relative}")
        folders.add(Path(*relative.parts[:2]))
    gaps = []
    for folder in sorted(folders):
        source = AUDIT / folder
        if source.is_dir():
            shutil.copytree(source, ARCHIVE / folder, dirs_exist_ok=True)
        else:
            gaps.append({"path": folder.as_posix(), "reason": "missing source directory"})
    freeze = Path("runs/step101-development-revised-1/freeze.json")
    if (AUDIT / freeze).is_file():
        (ARCHIVE / freeze).parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(AUDIT / freeze, ARCHIVE / freeze)
    else:
        gaps.append({"path": freeze.as_posix(), "reason": "missing development freeze"})
    for item in evidence:
        path = ARCHIVE / item["path"]
        if not path.is_file():
            gaps.append({"path": item["path"], "reason": "missing referenced evidence"})
        elif digest(path) != item["sha256"]:
            raise RuntimeError(f"Review evidence hash mismatch: {item['path']}")
    files = []
    for path in sorted(ARCHIVE.rglob("*")):
        if path.is_file() and path != ARCHIVE / "manifest.json":
            relative = path.relative_to(ARCHIVE)
            expected = digest(AUDIT / relative)
            actual = digest(path)
            if actual != expected:
                raise RuntimeError(f"Archive copy differs: {relative}")
            files.append({"path": relative.as_posix(), "bytes": path.stat().st_size, "sha256": actual})
    manifest = {"schema_version": 1, "source": str(AUDIT.relative_to(ROOT)),
                "archive": ARCHIVE.relative_to(ROOT).as_posix(), "files": files,
                "missing_evidence": gaps}
    manifest_path = ARCHIVE / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    for item in files:
        if digest(ARCHIVE / item["path"]) != item["sha256"]:
            raise RuntimeError(f"Fresh archive verification failed: {item['path']}")
    print(json.dumps({"files": len(files), "bytes": sum(x["bytes"] for x in files),
                      "missing_evidence": gaps, "manifest_sha256": digest(manifest_path)}, indent=2))


if __name__ == "__main__":
    main()
