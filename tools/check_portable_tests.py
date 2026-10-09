"""Run the ordinary Rust suite in an evidence-free snapshot of tracked files."""

from __future__ import annotations

import argparse
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import tempfile
from typing import Callable, TextIO


PORTABLE_FIXTURE = "src/facsimile/fixtures/inner_tomi_portable_audit_evidence.json"
TEST_PREFIX = "facsimile::reflow::tests::"
REQUIRED_TESTS = (
    "audit_omission_inventory_matches_frozen_review",
    "audit_omission_inventory_rejects_false_positive_and_duplicate_events",
    "audit_omission_diagnosis_covers_all_source_lines",
    "audit_omission_post_dedup_covers_all_source_lines",
    "audit_omission_source_signatures_match_proposed_repairs",
    "audit_tier_a_completion_requires_all_lines",
    "portable_audit_evidence_schema_and_size_are_valid",
    "portable_audit_evidence_has_every_required_document",
    "portable_evidence_document_rejects_missing_or_mismatched_reference",
    "portable_audit_projection_is_deterministic",
    "tier_b_baseline_manifest_rejects_missing_units_and_configuration_mismatch",
    "tier_b_restoration_selection_rejects_missing_events_and_wrong_counts",
    "reviewed_band_ledger_uses_production_crop_bounds",
    "reviewed_band_ledger_respects_shared_page_ownership",
    "tier_b_crop_edges_preserve_reviewed_head_glyphs",
    "tier_b_crop_edges_preserve_reviewed_foot_glyphs",
    "known_source_body_band_groups_preserve_subranges_and_parent_semantics",
    "tier_b_validation_preserves_historical_tier_a_evidence",
    "tier_b_restoration_projection_requires_coverage_fields",
    "tier_b_coverage_rejects_missing_wrong_region_and_invalid_owners",
    "tier_b_coverage_rejects_duplicate_and_split_lines",
    "tier_b_coverage_preserves_reviewed_duplicate_page_exclusions",
    "tier_b_delivery_rejects_missing_unresolved_evidence",
    "tier_b_disposition_rejects_wrong_event_and_digest_mismatch",
    "tier_b_promotions_preserve_initial_inventory",
    "tier_b_delivery_fixture_is_consistent",
    "tier_b_delivery_accepts_partial_and_complete",
    "tier_b_delivery_rejects_missing_lines_and_evidence",
    "tier_b_delivery_rejects_unresolved_complete_claim",
)
EVIDENCE_PROBE = TEST_PREFIX + "audit_evidence_historical_inventory_authenticated"
EXCLUDED_ROOTS = {"tmp", "archive", "target", "debug", "build_facsimile", ".git", ".aws", ".ssh"}
EXCLUDED_CACHE_DIRS = {"render_cache", "raster_cache", "rendered_cache", "__pycache__"}
EXCLUDED_SUFFIXES = {".pdf", ".png", ".jpg", ".jpeg", ".tif", ".tiff", ".bmp", ".zip", ".pem", ".key", ".p12", ".pfx"}
SECRET_VARIABLE = re.compile(r"API.?KEY|(?:^|_)KEY(?:_|$)|TOKEN|SECRET|PASSWORD|CREDENTIAL|PRIVATE.?KEY", re.I)
SECRET_PATH_VARIABLES = {"SSH_AUTH_SOCK", "SSH_AGENT_PID", "AWS_CONFIG_FILE", "AZURE_CONFIG_DIR"}
Runner = Callable[..., subprocess.CompletedProcess]


class PortableCheckError(RuntimeError):
    """A snapshot or required verification command did not meet its contract."""


def tracked_paths(repository: Path, runner: Runner = subprocess.run) -> list[str]:
    result = runner(
        ["git", "ls-files", "--cached", "-z"], cwd=repository,
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
    )
    if result.returncode:
        detail = result.stderr.decode("utf-8", errors="replace")
        raise PortableCheckError(f"git ls-files failed: {detail}")
    return [os.fsdecode(path) for path in result.stdout.split(b"\0") if path]


