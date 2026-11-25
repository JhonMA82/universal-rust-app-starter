---
name: egui-reconstruction
description: Orchestrates full screenshot-to-Rust-egui reproduction, from an image reference to a running application with measured visual fidelity. Use when the user wants to recreate, replicate, or match a GUI screenshot in egui/eframe, including an entire screen or multiple UI states.
license: MIT
compatibility: Rust/egui projects, Python 3.10+ managed with uv (Pillow, NumPy and PyYAML) for the supplied visual comparator; supports Agent Skills clients including OpenCode.
metadata:
  version: "1.0.0"
---

# Screenshot → egui: execution contract

This is the ENTRY skill. Use the companion skills only at the corresponding phase: `egui-scene-analysis`, `egui-ui-engineering`, `egui-runtime-verification`, `egui-visual-optimization`.

## Non-negotiable rules

1. Treat the screenshot as observed evidence, not a specification of invisible behavior. Mark interactions not visible in the reference as assumptions.
2. Never assert pixel-perfect or production-ready quality without a screenshot comparison AND working Rust checks.
3. Never modify the source screenshot to make it easier to match. Record its SHA-256, crop and pixels-per-point before implementation.
4. Prefer stable, relative egui layouts for panels and forms. Fixed sizes are permitted for screenshot-critical reference dimensions, but document responsive behavior.
5. Do not invent image assets or textual content silently. Transcribe visible strings and identify ambiguous glyphs/assets as unknowns.
6. Never accept a score from a comparison with unmatched window dimensions, unmatched state, or unmatched capture conditions.
7. Keep original reference imagery separate from candidate captures and generated diff images.

## Workflow

1. **Intake.** Obtain the screenshot(s), target framework/version, target platform and whether only appearance or working behavior is required. If unspecified, use latest verified stable egui/eframe release; record exact versions. Do not block on missing preferences; log assumptions.
2. **Scene specification.** Activate `egui-scene-analysis`. Create a `scene.json` with `uv run python tools/init_scene.py`; refine regions, known strings, assets, interaction states, masks and uncertainties. Save a measurement table. See `references/deliverables.md`.
3. **Construct.** Activate `egui-ui-engineering`. Create an eframe app from scene geometry and a small token system. First implement large structural regions; then typography, widgets, icons and interactions. Verify `cargo fmt --check` and `cargo check` if Rust tooling is available.
4. **Capture and interact.** Activate `egui-runtime-verification`. Reproduce the exact reference window content, scale, theme and state. Capture a fresh PNG and check critical interactions, ideally with `egui_kittest` and optionally `egui_mcp`/inspection.
5. **Compare and iterate.** Activate `egui-visual-optimization`. Run `uv run python tools/visual_check.py --scene scene.json --candidate output.png --out qa/iteration-01`. Read JSON and visual outputs; fix the highest-impact measured discrepancy and repeat. Do not change masks or thresholds simply to pass.
6. **Handoff.** Deliver code, reference/scene metadata, measured comparison, remaining discrepancies, tests run, tests not run, run command, and precise limitations. If execution/capture is impossible, explicitly say fidelity is unverified.

## Priority order

- P0: wrong dimensions/state/scale, missing major region, severe overflow, crash, or non-compiling code.
- P1: panel boundaries, spacing and typography metrics, text transcription, alignment and viewport clipping.
- P2: borders, shadows, icon shape, small color differences, hover/focus nuances.
- P3: decorative refinements only after P0–P2 are stable.

## Failure and uncertainty policy

If a tool or feature is unavailable, use the nearest honest fallback and report what was not tested. When visual iterations plateau, record the specific blocking constraint (font, platform, renderer, unknown asset, wrong screenshot crop). Do not disguise a plateau by raising tolerances.

## Quick commands

```bash
uv sync
uv run python tools/init_scene.py --image reference.png --out scene.json --pixels-per-point 1
uv run python tools/visual_check.py --scene scene.json --candidate candidate.png --out qa/run-01
uv run python tools/validate_toolkit.py
```

Paths refer to the project root; this profile ships a uv-managed `pyproject.toml` with `tools/` and `docs/`, so one `uv sync` provisions the dependencies. For a different project, copy `tools/` and `pyproject.toml` and run `uv sync`, or install Pillow, NumPy and PyYAML another way; use absolute paths as needed. The manifest resolves `reference.path` relative to its own location.
