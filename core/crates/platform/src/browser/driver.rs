use anyhow::Error;
use layout::BoxConstraints;
use std::rc::Rc;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::HtmlCanvasElement;

use super::shaders::ShaderLibrary;
use super::util::try_get_canvas;
use super::WebGl;
use crate::AppDriver;
use layout::{Color, LayoutBox, LayoutTree, Material};
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

    pub fn clear(&self, color: Color) {
        self.gl.clear(color.r, color.g, color.b, color.a);
    }

    pub fn try_tick(&mut self, time: f32) -> Result<(), Error> {
        self.clear(Color::black());

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

        if time % 5000.0 < 50.0 {
            //super::util::log(&format!("{:#?}", tree));
        }

        for (_, child, offset) in tree.iter() {
            let min = child.bounds.min + offset + child.margin.min();
            let max = child.bounds.max + offset - child.margin.max();
            let rect = Rect::new(min, max);
            self.draw_rect(rect, child.material)?;
        }
        Ok(())
    }

    pub fn draw_rect(&mut self, rect: Rect, material: Option<Material>) -> Result<(), Error> {
        match material {
            Some(material) => self.shaders.standard.paint_rect(rect, material),
            None => Ok(()),
        }
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
