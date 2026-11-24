#![forbid(unsafe_code)]
pub use glam::{Mat4, Vec2, Vec3};
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub ortho: bool,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 0.72,
            pitch: 0.48,
            distance: 6.0,
            ortho: false,
        }
    }
}
impl Camera {
    pub fn matrix(&self, aspect: f32) -> Mat4 {
        let eye = self.distance
            * Vec3::new(
                self.yaw.sin() * self.pitch.cos(),
                self.pitch.sin(),
                self.yaw.cos() * self.pitch.cos(),
            );
        let projection = if self.ortho {
            let r = self.distance * 0.4;
            Mat4::orthographic_rh(-r * aspect, r * aspect, -r, r, 0.025, 250.0)
        } else {
            Mat4::perspective_rh(0.75, aspect, 0.025, 250.0)
        };
        projection * Mat4::look_at_rh(eye, Vec3::ZERO, Vec3::Y)
    }
    pub fn ray(&self, uv: Vec2, aspect: f32) -> (Vec3, Vec3) {
        let view = self.matrix(aspect).inverse();
        let pos = Vec2::new(2.0 * uv.x - 1.0, 1.0 - 2.0 * uv.y);
        let near = view.project_point3(pos.extend(0.0));
        let far = view.project_point3(pos.extend(1.0));
        (near, (far - near).normalize())
    }
}
pub struct Mesh {
    pub id: u64,
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[usize; 3]>,
    pub transform: Mat4,
}
impl Mesh {
    pub fn cuboid(id: u64, half: Vec3) -> Self {
        let vertices = [
            [-1., -1., -1.],
            [1., -1., -1.],
            [1., 1., -1.],
            [-1., 1., -1.],
            [-1., -1., 1.],
            [1., -1., 1.],
            [1., 1., 1.],
            [-1., 1., 1.],
        ]
        .map(|v| Vec3::from_array(v) * half)
        .to_vec();
        Self {
            id,
            vertices,
            transform: Mat4::IDENTITY,
            triangles: vec![
                [0, 1, 2],
                [0, 2, 3],
                [4, 6, 5],
                [4, 7, 6],
                [0, 4, 5],
                [0, 5, 1],
                [1, 5, 6],
                [1, 6, 2],
                [2, 6, 7],
                [2, 7, 3],
                [3, 7, 4],
                [3, 4, 0],
            ],
        }
    }
    pub fn faces(&self) -> impl Iterator<Item = [Vec3; 3]> + '_ {
        self.triangles.iter().filter_map(|[a, b, c]| {
            Some(
                [
                    *self.vertices.get(*a)?,
                    *self.vertices.get(*b)?,
                    *self.vertices.get(*c)?,
                ]
                .map(|p| self.transform.transform_point3(p)),
            )
        })
    }
}
#[derive(Default)]
pub struct Scene(pub Vec<Mesh>);
impl Scene {
    pub fn pick(&self, origin: Vec3, dir: Vec3) -> Option<u64> {
        self.0
            .iter()
            .flat_map(|mesh| {
                mesh.faces().filter_map(move |[a, b, c]| {
                    let ab = b - a;
                    let ac = c - a;
                    let p = dir.cross(ac);
                    let det = ab.dot(p);
                    if det.abs() <= f32::EPSILON * ab.length() * ac.length() {
                        return None;
                    }
                    let t = origin - a;
                    let q = t.cross(ab);
                    let u = t.dot(p) / det;
                    let v = dir.dot(q) / det;
                    let distance = ac.dot(q) / det;
                    (u >= 0.0 && v >= 0.0 && u + v <= 1.0 && distance > 0.0)
                        .then_some((distance, mesh.id))
                })
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, id)| id)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn perspective_and_orthographic_picking() {
        let mut camera = Camera::default();
        let scene = Scene(vec![Mesh::cuboid(3, Vec3::ONE)]);
        for ortho in [false, true] {
            camera.ortho = ortho;
            let (o, d) = camera.ray(Vec2::splat(0.5), 1.0);
            assert_eq!(scene.pick(o, d), Some(3));
        }
    }
}
