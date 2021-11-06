use js_sys::WebAssembly;
use wasm_bindgen::JsCast;
use web_sys::WebGlBuffer;
use web_sys::WebGlProgram;
use web_sys::WebGlShader;
use web_sys::{HtmlCanvasElement, WebGlRenderingContext};

use crate::mesh::*;
use math::{Mat4, Vector2D, Vector3D, Vector4D};

pub type ObjectRef = usize;

pub trait Painter {
    fn start_frame(&mut self) -> Result<(), anyhow::Error>;
    fn load(&mut self, object: &Object) -> Result<ObjectRef, anyhow::Error>;
    fn paint(&mut self, object: ObjectRef) -> Result<(), anyhow::Error>;
    fn paint_line(&mut self, from: Vector3D, to: Vector3D) -> Result<(), anyhow::Error>;
    fn end_frame(&mut self) -> Result<(), anyhow::Error>;
}

pub struct WebGlPainter {
    canvas: Box<HtmlCanvasElement>,
    gl: Box<WebGlRenderingContext>,
    program: Box<WebGlProgram>,
    objects: Vec<(WebGlBuffer, usize, MatrixStack)>,
    pub dimensions: Vector2D,
    pub camera: Mat4,
}

impl WebGlPainter {
    pub fn new(
        canvas: HtmlCanvasElement,
        gl: WebGlRenderingContext,
        program: WebGlProgram,
        dimensions: Vector2D,
        camera: Mat4,
    ) -> Self {
        Self {
            canvas: Box::new(canvas),
            gl: Box::new(gl),
            program: Box::new(program),
            objects: Vec::new(),
            dimensions,
            camera,
        }
    }
}

impl WebGlPainter {
    pub fn set_camera(&mut self, camera: Mat4) {
        self.camera = camera;
    }

    pub fn paint_simple(&mut self, object_ref: ObjectRef) -> Result<(), anyhow::Error> {
        self.gl.use_program(Some(&self.program));

        let (buffer, buffer_len, transform) = self
            .objects
            .get(object_ref)
            .ok_or_else(|| anyhow::anyhow!("object must be loaded before it can be painted"))?;

        set_uniform_vec4(&self.gl, &self.program, "u_color", crate::Color::rgb(255.0, 248.0, 210.0).into_linear())?;

        let m_model = transform.product();
        //math::Mat4::new_unit()
        //    .scale(transform.scale)
        //    .rotate(transform.rotation)
        //    .translate(transform.position);

        //let aspect = self.dimensions.x / self.dimensions.y;
        //let fov: f32 = 50.0;
        //let m_projection = crate::projection::perspective2(fov, aspect, 1.0, 200.0);
        //set_uniform_mat4(&self.gl, &self.program, "m_projection", m_projection)?;

        let m_view = self.camera;
        let m_model_view = m_model;
        set_uniform_mat4(&self.gl, &self.program, "m_model_view", m_model_view)?;

        draw_triangles(&self.gl, &self.program, &buffer, buffer_len);
        Ok(())
    }
}

impl Painter for WebGlPainter {
    fn start_frame(&mut self) -> Result<(), anyhow::Error> {
        let (r, g, b, a) = crate::Color::rgb(31.0, 38.0, 72.0).into_linear().into();
        self.gl.clear_color(r, g, b, a);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        Ok(())
    }

    // Need to convert vertices into just floats
    fn load(&mut self, object: &Object) -> Result<ObjectRef, anyhow::Error> {
        let vertices: &[f32] = bytemuck::cast_slice(object.vertices);
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let arr_location = vertices.as_ptr() as u32 / 4;
        let js_array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(arr_location, arr_location + vertices.len() as u32);

        let buffer = self
            .gl
            .create_buffer()
            .ok_or_else(|| anyhow::anyhow!("could not create buffer"))?;
        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        self.gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);

