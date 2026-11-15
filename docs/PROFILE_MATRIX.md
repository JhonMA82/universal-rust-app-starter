# Profile matrix

| Template | Quality | External tools | Minimum smoke test | Agent specialist |
|---|---|---|---|---|
| library | Native CI | rustc/cargo | cargo test | rust-semver |
| cli | Native CI | rustc/cargo | greet command | rust-cli |
| desktop-egui | Native CI | OS window libraries; Python optional for visual QA | cargo run -p desktop | egui-visual-validation, egui-reconstruction, egui-scene-analysis, egui-ui-engineering, egui-runtime-verification, egui-visual-optimization |
| desktop-egui-web | Native+WASM CI | Trunk + window libraries | desktop + trunk serve | egui-visual-validation, wasm-compatibility |
| service-axum | Native CI | rustc/cargo | GET /health | rust-web-security, async-rust |
| mcp-stdio | Native CI | MCP Inspector | initialize, tools/list, echo | mcp-protocol-validation |
| mcp-http | Native CI | MCP Inspector | initialize, tools/list, echo | mcp-protocol-validation |
| mcp-client | Native CI | running server | list_tools | mcp-protocol-validation |
| web-leptos-csr | WASM CI | Trunk | browser render | wasm-compatibility |
| web-leptos-fullstack | Native CI | rustc/cargo | HTML at `/` | rust-web-security |
| web-dioxus | WASM CI | Dioxus CLI | browser render | wasm-compatibility |

Actual package compilation and browser/tool integration remain unverified in the initial packaging environment; see VALIDATION.md.
