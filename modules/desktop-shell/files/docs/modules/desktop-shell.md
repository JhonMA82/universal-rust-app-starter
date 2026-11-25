# Desktop shell

The serializable workspace owns a neutral tab/split tree and theme settings.
egui_dock projects that tree into the UI and reads the current main surface
back for persistence. The host opens its command palette with Ctrl/Cmd+P.
Preferences are available in the workspace toolbar. Panels that allow floating
can open in another native viewport. The browser uses embedded windows.
The shell confirms close requests when the host reports unsaved changes.

Run `cargo test -p starter-desktop` for layout and AccessKit tests. With a
supported adapter, set `STARTER_VISUAL_EVIDENCE` and run
`cargo test -p starter-desktop --features visual-evidence capture_gallery_interaction`.

Supply `PanelSpec` entries to `Shell::with_panels`. IDs identify instances
and kinds identify the application feature. The shell persists dock split
trees and which permitted panels are floating. Its menu restores closed panels.
Native floating viewports and embedded browser windows share panel content.
The gallery uses the same progress and unit-field helpers as the references.

## Use the components in your view

| Need | API |
|---|---|
| Labelled text and search | `field`, `search_field` |
| Numbers and editable properties | `unit_field`, `property_grid` |
| Progress and loading | `progress`, `loading` |
| Empty data, warnings and errors | `empty_state`, `notice` with `Tone` |
| Feedback the user can dismiss | `dismissible_notice` |
| Decisions that block other controls | `confirmation` with `DialogChoice` |
| Compact icon actions with text | `icon_button` |
| Choice, navigation and graphs | Use egui widgets, egui_dock and the selected data modules directly |

The **Components** tab renders the same API that views can import. Use **Controls**
for text, sliders and buttons, **States** for loading, feedback and errors, and
**Properties** for a filtered editor with units. Preview appearance offers
local theme and density overrides. Preferences change the application theme.
Use the gallery with a narrower window and toggle controls during visual
inspection. The AccessKit tests exercise the gallery and the snapshot test
compares frames before and after an interaction.

```rust,ignore
use starter_desktop::{empty_state, field, notice, Tone};

field(ui, "Project title", &mut draft);
if draft.trim().is_empty() {
    empty_state(ui, "No title", "Enter a title to save this project.");
    notice(ui, "A title is required", Tone::Warning);
}
```

In a view hosted by `starter-reference`, implement `View::design()` to provide
`Design { dark_accent, light_accent }`, using `starter_desktop::egui::Color32`.
Choose colors with readable contrast in both themes. The runtime applies the
design once per frame with the user's preferences. Implement
`View::initial_workspace()` to set the default layout and preference values.
A previously saved workspace supersedes those starting values. For an
independent egui window, call `Design::apply` with its context and `Tokens`.
Choose labels in the product's own locale; the shell's built-in English and
Spanish labels are selected through `Tokens::label`.
