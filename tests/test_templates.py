import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

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

    def test_generate_passes_existing_parent_destination(self):
        # cargo-generate 0.26 treats --destination as a parent directory that must
        # already exist and creates <destination>/<name>. A nonexistent destination
        # fails before generation, which is how every profile silently broke.
        calls=[]
        def fake_run(cmd, cwd):
            calls.append((list(cmd), Path(cwd)))
            if cmd[1]=='generate':
                dest=Path(cmd[cmd.index('--destination')+1])
                if not dest.is_dir():
                    raise RuntimeError('destination directory does not exist')
                project=dest/cmd[cmd.index('--name')+1]
                project.mkdir()
                (project/'Cargo.toml').write_text('[package]\nname="smoke"\n')
                (project/'scripts').mkdir()
                (project/'scripts/check_agents.py').write_text('')
        with mock.patch.object(mod,'run',fake_run), \
             mock.patch.object(mod.shutil,'which',lambda tool:'/usr/bin/'+tool):
            errors=mod.generate_and_check(cargo_check=False)
        self.assertEqual(errors,[])
        self.assertEqual(len(calls),2*len(mod.PROFILES))

    def test_project_placeholder_rendering(self):
        for name in mod.PROFILES:
            original=(ROOT/'templates'/name/'Cargo.toml').read_text()
            output=mod.RENDER.sub('integration-test-app',original)
            self.assertNotIn('{{project-name}}',output)
            self.assertIn('[',output)

if __name__=='__main__':
    unittest.main()
