# Engineering toolkit

Run `cargo xtask fast` for formatting, architecture and compilation, or
`cargo xtask verify` to include tests and Clippy. Reports are JSON files under
`target/starter-evidence`. Mark domain crates with
`package.metadata.starter.layer = "domain"`.

`cargo xtask security` delegates to cargo-audit and `coverage` delegates to
cargo-llvm-cov. Install the matching Cargo tool to run those checks. `ui-test`
runs the shell's functional tests. `inspect` compiles the development-only
inspection adapter. `bundle` runs the selected distribution module and forwards
its arguments. For example, `cargo xtask bundle --web-dist dist` packages a
browser build from `dist`.

Use `cargo xtask affected starter-core` to gate the graph and check workspace
packages that depend on a changed package, including transitive consumers.
The full `verify` check remains appropriate for release validation.
