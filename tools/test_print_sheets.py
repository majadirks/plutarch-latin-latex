"""Tests for the sheet-by-sheet print tool."""

import contextlib
import importlib.util
import io
import os
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from tools.print_sheets import (
    DEFAULT_DELAY_SECONDS, PrintSheetsError, Sheet, apply_resume,
    build_command, check_range_in_document, find_sumatra, main, parse_page_range,
    pdf_page_count, plan_sheets, print_settings, run_jobs,
)


class RangeTests(unittest.TestCase):
    def test_parse_single_page(self):
        self.assertEqual(parse_page_range("5"), (5, 5))

    def test_parse_range_with_whitespace(self):
        self.assertEqual(parse_page_range(" 33-64 "), (33, 64))

    def test_parse_rejects_malformed(self):
        for text in ("", "a-b", "3-", "3--5", "3,5"):
            with self.subTest(text=text):
                with self.assertRaisesRegex(PrintSheetsError, "invalid page range"):
                    parse_page_range(text)

    def test_parse_rejects_zero(self):
        with self.assertRaisesRegex(PrintSheetsError, "page numbers start at 1"):
            parse_page_range("0-3")

    def test_parse_rejects_reversed(self):
        with self.assertRaisesRegex(PrintSheetsError, "before start"):
            parse_page_range("9-3")

    def test_parse_rejects_even_start(self):
        with self.assertRaisesRegex(PrintSheetsError, "4"):
            parse_page_range("4-9")


class PlanTests(unittest.TestCase):
    def test_plan_even_end_pairs_all_pages(self):
        self.assertEqual(
            plan_sheets(33, 38),
            [Sheet(17, 33, 34), Sheet(18, 35, 36), Sheet(19, 37, 38)],
        )

    def test_plan_odd_end_single_final_sheet(self):
        sheets = plan_sheets(1, 5)
        self.assertEqual(sheets[-1], Sheet(3, 5, 5))
        self.assertTrue(sheets[-1].single_sided)

    def test_plan_single_page(self):
        self.assertEqual(plan_sheets(7, 7), [Sheet(4, 7, 7)])

    def test_page_label(self):
        self.assertEqual(Sheet(17, 33, 34).page_label(), "33-34")
        self.assertEqual(Sheet(33, 65, 65).page_label(), "65")

    def test_resume_none_keeps_all(self):
        sheets = plan_sheets(33, 38)
        self.assertEqual(apply_resume(sheets, None), sheets)

    def test_resume_skips_earlier_sheets(self):
        sheets = plan_sheets(33, 38)
        self.assertEqual(
            apply_resume(sheets, 18), [Sheet(18, 35, 36), Sheet(19, 37, 38)]
        )

    def test_resume_outside_range_rejected(self):
        sheets = plan_sheets(33, 38)
        for resume in (16, 20):
            with self.subTest(resume=resume):
                with self.assertRaisesRegex(PrintSheetsError, "outside sheets 17-19"):
                    apply_resume(sheets, resume)


class PageCountTests(unittest.TestCase):
    def test_range_within_document_accepted(self):
        self.assertIsNone(check_range_in_document(64, 64))

    def test_range_past_document_rejected(self):
        with self.assertRaisesRegex(PrintSheetsError, "65.*64"):
            check_range_in_document(65, 64)

    def test_missing_pdf_rejected(self):
        with TemporaryDirectory() as tmp:
            missing = Path(tmp) / "nope.pdf"
            with self.assertRaisesRegex(PrintSheetsError, "no such PDF"):
                pdf_page_count(missing)

    @unittest.skipUnless(
        importlib.util.find_spec("pymupdf"), "PyMuPDF not installed"
    )
    def test_page_count_reads_real_pdf(self):
        import pymupdf
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "three.pdf"
            with pymupdf.open() as document:
                for _ in range(3):
                    document.new_page()
                document.save(path)
            self.assertEqual(pdf_page_count(path), 3)


