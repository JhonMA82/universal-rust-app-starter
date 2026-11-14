# {{project-name}}

Generated with **Universal Rust App Starter** — profile `service-axum`.

Tokio + Axum HTTP API with /health.

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
cargo run
curl http://127.0.0.1:3000/health
```
`APP_BIND` defaults to localhost. Public exposure needs auth, TLS ingress and explicit review.
