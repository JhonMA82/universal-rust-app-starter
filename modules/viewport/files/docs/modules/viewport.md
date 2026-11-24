# Viewport

Scene, camera and picking are independent of wgpu and egui. The renderer uses
depth-buffered offscreen meshes and lines. The egui adapter displays the
texture and maps orbit, zoom and selection. Capture is blocking and intended
for evidence or export. Test CPU picking with `cargo test -p starter-viewport-core`.
Run `cargo test -p starter-viewport-wgpu --features gpu-evidence` with a GPU adapter.