        self.objects
            .push((buffer, vertices.len(), object.transform.clone()));
        Ok(self.objects.len() - 1)
    }

    fn paint(&mut self, object_ref: ObjectRef) -> Result<(), anyhow::Error> {
        self.gl.use_program(Some(&self.program));

        let (buffer, buffer_len, transform) = self
            .objects
            .get(object_ref)
            .ok_or_else(|| anyhow::anyhow!("object must be loaded before it can be painted"))?;

        set_uniform_vec4(&self.gl, &self.program, "u_color", crate::Color::rgb(255.0, 248.0, 210.0).into_linear())?;

        let m_model = transform.product();
        //math::Mat4::new_unit()
        //    .scale(transform.scale)
        //    .rotate(transform.rotation)
        //    .translate(transform.position);

        let aspect = self.dimensions.x / self.dimensions.y;
        let fov: f32 = 50.0;
        let m_projection = crate::projection::perspective2(fov, aspect, 1.0, 200.0);
        //set_uniform_mat4(&self.gl, &self.program, "m_projection", m_projection)?;

        let m_view = self.camera;
        let m_model_view = m_model * m_view * m_projection;
        set_uniform_mat4(&self.gl, &self.program, "m_model_view", m_model_view)?;

        draw_triangles(&self.gl, &self.program, &buffer, buffer_len);
        Ok(())
    }

    fn paint_line(&mut self, from: Vector3D, to: Vector3D) -> Result<(), anyhow::Error> {
        self.gl.use_program(Some(&self.program));

        let vertices = [Vertex { position: from }, Vertex { position: to }];
        let vertices: &[f32] = bytemuck::cast_slice(&vertices);
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let arr_location = vertices.as_ptr() as u32 / 4;
        let js_array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(arr_location, arr_location + vertices.len() as u32);

        let buffer = self
            .gl
            .create_buffer()
            .ok_or_else(|| anyhow::anyhow!("could not create buffer"))?;
        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        self.gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);

        set_uniform_vec4(&self.gl, &self.program, "u_color", crate::Color::rgb(255.0, 248.0, 210.0).into_linear())?;

        let buffer = buffer;
        let buffer_len = vertices.len();

        let aspect = self.dimensions.x / self.dimensions.y;
        let fov: f32 = 50.0;
        let m_projection = crate::projection::perspective2(fov, aspect, 1.0, 200.0);
        //set_uniform_mat4(&self.gl, &self.program, "m_projection", m_projection)?;

        let m_view = self.camera;
        let m_model_view = m_view * m_projection;
        set_uniform_mat4(&self.gl, &self.program, "m_model_view", m_model_view)?;

        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        let coord: u32 = self.gl.get_attrib_location(&self.program, "position") as u32;
        self.gl
            .vertex_attrib_pointer_with_i32(coord, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
        self.gl.enable_vertex_attrib_array(coord);
        self.gl
            .bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);

        self.gl
            .draw_arrays(WebGlRenderingContext::LINES, 0, (buffer_len / 3) as i32);

        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), anyhow::Error> {
        self.gl.viewport(
            0,
            0,
            self.canvas.width() as i32,
            self.canvas.height() as i32,
        );
        self.gl.flush();
        Ok(())
    }
}

pub fn draw_triangles(
    gl: &WebGlRenderingContext,
    program: &WebGlProgram,
    buffer: &WebGlBuffer,
    buffer_len: &usize,
) {
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(buffer));
    let coord: u32 = gl.get_attrib_location(&program, "position") as u32;
    gl.vertex_attrib_pointer_with_i32(coord, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(coord);
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);
    gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, (buffer_len / 3) as i32);
}

pub fn draw_line(
    gl: &WebGlRenderingContext,
    program: &WebGlProgram,
    buffer: &WebGlBuffer,
    buffer_len: &usize,
) {
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
    let coord: u32 = gl.get_attrib_location(&program, "position") as u32;
    gl.vertex_attrib_pointer_with_i32(coord, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(coord);
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);
    gl.draw_arrays(WebGlRenderingContext::LINES, 0, (buffer_len / 3) as i32);
}

pub fn set_uniform_vec4<I>(
    gl: &WebGlRenderingContext,
    program: &WebGlProgram,
    name: &str,
    value: I,
) -> Result<(), anyhow::Error>
where
    I: Into<[f32; 4]>,
{
    let location = gl
        .get_uniform_location(program, name)
        .ok_or_else(|| anyhow::anyhow!("could not get location for uniform {}", name))?;
    let value: [f32; 4] = value.into();
    gl.uniform4fv_with_f32_array(Some(&location), &value);
    Ok(())
}

