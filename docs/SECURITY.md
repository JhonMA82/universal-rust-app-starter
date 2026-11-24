# Security boundaries

Application operations enforce session identity, permission and optimistic
revision checks before dispatch. Public transport adapters cannot run internal
operations. Consult the registry for schemas and the operation contract before
adding a public operation. Idempotency is a bounded in-process retry cache.

On composed HTTP services, set `APP_OPERATIONS_TOKEN` to a unique URL-safe
bearer token of 32–128 characters. The example retains its existing health
or HTML route. Tokens are provided at run time, never emitted into generated
files. Configure TLS and credential handling outside the starter when enabling
a service beyond a trusted local boundary.

Live desktop MCP uses authenticated Streamable HTTP on IPv4 loopback.
Keep the discovery directory private and use a distinct token per trust
boundary. Discovery hints are not proof of identity. Authenticate the selected
endpoint and compare the application session IDs. The graphical development
endpoint is separate. It requires an explicit debug feature and launch flag,
binds to loopback, and relies on trusted local processes.

Document file targets and permissions are selected by the adapter. Signature
or hash checks do not replace semantic or revision validation. Advisory
locks coordinate cooperating writers. External filesystem effects are not
rolled back by application state transactions.
