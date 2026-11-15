# Fix generated projects: 5 broken profiles + generate verifier

Repair the five profiles whose generated projects fail `cargo check`, and the
`tools/verify_templates.py --generate` bug that hid the failure in CI.

## Specs

S1 — mcp-stdio and mcp-http generated projects must pass `cargo check --workspace --all-targets`.
Smoke evidence (`.test-generated/summary.txt`): `FAIL mcp-stdio: cargo check --workspace --all-targets`, same for `mcp-http`.
Cause: `#[derive(rmcp::schemars::JsonSchema)]` emits `schemars::` paths while `schemars` is not a declared dependency.

S2 — mcp-client must pass `cargo check --workspace --all-targets`.
Cause: `src/main.rs` calls `ClientConfig::default().serve()` without `use rmcp::ServiceExt;` (it imports the unused `ClientServiceExt`).

S3 — web-leptos-fullstack must pass `cargo check --workspace --all-targets`.
Cause: `src/main.rs` calls `leptos::ssr::render_to_string`, absent in leptos 0.8.

S4 — web-dioxus must pass `cargo check --target wasm32-unknown-unknown`.
Cause: `dioxus 0.7` with `default-features = false, features = ["web"]` lacks `launch`/`macro`/`signals`.

S5 — `tools/verify_templates.py --generate` must work against cargo-generate 0.26: `--destination <PATH>` names a parent directory that must already exist and the project is created at `<PATH>/<name>`. Today the tool passes a nonexistent path, so all 11 profiles fail in ~3 s and the CI `generate` job never detected a broken template.

S6 — `AGENTS.md` verification commands stay valid, and any command or message the change touches is updated in its help text and docs.

## Tasks

| ID | Specs | Route | Commit |
| --- | --- | --- | --- |
| T1 | S1 | inline | uncommitted |
| T2 | S2 | inline | uncommitted |
| T3 | S3 | inline | uncommitted |
| T4 | S4 | inline | uncommitted |
| T5 | S5, S6 | inline | uncommitted |
| T6 | S1-S6 | inline | uncommitted |

- T1 — Fix mcp-stdio and mcp-http JsonSchema dependency (S1).
- T2 — Fix mcp-client service-extension import (S2).
- T3 — Fix web-leptos-fullstack SSR render call (S3).
- T4 — Fix web-dioxus required features (S4).
- T5 — Fix `--generate` destination handling + regression test + docs (S5, S6).
- T6 — Re-run the python suites and the 11-profile smoke run (S1-S6).

Status: T1-T6 complete and verified. Work is uncommitted.

## Log

- L1 (verbatim, 2026-11-14 16:35): "verifica que funcione correctamente el template genera una de cada una"
- L2 (verbatim, 2026-11-14 18:45): "continua con los arreglos"
- L3 (verbatim, 2026-11-14 21:01): "continua de donde te quedaste"
- L4 (evidence): smoke run of all 11 profiles — 6 PASS, 5 FAIL, verifier `--generate` broken with cargo-generate 0.26.
- L5 (evidence, 2026-11-14): after the fixes, `python3 tools/verify_templates.py --generate` passes for all 11 profiles, `python3 -m unittest discover -s tests` passes 8 tests, and the per-profile smoke run reports `PASS=11 FAIL=0 SKIP=0` (library 4 tests, cli 3, desktop-egui 1, desktop-egui-web 1, service-axum 1, mcp-stdio 1, mcp-http 1, mcp-client 0, leptos-fullstack 0, leptos-csr and dioxus compile to wasm32). Docs updated: CHANGELOG.md, VALIDATION-REPORT.md, docs/VALIDATION.md.
