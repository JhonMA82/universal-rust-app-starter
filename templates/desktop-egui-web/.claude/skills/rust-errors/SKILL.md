---
name: rust-errors
description: Design structured, actionable Rust errors and correct boundary propagation.
---

# rust-errors

## When to use

Use when the task is clearly about: design structured, actionable rust errors and correct boundary propagation.
Do not force this skill into unrelated tasks.

## Procedure

Library public APIs return typed errors (`thiserror` when useful); applications may add context via `anyhow`. Do not `unwrap` on caller-controlled input. Report precise invalid input and sources without leaking secrets. Keep recovery at appropriate abstraction boundaries. Add tests for failure modes and check documented error behavior.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
