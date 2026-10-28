"""Print a PDF page range one duplex sheet (two pages) per print job via SumatraPDF."""

import argparse
import os
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path


DEFAULT_DELAY_SECONDS = 2.0
SUMATRA_ENV_VAR = "PLUTARCH_SUMATRA"


class PrintSheetsError(Exception):
    """An invalid print request or failed print job."""


@dataclass(frozen=True)
class Sheet:
    number: int      # absolute sheet number: pages 2*number-1 and 2*number
    first_page: int
    last_page: int   # equals first_page when the sheet holds a single page

    @property
    def single_sided(self) -> bool:
        return self.first_page == self.last_page

    def page_label(self) -> str:
        if self.single_sided:
            return f"{self.first_page}"
        return f"{self.first_page}-{self.last_page}"


def parse_page_range(text: str) -> tuple[int, int]:
    stripped = text.strip()
    parts = stripped.split("-")
    if len(parts) == 1:
        token = parts[0]
        if not token.isdigit():
            raise PrintSheetsError(f"invalid page range: {text!r} (expected N or N-M)")
        n = int(token)
        m = n
    elif len(parts) == 2:
        left, right = parts
        if not left.isdigit() or not right.isdigit():
            raise PrintSheetsError(f"invalid page range: {text!r} (expected N or N-M)")
        n = int(left)
        m = int(right)
    else:
        raise PrintSheetsError(f"invalid page range: {text!r} (expected N or N-M)")
    if n < 1 or m < 1:
        raise PrintSheetsError("page numbers start at 1")
    if m < n:
        raise PrintSheetsError(f"page range end {m} is before start {n}")
    if n % 2 == 0:
        raise PrintSheetsError(
            f"page range must start on an odd page (the front of a sheet); got {n}"
        )
    return (n, m)


def plan_sheets(first: int, last: int) -> list[Sheet]:
    assert first % 2 == 1, "plan_sheets requires an odd first page"
    sheets = []
    page = first
    while page <= last:
        number = (page + 1) // 2
        if page + 1 <= last:
            sheets.append(Sheet(number, page, page + 1))
        else:
            sheets.append(Sheet(number, page, page))
        page += 2
    return sheets


def apply_resume(sheets: list[Sheet], resume_from: int | None) -> list[Sheet]:
    if resume_from is None:
        return sheets
    kept = [sheet for sheet in sheets if sheet.number >= resume_from]
    numbers = {sheet.number for sheet in sheets}
    if resume_from not in numbers:
        raise PrintSheetsError(
            f"--resume-from {resume_from} is outside sheets "
            f"{sheets[0].number}-{sheets[-1].number}"
        )
    return kept


def pdf_page_count(path: Path) -> int:
    if not path.is_file():
        raise PrintSheetsError(f"no such PDF: {path}")
    try:
        import pymupdf
    except ImportError as exc:
        raise PrintSheetsError(
            "PyMuPDF is required to read the page count; install it with: "
            "python -m pip install -r tools/requirements-print-sheets.txt"
        ) from exc
    try:
        with pymupdf.open(path) as document:
            return document.page_count
    except Exception as exc:
        raise PrintSheetsError(f"cannot read PDF {path}: {exc}") from exc


def check_range_in_document(last: int, page_count: int) -> None:
    if last > page_count:
        raise PrintSheetsError(
            f"page range ends at {last} but the PDF has {page_count} pages"
        )


