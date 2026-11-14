---
name: rust-semver
description: Review Rust API compatibility and releases.
---

# rust-semver

## When to use

Use when the task is clearly about: review rust api compatibility and releases.
Do not force this skill into unrelated tasks.

## Procedure

Compare public signatures and feature flags against the last published release. Distinguish compatible additive changes from breaking changes; follow pre-1.0 policy and document exceptions. Use `cargo-semver-checks` when available. Run crate packaging checks and inspect published files before tagging.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
