import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]

class AgentGuardTests(unittest.TestCase):
    def test_catches_modified_claude_skill(self):
        source=ROOT/'templates/cli'
        with tempfile.TemporaryDirectory() as tmp:
            project=Path(tmp)/'project'
            shutil.copytree(source,project)
            p=project/'.claude/skills/idiomatic-rust/SKILL.md'
            p.write_text(p.read_text()+'\nDRIFT\n')
            result=subprocess.run([sys.executable,'scripts/check_agents.py'],cwd=project,
                                  capture_output=True,text=True)
            self.assertNotEqual(result.returncode,0)
            self.assertIn('Drifted Claude skill',result.stdout)

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
