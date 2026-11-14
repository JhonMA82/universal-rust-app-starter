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

Every generated app receives eight general skills plus profile skills. Canonical content: `.agents/skills`, mirrored at `.claude/skills`, no symlink dependencies. Run `scripts/check_agents.py`; it fails if mirrors drift. Provider adapters stay thin and never silently execute operations.

## Why no defaults for databases/auth/distribution

These vary too much between domains and can introduce remote dependencies and security assumptions. V1 favors a small base that compiles, with explicit optional extensions later.
