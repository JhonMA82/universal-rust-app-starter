---
name: egui-visual-validation
description: Develop and inspect egui desktop and WASM interfaces with layout and visual tests.
---

# egui-visual-validation

## When to use

Use when the task is clearly about: develop and inspect egui desktop and wasm interfaces with layout and visual tests.
Do not force this skill into unrelated tasks.

## Procedure

Keep domain state independent of rendering; use panels with stable IDs. Test light/dark themes, narrow screens, keyboard navigation and accessibility labels. Use `egui_kittest` and snapshots when the corresponding feature is enabled and version-compatible. A visual test passes only after real screenshot comparison; never claim pixel accuracy based on code inspection. Inspect warnings and test the target platform.

## Evidence to return

State relevant changed paths, commands executed, results, limitations and next steps. Follow repository `AGENTS.md`.
