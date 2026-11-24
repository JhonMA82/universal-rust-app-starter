# Application Core

Register typed handlers and discover public JSON schemas. Compose a trusted
session with explicit permissions. Commands validate against isolated owned
or copy-on-write state, and queries borrow state immutably. Keep external
effects out of the rollback boundary. Optional request IDs retain successful
responses within the configured in-memory limit. Restart and eviction end
retention. Clients should send expected revisions when editing shared state.

Run `cargo test -p starter-core` to check replay, rejection and rollback.
