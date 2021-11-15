use super::{BoxConstraints, Layout, LayoutBox, LayoutTree, SizedLayoutBox};
use math::{Vector2, Vector4};
use std::fmt::Debug;

// --------------------------------------------------
// Center
// --------------------------------------------------

#[derive(Debug)]
pub struct Center {
    pub child: Box<dyn Layout>,
}

impl Layout for Center {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let sbox = self.child.layout(tree, constraints);
        let pos = (constraints.max / 2.0) - (sbox.size / 2.0);
        let lbox = LayoutBox::from_child(sbox, pos);
        let id = tree.insert(lbox);
        SizedLayoutBox {
            size: constraints.max,
            children: vec![id],
            material: Material::None,
        }
    }
}

// --------------------------------------------------
// Stack
// --------------------------------------------------

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

// --------------------------------------------------
// Positioned
// --------------------------------------------------

#[derive(Debug)]
pub struct Positioned {
    pub position: Vector2,
    pub child: Box<dyn Layout>,
}

impl Layout for Positioned {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let child_constraints = BoxConstraints {
            min: Vector2::zero(),
            max: constraints.max - self.position,
        };
        let sbox = self.child.layout(tree, &child_constraints);
        let lbox = LayoutBox::from_child(sbox, self.position);
        let child_id = tree.insert(lbox);
        SizedLayoutBox {
            size: constraints.max,
            children: vec![child_id],
            material: Material::Solid(Color::black().alpha(0.1)),
        }
    }
}

// --------------------------------------------------
// Container
// --------------------------------------------------

#[derive(Debug, Default)]
pub struct EdgeInsets {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl EdgeInsets {
    pub fn zero() -> EdgeInsets {
        EdgeInsets::all(0.0)
    }

    pub fn all(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: inset,
            bottom: inset,
            left: inset,
            right: inset,
        }
    }

    pub fn vertical(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: inset,
            bottom: inset,
            left: 0.0,
            right: 0.0,
        }
    }

    pub fn horizontal(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: 0.0,
            bottom: 0.0,
            left: inset,
            right: inset,
        }
    }

    pub fn top(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: inset,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }

    pub fn bottom(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: 0.0,
            bottom: inset,
            left: 0.0,
            right: 0.0,
        }
    }

    pub fn left(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: 0.0,
            bottom: 0.0,
            left: inset,
            right: 0.0,
        }
    }

    pub fn right(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: inset,
        }
    }

    pub fn min(&self) -> Vector2 {
        Vector2::new(self.left, self.top)
    }

    pub fn max(&self) -> Vector2 {
        Vector2::new(self.right, self.bottom)
    }
}

#[derive(Debug)]
pub struct Container {
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
    pub size: Vector2,
    pub color: Color,
    pub child: Option<Box<dyn Layout>>,
}

impl Default for Container {
    fn default() -> Container {
        Container {
            padding: EdgeInsets::zero(),
            margin: EdgeInsets::zero(),
            size: Vector2::new(f32::INFINITY, f32::INFINITY),
            color: Color::transparent(),
            child: None,
        }
    }
}

impl Layout for Container {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        match &self.child {
            Some(child) => {
                let desired_max = Vector2::new(
                    self.size.x - self.padding.left - self.padding.right,
                    self.size.y - self.padding.top - self.padding.bottom,
                );
                let child_constraints = BoxConstraints {
                    min: Vector2::zero(),
                    max: Vector2::new(
                        desired_max.x.clamp(constraints.min.x, constraints.max.x),
                        desired_max.y.clamp(constraints.min.y, constraints.max.y),
                    ),
                };
                let sbox = child.layout(tree, &child_constraints);
                let child_size = sbox.size;
                let lbox = LayoutBox::from_child(sbox, self.padding.min() + self.margin.min());
                let child_id = tree.insert(lbox);
                SizedLayoutBox {
                    size: self.size.clamp_between(child_size, constraints.max),
                    children: vec![child_id],
                    material: Material::Solid(self.color),
                }
            }
            None => {
                let child = Rect {
                    size: Vector2::new(
                        self.size.x.clamp(constraints.min.x, constraints.max.x),
                        self.size.y.clamp(constraints.min.y, constraints.max.y),
                    ),
                    color: self.color,
                };
                let size = Vector2::new(
                    (self.size.x + self.margin.left + self.margin.right)
                        .clamp(constraints.min.x, constraints.max.x),
                    (self.size.y + self.margin.top + self.margin.bottom)
                        .clamp(constraints.min.y, constraints.max.y),
                );
                let sbox = child.layout(tree, constraints);
                let lbox = LayoutBox::from_child(sbox, self.margin.min());
                let id = tree.insert(lbox);
                SizedLayoutBox {
                    size,
                    children: vec![id],
                    material: Material::None,
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Rect {
    size: Vector2,
    color: Color,
}

impl Layout for Rect {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        SizedLayoutBox {
            size: Vector2::new(
                self.size.x.clamp(constraints.min.x, constraints.max.x),
                self.size.y.clamp(constraints.min.y, constraints.max.y),
            ),
            children: vec![],
            material: Material::Solid(self.color),
        }
    }
}

// --------------------------------------------------
// Decoration
// --------------------------------------------------

#[derive(PartialEq, Clone, Copy, Debug)]
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

    pub fn yellow() -> Color {
        Color::rgba(255.0, 255.0, 0.0, 255.0)
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
