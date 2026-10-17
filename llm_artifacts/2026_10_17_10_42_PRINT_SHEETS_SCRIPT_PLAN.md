# Sheet-by-sheet duplex printing script

## Context

The facsimile build writes imposed printing PDFs, `build_facsimile/tomus_N.pdf`.
Each landscape page of such a PDF is one side of one physical sheet: page 1 is the
front of sheet 1, page 2 its back, page 3 the front of sheet 2, and so on. The
README tells the operator to print them at actual size, two-sided, **flip on the
short edge**, in black and white.

Sending a whole Tomus as one print job causes two problems. A jam or misfeed
partway through ruins the whole job, and the operator cannot easily print only
part of a book, such as a reprint of one damaged signature. This plan adds a
small Windows command-line tool, `tools/print_sheets.py`. It prints a chosen page
range of any PDF as a series of **separate two-page jobs, one job per sheet**,
each one duplex short-edge and monochrome. It can wait between jobs, show what it
would send without sending anything, and resume from a given sheet after a
failure.

## Current state of the world

- **Python tools convention.** Standalone tools live in `tools/` as single
  modules with a module docstring used as the `argparse` description, a
  tool-specific exception class (for example `DustCoverError` in
  `tools/dust_cover.py`), pure helper functions, and a
  `main(argv: list[str] | None = None) -> int` that catches the tool's exception,
  prints `error: ...` to stderr, and returns `2`. They end with
  `if __name__ == "__main__": sys.exit(main())`.
- **Tests convention.** Tests are `unittest` modules beside the tool
  (`tools/test_dust_cover.py`). They import with `from tools.<module> import ...`
  and use `unittest.mock.patch` to stub `subprocess` calls. Run them from the
  repository root with `python -m unittest tools.test_dust_cover`.
- **Documentation convention.** Each tool has a `tools/README-<name>.md` with a
  PowerShell example and an argument table (`| Argument | Meaning | Required |
  Default |`). The root `README.md` has a matching section for user-facing tools
  (see `## Dust covers`).
- **Platform.** The project's operator works on Windows with PowerShell (see the
  MiKTeX/`PLUTARCH_LUALATEX` instructions in `README.md`). External executables
  are located through a `PLUTARCH_*` environment variable.
- **PDF library.** PyMuPDF (`import pymupdf`) is already a dependency of the
  audit tools (`tools/requirements-inner-audit.txt`, `PyMuPDF==1.27.2`).
- **No printing code exists** in the repository.

### Decisions already made (with the user)

| Topic | Decision |
|---|---|
| Platform | Windows only. |
| Print backend | SumatraPDF's command line (details below). |
| Batch size | Exactly one sheet (two PDF pages) per print job. |
| Between batches | No prompt. An optional `--delay SECONDS` (default `0`) sleeps between jobs. |
| Page numbering | 1-based physical PDF page numbers, inclusive range. |
| Alignment | The range's **start page must be odd** (the front of a sheet); otherwise it is an error. An odd **end** page is allowed. The last job then holds one page and prints single-sided. |
| Printer | Always the Windows default printer. No `--printer` option. |
| Extras | `--dry-run`, `--resume-from SHEET`, and page-count validation against the real PDF. |

## Desired end state

- `tools/print_sheets.py` exists, and this command:

  ```powershell
  python tools/print_sheets.py build_facsimile/tomus_3.pdf --pages 33-64
  ```

  sends 16 separate print jobs to the default printer (pages 33–34, 35–36, …,
  63–64). Each job is duplex flip-on-short-edge, monochrome, at actual size. It
  prints one line per job, for example `sheet 17: pages 33-34 ... sent`.
- `--dry-run` prints the same per-sheet lines plus the exact SumatraPDF command
  for each, and sends nothing.
- `--resume-from 25` skips sheets before sheet 25 (sheet numbers are absolute:
  sheet *k* is PDF pages 2k−1 and 2k).
- `--delay 20` waits 20 seconds between consecutive jobs. It does not wait before
  the first job or after the last.
- Bad input (even start page, end before start, range past the PDF's last page,
  resume sheet outside the range, missing PDF, SumatraPDF not found, negative
  delay) gives a one-line `error: ...` on stderr and exit code `2`, **before any
  job is sent**.
- If a SumatraPDF invocation fails, the tool stops, reports the failed sheet,
  and prints the `--resume-from` value to retry with. The exit code is `1`.
