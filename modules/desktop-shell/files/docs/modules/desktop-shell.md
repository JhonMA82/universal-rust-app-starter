# Desktop shell

The serializable workspace owns a neutral tab/split tree and theme settings.
egui_dock projects that tree into the UI and reads the current main surface
back for persistence. Cmd/Ctrl+P opens preferences. On desktop the inspector
can be shown in a second native viewport. WASM uses an embedded window.
The shell confirms close requests when the host reports unsaved changes.

Run `cargo test -p starter-desktop` for layout and AccessKit tests. With a
supported adapter, set `STARTER_VISUAL_EVIDENCE` and run
`cargo test -p starter-desktop --features visual-evidence capture_gallery_interaction`.
