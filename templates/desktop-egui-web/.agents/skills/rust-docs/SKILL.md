---
name: rust-docs
description: Maintain Rustdoc, examples and architecture documentation in sync with code.
---

# rust-docs

## When to use

Use when the task is clearly about: maintain rustdoc, examples and architecture documentation in sync with code.
Do not force this skill into unrelated tasks.

## Procedure

Check actual source and tests before documenting. Update public item documentation including errors, panics, units, safety and examples where relevant. Validate with `cargo doc --no-deps` and doctests. Prefer links to one canonical architecture document over duplicating whole explanations. For template work, verify that commands and file paths in `AGENTS.md` remain valid.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
