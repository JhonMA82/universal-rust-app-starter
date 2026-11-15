# egui 0.37 implementation patterns

## Layout responsibility

- Top-level fixed-height strip: use `TopBottomPanel` if the app API/context supports it and verify ordering, or create a measured top row in the root UI.
- Left/right explorer/properties: `SidePanel` with fixed or resizable widths, depending on screenshot evidence.
- Main workspace: `CentralPanel` or remaining `Ui`; use `Painter` only for complex custom canvas content.
- Forms: `Grid`, `horizontal`, `vertical` and `add_sized` for measured input widths; retain keyboard behavior.
- Scrolling: use `ScrollArea` around the correct subsection, not the entire viewport by default.
- Overlays: floating `Area` or window for controls that are truly detached from layout. Use stable IDs.

## State

State includes selected panel, active tabs, scroll offsets, modal visibility, text inputs, pressed/checked items and any temporary hover/focus state. For screenshot match, initialize to a reproducible state and document the capture protocol. Do not conflate what the screenshot shows with an actual application specification.

## Presentation

Use one palette/tokens module for background, surface, accent, disabled, text and border. Work in logical points. Use explicit fonts/assets where observable. If exact font unavailable, record fallback and do not claim text matching.

## Required extra checks

- Same screenshot at reference viewport size.
- At least one narrower viewport to detect clipping.
- A behavior test for important stateful controls.
- An inspection of the accessibility tree when available.

Official API: https://docs.rs/eframe/0.37.0/eframe/trait.App.html
