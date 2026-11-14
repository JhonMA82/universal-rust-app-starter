---
name: wasm-compatibility
description: Keep Rust WASM applications portable and verify browser constraints.
---

# wasm-compatibility

## When to use

Use when the task is clearly about: keep rust wasm applications portable and verify browser constraints.
Do not force this skill into unrelated tasks.

## Procedure

Separate native-only facilities behind `cfg`; avoid blocking threads and filesystem assumptions in browser code. Use `wasm-bindgen` and platform storage adapters deliberately. Test a wasm32 compile and browser smoke test; include CSP, offline caching and storage failure tests when PWA is enabled. Do not infer browser support from native compilation.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
