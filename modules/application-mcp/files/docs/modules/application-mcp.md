# Application MCP

Compose the official rmcp stdio adapter with a trusted least-privilege session.
The service exposes capabilities, execution and bounded batches. Batches
commit independently and can interleave with other callers. Diagnostics live
on stderr. The optional development UI inspection channel is separate.

The composition guide lists the enabled transport commands. The standalone
Streamable HTTP profile in composed projects binds to loopback. It assumes a
trusted host, while the distinct desktop live endpoint requires a bearer
token. Desktop discovery descriptors omit that token. Regular shutdown removes
the descriptor. The desktop headless transport exits on stdin EOF.
