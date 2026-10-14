"""Generate a parameterized dust-jacket LaTeX sheet + PDF for one facsimile Tomus."""

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path


MIN_FLAP_IN = 1.5
DEFAULT_PRINTABLE_MARGIN_IN = 0.25
FOLD_TICK_GAP_IN = 0.02
FOLD_TICK_LENGTH_IN = 0.10
SHEET_SIZES_IN = {
    "tabloid": (17.0, 11.0),
    "a3": (420.0 / 25.4, 297.0 / 25.4),
}
ROMAN_NUMERALS = (
    (1000, "M"), (900, "CM"), (500, "D"), (400, "CD"),
    (100, "C"), (90, "XC"), (50, "L"), (40, "XL"),
    (10, "X"), (9, "IX"), (5, "V"), (4, "IV"), (1, "I"),
)


class DustCoverError(Exception):
    """An invalid dust-cover request or failed generation."""


def int_to_roman(n: int) -> str:
    if n <= 0:
        raise DustCoverError("tomus must be a positive integer")
    result = []
    for value, numeral in ROMAN_NUMERALS:
        count, n = divmod(n, value)
        result.append(numeral * count)
    return "".join(result)


def roman_to_int(s: str) -> int:
    remaining = s
    value = 0
    for number, numeral in ROMAN_NUMERALS:
        while remaining.startswith(numeral):
            value += number
            remaining = remaining[len(numeral):]
    if remaining or not s or int_to_roman(value) != s:
        raise DustCoverError(f"invalid Roman numeral: {s!r}")
    return value


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


@dataclass(frozen=True)
class InnerTitle:
    center_block: str
    lives: list[str]


def parse_inner_tex(path: Path, expected_tomus: int) -> InnerTitle:
    try:
        source = path.read_text(encoding="utf-8")
    except OSError as exc:
        raise DustCoverError(f"cannot read inner TeX {path}: {exc}") from exc
    anchor = source.find("% Tomus title page")
    start = source.find(r"\begin{center}", anchor) if anchor >= 0 else -1
    end = source.find(r"\end{center}", start) if start >= 0 else -1
    if end < 0:
        raise DustCoverError(f"{path} has no Tomus title page")
    center_block = source[start + len(r"\begin{center}"):end]
    match = re.search(r"\\large\s+TOMUS\s+([IVXLCDM]+)", center_block)
    if not match:
        raise DustCoverError(f"{path} has no Tomus number on its title page")
    actual_tomus = roman_to_int(match.group(1))
    if actual_tomus != expected_tomus:
        raise DustCoverError(
            f"{path} title page is Tomus {actual_tomus}, expected Tomus {expected_tomus}"
        )
    lives = re.findall(
        r"\\mbox\{\\textsc\{\\MakeLowercase\{(.*?)\}\}\}", center_block
    )
    if not lives:
        raise DustCoverError(f"{path} title page lists no Lives")
    return InnerTitle(center_block, lives)


def render_preamble(sheet_long_in: float, sheet_short_in: float) -> str:
    return (
        "\\documentclass{article}\n"
        f"\\usepackage[paperwidth={sheet_long_in:.4f}in,"
        f"paperheight={sheet_short_in:.4f}in,margin=0pt]{{geometry}}\n"
        "\\usepackage{fontspec}\n"
        "\\setmainfont{OldStandard}[\n"
        "  Extension=.otf,\n"
        "  UprightFont=*-Regular,\n"
        "  BoldFont=*-Bold,\n"
        "  ItalicFont=*-Italic,\n"
        "  BoldItalicFont=*-BoldItalic]\n"
        "\\usepackage{tikz}\n"
        "\\usepackage{graphicx}\n"
        "\\pagestyle{empty}\n"
        "\\setlength{\\parindent}{0pt}\n"
    )


def panel_box(width_in: float, height_in: float, body: str) -> str:
    return (
        f"\\begin{{minipage}}[c][{height_in:.4f}in][c]{{{width_in:.4f}in}}"
        f"\\centering {body}\\end{{minipage}}"
    )


def front_cover_body(center_block: str, cover_width_in: float) -> str:
    width = f"{cover_width_in:.4f}in"
    return (
        f"{{\\setlength{{\\textwidth}}{{{width}}}"
        f"\\setlength{{\\columnwidth}}{{{width}}}\\hsize={width} "
        f"\\begin{{center}}{center_block}\\end{{center}}}}"
    )


