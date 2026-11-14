---
name: rust-cli
description: Build reliable Rust command-line applications with stable I/O and exit behavior.
---

# rust-cli

## When to use

Use when the task is clearly about: build reliable rust command-line applications with stable i/o and exit behavior.
Do not force this skill into unrelated tasks.

## Procedure

Treat CLI options, stdout, stderr and exit codes as public contracts. Keep argument parsing and domain logic testable; handle invalid input and non-TTY execution. Use `clap` when appropriate, avoid `process::exit` deep in libraries, and test `--help` and at least one error path. Document examples that users can paste into a terminal.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