- `tools/test_print_sheets.py` passes with
  `python -m unittest tools.test_print_sheets` on any OS. No real printer is
  needed, because all subprocess and sleep calls are mocked.
- `tools/README-print-sheets.md` and a `## Printing sheets` section in
  `README.md` document usage, SumatraPDF installation, and the printer-driver
  caveats below.

## Tech Learnings

- **Why SumatraPDF.** Windows has no built-in command-line way to print a PDF
  with chosen duplex and color settings. `os.startfile(path, "print")` hands the
  file to whichever app is registered for PDFs, with no control over options. The
  `win32print` API can set duplex and color in a `DEVMODE` structure, but it
  cannot render PDF pages itself. SumatraPDF is a small free PDF viewer whose
  command line does both:

  ```
  SumatraPDF.exe -print-to-default -print-settings "33-34,duplexshort,monochrome,noscale" -silent file.pdf
  ```

  `-print-settings` is a comma-separated list. A page range (`33-34`, or a single
  page `65`) limits the pages. `duplexshort` / `simplex` sets the sides.
  `monochrome` requests black and white. `noscale` prints at actual size, which
  the README requires because the trim marks must land at true scale. `-silent`
  suppresses error dialogs. With `-print-to-default`, SumatraPDF exits by itself
  once it has handed the job to the Windows spooler, so `subprocess.run` returns
  as soon as that job is queued. (Do not add `-exit-when-done`: that flag belongs
  to `-print-dialog` and is not needed here.)
- **What the exit code means.** A non-zero SumatraPDF exit code means that it
  could not open the PDF or could not spool the job. A zero exit code means only
  that the job reached the spooler, not that the sheet printed. A jam or an
  offline printer after spooling is invisible to the tool, so the operator uses
  `--resume-from` after reading the printer, not the tool output.
- **Driver caveat.** SumatraPDF passes `duplexshort` and `monochrome` to the
  printer driver through the standard Windows `DEVMODE` fields. A driver that
  ignores those fields (some do for color) will print in color or single-sided
  regardless. The tool cannot detect this, so the human verification in Phase 3
  checks it on the real printer.
- **Short-edge vs long-edge flip.** On a landscape page, "flip on short edge"
  turns the sheet like a calendar page. The back side stays upright relative to
  the front when the sheet is folded into a signature, as the imposition expects.
  Long-edge flip on these pages would print every back side upside down.
- **`subprocess.run` with an argument list.** Pass the command as a list
  (`[exe, "-print-to-default", ...]`), never as one string with `shell=True`, so
  paths with spaces need no quoting. Use `check=False` and inspect `returncode`
  yourself so the error message can name the sheet.
- **Testing with mocks.** `unittest.mock.patch("tools.print_sheets.subprocess.run")`
  replaces the function where the module looks it up. Each test can then assert
  on the exact argument lists without anything being printed. Patch
  `tools.print_sheets.time.sleep` the same way, so delay tests run instantly and
  can assert on the sleep calls.

## Implementation

All new code goes in `tools/print_sheets.py`, and all tests in
`tools/test_print_sheets.py`. Constants, names, and messages below are exact.

### Phase 1 — Pure planning logic (Suggested implementer: Sonnet)

#### Step 1.1 — Module skeleton, range parsing, and sheet batching

Create `tools/print_sheets.py` with:

- Module docstring:
  `"""Print a PDF page range one duplex sheet (two pages) per print job via SumatraPDF."""`
- Imports: `argparse`, `os`, `shutil`, `subprocess`, `sys`, `time`,
  `from dataclasses import dataclass`, `from pathlib import Path`.
- `class PrintSheetsError(Exception): """An invalid print request or failed print job."""`
- A frozen dataclass:

  ```python
  @dataclass(frozen=True)
  class Sheet:
      number: int      # absolute sheet number: pages 2*number-1 and 2*number
      first_page: int
      last_page: int   # equals first_page when the sheet holds a single page

      @property
      def single_sided(self) -> bool:
          return self.first_page == self.last_page

      def page_label(self) -> str:
          # "33-34", or "65" for a single page
  ```

