#![forbid(unsafe_code)]
mod actions;
mod components;
mod design;
mod gallery;
mod icons;

pub use actions::{ActionSpec, ActionState};
pub use components::{
    DialogChoice, Tone, confirmation, dismissible_notice, empty_state, field, icon_button, loading,
    notice, progress, property_grid, search_field, status, unit_field,
};
pub use design::{Design, Metrics, ThemeColors, Tokens};
pub use egui;
pub use gallery::Gallery;
pub use icons::{Icon, button_with_icon};

use egui::{Context, Ui};
use egui_dock::{DockArea, DockState, Node, NodeIndex, TabViewer, Tree};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug)]
pub struct PanelSpec {
    pub instance: u64,
    pub kind: &'static str,
    pub title: &'static str,
    pub floating: bool,
    pub initially_open: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Panel {
    Work,
    Inspector,
    Gallery,
    Custom(u64),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Layout {
    Tabs(Vec<Panel>, usize),
    Split {
        vertical: bool,
        fraction: f32,
        first: Box<Layout>,
        second: Box<Layout>,
    },
}
impl Default for Layout {
    fn default() -> Self {
        Self::Split {
            vertical: false,
            fraction: 0.72,
            first: Box::new(Self::Tabs(vec![Panel::Work, Panel::Gallery], 0)),
            second: Box::new(Self::Tabs(vec![Panel::Inspector], 0)),
        }
    }
}
impl Layout {
    fn collect(&self, depth: usize, seen: &mut Vec<Panel>) -> bool {
        if depth > 12 || seen.len() > 64 {
            return false;
        }
        match self {
            Self::Tabs(tabs, active) if *active < tabs.len() && tabs.len() <= 64 => {
                seen.extend(tabs);
                true
            }
            Self::Split {
                fraction,
                first,
                second,
                ..
            } if (0.0..=1.0).contains(fraction) => {
                first.collect(depth + 1, seen) && second.collect(depth + 1, seen)
            }
            _ => false,
        }
    }
    fn valid(&self) -> bool {
        let mut seen = Vec::new();
        self.collect(0, &mut seen)
            && seen.len() <= 64
            && seen.iter().enumerate().all(|(i, p)| !seen[..i].contains(p))
            && seen.contains(&Panel::Work)
    }
    fn prune(self, known: &BTreeMap<u64, PanelSpec>) -> Option<Self> {
        match self {
            Self::Tabs(tabs, active) => {
                let active_panel = tabs.get(active).copied();
                let tabs: Vec<_> = tabs
                    .into_iter()
                    .filter(|panel| match panel {
                        Panel::Custom(id) => known.contains_key(id),
                        _ => true,
                    })
                    .collect();
                if tabs.is_empty() {
                    return None;
                }
                let active = tabs
                    .iter()
                    .position(|&panel| Some(panel) == active_panel)
                    .unwrap_or(0);
                Some(Self::Tabs(tabs, active))
            }
            Self::Split {
                vertical,
                fraction,
                first,
                second,
            } => match (first.prune(known), second.prune(known)) {
                (Some(first), Some(second)) => Some(Self::Split {
                    vertical,
                    fraction,
                    first: Box::new(first),
                    second: Box::new(second),
                }),
                (one, two) => one.or(two),
            },
        }
    }

    fn tabs(&self) -> Vec<Panel> {
        match self {
            Self::Tabs(tabs, _) => tabs.clone(),
            Self::Split { first, .. } => first.tabs(),
        }
    }
    fn install(&self, tree: &mut Tree<Panel>, id: NodeIndex) {
        match self {
            Self::Tabs(_, active) => {
                if let Some(leaf) = tree[id].get_leaf_mut() {
                    let _ = leaf.set_active_tab(*active);
                }
            }
            Self::Split {
                vertical,
                fraction,
                first,
                second,
            } => {
                let split = if *vertical {
                    egui_dock::Split::Below
                } else {
                    egui_dock::Split::Right
                };
                let [a, b] = tree.split_tabs(id, split, *fraction, second.tabs());
                first.install(tree, a);
                second.install(tree, b);
            }
        }
    }
    fn read(tree: &Tree<Panel>, id: NodeIndex) -> Option<Self> {
        match &tree[id] {
            Node::Empty => None,
            Node::Leaf(leaf) => Some(Self::Tabs(leaf.tabs().to_vec(), leaf.active.0)),
            Node::Vertical(split) | Node::Horizontal(split) => Some(Self::Split {
                vertical: tree[id].is_vertical(),
                fraction: split.fraction,
                first: Box::new(Self::read(tree, id.left())?),
                second: Box::new(Self::read(tree, id.right())?),
            }),
        }
    }
}
#[derive(Default, Serialize, Deserialize)]
pub struct Workspace {
    pub layout: Layout,
    pub tokens: Tokens,
    pub document: Option<u64>,
    #[serde(default)]
    pub floating: BTreeSet<u64>,
    #[serde(default)]
    pub registered: BTreeSet<u64>,
}
pub struct Shell {
    pub workspace: Workspace,
    dock: DockState<Panel>,
    gallery: Gallery,
    design: Design,
    preferences: bool,
    pub commands_open: bool,
    panels: BTreeMap<u64, PanelSpec>,
    inspector: bool,
    confirm_close: bool,
    approved_close: bool,
    save_requested: bool,
}
impl Default for Shell {
    fn default() -> Self {
        Self::new(Workspace::default())
    }
}
impl Shell {
    pub fn new(workspace: Workspace) -> Self {
        Self::with_panels(workspace, &[])
    }

    pub fn with_panels(mut workspace: Workspace, specs: &[PanelSpec]) -> Self {
        let panels: BTreeMap<_, _> = specs
            .iter()
            .take(60)
            .map(|spec| (spec.instance, *spec))
            .collect();
        let layout = std::mem::take(&mut workspace.layout);
        workspace.layout = if layout.valid() {
            layout
                .prune(&panels)
                .filter(Layout::valid)
                .unwrap_or_default()
        } else {
            Layout::default()
        };
        let mut dock = DockState::new(workspace.layout.tabs());
        workspace
            .layout
            .install(dock.main_surface_mut(), NodeIndex::root());
        workspace.floating.retain(|id| {
            panels.get(id).is_some_and(|spec| spec.floating)
                && dock.find_tab(&Panel::Custom(*id)).is_none()
        });
        workspace.registered.retain(|id| panels.contains_key(id));
        for (&id, spec) in &panels {
            if workspace.registered.insert(id)
                && spec.initially_open
                && dock.find_tab(&Panel::Custom(id)).is_none()
                && !workspace.floating.contains(&id)
            {
                dock.push_to_focused_leaf(Panel::Custom(id));
            }
        }
        workspace.layout = Layout::read(dock.main_surface(), NodeIndex::root()).unwrap_or_default();
        Self {
            workspace,
            dock,
            gallery: Gallery::default(),
            design: Design::default(),
            preferences: false,
            commands_open: false,
            panels,
            inspector: false,
            confirm_close: false,
            approved_close: false,
            save_requested: false,
        }
    }

    pub fn set_design(&mut self, design: Design) {
        self.design = design;
    }

    pub fn take_save_request(&mut self) -> bool {
        std::mem::take(&mut self.save_requested)
    }
    pub fn finish_close(&mut self, ctx: &Context) {
        self.approved_close = true;
        self.confirm_close = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    pub fn open_panel(&mut self, id: u64) -> bool {
        if !self.panels.contains_key(&id) {
            return false;
        }
        self.workspace.floating.remove(&id);
        let panel = Panel::Custom(id);
        if let Some(path) = self.dock.find_tab(&panel) {
            let _ = self.dock.set_active_tab(path);
            self.dock.set_focused_node_and_surface(path.node_path());
        } else {
            self.dock.push_to_focused_leaf(panel);
        }
        true
    }

    pub fn float_panel(&mut self, id: u64) -> bool {
        if !self.panels.get(&id).is_some_and(|spec| spec.floating) {
            return false;
        }
        if let Some(path) = self.dock.find_tab(&Panel::Custom(id)) {
            self.dock.remove_tab(path);
        }
        self.workspace.floating.insert(id);
        true
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        title: &str,
        message: (&str, bool),
        dirty: bool,
        show: impl FnMut(Panel, &mut Ui),
    ) {
        self.show_with_actions(ui, title, message, dirty, &[], show);
    }

    /// Render the shell and return the selected action's index for the host to dispatch.
    pub fn show_with_actions(
        &mut self,
        ui: &mut Ui,
        title: &str,
        message: (&str, bool),
        dirty: bool,
        actions: &[ActionState],
        mut show: impl FnMut(Panel, &mut Ui),
    ) -> Option<usize> {
        let ctx = ui.ctx().clone();
        let tokens = self.workspace.tokens;
        self.design.apply(&ctx, tokens);
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::P,
            ))
        }) {
            self.commands_open = !self.commands_open;
        }
        if dirty && !self.approved_close && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }
        let commands_blocked = self.confirm_close || self.commands_open;
        let mut selected = None;
        actions::menu_bar(ui, actions, &mut selected);
        egui::Panel::top("starter-toolbar").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading(title);
                self.commands_open |=
                    button_with_icon(ui, Icon::Search, tokens.label("Commands", "Comandos"))
                        .clicked();
                self.preferences |= ui
                    .button(tokens.label("Preferences", "Preferencias"))
                    .clicked();
                self.inspector |= ui
                    .button(tokens.label("Inspector window", "Ventana de inspección"))
                    .clicked();
                actions::toolbar(ui, actions, &mut selected);
                if !self.panels.is_empty() {
                    ui.menu_button(tokens.label("Panels", "Paneles"), |ui| {
                        let panels = self.panels.clone();
                        for (id, spec) in panels {
                            ui.horizontal(|ui| {
                                if ui.button(spec.title).on_hover_text(spec.kind).clicked() {
                                    self.open_panel(id);
                                }
                                if spec.floating
                                    && ui.small_button(tokens.label("Float", "Flotar")).clicked()
                                {
                                    self.float_panel(id);
                                }
                            });
                        }
                    });
                }
            });
        });
        egui::Panel::bottom("starter-status").show(ui, |ui| {
            status(ui, message.0, message.1);
            if dirty {
                ui.label(tokens.label("Unsaved changes", "Cambios sin guardar"));
            }
        });
        egui::CentralPanel::default().show(ui, |ui| {
            DockArea::new(&mut self.dock).show_inside(
                ui,
                &mut Viewer {
                    tokens,
                    gallery: &mut self.gallery,
                    design: self.design,
                    panels: &self.panels,
                    show: &mut show,
                },
            );
        });
        if let Some(layout) =
            Layout::read(self.dock.main_surface(), NodeIndex::root()).filter(Layout::valid)
        {
            self.workspace.layout = layout;
        }
        if self.preferences {
            egui::Window::new(tokens.label("Preferences", "Preferencias"))
                .open(&mut self.preferences)
                .show(&ctx, |ui| {
                    ui.checkbox(
                        &mut self.workspace.tokens.dark,
                        tokens.label("Dark theme", "Tema oscuro"),
                    );
                    ui.checkbox(
                        &mut self.workspace.tokens.compact,
                        tokens.label("Compact density", "Densidad compacta"),
                    );
                    ui.checkbox(&mut self.workspace.tokens.spanish, "Español");
                });
        }
        if self.inspector {
            #[cfg(not(target_arch = "wasm32"))]
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("starter-inspector"),
                egui::ViewportBuilder::default()
                    .with_title("Inspector")
                    .with_inner_size([360.0, 260.0]),
                |ui, _| {
                    if ui.input(|i| i.viewport().close_requested())
                        || ui.button(tokens.label("Close", "Cerrar")).clicked()
                    {
                        self.inspector = false;
                    }
                    show(Panel::Inspector, ui);
                },
            );
            #[cfg(target_arch = "wasm32")]
            egui::Window::new("Inspector")
                .open(&mut self.inspector)
                .show(&ctx, |ui| show(Panel::Inspector, ui));
        }
        for id in self.workspace.floating.clone() {
            let spec = self.panels[&id];
            let mut open = true;
            #[cfg(not(target_arch = "wasm32"))]
            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of(("starter-panel", id)),
                egui::ViewportBuilder::default()
                    .with_title(spec.title)
                    .with_inner_size([480., 340.]),
                |ui, _| {
                    if ui.input(|i| i.viewport().close_requested())
                        || ui.button(tokens.label("Close", "Cerrar")).clicked()
                    {
                        open = false;
                    }
                    show(Panel::Custom(id), ui);
                },
            );
            #[cfg(target_arch = "wasm32")]
            egui::Window::new(spec.title)
                .id(egui::Id::unique(("starter-floating", id)))
                .open(&mut open)
                .show(&ctx, |ui| show(Panel::Custom(id), ui));
            if !open {
                self.workspace.floating.remove(&id);
            }
        }
        if self.confirm_close {
            let choice = confirmation(
                &ctx,
                egui::Id::unique("starter-confirm-close"),
                tokens.label("Unsaved changes", "Cambios sin guardar"),
                tokens.label(
                    "Choose what to do with your edits.",
                    "Elige qué hacer con los cambios.",
                ),
                (
                    tokens.label("Continue editing", "Seguir editando"),
                    tokens.label("Save and close", "Guardar y cerrar"),
                    Some(tokens.label("Discard and close", "Descartar y cerrar")),
                ),
            );
            match choice {
                Some(DialogChoice::Cancel) => self.confirm_close = false,
                Some(DialogChoice::Primary) => self.save_requested = true,
                Some(DialogChoice::Secondary) => self.finish_close(&ctx),
                None => {}
            }
        }
        if commands_blocked || self.confirm_close || self.commands_open {
            return None;
        }
        selected.or_else(|| {
            if ctx.text_edit_focused() || egui::Popup::is_any_open(&ctx) {
                return None;
            }
            actions::shortcut(&ctx, actions)
        })
    }
}
struct Viewer<'a, F> {
    tokens: Tokens,
    gallery: &'a mut Gallery,
    design: Design,
    panels: &'a BTreeMap<u64, PanelSpec>,
    show: &'a mut F,
}
impl<F: FnMut(Panel, &mut Ui)> TabViewer for Viewer<'_, F> {
    type Tab = Panel;
    fn id(&mut self, tab: &mut Panel) -> egui::Id {
        egui::Id::unique(("starter-tab", *tab))
    }
    fn title(&mut self, tab: &mut Panel) -> egui::WidgetText {
        match tab {
            Panel::Work => self.tokens.label("Workspace", "Área de trabajo"),
            Panel::Gallery => self.tokens.label("Components", "Componentes"),
            Panel::Inspector => "Inspector",
            Panel::Custom(id) => self.panels.get(id).map_or("Panel", |spec| spec.title),
        }
        .into()
    }
    fn ui(&mut self, ui: &mut Ui, tab: &mut Panel) {
        if *tab == Panel::Gallery {
            self.gallery.show_with_design(ui, self.tokens, self.design);
        } else {
            (self.show)(*tab, ui);
        }
    }
    fn is_closeable(&self, panel: &Panel) -> bool {
        matches!(panel, Panel::Custom(_))
    }
    fn allowed_in_windows(&self, _: &mut Panel) -> bool {
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };
    #[test]
    fn layout_adapter_roundtrips() {
        let shell = Shell::default();
        assert_eq!(
            Layout::read(shell.dock.main_surface(), NodeIndex::root()).unwrap(),
            shell.workspace.layout
        );
    }
    #[test]
    fn custom_instances_can_float_and_restore_independently() {
        let panels = [
            PanelSpec {
                instance: 1,
                kind: "diagnostic",
                title: "A",
                floating: true,
                initially_open: false,
            },
            PanelSpec {
                instance: 2,
                kind: "diagnostic",
                title: "B",
                floating: false,
                initially_open: true,
            },
        ];
        let mut shell = Shell::with_panels(Workspace::default(), &panels);
        assert!(shell.dock.find_tab(&Panel::Custom(2)).is_some());
        assert!(shell.dock.find_tab(&Panel::Custom(1)).is_none());
        assert!(!shell.float_panel(2));
        assert!(!shell.open_panel(99));
        assert!(shell.float_panel(1));
        let json = serde_json::to_string(&shell.workspace).unwrap();
        let mut restored = Shell::with_panels(serde_json::from_str(&json).unwrap(), &panels);
        assert!(restored.workspace.floating.contains(&1));
        assert!(restored.open_panel(1));
        assert!(restored.workspace.floating.is_empty());
        assert!(restored.dock.find_tab(&Panel::Custom(1)).is_some());
    }
    #[test]
    fn restored_panel_occupies_only_one_surface() {
        let panel = PanelSpec {
            instance: 1,
            kind: "custom",
            title: "Custom",
            floating: true,
            initially_open: true,
        };
        let workspace = Workspace {
            layout: Layout::Tabs(vec![Panel::Work, Panel::Custom(1), Panel::Custom(9)], 2),
            floating: BTreeSet::from([1, 9]),
            registered: BTreeSet::from([1, 9]),
            ..Workspace::default()
        };
        let shell = Shell::with_panels(workspace, &[panel]);
        assert!(shell.workspace.floating.is_empty());
        assert_eq!(
            shell.workspace.layout,
            Layout::Tabs(vec![Panel::Work, Panel::Custom(1)], 0)
        );
        assert_eq!(shell.workspace.registered, BTreeSet::from([1]));
    }
    #[test]
    fn gallery_control_is_accessible_and_interactive() {
        let mut gallery = Gallery::default();
        let mut h = Harness::new_ui(|ui| gallery.show(ui, Tokens::default()));
        h.get_by_label("Show grid").click();
        h.run();
        assert_eq!(
            h.get_by_label("Show grid").accesskit_node().toggled(),
            Some(egui::accesskit::Toggled::True)
        );
        h.get_by_label("Preview level");
    }
    #[test]
    fn gallery_reuses_state_widgets_after_switching_sections() {
        let mut gallery = Gallery::default();
        let mut h = Harness::new_ui(|ui| gallery.show(ui, Tokens::default()));
        h.get_by_label("States").click();
        h.run_steps(2);
        h.get_by_label("No items");
        h.get_by_label("Invalid value");
        h.get_by_label("Properties").click();
        h.run();
        h.get_by_label("Length");
    }
    #[test]
    fn gallery_icon_named_button_confirms_reset_and_dismisses_feedback() {
        let mut gallery = Gallery::default();
        let mut h = Harness::new_ui(|ui| gallery.show(ui, Tokens::default()));
        h.get_by_label("Show grid").click();
        h.run();
        h.get_by_label("Reset examples").click();
        h.run_steps(3);
        h.get_by_label("Cancel").click();
        h.run_steps(3);
        assert!(h.query_by_label("Preview level").is_some());

        h.get_by_label("Reset examples").click();
        h.run_steps(3);
        h.get_by_label("Reset").click();
        h.run_steps(3);
        assert!(h.query_by_label("Preview level").is_none());
        h.get_by_label("Examples reset");
        h.get_by_label("Dismiss").click();
        h.run_steps(3);
        assert!(h.query_by_label("Examples reset").is_none());
    }
    #[cfg(all(feature = "visual-evidence", not(target_arch = "wasm32")))]
    #[test]
    fn capture_gallery_interaction() {
        let mut gallery = Gallery::default();
        let mut h = Harness::builder()
            .with_size(egui::vec2(400.0, 290.0))
            .with_render_cursor(false)
            .wgpu()
            .build_ui(|ui| gallery.show(ui, Tokens::default()));
        let before = h.render().unwrap();
        h.get_by_label("Show grid").click();
        h.run();
        h.hover_at(egui::pos2(380.0, 270.0));
        h.run();
        let after = h.render().unwrap();
        assert!(before.pixels().zip(after.pixels()).any(|(a, b)| a != b));
        if let Some(dir) = std::env::var_os("STARTER_VISUAL_EVIDENCE") {
            let dir = std::path::PathBuf::from(dir);
            std::fs::create_dir_all(&dir).unwrap();
            before.save(dir.join("before.png")).unwrap();
            after.save(dir.join("after.png")).unwrap();
            std::fs::write(dir.join("widgets.txt"), format!("{:#?}", h.root())).unwrap();
        }
    }
}
