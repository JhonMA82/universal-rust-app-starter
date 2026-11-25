---
name: egui-scene-analysis
description: Extracts an evidence-backed, measurable scene specification from one or more application screenshots for egui implementation. Use to analyze geometry, coordinates, typography, visual tokens, uncertain assets, panel hierarchy, and screenshot-state assumptions before coding.
license: MIT
compatibility: PNG screenshots; Python 3.11+ managed with uv (Pillow) for the initializer.
metadata:
  version: "1.0.0"
---

# Convert screenshots into an observable scene contract

## Procedure

1. Inspect the image in its native pixel resolution. Record `width_px`, `height_px`, full-window chrome inclusion, content crop and `pixels_per_point`. Do not estimate scale from resolution alone; use evidence or mark an assumption.
2. Call `uv run python tools/init_scene.py --image screenshot.png --out scene.json --pixels-per-point 1` to create the valid initial manifest (run `uv sync` once from the project root first). The default value 1 is a **placeholder**, not a measured DPI.
3. Establish a nested visual hierarchy: root → shell regions → controls/text/assets. Prefer region boundaries that align with edges or repeated spacing lines. Identify z-order for overlays.
4. Transcribe visible text literally (preserving capitalization and punctuation); list uncertain glyphs, icons, and non-visible interactions as `uncertainties`.
5. Measure reference rectangles in pixels `[x, y, width, height]` from the **top-left corner** of the image. Define source of every measurement: edge, repeated grid, visible control, or estimate.
6. Extract design tokens from **repeated** visual values: background/surface/accent/text colors, corner radii, spacing, panel width and typography. Keep inferred values as approximate; do not fill everything with guessed precision.
7. Create separately named regions representing major layout responsibilities. Assign `weight` in `[0.1, 10]` for prioritization; do not treat weights as permission to ignore the rest.
8. Mark dynamic/redacted regions in `ignore_rects_px` only if genuinely unverifiable: blinking cursor, unpredictable timestamp, confidential data. Add a non-empty `reason`. Keep ignored area small; comparator enforces a global mask limit.
9. Specify the exact reference state (selected tab, scroll, hover, modal, etc.). Different screenshots of different states require different scene files or explicit state-specific records.
10. Review the screenshot-to-egui conversion math in `references/coordinates.md` before coding.

## Outputs

- `scene.json` using `docs/SCENE_FORMAT.md`.
- Short table of major regions, exact source rectangles, probable egui panel/layout primitive, confidence (high/medium/low).
- Token inventory and uncertainty register.
- A list of required extra reference assets, if any.

## Anti-patterns

- Assuming Windows/macOS title bar is part of egui content.
- Treating pixel dimensions as egui points at non-unit scale.
- Deriving a complete interaction flow from one static image.
- Using generated text when exact screenshot text is visible.
- Cropping, stretching, or shifting images for the official quality score.
