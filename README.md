# Universal Rust App Starter · v1.0.0

**A practical, multi-profile Rust application generator with agent-ready defaults.** This repository contains standalone subtemplates, not a runtime dependency for generated apps. Inspired by Rust AI-Driven Pipeline, efficient-egui-template (architecture only; PolyForm Noncommercial; no code copied), and rust_template (generation pattern only).

## Profiles

| Profile | Description | Run after generating |
|---|---|---|
| `library` | Public Rust crate | `cargo test` |
| `cli` | Clap CLI | `cargo run -- greet Rust` |
| `desktop-egui` | Native egui | `cargo run -p desktop` |
| `desktop-egui-web` | Native + WASM egui | `cargo run -p desktop`, `trunk serve` |
| `service-axum` | HTTP API | `cargo run`, then `/health` |
| `mcp-stdio` | MCP stdio server | `cargo run` (MCP host required) |
| `mcp-http` | MCP Streamable HTTP server | `cargo run` (localhost:8000/mcp) |
| `mcp-client` | MCP HTTP client | `cargo run` (needs an MCP server) |
| `web-leptos-csr` | Web/WASM CSR | `trunk serve` |
| `web-leptos-fullstack` | Minimal SSR Leptos + Axum | `cargo run` |
| `web-dioxus` | Web/WASM Dioxus | `dx serve --platform web` |

## Create a new project

Install Rust and [cargo-generate](https://cargo-generate.github.io/cargo-generate/):

```sh
cargo install cargo-generate --locked
cargo generate --path templates/cli --name my-tool
cd my-tool
cargo test
```

To generate from GitHub after publishing this starter:

```sh
cargo generate --git https://github.com/YOUR-ACCOUNT/universal-rust-app-starter --template-path templates/desktop-egui --name my-desktop-app
```

For a locally unpacked source, execute `cargo generate --path templates/<profile> --name <app-name>` in a parent directory other than the template itself. See each profile README for dependencies and commands.

`desktop-egui` also ships the vendored **Egui Visual Forge** toolkit (MIT): five reconstruction skills, Python comparison tools and a scene-format contract. Python dependencies are optional for normal builds; see the generated `docs/EGUI_VISUAL_FORGE.md`.

## Publish this starter to your GitHub account

The repository includes a local publishing helper using the official [GitHub CLI](https://cli.github.com/). **It creates a new repository, pushes all tracked files, and tries to enable the GitHub template flag**. Visibility defaults to private. Nothing is uploaded until you invoke it.

```sh
gh auth login
bash tools/publish-to-github.sh universal-rust-app-starter private
```

Use `public` as the second argument if you want the template to be publicly accessible. The helper refuses to overwrite an existing `origin`, never force-pushes, and does not store GitHub credentials. You must have permission to create repositories on the authenticated account.

## Agent-first out of the box

- Canonical `AGENTS.md` with profile-accurate checks and explicit delivery criteria.
- `CLAUDE.md` and `GEMINI.md` as minimal adapters.
- Eight original curated Rust skills, and specialized skills depending on the selected profile.
- Canonical skills in `.agents/skills`; `.claude/skills` is a relative symlink for cross-client discovery.
- `python3 scripts/check_agents.py` checks canonical documents, skill frontmatter and the Claude symlink offline; it recreates the link after cargo-generate when needed.
- No hooks install agents, connect accounts, send data or grant permissions automatically.

Skills are written specifically for this starter, with design references to BSD-3-Clause `rewrite-rs/skills`, Apache-2.0 `full-stack-skills/rust-skills`, and Agent Skills specification. No noncommercial code is included. See `docs/SOURCES.md`.

## Quality & CI

Each generated project gets format, check/test, clippy (native profiles), agent-readiness CI and a weekly `cargo audit` security workflow. Browser builds and release publishing need corresponding platform dependencies; distribution and cloud deployment are **not enabled by default**. See `docs/VALIDATION.md` for what was and was not tested in this delivery. Release procedures are in `docs/RELEASES.md`.

## Template repository self-check

```sh
python3 tools/verify_templates.py
python3 -m unittest discover -s tests -v
```

With Rust/cargo-generate installed, enable actual project generation and compilation:

```sh
python3 tools/verify_templates.py --generate --cargo-check
```

## License

Starter-authored code and files: MIT OR Apache-2.0. `LICENSE-MIT` and the complete Apache-2.0 text are included. Third-party libraries keep their respective licenses; verify your dependency tree for your distribution. Not a production security certification.
