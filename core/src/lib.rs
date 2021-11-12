use wasm_bindgen::prelude::wasm_bindgen;
use render::browser::{BrowserDriver, WebGl};
use render::AppDriver;
use math::Vector2;
use layout::{Layout, LayoutTree, Positioned, Container, LayoutBox};
use anyhow::Error;

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

pub struct App {
    tree: LayoutTree,
}

impl App {
    pub fn new() -> App {
        let mut tree = LayoutTree::new();

        let widgets = Box::new(Positioned {
            position: (30.0, 30.0).into(),
            child: Box::new(Container {
                size: (100.0, 100.0).into(),
            })
        });

        let root = widgets.layout(&mut tree);
        let root_lbox = LayoutBox::from_child(root, Vector2::zero());
        let root_id = tree.insert(root_lbox);
        tree.set_root(Some(root_id));

        App { tree }
    }
}

impl AppDriver for App {
    fn tick(&mut self, time: f32) -> Box<dyn Layout> {
        let x = 100.0 + 100.0 * (time / 200.0).sin();
        let y = 100.0 + 100.0 * (time / 200.0).cos();
        Box::new(Positioned {
            position: (x, y).into(),
            child: Box::new(Container {
                size: (100.0, 100.0).into(),
            })
        })
    }
}