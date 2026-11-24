#![forbid(unsafe_code)]
use egui::{Sense, Ui};
use egui_wgpu::RenderState;
use starter_viewport_core::{Camera, Scene, Vec2};
use starter_viewport_wgpu::{Renderer, wgpu};
pub struct Viewport {
    state: RenderState,
    renderer: Renderer,
    texture: egui::TextureId,
    pub camera: Camera,
    pub selected: Option<u64>,
}
impl Viewport {
    pub fn new(state: RenderState) -> Self {
        let renderer = Renderer::new(&state.device, 800, 480);
        let view = renderer.color.create_view(&Default::default());
        let texture = state.renderer.write().register_native_texture(
            &state.device,
            &view,
            wgpu::FilterMode::Linear,
        );
        Self {
            state,
            renderer,
            texture,
            camera: Camera::default(),
            selected: None,
        }
    }
    pub fn show(&mut self, ui: &mut Ui, scene: &Scene) {
        ui.checkbox(&mut self.camera.ortho, "Orthographic projection");
        let aspect = self.renderer.width as f32 / self.renderer.height as f32;
        let width = ui.available_width().clamp(1., 800.);
        let response = ui.add(
            egui::Image::new((self.texture, egui::vec2(width, width / aspect)))
                .sense(Sense::click_and_drag()),
        );
        if response.dragged() {
            let d = response.drag_delta();
            self.camera.yaw -= d.x * 0.012;
            self.camera.pitch = (self.camera.pitch + d.y * 0.012).clamp(-1.5, 1.5);
        }
        if response.hovered() {
            self.camera.distance = (self.camera.distance
                * (-0.005 * ui.input(|i| i.smooth_scroll_delta.y)).exp())
            .clamp(0.5, 40.0);
        }
        if let Some(pos) = response
            .interact_pointer_pos()
            .filter(|_| response.clicked())
        {
            let uv = (pos - response.rect.min) / response.rect.size();
            let (o, d) = self.camera.ray(Vec2::new(uv.x, uv.y), aspect);
            self.selected = scene.pick(o, d);
        }
        self.renderer.render(
            &self.state.device,
            &self.state.queue,
            scene,
            &self.camera,
            self.selected,
        );
    }
}
impl Drop for Viewport {
    fn drop(&mut self) {
        self.state.renderer.write().free_texture(&self.texture);
    }
}
