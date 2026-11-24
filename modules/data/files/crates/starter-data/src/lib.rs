#![forbid(unsafe_code)]
pub use starter_dataset::Data;
pub fn table(ui: &mut egui::Ui, data: &Data, indices: &[usize], active: &mut Option<usize>) {
    use egui_extras::{Column, TableBuilder};
    TableBuilder::new(ui)
        .striped(true)
        .max_scroll_height(245.0)
        .columns(
            Column::remainder().at_least(65.0).clip(true),
            data.headers.len(),
        )
        .header(24.0, |mut row| {
            for title in &data.headers {
                row.col(|ui| {
                    ui.strong(title);
                });
            }
        })
        .body(|body| {
            body.rows(22.0, indices.len(), |mut row| {
                let index = indices[row.index()];
                row.set_selected(*active == Some(index));
                for cell in &data.rows[index] {
                    row.col(|ui| {
                        if ui.selectable_label(*active == Some(index), cell).clicked() {
                            *active = Some(index);
                        }
                    });
                }
            })
        });
}
pub fn plot(ui: &mut egui::Ui, data: &Data, indices: &[usize], column: usize) {
    if let Some(title) = data.headers.get(column) {
        let points: Vec<_> = data
            .values(indices, column)
            .take(4000)
            .map(|(i, v)| [i as f64, v])
            .collect();
        egui_plot::Plot::new("starter-data")
            .height(180.)
            .show(ui, |plot| plot.line(egui_plot::Line::new(title, points)));
    }
}
