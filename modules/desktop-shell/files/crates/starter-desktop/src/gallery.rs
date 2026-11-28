use crate::{
    Design, DialogChoice, Icon, Tokens, Tone, button_with_icon, confirmation, dismissible_notice,
    empty_state, field, loading, notice, progress, property_grid, search_field, unit_field,
};
use egui::Ui;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Page {
    #[default]
    Controls,
    States,
    Properties,
}

#[derive(Default)]
pub struct Gallery {
    page: Page,
    preview: Option<Tokens>,
    grid: bool,
    level: u32,
    text: String,
    query: String,
    confirm: bool,
    notification: bool,
}

impl Gallery {
    pub fn show(&mut self, ui: &mut Ui, tokens: Tokens) {
        self.show_with_design(ui, tokens, Design::default());
    }

    pub(crate) fn show_with_design(&mut self, ui: &mut Ui, tokens: Tokens, design: Design) {
        ui.heading(tokens.label("Components", "Componentes"));
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(
                &mut self.page,
                Page::Controls,
                tokens.label("Controls", "Controles"),
            );
            ui.selectable_value(
                &mut self.page,
                Page::States,
                tokens.label("States", "Estados"),
            );
            ui.selectable_value(
                &mut self.page,
                Page::Properties,
                tokens.label("Properties", "Propiedades"),
            );
        });
        ui.collapsing(
            tokens.label("Preview appearance", "Previsualizar apariencia"),
            |ui| {
                let mut active = self.preview.is_some();
                if ui
                    .checkbox(
                        &mut active,
                        tokens.label("Override for preview", "Solo en vista previa"),
                    )
                    .changed()
                {
                    self.preview = active.then_some(tokens);
                }
                if let Some(preview) = &mut self.preview {
                    ui.checkbox(
                        &mut preview.dark,
                        tokens.label("Preview dark theme", "Previsualizar tema oscuro"),
                    );
                    ui.checkbox(
                        &mut preview.compact,
                        tokens.label("Preview compact density", "Previsualizar densidad compacta"),
                    );
                    preview.spanish = tokens.spanish;
                }
            },
        );
        let appearance = self.preview.unwrap_or(tokens);
        ui.scope(|ui| {
            design.preview(ui, appearance);
            egui::Frame::default()
                .fill(ui.visuals().panel_fill)
                .inner_margin(egui::Margin::same(appearance.metrics().gap))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, true])
                        .show(ui, |ui| match self.page {
                            Page::Controls => self.controls(ui, tokens),
                            Page::States => self.states(ui, tokens),
                            Page::Properties => self.properties(ui, tokens),
                        });
                });
        });
        if self.confirm {
            let choice = confirmation(
                ui.ctx(),
                ui.make_persistent_id("reset-gallery"),
                tokens.label("Reset examples?", "¿Restablecer ejemplos?"),
                tokens.label(
                    "This changes only the gallery.",
                    "Esto solo cambia la galería.",
                ),
                (
                    tokens.label("Cancel", "Cancelar"),
                    tokens.label("Reset", "Restablecer"),
                    None,
                ),
            );
            match choice {
                Some(DialogChoice::Primary) => {
                    self.grid = false;
                    self.level = 0;
                    self.text.clear();
                    self.query.clear();
                    self.notification = true;
                    self.confirm = false;
                }
                Some(DialogChoice::Cancel | DialogChoice::Secondary) => self.confirm = false,
                None => {}
            }
        }
    }

    fn controls(&mut self, ui: &mut Ui, tokens: Tokens) {
        ui.checkbox(
            &mut self.grid,
            tokens.label("Show grid", "Mostrar cuadrícula"),
        );
        if self.grid {
            property_grid(ui, "gallery-summary", |ui| {
                ui.label(tokens.label("Preview level", "Nivel de vista previa"));
                ui.monospace(self.level.to_string());
                ui.end_row();
            });
        }
        field(ui, tokens.label("Label", "Etiqueta"), &mut self.text);
        ui.add(egui::Slider::new(&mut self.level, 0..=100).text(tokens.label("Level", "Nivel")));
        ui.horizontal_wrapped(|ui| {
            if button_with_icon(
                ui,
                Icon::Reset,
                tokens.label("Reset examples", "Restablecer ejemplos"),
            )
            .clicked()
            {
                self.confirm = true;
            }
            ui.add_enabled(
                false,
                egui::Button::new(tokens.label("Disabled", "Deshabilitado")),
            );
        });
        ui.collapsing(
            tokens.label("Native egui controls", "Controles nativos de egui"),
            |ui| {
                egui::ComboBox::from_label(tokens.label("Density", "Densidad"))
                    .selected_text(self.level.to_string())
                    .show_ui(ui, |ui| {
                        ui.radio_value(&mut self.level, 25, tokens.label("Low", "Baja"));
                        ui.radio_value(&mut self.level, 75, tokens.label("High", "Alta"));
                    });
            },
        );
        if self.notification
            && dismissible_notice(
                ui,
                tokens.label("Examples reset", "Ejemplos restablecidos"),
                Tone::Normal,
                tokens.label("Dismiss", "Cerrar aviso"),
            )
        {
            self.notification = false;
        }
    }

    fn states(&self, ui: &mut Ui, tokens: Tokens) {
        loading(ui, tokens.label("Loading preview", "Cargando vista previa"));
        progress(ui, self.level as f32 / 100.0);
        empty_state(
            ui,
            tokens.label("No items", "Sin elementos"),
            tokens.label("Add an item to begin.", "Añade un elemento para empezar."),
        );
        notice(
            ui,
            tokens.label("Review this value", "Revisa este valor"),
            Tone::Warning,
        );
        notice(
            ui,
            tokens.label("Invalid value", "Valor inválido"),
            Tone::Error,
        );
    }

    fn properties(&mut self, ui: &mut Ui, tokens: Tokens) {
        search_field(
            ui,
            tokens.label("Search properties", "Buscar propiedades"),
            tokens.label("Type a property name", "Escribe el nombre"),
            &mut self.query,
        );
        let length = tokens.label("Length", "Longitud");
        if !length.to_lowercase().contains(&self.query.to_lowercase()) {
            empty_state(
                ui,
                tokens.label("No matches", "Sin coincidencias"),
                tokens.label(
                    "Clear the filter to show all properties.",
                    "Limpia el filtro para mostrar todo.",
                ),
            );
            return;
        }
        property_grid(ui, "gallery-properties", |ui| {
            let caption = ui.label(length);
            let mut sample = self.level as f64 / 10.0;
            let input = unit_field(ui, "", " m", &mut sample, 0.0..=10.0).labelled_by(caption.id);
            if input.changed() {
                self.level = (sample * 10.0).round() as u32;
            }
            ui.end_row();
        });
    }
}
