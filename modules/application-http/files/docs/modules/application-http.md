# Application HTTP

Mount the Axum router with its trusted session and an explicit bearer credential.
The caller owns the listener, TLS and credential lifecycle.

The service and Leptos SSR adapters mount `GET /operations` and `POST /invoke`.
Set a unique 32–128 character URL-safe token in `APP_OPERATIONS_TOKEN` when
starting the server, and provide it in the Authorization bearer header.
A request's session ID comes from the operation discovery response.
