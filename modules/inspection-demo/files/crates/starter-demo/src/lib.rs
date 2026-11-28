#![forbid(unsafe_code)]
use eframe::egui;
use serde_json::json;
use starter_desktop::Panel;
use starter_documents::storage::{Document, Recovery, RecoveryDraft, SaveReport};
use starter_inspection::{InspectionCheck, InspectionReport, format, recovery};
use starter_reference::{Shared, View, invoke};
use starter_tasks::TaskSlot;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

const AUTOSAVE: Duration = Duration::from_secs(15);

struct Inspector {
    app: Shared<Document<InspectionReport>>,
    draft: InspectionReport,
    baseline: InspectionReport,
    draft_revision: u64,
    application_revision: u64,
    path: String,
    recovery_directory: PathBuf,
    untitled_key: String,
    last_autosave: Instant,
    autosave: TaskSlot<Option<SaveReport>, String>,
    recovery_drafts: Option<Result<Vec<RecoveryDraft>, starter_documents::Error>>,
    status: (String, bool),
}

impl Inspector {
    fn new(app: Shared<Document<InspectionReport>>) -> Self {
        let (draft, draft_revision, application_revision) = {
            let app = app.lock().expect("application mutex");
            (
                app.state().history().value().clone(),
                app.state().history().revision(),
                app.revision(),
            )
        };
        Self {
            app,
            baseline: draft.clone(),
            draft,
            draft_revision,
            application_revision,
            path: "inspection.json".into(),
            recovery_directory: ".inspection-recovery".into(),
            untitled_key: format!("untitled-{}", uuid::Uuid::new_v4().simple()),
            last_autosave: Instant::now(),
            autosave: TaskSlot::default(),
            recovery_drafts: None,
            status: ("Ready".into(), false),
        }
    }

    fn sync(&mut self) {
        let app = self.app.lock().expect("application mutex");
        self.draft = app.state().history().value().clone();
        self.baseline = self.draft.clone();
        self.draft_revision = app.state().history().revision();
        self.application_revision = app.revision();
    }

    fn execute(&mut self, id: &str, parameters: serde_json::Value) -> Result<(), String> {
        invoke(
            &self.app,
            id,
            parameters,
            None,
            Some(self.application_revision),
        )
        .map_err(|error| error.message)?;
        self.sync();
        Ok(())
    }

    fn operation(&mut self, id: &str, parameters: serde_json::Value) -> bool {
        match self.execute(id, parameters) {
            Ok(()) => true,
            Err(error) => {
                self.status = (error, true);
                false
            }
        }
    }

    fn apply(&mut self) -> bool {
        let parameters = json!({"report": self.draft, "if_revision": self.draft_revision});
        self.operation("inspection.edit", parameters)
    }

    fn history(&mut self, id: &str) -> bool {
        if self.draft != self.baseline {
            self.status = (
                "Apply the pending draft before changing history".into(),
                true,
            );
            return false;
        }
        self.operation(id, json!({}))
    }

    fn save(&mut self) -> Result<(), String> {
        let draft = (self.draft != self.baseline).then_some(&self.draft);
        self.execute(
            "inspection.session.save",
            json!({"path": self.path, "draft": draft}),
        )
    }

    fn open(&mut self) -> Result<(), String> {
        self.require_clean()?;
        self.execute("inspection.session.open", json!({"path": self.path}))
    }

    fn require_clean(&self) -> Result<(), String> {
        if self.dirty() {
            Err("Save or recover the current work before switching documents".into())
        } else {
            Ok(())
        }
    }

