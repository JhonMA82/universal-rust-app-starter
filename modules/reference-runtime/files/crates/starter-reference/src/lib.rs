#![forbid(unsafe_code)]
mod palette;
#[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
mod task_panel;
use serde_json::{Value, json};
use starter_core::{Application, Error, Request, Response, Session};
use starter_desktop::{ActionSpec, ActionState, Panel, Shell};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
#[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
pub use task_panel::TaskPanel;
pub type Shared<S> = Arc<Mutex<Application<S>>>;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub fn local_session() -> Session {
    Session {
        id: "local".into(),
        permissions: ["read".into(), "write".into()].into(),
    }
}
pub fn invoke<S: 'static>(
    app: &Shared<S>,
    id: &str,
    parameters: Value,
    document: Option<u64>,
    revision: Option<u64>,
) -> std::result::Result<Response, Error> {
    let session = local_session();
    app.lock()
        .map_err(|_| Error::new("internal", "application unavailable"))?
        .invoke(
            &session,
            false,
            operation_request(&session, id, parameters, document, revision),
        )
}

fn operation_request(
    session: &Session,
    id: &str,
    parameters: Value,
    document: Option<u64>,
    revision: Option<u64>,
) -> Request {
    Request {
        command_id: id.into(),
        parameters,
        session_id: session.id.clone(),
        document_id: document.map(|id| id.to_string()),
        request_id: None,
        expected_revision: revision,
    }
}

fn result_feedback(output: Response) -> (String, bool) {
    (
        format!("Result · revision {}: {}", output.revision, output.result),
        false,
    )
}

pub trait View {
    fn show(&mut self, panel: Panel, ui: &mut eframe::egui::Ui);
    fn status(&self) -> (String, bool);
    fn dirty(&self) -> bool {
        false
    }
    fn tick(&mut self) {}
    fn active_document(&self) -> Option<u64> {
        None
    }
    fn design(&self) -> starter_desktop::Design {
        starter_desktop::Design::default()
    }
    fn initial_workspace(&self) -> starter_desktop::Workspace {
        starter_desktop::Workspace::default()
    }
    fn panels(&self) -> &[starter_desktop::PanelSpec] {
        &[]
    }
    /// Register menus, toolbar buttons and shortcuts for public operations.
    fn actions(&self) -> &'static [ActionSpec] {
        &[]
    }
    /// Capture typed operation parameters from the current view before rendering its actions.
    fn action_parameters(&self, _action: &ActionSpec) -> Value {
        json!({})
    }
    fn request_save(&mut self) -> bool {
        false
    }
}
struct Window<V, S> {
    app: Shared<S>,
    palette: palette::Palette,
    command_status: Option<(f64, (String, bool))>,
    shell: Shell,
    view: V,
    title: &'static str,
}
impl<V: View, S: 'static> eframe::App for Window<V, S> {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(text) = serde_json::to_string(&self.shell.workspace) {
            storage.set_string("starter-workspace", text);
        }
    }
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _: &mut eframe::Frame) {
        self.frame(ui);
    }
}
impl<V: View, S: 'static> Window<V, S> {
    fn frame(&mut self, ui: &mut eframe::egui::Ui) {
        self.view.tick();
        let time = ui.input(|input| input.time);
        let (status, error) = self
            .command_status
            .as_ref()
            .filter(|(expires, _)| *expires > time)
            .map_or_else(|| self.view.status(), |(_, message)| message.clone());
        let document = self.view.active_document();
        self.shell.workspace.document = document;
        let session = local_session();
        let specs = self.view.actions();
        let mut requests: Vec<_> = specs
            .iter()
            .map(|spec| {
                operation_request(
                    &session,
                    spec.command,
                    self.view.action_parameters(spec),
                    document,
                    None,
                )
            })
            .collect();
        let actions: Vec<_> = {
            let app = (!specs.is_empty()).then(|| self.app.lock().ok()).flatten();
            specs
                .iter()
                .zip(&mut requests)
                .map(|(&spec, request)| {
                    let enabled = app.as_ref().is_some_and(|app| {
                        request.expected_revision = Some(app.revision());
                        app.available(&session, true, request)
                    });
                    ActionState { spec, enabled }
                })
                .collect()
        };
        let selected = self.shell.show_with_actions(
            ui,
            self.title,
            (&status, error),
            self.view.dirty(),
            &actions,
            |panel, ui| self.view.show(panel, ui),
        );
        if let Some(index) = selected
            && let Some(request) = requests.into_iter().nth(index)
        {
            let result = if document == self.view.active_document() {
                self.app
                    .lock()
                    .map_err(|_| Error::new("internal", "application unavailable"))
                    .and_then(|mut app| app.invoke(&session, true, request))
            } else {
                Err(Error::new("conflict", "The active document changed"))
            };
            self.command_status = Some((
                time + 5.0,
                result.map_or_else(|error| (error.message, true), result_feedback),
            ));
            ui.ctx().request_repaint();
        }
        if self.shell.take_save_request() && self.view.request_save() && !self.view.dirty() {
            self.shell.finish_close(ui.ctx());
        }
        if self.shell.commands_open
            && let Some(message) = self.palette.show(
                ui.ctx(),
                &mut self.shell.commands_open,
                &self.app,
                self.view.active_document(),
            )
        {
            self.command_status = Some((time + 5.0, message));
            ui.ctx().request_repaint();
        }
        ui.ctx().request_repaint_after(Duration::from_millis(150));
    }
}
pub fn window<V: View + 'static, S: 'static>(
    cc: &eframe::CreationContext<'_>,
    title: &'static str,
    app: Shared<S>,
    view: V,
) -> impl eframe::App + use<V, S> {
    let workspace = cc
        .storage
        .and_then(|s| s.get_string("starter-workspace"))
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_else(|| view.initial_workspace());
    let mut shell = Shell::with_panels(workspace, view.panels());
    shell.set_design(view.design());
    Window {
        shell,
        view,
        title,
        app,
        palette: Default::default(),
        command_status: None,
    }
}

