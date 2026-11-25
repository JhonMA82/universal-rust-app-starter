#![forbid(unsafe_code)]
use eframe::egui;
use serde_json::json;
use starter_desktop::Panel;
use starter_documents::storage::{Document, Recovery};
use starter_inspection::{InspectionCheck, InspectionReport, format};
use starter_reference::{Shared, View, invoke};
use std::{
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const AUTOSAVE: Duration = Duration::from_secs(15);
const UNDO_CAPACITY: usize = 128;

struct Inspector {
    app: Shared<Document<InspectionReport>>,
    draft: InspectionReport,
    draft_revision: u64,
    path: String,
    recovery: Recovery,
    last_autosave: Instant,
    status: (String, bool),
}

impl Inspector {
    fn new(app: Shared<Document<InspectionReport>>) -> Self {
        let (draft, draft_revision) = {
            let app = app.lock().expect("application mutex");
            (
                app.state().history().value().clone(),
                app.state().history().revision(),
            )
        };
        Self {
            app,
            draft,
            draft_revision,
            path: "inspection.json".into(),
            recovery: Recovery::new(
                ".inspection-recovery",
                format!(
                    "report-{}-{}",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos()
                ),
            ),
            last_autosave: Instant::now(),
            status: ("Ready".into(), false),
        }
    }

    fn sync(&mut self) {
        let app = self.app.lock().expect("application mutex");
        self.draft = app.state().history().value().clone();
        self.draft_revision = app.state().history().revision();
    }

    fn operation(&mut self, id: &str, parameters: serde_json::Value) -> bool {
        match invoke(&self.app, id, parameters, None, None) {
            Ok(_) => {
                self.sync();
                true
            }
            Err(error) => {
                self.status = (error.message, true);
                false
            }
        }
    }

    fn apply(&mut self) -> bool {
        let parameters = json!({"report": self.draft, "if_revision": self.draft_revision});
        self.operation("inspection.edit", parameters)
    }

    fn undo(&mut self) -> bool {
        self.operation("inspection.undo", json!({}))
    }

    fn redo(&mut self) -> bool {
        self.operation("inspection.redo", json!({}))
    }

    fn save(&mut self) -> Result<(), starter_documents::Error> {
        let path = PathBuf::from(&self.path);
        let mut app = self.app.lock().expect("application mutex");
        app.state_mut().save(&path)?;
        app.state().discard_recovery(&self.recovery)?;
        drop(app);
        self.sync();
        Ok(())
    }

    fn open(&mut self) -> Result<(), starter_documents::Error> {
        let path = PathBuf::from(&self.path);
        let document = Document::open(format(), &path, UNDO_CAPACITY)?;
        let mut app = self.app.lock().expect("application mutex");
        *app.state_mut() = document;
        drop(app);
        self.sync();
        Ok(())
    }

    fn autosave(&mut self) -> bool {
        let app = self.app.lock().expect("application mutex");
        match app.state().autosave(&self.recovery) {
            Ok(Some(report)) => {
                self.status = (format!("Autosaved {} bytes", report.bytes), false);
                true
            }
            Ok(None) => false,
            Err(error) => {
                self.status = (format!("Autosave: {error}"), true);
                false
            }
        }
    }

    fn recover(&mut self) -> Result<bool, starter_documents::Error> {
        let recovered = {
            let mut app = self.app.lock().expect("application mutex");
            app.state_mut().recover(&self.recovery)?
        };
        if recovered {
            self.sync();
        }
        Ok(recovered)
    }

    fn discard_recovery(&mut self) -> Result<(), starter_documents::Error> {
        let app = self.app.lock().expect("application mutex");
        app.state().discard_recovery(&self.recovery)
    }
}

impl View for Inspector {
    fn show(&mut self, panel: Panel, ui: &mut egui::Ui) {
        if panel == Panel::Custom(110) {
            ui.heading("Recovery");
            ui.label(self.recovery.path().display().to_string());
            if !self.recovery.exists() {
                ui.label("No autosaved draft");
                return;
            }
            if ui.button("Recover autosaved draft").clicked() {
                self.status = match self.recover() {
                    Ok(true) => ("Recovered autosaved draft".into(), false),
                    Ok(false) => ("Autosaved draft is not newer".into(), false),
                    Err(error) => (error.to_string(), true),
                };
            }
            if ui.button("Discard autosaved draft").clicked() {
                self.status = match self.discard_recovery() {
                    Ok(()) => ("Discarded autosaved draft".into(), false),
                    Err(error) => (error.to_string(), true),
                };
            }
            return;
        }
        if panel == Panel::Inspector {
            let (passed, total, revision, dirty) = {
                let app = self.app.lock().expect("application mutex");
                let report = app.state().history().value();
                (
                    report.checks.iter().filter(|check| check.passed).count(),
                    report.checks.len(),
                    app.state().history().revision(),
                    app.state().history().dirty(),
                )
            };
            ui.label(format!("{passed}/{total} checks passed"));
            ui.label(format!(
                "Revision {revision} · {}",
                if dirty { "unsaved" } else { "saved" }
            ));
            return;
        }

        ui.heading("Inspection report");
        egui::Grid::new("inspection-header")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Reference");
                ui.text_edit_singleline(&mut self.draft.reference);
                ui.end_row();
                ui.label("Site");
                ui.text_edit_singleline(&mut self.draft.site);
                ui.end_row();
            });
        ui.separator();
        let mut remove = None;
        for (index, check) in self.draft.checks.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.checkbox(&mut check.passed, "");
                ui.text_edit_singleline(&mut check.label);
                if ui.small_button("Remove").clicked() {
                    remove = Some(index);
                }
            });
            let label = ui.label("Note");
            ui.text_edit_singleline(&mut check.note).labelled_by(label.id);
        }
        if let Some(index) = remove {
            self.draft.checks.remove(index);
        }
        if ui.button("Add check").clicked() {
            self.draft.checks.push(InspectionCheck {
                label: "New check".into(),
                passed: false,
                note: String::new(),
            });
        }
        ui.horizontal(|ui| {
            if ui.button("Apply changes").clicked() && self.apply() {
                self.status = ("Applied".into(), false);
            }
            if ui.button("Undo").clicked() && self.undo() {
                self.status = ("Undo complete".into(), false);
            }
            if ui.button("Redo").clicked() && self.redo() {
                self.status = ("Redo complete".into(), false);
            }
        });
        ui.separator();
        let label = ui.label("File path");
        ui.text_edit_singleline(&mut self.path).labelled_by(label.id);
        ui.horizontal(|ui| {
            if ui.button("Open").clicked() {
                self.status = match self.open() {
                    Ok(()) => ("Opened".into(), false),
                    Err(error) => (error.to_string(), true),
                };
            }
            if ui.button("Save").clicked() {
                self.status = match self.save() {
                    Ok(()) => ("Saved".into(), false),
                    Err(error) => (error.to_string(), true),
                };
            }
        });
    }

    fn status(&self) -> (String, bool) {
        self.status.clone()
    }

    fn dirty(&self) -> bool {
        let app = self.app.lock().expect("application mutex");
        app.state().history().dirty() || app.state().history().value() != &self.draft
    }

    fn tick(&mut self) {
        if self.last_autosave.elapsed() < AUTOSAVE {
            return;
        }
        self.last_autosave = Instant::now();
        self.autosave();
    }

    fn panels(&self) -> &[starter_desktop::PanelSpec] {
        static PANELS: [starter_desktop::PanelSpec; 1] = [starter_desktop::PanelSpec {
            instance: 110,
            kind: "inspection.recovery",
            title: "Recovery",
            floating: true,
            initially_open: false,
        }];
        &PANELS
    }

    fn request_save(&mut self) -> bool {
        {
            let app = self.app.lock().expect("application mutex");
            if app.state().history().value() != &self.draft {
                self.status = ("Apply the pending draft before saving".into(), true);
                return false;
            }
        }
        match self.save() {
            Ok(()) => true,
            Err(error) => {
                self.status = (error.to_string(), true);
                false
            }
        }
    }
}

