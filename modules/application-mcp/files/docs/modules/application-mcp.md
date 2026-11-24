# Application MCP

Compose the official rmcp stdio adapter with a trusted least-privilege session.
The service exposes capabilities, execution and bounded batches. Batches
commit independently and can interleave with other callers. Diagnostics live
on stderr. The optional development UI inspection channel is separate.
