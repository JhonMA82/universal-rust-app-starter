#!/usr/bin/env python3
"""Offline checks: canonical agent contract, skill frontmatter and mirror parity."""
from pathlib import Path
import re, sys
r = Path(__file__).resolve().parents[1]
errors=[]
def bad(msg): errors.append(msg)
for name in ['AGENTS.md','CLAUDE.md','GEMINI.md','docs/ARCHITECTURE.md']:
    if not (r/name).is_file(): bad(f'missing {name}')
contract=(r/'AGENTS.md').read_text() if (r/'AGENTS.md').exists() else ''
if 'cargo ' not in contract: bad('AGENTS.md must contain executable Cargo commands')
if (r/'CLAUDE.md').exists() and 'AGENTS.md' not in (r/'CLAUDE.md').read_text():
    bad('Claude adapter missing canonical reference')
if (r/'GEMINI.md').exists() and 'AGENTS.md' not in (r/'GEMINI.md').read_text():
    bad('Gemini adapter missing canonical reference')
a=r/'.agents/skills'; b=r/'.claude/skills'
if not a.exists(): bad('Missing .agents/skills')
else:
    src=sorted(a.glob('*/SKILL.md')); dst=sorted(b.glob('*/SKILL.md'))
    if not src: bad('No skills installed')
    if {x.parent.name for x in src}!={x.parent.name for x in dst}: bad('Claude skills not in sync')
    for p in src:
        body=p.read_text()
        match=re.match(r'^---\n(.*?)\n---\n',body,re.S)
        if not match or not re.search(r'^name:\s*\S+',match.group(1),re.M) or not re.search(r'^description:\s*\S+',match.group(1),re.M):
            bad(f'Invalid skill frontmatter: {p}')
        target=b/p.parent.name/'SKILL.md'
        if target.exists() and target.read_bytes()!=p.read_bytes(): bad(f'Drifted Claude skill: {p.parent.name}')
if errors:
    print('Agent readiness FAILED:')
    for error in errors: print(' -',error)
    sys.exit(1)
print(f'Agent readiness OK: {len(list(a.glob("*/SKILL.md")))} validated skills; mirrors match')
