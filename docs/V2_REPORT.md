# v2 stabilization

The local stabilization series is based on `e6498de0a63b63c043176dbf49d7e262047b9d7d`.
It continues the existing modular engine, providers and verification scripts.

| Finding | Change |
| --- | --- |
| Generic presets installed disconnected crates | One entrypoint planner is used by ordinary projects and reference configurations |
| Workspace panel set was fixed | Application-owned panel instances have stable IDs, types and floating restrictions |
| Ctrl+P opened preferences | The palette discovers and dispatches operations through the same application registry |
| Infrastructure could bypass the operation dispatcher | Save completion is a private checked operation and snapshots have an isolation contract |
| Automation could only be launched with inherited stdio | Authorized clients can opt into a live loopback transport and discover running processes |
| Documents, tooling and catalogs could diverge | Generated composition guides, a checked catalog, link and descriptor audits |
| Testing conflated capabilities and environments | Reports preserve command, tree identity, platform, status and artifacts |

[Generator contracts](GENERATOR.md), [architecture](ARCHITECTURE.md),
[module catalog](MODULE_CATALOG.md), [migration](MIGRATION_V2.md) and
[verification](VALIDATION.md) explain the current interfaces. The
[known limitations](KNOWN_LIMITATIONS.md) describe the remaining execution
boundaries. The validation reports supplied with the stabilization patch
show the tests actually run. A green compile or offscreen render is insufficient
to claim a stable release when native or browser runtime checks remain open.
