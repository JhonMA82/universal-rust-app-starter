# Known limitations

- Native-window lifecycle and the live UI MCP bridge need testing on a
  working display. Browser compilation and execution are separate checks.
- Application MCP serves one trusted session per reference process. Receipt
  retention is memory-only and authorization policies belong to the app.
- The document module supports bounded JSON files and retains undo/redo in
  memory. Project containers and persistent cross-process history are outside
  the current implementation. External editors can race advisory lock users.
- Native background workers use cooperative cancellation. WASM Web Workers
  require a separate adapter. Data export cannot reverse an external file write.
- Native packaging produces ZIP archives. Web packaging takes a separate web
  build. Signed installers and native plugin hosts are outside this version.
