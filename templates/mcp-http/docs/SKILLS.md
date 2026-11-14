# Installed skills

Canonical skills live in `.agents/skills`; `.claude/skills` is an identical mirror for clients that expect that path. Do not edit either set independently. The file `scripts/check_agents.py` enforces parity.

Installed (11): `idiomatic-rust`, `ownership-not-clone`, `rust-errors`, `rust-api-design`, `rust-testing`, `rust-code-review`, `rust-supply-chain`, `rust-docs`, `async-rust`, `rust-web-security`, `mcp-protocol-validation`.

These are locally authored, concise starter skills, **informed by** (not verbatim copies of) `rewrite-rs/skills` (BSD-3-Clause) and `full-stack-skills/rust-skills` (Apache-2.0). Review upstream references before extending or importing additional third-party code. Skills run no network operations by themselves.
