"""Tests for the standalone dust-cover generator."""

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from tools.dust_cover import (
    SHEET_SIZES_IN, DustCoverError, back_cover_body, check_printable,
    compute_layout, front_cover_body, int_to_roman, panel_box,
    parse_inner_tex, render_preamble, roman_to_int, spine_body,
    validate_exlibris,
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

    def test_a3_printable_area_smaller_than_tabloid(self):
        self.assertLess(SHEET_SIZES_IN["a3"][0] - 0.5, 16.5)

    def test_validate_exlibris_rejects_blank(self):
        for value in ("", "   ", "\t"):
            with self.subTest(value=value), self.assertRaises(DustCoverError):
                validate_exlibris(value)
        self.assertEqual(validate_exlibris(" Alex "), " Alex ")


class RomanNumeralTests(unittest.TestCase):
    def test_int_to_roman_and_back_roundtrips(self):
        for n in range(1, 13):
            with self.subTest(n=n):
                self.assertEqual(roman_to_int(int_to_roman(n)), n)


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
