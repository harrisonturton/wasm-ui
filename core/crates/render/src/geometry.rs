use math::Vector2;

/// The position of a point of a primitive, usually a triangle.
pub type Vertex = Vector2;

/// A mesh holds all the vertices to render a mesh on the screen.
#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
}

pub const QUAD: Mesh = Mesh { vertices: vec![] };
