# Installed skills

Canonical skills live in `.agents/skills`. `scripts/check_agents.py` creates the `.claude/skills` relative symlink after generation because cargo-generate cannot copy symlinks; do not replace the link with a copy.

Installed (10): `idiomatic-rust`, `ownership-not-clone`, `rust-errors`, `rust-api-design`, `rust-testing`, `rust-code-review`, `rust-supply-chain`, `rust-docs`, `async-rust`, `rust-web-security`.

These are locally authored, concise starter skills, **informed by** (not verbatim copies of) `rewrite-rs/skills` (BSD-3-Clause) and `full-stack-skills/rust-skills` (Apache-2.0). Review upstream references before extending or importing additional third-party code. Skills run no network operations by themselves.
