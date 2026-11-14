import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('verify_templates',ROOT/'tools/verify_templates.py')
mod=importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

class StarterTests(unittest.TestCase):
    def test_all_profiles_structurally_valid(self):
        self.assertEqual(mod.check_templates(),[])

    def test_all_six_categories_exist(self):
        profiles=set(mod.PROFILES)
        for name in ('library','cli','desktop-egui','service-axum','mcp-stdio','web-leptos-csr'):
            self.assertIn(name,profiles)

    def test_skill_mirrors_are_really_equal(self):
        for name in mod.PROFILES:
            root=ROOT/'templates'/name
            for p in (root/'.agents/skills').glob('*/SKILL.md'):
                self.assertEqual(p.read_bytes(),(root/'.claude/skills'/p.parent.name/'SKILL.md').read_bytes())

    def test_license_is_complete(self):
        text=(ROOT/'LICENSE-APACHE').read_text()
        self.assertIn('TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION',text)
        self.assertGreater(len(text),10000)
        for profile in mod.PROFILES:
            self.assertEqual((ROOT/'templates'/profile/'LICENSE-APACHE').read_text(),text)

    def test_project_placeholder_rendering(self):
        for name in mod.PROFILES:
            original=(ROOT/'templates'/name/'Cargo.toml').read_text()
            output=mod.RENDER.sub('integration-test-app',original)
            self.assertNotIn('{{project-name}}',output)
            self.assertIn('[',output)

if __name__=='__main__':
    unittest.main()
