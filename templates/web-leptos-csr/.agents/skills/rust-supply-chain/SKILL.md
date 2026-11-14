---
name: rust-supply-chain
description: Assess Rust advisories, dependency sources, licenses and upgrades.
---

# rust-supply-chain

## When to use

Use when the task is clearly about: assess rust advisories, dependency sources, licenses and upgrades.
Do not force this skill into unrelated tasks.

## Procedure

Inspect `Cargo.lock`, dependency tree and manifest before changing versions. Use `cargo audit` or `cargo deny` according to existing repository policy; avoid redundant scanners when they cover the same check. Investigate whether advisories affect reachable use and the fixed-version MSRV. Never casually remove a vulnerable dependency without checking reverse dependencies. Record upgrade/mitigation choices and reasons; never fetch or execute arbitrary scripts without review.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
