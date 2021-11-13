use super::{BoxConstraints, Layout, LayoutBox, LayoutTree, SizedLayoutBox};
use math::{Vector2, Vector4};
use std::fmt::Debug;

#[derive(Debug)]
pub struct Stack {
    pub children: Vec<Positioned>,
}

impl Layout for Stack {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let mut children = Vec::new();
        for child in &self.children {
            let child = child.layout(tree, constraints);
            let lbox = LayoutBox::from_child(child, Vector2::zero());
            let id = tree.insert(lbox);
            children.push(id);
        }
        SizedLayoutBox {
            size: constraints.max,
            children,
            material: Material::None,
        }
    }
}

#[derive(Debug)]
pub struct Positioned {
    pub position: Vector2,
    pub child: Box<dyn Layout>,
}

impl Layout for Positioned {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let max_child_size = constraints.max - self.position;
        let child = self.child.layout(
            tree,
            &BoxConstraints {
                min: constraints.min,
                max: max_child_size,
            },
        );

        let child_lbox = LayoutBox::from_child(child, self.position);
        let child_id = tree.insert(child_lbox);
        SizedLayoutBox {
            size: constraints.max,
            children: vec![child_id],
            material: Material::Solid(Color::black().alpha(0.1)),
        }
    }
}

#[derive(Default, Debug)]
pub struct Container {
    pub size: Option<Vector2>,
    pub color: Option<Color>,
    pub child: Option<Box<dyn Layout>>,
}

impl Layout for Container {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        match &self.child {
            Some(child) => {
                let child = child.layout(tree, constraints);
                let lbox = LayoutBox::from_child(child, Vector2::zero());
                let size = lbox.rect.max;
                let child_id = tree.insert(lbox);
                SizedLayoutBox {
                    size,
                    children: vec![child_id],
                    material: match self.color {
                        Some(color) => Material::Solid(color),
                        None => Material::None,
                    },
                }
            },
            None => {
                let size = self.size.unwrap_or_default();
                let size = size.clamp_between(constraints.min, constraints.max);
                SizedLayoutBox {
                    size,
                    children: vec![],
                    material: match self.color {
                        Some(color) => Material::Solid(color),
                        None => Material::None,
                    },
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Debug)]
pub enum Material {
    None,
    Solid(Color),
}

impl Default for Material {
    fn default() -> Material {
        Material::None
    }
}

/// A color stored as RGBA components, each ranging from 0 - 255.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Color {
        Color::transparent()
    }
}

impl Color {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
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

    pub fn black() -> Color {
        Color::rgba(0.0, 0.0, 0.0, 255.0)
    }

    // The alpha is between 0 and 1
    pub fn alpha(self, alpha: f32) -> Color {
        Color::rgba(self.r, self.g, self.b, alpha * 255.0)
    }

    pub fn to_linear(&self) -> Vector4 {
        let r = self.r / 255.0;
        let g = self.g / 255.0;
        let b = self.b / 255.0;
        let a = self.a / 255.0;
        Vector4::new(r, g, b, a)
    }
}
