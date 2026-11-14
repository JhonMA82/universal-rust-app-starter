---
name: rust-web-security
description: Review Rust web services for authorization, input limits, secrets and transport security.
---

# rust-web-security

## When to use

Use when the task is clearly about: review rust web services for authorization, input limits, secrets and transport security.
Do not force this skill into unrelated tasks.

## Procedure

Identify trust boundaries and attacker capabilities. Default to deny for privileged actions; authentication is not authorization. Validate request bodies and method/size limits, configure CORS/CSRF deliberately, avoid leaking errors and secrets. Use TLS at trusted ingress and review any externally bound address. Create negative tests for unauthorized and malformed requests; do not advertise localhost demo as Internet-ready.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
