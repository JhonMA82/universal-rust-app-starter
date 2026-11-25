# Desktop product starter

Put domain types, invariants and operations in `crates/app/src/domain.rs`.
Add egui views and dockable panels in `crates/app/src/view.rs`. The entrypoint
connects both to the inherited shell and operation registry. Run
`cargo run -p desktop -- --operations` to discover the example operations.
The registered Background work panel runs a word-count task on the current
draft. Editing the draft changes its revision so late results cannot replace
the current result. The panel can be floated or reopened using the shell.