pub fn set_uniform_mat4<I>(
    gl: &WebGlRenderingContext,
    program: &WebGlProgram,
    name: &str,
    value: I,
) -> Result<(), anyhow::Error>
where
    I: Into<[f32; 16]>,
{
    let location = gl
        .get_uniform_location(program, name)
        .ok_or_else(|| anyhow::anyhow!("could not get location for uniform {}", name))?;
    let value: [f32; 16] = value.into();
    gl.uniform_matrix4fv_with_f32_array(Some(&location), false, &value);
    Ok(())
}

// Try to get a [HtmlCanvasElement] from the document by it's element ID.
pub fn try_get_canvas(id: &str) -> Result<HtmlCanvasElement, anyhow::Error> {
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let canvas = document
        .get_element_by_id(id)
        .ok_or_else(|| anyhow::anyhow!("failed to get canvas"))?
        .dyn_into::<HtmlCanvasElement>()
        .expect("could not get canvas");
    Ok(canvas)
}

/// Try to get a `webgl` context from a [HtmlCanvasElement].
pub fn try_get_webgl_context(
    canvas: &HtmlCanvasElement,
) -> Result<WebGlRenderingContext, anyhow::Error> {
    let gl = canvas
        .get_context("webgl")
        .map_err(|_| anyhow::anyhow!("could not get webgl context"))?
        .ok_or_else(|| anyhow::anyhow!("could not get webgl context"))?
        .dyn_into::<WebGlRenderingContext>()
        .map_err(|_| anyhow::anyhow!("could not get webgl context"))?;
    Ok(gl)
}



/// This will try to create a shader program that uses the vertex and fragment
/// shaders defined in:
///
/// * `render/shaders/vertex.glsl`
/// * `render/shaders/fragment.glsl`
///
/// respectively.
pub fn try_create_shader_program(
    gl: &WebGlRenderingContext,
) -> Result<WebGlProgram, anyhow::Error> {
    let vertex_shader = compile_shader(
        gl,
        WebGlRenderingContext::VERTEX_SHADER,
        include_str!("../shaders/webgl/vertex.glsl"),
    )?;
    let fragment_shader = compile_shader(
        gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        include_str!("../shaders/webgl/fragment.glsl"),
    )?;
    link_program(gl, &vertex_shader, &fragment_shader)
}

fn link_program(
    gl: &WebGlRenderingContext,
    vertex_shader: &WebGlShader,
    fragment_shader: &WebGlShader,
) -> Result<WebGlProgram, anyhow::Error> {
    let program = gl
        .create_program()
        .ok_or_else(|| anyhow::anyhow!("could not create program"))?;

    gl.attach_shader(&program, vertex_shader);
    gl.attach_shader(&program, fragment_shader);
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

/*pub struct Object<'a> {
    mesh: &'a [f32],
}

impl<'a> Object<'a> {
    pub fn from_vertices(mesh: &'a [f32]) -> Self {
        Self { mesh }
    }

    pub fn draw(&self, gl: &WebGlRenderingContext, program: &WebGlProgram) -> Result<(), anyhow::Error> {
        let buffer = Buffer::try_from_array(gl, self.mesh)?;
        buffer.associate_with_attrib(gl, program, "position");
        buffer.draw(gl);
        Ok(())
    }
}*/

pub struct Buffer {
    length: usize,
    buffer: WebGlBuffer,
}

impl Buffer {
    pub fn try_from_array(gl: &WebGlRenderingContext, arr: &[f32]) -> Result<Self, anyhow::Error> {
        let memory_buffer = wasm_bindgen::memory()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
            .buffer();
        let arr_location = arr.as_ptr() as u32 / 4;
        let f32_array = js_sys::Float32Array::new(&memory_buffer)
            .subarray(arr_location, arr_location + arr.len() as u32);

        let buffer = gl
            .create_buffer()
            .ok_or_else(|| anyhow::anyhow!("could not create buffer"))?;
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &f32_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);

        Ok(Self {
            buffer,
            length: arr.len(),
        })
    }

    pub fn associate_with_attrib(
        &self,
        gl: &WebGlRenderingContext,
        program: &WebGlProgram,
        attrib_name: &str,
    ) {
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.buffer));
        let coord: u32 = gl.get_attrib_location(program, attrib_name) as u32;
        gl.vertex_attrib_pointer_with_i32(coord, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(coord);
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);
    }

    pub fn draw(&self, gl: &WebGlRenderingContext) {
        gl.draw_arrays(
            WebGlRenderingContext::TRIANGLES,
            0,
            (self.length / 3) as i32,
        );
    }
}
