"""Tests for the standalone dust-cover generator."""

import unittest

from tools.dust_cover import DustCoverError, compute_layout


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
