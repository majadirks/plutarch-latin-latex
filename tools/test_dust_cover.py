"""Tests for the standalone dust-cover generator."""

import unittest
import re
from subprocess import CompletedProcess
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from tools.dust_cover import (
    MAX_TOMUS, SHEET_SIZES_IN, DustCoverError, InnerTitle, back_cover_body,
    check_printable, compile_cover, compute_layout, escape_latex, front_cover_body,
    int_to_roman, main, panel_box, parse_inner_tex, render_cover, render_preamble,
    roman_to_int, spine_body, validate_exlibris, write_cover_tex,
)


class LayoutTests(unittest.TestCase):
    def test_flap_width_computed_from_total(self):
        layout = compute_layout(15.5, 9.5, 1.25, 5.5)
        self.assertAlmostEqual(layout.flap_in, 1.625)
        self.assertAlmostEqual(
            2 * layout.flap_in + 2 * layout.cover_in + layout.spine_in,
            layout.total_width_in,
        )

    def test_flap_below_minimum_raises(self):
        with self.assertRaisesRegex(DustCoverError, "minimum"):
            compute_layout(15.5, 9.5, 1.25, 6.5)

    def test_flap_exactly_minimum_allowed(self):
        self.assertEqual(compute_layout(15.25, 9.5, 1.25, 5.5).flap_in, 1.5)

    def test_panel_offsets_follow_flap_back_spine_front_flap_order(self):
        layout = compute_layout(15.5, 9.5, 1.25, 5.5)
        self.assertLess(0, layout.back_x)
        self.assertLess(layout.back_x, layout.spine_x)
        self.assertLess(layout.spine_x, layout.front_x)
        self.assertLess(layout.front_x, layout.right_flap_x)
        self.assertAlmostEqual(layout.right_flap_x + layout.flap_in, 15.5)


class ValidationTests(unittest.TestCase):
    def test_total_width_exceeds_printable_raises(self):
        with self.assertRaisesRegex(DustCoverError, "printable width"):
            check_printable(17, 9.5, "tabloid", 0.25)

    def test_total_height_exceeds_printable_raises(self):
        with self.assertRaisesRegex(DustCoverError, "printable height"):
            check_printable(15.5, 11, "tabloid", 0.25)

    def test_fold_ticks_need_printable_clearance(self):
        with self.assertRaisesRegex(DustCoverError, "exterior fold ticks"):
            check_printable(15.5, 10.5, "tabloid", 0.25)

    def test_a3_printable_area_smaller_than_tabloid(self):
        self.assertLess(SHEET_SIZES_IN["a3"][0] - 0.5, 16.5)

    def test_validate_exlibris_rejects_blank(self):
        for value in ("", "   ", "\t"):
            with self.subTest(value=value), self.assertRaises(DustCoverError):
                validate_exlibris(value)
        self.assertEqual(validate_exlibris(" Alex "), " Alex ")


class RomanNumeralTests(unittest.TestCase):
    def test_int_to_roman_and_back_roundtrips(self):
        for n in range(1, MAX_TOMUS + 1):
            with self.subTest(n=n):
                self.assertEqual(roman_to_int(int_to_roman(n)), n)
        self.assertEqual(int_to_roman(12), "XII")

    def test_tomus_outside_valid_range_raises(self):
        with self.assertRaisesRegex(DustCoverError, f"between 1 and {MAX_TOMUS}"):
            int_to_roman(MAX_TOMUS + 1)
        with self.assertRaisesRegex(DustCoverError, f"between 1 and {MAX_TOMUS}"):
            int_to_roman(0)
        with self.assertRaisesRegex(DustCoverError, "invalid Tomus numeral"):
            roman_to_int("nonsense")


TITLE_FIXTURE = r"""% Tomus title page
\unitrecto
\begin{center}
{\LARGE\bfseries PLUTARCHI}\\[0.55em]
{\LARGE\bfseries VITÆ}\\[1.4em]
{\large TOMUS II}\\[2.6em]
\parbox{0.86\textwidth}{\centering \mbox{\textsc{\MakeLowercase{THESEUS}}} \textbullet\ \mbox{\textsc{\MakeLowercase{ROMULUS}}}\par}
\end{center}
\begin{center}EX LIBRIS\\ ALEX\end{center}
"""