pub fn standard_window(
    cc: &eframe::CreationContext<'_>,
    title: &'static str,
) -> impl eframe::App + use<> {
    let app = Arc::new(Mutex::new(starter_core::sample::application()));
    window(cc, title, app.clone(), SettingsView::new(app))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn standard_launch(title: &'static str) -> Result<()> {
    launch(title, starter_core::sample::application(), |_, app| {
        SettingsView::new(app)
    })
}

struct SettingsView {
    app: Shared<starter_core::sample::Settings>,
    draft: String,
    status: (String, bool),
    #[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
    task: TaskPanel,
    #[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
    draft_revision: u64,
}
impl SettingsView {
    fn new(app: Shared<starter_core::sample::Settings>) -> Self {
        Self {
            app,
            draft: String::new(),
            status: ("Ready".into(), false),
            #[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
            task: TaskPanel::default(),
            #[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
            draft_revision: 0,
        }
    }
}
impl View for SettingsView {
    fn actions(&self) -> &'static [ActionSpec] {
        use eframe::egui::{Key, KeyboardShortcut, Modifiers};
        const ACTIONS: &[ActionSpec] = &[ActionSpec {
            command: "workspace.name",
            label: "Show workspace name",
            menu: Some("Workspace"),
            toolbar: true,
            shortcut: Some(KeyboardShortcut::new(
                Modifiers::COMMAND.plus(Modifiers::SHIFT),
                Key::L,
            )),
        }];
        ACTIONS
    }

    fn show(&mut self, panel: Panel, ui: &mut eframe::egui::Ui) {
        let mut app = self.app.lock().expect("settings mutex");
        if let Ok(name) = starter_core::sample::invoke(&mut app, "workspace.name", json!({})) {
            ui.heading(name.result.as_str().unwrap_or_default());
        }
        if panel == Panel::Inspector {
            ui.label(format!("Revision {}", app.revision()));
        } else {
            let _input = starter_desktop::field(ui, "Workspace name", &mut self.draft);
            #[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
            if _input.changed() {
                self.draft_revision = self.draft_revision.saturating_add(1);
            }
            if ui.button("Rename workspace").clicked() {
                self.status = match starter_core::sample::invoke(
                    &mut app,
                    "workspace.rename",
                    json!({"name": self.draft}),
                ) {
                    Ok(result) => (format!("Saved revision {}", result.revision), false),
                    Err(error) => (error.message, true),
                };
            }
            #[cfg(all(feature = "tasks", not(target_arch = "wasm32")))]
            self.task.show(ui, &self.draft, self.draft_revision);
        }
    }
    fn status(&self) -> (String, bool) {
        self.status.clone()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn launch<S: Send + 'static, V: View + 'static>(
    title: &'static str,
    app: Application<S>,
    factory: impl FnOnce(&eframe::CreationContext<'_>, Shared<S>) -> V + 'static,
) -> Result<()> {
    let app = Arc::new(Mutex::new(app));
    let mut args = std::env::args().skip(1);
    let mode = args.next();
    let inspect = mode.as_deref() == Some("--ui-inspect");
    #[cfg(feature = "mcp")]
    let mut live = None;
    match mode.as_deref() {
        #[cfg(feature = "mcp")]
        Some("--sessions") if args.next().is_none() => {
            let directory =
                std::env::var_os("STARTER_SESSION_DIR").ok_or("set STARTER_SESSION_DIR")?;
            println!(
                "{}",
                serde_json::to_string(&starter_mcp::live::discover(std::path::Path::new(
                    &directory
                ))?)?
            );
            return Ok(());
        }
        #[cfg(feature = "mcp")]
        Some("--mcp-live" | "--mcp-live-headless") if args.next().is_none() => {
            let directory =
                std::env::var_os("STARTER_SESSION_DIR").ok_or("set STARTER_SESSION_DIR")?;
            let token = std::env::var("STARTER_MCP_TOKEN")?;
            let service_app = app.clone();
            let service =
                starter_mcp::live::start(std::path::Path::new(&directory), title, &token, |id| {
                    starter_mcp::Server::new(
                        service_app,
                        Session {
                            id,
                            ..local_session()
                        },
                    )
                })?;
            eprintln!("Live MCP: {}", service.instance.endpoint());
            if mode.as_deref() == Some("--mcp-live-headless") {
                // Parent owns stdin. EOF disconnects the verification process.
                let mut line = String::new();
                std::io::stdin().read_line(&mut line)?;
                return Ok(());
            }
            live = Some(service);
        }
        #[cfg(feature = "mcp")]
        Some("--mcp") if args.next().is_none() => {
            return starter_mcp::stdio(starter_mcp::Server::new(app, local_session()));
        }
        #[cfg(feature = "mcp")]
        Some("--mcp-gui") if args.next().is_none() => {
            let server = starter_mcp::Server::new(app.clone(), local_session());
            std::thread::Builder::new()
                .name("application-mcp".into())
                .spawn(move || {
                    if let Err(e) = starter_mcp::stdio(server) {
                        eprintln!("{e}");
                    }
                })?;
        }
        Some("--operations") if args.next().is_none() => {
            let app = app.lock().map_err(|_| "application unavailable")?;
            let session = local_session();
            println!(
                "{}",
                json!({"revision": app.revision(), "session_id":session.id,
                                   "operations": app.discover(&session,true).collect::<Vec<_>>() })
            );
            return Ok(());
        }
        Some("--invoke") => {
            let input = args.next().ok_or("application request JSON required")?;
            if args.next().is_some() {
                return Err("unexpected arguments".into());
            }
            let response = app.lock().map_err(|_| "application unavailable")?.invoke(
                &local_session(),
                true,
                serde_json::from_str(&input)?,
            );
            match response {
                Ok(value) => println!("{}", json!({"ok":value})),
                Err(error) => {
                    println!("{}", json!({"error":error}));
                    return Err(error.message.into());
                }
            }
            return Ok(());
        }
        Some("--ui-inspect") if args.next().is_none() => {
            if !cfg!(all(feature = "ui-inspection", debug_assertions)) {
                return Err("requires a debug build with UI inspection enabled".into());
            }
        }
        Some("--help" | "-h") => {
            println!("{title} [--operations | --invoke <JSON> | --ui-inspect]");
            if cfg!(feature = "mcp") {
                println!("MCP: --mcp | --mcp-gui | --mcp-live | --sessions");
            }
            return Ok(());
        }
        None => {}
        _ => return Err("unrecognized arguments; use --help".into()),
    }
    let native = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1120., 760.]),
        ..Default::default()
    };
    let result = eframe::run_native(
        title,
        native,
        Box::new(move |cc| {
            if inspect {
                attach_inspection(&cc.egui_ctx, title)?;
            }
            Ok(Box::new(window(cc, title, app.clone(), factory(cc, app))))
        }),
    )
    .map_err(|e| e.to_string().into());
    #[cfg(feature = "mcp")]
    drop(live);
    result
}
#[cfg(all(
    not(target_arch = "wasm32"),
    feature = "ui-inspection",
    debug_assertions
))]
fn attach_inspection(ctx: &eframe::egui::Context, title: &str) -> Result<()> {
    ctx.add_plugin(egui_inspection::InspectionPlugin::new(Some(title.into())));
    egui_inspection::serve(ctx, "127.0.0.1:5719")?;
    Ok(())
}
#[cfg(all(
    not(target_arch = "wasm32"),
    not(all(feature = "ui-inspection", debug_assertions))
))]
fn attach_inspection(_: &eframe::egui::Context, _: &str) -> Result<()> {
    Err("UI inspection unavailable".into())
}