class SumatraTests(unittest.TestCase):
    def test_explicit_path_wins(self):
        with TemporaryDirectory() as tmp:
            exe = Path(tmp) / "SumatraPDF.exe"
            exe.write_text("", encoding="utf-8")
            with patch.dict(os.environ, {}, clear=True):
                self.assertEqual(find_sumatra(exe), exe)

    def test_env_var_used_when_no_explicit(self):
        with TemporaryDirectory() as tmp:
            exe = Path(tmp) / "SumatraPDF.exe"
            exe.write_text("", encoding="utf-8")
            with patch.dict(os.environ, {"PLUTARCH_SUMATRA": str(exe)}, clear=True):
                self.assertEqual(find_sumatra(None), exe)

    def test_explicit_missing_is_error_not_fallthrough(self):
        with TemporaryDirectory() as tmp:
            real = Path(tmp) / "SumatraPDF.exe"
            real.write_text("", encoding="utf-8")
            missing = Path(tmp) / "nope.exe"
            with patch.dict(os.environ, {"PLUTARCH_SUMATRA": str(real)}, clear=True):
                with self.assertRaisesRegex(PrintSheetsError, "--sumatra"):
                    find_sumatra(missing)

    def test_env_var_missing_is_error(self):
        with TemporaryDirectory() as tmp:
            missing = Path(tmp) / "nope.exe"
            with patch.dict(os.environ, {"PLUTARCH_SUMATRA": str(missing)}, clear=True):
                with self.assertRaisesRegex(PrintSheetsError, "PLUTARCH_SUMATRA"):
                    find_sumatra(None)

    def test_falls_back_to_localappdata(self):
        with TemporaryDirectory() as tmp:
            exe = Path(tmp) / "SumatraPDF" / "SumatraPDF.exe"
            exe.parent.mkdir(parents=True)
            exe.write_text("", encoding="utf-8")
            with patch.dict(os.environ, {"LOCALAPPDATA": tmp}, clear=True):
                with patch("tools.print_sheets.shutil.which", return_value=None):
                    self.assertEqual(find_sumatra(None), exe)

    def test_not_found_message_mentions_env_var(self):
        with patch.dict(os.environ, {}, clear=True):
            with patch("tools.print_sheets.shutil.which", return_value=None):
                with self.assertRaisesRegex(PrintSheetsError, "PLUTARCH_SUMATRA"):
                    find_sumatra(None)

    def test_settings_duplex_sheet(self):
        self.assertEqual(
            print_settings(Sheet(17, 33, 34)), "33-34,duplexshort,monochrome,noscale"
        )

    def test_settings_single_page_sheet(self):
        self.assertEqual(
            print_settings(Sheet(33, 65, 65)), "65,simplex,monochrome,noscale"
        )

    def test_build_command_exact(self):
        self.assertEqual(
            build_command(Path("Sumatra.exe"), Path("book.pdf"), Sheet(17, 33, 34)),
            [
                "Sumatra.exe", "-print-to-default", "-print-settings",
                "33-34,duplexshort,monochrome,noscale", "-silent", "book.pdf",
            ],
        )


class RunJobsTests(unittest.TestCase):
    def _run(self, sheets, delay=0.0, dry_run=False, run_side_effect=None,
             run_return=None):
        out, err = io.StringIO(), io.StringIO()
        with patch("tools.print_sheets.subprocess.run") as run, \
                patch("tools.print_sheets.time.sleep") as sleep:
            if run_side_effect is not None:
                run.side_effect = run_side_effect
            elif run_return is not None:
                run.return_value = run_return
            else:
                run.return_value = _completed(0)
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                status = run_jobs(
                    Path("Sumatra.exe"), Path("book.pdf"), sheets, delay, dry_run
                )
        return status, out.getvalue(), err.getvalue(), run, sleep

    def test_one_job_per_sheet_in_order(self):
        sheets = plan_sheets(33, 38)
        status, _, _, run, _ = self._run(sheets)
        self.assertEqual(status, 0)
        self.assertEqual(run.call_count, 3)
        settings = [call.args[0][3] for call in run.call_args_list]
        self.assertEqual(settings, [
            "33-34,duplexshort,monochrome,noscale",
            "35-36,duplexshort,monochrome,noscale",
            "37-38,duplexshort,monochrome,noscale",
        ])

    def test_delay_between_jobs_only(self):
        sheets = plan_sheets(33, 38)
        _, _, _, _, sleep = self._run(sheets, delay=5)
        self.assertEqual(sleep.call_count, 2)
        for call in sleep.call_args_list:
            self.assertEqual(call.args[0], 5)

    def test_zero_delay_never_sleeps(self):
        sheets = plan_sheets(33, 38)
        _, _, _, _, sleep = self._run(sheets, delay=0)
        sleep.assert_not_called()

    def test_dry_run_sends_nothing(self):
        sheets = plan_sheets(33, 38)
        status, out, _, run, sleep = self._run(sheets, dry_run=True)
        self.assertEqual(status, 0)
        run.assert_not_called()
        sleep.assert_not_called()
        self.assertIn("33-34,duplexshort,monochrome,noscale", out)
        self.assertIn("nothing sent", out)

    def test_failure_stops_and_reports_resume(self):
        sheets = plan_sheets(33, 38)
        status, _, err, run, _ = self._run(
            sheets, run_side_effect=[_completed(0), _completed(1), _completed(0)]
        )
        self.assertEqual(status, 1)
        self.assertEqual(run.call_count, 2)
        self.assertIn("--resume-from 18", err)

    def test_oserror_reported_as_failure(self):
        sheets = plan_sheets(33, 38)
        status, _, err, _, _ = self._run(
            sheets, run_side_effect=OSError("boom")
        )
        self.assertEqual(status, 1)
        self.assertIn("--resume-from 17", err)

    def test_single_sided_label_printed(self):
        sheets = plan_sheets(1, 5)
        _, out, _, _, _ = self._run(sheets)
        self.assertIn("(single-sided)", out)


