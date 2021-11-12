use wasm_bindgen::prelude::wasm_bindgen;
use platform::browser::BrowserDriver;

mod app;
use app::App;

// Use `wee_alloc` as the global allocator, because it is smaller.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// This is called from the browser as soon as the WASM package is loaded. It is
/// the main entrypoint to the application. This is similar to `index.js` in
/// React.
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> BrowserDriver {
    // Forward panic messages to console.error
    #[cfg(feature = "console_error_panic_hook")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let app = App::new();
    BrowserDriver::try_new(canvas_id, Box::new(app)).unwrap()
}