#[cfg(feature = "notes")]
pub fn notes_window(
    cc: &eframe::CreationContext<'_>,
    title: &'static str,
) -> impl eframe::App + use<> {
    let app = Arc::new(Mutex::new(starter_notes::application()));
    window(
        cc,
        title,
        app.clone(),
        NotesView {
            app,
            current: 1,
            draft: None,
            status: ("Ready".into(), false),
        },
    )
}

#[cfg(all(feature = "notes", not(target_arch = "wasm32")))]
pub fn notes_launch(title: &'static str) -> Result<()> {
    launch(title, starter_notes::application(), |_, app| NotesView {
        app,
        current: 1,
        draft: None,
        status: ("Ready".into(), false),
    })
}

#[cfg(feature = "notes")]
struct NotesView {
    app: Shared<starter_notes::Domain>,
    current: u64,
    draft: Option<(u64, String)>,
    status: (String, bool),
}
#[cfg(feature = "notes")]
impl View for NotesView {
    fn active_document(&self) -> Option<u64> {
        Some(self.current)
    }
    fn status(&self) -> (String, bool) {
        self.status.clone()
    }

    fn show(&mut self, panel: Panel, ui: &mut eframe::egui::Ui) {
        let (notes, current, revision) = {
            let app = self.app.lock().expect("notes mutex");
            (
                app.state().notes.keys().copied().collect::<Vec<_>>(),
                starter_notes::note(app.state(), self.current).ok(),
                app.revision(),
            )
        };
        if panel == Panel::Inspector {
            ui.label(format!("{} open notes · revision {revision}", notes.len()));
            return;
        }
        ui.horizontal_wrapped(|ui| {
            let new_note = ui.button("New note").on_hover_text("Alt+N").clicked();
            let shortcut = ui.input_mut(|input| {
                input.consume_key(eframe::egui::Modifiers::ALT, eframe::egui::Key::N)
            });
            if (new_note || shortcut)
                && let Ok(result) = invoke(&self.app, "notes.new", json!({}), None, Some(revision))
                && let Some(id) = result.result.as_u64()
            {
                self.current = id;
                self.draft = None;
                self.status = (format!("Note {id} created"), false);
                #[cfg(target_arch = "wasm32")]
                if let Some(status) = web_sys::window()
                    .and_then(|window| window.document())
                    .and_then(|document| document.get_element_by_id("starter-note-status"))
                {
                    status.set_text_content(Some(&self.status.0));
                }
            }
            for note in notes {
                if ui
                    .selectable_label(self.current == note, format!("Note {note}"))
                    .clicked()
                {
                    self.current = note;
                    self.draft = None;
                }
            }
        });
        if let Some(current) = current.filter(|note| note.id == self.current) {
            let (version, draft) = self.draft.get_or_insert((current.revision, current.text));
            let label = ui.label("Note body");
            ui.text_edit_multiline(draft).labelled_by(label.id);
            let operation = if ui.button("Apply edit").clicked() {
                Some(("notes.edit", json!({"text":draft,"if_revision":version})))
            } else if ui.button("Undo").clicked() {
                Some(("notes.undo", json!({})))
            } else if ui.button("Redo").clicked() {
                Some(("notes.redo", json!({})))
            } else {
                None
            };
            if let Some((id, args)) = operation {
                match invoke(&self.app, id, args, Some(self.current), Some(revision)) {
                    Ok(_) => {
                        self.draft = None;
                        self.status = ("Updated note".into(), false);
                    }
                    Err(err) => self.status = (err.message, true),
                }
            }
        }
    }
}

