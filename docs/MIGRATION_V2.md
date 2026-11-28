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

## From the Inspection Workbench candidate

Replace `Application::state_mut` calls with registered operations. Use a private
session operation for local filesystem effects and supply the observed
application revision on invocation. Keep runtime-only state in the view.
Use the typed `Document<T>` for file identity and history.

Use `Recovery::for_path` for saved documents and an application-defined
key for untitled work. The new discovery API lists validated drafts and their
original paths. Older PID-keyed drafts lack proof that they belong to a saved
document. Preserve those files and review their contents manually before import.
