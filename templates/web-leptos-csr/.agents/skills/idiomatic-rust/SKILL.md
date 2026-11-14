---
name: idiomatic-rust
description: Write idiomatic Rust using standard traits, patterns and simple readable control flow.
---

# idiomatic-rust

## When to use

Use when the task is clearly about: write idiomatic rust using standard traits, patterns and simple readable control flow.
Do not force this skill into unrelated tasks.

## Procedure

Before implementation, inspect the repository MSRV, lint policy and public interfaces. Prefer expressive `match`, `?`, `From` / `TryFrom`, newtypes when they encode invariants, and iterators only when clearer than loops. Avoid needless abstractions and large generic frameworks. Use `cargo fmt` and `cargo clippy` after changes. Deliver concrete file references and note any public API impact.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
