use crate::{Shared, local_session};
use eframe::egui::{self, Context};
use serde_json::Value;
use starter_core::{Operation, Request};

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
        let make_request = |operation: &Operation, parameters: Value| Request {
            command_id: operation.id.into(),
            parameters,
            session_id: session.id.clone(),
            document_id: document.map(|id| id.to_string()),
            request_id: None,
            expected_revision: None,
        };
        let (revision, available) = {
            let app = app.lock().ok()?;
            let operations: Vec<_> = app
                .discover(&session, true)
                .map(|op| {
                    (
                        op.clone(),
                        app.available(&session, true, &make_request(op, serde_json::json!({}))),
                    )
                })
                .collect();
            (app.revision(), operations)
        };
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
                        for (operation, enabled) in &available {
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
                                .add_enabled(
                                    *enabled,
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
                if let Some((operation, enabled)) = available
                    .iter()
                    .find(|(item, _)| Some(item.id) == self.selected)
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
                    if ui
                        .add_enabled(*enabled, egui::Button::new("Run operation"))
                        .clicked()
                    {
                        match serde_json::from_str(&self.arguments) {
                            Ok(args) => {
                                let mut request = make_request(operation, args);
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
                Some((
                    format!("Result · revision {}: {}", output.revision, output.result),
                    false,
                ))
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
