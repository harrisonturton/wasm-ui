use wasm_bindgen::prelude::wasm_bindgen;

use render::webgl::{Backend, WebGlBackend};
use math::Vector3;

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
pub fn start(canvas_id: &str) -> WebGlBackend {
    // Forward panic messages to console.error
    #[cfg(feature = "console_error_panic_hook")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let backend = WebGlBackend::try_new(canvas_id).unwrap();
    backend.render().expect("failed to render");

    backend
}