#![forbid(unsafe_code)]
use starter_viewport_core::{Camera, Scene, Vec3};
pub use wgpu;
use wgpu::util::DeviceExt;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 4],
    color: [f32; 4],
}
pub struct Renderer {
    pub color: wgpu::Texture,
    depth: wgpu::Texture,
    lines: wgpu::RenderPipeline,
    faces: wgpu::RenderPipeline,
    pub width: u32,
    pub height: u32,
}
impl Renderer {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture = |format, usage| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("scene.wgsl").into()),
        });
        let pipeline = |topology| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: None,
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: 32,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4],
                    })],
                },
                primitive: wgpu::PrimitiveState {
                    topology,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Less),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(FORMAT.into())],
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        Self {
            color: texture(
                FORMAT,
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
            ),
            depth: texture(DEPTH, wgpu::TextureUsages::RENDER_ATTACHMENT),
            lines: pipeline(wgpu::PrimitiveTopology::LineList),
            faces: pipeline(wgpu::PrimitiveTopology::TriangleList),
            width,
            height,
        }
    }
    pub fn render(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: &Scene,
        camera: &Camera,
        selected: Option<u64>,
    ) {
        let matrix = camera.matrix(self.width as f32 / self.height as f32);
        let vertex = |p: Vec3, color| Vertex {
            position: (matrix * p.extend(1.0)).to_array(),
            color,
        };
        let grid: Vec<_> = (-6..=6)
            .flat_map(|i| {
                let p = i as f32;
                [
                    Vec3::new(p, -1.01, -6.),
                    Vec3::new(p, -1.01, 6.),
                    Vec3::new(-6., -1.01, p),
                    Vec3::new(6., -1.01, p),
                ]
                .map(|p| vertex(p, [0.27, 0.30, 0.38, 1.]))
            })
            .collect();
        let mut faces = Vec::new();
        for mesh in &scene.0 {
            for [a, b, c] in mesh.faces() {
                let light = 0.50
                    + 0.50
                        * (b - a)
                            .cross(c - a)
                            .normalize_or_zero()
                            .dot(Vec3::new(0.2, 0.9, 0.4))
                            .abs();
                let color = if Some(mesh.id) == selected {
                    [light, 0.65 * light, 0.1, 1.]
                } else {
                    [0.1, 0.65 * light, light, 1.]
                };
                faces.extend([a, b, c].map(|p| vertex(p, color)));
            }
        }
        let color = self.color.create_view(&Default::default());
        let depth = self.depth.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&Default::default());
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &color,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.055,
                        g: 0.07,
                        b: 0.09,
                        a: 1.,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth,
                stencil_ops: None,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
            }),
            ..Default::default()
        });
        for (pipeline, vertices) in [(&self.lines, &grid), (&self.faces, &faces)] {
            if vertices.is_empty() {
                continue;
            }
            let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            pass.set_pipeline(pipeline);
            pass.set_vertex_buffer(0, buffer.slice(..));
            pass.draw(0..vertices.len() as u32, 0..1);
        }
        drop(pass);
        queue.submit([encoder.finish()]);
    }
    pub fn capture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let packed = self.width * 4;
        let pitch = packed.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: u64::from(pitch) * u64::from(self.height),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_texture_to_buffer(
            self.color.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(pitch),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        let submission = queue.submit([encoder.finish()]);
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
            });
        let timeout = std::time::Duration::from_secs(10);
        device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: Some(timeout),
        })?;
        rx.recv_timeout(timeout)??;
        let bytes = buffer
            .slice(..)
            .get_mapped_range()?
            .chunks_exact(pitch as usize)
            .flat_map(|row| row[..packed as usize].iter().copied())
            .collect();
        buffer.unmap();
        Ok(bytes)
    }
}
#[cfg(all(test, feature = "gpu-evidence"))]
mod tests {
    use super::*;
    use starter_viewport_core::Mesh;
    #[test]
    fn captures_scene_and_background() {
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle().with_env());
        let adapter = pollster::block_on(instance.request_adapter(&Default::default())).unwrap();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let renderer = Renderer::new(&device, 320, 240);
        renderer.render(
            &device,
            &queue,
            &Scene(vec![Mesh::cuboid(1, Vec3::ONE)]),
            &Camera::default(),
            None,
        );
        let image =
            image::RgbaImage::from_raw(320, 240, renderer.capture(&device, &queue).unwrap())
                .unwrap();
        assert!(image.get_pixel(160, 120)[2] > 100);
        assert!(image.get_pixel(0, 0)[2] < 40);
        if let Some(path) = std::env::var_os("STARTER_VIEWPORT_CAPTURE") {
            image.save(path).unwrap();
        }
    }
}