- `parse_page_range(text: str) -> tuple[int, int]`. Accepts `"N"` or `"N-M"`,
  with optional surrounding whitespace and digits only. Rules, in order:
  - Malformed text → `PrintSheetsError(f"invalid page range: {text!r} (expected N or N-M)")`.
  - A page number below 1 → `PrintSheetsError("page numbers start at 1")`.
  - M < N → `PrintSheetsError(f"page range end {m} is before start {n}")`.
  - Even N → `PrintSheetsError(f"page range must start on an odd page (the front of a sheet); got {n}")`.
  - `"N"` returns `(N, N)`.
- `plan_sheets(first: int, last: int) -> list[Sheet]`. Pairs pages
  `(first, first+1), (first+2, first+3), …`. If `last` is odd, the final sheet is
  `Sheet(n, last, last)`. Sheet number = `(first_page + 1) // 2`. Assumes `first`
  is odd (already validated), and asserts it.
- `apply_resume(sheets: list[Sheet], resume_from: int | None) -> list[Sheet]`.
  `None` returns `sheets` unchanged. Otherwise it returns the sheets with
  `number >= resume_from`. If `resume_from` is not the number of any sheet in
  `sheets`, it raises `PrintSheetsError(f"--resume-from {resume_from} is outside sheets {sheets[0].number}-{sheets[-1].number}")`.

Create `tools/test_print_sheets.py` (module docstring
`"""Tests for the sheet-by-sheet print tool."""`). Its imports are
`contextlib`, `importlib.util`, `io`, `os`, `unittest`,
`from pathlib import Path`, `from tempfile import TemporaryDirectory`,
`from unittest.mock import patch`, and `from tools.print_sheets import (...)`
(the names under test), as in `tools/test_dust_cover.py`. Every test class
subclasses `unittest.TestCase`. Begin with `class RangeTests` and
`class PlanTests`:

- `test_parse_single_page`: `"5"` → `(5, 5)`.
- `test_parse_range_with_whitespace`: `" 33-64 "` → `(33, 64)`.
- `test_parse_rejects_malformed` (subTests: `""`, `"a-b"`, `"3-"`, `"3--5"`, `"3,5"`).
- `test_parse_rejects_zero`: `"0-3"`.
- `test_parse_rejects_reversed`: `"9-3"`.
- `test_parse_rejects_even_start`: `"4-9"`, and the message names `4`.
- `test_plan_even_end_pairs_all_pages`: `plan_sheets(33, 38)` →
  sheets 17, 18, 19 with pages 33–34, 35–36, 37–38.
- `test_plan_odd_end_single_final_sheet`: `plan_sheets(1, 5)`: the last sheet is
  `Sheet(3, 5, 5)` and `single_sided` is `True`.
- `test_plan_single_page`: `plan_sheets(7, 7)` → `[Sheet(4, 7, 7)]`.
- `test_page_label`: `"33-34"` and `"65"`.
- `test_resume_none_keeps_all`.
- `test_resume_skips_earlier_sheets`: resume 18 on sheets 17–19 → 18, 19.
- `test_resume_outside_range_rejected`: subTests 16 and 20 on sheets 17–19.

**LLM verification**
- `python -m unittest tools.test_print_sheets -v` → all pass.
- `python -c "import tools.print_sheets"` from the repository root → no error.

**Human verification**
- Read `plan_sheets` and the tests and confirm that the sheet numbering (sheet *k*
  = pages 2k−1, 2k) matches how you think of sheets in `tomus_N.pdf`.

#### Step 1.2 — Page-count validation

Add to `tools/print_sheets.py`:

- `pdf_page_count(path: Path) -> int`. Imports `pymupdf` **inside the function**
  (as `tools/audit_inner_tomi.py` does), so the module and the Phase 1 tests
  import without PyMuPDF. Behavior:
  - If `path` is not a file → `PrintSheetsError(f"no such PDF: {path}")`.
  - If `import pymupdf` fails → `PrintSheetsError("PyMuPDF is required to read the page count; install it with: python -m pip install PyMuPDF==1.27.2")`.
  - If opening fails → `PrintSheetsError(f"cannot read PDF {path}: {exc}")`.
  - Otherwise returns `document.page_count`, opening with
    `with pymupdf.open(path) as document:`.
- Create `tools/requirements-print-sheets.txt`, following the existing
  `tools/requirements-*.txt` files:

  ```
  # Runtime dependency of tools/print_sheets.py (page-count validation).
  PyMuPDF==1.27.2
  ```
- `check_range_in_document(last: int, page_count: int) -> None`. If
  `last > page_count`, it raises
  `PrintSheetsError(f"page range ends at {last} but the PDF has {page_count} pages")`.

