# Universal Rust App Starter · v2.0.0 candidate

**A practical, multi-profile Rust application generator with agent-ready defaults.** This repository contains standalone subtemplates, not a runtime dependency for generated apps. Inspired by Rust AI-Driven Pipeline, efficient-egui-template (architecture only; PolyForm Noncommercial; no code copied), and rust_template (generation pattern only).

## Create and start developing

Install Rust, Cargo Generate 0.26 and uv. The interactive command shows
application types, eligible capabilities and a composition preview.

```sh
cargo install cargo-generate --version 0.26.0 --locked
uv run --with-requirements generator/requirements.txt python starter.py create
```

For scripts or agents, select a product and capabilities explicitly.

```sh
uv run --with-requirements generator/requirements.txt python starter.py create --product desktop-application --name my-app --capability automation --verify
cd my-app
cargo run -p desktop
```

Open the generated `docs/GETTING_STARTED.md` and `infrastructure.json`.
Desktop Application puts your operations in `crates/app/src/domain.rs` and
your views in `crates/app/src/view.rs`.

See [products and capabilities](docs/PRODUCT_GUIDE.md),
[generator internals](docs/GENERATOR.md), [reuse decisions](docs/REUSE_MATRIX.md),
[validation](docs/VALIDATION.md) and [migration](docs/MIGRATION_V2.md).

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

## Direct Cargo Generate

Install Rust and [cargo-generate](https://cargo-generate.github.io/cargo-generate/):

```sh
cargo install cargo-generate --locked
cargo generate --path templates/cli --name my-tool
cd my-tool
python3 scripts/check_agents.py
cargo test
```

To generate the original desktop profile directly from GitHub:

```sh
cargo generate --git https://github.com/JhonMA82/universal-rust-app-starter templates/desktop-egui --name my-desktop-app
cd my-desktop-app
python3 scripts/check_agents.py
```

`cargo-generate` cannot copy the `.claude/skills` symlink; the readiness check creates it after generation (copy fallback where symlinks are unavailable).

For a locally unpacked source, execute `cargo generate --path templates/<profile> --name <app-name>` in a parent directory other than the template itself. See each profile README for dependencies and commands.

`desktop-egui` also ships the vendored **Egui Visual Forge** toolkit (MIT): five reconstruction skills, Python comparison tools and a scene-format contract. Its Python tooling is a `uv` project (`pyproject.toml` in the template) and is optional for normal builds; see the generated `docs/EGUI_VISUAL_FORGE.md`.

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
- Canonical skills in `.agents/skills`; `.claude/skills` is a relative symlink for cross-client discovery, created by the readiness check.
- `python3 scripts/check_agents.py` checks canonical documents, skill frontmatter and the Claude symlink offline; run it after generation.
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
