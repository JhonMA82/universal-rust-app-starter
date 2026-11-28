#![forbid(unsafe_code)]
pub mod domain;

use domain::{Lab, Tank, application, evaluate};
use eframe::egui;
use serde_json::{Value, json};
use starter_desktop::{ActionSpec, Panel, PanelSpec};
use starter_reference::{Shared, View, invoke};
use starter_tasks::TaskSlot;
use starter_viewport_core::{Mesh, Scene, Vec3};
use std::time::Duration;

const DATA: u64 = 10;
const GEOMETRY: u64 = 20;
const PANELS: [PanelSpec; 2] = [
    PanelSpec {
        instance: DATA,
        kind: "tank.results",
        title: "Results",
        floating: true,
        initially_open: true,
    },
    PanelSpec {
        instance: GEOMETRY,
        kind: "tank.geometry",
        title: "Geometry",
        floating: true,
        initially_open: true,
    },
];
const ACTIONS: [ActionSpec; 1] = [ActionSpec {
    command: "tank.undo",
    label: "Undo tank edit",
    menu: Some("Edit"),
    toolbar: true,
    shortcut: Some(egui::KeyboardShortcut::new(
        egui::Modifiers::COMMAND,
        egui::Key::Z,
    )),
}];

struct Workspace {
    app: Shared<Lab>,
    path: String,
    notice: (String, bool),
    tasks: TaskSlot<f64, String>,
    canvas: starter_canvas::Camera,
    viewport: Option<starter_viewport_egui::Viewport>,
    selected: bool,
    selected_row: Option<usize>,
    filter: String,
}

impl Workspace {
    fn new(app: Shared<Lab>, viewport: Option<starter_viewport_egui::Viewport>) -> Self {
        Self {
            app,
            path: "tank.json".into(),
            notice: ("Ready".into(), false),
            tasks: TaskSlot::default(),
            canvas: starter_canvas::Camera::default(),
            viewport,
            selected: false,
            selected_row: None,
            filter: String::new(),
        }
    }

    fn apply(&mut self, command: &str, arguments: Value, revision: u64) -> bool {
        match invoke(&self.app, command, arguments, None, Some(revision)) {
            Ok(_) => {
                self.notice = ("Operation completed".into(), false);
                true
            }
            Err(error) => {
                self.notice = (error.message, true);
                false
            }
        }
    }

    fn snapshot(&self) -> (Tank, u64, Option<f64>) {
        let app = self.app.lock().expect("application mutex");
        (
            *app.state().document.history().value(),
            app.revision(),
            app.state().result,
        )
    }

    fn geometry(&mut self, ui: &mut egui::Ui, tank: Tank) {
        let area = self.canvas.interact(ui, 160.0);
        self.canvas.grid(ui, area.rect);
        let (width, depth) = (tank.width_m as f32 / 2.0, tank.depth_m as f32 / 2.0);
        let plan = egui::Rect::from_two_pos(
            self.canvas.screen(area.rect, egui::pos2(-width, -depth)),
            self.canvas.screen(area.rect, egui::pos2(width, depth)),
        );
        if let Some(pointer) = area.interact_pointer_pos().filter(|_| area.clicked()) {
            self.selected = plan.contains(pointer);
        }
        let color = if self.selected {
            ui.visuals().selection.bg_fill
        } else {
            ui.visuals().widgets.inactive.bg_fill
        };
        ui.painter_at(area.rect).rect_filled(plan, 2.0, color);

        if let Some(viewport) = &mut self.viewport {
            viewport.selected = self.selected.then_some(1);
            viewport.show(
                ui,
                &Scene(vec![Mesh::cuboid(
                    1,
                    Vec3::new(width, tank.height_m as f32 / 2.0, depth),
                )]),
            );
            self.selected = viewport.selected.is_some();
        } else {
            ui.label("3D view requires wgpu");
        }
    }

    fn results(&mut self, ui: &mut egui::Ui, tank: Tank, result: Option<f64>) {
        let label = ui.label("Filter by fill or litres");
        ui.text_edit_singleline(&mut self.filter)
            .labelled_by(label.id);
        let data = starter_data::Data {
            headers: vec!["Fill (%)".into(), "Litres".into()],
            rows: (0..=10)
                .map(|step| {
                    let fill = f64::from(step) / 10.0;
                    vec![
                        (step * 10).to_string(),
                        format!("{:.3}", Tank { fill, ..tank }.litres()),
                    ]
                })
                .collect(),
        };
        let rows = data.filter(&self.filter, None);
        starter_data::table(ui, &data, &rows, &mut self.selected_row);
        starter_data::plot(ui, &data, &rows, 1);
        if let Some(litres) = result {
            ui.label(format!("Verified background result: {litres:.3} L"));
        }
    }
}

