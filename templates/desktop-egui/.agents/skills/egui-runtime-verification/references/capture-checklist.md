# Deterministic capture checklist

- [ ] Reference and candidate are both PNG, same physical pixel dimensions.
- [ ] Window decorations either consistently included or consistently excluded.
- [ ] `pixels_per_point` was measured or consciously set on both sides.
- [ ] System font, fallback fonts, language, glyphs, theme and icon assets recorded.
- [ ] State and scroll position match.
- [ ] Pointer not over a hover-sensitive element unless reference shows hover.
- [ ] Animations and blinking cursor stabilized or narrowly excluded with reason.
- [ ] No automated geometric registration applied to official comparison.
- [ ] Candidate screenshot actually came from the app build currently being tested.

## Inspection notes

Official egui sources document that `egui_inspection` supports tree queries, event injection, screenshots and window resizing, with a visible window needed for screenshot capture. Set `EGUI_INSPECTION=1` only for a development binary that includes the eframe `inspection` feature. Binding inspection to a non-loopback address exposes remote control without authentication, so use loopback or a protected tunnel.

Docs: https://github.com/emilk/egui/blob/main/crates/egui_inspection/README.md
Docs: https://docs.rs/egui_mcp/latest/egui_mcp/
Docs: https://github.com/emilk/egui/blob/main/crates/egui_kittest/README.md
