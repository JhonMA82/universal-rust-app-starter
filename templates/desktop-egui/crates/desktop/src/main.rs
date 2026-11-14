fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("{{project-name}}")
            .with_inner_size([1100.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "{{project-name}}",
        opts,
        Box::new(|cc| Ok(Box::new(ui_app::TemplateApp::new(cc)))),
    )
}
