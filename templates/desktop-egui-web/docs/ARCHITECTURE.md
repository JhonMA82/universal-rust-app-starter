# Architecture — desktop-egui-web

**Profile:** Shared egui UI for native desktop and WASM/Trunk.

The generated project is independent of the starter repository. The example domain is intentionally minimal: replace it with your application's own types and constraints. Do not create more crates unless a genuine compilation/dependency boundary exists.

## Change policy

- Keep platform/environment concerns at the entrypoint.
- Keep business logic testable without network or GUI where possible.
- Edit only the components your project actually uses.
- Never commit credentials or blindly run generated shell commands.
- Review and adapt the current framework versions before production release.

## References

- Rust Book: https://doc.rust-lang.org/book/
- Cargo Book: https://doc.rust-lang.org/cargo/
- Agent Skills: https://agentskills.io/
