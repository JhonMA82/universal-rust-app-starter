import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]

class AgentGuardTests(unittest.TestCase):
    def test_catches_replaced_claude_symlink(self):
        source=ROOT/'templates/cli'
        with tempfile.TemporaryDirectory() as tmp:
            project=Path(tmp)/'project'
            shutil.copytree(source,project)
            result=subprocess.run([sys.executable,'scripts/check_agents.py'],cwd=project,
                                  capture_output=True,text=True)
            self.assertNotEqual(result.returncode,0)
            self.assertIn('.claude/skills must be a symlink',result.stdout)

    def test_catches_wrong_claude_symlink(self):
        source=ROOT/'templates/cli'
        with tempfile.TemporaryDirectory() as tmp:
            project=Path(tmp)/'project'
            shutil.copytree(source,project,symlinks=True)
            link=project/'.claude/skills'
            link.unlink()
            link.symlink_to('../.agents/missing',target_is_directory=True)
            result=subprocess.run([sys.executable,'scripts/check_agents.py'],cwd=project,
                                  capture_output=True,text=True)
            self.assertNotEqual(result.returncode,0)
            self.assertIn('points to',result.stdout)

    def test_accepts_marked_copy_fallback(self):
        source=ROOT/'templates/cli'
        with tempfile.TemporaryDirectory() as tmp:
            project=Path(tmp)/'project'
            shutil.copytree(source,project)
            (project/'.claude/skills/.claude-copy-fallback').write_text('no symlink support\n')
            result=subprocess.run([sys.executable,'scripts/check_agents.py'],cwd=project,
                                  capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stdout)
            self.assertIn('copy fallback',result.stdout)

    def test_recreates_missing_claude_symlink(self):
        source=ROOT/'templates/cli'
        with tempfile.TemporaryDirectory() as tmp:
            project=Path(tmp)/'project'
            shutil.copytree(source,project)
            shutil.rmtree(project/'.claude/skills')
            result=subprocess.run([sys.executable,'scripts/check_agents.py'],cwd=project,
                                  capture_output=True,text=True)
            self.assertEqual(result.returncode,0,result.stdout)
            self.assertIn('Created .claude/skills',result.stdout)
            self.assertTrue((project/'.claude/skills').is_symlink())

    def test_catches_missing_canonical_contract(self):
        source=ROOT/'templates/library'
        with tempfile.TemporaryDirectory() as tmp:
            project=Path(tmp)/'project'
            shutil.copytree(source,project)
            (project/'AGENTS.md').unlink()
            result=subprocess.run([sys.executable,'scripts/check_agents.py'],cwd=project,
                                  capture_output=True,text=True)
            self.assertNotEqual(result.returncode,0)
            self.assertIn('missing AGENTS.md',result.stdout)

if __name__=='__main__':
    unittest.main()
