# Validation report — v1.0.0

**Packaging date:** 2026-11-14

## Completed in the packaging environment

- Checked structural integrity of all **11 subtemplates** and the six profile families.
- Parsed all Cargo and template TOML manifests with Python `tomllib`, substituting the project-name placeholder offline.
- Parsed **23 GitHub Actions YAML files** for valid syntax.
- Checked `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, architecture docs, required files and canonical skill frontmatter.
- Verified every `.agents/skills` / `.claude/skills` pair is byte-identical.
- Executed **7 Python unit tests** successfully, including negative tests proving the agent checker rejects missing instructions and changed skills.
- Confirmed both MIT and complete Apache-2.0 license text exist in all profiles.
- Reviewed relevant current public examples for `eframe 0.36`, `rmcp 3.10`, and `Leptos 0.8` to improve alignment.

## Not completed in this environment

`rustc`, `cargo`, `cargo-generate`, `trunk` and `dx` were not installed. Therefore **no generated application has been compiled**, no Rust unit tests run, no browser smoke tests run, and no MCP tool calls executed. The project is an implemented v1 starter package with static validation, **not an integration-verified production release**.

## How to finish integration verification

```sh
rustup target add wasm32-unknown-unknown
cargo install cargo-generate --locked
python3 tools/verify_templates.py --generate --cargo-check
```

Then inspect GUI runners and browser builds and test MCP protocol handshakes using actual inspector clients. See `docs/VALIDATION.md` and `docs/PROFILE_MATRIX.md`. CI in `.github/workflows/validate-templates.yml` is set up to run generation + Rust checks when the repository is hosted on GitHub, but it has not been executed at packaging time.
