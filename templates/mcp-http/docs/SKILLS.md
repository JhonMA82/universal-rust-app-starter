# Installed skills

Canonical skills live in `.agents/skills`; `.claude/skills` is a relative symlink to that directory for clients that expect that path. Do not replace it with a copy. `scripts/check_agents.py` enforces the link and recreates it after generation when cargo-generate omits symlinks.

Installed (11): `idiomatic-rust`, `ownership-not-clone`, `rust-errors`, `rust-api-design`, `rust-testing`, `rust-code-review`, `rust-supply-chain`, `rust-docs`, `async-rust`, `rust-web-security`, `mcp-protocol-validation`.

These are locally authored, concise starter skills, **informed by** (not verbatim copies of) `rewrite-rs/skills` (BSD-3-Clause) and `full-stack-skills/rust-skills` (Apache-2.0). Review upstream references before extending or importing additional third-party code. Skills run no network operations by themselves.