Tests (`class PageCountTests`):
- `test_range_within_document_accepted`: `check_range_in_document(64, 64)` does not raise.
- `test_range_past_document_rejected`: `(65, 64)` raises, and the message names both numbers.
- `test_missing_pdf_rejected`: a nonexistent path in a `TemporaryDirectory`.
- `test_page_count_reads_real_pdf`: decorated with
  `@unittest.skipUnless(importlib.util.find_spec("pymupdf"), "PyMuPDF not installed")`.
  It writes a 3-page PDF with `pymupdf.open()` / `new_page()` / `save()` into a
  temp dir and asserts `pdf_page_count(...) == 3`.

**LLM verification**
- `python -m unittest tools.test_print_sheets -v` → all pass, and the PyMuPDF test
  either passes or is reported as skipped (not errored).

**Human verification**
- None beyond reading the error messages for clarity.

### Phase 2 — SumatraPDF backend and CLI (Suggested implementer: Sonnet)

#### Step 2.1 — Locate SumatraPDF and build the command

Add:

- `SUMATRA_ENV_VAR = "PLUTARCH_SUMATRA"`.
- `find_sumatra(explicit: Path | None) -> Path`. Search order:
  1. `explicit` (from `--sumatra`), if given.
  2. `os.environ[SUMATRA_ENV_VAR]`, if set and non-empty.
  3. `shutil.which("SumatraPDF")`.
  4. `Path(os.environ["LOCALAPPDATA"]) / "SumatraPDF" / "SumatraPDF.exe"`
     (the default per-user install), if `LOCALAPPDATA` is set.
  5. `Path(os.environ["ProgramFiles"]) / "SumatraPDF" / "SumatraPDF.exe"`, if
     `ProgramFiles` is set.

  The first candidate that is an existing file wins. For an explicit path or the
  env var, the path **must** exist: if it does not, raise
  `PrintSheetsError(f"SumatraPDF not found at {path} (from {source})")`, where
  `source` is `"--sumatra"` or `"PLUTARCH_SUMATRA"`. Do not fall through to the
  other candidates in that case, because a wrong explicit setting should be loud.
  If nothing is found, raise
  `PrintSheetsError("SumatraPDF not found; install it from https://www.sumatrapdfreader.org/ or set PLUTARCH_SUMATRA to SumatraPDF.exe")`.
- `print_settings(sheet: Sheet) -> str`. Returns
  `f"{sheet.page_label()},{sides},monochrome,noscale"`, where `sides` is
  `"simplex"` for a single-sided sheet and `"duplexshort"` otherwise.
- `build_command(sumatra: Path, pdf: Path, sheet: Sheet) -> list[str]`. Returns
  `[str(sumatra), "-print-to-default", "-print-settings", print_settings(sheet), "-silent", str(pdf)]`.

Tests (`class SumatraTests`). Use `patch.dict(os.environ, {...}, clear=True)` and
temp files standing in for `SumatraPDF.exe`:
- `test_explicit_path_wins`.
- `test_env_var_used_when_no_explicit`.
- `test_explicit_missing_is_error_not_fallthrough`: an env var points to a real file,
  but the explicit path is missing → error naming `--sumatra`.
- `test_env_var_missing_is_error`.
- `test_falls_back_to_localappdata`: patch `tools.print_sheets.shutil.which` to
  return `None`, create `<tmp>/SumatraPDF/SumatraPDF.exe` (an empty file), and set
  only `LOCALAPPDATA=<tmp>` in the cleared environment. Expect that path back.
- `test_not_found_message_mentions_env_var`.
- `test_settings_duplex_sheet`: `"33-34,duplexshort,monochrome,noscale"`.
- `test_settings_single_page_sheet`: `"65,simplex,monochrome,noscale"`.
- `test_build_command_exact`: asserts the full list.

**LLM verification**
- `python -m unittest tools.test_print_sheets -v` → all pass.

