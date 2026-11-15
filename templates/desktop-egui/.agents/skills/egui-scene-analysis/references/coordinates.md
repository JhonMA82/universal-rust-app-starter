# Coordinate and measurement protocol

- Screenshot rectangles: `rect_px = [x, y, w, h]`; origin `(0,0)` top-left of the **source PNG**. All endpoints are half-open: `[x, x+w) × [y, y+h)`.
- Full-window PNG may include OS decorations. `content_rect_px` identifies the app content bounding box in screenshot pixels.
- For a scalar scale factor `s = pixels_per_point`, content-local `egui` point coordinate is `((x_px-content_x)/s, (y_px-content_y)/s)` and size is `(w_px/s, h_px/s)`.
- Do not assume `s=1` for HiDPI screens. If operating system and egui render scales differ, record screenshot effective scale, not device marketing DPI.
- Explicitly record if candidate screenshot includes/excludes window borders. For official evaluation, align capture **upstream**, not inside the comparator.
- Prefer major measurable boundaries (header bottom, side panel right, button baselines) over manually guessed offsets.
- An element may be visually centered yet pixel-asymmetric due to rasterization; reason in point units and confirm in screenshot pixels.
- Font fallback and subpixel antialiasing can cause differences even when layout is correct. Track the actual font source and renderer.

Measurement record example:

| Region | rect_px | origin | confidence | implementation |
| --- | --- | --- | --- | --- |
| Header | [0,0,1280,56] | visible separator | high | top panel |
| Explorer | [0,56,264,664] | boundary edge | medium | side panel |
| Work area | [264,56,1016,664] | remaining content | medium | central panel |

All numbers above are examples, not universal egui defaults.
