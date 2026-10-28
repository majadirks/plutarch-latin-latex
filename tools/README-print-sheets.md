# Sheet-by-sheet duplex printing

`tools/print_sheets.py` prints a chosen page range of an imposed facsimile PDF
(`build_facsimile/tomus_N.pdf`) to the Windows default printer as a series of
**separate two-page jobs, one job per physical sheet**. Each job is duplex
**flip on short edge**, monochrome, and at actual size (`noscale`), matching the
print settings the facsimile README requires. Because each sheet is its own job,
a jam or misfeed ruins only that sheet, and a single damaged signature can be
reprinted on its own with `--resume-from`.

Page 1 of the PDF is the front of sheet 1, page 2 its back, page 3 the front of
sheet 2, and so on. Sheet *k* is PDF pages 2k−1 and 2k.

## Setup

1. Install [SumatraPDF](https://www.sumatrapdfreader.org/). Windows has no
   built-in command-line way to print a PDF with chosen duplex and color
   settings; SumatraPDF's command line does both. Its default per-user install
   is `%LOCALAPPDATA%\SumatraPDF\SumatraPDF.exe`.
2. The tool finds `SumatraPDF.exe` automatically on `PATH`, in `%LOCALAPPDATA%`,
   or in `%ProgramFiles%`. To point it elsewhere, either pass `--sumatra` or set
   the environment variable once:

   ```powershell
   $env:PLUTARCH_SUMATRA = "$env:LOCALAPPDATA\SumatraPDF\SumatraPDF.exe"
   ```

3. Install PyMuPDF, used to validate the page range against the real PDF:

   ```powershell
   python -m pip install -r tools/requirements-print-sheets.txt
   ```

4. Set the printer you want as the **Windows default printer** before running.
   The tool always prints to the default printer.

## Examples

Print a whole Tomus range (16 sheets, pages 145–160), 2 s between jobs:

```powershell
python tools/print_sheets.py build_facsimile/tomus_1.pdf --pages 145-160
```

Show what would be sent, without printing anything:

```powershell
python tools/print_sheets.py build_facsimile/tomus_1.pdf --pages 145-160 --dry-run
```

Wait longer between jobs, to watch the queue:

```powershell
python tools/print_sheets.py build_facsimile/tomus_1.pdf --pages 145-160 --delay 20
```

Send the jobs back-to-back, with no wait:

```powershell
python tools/print_sheets.py build_facsimile/tomus_1.pdf --pages 145-160 --delay 0
```

Resume after a jam on sheet 80 (sheet numbers are absolute):

```powershell
python tools/print_sheets.py build_facsimile/tomus_1.pdf --pages 145-160 --resume-from 80
```

Each job prints one line, for example `sheet 73: pages 145-146 ... sent`.

| Argument | Meaning | Required | Default |
|----------|---------|----------|---------|
| `pdf` | The imposed PDF to print (positional) | Yes | — |
| `--pages` | Page range `N` or `N-M`, 1-based, inclusive, odd start | Yes | — |
| `--delay` | Seconds to wait between consecutive jobs; `0` disables the wait | No | `2` |
| `--resume-from` | Absolute sheet number to start from (skip earlier sheets) | No | None |
| `--dry-run` | Print the jobs and SumatraPDF commands, and send nothing | No | Off |
| `--sumatra` | Explicit path to `SumatraPDF.exe` | No | Auto-detected |

## Page and sheet numbering

Pages are 1-based physical PDF page numbers, and the range is inclusive. The
**start page must be odd** (the front of a sheet); an even start is an error. An
odd **end** page is allowed: the final sheet then holds a single page and prints
single-sided. Sheet *k* = pages 2k−1 and 2k, so `--resume-from 80` restarts at
PDF pages 159–160.

## Exit codes

- `0` — all jobs were sent (or a `--dry-run` finished).
- `1` — a SumatraPDF job failed. The message names the failed sheet and prints
  the `--resume-from` value to retry with.
- `2` — invalid input or setup (even start page, end before start, range past
  the PDF's last page, resume sheet outside the range, missing PDF, SumatraPDF
  not found, negative delay). **Nothing is sent** in this case; all validation
  happens before the first job.

## What "sent" means

A job is "sent" once SumatraPDF hands it to the Windows print spooler. A zero
exit code means the job reached the spooler, **not** that the sheet physically
printed. A jam or an offline printer after spooling is invisible to the tool, so
after a problem look at which sheet the printer actually finished and resume from
the next one with `--resume-from`.

## Caveats

- SumatraPDF passes `duplexshort` and `monochrome` to the printer driver through
  the standard Windows `DEVMODE` fields. Some drivers ignore those fields and
  print in color or single-sided regardless. The tool cannot detect this, so
  **check the first sheet** on the real printer. If the driver ignores the
  request, set that printer's own default preferences to grayscale + short-edge
  duplex as a workaround, and record the printer model here.
- "Flip on short edge" keeps the back side upright when the sheet is folded into
  a signature, as the imposition expects. Long-edge flip would print every back
  side upside down.
- Leave the driver's own booklet and pages-per-sheet settings **off**, as the
  facsimile `README.md` already says; this tool already selects the exact pages
  and scale.
