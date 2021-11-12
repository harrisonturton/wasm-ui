use wasm_bindgen::prelude::wasm_bindgen;
use math::Vector2;
use render::browser::BrowserDriver;
use render::AppDriver;
use layout::{Layout, Positioned, Container};

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

    let backend = BrowserDriver::try_new(canvas_id, Box::new(app)).unwrap();
    backend
}

pub struct App {
    position: Vector2,
}

impl App {
    pub fn new() -> App {
        let position = Vector2::zero();
        App { position }
    }
}

impl AppDriver for App {
    fn tick(&mut self, time: f32) -> Box<dyn Layout> {
        let speed = 0.005;
        let radius = 100.0;
        let offset = Vector2::new(100.0, 100.0);
        self.position.x = 100.0 + radius * (time * speed).sin();
        self.position.y = 100.0 + radius * (time * speed).cos();
        self.position += offset;

        Box::new(Positioned {
            position: self.position,
            child: Box::new(Container {
                size: (100.0, 100.0).into(),
            })
        })
    }
}