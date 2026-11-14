---
name: rust-testing
description: Build Rust tests that catch regressions rather than restating implementation.
---

# rust-testing

## When to use

Use when the task is clearly about: build rust tests that catch regressions rather than restating implementation.
Do not force this skill into unrelated tasks.

## Procedure

State the regression each test is intended to catch. Use unit tests for internal invariants, integration tests for public contracts, doctests for examples, and property/golden tests where valuable. Include failing inputs, boundaries and cancellation/error paths. Use independent expected values. Prefer deterministic fixtures over flaky timing or network dependencies; run tests and report exact commands and results.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