def snapshot_relative_path(path: str) -> PurePosixPath:
    normalized = path.replace("\\", "/")
    parts = normalized.split("/")
    if (
        not normalized or normalized.startswith("/") or ":" in normalized
        or any(part in {"", ".", ".."} for part in parts)
    ):
        raise PortableCheckError(f"Unsafe tracked path: {path!r}")
    return PurePosixPath(normalized)


def excluded_snapshot_path(path: PurePosixPath) -> bool:
    parts = tuple(part.lower() for part in path.parts)
    name = parts[-1]
    return (
        parts[0] in EXCLUDED_ROOTS
        or any(part in EXCLUDED_CACHE_DIRS for part in parts[:-1])
        or path.suffix.lower() in EXCLUDED_SUFFIXES
        or name == ".env" or name.startswith(".env.")
        or (name.startswith(("credentials", "secrets."))
            and path.suffix.lower() in {".json", ".toml", ".yaml", ".yml", ".env", ".txt"})
    )


def copy_tracked_snapshot(repository: Path, snapshot: Path, paths: list[str]) -> int:
    if PORTABLE_FIXTURE not in paths:
        raise PortableCheckError(
            f"Portable fixture is not Git-tracked: {PORTABLE_FIXTURE}. "
            "Stage the intended new fixture with git add before running this check."
        )
    repository = repository.resolve(strict=True)
    snapshot.mkdir(parents=True, exist_ok=False)
    snapshot = snapshot.resolve(strict=True)
    if snapshot == repository or repository.is_relative_to(snapshot):
        raise PortableCheckError("Snapshot must not contain the original checkout")
    copied = 0
    for logical in paths:
        relative = snapshot_relative_path(logical)
        if excluded_snapshot_path(relative):
            continue
        source = repository.joinpath(*relative.parts)
        try:
            resolved = source.resolve(strict=True)
        except OSError as error:
            raise PortableCheckError(f"Cannot read tracked working-tree file {logical}: {error}") from error
        if not resolved.is_relative_to(repository):
            raise PortableCheckError(f"Tracked link escapes checkout: {logical} -> {resolved}")
        resolved_relative = PurePosixPath(resolved.relative_to(repository).as_posix())
        if excluded_snapshot_path(resolved_relative):
            raise PortableCheckError(f"Tracked link targets excluded evidence or secrets: {logical} -> {resolved}")
        if not resolved.is_file():
            raise PortableCheckError(f"Tracked path is not a regular file: {logical}")
        destination = snapshot.joinpath(*relative.parts)
        if not destination.resolve().is_relative_to(snapshot):
            raise PortableCheckError(f"Snapshot destination escapes owned directory: {logical}")
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(resolved, destination)
        copied += 1
    return copied


def child_environment(owned: Path, inherited: dict[str, str] | None = None) -> dict[str, str]:
    environment = dict(os.environ if inherited is None else inherited)
    for key in list(environment):
        if SECRET_VARIABLE.search(key) or key.upper() in SECRET_PATH_VARIABLES:
            del environment[key]
    target = owned / "cargo-target"
    evidence = owned / "empty-evidence"
    target.mkdir()
    evidence.mkdir()
    environment["CARGO_TARGET_DIR"] = str(target.resolve())
    environment["PLUTARCH_AUDIT_EVIDENCE_ROOT"] = str(evidence.resolve())
    environment["CARGO_TERM_COLOR"] = "never"
    environment.pop("PLUTARCH_PORTABLE_FIXTURE_OUT", None)
    return environment


def logged_command(command: list[str], snapshot: Path, environment: dict[str, str],
                   log: TextIO, runner: Runner = subprocess.run) -> subprocess.CompletedProcess:
    description = " ".join(command)
    print(f"Running: {description}", flush=True)
    log.write(f"\n$ {description}\n")
    log.flush()
    result = runner(
        command, cwd=snapshot, env=environment, stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace", check=False,
    )
    log.write(result.stdout)
    log.write(f"\nExit status: {result.returncode}\n")
    log.flush()
    return result


