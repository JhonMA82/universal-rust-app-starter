# {{project-name}}

Generated with **Universal Rust App Starter** — profile `web-leptos-csr`.

Leptos client-side WASM app with Trunk.

## Requirements

- Rust stable (`rustup`, `cargo`, Rust 2024 edition)
- Python 3.11+ for the offline agent-readiness check
- Profile-specific tooling only when relevant (see below)

## Quick start

```sh
cargo check --target wasm32-unknown-unknown
rustup target add wasm32-unknown-unknown
python3 scripts/check_agents.py
```

See `docs/ARCHITECTURE.md`, `AGENTS.md`, and `docs/SKILLS.md`. CI runs in GitHub Actions and includes dependency advisory checks on a weekly schedule. Replace copyright holders in `LICENSE-MIT` and add a repository URL before distribution. For an application, generate and commit `Cargo.lock` before release (`cargo generate-lockfile`).

## Profile-specific startup

See below; adapt the starter example rather than building your production domain into it.

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve
trunk build --release
```
CSR is a browser-targeted application; do not substitute native tests for browser validation.