    fn start_autosave(&mut self) -> Result<bool, String> {
        if self.draft != self.baseline {
            let parameters = json!({"report": self.draft, "if_revision": self.draft_revision});
            self.execute("inspection.edit", parameters)?;
        }
        let document = self.app.lock().expect("application mutex").state().clone();
        if !document.history().dirty() {
            return Ok(false);
        }
        let directory = self.recovery_directory.clone();
        let key = self.untitled_key.clone();
        // The worker saves an immutable draft. Its result only changes UI feedback.
        self.autosave
            .start(0, move |_| {
                let handle = recovery(directory, document.bound_path().map(PathBuf::from), &key)
                    .map_err(|error| error.message)?;
                document
                    .autosave(&handle)
                    .map_err(|error| error.to_string())
            })
            .map_err(|error| error.to_string())?;
        Ok(true)
    }

    fn recover(&mut self, draft: &RecoveryDraft) -> Result<(), String> {
        self.require_clean()?;
        self.execute(
            "inspection.session.recover",
            json!({"directory": self.recovery_directory, "path": draft.original_path, "key": draft.recovery.key()}),
        )?;
        if let Some(path) = &draft.original_path {
            self.path = path.to_string_lossy().into_owned();
        } else {
            self.untitled_key = draft.recovery.key().into();
        }
        Ok(())
    }
}

impl View for Inspector {
    fn show(&mut self, panel: Panel, ui: &mut egui::Ui) {
        let refresh = if self.autosave.progress().is_some() {
            Duration::from_millis(50)
        } else {
            AUTOSAVE.saturating_sub(self.last_autosave.elapsed())
        };
        ui.ctx().request_repaint_after(refresh);
        if panel == Panel::Custom(110) {
            ui.heading("Recovery");
            let elapsed = ui.button("Refresh recovery list").clicked();
            let mut refresh = elapsed;
            let drafts = self
                .recovery_drafts
                .take()
                .filter(|_| !elapsed)
                .unwrap_or_else(|| Recovery::discover(&self.recovery_directory, &format()));
            match &drafts {
                Ok(drafts) if drafts.is_empty() => {
                    ui.label("No autosaved drafts");
                }
                Ok(drafts) => {
                    for draft in drafts {
                        ui.push_id(draft.recovery.key(), |ui| {
                            ui.label(draft.original_path.as_deref().map_or_else(
                                || "Untitled".into(),
                                |path| path.display().to_string(),
                            ));
                            if ui.button("Recover autosaved draft").clicked() {
                                self.status = match self.recover(draft) {
                                    Ok(()) => ("Recovered autosaved draft".into(), false),
                                    Err(error) => (error, true),
                                };
                            }
                            if ui.button("Discard autosaved draft").clicked() {
                                refresh = true;
                                self.status = match draft.discard() {
                                    Ok(()) => ("Discarded autosaved draft".into(), false),
                                    Err(error) => (error.to_string(), true),
                                };
                            }
                        });
                    }
                }
                Err(error) => {
                    starter_desktop::status(ui, &error.to_string(), true);
                }
            }
            if !refresh {
                self.recovery_drafts = Some(drafts);
            } else {
                ui.ctx().request_repaint();
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
            ui.text_edit_singleline(&mut check.note)
                .labelled_by(label.id);
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
            if ui.button("Undo").clicked() && self.history("inspection.undo") {
                self.status = ("Undo complete".into(), false);
            }
            if ui.button("Redo").clicked() && self.history("inspection.redo") {
                self.status = ("Redo complete".into(), false);
            }
        });
        ui.separator();
        let label = ui.label("File path");
        ui.text_edit_singleline(&mut self.path)
            .labelled_by(label.id);
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
        app.state().history().dirty() || self.baseline != self.draft
    }

    fn tick(&mut self) {
        if self.draft == self.baseline
            && self.app.lock().expect("application mutex").revision() != self.application_revision
        {
            self.sync();
        }
        if let Some(result) = self.autosave.poll(0) {
            match result {
                Ok(Some(report)) => {
                    self.recovery_drafts = None;
                    self.status = (format!("Autosaved {} bytes", report.bytes), false);
                }
                Ok(None) => {}
                Err(error) => self.status = (format!("Autosave: {error:?}"), true),
            }
        }
        if self.autosave.progress().is_none() && self.last_autosave.elapsed() >= AUTOSAVE {
            self.last_autosave = Instant::now();
            if let Err(error) = self.start_autosave() {
                self.status = (format!("Autosave: {error}"), true);
            }
        }
    }

