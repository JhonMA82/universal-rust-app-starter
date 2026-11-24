# Reference entrypoints

Run `cargo run -p desktop` for the workspace, add `-- --mcp` for stdio,
`-- --mcp-gui` for stdio sharing the running window state, or
`-- --invoke '<request JSON>'` for one operation in a fresh process.

For development UI inspection, run `cargo run -p desktop --features
ui-inspection -- --ui-inspect` and attach the official egui_mcp 0.3 bridge
to 127.0.0.1:5719. Release and default builds do not bind the socket.
Other local processes can reach an explicitly enabled inspection session.

The window opens its registry-backed palette with Ctrl/Cmd+P or Commands.
Search by operation ID or description, select an operation, review the schema
and enter its JSON parameters. The palette uses the active document and current
revision, and the engine checks permissions and availability again on dispatch.

Applications can register a typed panel instance through `View::panels`.
Use stable numeric instance IDs and stable kinds to keep layouts restorable.
Panels with floating enabled become native viewports or embedded browser windows.

When Application MCP is selected, `--mcp-live` starts an authenticated
loopback endpoint alongside the GUI. Set a private `STARTER_SESSION_DIR`
and a unique URL-safe `STARTER_MCP_TOKEN` of at least 32 characters. Use
`--sessions` from another process with the same directory to discover the
endpoint, then configure an MCP HTTP client with the bearer token. Verify
that capabilities return the advertised session ID. Descriptors contain no
token. For normal process exit they are removed automatically. Discovery filters
unreachable entries. `--mcp-live-headless` uses the same transport and ends
on stdin EOF.
