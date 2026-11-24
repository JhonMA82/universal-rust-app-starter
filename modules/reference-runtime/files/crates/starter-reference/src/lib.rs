#![forbid(unsafe_code)]
use serde_json::{Value, json};
use starter_core::{Application, Error, Request, Response, Session};
use starter_desktop::{Panel, Shell};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
pub type Shared<S> = Arc<Mutex<Application<S>>>;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub fn local_session() -> Session {
    Session {
        id: "local".into(),
        permissions: ["read".into(), "write".into()].into(),
    }
}
pub fn invoke<S: Clone + 'static>(
    app: &Shared<S>,
    id: &str,
    parameters: Value,
    document: Option<u64>,
    revision: Option<u64>,
) -> std::result::Result<Response, Error> {
    app.lock()
        .map_err(|_| Error::new("internal", "application unavailable"))?
        .invoke(
            &local_session(),
            false,
            Request {
                command_id: id.into(),
                parameters,
                session_id: "local".into(),
                document_id: document.map(|id| id.to_string()),
                request_id: None,
                expected_revision: revision,
            },
        )
}
pub trait View {
    fn show(&mut self, panel: Panel, ui: &mut eframe::egui::Ui);
    fn status(&self) -> (String, bool);
    fn dirty(&self) -> bool {
        false
    }
    fn tick(&mut self) {}
}
struct Window<V> {
    shell: Shell,
    view: V,
    title: &'static str,
}
impl<V: View> eframe::App for Window<V> {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Ok(text) = serde_json::to_string(&self.shell.workspace) {
            storage.set_string("starter-workspace", text);
        }
    }
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _: &mut eframe::Frame) {
        self.view.tick();
        let (status, error) = self.view.status();
        self.shell.show(
            ui,
            self.title,
            (&status, error),
            self.view.dirty(),
            |panel, ui| self.view.show(panel, ui),
        );
        ui.ctx().request_repaint_after(Duration::from_millis(150));
    }
}
pub fn launch<S: Clone + Send + 'static, V: View + 'static>(
    title: &'static str,
    app: Application<S>,
    factory: impl FnOnce(&eframe::CreationContext<'_>, Shared<S>) -> V + 'static,
) -> Result<()> {
    let app = Arc::new(Mutex::new(app));
    let mut args = std::env::args().skip(1);
    let mode = args.next();
    let inspect = mode.as_deref() == Some("--ui-inspect");
    match mode.as_deref() {
        Some("--mcp") if args.next().is_none() => {
            return starter_mcp::stdio(starter_mcp::Server::new(app, local_session()));
        }
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
            println!("{title} [--mcp | --mcp-gui | --invoke <JSON> | --ui-inspect]");
            return Ok(());
        }
        None => {}
        _ => return Err("unrecognized arguments; use --help".into()),
    }
    let native = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1120., 760.]),
        ..Default::default()
    };
    eframe::run_native(
        title,
        native,
        Box::new(move |cc| {
            if inspect {
                attach_inspection(&cc.egui_ctx, title)?;
            }
            let workspace = cc
                .storage
                .and_then(|s| s.get_string("starter-workspace"))
                .and_then(|text| serde_json::from_str(&text).ok())
                .unwrap_or_default();
            Ok(Box::new(Window {
                shell: Shell::new(workspace),
                view: factory(cc, app),
                title,
            }))
        }),
    )
    .map_err(|e| e.to_string().into())
}
#[cfg(all(feature = "ui-inspection", debug_assertions))]
fn attach_inspection(ctx: &eframe::egui::Context, title: &str) -> Result<()> {
    ctx.add_plugin(egui_inspection::InspectionPlugin::new(Some(title.into())));
    egui_inspection::serve(ctx, "127.0.0.1:5719")?;
    Ok(())
}
#[cfg(not(all(feature = "ui-inspection", debug_assertions)))]
fn attach_inspection(_: &eframe::egui::Context, _: &str) -> Result<()> {
    Err("UI inspection unavailable".into())
}
