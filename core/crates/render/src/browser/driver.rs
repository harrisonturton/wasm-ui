use anyhow::Error;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::HtmlCanvasElement;
use std::rc::Rc;

use crate::AppDriver;
use super::WebGl;
use super::shaders::{ShaderLibrary, MeshPainter};
use math::{Rect, Vector2, Vector3};

#[wasm_bindgen]
extern "C" {
    // `console.log` in javascript
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    // `console.error` in javascript
    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);
}

#[wasm_bindgen]
pub struct BrowserDriver {
    canvas: HtmlCanvasElement,
    gl: Rc<WebGl>,
    shaders: ShaderLibrary,
    app: Box<dyn AppDriver>,
}

#[wasm_bindgen]
impl BrowserDriver {
    pub fn tick(&mut self, time: f32) {
        self.try_tick(time).unwrap();
    }
}

impl BrowserDriver {
    pub fn try_new(canvas: HtmlCanvasElement, gl: WebGl, app: Box<dyn AppDriver>) -> Result<BrowserDriver, Error> {
        let gl = Rc::new(gl);
        let shaders = ShaderLibrary::try_new(&gl)?;
        Ok(BrowserDriver { canvas, shaders, gl, app })
    }

    pub fn render(&self) -> Result<(), Error> {
        self.clear();
        Ok(())
    }

    pub fn clear(&self) {
        self.gl.clear(0.0, 0.0, 0.0, 1.0);
    }

    pub fn try_tick(&mut self, time: f32) -> Result<(), Error> {
        self.app.tick(time)?;
        self.clear();

        let width = self.canvas.client_width() as f32;
        let height = self.canvas.client_height() as f32;
        let viewport = Vector2::new(width, height);
        self.shaders.standard.set_viewport(viewport);

        // Rotating line in center
        let speed = 0.003;
        let x = (time * speed).sin();
        let y = (time * speed).cos();
        let center = Vector2::new(width / 2.0, height / 2.0);
        self.draw_line(center, center + Vector2::new(x, y) * 300.0)?;

        // Rectangle
        let origin = Vector2::new(100.0, 50.0);
        let end = Vector2::new(200.0, 500.0);
        let rect = Rect::new(origin, end);
        self.draw_rect(rect)?;

        Ok(())
    }

    pub fn draw_rect(&self, rect: Rect) -> Result<(), Error> {
        let (min_x, min_y) = rect.min.into();
        let (max_x, max_y) = rect.max.into();
        let vertices: [Vector3; 6] = [
            Vector3::new(min_x, min_y, 0.0),
            Vector3::new(min_x, max_y, 0.0),
            Vector3::new(max_x, min_y, 0.0),
            Vector3::new(min_x, max_y, 0.0),
            Vector3::new(max_x, min_y, 0.0),
            Vector3::new(max_x, max_y, 0.0),
        ];
        self.shaders.standard.paint_mesh(&vertices)?;

        Ok(())
    }

    pub fn draw_line(&self, start: Vector2, end: Vector2) -> Result<(), Error> {
        let vertices = [
            Vector3::new(start.x, start.y, 0.0),
            Vector3::new(end.x, end.y, 0.0)
        ];
        self.shaders.standard.paint_line(&vertices)?;
        Ok(())
    }
}
