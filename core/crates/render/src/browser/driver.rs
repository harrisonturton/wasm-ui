use anyhow::{Error, anyhow};
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::HtmlCanvasElement;
use std::rc::Rc;
use std::collections::VecDeque;

use crate::AppDriver;
use super::WebGl;
use super::shaders::{ShaderLibrary, MeshPainter};
use math::{Rect, Vector2, Vector3};
use layout::{LayoutTree, Color};

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
        self.clear(Color::white());
        Ok(())
    }

    pub fn clear(&self, color: Color) {
        self.gl.clear(color.r, color.g, color.b, color.a);
    }

    pub fn try_tick(&mut self, time: f32) -> Result<(), Error> {
        self.app.tick(time)?;
        self.clear(Color::white());

        let width = self.canvas.client_width() as f32;
        let height = self.canvas.client_height() as f32;
        let viewport = Vector2::new(width, height);
        self.shaders.standard.set_viewport(viewport);

        // Rotating line in center
        let speed = 0.003;
        let x = (time * speed).sin();
        let y = (time * speed).cos();
        let center = Vector2::new(width / 2.0, height / 2.0);
        self.draw_line(center, center + Vector2::new(x, y) * 300.0, Color::blue())?;

        // Walk the tree and draw each LayoutBox as a rect
        /*let mut tree = LayoutTree::new();
        let widgets: Box<dyn layout::Layout> = Box::new(layout::Container {
            size: (200.0 * (time / 200.0).sin(), 200.0 * (time / 200.0).sin()).into(),
        });
        let root = widgets.layout(&mut tree);
        let root_lbox = layout::LayoutBox::from_child(root, Vector2::zero());
        let root_id = tree.insert(root_lbox);
        tree.set_root(Some(root_id));*/

        let tree = self.app.tick(time)?;

        let root = match tree.root {
            Some(id) => id,
            None => return Ok(()),
        };

        let mut remaining = VecDeque::new();
        let mut parent_offsets = VecDeque::new();
        remaining.push_front(root);
        parent_offsets.push_front(Vector2::zero());
        while let Some(lbox_id) = remaining.pop_front() {
            let offset = parent_offsets.pop_front().unwrap();
            let lbox = tree.get(lbox_id).unwrap();

            let min = lbox.rect.min + offset;
            let max = lbox.rect.max + offset;
            let rect = Rect::new(min, max);
            let color = match lbox.content.material {
                layout::Material::Solid(color) => color,
                layout::Material::None => Color::transparent(),
            };
            self.draw_rect(rect, color)?;
            for child in &lbox.children {
                remaining.push_front(*child);
                parent_offsets.push_front(min);
            }
        }

        // Rectangle
        //let origin = Vector2::new(100.0, 50.0);
        //let end = Vector2::new(200.0, 500.0);
        //let rect = Rect::new(origin, end);
        //self.draw_rect(rect, Color::red())?;

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
            Vector3::new(end.x, end.y, 0.0)
        ];
        self.shaders.standard.set_color(color.to_linear());
        self.shaders.standard.paint_line(&vertices)?;
        Ok(())
    }
}
