use egui::{Color32, Context, Ui};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Tokens {
    pub dark: bool,
    pub compact: bool,
    pub spanish: bool,
}
impl Default for Tokens {
    fn default() -> Self {
        Self {
            dark: true,
            compact: false,
            spanish: false,
        }
    }
}
impl Tokens {
    pub fn label(self, en: &'static str, es: &'static str) -> &'static str {
        if self.spanish { es } else { en }
    }
    pub fn apply(self, ctx: &Context) {
        Design::default().apply(ctx, self);
    }

    fn theme(self) -> egui::Theme {
        if self.dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        }
    }
}

/// Product-level visual overrides. User theme and density preferences are kept in Tokens.
#[derive(Clone, Copy, Debug, Default)]
pub struct Design {
    pub dark_accent: Option<Color32>,
    pub light_accent: Option<Color32>,
}
impl Design {
    pub fn apply(self, ctx: &Context, tokens: Tokens) {
        let theme = tokens.theme();
        ctx.set_theme(theme);
        ctx.style_mut_of(theme, |style| self.configure(style, tokens));
    }

    pub(crate) fn preview(self, ui: &mut Ui, tokens: Tokens) {
        let mut style = ui.ctx().style_of(tokens.theme()).as_ref().clone();
        self.configure(&mut style, tokens);
        ui.set_style(style);
    }

    fn configure(self, style: &mut egui::Style, tokens: Tokens) {
        let gap = if tokens.compact { 4.0 } else { 8.0 };
        let body = if tokens.compact { 13.0 } else { 15.0 };
        let heading = if tokens.compact { 19.0 } else { 23.0 };
        style.spacing.item_spacing = egui::vec2(gap, gap);
        style.spacing.button_padding = egui::vec2(gap + 1.0, gap / 2.0);
        style.spacing.interact_size.y = if tokens.compact { 23.0 } else { 30.0 };
        for (role, size) in [
            (egui::TextStyle::Heading, heading),
            (egui::TextStyle::Body, body),
            (egui::TextStyle::Button, body),
            (egui::TextStyle::Small, body - 2.0),
        ] {
            style
                .text_styles
                .insert(role, egui::FontId::proportional(size));
        }
        let palette = if tokens.dark {
            (
                self.dark_accent.unwrap_or(Color32::from_rgb(0, 91, 135)),
                Color32::from_rgb(255, 158, 166),
                Color32::from_rgb(249, 210, 121),
            )
        } else {
            (
                self.light_accent
                    .unwrap_or(Color32::from_rgb(186, 220, 246)),
                Color32::from_rgb(150, 26, 42),
                Color32::from_rgb(112, 66, 4),
            )
        };
        let visuals = &mut style.visuals;
        visuals.selection.bg_fill = palette.0;
        visuals.error_fg_color = palette.1;
        visuals.warn_fg_color = palette.2;
        visuals.selection.stroke.width = 1.5;
        let radius = egui::CornerRadius::same(if tokens.compact { 3 } else { 5 });
        for visual in [
            &mut visuals.widgets.noninteractive,
            &mut visuals.widgets.inactive,
            &mut visuals.widgets.hovered,
            &mut visuals.widgets.active,
            &mut visuals.widgets.open,
        ] {
            visual.corner_radius = radius;
        }
    }
}