class MainTests(unittest.TestCase):
    def _main(self, argv):
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            status = main(argv)
        return status, out.getvalue(), err.getvalue()

    def test_main_passes_planned_sheets_to_runner(self):
        with patch("tools.print_sheets.pdf_page_count", return_value=100), \
                patch("tools.print_sheets.find_sumatra", return_value=Path("S.exe")), \
                patch("tools.print_sheets.run_jobs", return_value=0) as runner:
            self._main(["book.pdf", "--pages", "33-38"])
        sheets = runner.call_args.args[2]
        self.assertEqual([s.number for s in sheets], [17, 18, 19])

    def test_main_resume_from(self):
        with patch("tools.print_sheets.pdf_page_count", return_value=100), \
                patch("tools.print_sheets.find_sumatra", return_value=Path("S.exe")), \
                patch("tools.print_sheets.run_jobs", return_value=0) as runner:
            self._main(["book.pdf", "--pages", "33-38", "--resume-from", "18"])
        sheets = runner.call_args.args[2]
        self.assertEqual([s.number for s in sheets], [18, 19])

    def test_main_even_start_exit_2_no_jobs(self):
        with patch("tools.print_sheets.pdf_page_count", return_value=100), \
                patch("tools.print_sheets.find_sumatra", return_value=Path("S.exe")), \
                patch("tools.print_sheets.run_jobs", return_value=0) as runner:
            status, _, err = self._main(["book.pdf", "--pages", "4-9"])
        self.assertEqual(status, 2)
        runner.assert_not_called()
        self.assertIn("error:", err)

    def test_main_range_past_end_exit_2(self):
        with patch("tools.print_sheets.pdf_page_count", return_value=10), \
                patch("tools.print_sheets.run_jobs", return_value=0) as runner:
            status, _, _ = self._main(["book.pdf", "--pages", "1-20"])
        self.assertEqual(status, 2)
        runner.assert_not_called()

    def test_main_negative_delay_exit_2(self):
        with patch("tools.print_sheets.pdf_page_count", return_value=100), \
                patch("tools.print_sheets.run_jobs", return_value=0) as runner:
            status, _, _ = self._main(["book.pdf", "--pages", "1-2", "--delay", "-1"])
        self.assertEqual(status, 2)
        runner.assert_not_called()

    def test_main_default_delay_is_two_seconds(self):
        with patch("tools.print_sheets.pdf_page_count", return_value=100), \
                patch("tools.print_sheets.find_sumatra", return_value=Path("S.exe")), \
                patch("tools.print_sheets.run_jobs", return_value=0) as runner:
            self._main(["book.pdf", "--pages", "1-2"])
        self.assertEqual(runner.call_args.args[3], DEFAULT_DELAY_SECONDS)
        self.assertEqual(runner.call_args.args[3], 2.0)

    def test_main_delay_zero_accepted(self):
        with patch("tools.print_sheets.pdf_page_count", return_value=100), \
                patch("tools.print_sheets.find_sumatra", return_value=Path("S.exe")), \
                patch("tools.print_sheets.run_jobs", return_value=0) as runner:
            status, _, _ = self._main(["book.pdf", "--pages", "1-2", "--delay", "0"])
        self.assertEqual(status, 0)
        self.assertEqual(runner.call_args.args[3], 0.0)

    def test_main_missing_sumatra_exit_2_even_on_dry_run(self):
        with patch("tools.print_sheets.pdf_page_count", return_value=100), \
                patch("tools.print_sheets.find_sumatra",
                      side_effect=PrintSheetsError("SumatraPDF not found")), \
                patch("tools.print_sheets.run_jobs", return_value=0) as runner:
            status, _, err = self._main(["book.pdf", "--pages", "1-2", "--dry-run"])
        self.assertEqual(status, 2)
        runner.assert_not_called()
        self.assertIn("SumatraPDF not found", err)

    def test_main_returns_runner_status(self):
        with patch("tools.print_sheets.pdf_page_count", return_value=100), \
                patch("tools.print_sheets.find_sumatra", return_value=Path("S.exe")), \
                patch("tools.print_sheets.run_jobs", return_value=1):
            status, _, _ = self._main(["book.pdf", "--pages", "1-2"])
        self.assertEqual(status, 1)


def _completed(returncode, stderr=""):
    from subprocess import CompletedProcess
    return CompletedProcess(args=[], returncode=returncode, stdout="", stderr=stderr)


if __name__ == "__main__":
    unittest.main()
