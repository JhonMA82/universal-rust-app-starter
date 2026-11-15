---
name: egui-ui-engineering
description: Implements screenshot-derived scene specifications as robust Rust egui/eframe applications, preserving pixel-measured geometry, functional controls, themes, and responsive behavior. Use when writing or restructuring egui source code for faithful visual reconstruction.
license: MIT
compatibility: Rust toolchain, egui/eframe 0.37.0 baseline; confirm breaking changes for other versions.
metadata:
  version: "1.0.0"
---

# Build a faithful egui app

## Implementation sequence

1. Read the scene and its scale/crop provenance. Pin `eframe` / `egui` versions and identify the native/web renderer. Do not assume old tutorials use the current `eframe::App` API.
2. Create minimal app with deterministic initial state and fixed reference viewport size for comparison. Use normal responsive layouts outside the reference size; avoid absolute placement of entire forms.
3. Implement outer shell first: header/toolbars, bottom status, side panels, main work area and overlays. Confirm dimensions with a real capture before writing complex content.
4. Centralize palette, typography, spacing and key dimensions. Distinguish **observed** token values from assumed design conveniences.
5. Implement inner rows and controls with `horizontal`, `vertical`, `Grid`, panels, `Frame` and sensible `add_sized`/`allocate` where measured geometry calls for it. Use painter-based shapes for genuine custom graphics, not to fake clickable widgets.
6. Use stable IDs for repeated/dynamic egui widgets. Keep state in the app struct and separate state transitions from rendering. Give controls accessible names.
7. Put graphics, icons and fonts under explicit asset management; check licensing of any asset. Do not replace a missing brand icon with vaguely similar Unicode and claim fidelity.
8. Call `cargo fmt --check`, `cargo check`, `cargo test` when possible; then use the runtime verification skill to capture and inspect.

## Correctness before cosmetics

- Reference behavior from egui 0.37 docs and current source; `eframe::App` in 0.37 provides `fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)`.
- Do not blindly paste deprecated `fn update(&mut self, ctx: &egui::Context, ...)` implementations from older major versions.
- Build geometry in egui logical points; use `scene.reference.pixels_per_point` for conversion.
- Do not introduce magic offsets to compensate for inconsistent screenshot crop or OS title bars.
- If a frame demands custom text metrics, load the known font and verify measured output; never stretch bitmap text.
- Preserve keyboard focus, tab order and accessible labels for interactive controls.

## Required output

- `Cargo.toml` with tested/pinned dependencies and explicit target platform.
- `src/main.rs` or project modules, token constants and state models.
- A clear record of which behaviors were visible and which were assumptions.
- `cargo check` / tests output or an explicit statement that Rust compilation was unavailable.

See `references/egui-patterns.md` and the standalone `examples/eframe-starter/` app. That example is a construction starter, **not** proof of screenshot fidelity.