#[cfg(all(test, feature = "ui-inspection", not(target_arch = "wasm32")))]
mod inspection_tests {

    #[test]
    fn official_inspection_plugin_returns_the_gallery_tree() {
        let mut gallery = starter_desktop::Gallery::default();
        let mut harness = egui_kittest::Harness::new_ui(|ui| gallery.show(ui, Default::default()));
        harness
            .ctx
            .add_plugin(egui_inspection::InspectionPlugin::new(Some(
                "Starter".into(),
            )));
        let (send, receive) = std::sync::mpsc::channel();
        harness
            .ctx
            .with_plugin(|plugin: &mut egui_inspection::InspectionPlugin| {
                plugin.submit(egui_inspection::Request::GetTree, move |response| {
                    let _ = send.send(response);
                });
            });
        harness.run_steps(3);
        let response = receive
            .try_recv()
            .expect("the next frames served an inspection request");
        if let egui_inspection::Response::Tree {
            accesskit: Some(tree),
            ..
        } = response
        {
            assert!(
                tree.nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some("Show grid"))
            );
        } else {
            panic!("missing gallery tree: {response:?}");
        }
    }
}

#[cfg(test)]
mod action_tests {
    use super::*;
    use eframe::egui::{Key, KeyboardShortcut, Modifiers, Ui};
    use egui_kittest::{Harness, kittest::Queryable};

