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

    /// Metrics for product widgets that need to match the surrounding shell.
    pub fn metrics(self) -> Metrics {
        if self.compact {
            Metrics {
                gap: 4,
                body: 13.0,
                heading: 19.0,
                control: 23.0,
                icon: 14.0,
                radius: 3,
            }
        } else {
            Metrics {
                gap: 8,
                body: 15.0,
                heading: 23.0,
                control: 30.0,
                icon: 16.0,
                radius: 5,
            }
        }
    }

    fn theme(self) -> egui::Theme {
        if self.dark {
            egui::Theme::Dark
        } else {
            egui::Theme::Light
        }
    }
}

/// The shell's density scale, expressed in egui points.
#[derive(Clone, Copy, Debug)]
pub struct Metrics {
    pub gap: i8,
    pub body: f32,
    pub heading: f32,
    pub control: f32,
    pub icon: f32,
    pub radius: u8,
}

/// Semantic colors selected for the current appearance and product brand.
#[derive(Clone, Copy, Debug)]
pub struct ThemeColors {
    pub accent: Color32,
    pub on_accent: Color32,
    pub warning: Color32,
    pub error: Color32,
}

/// Product-level visual overrides. User theme and density preferences are kept in Tokens.
#[derive(Clone, Copy, Debug, Default)]
pub struct Design {
    pub dark_accent: Option<Color32>,
    pub light_accent: Option<Color32>,
}
impl Design {
    pub fn colors(self, tokens: Tokens) -> ThemeColors {
        let (accent, warning, error) = if tokens.dark {
            (
                self.dark_accent.unwrap_or(Color32::from_rgb(0, 91, 135)),
                Color32::from_rgb(249, 210, 121),
                Color32::from_rgb(255, 158, 166),
            )
        } else {
            (
                self.light_accent
                    .unwrap_or(Color32::from_rgb(186, 220, 246)),
                Color32::from_rgb(112, 66, 4),
                Color32::from_rgb(150, 26, 42),
            )
        };
        let accent = accent.to_opaque();
        // At this luminance the contrast against white and black is equal.
        let on_accent = if luminance(accent) >= 0.179 {
            Color32::BLACK
        } else {
            Color32::WHITE
        };
        ThemeColors {
            accent,
            on_accent,
            warning,
            error,
        }
    }

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
        let metrics = tokens.metrics();
        let gap = f32::from(metrics.gap);
        style.spacing.item_spacing = egui::vec2(gap, gap);
        style.spacing.button_padding = egui::vec2(gap + 1.0, gap / 2.0);
        style.spacing.interact_size.y = metrics.control;
        style.spacing.icon_width = metrics.icon;
        style.spacing.icon_spacing = gap;
        for (role, size) in [
            (egui::TextStyle::Heading, metrics.heading),
            (egui::TextStyle::Body, metrics.body),
            (egui::TextStyle::Button, metrics.body),
            (egui::TextStyle::Small, metrics.body - 2.0),
        ] {
            style
                .text_styles
                .insert(role, egui::FontId::proportional(size));
        }
        let colors = self.colors(tokens);
        let visuals = &mut style.visuals;
        visuals.selection.bg_fill = colors.accent;
        visuals.selection.stroke = egui::Stroke::new(1.5, colors.on_accent);
        visuals.error_fg_color = colors.error;
        visuals.warn_fg_color = colors.warning;
        let radius = egui::CornerRadius::same(metrics.radius);
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

fn luminance(color: Color32) -> f32 {
    // Rgba channels are linear, unlike the source Color32.
    let linear = egui::Rgba::from(color);
    0.2126 * linear.r() + 0.7152 * linear.g() + 0.0722 * linear.b()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contrast(first: Color32, second: Color32) -> f32 {
        let first = luminance(first);
        let second = luminance(second);
        (first.max(second) + 0.05) / (first.min(second) + 0.05)
    }

    #[test]
    fn semantic_feedback_and_selection_keep_readable_contrast() {
        for dark in [true, false] {
            let preferences = Tokens {
                dark,
                ..Default::default()
            };
            let design = Design::default();
            let colors = design.colors(preferences);
            let background = if dark {
                egui::Visuals::dark()
            } else {
                egui::Visuals::light()
            };
            assert!(contrast(colors.on_accent, colors.accent) >= 4.5);
            assert!(contrast(colors.error, background.panel_fill) >= 4.5);
            assert!(contrast(colors.warning, background.panel_fill) >= 4.5);
        }
        let custom = Design {
            dark_accent: Some(Color32::from_rgb(200, 215, 245)),
            light_accent: None,
        }
        .colors(Tokens::default());
        assert_eq!(custom.on_accent, Color32::BLACK);
        assert!(contrast(custom.on_accent, custom.accent) >= 4.5);
    }
}
