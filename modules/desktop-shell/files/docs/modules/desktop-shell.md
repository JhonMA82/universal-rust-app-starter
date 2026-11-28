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
| Icon actions with text | `button_with_icon` and `Icon` |
| Choice, navigation and graphs | Use egui widgets, egui_dock and the selected data modules directly |

The **Components** tab renders the same API that views can import. Use **Controls**
for text, sliders and buttons, **States** for loading, feedback and errors, and
**Properties** for a filtered editor with units. Preview appearance offers
local theme and density overrides. Preferences change the application theme.
The reset example uses the same named vector buttons available to your view.
The icon is decoration inside a native egui button with a visible caption,
and no external icons or fonts are required. Use the gallery with a
narrower window and toggle controls during visual inspection. The AccessKit tests exercise the gallery and the snapshot test
compares frames before and after an interaction. The dock divider in
egui_dock 0.22 is exposed to AccessKit as a splitter without a name.

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
The runtime applies the design once per frame with the user's preferences.
`Design::colors(tokens)` returns the accent, its foreground, and semantic
feedback colors. The accent is opaque and the foreground is chosen for
contrast. Use `tokens.metrics()` to match the shell's gaps, typography,
control height, radii and icon size in a custom view. Implement
`View::initial_workspace()` to set the default layout and preference values.
A previously saved workspace supersedes those starting values. For an
independent egui window, call `Design::apply` with its context and `Tokens`.
Choose labels in the product's own locale; the shell's built-in English and
Spanish labels are selected through `Tokens::label`.

## Register an action

In the desktop host, implement `View::actions()` with a static slice of
`ActionSpec` entries. Each entry names a public operation already registered
with Application Core. `menu` groups actions under a top-level menu. Set
`toolbar: true` to show an action on the toolbar as well. The optional
`KeyboardShortcut` is shown alongside the label.

An action can appear in more than one place. All entry points dispatch the
same command ID. The host checks availability and permissions using the
active document and supplies the current revision. Override
`View::action_parameters()` to provide parameters from the view. The default
is an empty JSON object. Other operation input can be provided from the
generic command palette.

Shortcuts match a complete chord, ignore key repeats and stay inactive while
a text editor, popup, command palette or close dialog has focus. On duplicates, the first enabled registered action wins.
Ctrl/Cmd+P remains the shell's shortcut for the palette.

```rust,ignore
use starter_desktop::{ActionSpec, egui};

static ACTIONS: &[ActionSpec] = &[ActionSpec {
    command: "workspace.name",
    label: "Show workspace name",
    menu: Some("Workspace"),
    toolbar: true,
    shortcut: Some(egui::KeyboardShortcut::new(egui::Modifiers::ALT, egui::Key::L)),
}];
```

Return `ACTIONS` from `View::actions()`. The `workspace.name` example is
available in the standard desktop composition. For another operation,
register its input, output and handler in the product's domain first.
