---
name: async-rust
description: Design Tokio async Rust for cancellation, bounded concurrency and reliability.
---

# async-rust

## When to use

Use when the task is clearly about: design tokio async rust for cancellation, bounded concurrency and reliability.
Do not force this skill into unrelated tasks.

## Procedure

Avoid blocking operations inside async tasks; use spawn_blocking only when necessary. Plan ownership and shutdown, bound queues and parallelism, and define timeouts at request boundaries. Prefer structured cancellation to abandoned detached tasks. Test shutdown, retry boundaries and failure propagation with deterministic time controls where possible.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
