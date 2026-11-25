import shutil
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from generator.compose import ROOT, CompositionError, catalog, check_files, composition_guide, merge, read_json, resolve
from generator import integration
from tools.check_docs import project_contract
from tools import verify_templates


def module(deps=(), *, targets=("native", "wasm"), conflicts=()):
    return {"version": "1.0.0", "dependencies": list(deps), "conflicts": list(conflicts),
            "profiles": [], "targets": list(targets)}


class CompositionTests(unittest.TestCase):
    def test_all_reference_specs_resolve(self):
        modules = catalog()
        presets = read_json(ROOT / "presets/catalog.json")
        for name, config in read_json(ROOT / "examples/catalog.json").items():
            with self.subTest(name=name):
                requested = presets[config["preset"]] + config["modules"]
                self.assertTrue(set(requested) <= set(resolve(modules, requested, config["profile"], {"native"})))

    def test_transitive_order_and_failures(self):
        spec = {"a": module(["b@1.0.0"]), "b": module(targets=["native"])}
        self.assertEqual(resolve(spec, ["a"], "cli", {"native"}), ["b", "a"])
        for requested, targets, expected in [
            (["b@2.0.0"], {"native"}, "version"), (["a"], {"wasm"}, "does not support"),
            (["missing"], {"native"}, "unknown"),
        ]:
            with self.assertRaisesRegex(CompositionError, expected):
                resolve(spec, requested, "cli", targets)
        with self.assertRaisesRegex(CompositionError, "cycle"):
            resolve({"a": module(["b"]), "b": module(["a"])}, ["a"], "cli", {"native"})
        with self.assertRaisesRegex(CompositionError, "conflicts"):
            resolve({"a": module(conflicts=["b"]), "b": module()}, ["a", "b"], "cli", {"native"})

    def test_manifest_conflict_and_portable_file_collisions(self):
        with self.assertRaisesRegex(CompositionError, "dependencies.serde"):
            merge({"dependencies": {"serde": "1"}}, {"dependencies": {"serde": "2"}})
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Cargo.toml").touch()
            for paths in ({"../escape"}, {"cargo.toml"}, {"a/Foo", "A/Bar"}):
                with self.subTest(paths=paths), self.assertRaises(CompositionError):
                    check_files(root, paths, set())

    def test_application_adapters_connect_all_profile_templates(self):
        modules = catalog()
        profiles = read_json(ROOT / "profiles.json")
        expected = {
            "library": "operations-api", "cli": "operations-cli",
            "desktop-egui": "desktop", "desktop-egui-web": "operations-web",
            "service-axum": "application-http", "mcp-stdio": "mcp-stdio",
            "mcp-http": "mcp-http", "mcp-client": "operations-cli-companion",
            "web-leptos-csr": "operations-web", "web-leptos-fullstack": "application-http",
            "web-dioxus": "operations-web",
        }
        self.assertEqual(profiles.keys(), expected.keys())
        for profile, descriptor in profiles.items():
            with self.subTest(profile=profile), tempfile.TemporaryDirectory() as temporary:
                project = Path(temporary) / "generated"
                shutil.copytree(ROOT / "templates" / profile, project, symlinks=True)
                requested = ["application-core"]
                ordered = resolve(modules, requested, profile, set(descriptor["targets"]))
                while needed := integration.plan(profile, "application", ordered, modules):
                    requested.extend(sorted(needed))
                    ordered = resolve(modules, requested, profile, set(descriptor["targets"]))
                manifests, interfaces = integration.install(
                    project, profile, ordered, modules, name="contract-app", merge=merge,
                )
                self.assertIn(expected[profile], interfaces)
                self.assertTrue(manifests)

    def test_missing_duplicate_and_noop_integration_anchors_are_rejected(self):
        for source, replacement in [("", "replacement"), ("marker marker", "replacement"), ("marker", "marker")]:
            with self.subTest(source=source), self.assertRaises(integration.IntegrationError):
                integration.replace_once(Path("main.rs"), source, "marker", replacement)

    def test_generated_guide_detects_unselected_operation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            modules = {"core": {"documentation": "docs/core.md", "description": "Core", "files": ["docs/core.md"]}}
            report = {"profile": "cli", "preset": "application", "entrypoints": ["operations-cli"], "modules": {"core": "1"}}
            guide = root / "docs/COMPOSITION.md"
            guide.parent.mkdir()
            (root / "docs/core.md").touch()
            guide.write_text(composition_guide("cli", "application", ["operations-cli", "mcp-stdio"], modules))
            self.assertIn("generated composition guide diverges", next(project_contract(root, report, modules)))
            guide.write_text(composition_guide("cli", "application", ["operations-cli"], modules))
            self.assertEqual([], list(project_contract(root, report, modules)))

    def test_hybrid_checks_both_targets(self):
        calls = []
        def run(cmd, _):
            calls.append(cmd)
            if cmd[1] == "generate":
                path = Path(cmd[cmd.index("--destination") + 1]) / cmd[cmd.index("--name") + 1]
                path.mkdir()
                (path / "Cargo.toml").touch()
        hybrid = {"desktop-egui-web": verify_templates.PROFILES["desktop-egui-web"]}
        with patch.object(verify_templates, "PROFILES", hybrid), patch.object(verify_templates, "run", run), \
             patch.object(verify_templates.shutil, "which", lambda tool: tool):
            self.assertEqual(verify_templates.generate_and_check(True), [])
        self.assertIn(["cargo", "check", "-p", "web", "--target", "wasm32-unknown-unknown"], calls)
        self.assertIn(["cargo", "test", "--workspace"], calls)
