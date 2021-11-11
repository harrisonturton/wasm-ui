use wasm_bindgen::prelude::wasm_bindgen;
use render::browser::{BrowserDriver, WebGl};
use render::AppDriver;

// Use `wee_alloc` as the global allocator, because it is smaller.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // `console.log` in javascript
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    // `console.error` in javascript
    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);
}

/// This is the entrypoint to the application. This is called from the browser.
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> BrowserDriver {
    // Forward panic messages to console.error
    #[cfg(feature = "console_error_panic_hook")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let app = App::new();

    // TODO internal world representation that can then
    // be passed to the BrowserDriver to render to the webgl canvas

    let canvas = render::browser::util::try_get_canvas(canvas_id).unwrap();
    let gl = WebGl::try_new(&canvas).unwrap();
    let backend = BrowserDriver::try_new(canvas, gl, Box::new(app)).unwrap();
    backend.render().unwrap();
    backend
}

// AppDriver -> Implemented by app so it can be driven
// RenderDriver -> Implemented by WebGl renderer so it can render common world representation

pub struct App {}

impl App {
    pub fn new() -> App {
        App {}
    }
}

impl AppDriver for App {
    fn tick(&mut self, _time: f32) -> Result<(), anyhow::Error> {
        Ok(())
    }
}