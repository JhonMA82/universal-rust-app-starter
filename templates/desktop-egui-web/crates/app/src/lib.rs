//! Shared UI: no platform-specific file or networking operations here.
use eframe::egui;

#[derive(Default)]
pub struct TemplateApp {
    clicks: u64,
}

impl TemplateApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    pub fn clicks(&self) -> u64 {
        self.clicks
    }

    fn increment(&mut self) {
        self.clicks = self.clicks.saturating_add(1);
    }
}

impl eframe::App for TemplateApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top").show(ui, |ui| {
            ui.heading("{{project-name}}");
        });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Hello from Rust + egui");
            ui.label(format!("Count: {}", self.clicks));
            if ui.button("Increment").clicked() {
                self.increment();
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_increments() {
        let mut app = TemplateApp::default();
        app.increment();
        assert_eq!(app.clicks(), 1);
    }
}
