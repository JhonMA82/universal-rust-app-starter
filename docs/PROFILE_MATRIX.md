# Profile matrix

Every profile is available through direct Cargo Generate and through the
compositor's Minimal preset. Application connects the typed operation registry.

| Profile | Targets | Application integration | Runtime verification |
|---|---|---|---|
| `library` | Native | Public operations API | Doctest |
| `cli` | Native | Operation CLI, optional MCP stdio | Greeting and operation process |
| `desktop-egui` | Native | Egui settings workspace and operation CLI, optional MCP | Operation process and reference window smoke |
| `desktop-egui-web` | Native, WASM | Shared settings UI or Workbench notes | Native tests, WASM check and browser interaction |
| `service-axum` | Native | Authenticated application operations beside `/health` | Process with HTTP requests |
| `mcp-stdio` | Native | Operation server over stdio | Handshake, tool discovery and invocation |
| `mcp-http` | Native | Operation server over local Streamable HTTP | Handshake, tool discovery and invocation |
| `mcp-client` | Native | Existing client and local operations companion | External discovery and companion process |
| `web-leptos-csr` | WASM | Registry-backed settings controls | Trunk and browser interaction |
| `web-leptos-fullstack` | Native | Authenticated operations beside server-rendered HTML | HTML and operation HTTP requests |
| `web-dioxus` | WASM | Registry-backed settings controls | Dioxus CLI and browser interaction |

Workbench supports both egui profiles. The hybrid notes view keeps its state
in memory. Engineering supports native egui. See the
[module catalog](MODULE_CATALOG.md) for module constraints and
[validation](VALIDATION.md) for the commands and evidence categories.
