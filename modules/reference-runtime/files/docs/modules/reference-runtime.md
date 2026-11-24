# Reference entrypoints

Run `cargo run -p desktop` for the workspace, add `-- --mcp` for stdio,
`-- --mcp-gui` for stdio sharing the running window state, or
`-- --invoke '<request JSON>'` for one operation in a fresh process.

For development UI inspection, run `cargo run -p desktop --features
ui-inspection -- --ui-inspect` and attach the official egui_mcp 0.3 bridge
to 127.0.0.1:5719. Release and default builds do not bind the socket.
Other local processes can reach an explicitly enabled inspection session.
