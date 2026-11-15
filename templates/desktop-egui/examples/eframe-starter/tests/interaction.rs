//! Uses the public egui_kittest 0.37 Harness/AccessKit interaction pattern.
use egui::accesskit::Toggled;
use egui_kittest::{kittest::{NodeT, Queryable}, Harness};

#[test]
fn checkbox_toggles_on_click() {
    let mut show_grid = false;
    let app = |ui: &mut egui::Ui| {
        ui.checkbox(&mut show_grid, "Show grid");
    };
    let mut harness = Harness::new_ui(app);
    let checkbox = harness.get_by_label("Show grid");
    assert_eq!(checkbox.accesskit_node().toggled(), Some(Toggled::False));
    checkbox.click();
    harness.run();
    let checkbox = harness.get_by_label("Show grid");
    assert_eq!(checkbox.accesskit_node().toggled(), Some(Toggled::True));
}
