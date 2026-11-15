# Validation report — v1.0.0 (unreleased fixes)

**Packaging date:** 2026-11-14
**Integration verification:** 2026-11-14, Rust 1.100.0 stable, cargo-generate 0.26.0, `wasm32-unknown-unknown` installed.

## Completed in the packaging environment

- Checked structural integrity of all **11 subtemplates** and the six profile families.
- Parsed all Cargo and template TOML manifests with Python `tomllib`, substituting the project-name placeholder offline.
- Parsed **23 GitHub Actions YAML files** for valid syntax.
- Checked `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, architecture docs, required files and canonical skill frontmatter.
- Verified every `.agents/skills` / `.claude/skills` pair is byte-identical.
- Executed **8 Python unit tests** successfully, including negative tests proving the agent checker rejects missing instructions and changed skills, and a regression test pinning the cargo-generate `--destination` contract.
- Confirmed both MIT and complete Apache-2.0 license text exist in all profiles.
- Reviewed relevant current public examples for `eframe 0.36`, `rmcp 3.10`, and `Leptos 0.8` to improve alignment.

## Completed integration verification

A real toolchain generated one project per profile in a scratch directory (removed afterwards), then ran `scripts/check_agents.py`, `cargo fmt --all -- --check`, compilation, and the generated unit tests:

| Profile | Generated | check_agents | fmt | Compile | Tests |
| --- | --- | --- | --- | --- | --- |
| library | yes | yes | yes | `cargo check --workspace --all-targets` | 4 passed |
| cli | yes | yes | yes | `cargo check --workspace --all-targets` | 3 passed |
| desktop-egui | yes | yes | yes | `cargo check --workspace --all-targets` | 1 passed |
| desktop-egui-web | yes | yes | yes | `cargo check --workspace --all-targets` | 1 passed |
| service-axum | yes | yes | yes | `cargo check --workspace --all-targets` | 1 passed |
| mcp-stdio | yes | yes | yes | `cargo check --workspace --all-targets` | 1 passed |
| mcp-http | yes | yes | yes | `cargo check --workspace --all-targets` | 1 passed |
| mcp-client | yes | yes | yes | `cargo check --workspace --all-targets` | 0 passed |
| web-leptos-csr | yes | yes | yes | `cargo check --target wasm32-unknown-unknown` | — |
| web-leptos-fullstack | yes | yes | yes | `cargo check --workspace --all-targets` | 0 passed |
| web-dioxus | yes | yes | yes | `cargo check --target wasm32-unknown-unknown` | — |

Result: **11 PASS, 0 FAIL, 0 SKIP.** No `{{project-name}}` placeholder survived generation.

`python3 tools/verify_templates.py --generate` also passes for all 11 profiles. Before the fix it failed for every profile in about three seconds, because cargo-generate 0.26 treats `--destination` as a parent directory that must already exist; the invocation and the misleading comment are corrected and pinned by `tests/test_templates.py::test_generate_passes_existing_parent_destination`.

Five profiles failed compilation before these fixes and now pass: `mcp-stdio` and `mcp-http` (missing `schemars` dependency for the `JsonSchema` derive), `mcp-client` (missing `rmcp::ServiceExt`), `web-leptos-fullstack` (`leptos::ssr::render_to_string` removed in leptos 0.8) and `web-dioxus` (missing `macro`/`signals`/`launch` features).

## Not covered by this verification

- GUI runners were not executed: no `cargo run -p desktop` session with a display server and no Trunk/Dioxus browser smoke test.
- No live MCP Inspector handshake: the MCP profiles compile and pass their unit tests, but no `initialize`/`list_tools`/`call_tool` exchange was performed against a running server.
- `cargo audit` and the per-profile GitHub Actions workflows have not run on a hosted repository.

## Addendum — Egui Visual Forge integration (2026-11-14)

- Installed the five MIT `egui-*` reconstruction skills in canonical `desktop-egui` `.agents/skills`, exposed through the `.claude/skills` symlink; `scripts/check_agents.py` reports **14 validated skills**.
- Vendored the forge Python tools, scene-format docs, `requirements.txt`, a standalone eframe 0.37 example and `tests/test_tools.py`.
- `python3 tools/validate_toolkit.py`: **PASS**. `python3 -m unittest discover -s tests -v`: **13/13 PASS** (Pillow/NumPy/PyYAML venv).
- Generated `desktop-egui` with cargo-generate 0.26.0, then `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo test --workspace` (1 passed) and `cargo clippy --workspace --all-targets -- -D warnings`: **PASS** with Rust 1.100.0.
- `examples/eframe-starter` (standalone eframe/egui/egui_kittest 0.37.0): `cargo check --all-targets` **PASS**; `cargo test` **1 passed**.
- Not performed: real display-server captures or pixel-fidelity claims against a user reference. The profile pins eframe/egui **0.36** while the bundled example targets **0.37**; this mismatch is documented in the generated `AGENTS.md`.

## Addendum — Claude skills symlink (2026-11-14)

- Replaced the duplicated `.claude/skills` trees in all 11 templates with relative symlinks to canonical `.agents/skills`.
- `python3 tools/verify_templates.py --generate` generated all 11 profiles and ran each generated `scripts/check_agents.py`; every project recreated the symlink and passed.
- Guard tests cover a replaced symlink, a wrong target, the marked copy fallback, and recreation of a missing link.
- cargo-generate 0.26 prints `[Skipping] Symbolic links not supported` during generation; the generated checker recreates the link. Platforms without symlink support get a marked copy fallback.

## How to finish the remaining checks

```sh
# from a generated project
cargo run -p desktop            # desktop-egui / desktop-egui-web
trunk serve                     # web-leptos-csr
dx serve                        # web-dioxus
cargo run                       # mcp-stdio / mcp-http, then connect an MCP Inspector client
```

See `docs/VALIDATION.md` and `docs/PROFILE_MATRIX.md`.
