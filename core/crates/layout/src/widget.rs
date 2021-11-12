use std::fmt::Debug;
use math::{Vector2, Vector4};
use super::{Layout, LayoutTree, SizedLayoutBox, LayoutBox, RenderBox};

#[derive(Debug)]
pub struct Positioned {
    pub position: Vector2,
    pub child: Box<dyn Layout>,
}

impl Layout for Positioned {
    fn layout(&self, tree: &mut LayoutTree) -> SizedLayoutBox {
        let child = self.child.layout(tree);
        let size = self.position + child.size;

        let child_lbox = LayoutBox::from_child(child, self.position);
        let child_id = tree.insert(child_lbox);
        SizedLayoutBox {
            size,
            children: vec![child_id],
            content: RenderBox {
                material: Material::Solid(Color::green()),
            },
        }
    }
}

#[derive(Debug)]
pub struct Container {
    pub size: Vector2,
}

impl Layout for Container {
    fn layout(&self, _: &mut LayoutTree) -> SizedLayoutBox {
        SizedLayoutBox {
            size: self.size,
            children: vec![],
            content: RenderBox {
                material: Material::Solid(Color::blue()),
            }
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Material {
    None,
    Solid(Color)
}

/// A color stored as RGBA components, each ranging from 0 - 255.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color{ r, g, b, a }
    }

    pub fn transparent() -> Color {
        Color::rgba(0.0, 0.0, 0.0, 0.0)
    }

    pub fn red() -> Color {
        Color::rgba(255.0, 0.0, 0.0, 255.0)
    }

    pub fn green() -> Color {
        Color::rgba(0.0, 255.0, 0.0, 255.0)
    }

    pub fn blue() -> Color {
        Color::rgba(0.0, 0.0, 255.0, 255.0)
    }

    pub fn white() -> Color {
        Color::rgba(255.0, 255.0, 255.0, 255.0)
    }

    pub fn to_linear(&self) -> Vector4 {
        let r = self.r / 255.0;
        let g = self.g / 255.0;
        let b = self.b / 255.0;
        let a = self.a / 255.0;
        Vector4::new(r, g, b, a)
    }
}