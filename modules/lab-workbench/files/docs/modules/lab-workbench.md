# Combined desktop workbench

When a native composition requests multiple document, data and engineering views, the compositor connects them using one application host. The starting domain is a liquid-volume tank. Change its format and operations in `crates/starter-lab/src/domain.rs` and its panel registrations in `crates/starter-lab/src/lib.rs`.

- **Tank project** edits dimensions and fill, saves and opens typed projects, and starts or cancels numerical evaluation.
- **Results** presents a filtered table and a plot. A calculation publishes only against its source revision.
- **Geometry** presents the same tank in a pannable 2D canvas and a native wgpu viewport.
- **Edit → Undo tank edit** uses the same operation as the palette and MCP. File operations stay local to the host.

Run `cargo run -p desktop`. List registered operations with `cargo run -p desktop -- --operations`. Verify the model with `cargo test -p starter-lab`. This composition targets native desktop. The additional automation capability enables MCP, while the standard UI inspection feature is opt-in.
