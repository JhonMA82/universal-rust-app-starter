use egui::{Response, Ui};
use std::ops::RangeInclusive;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Normal,
    Warning,
    Error,
}

pub fn notice(ui: &mut Ui, message: &str, tone: Tone) -> Response {
    match tone {
        Tone::Normal => ui.label(message),
        Tone::Warning => ui.colored_label(ui.visuals().warn_fg_color, message),
        Tone::Error => ui.colored_label(ui.visuals().error_fg_color, message),
    }
}

/// Returns true when the user dismisses a notice held by the caller.
pub fn dismissible_notice(ui: &mut Ui, message: &str, tone: Tone, dismiss: &str) -> bool {
    ui.group(|ui| {
        ui.horizontal_wrapped(|ui| {
            notice(ui, message, tone);
            ui.small_button(dismiss).clicked()
        })
        .inner
    })
    .inner
}

pub fn status(ui: &mut Ui, message: &str, error: bool) {
    notice(ui, message, if error { Tone::Error } else { Tone::Normal });
}

pub fn loading(ui: &mut Ui, message: &str) {
    ui.horizontal(|ui| {
        ui.spinner();
        ui.label(message);
    });
}

pub fn empty_state(ui: &mut Ui, title: &str, guidance: &str) {
    ui.group(|ui| {
        ui.strong(title);
        ui.label(guidance);
    });
}

fn text_input(ui: &mut Ui, label: &str, hint: &str, value: &mut String) -> Response {
    let caption = ui.label(label);
    ui.add(egui::TextEdit::singleline(value).hint_text(hint))
        .labelled_by(caption.id)
}

/// Single-line input with a visible label linked to the accessibility tree.
///
/// ```
/// use starter_desktop::{egui, field, notice, Tone};
///
/// fn title_editor(ui: &mut egui::Ui, draft: &mut String) {
///     field(ui, "Project title", draft);
///     if draft.trim().is_empty() {
///         notice(ui, "A title is required", Tone::Warning);
///     }
/// }
/// ```
pub fn field(ui: &mut Ui, label: &str, value: &mut String) -> Response {
    text_input(ui, label, "", value)
}

/// Search field. The application applies its own filtering rules.
pub fn search_field(ui: &mut Ui, label: &str, hint: &str, value: &mut String) -> Response {
    text_input(ui, label, hint, value)
}

pub fn property_grid(ui: &mut Ui, id: impl egui::AsIdSalt, rows: impl FnOnce(&mut Ui)) {
    egui::Grid::new(id)
        .num_columns(2)
        .striped(true)
        .show(ui, rows);
}

pub fn progress(ui: &mut Ui, progress: f32) {
    let progress = if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    };
    ui.add(egui::ProgressBar::new(progress).show_percentage());
}

pub fn unit_field(
    ui: &mut Ui,
    label: &str,
    unit: &str,
    value: &mut f64,
    range: RangeInclusive<f64>,
) -> Response {
    ui.add(
        egui::DragValue::new(value)
            .range(range)
            .prefix(label)
            .suffix(unit),
    )
}

/// Keep icon actions readable with a caption, including when the font lacks the icon.
pub fn icon_button(ui: &mut Ui, glyph: &str, caption: &str) -> Response {
    let font = egui::TextStyle::Button.resolve(ui.style());
    if ui.ctx().fonts_mut(|fonts| fonts.has_glyphs(&font, glyph)) {
        ui.button(format!("{glyph} {caption}"))
    } else {
        ui.button(caption)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogChoice {
    Cancel,
    Primary,
    Secondary,
}

/// An egui modal for user decisions. Esc and clicking the backdrop cancel.
pub fn confirmation(
    ctx: &egui::Context,
    id: egui::Id,
    title: &str,
    message: &str,
    labels: (&str, &str, Option<&str>),
) -> Option<DialogChoice> {
    let response = egui::Modal::new(id).show(ctx, |ui| {
        ui.heading(title);
        ui.label(message);
        let (cancel, primary, secondary) = labels;
        ui.horizontal_wrapped(|ui| {
            if ui.button(cancel).clicked() {
                return Some(DialogChoice::Cancel);
            }
            if ui.button(primary).clicked() {
                return Some(DialogChoice::Primary);
            }
            if secondary.is_some_and(|label| ui.button(label).clicked()) {
                return Some(DialogChoice::Secondary);
            }
            None
        })
        .inner
    });
    response
        .inner
        .or_else(|| response.should_close().then_some(DialogChoice::Cancel))
}
