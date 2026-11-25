# Migrate from v1

Direct Cargo Generate commands remain valid. Generate a v2 composition in
a new directory, then copy only the required local crates, integrations and
manifest changes into an existing project. Keep the domain and its tests.

Update the egui family together. Both base desktop profiles and Visual Forge
use 0.37. Run the native and WASM profile checks separately when targeting both.
Run the generated agent checker to synchronize the relative Claude adapter
or its portable copy. Gemini continues to import the same AGENTS contract.

Choose retention and authorization before exposing operations. Each domain
defines its own format, version and migrations. Commit the resulting lockfile
and run the existing tests. If the composition includes `engineering-tools`,
run `cargo xtask verify`. Read the generated composition guide for the
entrypoints in the new project, including any selected automation adapter.
