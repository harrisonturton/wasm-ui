use crate::webgl::util;
use anyhow::Error;
use web_sys::{WebGlProgram, WebGlRenderingContext};

/// Holds all the different shaders we might want to use when rendering.  This a
/// utility struct that makes it easier to swap between different shaders.
pub struct ShaderLibrary {
    pub standard: WebGlProgram,
}

impl ShaderLibrary {
    pub fn try_new(gl: &WebGlRenderingContext) -> Result<ShaderLibrary, Error> {
        let standard_shader = util::try_create_shader_program(
            gl,
            include_str!("shaders/standard_vertex.glsl"),
            include_str!("shaders/standard_fragment.glsl"),
        )?;
        Ok(Self {
            standard: standard_shader,
        })
    }
}
