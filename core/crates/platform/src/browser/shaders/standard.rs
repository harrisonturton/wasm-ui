use anyhow::Error;
use math::{Vector2, Vector3, Vector4, Rect};
use web_sys::WebGlProgram;

use super::WebGl;
use std::rc::Rc;
use layout::{Borders, BorderSide, Color, Material};

const VERTEX_SHADER: &str = r#"
// Position of the vertex
attribute vec2 a_position;

// Pixel dimensions of the canvas
uniform vec2 u_viewport;

varying vec2 v_position;

void main() {
    v_position = a_position;
    vec2 zero_to_one = a_position / u_viewport;
    vec2 zero_to_two = zero_to_one * 2.0;
    vec2 clip_space = zero_to_two - 1.0;
    vec2 origin_top_left = vec2(1.0, -1.0) * clip_space;
    gl_Position = vec4(origin_top_left, 0.0, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"
precision mediump float;

// Color of the rect
uniform vec4 u_color;

struct border_side {
    float width;
    vec4 color;
};

uniform border_side u_borders[4];

uniform vec2 u_rect_min;
uniform vec2 u_rect_max;

// Current position
varying vec2 v_position;

void main() {
    border_side top_border = u_borders[0];
    border_side bottom_border = u_borders[1];
    border_side left_border = u_borders[2];
    border_side right_border = u_borders[3];

    if (v_position.y <= u_rect_min.y + top_border.width) {
        gl_FragColor = top_border.color;
        return;
    }
    if (v_position.y >= u_rect_max.y - bottom_border.width) {
        gl_FragColor = bottom_border.color;
        return;
    }
    if (v_position.x <= u_rect_min.x + left_border.width) {
        gl_FragColor = left_border.color;
        return;
    }
    if (v_position.x >= u_rect_max.x - right_border.width) {
        gl_FragColor = right_border.color;
        return;
    }
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
        crate::browser::util::log("before create shader program");
        let program = gl.try_create_shader_program(VERTEX_SHADER, FRAGMENT_SHADER)?;
        crate::browser::util::log("after create shader program");
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
        self.gl
            .set_uniform_vec2(&self.program, "u_viewport", self.viewport)?;
        Ok(())
    }

    pub fn paint_mesh(&self, vertices: &[Vector3]) -> Result<(), Error> {
        self.set_uniforms()?;
        let buffer = self.gl.new_array_buffer(vertices)?;
        self.gl.draw_mesh(&self.program, "a_position", &buffer)?;
        Ok(())
    }

    pub fn paint_line(&self, vertices: &[Vector3]) -> Result<(), Error> {
        self.set_uniforms()?;
        let buffer = self.gl.new_array_buffer(vertices)?;
        self.gl.draw_line(&self.program, "a_position", &buffer)?;
        Ok(())
    }

    pub fn paint_rect(&mut self, rect: Rect, material: Material) -> Result<(), Error> {
        self.set_color(material.fill.to_linear());
        let (min_x, min_y) = rect.min.into();
        let (max_x, max_y) = rect.max.into();
        self.gl
            .set_uniform_vec4(&self.program, "u_color", self.color)?;
        self.set_borders(material.borders)?;
        self.gl
            .set_uniform_vec2(&self.program, "u_rect_max", rect.max)?;
        self.gl
            .set_uniform_vec2(&self.program, "u_rect_min", rect.min)?;
        let vertices: [Vector3; 6] = [
            Vector3::new(min_x, min_y, 0.0),
            Vector3::new(min_x, max_y, 0.0),
            Vector3::new(max_x, min_y, 0.0),
            Vector3::new(min_x, max_y, 0.0),
            Vector3::new(max_x, min_y, 0.0),
            Vector3::new(max_x, max_y, 0.0),
        ];
        self.paint_mesh(&vertices)?;
        Ok(())
    }

    fn set_borders(&mut self, borders: Borders) -> Result<(), Error> {
        self.set_border(0, borders.top)?;
        self.set_border(1, borders.bottom)?;
        self.set_border(2, borders.left)?;
        self.set_border(3, borders.right)?;
        Ok(())
    }

    fn set_border(&mut self, i: usize, maybe_border: Option<BorderSide>) -> Result<(), Error> {
        let BorderSide { color, width } = match maybe_border {
            Some(border) => border,
            None => BorderSide::new(Color::transparent(), 0.0),
        };
        self.gl.set_uniform_f32(&self.program, &format!("u_borders[{:?}].width", i), width)?;
        self.gl.set_uniform_vec4(&self.program, &format!("u_borders[{:?}].color", i), color.to_linear())?;
        Ok(())
    }
}