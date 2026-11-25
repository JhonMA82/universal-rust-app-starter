# Reference entrypoints

Run `cargo run -p desktop` for the workspace or use
`cargo run -p desktop -- --operations` to list the registered operations.
Add `-- --invoke '<request JSON>'` to run one operation in a fresh process.
The composition guide lists the automation entrypoints installed in this project.

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

For product views, supply your own workspace defaults with
`View::initial_workspace` and provide product colors with `View::design`.
The user's saved layout and appearance preferences still apply. Design
overrides only the selected accent; the UI kit owns the base typography,
spacing, radii and semantic feedback colors.
