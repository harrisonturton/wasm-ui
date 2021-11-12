use super::WebGl;
use anyhow::Error;
use math::Vector3;
use std::rc::Rc;

mod standard;
pub use standard::*;

pub trait MeshPainter {
    fn paint_mesh(&self, vertices: &[Vector3]) -> Result<(), Error>;
    fn paint_line(&self, vertices: &[Vector3]) -> Result<(), Error>;
}

pub struct ShaderLibrary {
    pub standard: StandardShader,
}

impl ShaderLibrary {
    pub fn try_new(gl: &Rc<WebGl>) -> Result<ShaderLibrary, Error> {
        let standard = StandardShader::try_new(gl)?;
        Ok(ShaderLibrary { standard })
    }
}
