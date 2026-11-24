# Agent instructions — Universal Rust App Starter

## Goal
Build an extensible but lean Rust *project generator*, not a methodology, personal agent framework, or shared runtime. Each generated project must stand alone.

## Verify changes

```sh
python3 tools/verify_templates.py
python3 tools/check_docs.py
python3 -m unittest discover -s tests -v
# When cargo-generate and rustc are installed:
python3 tools/verify_templates.py --generate --cargo-check
# Modular composition (writes evidence; never commit it):
python3 tools/verify_composed.py --reference document-workbench --evidence evidence/reference.json
python3 tools/verify_composed.py --ordinary --evidence evidence/ordinary.json
python3 tools/verify_composed.py --hybrid --evidence evidence/hybrid.json
```

## Rules
- `templates/<profile>/` is a separate cargo-generate template. Do not introduce dependencies between generated projects and this repository.
- `generator/compose.py` is the only composer. Modules live in `modules/<id>/module.json`; their `files` list must match the `files/` payload, and `docs/MODULE_CATALOG.md` mirrors ids, targets, dependencies and documentation links.
- `generator/`, `modules/`, `presets/` and `examples/` changes require a composed validation run; `verify_templates.py` covers only the base profiles.
- `evidence/` is generated locally or by CI. Do not commit evidence artifacts; regenerate them with the commands in `docs/VALIDATION.md`.
- Maintain AGENTS.md as the single project contract; Claude and Gemini adapters point to it.
- `.agents/skills` is canonical; `.claude/skills` must be a relative symlink to `../.agents/skills`; run verification after edits.
- Keep each profile compilable without credentials. No deployed services or auto-publish.
- Avoid copying from noncommercial source code. Record third-party sources/attributions.
- Validate Liquid placeholders, Cargo files, scripts, license policy, and required docs.
- Don't claim compilation or real tool discovery without executing the relevant tools.
- No pushes, remote repository creation, deployments, or publishing without approval.
