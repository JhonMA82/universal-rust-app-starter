//! Demonstration app for egui 0.37: stable state, measured panels, and genuine widgets.
//! This starter is deliberately NOT a visual reconstruction of any supplied screenshot.

use eframe::egui;

const SIDEBAR_WIDTH: f32 = 224.0;
const CANVAS_HEIGHT: f32 = 430.0;

struct StarterApp {
    show_grid: bool,
    active_view: usize,
    filter: String,
}

impl StarterApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Self {
            show_grid: true,
            active_view: 0,
            filter: String::new(),
        }
    }
}

impl eframe::App for StarterApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("VectorDesk · Example");
                ui.separator();
                ui.label("Reference size: 1100 × 700");
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(SIDEBAR_WIDTH);
                    ui.heading("Explorer");
                    ui.selectable_value(&mut self.active_view, 0, "Overview");
                    ui.selectable_value(&mut self.active_view, 1, "Components");
                    ui.separator();
                    ui.checkbox(&mut self.show_grid, "Show grid");
                    ui.horizontal(|ui| {
                        let label = ui.label("Filter");
                        ui.add(egui::TextEdit::singleline(&mut self.filter).desired_width(130.0))
                            .labelled_by(label.id);
                    });
                    ui.separator();
                    ui.label(format!("Active view: {}", self.active_view));
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.heading(if self.active_view == 0 {
                        "Workspace"
                    } else {
                        "Component gallery"
                    });
                    let canvas_width = ui.available_width().max(180.0);
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(canvas_width, CANVAS_HEIGHT),
                        egui::Sense::hover(),
                    );
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 6.0, egui::Color32::from_rgb(30, 39, 53));
                    if self.show_grid {
                        let line_color = egui::Color32::from_rgb(53, 68, 84);
                        let step = 32.0;
                        let mut x = rect.left() + step;
                        while x < rect.right() {
                            painter.line_segment(
                                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                                egui::Stroke::new(1.0, line_color),
                            );
                            x += step;
                        }
                        let mut y = rect.top() + step;
                        while y < rect.bottom() {
                            painter.line_segment(
                                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                                egui::Stroke::new(1.0, line_color),
                            );
                            y += step;
                        }
                    }
                    painter.circle_filled(rect.center(), 35.0, egui::Color32::from_rgb(50, 181, 173));
                    ui.small("The layout and colors are examples, not an accuracy claim.");
                });
            });
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Ready");
                ui.separator();
                ui.small("Use this app to practice fixed-state captures and visual iteration");
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1100.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui visual forge starter",
        native_options,
        Box::new(|cc| Ok(Box::new(StarterApp::new(cc)))),
    )
}
