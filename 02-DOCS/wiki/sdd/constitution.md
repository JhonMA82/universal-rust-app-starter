---
type: constitution
title: universal-rust-app-starter — Constitution
description: Non-negotiables for the starter and its generated projects.
tags: [sdd, constitution]
timestamp: 2026-11-25T06:12:04Z
topic: sdd
status: ratified
version: v1.0.0
ratified: 2026-11-25
---

# universal-rust-app-starter — Constitution

## Principles

1. **Stack canon.** Templates target Rust (MSRV pinned by `rust-toolchain.toml` per profile); generator/tools/tests run on Python ≥3.11 with `cargo-generate` 0.26.0. No top-level `Cargo.toml`; the starter is a generator repository. Enforced by `tools/verify_templates.py`.
2. **Autonomy of generated projects.** Every generated project builds and tests without the starter. No unpinned Git dependencies (40-hex `rev`), no external Cargo `path` dependencies, no symlinks escaping the project root. Enforced by `verify_autonomy` in `generator/compose.py`.
3. **Module integrity.** A module's `files` list must exactly match its `files/` payload; manifests merge structurally without conflict. Enforced by `catalog()` and `check_files()` in `generator/compose.py`.
4. **Compatibility completeness.** Every module must support all entrypoints shipped by its profile; composition aborts on profile/target incompatibility. Enforced by `resolve()` in `generator/compose.py`.
5. **Quality gate (starter).** `python3 tools/verify_templates.py`, `python3 -m unittest discover -s tests`, `python3 tools/check_docs.py` must pass. Rust generated projects must pass `cargo fmt --check`, `cargo check --all-targets`, `cargo test`, `cargo clippy -D warnings`, and `python3 scripts/check_agents.py`.
6. **Agent readiness.** Every generated project ships `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `docs/ARCHITECTURE.md`, `.agents/skills/*/SKILL.md`, and a `.claude/skills` symlink resolving to `.agents/skills`. Enforced by `scripts/check_agents.py`.
7. **Licensing.** Starter code is MIT OR Apache-2.0. No GPL/brand-restricted code copied; Git dependencies pinned and documented. Enforced by module manifests (`license` field) and `verify_autonomy`.
8. **Documentation consistency.** Generated `docs/COMPOSITION.md`, `AGENTS.md`, `README.md` must reflect the actual composition; `check_docs.py` audits links and descriptors. No capability announced that is not installed.
9. **No duplicate infrastructure.** Prefer reusing mature libraries; own crates must add integration/adaptation/value. Justify any new own implementation.
10. **Git authorship.** Commits and PRs authored by the responsible human (configured git user). No `Co-Authored-By` AI, no generated-with footer. Enforced at `ship`.
11. **Decisions logged.** Significant decisions appended to `02-DOCS/wiki/harness/decisions.md` with date, options, rationale.

## Definition of Done (merge bar)

- [ ] `python3 tools/verify_templates.py` passes (offline).
- [ ] `python3 -m unittest discover -s tests` passes.
- [ ] `python3 tools/check_docs.py` passes.
- [ ] Generated projects pass `cargo fmt --check`, `cargo check --all-targets`, `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- [ ] `scripts/check_agents.py` passes in generated projects.
- [ ] No unpinned Git deps, no external paths, no escaping symlinks.
- [ ] Documentation matches manifests and composition.

## Amendment log (append-only)

| Date | Version | Change | Why |
|------|---------|--------|-----|
| 2026-11-25 | v1.0.0 | Ratified from draft. | Base stage for the usability-layer program. |
