use math::Vector2D;
use render::mesh::Transform;
use render::webgl::*;
use render::Painter;
use wasm_bindgen::prelude::wasm_bindgen;

mod app;
mod input;

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

/// This is the main entrypoint to the application. This is called from the
/// browser to construct a driver instance that is then used to pipe through
/// browser events.
#[wasm_bindgen]
pub fn create(canvas_id: &str) -> WebDriver {
    // Forward panic messages to console.error
    #[cfg(feature = "console_error_panic_hook")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    WebDriver::try_new(canvas_id)
}

#[wasm_bindgen]
pub struct WebDriver {
    app: app::App,
}

/// The [WebDriver] exposes a set of methods that are called by the browser.
/// These methods are called by the browser and is used to convert browser
/// events (e.g. resizing, keyboard input) into something we can understand.
#[wasm_bindgen]
impl WebDriver {
    pub fn on_mouse_move(&mut self, x: f32, y: f32) {
        self.app.input.on_mouse_move(x, y);
    }

    pub fn on_key_down(&mut self, key: input::Key) {
        self.app.input.on_key_down(key);
    }

    pub fn on_key_up(&mut self, key: input::Key) {
        self.app.input.on_key_up(key);
    }

    pub fn tick(&mut self, time: f32) {
        if let Err(err) = self.try_update(time) {
            error(&format!("encounted error when rendering frame: {}", err));
        }
    }
}

impl WebDriver {
    pub fn try_new(canvas_id: &str) -> Self {
        let canvas = try_get_canvas(canvas_id).unwrap();
        let gl = try_get_webgl_context(&canvas).unwrap();
        let program = try_create_shader_program(&gl).unwrap();

        let width = canvas.client_width() as f32;
        let height = canvas.client_height() as f32;
        let painter = WebGlPainter::new(
            canvas,
            gl,
            program,
            Vector2D::new(width, height),
            Default::default(),
        );
        let mut app = app::App::new(painter);
        app.start();

        Self { app }
    }

    pub fn try_update(&mut self, time: f32) -> Result<(), anyhow::Error> {
        self.app.time.set_time(time);
        self.app.painter.start_frame()?;

        self.app.update()?;

        self.app.painter.end_frame()?;
        Ok(())
    }
}
