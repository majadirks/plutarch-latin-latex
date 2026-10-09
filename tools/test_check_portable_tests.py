"""Small standard-library tests for the tracked-files verification command."""

import io
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

import check_portable_tests as portable


class PortableTests(unittest.TestCase):
    def test_required_tests_include_staged_tier_b_evidence(self):
        self.assertEqual(len(portable.REQUIRED_TESTS), len(set(portable.REQUIRED_TESTS)))
        for name in (
            "tier_b_baseline_manifest_rejects_missing_units_and_configuration_mismatch",
            "tier_b_validation_preserves_historical_tier_a_evidence",
            "tier_b_delivery_accepts_partial_and_complete",
            "tier_b_coverage_rejects_duplicate_and_split_lines",
            "tier_b_disposition_rejects_wrong_event_and_digest_mismatch",
        ):
            self.assertIn(name, portable.REQUIRED_TESTS)

    def write_file(self, repository, path, content=b"tracked"):
        file = repository / path
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_bytes(content)
        return file

    def test_snapshot_excludes_local_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            owned = Path(directory)
            repository = owned / "repository"
            repository.mkdir()
            paths = [portable.PORTABLE_FIXTURE, "src/test.rs", "src/secrets.rs", "tmp/old.json", "archive/manifest.json",
                     "target/old", "build_facsimile/old", "scan.pdf", "render_cache/old.json",
                     "secrets.json", ".env", ".cargo/credentials.toml", "old.png"]
            originals = [self.write_file(repository, path) for path in paths]

            def git_runner(command, **kwargs):
                self.assertEqual(command, ["git", "ls-files", "--cached", "-z"])
                self.assertEqual(kwargs["cwd"], repository)
                return subprocess.CompletedProcess(command, 0, b"\0".join(os.fsencode(p) for p in paths) + b"\0", b"")

            tracked = portable.tracked_paths(repository, git_runner)
            snapshot = owned / "snapshot"
            self.assertEqual(portable.copy_tracked_snapshot(repository, snapshot, tracked), 3)
            self.assertEqual({p.relative_to(snapshot).as_posix() for p in snapshot.rglob("*") if p.is_file()},
                             {portable.PORTABLE_FIXTURE, "src/test.rs", "src/secrets.rs"})
            self.assertTrue(all(file.read_bytes() == b"tracked" for file in originals))
            environment = portable.child_environment(owned, {"PATH": "keep", "ANTHROPIC_API_KEY": "secret",
                "OPENAI_API_KEY": "secret", "SECRET_PATH": "outside", "AWS_SHARED_CREDENTIALS_FILE": "outside",
                "AWS_ACCESS_KEY_ID": "secret", "KEY_PATH": "outside", "SSH_AUTH_SOCK": "outside",
                "PLUTARCH_PORTABLE_FIXTURE_OUT": "outside"})
            self.assertEqual(environment["PATH"], "keep")
            self.assertFalse(any(name in environment for name in ["ANTHROPIC_API_KEY", "OPENAI_API_KEY",
                "SECRET_PATH", "AWS_SHARED_CREDENTIALS_FILE", "AWS_ACCESS_KEY_ID", "KEY_PATH",
                "SSH_AUTH_SOCK", "PLUTARCH_PORTABLE_FIXTURE_OUT"]))
            self.assertEqual(list(Path(environment["PLUTARCH_AUDIT_EVIDENCE_ROOT"]).iterdir()), [])
            self.assertFalse(Path(environment["CARGO_TARGET_DIR"]).is_relative_to(snapshot))
            with self.assertRaisesRegex(portable.PortableCheckError, "Unsafe tracked path"):
                portable.copy_tracked_snapshot(repository, owned / "escaping", [portable.PORTABLE_FIXTURE, "../outside"])

    def test_snapshot_requires_tracked_portable_fixture(self):
        with tempfile.TemporaryDirectory() as directory:
            owned = Path(directory)
            repository = owned / "repository"
            repository.mkdir()
            self.write_file(repository, portable.PORTABLE_FIXTURE)
            with self.assertRaisesRegex(portable.PortableCheckError, "Stage the intended new fixture"):
                portable.copy_tracked_snapshot(repository, owned / "snapshot", ["src/test.rs"])
            self.assertFalse((owned / "snapshot").exists())

    def test_failed_cargo_command_is_reported(self):
        log = io.StringIO()

        def failed_runner(command, **kwargs):
            return subprocess.CompletedProcess(command, 101, "intentional build failure\n")

        with self.assertRaisesRegex(portable.PortableCheckError, "Command failed.*exit 101"):
            portable.required_command(["cargo", "test"], Path("."), {}, log, failed_runner)
        self.assertIn("intentional build failure", log.getvalue())
        self.assertIn("Exit status: 101", log.getvalue())

    def test_evidence_probe_requires_one_missing_evidence_failure(self):
        expected = ("running 1 test\nError: missing historical evidence: logical archive/test, resolved /empty/test\n"
                    f"test {portable.EVIDENCE_PROBE} ... FAILED\n"
                    "test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 451 filtered out;\n")
        portable.validate_evidence_probe(subprocess.CompletedProcess([], 101, expected))
        for status, output in [(0, expected), (101, "running 0 tests\n"),
            (101, expected.replace("missing historical evidence: logical archive/test", "hash mismatch")),
            (101, expected.replace("running 1 test", "running 2 tests")),
            (101, expected.replace(portable.EVIDENCE_PROBE, "unrelated_test"))]:
            with self.subTest(status=status, output=output):
                with self.assertRaises(portable.PortableCheckError):
                    portable.validate_evidence_probe(subprocess.CompletedProcess([], status, output))
        commands = []

        def cargo_runner(command, **kwargs):
            commands.append(command)
            if "--list" in command:
                output = "\n".join(f"{portable.TEST_PREFIX}{name}: test" for name in portable.REQUIRED_TESTS)
                return subprocess.CompletedProcess(command, 0, output)
            if "--ignored" in command:
                return subprocess.CompletedProcess(command, 101, expected)
            output = "\n".join(f"test {portable.TEST_PREFIX}{name} ... ok" for name in portable.REQUIRED_TESTS)
            return subprocess.CompletedProcess(command, 0, output)

        portable.run_cargo_checks(Path("."), {}, True, io.StringIO(), cargo_runner)
        self.assertEqual(len(commands), 3)
        self.assertEqual(commands[1], ["cargo", "test", "--offline"])
        self.assertIn(portable.EVIDENCE_PROBE, commands[2])
        self.assertIn("--exact", commands[2])
        with self.assertRaisesRegex(portable.PortableCheckError, "absent from list"):
            portable.run_cargo_checks(Path("."), {}, False, io.StringIO(),
                lambda command, **kwargs: subprocess.CompletedProcess(command, 0, "running 0 tests\n"))


if __name__ == "__main__":
    unittest.main()