class InnerTitleTests(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / "tomus_2_inner.tex"
        self.path.write_text(TITLE_FIXTURE, encoding="utf-8")

    def test_lives_parsed_from_inner_tex(self):
        self.assertEqual(parse_inner_tex(self.path, 2).lives, ["THESEUS", "ROMULUS"])

    def test_center_block_excludes_exlibris_plate(self):
        title = parse_inner_tex(self.path, 2)
        self.assertIn("PLUTARCHI", title.center_block)
        self.assertNotIn("EX LIBRIS", title.center_block)

    def test_tomus_mismatch_between_arg_and_tex_raises(self):
        with self.assertRaisesRegex(DustCoverError, "Tomus 2.*Tomus 1"):
            parse_inner_tex(self.path, 1)

    def test_missing_title_page_raises(self):
        self.path.write_text(r"\begin{center}No title\end{center}", encoding="utf-8")
        with self.assertRaisesRegex(DustCoverError, "no Tomus title page"):
            parse_inner_tex(self.path, 2)


class RenderPreambleTests(unittest.TestCase):
    def test_sheet_and_book_font_are_in_preamble(self):
        preamble = render_preamble(17, 11)
        self.assertIn("paperwidth=17.0000in,paperheight=11.0000in", preamble)
        self.assertIn(r"\setmainfont{OldStandard}", preamble)
        self.assertIn(r"\usepackage{tikz}", preamble)


class PanelBuilderTests(unittest.TestCase):
    def test_front_cover_reproduces_title_block_without_exlibris(self):
        body = front_cover_body("TITLE BLOCK", 5.5)
        self.assertIn("TITLE BLOCK", body)
        self.assertIn(r"\setlength{\textwidth}{5.5000in}", body)
        self.assertNotIn("EX LIBRIS", body)

    def test_back_cover_lists_each_life_on_its_own_line(self):
        body = back_cover_body(["THESEUS", "ROMULUS"])
        self.assertEqual(
            body,
            r"\textsc{\MakeLowercase{THESEUS}}\\[0.8em]"
            r"\textsc{\MakeLowercase{ROMULUS}}",
        )

    def test_spine_lists_plutarchi_vitae_and_tomus(self):
        body = spine_body("III", None, 1.25)
        for word in ("PLUTARCHI", "VITÆ", "TOMUS III"):
            self.assertEqual(body.count(word), 1)
        self.assertIn(r"\resizebox{1.0500in}{!}", body)

    def test_spine_exlibris_uppercased_at_foot(self):
        body = spine_body("I", "alex smith", 1.25)
        self.assertIn("ALEX SMITH", body)
        self.assertIn("EX LIBRIS", body)
        self.assertGreater(body.index("EX LIBRIS"), body.index("TOMUS I"))
        self.assertNotIn("EX LIBRIS", spine_body("I", None, 1.25))

    def test_panel_box_has_fixed_size(self):
        box = panel_box(5.5, 9.5, "BODY")
        self.assertIn(r"\begin{minipage}[c][9.5000in][c]{5.5000in}", box)
        self.assertIn("BODY", box)

    def test_exlibris_name_uppercased_then_escaped(self):
        body = spine_body("I", "a & b_c", 1.25)
        self.assertIn(r"A \& B\_C", body)

    def test_owner_name_special_characters_are_escaped(self):
        self.assertEqual(
            escape_latex("\\{}%_#$~^&"),
            r"\textbackslash{}\{\}\%\_\#\$\textasciitilde{}"
            r"\textasciicircum{}\&",
        )


class RenderCoverTests(unittest.TestCase):
    def setUp(self):
        self.layout = compute_layout(15.5, 9.5, 1.25, 5.5)
        self.title = InnerTitle("\nTITLE BLOCK\n", ["THESEUS", "ROMULUS"])
        self.tex = render_cover(self.layout, 17, 11, self.title, "III", None)

    def test_layout_order_flap_back_spine_front_flap(self):
        xs = [float(x) for x in re.findall(
            r"\\node\[anchor=south west,inner sep=0\] at \(([\d.]+),", self.tex
        )]
        self.assertEqual(len(xs), 3)
        self.assertEqual(xs, sorted(xs))
        self.assertGreater(xs[0], 0.75)
        self.assertLess(xs[-1], 16.25)

    def test_cut_rectangle_matches_total_size(self):
        match = re.search(
            r"\\draw\[dotted,thick\] \(([\d.]+),([\d.]+)\) "
            r"rectangle \(([\d.]+),([\d.]+)\)", self.tex
        )
        self.assertIsNotNone(match)
        x0, y0, x1, y1 = map(float, match.groups())
        self.assertAlmostEqual(x1 - x0, 15.5)
        self.assertAlmostEqual(y1 - y0, 9.5)

    def test_front_cover_reproduces_title_block_without_exlibris(self):
        self.assertIn("TITLE BLOCK", self.tex)
        self.assertEqual(self.tex.count("TITLE BLOCK"), 1)
        self.assertNotIn("EX LIBRIS", self.tex)

    def test_fold_ticks_at_four_seams_outside_cut(self):
        segments = re.findall(
            r"\\draw\[dotted\] \(([\d.]+),([\d.]+)\) -- "
            r"\(([\d.]+),([\d.]+)\)", self.tex
        )
        self.assertEqual(len(segments), 8)
        self.assertEqual(len({float(s[0]) for s in segments}), 4)
        for x0, y0, x1, y1 in segments:
            self.assertEqual(x0, x1)
            self.assertTrue(max(float(y0), float(y1)) < 0.75 or
                            min(float(y0), float(y1)) > 10.25)

    def test_exlibris_appears_only_on_spine(self):
        tex = render_cover(self.layout, 17, 11, self.title, "III", "alex smith")
        self.assertEqual(tex.count("EX LIBRIS"), 1)
        self.assertIn("ALEX SMITH", tex)


class OutputTests(unittest.TestCase):
    def test_write_cover_tex_creates_file(self):
        with TemporaryDirectory() as directory:
            path = write_cover_tex(Path(directory), 1, "COVER")
            self.assertEqual(path.name, "tomus_1_cover.tex")
            self.assertEqual(path.read_text(encoding="utf-8"), "COVER")

    def test_compile_invokes_lualatex(self):
        with TemporaryDirectory() as directory:
            path = Path(directory) / "tomus_1_cover.tex"
            calls = []

            def runner(command, **kwargs):
                calls.append((command, kwargs))
                return CompletedProcess(command, 0)

            self.assertEqual(compile_cover(path, runner), path.with_suffix(".pdf"))
            self.assertEqual(calls[0][0][0], "lualatex")
            self.assertEqual(calls[0][1]["cwd"], path.parent)

    def test_compile_failure_raises(self):
        with TemporaryDirectory() as directory:
            path = Path(directory) / "tomus_1_cover.tex"

            def runner(command, **kwargs):
                return CompletedProcess(command, 1, stderr="TeX failed")

            with self.assertRaisesRegex(DustCoverError, "lualatex failed"):
                compile_cover(path, runner)


class MainTests(unittest.TestCase):
    def setUp(self):
        self.temp = TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.out = Path(self.temp.name)
        (self.out / "tomus_2_inner.tex").write_text(TITLE_FIXTURE, encoding="utf-8")
        self.args = [
            "--tomus", "2", "--total-width", "15.5", "--total-height", "9.5",
            "--spine-width", "1.25", "--cover-width", "5.5",
            "--out", str(self.out),
        ]

    def test_no_compile_writes_tex_only(self):
        with patch("tools.dust_cover.compile_cover") as compile_mock:
            self.assertEqual(main(self.args + ["--no-compile"]), 0)
            compile_mock.assert_not_called()
        self.assertTrue((self.out / "tomus_2_cover.tex").exists())
        self.assertFalse((self.out / "tomus_2_cover.pdf").exists())

    def test_main_exits_2_on_impossible_geometry(self):
        args = self.args.copy()
        args[args.index("15.5")] = "17"
        self.assertEqual(main(args + ["--no-compile"]), 2)
        self.assertFalse((self.out / "tomus_2_cover.tex").exists())
