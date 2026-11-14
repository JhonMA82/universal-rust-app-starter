# Agent instructions — Universal Rust App Starter

## Goal
Build an extensible but lean Rust *project generator*, not a methodology, personal agent framework, or shared runtime. Each generated project must stand alone.

## Verify changes

```sh
python3 tools/verify_templates.py
python3 -m unittest discover -s tests -v
# When cargo-generate and rustc are installed:
python3 tools/verify_templates.py --generate --cargo-check
```

## Rules
- `templates/<profile>/` is a separate cargo-generate template. Do not introduce dependencies between generated projects and this repository.
- Maintain AGENTS.md as the single project contract; Claude and Gemini adapters point to it.
- `.agents/skills` and `.claude/skills` must be byte-identical per skill; run verification after edits.
- Keep each profile compilable without credentials. No deployed services or auto-publish.
- Avoid copying from noncommercial source code. Record third-party sources/attributions.
- Validate Liquid placeholders, Cargo files, scripts, license policy, and required docs.
- Don't claim compilation or real tool discovery without executing the relevant tools.
- No pushes, remote repository creation, deployments, or publishing without approval.