def required_command(command: list[str], snapshot: Path, environment: dict[str, str],
                     log: TextIO, runner: Runner = subprocess.run) -> str:
    result = logged_command(command, snapshot, environment, log, runner)
    if result.returncode:
        raise PortableCheckError(f"Command failed (exit {result.returncode}): {' '.join(command)}; see the log")
    return result.stdout


def validate_evidence_probe(result: subprocess.CompletedProcess) -> None:
    output = result.stdout
    if not (
        result.returncode != 0
        and re.search(r"^running 1 test\s*$", output, re.M)
        and f"test {EVIDENCE_PROBE} ... FAILED" in output
        and re.search(r"test result: FAILED\. 0 passed; 1 failed; 0 ignored;", output)
        and "missing historical evidence: logical archive/" in output
    ):
        raise PortableCheckError(
            "Evidence probe must execute exactly one historical inventory test and fail "
            "with a missing-evidence diagnostic; unexpected success, zero tests, "
            "or an unrelated failure does not prove portability"
        )


def run_cargo_checks(snapshot: Path, environment: dict[str, str], offline: bool,
                     log: TextIO, runner: Runner = subprocess.run) -> None:
    cargo = ["cargo", "test"] + (["--offline"] if offline else [])
    listing = required_command(cargo + ["--lib", "--", "--list"], snapshot, environment, log, runner)
    listed = set(re.findall(r"^(\S+): test$", listing, re.M))
    absent = [name for name in REQUIRED_TESTS if TEST_PREFIX + name not in listed]
    if absent:
        raise PortableCheckError(f"Required ordinary tests absent from list: {', '.join(absent)}")
    output = required_command(cargo, snapshot, environment, log, runner)
    for name in REQUIRED_TESTS:
        if f"test {TEST_PREFIX}{name} ... ok" not in output:
            raise PortableCheckError(f"Required ordinary test did not execute and pass: {name}")
    for result in re.findall(r"^test result: ok\..*$", output, re.M):
        print(result, flush=True)
    probe = logged_command(
        cargo + ["--lib", EVIDENCE_PROBE, "--", "--ignored", "--exact", "--nocapture"],
        snapshot, environment, log, runner,
    )
    validate_evidence_probe(probe)
    print("Expected negative probe passed: one evidence test failed clearly because archives are absent.", flush=True)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--offline", action="store_true", help="Use an already populated Cargo dependency cache")
    parser.add_argument("--log", required=True, type=Path, help="Save complete command stdout/stderr to this file")
    args = parser.parse_args(argv)
    repository = Path(__file__).resolve().parents[1]
    try:
        paths = tracked_paths(repository)
        args.log.parent.mkdir(parents=True, exist_ok=True)
        with args.log.open("w", encoding="utf-8", newline="\n") as log:
            claim = "Tracked-files snapshot of current working-tree contents; this does not authenticate a committed revision."
            print(claim, flush=True)
            log.write(claim + "\n")
            with tempfile.TemporaryDirectory(prefix="plutarch-portable-tests-") as directory:
                owned = Path(directory).resolve(strict=True)
                snapshot = owned / "checkout"
                count = copy_tracked_snapshot(repository, snapshot, paths)
                environment = child_environment(owned)
                log.write(f"Copied {count} tracked files; evidence, scans, rendered caches, build outputs and secrets excluded.\n")
                print(f"Copied {count} tracked files into an owned temporary checkout.", flush=True)
                run_cargo_checks(snapshot, environment, args.offline, log)
        print(f"Portable ordinary suite and missing-evidence probe passed. Log: {args.log.resolve()}", flush=True)
        return 0
    except (PortableCheckError, OSError, subprocess.SubprocessError) as error:
        print(f"Portable check failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
