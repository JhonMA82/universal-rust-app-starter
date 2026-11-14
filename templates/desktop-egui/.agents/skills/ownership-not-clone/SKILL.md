---
name: ownership-not-clone
description: Resolve borrow-checker issues through ownership design rather than unnecessary clones.
---

# ownership-not-clone

## When to use

Use when the task is clearly about: resolve borrow-checker issues through ownership design rather than unnecessary clones.
Do not force this skill into unrelated tasks.

## Procedure

Identify data ownership and required lifetimes before changing code. Prefer borrowing (`&str`, slices, `AsRef<Path>`), splitting disjoint mutable borrows, moving values deliberately, or `Cow` when both owned and borrowed values are needed. Do not add `clone`, `Rc<RefCell<_>>`, or `Arc<Mutex<_>>` solely to silence errors. Explain the tradeoff and test paths that mutate or share state.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
