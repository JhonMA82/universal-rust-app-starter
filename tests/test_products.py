"""Exercise capability resolution without requiring Rust or writing a project."""
import unittest

import starter
from generator.compose import CompositionError


class ProductPlanTests(unittest.TestCase):
    def test_general_desktop_installs_domain_ui_and_tasks(self):
        request = {"product": "desktop-application", "capabilities": ["automation"], "name": "test-app"}
        plan = starter.plan(request)
        self.assertEqual(plan, starter.plan(plan["request"]))
        self.assertEqual(plan["providers"], ["desktop-product"])
        self.assertEqual(plan["targets"], ["native"])
        self.assertTrue({"operations", "desktop-ui", "automation", "background-work"} <= set(plan["capabilities"]))
        self.assertIn("mcp-stdio", plan["entrypoints"])
        self.assertIn("ui_app", plan["crates"])
        self.assertIn("schemars", plan["declared_dependency_constraints"])

    def test_ready_workspaces_install_integrated_providers(self):
        for request, provider, capability in (
            ({"product": "desktop-application", "capabilities": ["data-visualization"]}, "data-demo", "data-visualization"),
            ({"product": "document-workbench"}, "document-demo", "documents"),
            ({"product": "engineering-workbench"}, "engineering-demo", "engineering-views"),
            ({"profile": "desktop-egui", "capabilities": ["background-work"]}, "desktop-product", "background-work"),
        ):
            with self.subTest(request=request):
                plan = starter.plan(request)
                self.assertEqual(plan["providers"], [provider])
                self.assertIn(capability, plan["capabilities"])
                self.assertIn("desktop", plan["entrypoints"])

    def test_documents_and_data_cannot_mask_each_other(self):
        with self.assertRaisesRegex(CompositionError, "one integrated workspace"):
            starter.plan({"product": "document-workbench", "capabilities": ["data-visualization"]})

    def test_browser_requires_a_supported_adapter_even_when_checking_native(self):
        with self.assertRaisesRegex(CompositionError, "incompatible"):
            starter.plan({"profile": "desktop-egui-web", "targets": ["native"], "capabilities": ["background-work"]})

    def test_unknown_ids_and_invalid_config_types_are_rejected(self):
        for request in (
            {"product": "unknown"},
            {"profile": "desktop-egui", "capabilities": "automation"},
            {"profile": "desktop-egui", "capabilities": ["unknown"]},
        ):
            with self.subTest(request=request):
                with self.assertRaises(CompositionError):
                    starter.plan(request)

    def test_plans_cover_all_existing_profiles_and_baseline_presets(self):
        for profile in starter.PROFILES:
            for preset in ("minimal", "application"):
                with self.subTest(profile=profile, preset=preset):
                    self.assertEqual(starter.plan({"profile": profile, "preset": preset})["composition"]["profile"], profile)
