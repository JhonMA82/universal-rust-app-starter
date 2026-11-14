# Validation status (v1.0.0)

This is a starter implementation, **not a validated compiled release** until Rust and each framework toolchain are executed. Python static audits can verify TOML structure, placeholders, instruction completeness and file parity offline; they cannot prove crate APIs, framework integration, native linking, browser execution or MCP handshake. The build environment used for initial packaging did not include `rustc`, `cargo`, `cargo-generate`, `trunk` or `dx`.

## Required real integration pass (after extracting)

1. Install current Rust stable, supported native GUI dependencies, cargo-generate, trunk, and Dioxus CLI where applicable.
2. Run `python3 tools/verify_templates.py --generate --cargo-check`. Inspect any failures without weakening checks.
3. For egui desktop, run `cargo run -p desktop` on Linux/macOS/Windows with a display server. Visual inspect.
4. For `desktop-egui-web` and Leptos CSR, run Trunk with browser smoke tests.
5. For `web-dioxus`, run dx serve on browser.
6. For MCP, launch stdio and HTTP servers and use a real MCP Inspector client for initialize/list/call.
7. Check `cargo audit` and GitHub Actions workflows on a generated repository.

## Known limits

- `web-leptos-fullstack` is intentionally SSR-only, not a hydrated full-stack auth/database starter.
- No automatic GitHub publishing or deployment is configured.
- No end-to-end UI snapshots or live MCP handshakes can be attested from static tests.
- Dependency ranges are current-intent examples, not a replacement for a resolved `Cargo.lock`.
- The complete Apache-2.0 and MIT text is bundled; replace copyright holders when you create your own project.
