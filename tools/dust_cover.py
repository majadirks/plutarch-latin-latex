"""Generate a parameterized dust-jacket LaTeX sheet + PDF for one facsimile Tomus."""

import argparse
import sys
from pathlib import Path


MIN_FLAP_IN = 1.5
DEFAULT_PRINTABLE_MARGIN_IN = 0.25
SHEET_SIZES_IN = {
    "tabloid": (17.0, 11.0),
    "a3": (420.0 / 25.4, 297.0 / 25.4),
}


class DustCoverError(Exception):
    """An invalid dust-cover request or failed generation."""


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tomus", type=int, required=True)
    parser.add_argument("--total-width", type=float, required=True)
    parser.add_argument("--total-height", type=float, required=True)
    parser.add_argument("--spine-width", type=float, required=True)
    parser.add_argument("--cover-width", type=float, required=True)
    parser.add_argument("--paper-size", choices=SHEET_SIZES_IN, default="tabloid")
    parser.add_argument("--exlibris")
    parser.add_argument("--printable-margin", type=float, default=DEFAULT_PRINTABLE_MARGIN_IN)
    parser.add_argument("--out", type=Path, default=Path("build_facsimile"))
    parser.add_argument("--inner-tex", type=Path)
    parser.add_argument("--no-compile", action="store_true")
    args = parser.parse_args(argv)
    try:
        print(
            f"sheet={args.paper_size} jacket={args.total_width}x{args.total_height}in "
            f"cover={args.cover_width}in spine={args.spine_width}in"
        )
    except DustCoverError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
