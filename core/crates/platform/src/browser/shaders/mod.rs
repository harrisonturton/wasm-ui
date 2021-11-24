use super::WebGl;
use anyhow::Error;
use math::Vector3;
use std::rc::Rc;

mod standard;
pub use standard::*;

pub struct ShaderLibrary {
    pub standard: StandardShader,
}

impl ShaderLibrary {
    pub fn try_new(gl: &Rc<WebGl>) -> Result<ShaderLibrary, Error> {
        super::util::log("before standard shader");
        let standard = StandardShader::try_new(gl)?;
        super::util::log("after standard shader");
        Ok(ShaderLibrary { standard })
    }
}
