# Sources, licenses and curation

References were inspected for architectural patterns; all starter scripts and bundled skills are original text written for this package. No code from the PolyForm Noncommercial egui template is copied.

- https://github.com/link-foundation/rust-ai-driven-development-pipeline-template — Unlicense — CI and verification patterns.
- https://github.com/Spodeian/efficient-egui-template — **PolyForm Noncommercial-1.0.0** — architecture reference only, no source reused.
- https://github.com/gauravsonii/rust_template — MIT OR Apache-2.0 — cargo-generate workflow design reference.
- https://github.com/rewrite-rs/skills — BSD-3-Clause — Rust skill topic/quality reference (not copied).
- https://github.com/full-stack-skills/rust-skills — Apache-2.0 — additional skill topic/quality reference (not copied).
- `egui-visual-forge` v1.1 (user-supplied archive) — MIT — vendored into `templates/desktop-egui` as five skills, Python comparison tools, scene-format docs and an eframe example. No noncommercial code included; see the generated `LICENSE-EGUI-VISUAL-FORGE`.
- https://agentskills.io/ — SKILL.md format.
- https://github.com/modelcontextprotocol/rust-sdk — rmcp official examples and API reference.
- https://github.com/emilk/eframe_template — egui/eframe reference.
- https://github.com/leptos-rs/start-trunk and https://github.com/leptos-rs/start-axum — Leptos references.
- https://github.com/DioxusLabs/dioxus-template — Dioxus reference.

This directory includes no third-party proprietary assets, fonts, logos or icons.

## v2 patterns and dependencies

- CraftKit at `fd555e1f5866` (MIT or Apache-2.0), architecture and agent playbook.
  Inspiration for the operation registry, schemas and verification gates.
- ArtCraft shared craft-libs at `249a41bd9306` (MIT or Apache-2.0 for its own
  workspace), UI, frame and quit crates. Inspiration for shell and close handling.
- SolidCraft at `88617b40ae4c`, architecture, layers, workspace/viewport inventory
  and LICENSE-BRAND. Inspiration for neutral layouts and rendering layers. No
  code or branded assets adopted.
- OpenCADStudio at `e376c6b0d15a` (GPL-3.0), automation and plugin docs.
  Conceptual reference for optimistic revisions and verified persistence.
- Official egui/eframe/egui-wgpu/egui_inspection/egui_kittest 0.37, egui_dock 0.22,
  egui_plot 0.38 and wgpu 30 from crates.io. Their original notices apply.
  Default egui fonts have additional notices copied by the packaging helper.
- Official rmcp 3.11 for MCP. Axum 0.8 for HTTP, petgraph 0.8 for graph algorithms
  and uom 0.38 for dimensions. Cargo Generate 0.26.0 renders base templates,
  and tomli-w 1.2.0 emits composed TOML.

The modular implementation is original. No application source or restricted
assets from the references above are included. Review the resolved dependency
inventory before distributing generated apps.
