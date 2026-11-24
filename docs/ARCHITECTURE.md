# Architecture

The eleven [base profiles](PROFILE_MATRIX.md) remain independent Cargo
Generate templates. The [composer](GENERATOR.md) adds modules to their
staged copies and selects an entrypoint using the effective capabilities.
References select a provider through this same path as ordinary projects.

| Concern | Runtime boundary |
| --- | --- |
| Registered operations, schemas, permissions | starter-core |
| Typed note model and history | starter-notes and starter-documents |
| Windowing, docking, tokens | starter-desktop and the egui composition host |
| Domain-specific views | desktop providers for notes, data and geometry |
| Local automation and HTTP | starter-mcp and starter-http |
| Geometry, GPU and UI | viewport-core, viewport-wgpu and viewport-egui |
| Developer verification | xtask and the repository validation scripts |

Each application owns its domain and the composition root joins its adapters.
A desktop provider supplies a view and an engine. The window reads the engine
registry for the command palette. Typed operations are shared by the GUI, CLI
and authorized transports. Native filesystem operations remain in the editor
adapter. Autosave uses a cancellable background job over immutable snapshots.

Cargo metadata follows transitive dependencies to audit layer boundaries.
Generated projects receive only their selected module files and skills. The
generated `docs/COMPOSITION.md` records where those capabilities connect.
