# Products and capabilities

A profile chooses a technology and its entrypoints. A preset composes a
baseline. A product provides a ready-to-use preset and a configured view.
Extra capabilities are resolved using the same composer as
[the earlier module interface](GENERATOR.md).

| Product | Starting workspace | Extend first |
| --- | --- | --- |
| `desktop-application` | Workspace settings, docking, component gallery, palette and a task panel | `crates/app/src/domain.rs`, then `crates/app/src/view.rs` |
| `document-workbench` | Notes, undo/redo, persistence and recovery | `crates/starter-notes/src/lib.rs` for model and format |
| `data-workbench` | CSV import/export, table, filter, charts and background work | `crates/starter-demo-domain/src/lib.rs` for operations |
| `engineering-workbench` | Parameters, units, canvas, 3D viewport, history and calculation task | `crates/starter-demo-domain/src/lib.rs` for the model |

Use `--profile desktop-egui --preset minimal` for a bare egui project. The
existing presets and eleven profiles continue to work. Automation is opt-in.
The local MCP adapter discovers public operations from the same registry
used by the command palette. Native Visual Forge remains an independent
development channel and takes explicit configuration.

## Discover and preview

```sh
python3 starter.py list-profiles
python3 starter.py list-presets
python3 starter.py list-capabilities --json
python3 starter.py describe automation
python3 starter.py plan --product desktop-application --capability automation --name my-app --json > project-plan.json
```

The preview shows the capability, view provider, modules, crates, declared
dependency constraints, features, and public entrypoints. Cargo selects exact
versions when it resolves the workspace. The `request` field can be checked
in as project configuration, or passed back as a complete plan file.

```sh
uv run --with-requirements generator/requirements.txt python starter.py doctor --config project-plan.json --verify
uv run --with-requirements generator/requirements.txt python starter.py create --config project-plan.json --verify
```

A native desktop app can switch from the general view to a data or document
workspace by enabling the corresponding capability. The CLI rejects a request
that combines competing ready-made views. The scientific computation capability
adds libraries. Pair it with `engineering-views` for a complete editor.
The browser-only parts of a hybrid profile still use web-compatible modules
when you select native-only verification. `--target` selects checks while
preserving the profile's entrypoints.

## Start developing

Read the generated `docs/GETTING_STARTED.md`. It links to the installed
extension points and recipes. `infrastructure.json` lists actual Cargo
workspace members, constraints, features, skills, examples and commands.
Use `cargo run -p desktop -- --operations` to inspect the desktop registry.
The generated workspace contains the library adapters and its own Cargo
manifests. It builds and evolves outside the starter checkout.

`--verify` checks project structure and documents, tests the native workspace,
and checks selected WASM targets. Interactive creation offers this verification
at confirmation. Native windows and browser startup have separate validation
requirements documented in [VALIDATION.md](VALIDATION.md).