**Human verification**
- Compare `build_command` against SumatraPDF's command-line documentation
  (https://www.sumatrapdfreader.org/docs/Command-line-arguments) and confirm the
  flag spellings.

#### Step 2.2 — Job runner with delay, dry run, and failure reporting

Add:

- `run_jobs(sumatra: Path, pdf: Path, sheets: list[Sheet], delay: float, dry_run: bool) -> int`.
  For each sheet, in order:
  - `prefix = f"sheet {sheet.number}: pages {sheet.page_label()}"`, with
    `" (single-sided)"` appended when `sheet.single_sided`.
  - **Dry run:** print `prefix`, then on the next line two spaces followed by
    `subprocess.list2cmdline(command)`. Do not call `subprocess.run` or sleep.
    Return `0` at the end.
  - **Real run:** sleep first, but only for sheets after the first:
    `if index > 0 and delay > 0: time.sleep(delay)`. Then
    `subprocess.run(command, check=False, capture_output=True, text=True)`.
    - An `OSError` (for example, the exe vanished) or a non-zero `returncode`:
      print `f"{prefix} ... FAILED"` to stdout, and to stderr print
      `f"error: SumatraPDF exited with {code} for sheet {n}"`, adding stderr text
      if there is any (for `OSError`, use `f"error: cannot run SumatraPDF: {exc}"`).
      Then print
      `f"resume with: --resume-from {sheet.number}"` to stderr and return `1`
      immediately, sending no further jobs.
    - Success: print `f"{prefix} ... sent"`, flushing stdout
      (`print(..., flush=True)`) so progress shows during delays.
  - After the loop, print `f"{len(sheets)} sheet(s) sent"` on a real run and
    `f"{len(sheets)} sheet(s) planned (dry run; nothing sent)"` on a dry run.
    Return `0`.

Tests (`class RunJobsTests`). Patch `tools.print_sheets.subprocess.run` and
`tools.print_sheets.time.sleep`, and capture stdout/stderr with
`contextlib.redirect_stdout` / `redirect_stderr` into `io.StringIO`:
- `test_one_job_per_sheet_in_order`: 3 sheets → 3 `run` calls, whose
  `-print-settings` values are `33-34…`, `35-36…`, `37-38…`.
- `test_delay_between_jobs_only`: delay 5, 3 sheets → `sleep` called exactly twice, each time with `5`.
- `test_zero_delay_never_sleeps`.
- `test_dry_run_sends_nothing`: `run` and `sleep` are not called, and the output
  contains each command line and `nothing sent`.
- `test_failure_stops_and_reports_resume`: the second call returns
  `returncode=1` → only 2 calls, the return value is `1`, and stderr contains
  `--resume-from 18`.
- `test_oserror_reported_as_failure`: `run` raises `OSError` → return value `1`, with a resume hint.
- `test_single_sided_label_printed`.

**LLM verification**
- `python -m unittest tools.test_print_sheets -v` → all pass.

**Human verification**
- Read the dry-run output format in a test and confirm it is what you want to
  see on screen.

#### Step 2.3 — `main` and argument parsing

Add `main(argv: list[str] | None = None) -> int` in the `dust_cover.py` style:

| Argument | Type / default | Meaning |
|---|---|---|
| `pdf` (positional) | `Path` | The PDF to print. |
| `--pages` | `str`, **required** | `N` or `N-M`, 1-based, inclusive, with an odd start. |
| `--delay` | `float`, default `0.0` | Seconds to wait between jobs. |
| `--resume-from` | `int`, default `None` | The absolute sheet number to start from. |
| `--dry-run` | flag | Show the jobs and commands, and send nothing. |
| `--sumatra` | `Path`, default `None` | Explicit path to `SumatraPDF.exe`. |

Order of operations inside `try:` (all validation happens before any job):
1. If `args.delay < 0`, raise `PrintSheetsError("--delay must not be negative")`.
2. `first, last = parse_page_range(args.pages)`.
3. `check_range_in_document(last, pdf_page_count(args.pdf))`.
4. `sheets = apply_resume(plan_sheets(first, last), args.resume_from)`.
5. `sumatra = find_sumatra(args.sumatra)`. This runs on dry runs too, so a dry
   run proves the real command would work.
6. `return run_jobs(sumatra, args.pdf, sheets, args.delay, args.dry_run)`.

`except PrintSheetsError as exc:` prints `f"error: {exc}"` to stderr and returns
`2`. End the module with `if __name__ == "__main__": sys.exit(main())`.

Tests (`class MainTests`). Patch `pdf_page_count`, `find_sumatra`, and `run_jobs`
where needed:
- `test_main_passes_planned_sheets_to_runner`: `--pages 33-38` → `run_jobs`
  receives sheets 17–19.
- `test_main_resume_from`: `--pages 33-38 --resume-from 18` → sheets 18–19.
- `test_main_even_start_exit_2_no_jobs`: `run_jobs` is not called, and the return value is `2`.
- `test_main_range_past_end_exit_2`.
- `test_main_negative_delay_exit_2`.
- `test_main_missing_sumatra_exit_2_even_on_dry_run`.
- `test_main_returns_runner_status`: `run_jobs` returns `1` → `main` returns `1`.

**LLM verification**
- `python -m unittest tools.test_print_sheets -v` → all pass.
- `python tools/print_sheets.py --help` → shows all six arguments.
- `python tools/print_sheets.py nonexistent.pdf --pages 1-2` → `error: no such PDF: nonexistent.pdf`, exit code 2.
- `python -m unittest tools.test_dust_cover` → still passes (nothing shared, as a sanity check).

**Human verification**
- On Windows with SumatraPDF installed, run
  `python tools/print_sheets.py build_facsimile/tomus_1.pdf --pages 1-6 --dry-run`
  and confirm that the three sheet lines and commands look right.

### Phase 3 — Documentation and real-printer check (Suggested implementer: Sonnet)

#### Step 3.1 — Docs

- Create `tools/README-print-sheets.md`, following `tools/README-dust-cover.md`:
  - What it does (one job per sheet; duplex short-edge, monochrome, actual size;
    default printer).
  - Install SumatraPDF, then optionally set
    `$env:PLUTARCH_SUMATRA = "$env:LOCALAPPDATA\SumatraPDF\SumatraPDF.exe"`.
  - PyMuPDF requirement:
    `python -m pip install -r tools/requirements-print-sheets.txt`.
  - PowerShell examples: a full Tomus range, a dry run, a delay, and a resume after a jam.
  - The argument table, in the existing `| Argument | Meaning | Required | Default |` format.
  - Exit codes: `0` all jobs sent (or a dry run finished); `1` a SumatraPDF job
    failed (the message gives the `--resume-from` value); `2` invalid input or
    setup, with nothing sent.
  - "Sent" means "handed to the Windows spooler". After a jam, look at which
    sheet the printer actually finished, and resume from the next one.
  - Page and sheet numbering: sheet *k* = pages 2k−1, 2k; odd start required;
    an odd end prints the final page single-sided.
  - Caveats: set the desired printer as the Windows default before running.
    Some drivers ignore the duplex or monochrome request, so check the first
    sheet. Leave the driver's own booklet and pages-per-sheet settings off, as
    `README.md` already says.
- Add a short `## Printing sheets` section to `README.md`, directly after
  `## Dust covers`. It holds one PowerShell example and a link to
  `tools/README-print-sheets.md`.

**LLM verification**
- `grep -n "Printing sheets" README.md` → one match.
- `grep -n "resume-from" tools/README-print-sheets.md` → at least one match.
- `grep -n "Exit codes" tools/README-print-sheets.md` → one match.
- Every argument in `main` appears in the README table (compare against the
  `--help` output).

**Human verification**
- Read both docs and confirm that the steps are enough for someone setting up on a
  fresh Windows machine.

#### Step 3.2 — Real-printer acceptance (human only)

No code. On the operator's Windows machine:

1. `python tools/print_sheets.py build_facsimile/tomus_1.pdf --pages 1-2` → one
   sheet comes out with both sides printed, in black and white, and the back
   upright when flipped on the short edge. Check that the trim border measures
   21 × 16 cm (actual size, not scaled).
2. `--pages 1-5 --delay 10` → three separate jobs appear in the Windows print
   queue about 10 s apart. The third sheet is single-sided.
3. Pause the printer or take it offline mid-run and confirm the behavior.
   SumatraPDF usually still succeeds, because spooling works, so this mainly
   confirms that jobs queue separately. Then try `--resume-from` to reprint one
   sheet.

If step 1 prints in color or single-sided, the driver is ignoring SumatraPDF's
DEVMODE request. Record the printer model in `tools/README-print-sheets.md`
and, as a workaround, set that printer's own default preferences to grayscale +
short-edge duplex. Implementing a grayscale-conversion fallback is out of scope
for this plan.

**LLM verification**
- None (requires physical hardware).

**Human verification**
- Steps 1–3 above.

## Out of scope

- macOS/Linux (CUPS `lp`) support.
- Choosing a non-default printer.
- Interactive pause-for-Enter between sheets.
- Converting the PDF to grayscale before printing.
- Copies, paper trays, or paper-size selection.
