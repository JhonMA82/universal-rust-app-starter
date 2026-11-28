use egui::{Atom, Button, Painter, Rect, Response, Stroke, Ui, Vec2, pos2};

/// Decorative marks for common actions. Each Button still has a visible caption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Add,
    Close,
    Search,
    Reset,
}

/// A native egui button with a scalable vector mark and a single accessible caption.
pub fn button_with_icon(ui: &mut Ui, icon: Icon, caption: &str) -> Response {
    let width = ui.spacing().icon_width;
    let mark = Atom::paint(Vec2::splat(width), move |ui, args| {
        let stroke = Stroke::new((width / 9.0).max(1.0), args.fallback_text_color);
        icon.paint(ui.painter(), args.rect, stroke);
    });
    ui.add(Button::new((mark, caption)).gap(ui.spacing().icon_spacing))
}

impl Icon {
    fn paint(self, painter: &Painter, rect: Rect, stroke: Stroke) {
        let point = |x: f32, y: f32| {
            pos2(
                rect.left() + rect.width() * x,
                rect.top() + rect.height() * y,
            )
        };
        let line = |a, b| {
            painter.line_segment([a, b], stroke);
        };
        match self {
            Self::Add => {
                line(point(0.2, 0.5), point(0.8, 0.5));
                line(point(0.5, 0.2), point(0.5, 0.8));
            }
            Self::Close => {
                line(point(0.23, 0.23), point(0.77, 0.77));
                line(point(0.23, 0.77), point(0.77, 0.23));
            }
            Self::Search => {
                painter.circle_stroke(point(0.44, 0.44), rect.width() * 0.22, stroke);
                line(point(0.6, 0.6), point(0.81, 0.81));
            }
            Self::Reset => {
                painter.line(
                    vec![
                        point(0.17, 0.46),
                        point(0.32, 0.25),
                        point(0.57, 0.21),
                        point(0.78, 0.35),
                        point(0.83, 0.58),
                        point(0.67, 0.78),
                        point(0.44, 0.81),
                    ],
                    stroke,
                );
                line(point(0.17, 0.46), point(0.16, 0.24));
                line(point(0.17, 0.46), point(0.39, 0.46));
            }
        }
    }
}
