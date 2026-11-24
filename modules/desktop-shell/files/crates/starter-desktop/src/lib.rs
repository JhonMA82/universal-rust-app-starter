#![forbid(unsafe_code)]
use egui::{Context, Ui};
use egui_dock::{DockArea, DockState, Node, NodeIndex, TabViewer, Tree};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Panel {
    Work,
    Inspector,
    Gallery,
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
        if depth > 2 {
            return false;
        }
        match self {
            Self::Tabs(tabs, active) if *active < tabs.len() && tabs.len() <= 3 => {
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
            && seen.len() == 3
            && [Panel::Work, Panel::Gallery, Panel::Inspector]
                .iter()
                .all(|p| seen.contains(p))
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
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Tokens {
    pub dark: bool,
    pub compact: bool,
    pub spanish: bool,
}
impl Default for Tokens {
    fn default() -> Self {
        Self {
            dark: true,
            compact: false,
            spanish: false,
        }
    }
}
impl Tokens {
    pub fn label(self, en: &'static str, es: &'static str) -> &'static str {
        if self.spanish { es } else { en }
    }
    pub fn apply(self, ctx: &Context) {
        ctx.set_theme(if self.dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        });
        ctx.style_mut_of(ctx.theme(), |style| {
            let gap = if self.compact { 4.0 } else { 8.0 };
            style.spacing.item_spacing = egui::vec2(gap, gap);
            style.spacing.button_padding = egui::vec2(gap + 1.0, gap / 2.0);
            style.spacing.interact_size.y = if self.compact { 23.0 } else { 30.0 };
        });
    }
}
pub fn status(ui: &mut Ui, message: &str, error: bool) {
    if error {
        ui.colored_label(ui.visuals().error_fg_color, message);
    } else {
        ui.label(message);
    }
}
#[derive(Default)]
pub struct Gallery {
    grid: bool,
    level: u32,
    text: String,
}
impl Gallery {
    pub fn show(&mut self, ui: &mut Ui, tokens: Tokens) {
        ui.heading(tokens.label("Components", "Componentes"));
        ui.checkbox(
            &mut self.grid,
            tokens.label("Show grid", "Mostrar cuadrícula"),
        );
        ui.add(
            egui::TextEdit::singleline(&mut self.text)
                .hint_text(tokens.label("Filter text", "Filtrar texto")),
        );
        ui.add(egui::Slider::new(&mut self.level, 0..=100).text(tokens.label("Level", "Nivel")));
        ui.add(egui::ProgressBar::new(self.level as f32 / 100.0));
        ui.add_enabled(
            false,
            egui::Button::new(tokens.label("Disabled", "Deshabilitado")),
        );
        status(ui, tokens.label("Invalid value", "Valor inválido"), true);
    }
}
#[derive(Default, Serialize, Deserialize)]
pub struct Workspace {
    pub layout: Layout,
    pub tokens: Tokens,
    pub document: Option<u64>,
}
pub struct Shell {
    pub workspace: Workspace,
    dock: DockState<Panel>,
    gallery: Gallery,
    palette: bool,
    inspector: bool,
    confirm_close: bool,
    approved_close: bool,
}
impl Default for Shell {
    fn default() -> Self {
        Self::new(Workspace::default())
    }
}
impl Shell {
    pub fn new(mut workspace: Workspace) -> Self {
        if !workspace.layout.valid() {
            workspace.layout = Layout::default();
        }
        let mut dock = DockState::new(workspace.layout.tabs());
        workspace
            .layout
            .install(dock.main_surface_mut(), NodeIndex::root());
        Self {
            workspace,
            dock,
            gallery: Gallery::default(),
            palette: false,
            inspector: false,
            confirm_close: false,
            approved_close: false,
        }
    }
    pub fn show(
        &mut self,
        ui: &mut Ui,
        title: &str,
        message: (&str, bool),
        dirty: bool,
        mut show: impl FnMut(Panel, &mut Ui),
    ) {
        let ctx = ui.ctx().clone();
        let tokens = self.workspace.tokens;
        tokens.apply(&ctx);
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::COMMAND,
                egui::Key::P,
            ))
        }) {
            self.palette = !self.palette;
        }
        if dirty && !self.approved_close && ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }
        egui::Panel::top("starter-toolbar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(title);
                self.palette |= ui
                    .button(tokens.label("Preferences", "Preferencias"))
                    .clicked();
                self.inspector |= ui
                    .button(tokens.label("Inspector window", "Ventana de inspección"))
                    .clicked();
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
                    show: &mut show,
                },
            );
        });
        if let Some(layout) =
            Layout::read(self.dock.main_surface(), NodeIndex::root()).filter(Layout::valid)
        {
            self.workspace.layout = layout;
        }
        if self.palette {
            egui::Window::new(tokens.label("Preferences", "Preferencias"))
                .open(&mut self.palette)
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
        if self.confirm_close {
            egui::Window::new(tokens.label("Unsaved changes", "Cambios sin guardar")).show(
                &ctx,
                |ui| {
                    if ui
                        .button(tokens.label("Continue editing", "Seguir editando"))
                        .clicked()
                    {
                        self.confirm_close = false;
                    }
                    if ui
                        .button(tokens.label("Discard and close", "Descartar y cerrar"))
                        .clicked()
                    {
                        self.approved_close = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                },
            );
        }
    }
}
struct Viewer<'a, F> {
    tokens: Tokens,
    gallery: &'a mut Gallery,
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
        }
        .into()
    }
    fn ui(&mut self, ui: &mut Ui, tab: &mut Panel) {
        if *tab == Panel::Gallery {
            self.gallery.show(ui, self.tokens);
        } else {
            (self.show)(*tab, ui);
        }
    }
    fn is_closeable(&self, _: &Panel) -> bool {
        false
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
    fn gallery_control_is_accessible_and_interactive() {
        let mut gallery = Gallery::default();
        let mut h = Harness::new_ui(|ui| gallery.show(ui, Tokens::default()));
        h.get_by_label("Show grid").click();
        h.run();
        assert_eq!(
            h.get_by_label("Show grid").accesskit_node().toggled(),
            Some(egui::accesskit::Toggled::True)
        );
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
