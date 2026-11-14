---
name: mcp-protocol-validation
description: Validate MCP tools, schemas and stdio or Streamable HTTP protocol behavior.
---

# mcp-protocol-validation

## When to use

Use when the task is clearly about: validate mcp tools, schemas and stdio or streamable http protocol behavior.
Do not force this skill into unrelated tasks.

## Procedure

Keep stdio stdout reserved for JSON-RPC and send diagnostics to stderr. Define tool contracts using typed schemas. Test initialize, tools/list and tools/call using a real client or inspector. For HTTP, use Streamable HTTP (not legacy two-endpoint SSE), bind localhost by default and require an explicit authentication plan before exposing publicly. Check transport SDK version and protocol compatibility.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
