use anyhow::{anyhow, Error};
use bytemuck::cast_slice;
use js_sys::WebAssembly;
use math::{Vector2, Vector3};
use wasm_bindgen::JsCast;
use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext, HtmlCanvasElement};


pub struct WebGl {
    pub gl: WebGlRenderingContext,
}

impl WebGl {
    pub fn try_new(canvas: &HtmlCanvasElement) -> Result<WebGl, Error> {
        let gl = super::util::try_get_webgl_context(&canvas)?;
        Ok(WebGl { gl })
    }

    pub fn clear(&self, r: f32, g: f32, b: f32, a: f32) {
        self.gl.clear_color(r, g, b, a);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    }

    pub fn draw_line(
        &self,
        program: &WebGlProgram,
        vertex_attribute_name: &str,
        buffer: &Buffer,
    ) -> Result<(), Error> {
        self.gl.use_program(Some(program));

        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer.buffer));
        let location = self.gl.get_attrib_location(program, vertex_attribute_name) as u32;
        self.gl.vertex_attrib_pointer_with_i32(
            location,
            buffer.element_size as i32,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        self.gl.enable_vertex_attrib_array(location);
        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);

        self.gl.draw_arrays(
            WebGlRenderingContext::LINES,
            0,
            (buffer.len / buffer.element_size) as i32,
        );

        Ok(())
    }

    pub fn draw_mesh(
        &self,
        program: &WebGlProgram,
        vertex_attribute_name: &str,
        buffer: &Buffer,
    ) -> Result<(), Error> {
        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer.buffer));

        // Upload to GPU
        self.gl.use_program(Some(program));
        let location = self.gl.get_attrib_location(program, vertex_attribute_name) as u32;
        self.gl.vertex_attrib_pointer_with_i32(
            location,
            buffer.element_size as i32,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        self.gl.enable_vertex_attrib_array(location);
        self.gl.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            (buffer.len / buffer.element_size) as i32,
        );
        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);
        Ok(())
    }

    pub fn new_array_buffer(&self, values: &[Vector3]) -> Result<Buffer, Error> {
        let bytes: &[f32] = cast_slice(values);
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .map_err(|_| anyhow!("could not get web assembly memory"))?
            .buffer();
        // Divide by 4 to get an index to individual f32 elements, because an f32
        // is 4 bytes long.
        let arr_location = bytes.as_ptr() as u32 / 4;
        let js_array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(arr_location, arr_location + bytes.len() as u32);

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
        Ok(Buffer {
            buffer,
            len: bytes.len() as u32,
            element_size: 3,
        })
    }

    pub fn set_uniform_vec2(
        &self,
        program: &WebGlProgram,
        field: &str,
        value: Vector2,
    ) -> Result<(), Error> {
        self.gl.use_program(Some(program));
        let location = self
            .gl
            .get_uniform_location(program, field)
            .ok_or_else(|| anyhow::anyhow!("could not get location for uniform {}", field))?;
        let value: [f32; 2] = value.into();
        self.gl.uniform2fv_with_f32_array(Some(&location), &value);
        Ok(())
    }

    pub fn try_create_shader_program(&self, vertex_shader_src: &str, fragment_shader_src: &str) -> Result<WebGlProgram, Error> {
        super::util::try_create_shader_program(&self.gl, vertex_shader_src, fragment_shader_src)
    }
}

/// Represents an arbitrary amount of data held in the GPU.
#[derive(Clone, Debug)]
pub struct Buffer {
    /// The [WebGlBuffer] that holds the data.
    pub buffer: WebGlBuffer,
    /// The number of elements in the buffer.
    pub len: u32,
    /// The number of elements per type. For example, Vector2 is size 2.
    pub element_size: u32,
}
