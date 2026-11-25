use eframe::egui::Ui;
use serde_json::json;
use starter_desktop::{Panel, PanelSpec};
use starter_reference::{Shared, TaskPanel, View, invoke};

use crate::domain::Workspace;

pub struct WorkspaceView {
    app: Shared<Workspace>,
    draft: String,
    revision: u64,
    task: TaskPanel,
    message: (String, bool),
}

impl WorkspaceView {
    pub fn new(app: Shared<Workspace>) -> Self {
        Self {
            app,
            draft: String::new(),
            revision: 0,
            task: TaskPanel::default(),
            message: ("Ready".into(), false),
        }
    }
}

impl View for WorkspaceView {
    fn panels(&self) -> &[PanelSpec] {
        &[PanelSpec {
            instance: 101,
            kind: "background-work",
            title: "Background work",
            floating: true,
            initially_open: true,
        }]
    }

    fn status(&self) -> (String, bool) {
        self.message.clone()
    }

    fn show(&mut self, panel: Panel, ui: &mut Ui) {
        if panel == Panel::Custom(101) {
            self.task.show(ui, &self.draft, self.revision);
            return;
        }
        let current = invoke(&self.app, "workspace.name", json!({}), None, None);
        let Ok(current) = current else {
            starter_desktop::status(ui, "Workspace name unavailable", true);
            return;
        };
        if panel == Panel::Inspector {
            ui.label(format!("Domain revision {}", current.revision));
        } else {
            ui.heading(current.result.as_str().unwrap_or_default());
            if starter_desktop::field(ui, "Workspace name", &mut self.draft).changed() {
                self.revision += 1;
            }
            if ui.button("Rename workspace").clicked() {
                self.message = match invoke(
                    &self.app,
                    "workspace.rename",
                    json!({"name": self.draft}),
                    None,
                    Some(current.revision),
                ) {
                    Ok(response) => (format!("Saved revision {}", response.revision), false),
                    Err(error) => (error.message, true),
                };
            }
        }
    }
}
