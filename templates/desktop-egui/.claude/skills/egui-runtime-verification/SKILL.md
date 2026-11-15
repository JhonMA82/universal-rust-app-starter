---
name: egui-runtime-verification
description: Verifies live egui applications using repeatable screenshots, interaction tests, accessibility inspection, egui_kittest, and optional egui_mcp. Use after implementing egui screens or when screenshot comparison reveals state, scaling, or capture inconsistencies.
license: MIT
compatibility: Running native egui app, Rust toolchain; optional egui_kittest 0.37 and egui_inspection/egui_mcp.
metadata:
  version: "1.0.0"
---

# Runtime evidence, not simulated confidence

## Capture preflight

1. Fix window/client-area size, display scale, egui `pixels_per_point`, platform, renderer, theme, font and locale. Record all available fields.
2. Move the application to the known initial state. Reset active tab, input contents, scroll offsets, modal visibility, selection and cursor state. Avoid variable timestamps and random content.
3. Check whether the reference has OS window decoration. Capture equivalent content bounds; do not make the comparator resize/stretch/auto-align a mismatched image.
4. Bring the window to the foreground when the capture transport requires a visible rendered frame.
5. Save candidate PNG without modifying it. Compare reference and candidate dimensions before interpreting any metric.

## Testing tiers

**Tier 1 — required when Rust tools are available:** `cargo fmt --check`, `cargo check`, `cargo test`. Verify intended state transitions (e.g. selection/checkboxes/tab changes).

**Tier 2 — recommended:** `egui_kittest` for accessibility-node queries and widget interaction tests. Snapshot tests are useful for regressions against *your own accepted* rendering, but an automatically updated snapshot is **not** evidence of matching the user's screenshot. Snapshot functionality requires the crate's relevant `snapshot` and `wgpu` features.

**Tier 3 — optional:** `egui_inspection` + `egui_mcp` to inspect the AccessKit tree, inject interactions and request screenshots of a live window. Enable only for development, prefer loopback binding. Exact startup depends on how your app builds eframe with inspection support; do not invent a tool invocation when no MCP server is connected.

## Distinguish test classes

- **Semantic tests**: role, label, value, toggled/selected state.
- **Functional tests**: trigger actions and verify resulting state.
- **Visual reference test**: compare candidate PNG against the reference screenshot using `visual_check.py`.
- **Regression snapshots**: compare against previously accepted output; not necessarily the original target reference.
- **Responsive tests**: repeat at a smaller and wider client area, watching for clipping and overlap.

## Reporting

For every test, record `pass`, `fail`, or `not run`, the actual command/tool, and the result. If screenshot capture is unavailable, do not infer visual success from `cargo check` or kittest alone.

Read `references/capture-checklist.md` before official image comparison.
