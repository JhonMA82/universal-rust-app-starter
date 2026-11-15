# Validation status (v1.0.0 + unreleased fixes)

Static audits prove TOML structure, placeholders, instruction completeness and file parity offline. They cannot prove crate APIs, framework integration, native linking, browser execution or MCP handshake. On 2026-11-14 all eleven profiles were also generated and compiled with a real toolchain (Rust 1.100.0, cargo-generate 0.26.0, `wasm32-unknown-unknown`): 11 PASS, 0 FAIL, 0 SKIP — see `VALIDATION-REPORT.md`. GUI runners, browser builds and live MCP handshakes are still unverified.

The `desktop-egui` profile additionally carries the Egui Visual Forge Python toolkit (13 tests) and a standalone eframe 0.37 example; both were validated separately. This does not constitute a pixel-fidelity claim for any real screenshot.

## Integration pass status

1. ~~Install current Rust stable, supported native GUI dependencies, cargo-generate, trunk, and Dioxus CLI where applicable.~~ Done for Rust stable and cargo-generate; `trunk`/`dx` were not needed for the compile pass.
2. ~~Run `python3 tools/verify_templates.py --generate --cargo-check` (or the equivalent per-profile smoke run). Inspect any failures without weakening checks.~~ Done: fixed `--generate` destination handling, then generated, formatted, compiled and unit-tested one project per profile.
3. **Pending** — for egui desktop, run `cargo run -p desktop` on Linux/macOS/Windows with a display server. Visual inspect.
4. **Pending** — for `desktop-egui-web` and Leptos CSR, run Trunk with browser smoke tests.
5. **Pending** — for `web-dioxus`, run dx serve on browser.
6. **Pending** — for MCP, launch stdio and HTTP servers and use a real MCP Inspector client for initialize/list/call.
7. **Pending** — check `cargo audit` and GitHub Actions workflows on a generated repository.

## Known limits

- `web-leptos-fullstack` is intentionally SSR-only, not a hydrated full-stack auth/database starter. It renders with `RenderHtml::to_html()`.
- No automatic GitHub publishing or deployment is configured.
- No end-to-end UI snapshots or live MCP handshakes can be attested from static tests or from the compile pass.
- Dependency ranges are current-intent examples, not a replacement for a resolved `Cargo.lock`.
- The complete Apache-2.0 and MIT text is bundled; replace copyright holders when you create your own project.
