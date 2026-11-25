# {{project-name}}

Generated with **Universal Rust App Starter** — profile `desktop-egui-web`.

Shared egui UI for native desktop and WASM/Trunk.

## Requirements

- Rust stable (`rustup`, `cargo`, Rust 2024 edition)
- Python 3.11+ for the offline agent-readiness check
- Profile-specific tooling only when relevant (see below)

## Quick start

```sh
cargo check --workspace --all-targets
cargo test --workspace
python3 scripts/check_agents.py
```

See `docs/ARCHITECTURE.md`, `AGENTS.md`, and `docs/SKILLS.md`. CI runs in GitHub Actions and includes dependency advisory checks on a weekly schedule. Replace copyright holders in `LICENSE-MIT` and add a repository URL before distribution. For an application, generate and commit `Cargo.lock` before release (`cargo generate-lockfile`).

## Profile-specific startup

See below; adapt the starter example rather than building your production domain into it.

```sh
cargo run -p desktop
cargo test --workspace
```
Prefer modifying `crates/app` rather than platform runners for shared UI behavior. Linux needs Wayland or X11 libraries.

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve
trunk build --release
```
Desktop and Web share `crates/app`. The native runner uses wgpu, and the browser runner uses egui Glow with WebGL. Trunk serves on `http://127.0.0.1:8080`.