def back_cover_body(lives: list[str]) -> str:
    return r"\\[0.8em]".join(
        rf"\textsc{{\MakeLowercase{{{life}}}}}" for life in lives
    )


def escape_latex(s: str) -> str:
    replacements = {
        "\\": r"\textbackslash{}", "&": r"\&", "%": r"\%",
        "#": r"\#", "_": r"\_", "$": r"\$",
        "{": r"\{", "}": r"\}", "~": r"\textasciitilde{}",
        "^": r"\textasciicircum{}",
    }
    return "".join(replacements.get(char, char) for char in s)


def spine_body(roman: str, exlibris: str | None, spine_width_in: float) -> str:
    usable_width = spine_width_in - 0.2
    title = (
        r"\shortstack[c]{{\large\bfseries PLUTARCHI}\\[0.4em]"
        r"{\large\bfseries VITÆ}\\[1.2em]"
        rf"{{\large TOMUS {roman}}}}}"
    )
    body = f"\\resizebox{{{usable_width:.4f}in}}{{!}}{{{title}}}"
    if exlibris is not None:
        owner = escape_latex(exlibris.upper())
        foot = rf"\shortstack[c]{{EX LIBRIS\\[0.6em]{{\large {owner}}}}}"
        body = (
            r"\par\vspace*{\fill}" + body + r"\par\vspace*{\fill}" +
            f"\\resizebox{{{usable_width:.4f}in}}{{!}}{{{foot}}}"
            r"\par\vspace{0.15in}"
        )
    return body


def render_cover(layout: Layout, sheet_long_in: float, sheet_short_in: float,
                 title: InnerTitle, roman: str, exlibris: str | None) -> str:
    ox = (sheet_long_in - layout.total_width_in) / 2
    oy = (sheet_short_in - layout.total_height_in) / 2
    top = oy + layout.total_height_in
    right = ox + layout.total_width_in
    lines = [
        render_preamble(sheet_long_in, sheet_short_in),
        r"\begin{document}",
        r"\noindent\begin{tikzpicture}[x=1in,y=1in]",
        f"  \\useasboundingbox (0,0) rectangle ({sheet_long_in:.4f},{sheet_short_in:.4f});",
        "  % The two flaps are blank.",
    ]
    panels = (
        (layout.back_x, layout.cover_in, back_cover_body(title.lives)),
        (layout.spine_x, layout.spine_in, spine_body(roman, exlibris, layout.spine_in)),
        (layout.front_x, layout.cover_in,
         front_cover_body(title.center_block, layout.cover_in)),
    )
    for panel_x, width, body in panels:
        lines.append(
            f"  \\node[anchor=south west,inner sep=0] at ({ox + panel_x:.4f},{oy:.4f}) "
            f"{{{panel_box(width, layout.total_height_in, body)}}};"
        )
    lines.append(
        f"  \\draw[dotted,thick] ({ox:.4f},{oy:.4f}) "
        f"rectangle ({right:.4f},{top:.4f});"
    )
    for seam in (layout.back_x, layout.spine_x,
                 layout.front_x, layout.right_flap_x):
        x = ox + seam
        lines.append(
            f"  \\draw[dotted] ({x:.4f},{oy - FOLD_TICK_GAP_IN:.4f}) -- "
            f"({x:.4f},{oy - FOLD_TICK_GAP_IN - FOLD_TICK_LENGTH_IN:.4f});"
        )
        lines.append(
            f"  \\draw[dotted] ({x:.4f},{top + FOLD_TICK_GAP_IN:.4f}) -- "
            f"({x:.4f},{top + FOLD_TICK_GAP_IN + FOLD_TICK_LENGTH_IN:.4f});"
        )
    lines.extend((r"\end{tikzpicture}", r"\end{document}"))
    return "\n".join(lines) + "\n"


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
    if total_height + 2 * (FOLD_TICK_GAP_IN + FOLD_TICK_LENGTH_IN) > printable_h + 1e-9:
        raise DustCoverError(
            f"jacket height {total_height:.3f}in plus exterior fold ticks exceeds "
            f"printable height {printable_h:.3f}in"
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
