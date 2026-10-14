"""Generate a parameterized dust-jacket LaTeX sheet + PDF for one facsimile Tomus."""

import argparse
import sys
from dataclasses import dataclass
from pathlib import Path


MIN_FLAP_IN = 1.5
DEFAULT_PRINTABLE_MARGIN_IN = 0.25
SHEET_SIZES_IN = {
    "tabloid": (17.0, 11.0),
    "a3": (420.0 / 25.4, 297.0 / 25.4),
}


class DustCoverError(Exception):
    """An invalid dust-cover request or failed generation."""


@dataclass(frozen=True)
class Layout:
    flap_in: float
    cover_in: float
    spine_in: float
    total_width_in: float
    total_height_in: float
    back_x: float
    spine_x: float
    front_x: float
    right_flap_x: float


def compute_layout(total_width: float, total_height: float,
                   spine_width: float, cover_width: float) -> Layout:
    flap = (total_width - 2 * cover_width - spine_width) / 2
    if flap < MIN_FLAP_IN - 1e-9:
        raise DustCoverError(
            f"flap width {flap:.3f}in is below the {MIN_FLAP_IN}in minimum; "
            "reduce spine/cover widths or increase total width"
        )
    return Layout(
        flap, cover_width, spine_width, total_width, total_height,
        flap, flap + cover_width, flap + cover_width + spine_width,
        flap + 2 * cover_width + spine_width,
    )


def validate_exlibris(name: str) -> str:
    if not name.strip():
        raise DustCoverError("exlibris name must not be blank")
    return name


def check_printable(total_width: float, total_height: float,
                    paper_size: str, margin: float) -> None:
    long_in, short_in = SHEET_SIZES_IN[paper_size]
    printable_w = long_in - 2 * margin
    printable_h = short_in - 2 * margin
    if total_width > printable_w + 1e-9:
        raise DustCoverError(
            f"jacket width {total_width:.3f}in exceeds printable width "
            f"{printable_w:.3f}in"
        )
    if total_height > printable_h + 1e-9:
        raise DustCoverError(
            f"jacket height {total_height:.3f}in exceeds printable height "
            f"{printable_h:.3f}in"
        )


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
        if args.exlibris is not None:
            validate_exlibris(args.exlibris)
        check_printable(args.total_width, args.total_height,
                        args.paper_size, args.printable_margin)
        layout = compute_layout(args.total_width, args.total_height,
                                args.spine_width, args.cover_width)
        print(
            f"sheet={args.paper_size} jacket={args.total_width}x{args.total_height}in "
            f"cover={args.cover_width}in spine={args.spine_width}in "
            f"flap={layout.flap_in:.3f}in"
        )
    except DustCoverError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
