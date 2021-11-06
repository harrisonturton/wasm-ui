use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::WebGlBuffer;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

#[wasm_bindgen]
pub struct App {
    gl: WebGlRenderingContext,
    pos_memory_buffer: JsValue,
    index_buffer: WebGlBuffer,

    mouse_pos: (f32, f32),
    prev_timestamp: f32,
}

#[wasm_bindgen]
impl App {
    pub fn new(gl: WebGlRenderingContext) -> Self {
        let index_buffer = gl.create_buffer().unwrap();
        let pos_memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        Self {
            gl,
            mouse_pos: (0.0, 0.0),
            prev_timestamp: 0.0,
            pos_memory_buffer,
            index_buffer,
        }
    }

    pub fn init(&self) {
        let vert_shader = compile_shader(
            &self.gl,
            WebGlRenderingContext::VERTEX_SHADER,
            include_str!("../shaders/webgl/vertex.glsl"),
        )
        .expect("Could not compile vert shader");
        let frag_shader = compile_shader(
            &self.gl,
            WebGlRenderingContext::FRAGMENT_SHADER,
            include_str!("../shaders/webgl/fragment.glsl"),
        )
        .expect("could not compile frag shader");
        let program =
            link_program(&self.gl, &vert_shader, &frag_shader).expect("could not link program");
        self.gl.use_program(Some(&program));
    }

    pub fn tick(&mut self, timestamp: f32) {
        self.gl.clear_color(1.0, 1.0, 1.0, 1.0);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        let speed = 0.0001;
        let vertices: [f32; 9] = [
            -0.7 + timestamp * speed,
            -0.7 + timestamp * speed,
            0.0,
            0.7 + timestamp * speed,
            -0.7,
            0.0,
            0.0,
            0.7,
            0.0,
        ];

        let pos_ptr = vertices.as_ptr() as u32 / 4;
        let pos_array = js_sys::Float32Array::new(&self.pos_memory_buffer)
            .subarray(pos_ptr, pos_ptr + vertices.len() as u32);

        self.gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.index_buffer),
        );
        self.gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &pos_array,
            WebGlRenderingContext::STREAM_DRAW,
        );

        self.gl
            .vertex_attrib_pointer_with_i32(0, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
        self.gl.enable_vertex_attrib_array(0);

        self.gl.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            (vertices.len() / 3) as i32,
        );

        self.prev_timestamp = timestamp;
    }

    pub fn update_mouse_pos(&mut self, mouse_x: f32, mouse_y: f32) {
        self.mouse_pos = (mouse_x, mouse_y);
    }
}

fn link_program(
    gl: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, anyhow::Error> {
    let program = gl
        .create_program()
        .ok_or_else(|| anyhow::anyhow!("could not create program"))?;

    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        match gl.get_program_info_log(&program) {
            Some(log) => Err(anyhow::anyhow!(log)),
            None => Err(anyhow::anyhow!("unknown error when creating program")),
        }
    }
}

fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, anyhow::Error> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| anyhow::anyhow!("unable to create shader object"))?;

    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        match gl.get_shader_info_log(&shader) {
            Some(log) => Err(anyhow::anyhow!(log)),
            None => Err(anyhow::anyhow!("unknown error when creating shader object")),
        }
    }
}
