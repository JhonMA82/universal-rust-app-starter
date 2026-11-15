"""Synthetic image tests for the artifact tools, not Rust/egui rendering tests."""
from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))

from init_scene import create_scene  # noqa: E402
from visual_check import run, validate_scene  # noqa: E402
from evaluate_runs import aggregate  # noqa: E402
from validate_toolkit import validate  # noqa: E402


class ToolkitTests(unittest.TestCase):
    def setUp(self):
        self.context = tempfile.TemporaryDirectory()
        self.addCleanup(self.context.cleanup)
        self.tmp = Path(self.context.name)
        self.reference = self.tmp / "reference.png"
        self.candidate = self.tmp / "candidate.png"
        self.scene_path = self.tmp / "scene.json"
        self.ref_arr = np.full((48, 80, 3), 40, np.uint8)
        self.ref_arr[10:38, 12:68] = [90, 110, 140]
        self.ref_arr[18:29, 20:40] = [210, 220, 230]
        self.save(self.ref_arr, self.reference)
        self.save(self.ref_arr, self.candidate)
        self.scene = create_scene(self.reference, self.scene_path, 1.0, "default")
        self.write_scene()

    @staticmethod
    def save(array, path):
        Image.fromarray(array, "RGB").save(path)

    def write_scene(self):
        self.scene_path.write_text(json.dumps(self.scene), encoding="utf-8")

    def compare(self, name="run", tag=None, case_id=None):
        return run(self.scene_path, self.candidate, self.tmp / name, tag, case_id, None, 0)

    def test_initializer_uses_relative_reference_and_checksum(self):
        ref_path, width, height = validate_scene(self.scene, self.scene_path)
        self.assertEqual(ref_path, self.reference.resolve())
        self.assertEqual((width, height), (80, 48))
        self.assertEqual(len(self.scene["reference"]["sha256"]), 64)

    def test_identical_image_passes_all_gates(self):
        report = self.compare()
        self.assertTrue(report["passed"])
        self.assertEqual(report["global"]["mae_rgb"], 0)
        self.assertEqual(report["global"]["edge_f1"], 1)
        for name in ("report.json", "report.md", "comparison.png", "heatmap.png", "overlay.png"):
            self.assertTrue((self.tmp / "run" / name).exists(), name)

    def test_difference_produces_gate_failure(self):
        changed = np.full_like(self.ref_arr, 255)
        self.save(changed, self.candidate)
        report = self.compare()
        self.assertFalse(report["passed"])
        self.assertGreater(report["global"]["mae_rgb"], 12)
        self.assertGreater(report["global"]["changed_fraction"], 0.12)

    def test_reference_modified_is_rejected(self):
        altered = self.ref_arr.copy()
        altered[2, 2] = 0
        self.save(altered, self.reference)
        with self.assertRaisesRegex(ValueError, "checksum changed"):
            self.compare()

    def test_dimension_mismatch_rejected(self):
        self.save(np.zeros((49, 80, 3), np.uint8), self.candidate)
        with self.assertRaisesRegex(ValueError, "mismatched images"):
            self.compare()

    def test_mask_budget_prevents_hiding_most_pixels(self):
        self.scene["ignore_rects_px"] = [{"rect_px": [0, 0, 80, 40], "reason": "a bad exclusion"}]
        self.write_scene()
        with self.assertRaisesRegex(ValueError, "ignored area"):
            self.compare()

    def test_missing_mask_reason_is_rejected(self):
        self.scene["ignore_rects_px"] = [{"rect_px": [0, 0, 2, 2], "reason": ""}]
        self.write_scene()
        with self.assertRaisesRegex(ValueError, "nonempty reason"):
            self.compare()

    def test_region_gate_can_fail_when_global_gate_is_lenient(self):
        altered = self.ref_arr.copy()
        altered[15:25, 10:20] = [255, 255, 255]
        self.save(altered, self.candidate)
        self.scene["tolerances"].update(max_mae=255, max_changed_fraction=1, min_edge_f1=0)
        self.scene["regions"] = [
            {"id": "affected", "rect_px": [10, 15, 10, 10], "weight": 2,
             "quality": {"max_mae": 1, "max_changed_fraction": 0.01, "min_edge_f1": 0}}
        ]
        self.write_scene()
        report = self.compare()
        self.assertEqual(report["global_failures"], [])
        self.assertFalse(report["passed"])
        self.assertTrue(report["regions"][0]["failures"])

    def test_small_mask_excludes_a_known_dynamic_area(self):
        altered = self.ref_arr.copy()
        altered[0:2, 0:2] = [255, 255, 255]
        self.save(altered, self.candidate)
        self.scene["ignore_rects_px"] = [{"rect_px": [0, 0, 2, 2], "reason": "synthetic blinking cursor"}]
        self.write_scene()
        self.assertTrue(self.compare()["passed"])

    def test_diagnostic_translation_does_not_change_gate(self):
        changed = np.roll(self.ref_arr, 2, axis=1)
        self.save(changed, self.candidate)
        report = run(self.scene_path, self.candidate, self.tmp / "shift", None, None, None, 3)
        self.assertEqual(report["shift_diagnostic"]["candidate_offset_x"], 2)
        self.assertGreater(report["global"]["mae_rgb"], 0)

    def test_case_paired_evaluation(self):
        self.compare(name="baseline-run", tag="baseline", case_id="synthetic-01")
        altered = self.ref_arr.copy()
        altered[10:25, 10:25] = 250
        self.save(altered, self.candidate)
        self.compare(name="improved-run", tag="improved", case_id="synthetic-01")
        result = aggregate(self.tmp)
        self.assertEqual(result["case_ids"], ["synthetic-01"])
        self.assertIn("baseline", result["condition_summary"])
        self.assertIn("improved", result["condition_summary"])

    def test_paired_evaluation_rejects_unmatched_reference(self):
        self.compare(name="baseline-run", tag="baseline", case_id="synthetic-01")
        report = self.compare(name="improved-run", tag="improved", case_id="synthetic-01")
        path = self.tmp / "improved-run" / "report.json"
        report["reference_sha256"] = "0" * 64
        path.write_text(json.dumps(report), encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "differs across conditions"):
            aggregate(self.tmp)

    def test_skill_pack_validator(self):
        self.assertEqual(validate(ROOT), [])


if __name__ == "__main__":
    unittest.main()