def find_sumatra(explicit: Path | None) -> Path:
    if explicit is not None:
        if explicit.is_file():
            return explicit
        raise PrintSheetsError(
            f"SumatraPDF not found at {explicit} (from --sumatra)"
        )
    env_value = os.environ.get(SUMATRA_ENV_VAR)
    if env_value:
        env_path = Path(env_value)
        if env_path.is_file():
            return env_path
        raise PrintSheetsError(
            f"SumatraPDF not found at {env_path} (from {SUMATRA_ENV_VAR})"
        )
    which = shutil.which("SumatraPDF")
    if which:
        return Path(which)
    localappdata = os.environ.get("LOCALAPPDATA")
    if localappdata:
        candidate = Path(localappdata) / "SumatraPDF" / "SumatraPDF.exe"
        if candidate.is_file():
            return candidate
    program_files = os.environ.get("ProgramFiles")
    if program_files:
        candidate = Path(program_files) / "SumatraPDF" / "SumatraPDF.exe"
        if candidate.is_file():
            return candidate
    raise PrintSheetsError(
        "SumatraPDF not found; install it from "
        "https://www.sumatrapdfreader.org/ or set "
        "PLUTARCH_SUMATRA to SumatraPDF.exe"
    )


def print_settings(sheet: Sheet) -> str:
    sides = "simplex" if sheet.single_sided else "duplexshort"
    return f"{sheet.page_label()},{sides},monochrome,noscale"


def build_command(sumatra: Path, pdf: Path, sheet: Sheet) -> list[str]:
    return [
        str(sumatra),
        "-print-to-default",
        "-print-settings",
        print_settings(sheet),
        "-silent",
        str(pdf),
    ]


def run_jobs(sumatra: Path, pdf: Path, sheets: list[Sheet],
             delay: float, dry_run: bool) -> int:
    for index, sheet in enumerate(sheets):
        command = build_command(sumatra, pdf, sheet)
        prefix = f"sheet {sheet.number}: pages {sheet.page_label()}"
        if sheet.single_sided:
            prefix += " (single-sided)"
        if dry_run:
            print(prefix)
            print("  " + subprocess.list2cmdline(command))
            continue
        if index > 0 and delay > 0:
            time.sleep(delay)
        try:
            result = subprocess.run(
                command, check=False, capture_output=True, text=True
            )
        except OSError as exc:
            print(f"{prefix} ... FAILED")
            print(f"error: cannot run SumatraPDF: {exc}", file=sys.stderr)
            print(f"resume with: --resume-from {sheet.number}", file=sys.stderr)
            return 1
        if result.returncode != 0:
            print(f"{prefix} ... FAILED")
            message = (
                f"error: SumatraPDF exited with {result.returncode} "
                f"for sheet {sheet.number}"
            )
            detail = (result.stderr or "").strip()
            if detail:
                message += f": {detail}"
            print(message, file=sys.stderr)
            print(f"resume with: --resume-from {sheet.number}", file=sys.stderr)
            return 1
        print(f"{prefix} ... sent", flush=True)
    if dry_run:
        print(f"{len(sheets)} sheet(s) planned (dry run; nothing sent)")
    else:
        print(f"{len(sheets)} sheet(s) sent")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pdf", type=Path, help="The PDF to print.")
    parser.add_argument(
        "--pages", required=True,
        help="N or N-M, 1-based, inclusive, with an odd start.",
    )
    parser.add_argument(
        "--delay", type=float, default=DEFAULT_DELAY_SECONDS,
        help="Seconds to wait between consecutive jobs; 0 disables the wait.",
    )
    parser.add_argument(
        "--resume-from", type=int, default=None,
        help="The absolute sheet number to start from.",
    )
    parser.add_argument(
        "--dry-run", action="store_true",
        help="Show the jobs and commands, and send nothing.",
    )
    parser.add_argument(
        "--sumatra", type=Path, default=None,
        help="Explicit path to SumatraPDF.exe.",
    )
    args = parser.parse_args(argv)
    try:
        if args.delay < 0:
            raise PrintSheetsError("--delay must not be negative")
        first, last = parse_page_range(args.pages)
        check_range_in_document(last, pdf_page_count(args.pdf))
        sheets = apply_resume(plan_sheets(first, last), args.resume_from)
        sumatra = find_sumatra(args.sumatra)
        return run_jobs(sumatra, args.pdf, sheets, args.delay, args.dry_run)
    except PrintSheetsError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
