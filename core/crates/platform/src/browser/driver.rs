use anyhow::Error;
use layout::BoxConstraints;
use std::collections::VecDeque;
use std::rc::Rc;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::HtmlCanvasElement;

use super::shaders::{MeshPainter, ShaderLibrary};
use super::util::try_get_canvas;
use super::WebGl;
use crate::AppDriver;
use layout::{Color, LayoutBox, LayoutTree};
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
    pub fn try_new(canvas_id: &str, app: Box<dyn AppDriver>) -> Result<BrowserDriver, Error> {
        let canvas = try_get_canvas(canvas_id)?;
        let gl = WebGl::try_new(&canvas)?;
        let gl = Rc::new(gl);
        let shaders = ShaderLibrary::try_new(&gl)?;
        Ok(BrowserDriver {
            canvas,
            gl,
            shaders,
            app,
        })
    }

    pub fn render(&self) -> Result<(), Error> {
        self.clear(Color::white());
        Ok(())
    }

    pub fn clear(&self, color: Color) {
        self.gl.clear(color.r, color.g, color.b, color.a);
    }

    pub fn try_tick(&mut self, time: f32) -> Result<(), Error> {
        self.clear(Color::white());

        let width = self.canvas.client_width() as f32;
        let height = self.canvas.client_height() as f32;
        let viewport = Vector2::new(width, height);
        self.shaders.standard.set_viewport(viewport);

        self.paint(time, viewport)?;
        Ok(())
    }

    pub fn paint(&mut self, time: f32, viewport: Vector2) -> Result<(), Error> {
        let mut tree = LayoutTree::new();

        let widget_tree = self.app.tick(time);
        let constraints = BoxConstraints {
            min: Vector2::zero(),
            max: viewport,
        };
        let root_sbox = widget_tree.layout(&mut tree, &constraints);
        let root_lbox = LayoutBox::from_child(root_sbox, (0.0, 0.0));
        let root_id = tree.insert(root_lbox);
        tree.set_root(Some(root_id));

        for (parent, child, offset) in tree.iter() {
            let min = child.rect.min + offset;
            let max = child.rect.max + offset;
            let rect = Rect::new(min, max);
            let color = match child.material {
                layout::Material::Solid(color) => color,
                layout::Material::None => Color::transparent(),
            };
            self.draw_rect(rect, color)?;
        }
        Ok(())
    }

    pub fn draw_rect(&mut self, rect: Rect, color: Color) -> Result<(), Error> {
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
        self.shaders.standard.set_color(color.to_linear());
        self.shaders.standard.paint_mesh(&vertices)?;
        Ok(())
    }

    pub fn draw_line(&mut self, start: Vector2, end: Vector2, color: Color) -> Result<(), Error> {
        let vertices = [
            Vector3::new(start.x, start.y, 0.0),
            Vector3::new(end.x, end.y, 0.0),
        ];
        self.shaders.standard.set_color(color.to_linear());
        self.shaders.standard.paint_line(&vertices)?;
        Ok(())
    }
}
