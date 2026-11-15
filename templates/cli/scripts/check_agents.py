#!/usr/bin/env python3
"""Offline checks: canonical agent contract, skill frontmatter and Claude symlink."""
from pathlib import Path
import os, re, shutil, sys
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
link=Path('..')/'.agents'/'skills'
if not a.is_dir():
    bad('Missing .agents/skills')
else:
    if b.is_symlink():
        target=Path(os.readlink(b))
        if target != link:
            bad(f'.claude/skills points to {target}, expected {link}')
    elif not b.exists():
        b.parent.mkdir(parents=True, exist_ok=True)
        try:
            b.symlink_to(link, target_is_directory=True)
            print(f'Created .claude/skills -> {link}')
        except OSError as exc:
            shutil.copytree(a, b)
            (b/'.claude-copy-fallback').write_text(f'symlink unavailable: {exc}\n')
            print(f'WARNING: symlink unavailable ({exc}); copied skills instead')
    elif (b/'.claude-copy-fallback').is_file():
        print('WARNING: .claude/skills is a copy fallback; symlink preferred')
    else:
        bad('.claude/skills must be a symlink to ../.agents/skills, not a copy')
    if not (b/'.claude-copy-fallback').is_file() and b.exists() and b.resolve() != a.resolve():
        bad('.claude/skills does not resolve to .agents/skills')
    src=sorted(a.glob('*/SKILL.md'))
    if not src: bad('No skills installed')
    if b.exists() and {x.parent.name for x in src} != {x.parent.name for x in b.glob('*/SKILL.md')}:
        bad('Claude skills not in sync')
    for p in src:
        body=p.read_text()
        match=re.match(r'^---\n(.*?)\n---\n',body,re.S)
        if not match or not re.search(r'^name:\s*\S+',match.group(1),re.M) or not re.search(r'^description:\s*\S+',match.group(1),re.M):
            bad(f'Invalid skill frontmatter: {p}')
if errors:
    print('Agent readiness FAILED:')
    for error in errors: print(' -',error)
    sys.exit(1)
mode='symlink' if b.is_symlink() else 'copy fallback'
print(f'Agent readiness OK: {len(list(a.glob("*/SKILL.md")))} validated skills; .claude/skills ({mode}) -> .agents/skills')
