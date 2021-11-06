use math::{Vector3D, Mat4};
use std::collections::VecDeque;

#[derive(Clone, Debug, Default)]
pub struct MatrixStack {
    pub matrices: VecDeque<Mat4>
}

impl MatrixStack {
    pub fn new() -> Self {
        Self { matrices: VecDeque::new() }
    }

    pub fn push(&mut self, matrix: Mat4) -> &mut Self {
        self.matrices.push_back(matrix);
        self
    }

    pub fn pop(&mut self) -> Option<Mat4> {
        self.matrices.pop_front()
    }

    pub fn translate(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        let mat = Mat4::new_translate((x, y, z));
        self.push(mat);
        self
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        let mat = Mat4::new_scale((x, y, z));
        self.push(mat);
        self
    }

    pub fn rotate(&mut self, x: f32, y: f32, z: f32) -> &mut Self {
        let mat = Mat4::new_rotate((x, y, z));
        self.push(mat);
        self
    }

    pub fn current(&self) -> Option<&Mat4> {
        self.matrices.get(0)
    }

    pub fn product(&self) -> Mat4 {
        let mut result = Mat4::new_unit();
        for mat in &self.matrices {
            result = result * *mat;
        }
        result
    }
}

#[derive(Clone, Debug)]
pub struct Object<'a> {
    pub transform: MatrixStack,
    pub vertices: &'a [Vertex],
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Transform {
    pub position: Vector3D,
    pub scale: Vector3D,
    pub rotation: Vector3D,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vector3D,
}

impl Vertex {
    pub fn from_pos(x: f32, y: f32, z: f32) -> Self {
        Self { position: Vector3D::new(x, y, z) }
    }
}

pub fn cube() -> [Vertex; 36] {
    [
        // Front face
        Vertex::from_pos(0.0, 1.0, 0.0),
        Vertex::from_pos(0.0, 0.0, 0.0),
        Vertex::from_pos(1.0, 0.0, 0.0),
        Vertex::from_pos(0.0, 1.0, 0.0),
        Vertex::from_pos(1.0, 0.0, 0.0),
        Vertex::from_pos(1.0, 1.0, 0.0),
        // Top face
        Vertex::from_pos(0.0, 1.0, 0.0),
        Vertex::from_pos(0.0, 1.0, 1.0),
        Vertex::from_pos(1.0, 1.0, 1.0),
        Vertex::from_pos(0.0, 1.0, 0.0),
        Vertex::from_pos(0.0, 1.0, 1.0),
        Vertex::from_pos(0.0, 1.0, 0.0),
        // Bottom face
        Vertex::from_pos(0.0, 0.0, 0.0),
        Vertex::from_pos(0.0, 0.0, 1.0),
        Vertex::from_pos(1.0, 0.0, 1.0),
        Vertex::from_pos(0.0, 0.0, 0.0),
        Vertex::from_pos(1.0, 0.0, 1.0),
        Vertex::from_pos(1.0, 0.0, 0.0),
        // Left face
        Vertex::from_pos(0.0, 1.0, 0.0),
        Vertex::from_pos(0.0, 0.0, 0.0),
        Vertex::from_pos(0.0, 0.0, 1.0),
        Vertex::from_pos(0.0, 1.0, 0.0),
        Vertex::from_pos(0.0, 0.0, 1.0),
        Vertex::from_pos(0.0, 1.0, 1.0),
        // Right face
        Vertex::from_pos(1.0, 1.0, 0.0),
        Vertex::from_pos(1.0, 0.0, 0.0),
        Vertex::from_pos(1.0, 0.0, 1.0),
        Vertex::from_pos(1.0, 1.0, 0.0),
        Vertex::from_pos(1.0, 0.0, 1.0),
        Vertex::from_pos(1.0, 1.0, 1.0),
        // Back face
        Vertex::from_pos(0.0, 1.0, 1.0),
        Vertex::from_pos(0.0, 0.0, 1.0),
        Vertex::from_pos(1.0, 0.0, 1.0),
        Vertex::from_pos(0.0, 1.0, 1.0),
        Vertex::from_pos(1.0, 0.0, 1.0),
        Vertex::from_pos(1.0, 1.0, 1.0),
    ]
}