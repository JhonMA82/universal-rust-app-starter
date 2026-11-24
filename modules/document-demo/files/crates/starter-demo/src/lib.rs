#![forbid(unsafe_code)]
use eframe::egui;
use serde_json::{Value, json};
use starter_demo_domain::{Domain, application, format, note};
use starter_desktop::Panel;
use starter_documents::{
    Envelope, Error,
    storage::{self, Checksum},
};
use starter_reference::{Shared, View, invoke};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
struct FileBinding {
    path: PathBuf,
    checksum: Checksum,
    extra: BTreeMap<String, Value>,
}
struct Editor {
    app: Shared<Domain>,
    current: u64,
    drafts: BTreeMap<u64, (u64, String)>,
    files: BTreeMap<u64, FileBinding>,
    path: String,
    status: (String, bool),
    last_save: Instant,
    recovery: String,
}
impl Editor {
    fn select(&mut self, id: u64) {
        self.current = id;
        self.path = self
            .files
            .get(&id)
            .map(|file| file.path.to_string_lossy().into_owned())
            .unwrap_or_else(|| format!("note-{id}.json"));
    }
    fn operation(&mut self, op: &str, params: Value, doc: Option<u64>) -> Option<Value> {
        match invoke(&self.app, op, params, doc, None) {
            Ok(value) => Some(value.result),
            Err(e) => {
                self.status = (e.message, true);
                None
            }
        }
    }
    fn recovery_path(&self, id: u64) -> PathBuf {
        Path::new(".note-recovery").join(format!("{}-{id}.json", self.recovery))
    }
    fn read(&mut self, path: &Path, recovery: bool) -> Result<(), Error> {
        let (content, checksum) = storage::open(path, &format())?;
        let id = self
            .operation(
                "notes.new",
                json!({"text":content.content,"recovered":recovery}),
                None,
            )
            .and_then(|v| v.as_u64())
            .ok_or_else(|| Error::Invalid("could not open a note".into()))?;
        if !recovery {
            self.files.insert(
                id,
                FileBinding {
                    path: path.to_owned(),
                    checksum,
                    extra: content.extra,
                },
            );
        }
        self.select(id);
        Ok(())
    }
    fn save(&mut self, id: u64, path: PathBuf, recovery: bool) -> Result<(), Error> {
        let (value, revision) = {
            let app = self.app.lock().expect("application mutex");
            let h = app
                .state()
                .notes
                .get(&id)
                .ok_or_else(|| Error::Invalid("unknown document".into()))?;
            (h.snapshot(), h.revision())
        };
        let format = format();
        let file = self.files.get(&id);
        let doc = Envelope {
            format: format.id.into(),
            version: format.version,
            revision,
            content: (*value).clone(),
            extra: file.map(|f| f.extra.clone()).unwrap_or_default(),
        };
        let checksum = if recovery {
            match storage::open(&path, &format) {
                Ok((_, sum)) => Some(sum),
                Err(Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => None,
                Err(e) => return Err(e),
            }
        } else {
            file.filter(|f| f.path == path).map(|f| f.checksum)
        };
        let report = storage::save(&path, &format, &doc, checksum)?;
        if !recovery {
            self.files.insert(
                id,
                FileBinding {
                    path,
                    checksum: report.checksum,
                    extra: doc.extra,
                },
            );
            self.app
                .lock()
                .expect("application mutex")
                .acknowledge(|state| {
                    if let Some(h) = state.notes.get_mut(&id) {
                        h.mark_saved(value);
                    }
                });
            match std::fs::remove_file(self.recovery_path(id)) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => eprintln!("Recovery cleanup: {e}"),
            }
        }
        Ok(())
    }
}
impl View for Editor {
    fn status(&self) -> (String, bool) {
        self.status.clone()
    }
    fn dirty(&self) -> bool {
        self.app
            .lock()
            .expect("application mutex")
            .state()
            .notes
            .iter()
            .any(|(id, h)| h.dirty() || self.drafts.get(id).is_some_and(|(_, s)| s != h.value()))
    }
    fn tick(&mut self) {
        if self.last_save.elapsed() < Duration::from_secs(15) {
            return;
        }
        self.last_save = Instant::now();
        let dirty: Vec<_> = self
            .app
            .lock()
            .expect("application mutex")
            .state()
            .notes
            .iter()
            .filter_map(|(&id, h)| h.dirty().then_some(id))
            .collect();
        if dirty.is_empty() {
            return;
        }
        if let Err(e) = std::fs::create_dir_all(".note-recovery") {
            self.status = (e.to_string(), true);
            return;
        }
        for id in dirty {
            if let Err(e) = self.save(id, self.recovery_path(id), true) {
                self.status = (e.to_string(), true);
            }
        }
    }
    fn show(&mut self, panel: Panel, ui: &mut egui::Ui) {
        let (ids, current) = {
            let app = self.app.lock().expect("application mutex");
            let state = app.state();
            (
                state.notes.keys().copied().collect::<Vec<_>>(),
                note(state, self.current).ok(),
            )
        };
        if panel == Panel::Inspector {
            if let Some(note) = current {
                ui.label(format!("Note {} · revision {}", note.id, note.revision));
                ui.label(format!("{} bytes", note.text.len()));
            }
            return;
        }
        ui.horizontal_wrapped(|ui| {
            if ui.button("New note").clicked()
                && let Some(id) = self
                    .operation("notes.new", json!({}), None)
                    .and_then(|v| v.as_u64())
            {
                self.select(id);
            }
            for id in ids {
                if ui
                    .selectable_label(self.current == id, format!("Note {id}"))
                    .clicked()
                {
                    self.select(id);
                }
            }
        });
        if let Some(note) = current.filter(|n| n.id == self.current) {
            let draft = self
                .drafts
                .entry(note.id)
                .or_insert((note.revision, note.text));
            ui.text_edit_multiline(&mut draft.1);
            if draft.0 != note.revision {
                if ui.button("Reload current version").clicked() {
                    self.drafts.remove(&note.id);
                }
            } else if ui.button("Apply edit").clicked() {
                let edit = json!({"text":draft.1,"if_revision":draft.0});
                if self.operation("notes.edit", edit, Some(note.id)).is_some() {
                    self.drafts.remove(&note.id);
                }
            }
            ui.horizontal(|ui| {
                for (title, op) in [("Undo", "notes.undo"), ("Redo", "notes.redo")] {
                    if ui.button(title).clicked()
                        && self.operation(op, json!({}), Some(note.id)).is_some()
                    {
                        self.drafts.remove(&note.id);
                    }
                }
            });
        }
        ui.separator();
        ui.label("File path");
        ui.text_edit_singleline(&mut self.path);
        ui.horizontal(|ui| {
            let path = PathBuf::from(&self.path);
            if ui.button("Open").clicked() {
                self.status = message(self.read(&path, false), "Opened");
            }
            if ui.button("Save committed changes").clicked() {
                self.status = message(self.save(self.current, path, false), "Verified and saved");
            }
        });
        if let Ok(files) = std::fs::read_dir(".note-recovery") {
            for path in files
                .flatten()
                .map(|f| f.path())
                .filter(|p| p.extension().is_some_and(|x| x == "json"))
                .take(64)
            {
                if ui
                    .button(format!(
                        "Recover {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ))
                    .clicked()
                {
                    self.status = message(self.read(&path, true), "Recovered into a new note");
                }
            }
        }
    }
}
fn message(result: Result<(), Error>, success: &str) -> (String, bool) {
    match result {
        Ok(()) => (success.into(), false),
        Err(e) => (e.to_string(), true),
    }
}
pub fn run() -> starter_reference::Result<()> {
    starter_reference::launch("Document Workbench", application(), |_, app| Editor {
        app,
        current: 1,
        drafts: BTreeMap::new(),
        files: BTreeMap::new(),
        path: "note-1.json".into(),
        status: ("Ready".into(), false),
        last_save: Instant::now(),
        recovery: format!(
            "note-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    #[test]
    fn verifies_reopen_and_recovery_through_reference_adapter() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("published.json");
        let recovery = temp.path().join("recovery.json");
        let mut editor = Editor {
            app: Arc::new(Mutex::new(application())),
            current: 1,
            drafts: BTreeMap::new(),
            files: BTreeMap::new(),
            path: String::new(),
            status: (String::new(), false),
            last_save: Instant::now(),
            recovery: "adapter-test".into(),
        };
        assert!(
            editor
                .operation("notes.edit", json!({"text":"published"}), Some(1))
                .is_some()
        );
        editor.save(1, path.clone(), false).unwrap();
        assert!(!editor.dirty());
        assert!(
            editor
                .operation("notes.edit", json!({"text":"after save"}), Some(1))
                .is_some()
        );
        editor.save(1, recovery.clone(), true).unwrap();
        editor.read(&path, false).unwrap();
        let reopened = note(editor.app.lock().unwrap().state(), editor.current).unwrap();
        assert_eq!(reopened.text, "published");
        editor.read(&recovery, true).unwrap();
        let recovered = note(editor.app.lock().unwrap().state(), editor.current).unwrap();
        assert_eq!(recovered.text, "after save");
        assert!(recovered.dirty);
    }
}
