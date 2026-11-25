# Changelog

## Unreleased stabilization

- Manage the `desktop-egui` Visual Forge Python tooling with `uv`:
  `pyproject.toml` replaces `requirements.txt`, and the egui skills and profile
  docs use `uv sync` / `uv run`.
- Reject missing, ambiguous or ineffective profile adapter edits.
- Validate Cargo features and document only the composed entrypoints.
- Verify Minimal and Application across all profiles and drive Leptos, Dioxus
  and hybrid egui interactions in Chromium.
- Use the egui Glow backend on the web to support WebGL presentation while retaining wgpu on desktop.
- Supply the X11 keyboard runtime for native window CI and compare native
  captures with Visual Forge.
- Reconcile duplicated floating/docked panels and evaluate command palette
  availability with the entered parameters.

- Route ordinary presets and references through the same generated entrypoints.
- Register dynamic panel instances and dispatch operations from the command palette.
- Require explicit snapshot isolation and a registered document-save completion.
- Enable opt-in authenticated live MCP and verify discovery and disconnection.
- Validate documentation and expand native, browser and composition checks.

## Unreleased

- Add the optional modular v2 candidate: `generator/compose.py` composes the existing Cargo Generate profiles with 17 versioned modules and four presets (`minimal`, `application`, `workbench`, `engineering`), plus three autonomous reference projects (Document Workbench, Data Explorer, Engineering Workbench).
- Add `validate-modules.yml` to generate, test and lint the composed references in CI; distribution stays opt-in through the `distribution` module and its manual workflow.
- Document the modular workflow in `docs/GENERATOR.md`, `docs/MODULE_CATALOG.md`, `docs/V2_REPORT.md`, `docs/MIGRATION_V2.md`, `docs/SOURCES.md`, `docs/KNOWN_LIMITATIONS.md` and `docs/VALIDATION.md`.
- Install the MIT-licensed Egui Visual Forge v1.1 toolkit in `desktop-egui`: five `egui-*` skills (canonical in `.agents/skills`, exposed through the Claude symlink), Python scene/comparison tools, scene-format docs, an eframe starter example and toolkit tests.
- Harden the `desktop-egui` generator config to ignore Python caches and skip Liquid parsing of Python/requirements files.
- Replace duplicated `.claude/skills` copies with relative symlinks to canonical `.agents/skills`; generated projects recreate the link in `scripts/check_agents.py`.
- Stop `cargo-generate`'s unsupported-symlink warning: templates no longer ship `.claude/skills`; `scripts/check_agents.py` creates the link after generation and now repairs an empty leftover directory as well as a missing one.
- Fix generated MCP profiles: declare the `schemars` dependency that `rmcp::schemars::JsonSchema` derive requires in `mcp-stdio` and `mcp-http`.
- Fix `mcp-client`: import `rmcp::ServiceExt` so `serve`, `list_tools` and `cancel` resolve.
- Fix `web-leptos-fullstack`: render SSR HTML with `RenderHtml::to_html()` instead of the removed `leptos::ssr::render_to_string`.
- Fix `web-dioxus`: enable the `minimal` feature set (`macro`, `html`, `signals`, `hooks`, `launch`) alongside `web`.
- Fix `tools/verify_templates.py --generate`: cargo-generate 0.26 treats `--destination` as an existing parent directory, so the checker no longer fails for every profile before generating.
- Add a regression test that pins the cargo-generate destination contract.

## 1.0.0 — 2026-11-14

- Introduces eleven independent Rust project subtemplates across six profile families.
- Adds native/wasm/MCP sample entrypoints, tests and profile-specific README commands.
- Includes eight baseline agent skills plus specialist skills; canonical AGENTS.md and Claude/Gemini adapters.
- Includes offline structural checks and optional real cargo-generate/cargo integration checks.
- Includes CI for the starter and for each generated project, with a weekly dependency audit workflow.
- Bundles MIT and Apache-2.0 license texts and source licensing notes.
