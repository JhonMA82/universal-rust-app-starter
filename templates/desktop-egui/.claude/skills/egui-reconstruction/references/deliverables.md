# Required handoff evidence

A reproduction task is not complete merely because it displays a plausible screen.

- Original screenshot path and recorded SHA-256.
- Target/reference PNG dimensions and pixel-to-point scaling; distinguish full window and content crop.
- `scene.json`: regions, weights, uncertainty notes, supported interaction states, ignore masks with reasons.
- Rust project, pinned egui/eframe versions, explicit run instructions.
- Candidate screenshot and visual comparison report. Include unchanged gate thresholds and failures.
- Evidence of `cargo check`, `cargo fmt --check`, UI behavioral tests, and resize tests **only if they actually ran**.
- A short discrepancy register: severity, location, probable cause, changed file, disposition, remaining issue.

## Suggested acceptance checklist

- Each required region exists and is not clipped.
- All transcribed strings visually agree with the screenshot, unless labeled uncertain.
- Gating passes under stable capture conditions, OR every failed gate is documented.
- Test states align with the reference state (for example, selected tab and scroll position).
- Controls with inferred behavior are identified as inferred.
- No ignored area exists merely to hide implementation differences.