    fn panels(&self) -> &[starter_desktop::PanelSpec] {
        static PANELS: [starter_desktop::PanelSpec; 1] = [starter_desktop::PanelSpec {
            instance: 110,
            kind: "inspection.recovery",
            title: "Recovery",
            floating: true,
            initially_open: true,
        }];
        &PANELS
    }

    fn request_save(&mut self) -> bool {
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
    starter_reference::launch(
        "Inspection Workbench",
        starter_inspection::application(),
        |_, app| Inspector::new(app),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn inspector() -> Inspector {
        Inspector::new(Arc::new(Mutex::new(starter_inspection::application())))
    }

    fn autosave(editor: &mut Inspector) {
        assert!(editor.start_autosave().unwrap());
        let expiry = Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(result) = editor.autosave.poll(0) {
                assert!(result.unwrap().is_some());
                return;
            }
            assert!(Instant::now() < expiry, "autosave worker stalled");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn autosave_and_recovery_use_the_shared_document_session() {
        let directory = tempfile::tempdir().unwrap();
        let report = directory.path().join("inspection.json");
        let mut editor = inspector();
        editor.recovery_directory = directory.path().join("recovery");
        editor.path = report.to_string_lossy().into_owned();
        editor.draft.site = "Bridge 7".into();
        editor.save().unwrap();
        assert!(!editor.dirty(), "saving includes pending UI edits");
        editor.draft.site = "Bridge 8".into();
        assert!(editor.dirty());
        autosave(&mut editor);
        editor.draft.site = "Bridge 9".into();
        autosave(&mut editor);
        assert!(
            editor.open().is_err(),
            "opening must not discard the pending edit"
        );
        let recovery_directory = editor.recovery_directory.clone();
        drop(editor);

        let mut editor = inspector();
        editor.recovery_directory = recovery_directory;
        let mut drafts = Recovery::discover(&editor.recovery_directory, &format()).unwrap();
        assert_eq!(drafts.len(), 1);
        let draft = drafts.remove(0);
        assert_eq!(draft.original_path.as_deref(), Some(report.as_path()));
        editor.recover(&draft).unwrap();
        assert_eq!(editor.draft.site, "Bridge 9");
        assert!(editor.dirty(), "recovered drafts stay dirty");
        editor.save().unwrap();
        assert!(!editor.dirty());
        let stored = Document::open(format(), &report, 128).unwrap();
        assert_eq!(stored.history().value().site, "Bridge 9");
        assert!(editor.recover(&draft).is_err(), "saved drafts are stale");
        draft.discard().unwrap();
        assert!(!draft.recovery.exists());
    }

    #[test]
    fn untitled_views_keep_independent_discoverable_drafts() {
        let directory = tempfile::tempdir().unwrap();
        let mut first = inspector();
        let mut second = inspector();
        first.recovery_directory = directory.path().into();
        second.recovery_directory = directory.path().into();
        first.draft.site = "Bridge 11".into();
        second.draft.site = "Bridge 12".into();
        autosave(&mut first);
        autosave(&mut second);
        let drafts = Recovery::discover(directory.path(), &format()).unwrap();
        assert_eq!(drafts.len(), 2);
        assert!(drafts.iter().all(|draft| draft.original_path.is_none()));
        let expected = drafts
            .iter()
            .find(|draft| draft.recovery.key() == &first.untitled_key)
            .unwrap();
        drop(first);
        drop(second);
        let mut reopened = inspector();
        reopened.recovery_directory = directory.path().into();
        reopened.recover(expected).unwrap();
        assert_eq!(reopened.draft.site, "Bridge 11");
        assert_eq!(reopened.untitled_key, expected.recovery.key());
    }
}
