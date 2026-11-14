# {{project-name}}

Generated with **Universal Rust App Starter** — profile `mcp-stdio`.

rmcp stdio server with an echo tool.

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
# In a separate terminal, use an MCP host or the MCP Inspector CLI:
# cargo build
# npx @modelcontextprotocol/inspector --cli ./target/debug/{{project-name}} --method tools/list
```
All stdout is reserved for protocol messages; use stderr for logs.
