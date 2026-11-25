#![forbid(unsafe_code)]
use eframe::egui;
use serde_json::{Value, json};
use starter_demo_domain::{Derived, Dimensions, Domain, application};
use starter_desktop::Panel;
use starter_reference::{Shared, View, invoke};
use starter_tasks::{Job, Pool};
use starter_viewport_core::{Mesh, Scene, Vec3};
struct Engineering {
    app: Shared<Domain>,
    canvas: starter_canvas::Camera,
    viewport: Option<starter_viewport_egui::Viewport>,
    draft: Dimensions,
    draft_revision: u64,
    derived: Derived,
    metrics: (usize, f64, bool),
    selected: bool,
    pool: Pool,
    job: Option<Job<f64, String>>,
    background_result: Option<(u64, f64)>,
    status: (String, bool),
}
impl Engineering {
    fn update(&mut self, operation: &str, value: Value) {
        self.status = match invoke(&self.app, operation, value, None, Some(self.draft_revision)) {
            Ok(response) => {
                self.draft = serde_json::from_value(response.result).expect("typed resize result");
                self.draft_revision = response.revision;
                ("Updated geometry".into(), false)
            }
            Err(e) => (e.message, true),
        }
    }
}
impl View for Engineering {
    fn status(&self) -> (String, bool) {
        self.status.clone()
    }
    fn tick(&mut self) {
        let (dim, revision) = {
            let app = self.app.lock().expect("application mutex");
            (*app.state().value(), app.revision())
        };
        if let Some(result) = self.job.as_mut().and_then(|job| job.poll(revision)) {
            self.job = None;
            match result {
                Ok(volume) => self.background_result = Some((revision, volume)),
                Err(error) => self.status = (format!("Background calculation: {error:?}"), false),
            }
        }
        let next = self.derived.update(dim);
        if next.0 > 0 {
            self.metrics = next;
        }
    }
    fn show(&mut self, panel: Panel, ui: &mut egui::Ui) {
        if panel == Panel::Inspector {
            ui.label(format!("Volume: {:.6} m³", self.metrics.1));
            ui.label(format!("{} recomputed nodes", self.metrics.0));
            starter_desktop::status(
                ui,
                if self.metrics.2 {
                    "Volume verified"
                } else {
                    "Volume discrepancy"
                },
                !self.metrics.2,
            );
            if self.selected {
                ui.label("Cuboid selected");
            }
            return;
        }
        let (dim, revision) = {
            let app = self.app.lock().expect("application mutex");
            (*app.state().value(), app.revision())
        };
        ui.horizontal_wrapped(|ui| {
            if let Some(job) = &self.job {
                starter_desktop::progress(ui, f32::from(job.progress()) / 10_000.0);
                if ui.button("Cancel calculation").clicked() {
                    job.cancel();
                }
            } else if ui.button("Calculate volume in background").clicked() {
                match self
                    .pool
                    .start(revision, move |control| -> Result<f64, String> {
                        control.report(10_000);
                        Ok(dim.volume())
                    }) {
                    Ok(job) => self.job = Some(job),
                    Err(error) => self.status = (error.to_string(), true),
                }
            }
            if let Some((source, volume)) = self
                .background_result
                .filter(|(source, _)| *source == revision)
            {
                ui.label(format!(
                    "Background result at revision {source}: {volume:.6} m³"
                ));
            }
        });
        ui.horizontal_wrapped(|ui| {
            for (name, dimension) in [
                ("Width ", &mut self.draft.width_m),
                ("Height ", &mut self.draft.height_m),
                ("Depth ", &mut self.draft.depth_m),
            ] {
                starter_desktop::unit_field(ui, name, " m", dimension, 0.01..=10.0);
            }
        });
        ui.horizontal(|ui| {
            if revision != self.draft_revision {
                if ui.button("Reload current dimensions").clicked() {
                    self.draft = dim;
                    self.draft_revision = revision;
                }
            } else if ui.button("Apply dimensions").clicked() {
                self.update("box.resize", json!(self.draft));
            }
            if ui.button("Undo").clicked() {
                self.update("box.undo", json!({}));
            }
            if ui.button("Redo").clicked() {
                self.update("box.redo", json!({}));
            }
        });
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label("Plan view");
            let r = self.canvas.interact(ui, 150.0);
            self.canvas.grid(ui, r.rect);
            let (w, d) = (dim.width_m as f32 / 2., dim.depth_m as f32 / 2.);
            let rect = egui::Rect::from_two_pos(
                self.canvas.screen(r.rect, egui::pos2(-w, -d)),
                self.canvas.screen(r.rect, egui::pos2(w, d)),
            );
            if let Some(pos) = r.interact_pointer_pos().filter(|_| r.clicked()) {
                self.selected = rect.contains(pos);
            }
            ui.painter_at(r.rect).rect_filled(
                rect,
                2.0,
                if self.selected {
                    egui::Color32::GOLD
                } else {
                    ui.visuals().selection.bg_fill
                },
            );
            ui.label("3D view · drag to orbit");
            if let Some(view) = &mut self.viewport {
                view.selected = self.selected.then_some(1);
                view.show(
                    ui,
                    &Scene(vec![Mesh::cuboid(
                        1,
                        Vec3::new(w, dim.height_m as f32 / 2., d),
                    )]),
                );
                self.selected = view.selected.is_some();
            } else {
                ui.label("No wgpu renderer available");
            }
        });
    }
}
pub fn run() -> starter_reference::Result<()> {
    starter_reference::launch("Engineering Workbench", application(), |cc, app| {
        Engineering {
            app,
            canvas: starter_canvas::Camera::default(),
            viewport: cc
                .wgpu_render_state
                .clone()
                .map(starter_viewport_egui::Viewport::new),
            draft: Dimensions::default(),
            draft_revision: 0,
            derived: Derived::default(),
            metrics: (0, 4., true),
            selected: false,
            pool: Pool::new(1),
            job: None,
            background_result: None,
            status: ("Ready".into(), false),
        }
    })
}
