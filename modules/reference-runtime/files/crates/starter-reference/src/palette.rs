use crate::{Shared, local_session, operation_request, result_feedback};
use eframe::egui::{self, Context};
use serde_json::Value;
use starter_core::Operation;

#[derive(Default)]
pub struct Palette {
    search: String,
    selected: Option<&'static str>,
    arguments: String,
    error: Option<String>,
}

impl Palette {
    pub fn show<S: 'static>(
        &mut self,
        ctx: &Context,
        open: &mut bool,
        app: &Shared<S>,
        document: Option<u64>,
    ) -> Option<(String, bool)> {
        let session = local_session();
        let make_request = |operation: &Operation, parameters: Value| {
            operation_request(&session, operation.id, parameters, document, None)
        };
        let operations: Vec<_> = app.lock().ok()?.discover(&session, true).cloned().collect();
        let mut invoke = None;
        egui::Window::new("Command palette · Ctrl/Cmd+P")
            .open(open)
            .default_width(430.)
            .show(ctx, |ui| {
                ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Search commands"));
                let search = self.search.to_lowercase();
                egui::ScrollArea::vertical()
                    .max_height(220.)
                    .show(ui, |ui| {
                        let mut previous = "";
                        for operation in &operations {
                            if !operation.id.contains(&search)
                                && !operation.description.to_lowercase().contains(&search)
                            {
                                continue;
                            }
                            let group = operation.id.split('.').next().unwrap_or("general");
                            if group != previous {
                                ui.strong(group);
                                previous = group;
                            }
                            if ui
                                .add(
                                    egui::Button::new(operation.id)
                                        .selected(self.selected == Some(operation.id)),
                                )
                                .on_hover_text(operation.description)
                                .clicked()
                            {
                                self.selected = Some(operation.id);
                                self.arguments = "{}".into();
                                self.error = None;
                            }
                        }
                    });
                if let Some(operation) = operations
                    .iter()
                    .find(|item| Some(item.id) == self.selected)
                {
                    ui.separator();
                    ui.label(operation.description);
                    ui.collapsing("Parameter schema", |ui| {
                        ui.monospace(
                            serde_json::to_string_pretty(&operation.input_schema)
                                .unwrap_or_default(),
                        );
                    });
                    let label = ui.label("Parameters (JSON)");
                    ui.text_edit_multiline(&mut self.arguments)
                        .labelled_by(label.id);
                    if let Some(error) = &self.error {
                        starter_desktop::status(ui, error, true);
                    }
                    let request = serde_json::from_str(&self.arguments)
                        .map(|args| make_request(operation, args));
                    let (revision, enabled) = match app.lock() {
                        Ok(app) => (
                            app.revision(),
                            request
                                .as_ref()
                                .map_or(true, |request| app.available(&session, true, request)),
                        ),
                        Err(_) => return,
                    };
                    if !enabled {
                        ui.label("Unavailable for the current document or parameters");
                    }
                    if ui
                        .add_enabled(enabled, egui::Button::new("Run operation"))
                        .clicked()
                    {
                        match request {
                            Ok(mut request) => {
                                request.expected_revision = Some(revision);
                                invoke = Some(request);
                            }
                            Err(error) => self.error = Some(error.to_string()),
                        }
                    }
                }
            });
        let request = invoke?;
        let mut app = app.lock().ok()?;
        match app.invoke(&session, true, request) {
            Ok(output) => {
                *open = false;
                self.error = None;
                Some(result_feedback(output))
            }
            Err(error) => {
                self.error = Some(error.message);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};
    use std::sync::{Arc, Mutex};

    #[test]
    fn configured_parameters_enable_an_operation() {
        let mut engine = starter_core::Application::new(0, 2);
        engine
            .command(
                starter_core::Definition {
                    id: "data.set",
                    description: "Set data",
                    permission: "write",
                    public: true,
                    available: |_, request| request.parameters.as_i64() == Some(3),
                },
                starter_core::Kind::Command,
                |state, value: i32, _| {
                    *state = value;
                    Ok(value)
                },
            )
            .unwrap();
        let app = Arc::new(Mutex::new(engine));
        let mut open = true;
        let mut palette = Palette {
            selected: Some("data.set"),
            arguments: "3".into(),
            ..Default::default()
        };
        let mut harness = Harness::new_ui(|ui| {
            palette.show(ui.ctx(), &mut open, &app, None);
        });
        harness.get_by_label("Run operation").click();
        harness.run();
        assert_eq!(*app.lock().unwrap().state(), 3);
    }

    #[test]
    fn palette_button_dispatches_registered_operation() {
        let app = Arc::new(Mutex::new(starter_core::sample::application()));
        let mut open = true;
        let mut palette = Palette {
            selected: Some("workspace.rename"),
            arguments: r#"{"name":"Palette verification"}"#.into(),
            ..Default::default()
        };
        let mut harness = Harness::new_ui(|ui| {
            palette.show(ui.ctx(), &mut open, &app, None);
        });
        harness.get_by_label("Run operation").click();
        harness.run();
        drop(harness);
        assert!(!open);
        let mut engine = app.lock().unwrap();
        let answer =
            starter_core::sample::invoke(&mut engine, "workspace.name", serde_json::json!({}))
                .unwrap();
        assert_eq!(answer.result, "Palette verification");
        assert_eq!(answer.revision, 1);
    }
}
