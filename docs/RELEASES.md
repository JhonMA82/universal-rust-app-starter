# Release & distribution policy

The starter intentionally does **not** publish applications, crates or containers on creation. This avoids accidental releases and credential use. CI runs on PRs and main.

Recommended post-generation actions by profile:

| Profile | Suggested distribution route |
|---|---|
| Library | `cargo package --list`, `cargo publish --dry-run`, then approved `cargo publish` |
| CLI | `cargo-dist`/`dist` bootstrap and test a matrix of native artifacts |
| Desktop | `cargo-dist` or platform packaging only after OS smoke tests |
| Service / MCP HTTP | Container or service packaging after auth/security review |
| MCP stdio/client | Native binary release with protocol smoke tests |
| Web | Trunk/Dioxus build, security headers and hosting validation |

Release checks should be attached to immutable tag revisions. Keep GitHub Actions tokens read-only during tests; grant `contents: write` only to specific approved release jobs. Separate dependency upgrades from releases and require an explicit user action to create a tag or deploy.

This v1 ships a real CI pipeline and a release **procedure**, not preconfigured automatic publishing or universal packaging. Once a generated project chooses its destinations, use the maintained `cargo-dist` initialization workflow rather than copying untested release YAML.