pub fn run() -> starter_reference::Result<()> {
    starter_reference::launch("Inspection Workbench", starter_inspection::application(), |_, app| {
        Inspector::new(app)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn inspector() -> Inspector {
        Inspector::new(Arc::new(Mutex::new(starter_inspection::application())))
    }

    #[test]
    fn autosave_and_recovery_use_the_shared_document_session() {
        let directory = tempfile::tempdir().unwrap();
        let mut editor = inspector();
        editor.recovery = Recovery::new(directory.path(), "inspection-test");
        editor.path = directory
            .path()
            .join("inspection.json")
            .to_string_lossy()
            .into_owned();
        editor.draft.site = "Bridge 7".into();
        assert!(editor.apply());
        editor.save().unwrap();
        assert!(!editor.dirty(), "saved documents are clean");
        editor.draft.site = "Bridge 8".into();
        assert!(editor.apply());
        assert!(editor.dirty());
        assert!(editor.autosave());
        assert!(editor.recovery.exists());
        editor.open().unwrap();
        assert_eq!(editor.draft.site, "Bridge 7");
        assert!(editor.recover().unwrap());
        assert_eq!(editor.draft.site, "Bridge 8");
        assert!(editor.dirty(), "recovered drafts stay dirty");
        editor.discard_recovery().unwrap();
        assert!(!editor.recovery.exists());
    }
}