    struct TestView;

    impl View for TestView {
        fn show(&mut self, _panel: Panel, _ui: &mut Ui) {}

        fn status(&self) -> (String, bool) {
            ("Ready".to_owned(), false)
        }

        fn actions(&self) -> &'static [ActionSpec] {
            const ACTIONS: &[ActionSpec] = &[
                ActionSpec {
                    command: "test.add",
                    label: "Add from menu",
                    menu: Some("Project commands"),
                    toolbar: false,
                    shortcut: None,
                },
                ActionSpec {
                    command: "test.add",
                    label: "Add from toolbar",
                    menu: None,
                    toolbar: true,
                    shortcut: Some(KeyboardShortcut::new(Modifiers::ALT, Key::L)),
                },
                ActionSpec {
                    command: "test.private",
                    label: "Private action",
                    menu: None,
                    toolbar: true,
                    shortcut: Some(KeyboardShortcut::new(Modifiers::ALT, Key::K)),
                },
            ];
            ACTIONS
        }

        fn action_parameters(&self, action: &ActionSpec) -> Value {
            if action.label == "Add from menu" {
                json!(3)
            } else {
                json!(2)
            }
        }

        fn active_document(&self) -> Option<u64> {
            Some(7)
        }
    }

    #[test]
    fn menus_toolbar_and_shortcuts_dispatch_the_public_operation() {
        let mut engine = Application::new(0, 10);
        for (id, public) in [("test.add", true), ("test.private", false)] {
            engine
                .command(
                    starter_core::Definition {
                        id,
                        description: "Add to the test state",
                        permission: "write",
                        public,
                        available: |_, request| request.document_id.as_deref() == Some("7"),
                    },
                    starter_core::Kind::Command,
                    |state, value: i32, request| {
                        assert!(request.expected_revision.is_some());
                        *state += value;
                        Ok(*state)
                    },
                )
                .unwrap();
        }
        let app = Arc::new(Mutex::new(engine));
        let mut window = Window {
            app: app.clone(),
            palette: Default::default(),
            command_status: None,
            // A single tab keeps this test focused on actions. The upstream dock splitter
            // has a role, but egui_dock does not currently give it an accessible name.
            shell: Shell::new(starter_desktop::Workspace {
                layout: starter_desktop::Layout::Tabs(vec![Panel::Work], 0),
                ..Default::default()
            }),
            view: TestView,
            title: "Registered action test",
        };
        let mut harness = Harness::new_ui(|ui| window.frame(ui));
        let state = || *app.lock().unwrap().state();

        harness.get_by_label_contains("Add from toolbar").click();
        harness.run();
        assert_eq!(state(), 2);
        harness.get_by_label("Project commands").click();
        harness.run();
        harness.get_by_label("Add from menu").click();
        harness.run();
        assert_eq!(state(), 5);
        harness.key_down_modifiers(Modifiers::ALT, Key::L);
        harness.run();
        assert_eq!(state(), 7);
        harness.key_down_modifiers(Modifiers::ALT, Key::L);
        harness.run();
        assert_eq!(state(), 7, "repeated key down must not execute a command");
        harness.key_up_modifiers(Modifiers::ALT, Key::L);
        harness.key_combination_modifiers(Modifiers::ALT, &[Key::K]);
        harness.run();
        assert_eq!(state(), 7, "private commands stay unavailable");
    }
}
