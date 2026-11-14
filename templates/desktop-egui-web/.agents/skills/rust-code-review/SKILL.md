---
name: rust-code-review
description: Review Rust changes for requirement compliance, safety and maintainability.
---

# rust-code-review

## When to use

Use when the task is clearly about: review rust changes for requirement compliance, safety and maintainability.
Do not force this skill into unrelated tasks.

## Procedure

Read diff and test output. Do two passes: (1) behavior and requirements, (2) Rust safety, ownership, API compatibility, dependencies and quality. Prioritize concrete defects with path and line; distinguish blockers, improvements and optional notes. Do not repeat clippy warnings as independent discoveries. Report unverified assumptions and propose minimal fixes.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
