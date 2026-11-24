#![forbid(unsafe_code)]
use eframe::egui;
use serde_json::json;
use starter_demo_domain::{Domain, MAX_BYTES, Summary, application, metrics};
use starter_desktop::Panel;
use starter_reference::{Shared, View, invoke};
use starter_tasks::{Job, Pool};
use std::io::{Read, Write};

enum Work {
    Imported(String),
    Exported,
    Summary(Summary),
}
struct Explorer {
    app: Shared<Domain>,
    input: String,
    query: String,
    column: usize,
    selected: Option<usize>,
    pool: Pool,
    job: Option<Job<Work, String>>,
    status: (String, bool),
}
impl View for Explorer {
    fn status(&self) -> (String, bool) {
        self.status.clone()
    }
    fn tick(&mut self) {
        let revision = self.app.lock().expect("application mutex").revision();
        if let Some(result) = self.job.as_mut().and_then(|job| job.poll(revision)) {
            self.job = None;
            self.status = match result {
                Ok(Work::Imported(csv)) => match invoke(
                    &self.app,
                    "data.import",
                    json!({"csv":csv}),
                    None,
                    Some(revision),
                ) {
                    Ok(answer) => {
                        self.selected = None;
                        (format!("Imported {} rows", answer.result), false)
                    }
                    Err(e) => (e.message, true),
                },
                Ok(Work::Exported) => ("Exported".into(), false),
                Ok(Work::Summary(s)) => (
                    format!("{} numeric rows · mean {:?}", s.numeric, s.mean),
                    false,
                ),
                Err(e) => (format!("{e:?}"), true),
            };
        }
    }
    fn show(&mut self, panel: Panel, ui: &mut egui::Ui) {
        let (state, revision) = {
            let app = self.app.lock().expect("application mutex");
            (app.state().clone(), app.revision())
        };
        if panel == Panel::Inspector {
            ui.label(format!(
                "{} rows · {} columns",
                state.table.rows.len(),
                state.table.headers.len()
            ));
            ui.label(format!("Active filter: {}", state.view.filter));
            if let Some(i) = self.selected {
                ui.label(format!("Selected row: {}", i + 1));
            }
            return;
        }
        self.column = self.column.min(state.table.headers.len() - 1);
        ui.horizontal(|ui| {
            ui.label("CSV path");
            ui.text_edit_singleline(&mut self.input);
        });
        ui.horizontal_wrapped(|ui| {
            if let Some(job) = &self.job {
                ui.add(
                    egui::ProgressBar::new(f32::from(job.progress()) / 10_000.0).show_percentage(),
                );
                if ui.button("Cancel task").clicked() {
                    job.cancel();
                }
                return;
            }
            let path = self.input.clone();
            let start = if ui.button("Import").clicked() {
                Some(self.pool.start(revision, move |cx| {
                    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
                    let mut data = Vec::new();
                    let mut buffer = [0; 16_384];
                    loop {
                        if cx.cancelled() {
                            return Err("cancelled".to_owned());
                        }
                        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
                        if count == 0 {
                            break;
                        }
                        if data.len() + count > MAX_BYTES {
                            return Err("CSV byte limit".into());
                        }
                        data.extend_from_slice(&buffer[..count]);
                        cx.report((data.len() * 9000 / MAX_BYTES) as u16);
                    }
                    cx.report(10_000);
                    Ok(Work::Imported(
                        String::from_utf8(data).map_err(|e| e.to_string())?,
                    ))
                }))
            } else if ui.button("Calculate mean").clicked() {
                let data = state.table.clone();
                let view = state.view.clone();
                Some(self.pool.start(revision, move |cx| {
                    if cx.cancelled() {
                        return Err("cancelled".into());
                    }
                    let result = metrics(&data, &view);
                    cx.report(10_000);
                    Ok(Work::Summary(result))
                }))
            } else if ui.button("Export").clicked() {
                let data = state.table.clone();
                let view = state.view.clone();
                let selected = self.selected;
                Some(self.pool.start(revision, move |cx| {
                    let rows = selected
                        .as_ref()
                        .map_or(view.indices.as_slice(), std::slice::from_ref);
                    let mut csv = Vec::new();
                    data.export(&mut csv, rows)?;
                    if cx.cancelled() {
                        return Err("cancelled".into());
                    }
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(path)
                        .map_err(|e| e.to_string())?;
                    file.write_all(&csv).map_err(|e| e.to_string())?;
                    cx.report(10_000);
                    Ok(Work::Exported)
                }))
            } else {
                None
            };
            if let Some(start) = start {
                match start {
                    Ok(job) => self.job = Some(job),
                    Err(e) => self.status = (e.to_string(), true),
                }
            }
        });
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.query);
            egui::ComboBox::from_label("Plot column")
                .selected_text(&state.table.headers[self.column])
                .show_ui(ui, |ui| {
                    for (column, name) in state.table.headers.iter().enumerate() {
                        ui.selectable_value(&mut self.column, column, name);
                    }
                });
            if ui.button("Apply filter").clicked() {
                let args = json!({"query":self.query,"column":self.column,"sort":self.column});
                self.status = match invoke(&self.app, "data.filter", args, None, Some(revision)) {
                    Ok(result) => (format!("{} visible rows", result.result), false),
                    Err(e) => (e.message, true),
                };
            }
        });
        starter_data::plot(ui, &state.table, &state.view.indices, state.view.column);
        starter_data::table(ui, &state.table, &state.view.indices, &mut self.selected);
    }
}
pub fn run() -> starter_reference::Result<()> {
    starter_reference::launch("Data Explorer", application(), |_, app| Explorer {
        app,
        input: "data.csv".into(),
        query: String::new(),
        column: 1,
        selected: None,
        pool: Pool::new(1),
        job: None,
        status: ("Sample data loaded".into(), false),
    })
}
