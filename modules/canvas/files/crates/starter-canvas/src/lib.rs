#![forbid(unsafe_code)]
use egui::{Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, pos2, vec2};
pub struct Camera {
    pan: Vec2,
    zoom: f32,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 42.0,
        }
    }
}
impl Camera {
    pub fn screen(&self, rect: Rect, world: Pos2) -> Pos2 {
        rect.center() + self.pan + vec2(world.x, -world.y) * self.zoom
    }
    pub fn world(&self, rect: Rect, screen: Pos2) -> Pos2 {
        let offset = (screen - rect.center() - self.pan) / self.zoom;
        pos2(offset.x, -offset.y)
    }
    pub fn snap(point: Pos2, step: f32) -> Pos2 {
        if step.is_finite() && step > 0.0 {
            pos2(
                (point.x / step).round() * step,
                (point.y / step).round() * step,
            )
        } else {
            point
        }
    }
    pub fn interact(&mut self, ui: &mut Ui, height: f32) -> Response {
        let (_, r) =
            ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click_and_drag());
        if r.dragged() {
            self.pan += r.drag_delta();
        }
        if let Some(cursor) = r.hover_pos() {
            let anchor = self.world(r.rect, cursor);
            let delta = ui.input(|i| i.smooth_scroll_delta.y);
            self.zoom = (self.zoom * (delta * 0.006).exp()).clamp(4.0, 600.0);
            self.pan += cursor - self.screen(r.rect, anchor);
        }
        r
    }
    pub fn grid(&self, ui: &Ui, rect: Rect) {
        let painter = ui.painter_at(rect);
        let step = self.zoom * (24.0 / self.zoom).ceil();
        let origin = self.screen(rect, Pos2::ZERO);
        let stroke = Stroke::new(1.0, ui.visuals().weak_text_color().gamma_multiply(0.35));
        let mut x = rect.left() + (origin.x - rect.left()).rem_euclid(step);
        while x <= rect.right() {
            painter.line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], stroke);
            x += step;
        }
        let mut y = rect.top() + (origin.y - rect.top()).rem_euclid(step);
        while y <= rect.bottom() {
            painter.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], stroke);
            y += step;
        }
    }
}
