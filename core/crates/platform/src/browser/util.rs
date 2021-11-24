use anyhow::{anyhow, Error};
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, WebGlProgram, WebGlRenderingContext, WebGlShader};

#[wasm_bindgen]
extern "C" {
    // `console.log` in javascript
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    // `console.error` in javascript
    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);
}

// The JS object passsed into the [get_context_with_context_options]
// WebGlRenderingContext constructor. For some reason cargo sees this as dead
// code.
#[wasm_bindgen]
struct WebGlOptions {
    #[allow(dead_code)]
    premultiplied_alpha: bool,
}

/// Try to get a reference to the [WebGlCanvasElement] identified by the provided ID.
pub fn try_get_canvas(canvas_id: &str) -> Result<HtmlCanvasElement, Error> {
    let window = web_sys::window().ok_or_else(|| anyhow!("could not get window"))?;
    let document = window
        .document()
        .ok_or_else(|| anyhow!("could not get document"))?;
    document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| anyhow!("failed to get canvas"))?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| anyhow!("failed to get canvas"))
}

/// Try to get a [WebGlRenderingContext] from a reference to a [HtmlCanvasElement].
pub fn try_get_webgl_context(canvas: &HtmlCanvasElement) -> Result<WebGlRenderingContext, Error> {
    let options = WebGlOptions {
        // This is needed otherwise semi-transparent colors are assumed to have
        // transparency multiplied into their color, and are rendered weirdly.
        premultiplied_alpha: false,
    };
    canvas
        .get_context_with_context_options("webgl", &options.into())
        .map_err(|_| anyhow::anyhow!("could not get webgl context"))?
        .ok_or_else(|| anyhow::anyhow!("could not get webgl context"))?
        .dyn_into::<WebGlRenderingContext>()
        .map_err(|_| anyhow::anyhow!("could not get webgl context"))
}

/// This will try to create a shader program that uses the provided vertex and
/// fragment shader sourcecodes.
pub fn try_create_shader_program(
    gl: &WebGlRenderingContext,
    vertex_shader_src: &str,
    fragment_shader_src: &str,
) -> Result<WebGlProgram, Error> {
    crate::browser::util::log("before try compile vertex shader");
    let vertex_shader =
        try_compile_shader(gl, WebGlRenderingContext::VERTEX_SHADER, vertex_shader_src)?;
    crate::browser::util::log("after try compile vertex shader");
    let fragment_shader = try_compile_shader(
        gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        fragment_shader_src,
    )?;
    crate::browser::util::log("after try compile frag shader");
    try_link_program(gl, &vertex_shader, &fragment_shader)
}

/// Try to link the shader program. If linking fails, this will attempt to get
/// the error message from the program info log. If that also fails, it
/// will return a generic error.
pub fn try_link_program(
    gl: &WebGlRenderingContext,
    vertex_shader: &WebGlShader,
    fragment_shader: &WebGlShader,
) -> Result<WebGlProgram, Error> {
    let program = gl
        .create_program()
        .ok_or_else(|| anyhow!("could not create shader program"))?;
    gl.attach_shader(&program, vertex_shader);
    gl.attach_shader(&program, fragment_shader);
    gl.link_program(&program);

    let maybe_link_status = gl.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS);
    if !maybe_link_status.as_bool().unwrap_or(false) {
        return match gl.get_program_info_log(&program) {
            Some(log) => Err(anyhow!(log)),
            None => Err(anyhow!("unknown error occured when linking shader program")),
        };
    }
    Ok(program)
}

/// Attempt to compile the shader. If compilation fails, this will attempt to
/// get the error message from the shader info log. If that also fails, it will
/// return a generic error.
pub fn try_compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    src: &str,
) -> Result<WebGlShader, Error> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| anyhow!("failed to create shader"))?;
    gl.shader_source(&shader, src);
    gl.compile_shader(&shader);

    let maybe_compile_status =
        gl.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS);
    if !maybe_compile_status.as_bool().unwrap_or(false) {
        return match gl.get_shader_info_log(&shader) {
            Some(log) => Err(anyhow!(log)),
            None => Err(anyhow!("unknown error occured when compiling shader")),
        };
    }
    Ok(shader)
}
