# Architecture

The starter is a repository of **independent cargo-generate subtemplates**. No custom generator binary is needed in v1. Each profile contains its own Cargo.toml, source, agent rules, CI, and specialized skills. The `tools/` scripts audit the whole template repository, not generated apps.

## Boundaries

- `templates/library`: small crate; no forced workspace.
- `templates/cli`: single binary.
- `templates/desktop-*`: two-crate or three-crate workspace separating shared egui UI and platform runners.
- `templates/service-axum`: testable HTTP router and binary.
- `templates/mcp-*`: protocol-specific servers/clients using rmcp; stdout reserved for protocol on stdio.
- `templates/web-*`: CSR, minimalist server-side-rendered UI, or Dioxus web.

The web/fullstack profile is **SSR-only** in v1. It does not promise hydration/auth/db. Egui universal targets native and WASM, but no PWA/service worker is installed by default. Security tests and deployment are separate decisions. Distribution workflow choices are in `RELEASES.md`.

## Agent-first behavior

Every generated app receives eight general skills plus profile skills. Canonical content: `.agents/skills`; `.claude/skills` is a relative symlink to it. cargo-generate skips symlinks, so `scripts/check_agents.py` recreates the link after generation (with a copy fallback only where symlinks are unavailable) and fails on wrong targets. Provider adapters stay thin and never silently execute operations.

## Why no defaults for databases/auth/distribution

These vary too much between domains and can introduce remote dependencies and security assumptions. V1 favors a small base that compiles, with explicit optional extensions later.

## Modular workspaces

The [composer](GENERATOR.md) wraps the existing profiles with local modules.
Reference domains physically separate business logic from egui and transport
dependencies. The same typed operation registry handles desktop, CLI and
local MCP. An optional Axum router accepts the same request objects and
connects to the same application state. Document storage and external
effects are infrastructure boundaries. The viewport has distinct scene,
wgpu and egui crates. The data parser is separate from table presentation.
