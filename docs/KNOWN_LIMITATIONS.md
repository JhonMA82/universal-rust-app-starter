# Known limitations

- Native-window runs require a host or virtual display. Offscreen rendering,
  a virtual-display window, browser execution and native compilation are
  reported separately. See [verification](VALIDATION.md) for setup.
- Live Application MCP is opt-in, loopback-only and bearer authenticated.
  Discovery files are hints. The client verifies the advertised session after
  MCP initialization. The separately enabled graphical inspection channel
  has no application authentication and requires trusted local processes.
- Receipts are bounded and memory-only. An app using mutable shared references
  must provide an isolating snapshot implementation.
- Native documents use bounded JSON files and retain undo/redo in memory.
  Explicit Open and Save perform synchronous bounded I/O. The notes editor
  runs periodic recovery writes in a worker. Inspection Workbench currently
  performs those bounded writes in its window event loop. External editors can
  bypass advisory locks. Inspection Reports uses its own workspace. The
  combined tank workspace supports document, data and engineering views.
- Browser notes are in memory. The browser build does not start native MCP,
  write a local path or run a native thread pool. No Web Worker adapter ships.
- Native packages are ZIP archives. Web packaging takes an application-built
  distribution directory. Signatures, installers and publishing are external.
