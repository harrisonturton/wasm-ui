use anyhow::{anyhow, Error};
use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext};
use math::Vector2;

pub struct WebGl {
    pub gl: WebGlRenderingContext,
}

impl WebGl {
    pub fn new(gl: WebGlRenderingContext) -> WebGl {
        Self { gl }
    }

    pub fn use_shader(&self, shader: &WebGlProgram) {
        self.gl.use_program(Some(shader));
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        self.gl.clear_color(r, g, b, a);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    }

    pub fn set_uniform_vec2(&self, program: &WebGlProgram, field: &str, value: Vector2) -> Result<(), Error> {
        self.use_shader(program);
        let location = self.gl
            .get_uniform_location(program, field)
            .ok_or_else(|| anyhow::anyhow!("could not get location for uniform {}", field))?;
        let value: [f32; 2] = value.into();
        self.gl.uniform2fv_with_f32_array(Some(&location), &value);
        Ok(())
    }

    pub fn new_array_buffer(&self, values: &[f32]) -> Result<WebGlBuffer, Error> {
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .map_err(|_| anyhow!("could not get web assembly memory"))?
            .buffer();
        // Divide by 4 because an f32 is 4 bytes long
        let arr_location = values.as_ptr() as u32 / 4;
        let js_array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(arr_location, arr_location + values.len() as u32);

        let buffer = self
            .gl
            .create_buffer()
            .ok_or_else(|| anyhow!("could not create buffer"))?;
        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        self.gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);
        Ok(buffer)
    }
}