impl View for Workspace {
    fn panels(&self) -> &[PanelSpec] {
        &PANELS
    }
    fn actions(&self) -> &'static [ActionSpec] {
        &ACTIONS
    }
    fn status(&self) -> (String, bool) {
        self.notice.clone()
    }
    fn dirty(&self) -> bool {
        self.app
            .lock()
            .expect("application mutex")
            .state()
            .document
            .history()
            .dirty()
            || self.tasks.progress().is_some()
    }
    fn request_save(&mut self) -> bool {
        if self.tasks.progress().is_some() {
            self.notice = (
                "Wait for the calculation or cancel it before closing".into(),
                true,
            );
            return false;
        }
        let (_, revision, _) = self.snapshot();
        self.apply("tank.save", json!({"path": self.path}), revision)
    }
    fn tick(&mut self) {
        let (tank, revision, _) = self.snapshot();
        match self.tasks.poll(revision) {
            Some(Ok(litres)) if (litres - tank.litres()).abs() <= 1e-8 => {
                self.apply("tank.publish", json!({"litres": litres}), revision);
            }
            Some(Ok(_)) => {
                self.notice = (
                    "Calculated result failed the volume invariants".into(),
                    true,
                )
            }
            Some(Err(error)) => self.notice = (format!("Calculation: {error:?}"), true),
            None => {}
        }
    }

    fn show(&mut self, panel: Panel, ui: &mut egui::Ui) {
        let (mut tank, revision, result) = self.snapshot();
        match panel {
            Panel::Custom(DATA) => self.results(ui, tank, result),
            Panel::Custom(GEOMETRY) => self.geometry(ui, tank),
            Panel::Inspector => {
                ui.label(format!("Revision {revision}"));
                ui.label(format!("Liquid volume: {:.3} L", tank.litres()));
            }
            Panel::Work => {
                ui.heading("Tank project");
                let mut changed = false;
                for (label, value) in [
                    ("Width ", &mut tank.width_m),
                    ("Height ", &mut tank.height_m),
                    ("Depth ", &mut tank.depth_m),
                ] {
                    changed |=
                        starter_desktop::unit_field(ui, label, " m", value, 0.01..=10.0).changed();
                }
                changed |= ui
                    .add(egui::Slider::new(&mut tank.fill, 0.0..=1.0).text("Fill fraction"))
                    .changed();
                if changed {
                    self.apply("tank.resize", json!(tank), revision);
                }
                let label = ui.label("Tank document path");
                ui.text_edit_singleline(&mut self.path)
                    .labelled_by(label.id);
                ui.horizontal(|ui| {
                    if ui.button("Save project").clicked() {
                        self.request_save();
                    }
                    if ui.button("Open project").clicked() {
                        self.apply("tank.open", json!({"path": self.path}), revision);
                    }
                });
                if let Some(progress) = self.tasks.progress() {
                    starter_desktop::progress(ui, f32::from(progress) / 10_000.0);
                    if ui.button("Cancel calculation").clicked() {
                        self.tasks.cancel();
                    }
                    ui.ctx().request_repaint_after(Duration::from_millis(40));
                } else if ui.button("Evaluate tank").clicked() {
                    if let Err(error) = self.tasks.start(revision, move |control| {
                        control.report(2000);
                        let result = evaluate(tank, || control.cancelled());
                        control.report(10_000);
                        result
                    }) {
                        self.notice = (error.to_string(), true);
                    }
                    ui.ctx().request_repaint_after(Duration::from_millis(40));
                }
            }
            _ => {}
        }
    }
}

pub fn run(title: &'static str) -> starter_reference::Result<()> {
    starter_reference::launch(title, application(), |cc, app| {
        let viewport = cc
            .wgpu_render_state
            .clone()
            .map(starter_viewport_egui::Viewport::new);
        Workspace::new(app, viewport)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::{Harness, kittest::Queryable};
    use std::{
        sync::{Arc, Mutex},
        time::Instant,
    };

    #[test]
    fn evaluation_control_publishes_shared_result_and_table() {
        let app = Arc::new(Mutex::new(application()));
        let mut workspace = Workspace::new(app.clone(), None);
        let mut harness = Harness::new_ui(|ui| {
            workspace.tick();
            workspace.show(Panel::Work, ui);
            ui.separator();
            workspace.show(Panel::Custom(DATA), ui);
        });
        harness.get_by_label("Evaluate tank").click();
        let deadline = Instant::now() + Duration::from_secs(3);
        while app.lock().unwrap().state().result.is_none() {
            assert!(
                Instant::now() < deadline,
                "result did not reach the shared engine"
            );
            harness.run();
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(app.lock().unwrap().state().result, Some(1000.0));
        harness.get_by_label("Verified background result: 1000.000 L");
        harness.get_by_label("Litres");
    }
}
