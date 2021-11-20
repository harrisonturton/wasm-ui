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
    #[must_use]
    pub fn zero() -> EdgeInsets {
        EdgeInsets::all(0.0)
    }

    #[must_use]
    pub fn all(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: inset,
            bottom: inset,
            left: inset,
            right: inset,
        }
    }

    #[must_use]
    pub fn vertical(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: inset,
            bottom: inset,
            left: 0.0,
            right: 0.0,
        }
    }

    #[must_use]
    pub fn horizontal(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: 0.0,
            bottom: 0.0,
            left: inset,
            right: inset,
        }
    }

    #[must_use]
    pub fn top(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: inset,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }

    #[must_use]
    pub fn bottom(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: 0.0,
            bottom: inset,
            left: 0.0,
            right: 0.0,
        }
    }

    #[must_use]
    pub fn left(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: 0.0,
            bottom: 0.0,
            left: inset,
            right: 0.0,
        }
    }

    #[must_use]
    pub fn right(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: inset,
        }
    }

    #[must_use]
    pub fn min(&self) -> Vector2 {
        Vector2::new(self.left, self.top)
    }

    #[must_use]
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
                let h_padding = self.padding.left + self.padding.right;
                let v_padding = self.padding.top + self.padding.bottom;
                let desired_max = Vector2::new(self.size.x - h_padding, self.size.y - v_padding);
                let child_constraints = BoxConstraints {
                    min: Vector2::zero(),
                    max: Vector2::new(
                        desired_max
                            .x
                            .clamp(constraints.min.x - h_padding, constraints.max.x - h_padding),
                        desired_max
                            .y
                            .clamp(constraints.min.y - v_padding, constraints.max.y - v_padding),
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
    pub size: Vector2,
    pub color: Color,
}

impl Layout for Rect {
    fn layout(&self, _: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
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
    #[must_use]
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    #[must_use]
    pub fn transparent() -> Color {
        Color::rgba(0.0, 0.0, 0.0, 0.0)
    }

    #[must_use]
    pub fn red() -> Color {
        Color::rgba(255.0, 0.0, 0.0, 255.0)
    }

    #[must_use]
    pub fn green() -> Color {
        Color::rgba(0.0, 255.0, 0.0, 255.0)
    }

    #[must_use]
    pub fn blue() -> Color {
        Color::rgba(0.0, 0.0, 255.0, 255.0)
    }

    #[must_use]
    pub fn yellow() -> Color {
        Color::rgba(255.0, 255.0, 0.0, 255.0)
    }

    #[must_use]
    pub fn white() -> Color {
        Color::rgba(255.0, 255.0, 255.0, 255.0)
    }

    #[must_use]
    pub fn black() -> Color {
        Color::rgba(0.0, 0.0, 0.0, 255.0)
    }

    // The alpha is between 0 and 1
    #[must_use]
    pub fn alpha(self, alpha: f32) -> Color {
        Color::rgba(self.r, self.g, self.b, alpha * 255.0)
    }

    #[must_use]
    pub fn to_linear(&self) -> Vector4 {
        let r = self.r / 255.0;
        let g = self.g / 255.0;
        let b = self.b / 255.0;
        let a = self.a / 255.0;
        Vector4::new(r, g, b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_partial_eq_with_same_color_returns_true() {
        let red_lhs = Color::red();
        let red_rhs = Color::red();
        assert_eq!(red_lhs, red_rhs);
    }

    #[test]
    fn color_partial_eq_with_different_color_returns_false() {
        let red = Color::red();
        let green = Color::green();
        assert_ne!(red, green);
    }

    #[test]
    fn material_partial_eq_with_different_color_returns_false() {
        let red = Material::Solid(Color::red());
        let green = Material::Solid(Color::green());
        assert_ne!(red, green);
    }
}
