# Known limitations

- Native-window and browser runs require a host display or a supported
  headless browser. Cross-compilation and offscreen rendering cover different
  behaviors. The local execution environment blocks sockets needed by its
  X11 and Chromium processes.
- Live Application MCP is opt-in, loopback-only and bearer authenticated.
  Discovery files are hints. The client verifies the advertised session after
  MCP initialization. The separately enabled graphical inspection channel
  has no application authentication and requires trusted local processes.
- Receipts are bounded and memory-only. An app using mutable shared references
  must provide an isolating snapshot implementation.
- Native documents use bounded JSON files and retain undo/redo in memory.
  Explicit Open and Save still perform synchronous bounded I/O. Periodic
  recovery writes run in a worker. External editors can bypass advisory locks.
- Browser notes are in memory. The browser build does not start native MCP,
  write a local path or run a native thread pool. No Web Worker adapter ships.
- Native packages are ZIP archives. Web packaging takes an application-built
  distribution directory. Signatures, installers and publishing are external.
