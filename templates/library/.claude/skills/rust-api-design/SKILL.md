---
name: rust-api-design
description: Design stable, ergonomic Rust public APIs and compatibility contracts.
---

# rust-api-design

## When to use

Use when the task is clearly about: design stable, ergonomic rust public apis and compatibility contracts.
Do not force this skill into unrelated tasks.

## Procedure

Before editing exported items, examine public callers and existing guarantees. Use idiomatic names, predictable ownership and trait bounds; minimize `pub` surface. Avoid accidental panics, flag booleans and stringly typed states; prefer enums/newtypes when beneficial. Consider SemVer and deprecation rather than unannounced breaking changes. Add public-facing integration tests and rustdoc examples.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
