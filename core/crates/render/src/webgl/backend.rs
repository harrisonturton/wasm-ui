use crate::geometry::Vertex;
use crate::webgl::shaders::ShaderLibrary;
use crate::webgl::util;
use crate::webgl::WebGl;
use anyhow::Error;
use bytemuck::cast_slice;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};

use math::{Vector2, Vector3, Rect};

pub trait Backend {
    fn render(&self) -> Result<(), Error>;
}

#[wasm_bindgen]
extern "C" {
    // `console.log` in javascript
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    // `console.error` in javascript
    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);
}

#[wasm_bindgen]
pub struct WebGlBackend {
    canvas: HtmlCanvasElement,
    gl: WebGl,
    shaders: ShaderLibrary,
}

#[wasm_bindgen]
impl WebGlBackend {
    pub fn tick(&self, time: f32) {
        match self.try_tick(time) {
            Ok(()) => (),
            Err(_) => error("failed to tick"),
        };
    }
}

impl Backend for WebGlBackend {
    fn render(&self) -> Result<(), Error> {
        self.clear();
        Ok(())
    }
}

impl WebGlBackend {
    pub fn try_new(canvas_id: &str) -> Result<WebGlBackend, Error> {
        let canvas = util::try_get_canvas(canvas_id)?;
        let gl = util::try_get_webgl_context(&canvas)?;
        let shaders = ShaderLibrary::try_new(&gl)?;
        let context = WebGl::new(gl);
        Ok(WebGlBackend {
            canvas,
            shaders,
            gl: context,
        })
    }

    pub fn clear(&self) {
        self.gl.clear(0.0, 0.0, 0.0, 1.0);
    }

    pub fn try_tick(&self, time: f32) -> Result<(), Error> {
        self.clear();

        let width = self.canvas.client_width() as f32;
        let height = self.canvas.client_height() as f32;
        let viewport = Vector2::new(width, height);
        self.gl.set_uniform_vec2(&self.shaders.standard, "u_viewport", viewport)?;

        // Rotating line in center
        let speed = 0.003;
        let x = (time * speed).sin();
        let y = (time * speed).cos();
        let center = Vector2::new(width / 2.0, height / 2.0);
        self.draw_line(center, center + Vector2::new(x, y) * 100.0)?;

        // Rectangle
        let origin = Vector2::new(100.0, 50.0);
        let end = Vector2::new(200.0, 500.0);
        let rect = Rect::new(origin, end);
        self.draw_rect(rect)?;

        Ok(())
    }

    pub fn draw_rect(&self, rect: Rect) -> Result<(), Error> {
        self.gl.use_shader(&self.shaders.standard);

        let (min_x, min_y) = rect.min.into();
        let (max_x, max_y) = rect.max.into();
        let vertices: [Vector3; 6] = [
            Vector3::new(min_x, min_y, 0.0),
            Vector3::new(min_x, max_y, 0.0),
            Vector3::new(max_x, min_y, 0.0),
            Vector3::new(min_x, max_y, 0.0),
            Vector3::new(max_x, min_y, 0.0),
            Vector3::new(max_x, max_y, 0.0),
        ];

        let vertices: &[f32] = cast_slice(&vertices);
        let buffer = self.gl.new_array_buffer(vertices)?;
        self.gl
            .gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        let coord: u32 =
            self.gl
                .gl
                .get_attrib_location(&self.shaders.standard, "a_position") as u32;
        self.gl.gl.vertex_attrib_pointer_with_i32(
            coord,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        self.gl.gl.enable_vertex_attrib_array(coord);
        self.gl
            .gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);

        self.gl
            .gl
            .draw_arrays(WebGlRenderingContext::TRIANGLES, 0, (vertices.len() / 3) as i32);

        Ok(())
    }

    pub fn draw_line(&self, start: Vector2, end: Vector2) -> Result<(), Error> {
        self.gl.use_shader(&self.shaders.standard);

        let start: Vector3 = start.into();
        let end: Vector3 = end.into();
        let vertices = [start, end];

        let vertices: &[f32] = cast_slice(&vertices);
        let buffer = self.gl.new_array_buffer(vertices)?;
        self.gl
            .gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        let coord: u32 =
            self.gl
                .gl
                .get_attrib_location(&self.shaders.standard, "a_position") as u32;
        self.gl.gl.vertex_attrib_pointer_with_i32(
            coord,
            3,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        self.gl.gl.enable_vertex_attrib_array(coord);
        self.gl
            .gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);

        self.gl
            .gl
            .draw_arrays(WebGlRenderingContext::LINES, 0, (vertices.len() / 3) as i32);

        Ok(())
    }
}
