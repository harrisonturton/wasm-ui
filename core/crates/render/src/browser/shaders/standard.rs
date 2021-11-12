use web_sys::WebGlProgram;
use anyhow::Error;
use math::{Vector2, Vector3, Vector4};

use super::WebGl;
use super::MeshPainter;
use std::rc::Rc;

const VERTEX_SHADER: &str = r#"
// Position of the vertex
attribute vec2 a_position;

// Pixel dimensions of the canvas
uniform vec2 u_viewport;

void main() {
    vec2 zero_to_one = a_position / u_viewport;
    vec2 zero_to_two = zero_to_one * 2.0;
    vec2 clip_space = zero_to_two - 1.0;
    gl_Position = vec4(clip_space, 0.0, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"
precision mediump float;

// Color of the rect
uniform vec4 u_color;

void main() {
    gl_FragColor = u_color;
}
"#;

pub struct StandardShader {
    gl: Rc<WebGl>,
    program: WebGlProgram,

    viewport: Vector2,
    color: Vector4,
}

impl StandardShader {
    pub fn try_new(gl: &Rc<WebGl>) -> Result<StandardShader, Error> {
        let program = gl.try_create_shader_program(
            VERTEX_SHADER,
            FRAGMENT_SHADER
        )?;
        Ok(StandardShader { 
            gl: Rc::clone(gl),
            program,
            viewport: Vector2::zero(),
            color: Vector4::new(1.0, 1.0, 1.0, 1.0),
        })
    }

    pub fn set_viewport(&mut self, viewport: Vector2) {
        self.viewport = viewport;
    }

    pub fn set_color(&mut self, color: Vector4) {
        self.color = color;
    }

    fn set_uniforms(&self) -> Result<(), Error> {
        self.gl.set_uniform_vec2(&self.program, "u_viewport", self.viewport)?;
        self.gl.set_uniform_vec4(&self.program, "u_color", self.color)?;
        Ok(())
    }
}

impl MeshPainter for StandardShader {
    fn paint_mesh(&self, vertices: &[Vector3]) -> Result<(), Error> {
        self.set_uniforms()?;
        let buffer = self.gl.new_array_buffer(vertices)?;
        self.gl.draw_mesh(&self.program, "a_position", &buffer)?;
        Ok(())
    }

    fn paint_line(&self, vertices: &[Vector3]) -> Result<(), Error> {
        self.set_uniforms()?;
        let buffer = self.gl.new_array_buffer(vertices)?;
        self.gl.draw_line(&self.program, "a_position", &buffer)?;
        Ok(())
    }
}