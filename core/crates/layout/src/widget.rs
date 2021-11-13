use super::{BoxConstraints, Layout, LayoutBox, LayoutTree, SizedLayoutBox};
use math::{Vector2, Vector4};
use std::fmt::Debug;

// --------------------------------------------------
// Flex
// --------------------------------------------------

#[derive(Debug)]
pub struct Flex {
    pub flex: Option<f32>,
    pub child: Box<dyn Layout>,
}

// The [Flex] widget doesn't render any additional primitives. It just wraps an
// existing widget in order to provide the `flex` property to the parent.
impl Layout for Flex {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        self.child.layout(tree, constraints)
    }
}

// --------------------------------------------------
// Row
// --------------------------------------------------

#[derive(Default, Debug)]
pub struct Row {
    pub children: Vec<Flex>,
}

impl Layout for Row {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        // First pass calculates the sum of the heights of inflexible children
        // and the sum of all the flex factors. These are required for the flex
        // layout.
        let mut sum_inflexible_width = 0.0; 
        let mut sum_flex_factor = 0.0;
        let child_constraints = BoxConstraints {
            min: Vector2::new(0.0, 0.0),
            max: Vector2::new(f32::INFINITY, constraints.max.x),
        };
        for widget in &self.children {
            match widget.flex {
                Some(flex) => {
                    sum_flex_factor += flex;
                },
                None => {
                    let sized = widget.layout(tree, &child_constraints);
                    sum_inflexible_width += sized.size.x;
                }
            }
        }
        let free_space = constraints.max.x - sum_inflexible_width;
        let space_per_flex = free_space / sum_flex_factor;

        // Second pass performs the actual layout.
        let mut x_pos = 0.0;
        let mut children = vec![];
        for widget in &self.children {
            match widget.flex {
                Some(flex) => {
                    let flex_constraints = BoxConstraints {
                        min: Vector2::new(flex * space_per_flex, 100.0),
                        max: Vector2::new(flex * space_per_flex, constraints.max.y),
                    };
                    let sized = widget.layout(tree, &flex_constraints);
                    let child = LayoutBox::from_child(sized, Vector2::new(x_pos, 0.0));
                    let child_id = tree.insert(child);
                    children.push(child_id);
                    x_pos += flex * space_per_flex;
                },
                None => {
                    let sized = widget.layout(tree, constraints);
                    let child = LayoutBox::from_child(sized, Vector2::new(x_pos, 0.0));
                    x_pos += child.rect.max.x - child.rect.min.x;
                    let child_id = tree.insert(child);
                    children.push(child_id);
                },
            }
        }

        SizedLayoutBox {
            size: constraints.max,
            children,
            material: Material::Solid(Color::blue()),
        }
    }
}

// --------------------------------------------------
// Column
// --------------------------------------------------

#[derive(Default, Debug)]
pub struct Column {
    pub children: Vec<Flex>,
}

impl Layout for Column {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        // First pass calculates the sum of the heights of inflexible children
        // and the sum of all the flex factors. These are required for the flex
        // layout.
        let mut sum_inflexible_height = 0.0; 
        let mut sum_flex_factor = 0.0;
        let child_constraints = BoxConstraints {
            min: Vector2::new(0.0, 0.0),
            max: Vector2::new(constraints.max.x, f32::INFINITY),
        };
        for widget in &self.children {
            match widget.flex {
                Some(flex) => {
                    sum_flex_factor += flex;
                },
                None => {
                    let sized = widget.layout(tree, &child_constraints);
                    sum_inflexible_height += sized.size.y;
                }
            }
        }
        let free_space = constraints.max.y - sum_inflexible_height;
        let space_per_flex = free_space / sum_flex_factor;

        // Second pass performs the actual layout.
        let mut y_pos = 0.0;
        let mut children = vec![];
        for widget in &self.children {
            match widget.flex {
                Some(flex) => {
                    let flex_constraints = BoxConstraints {
                        min: Vector2::new(100.0, flex * space_per_flex),
                        max: Vector2::new(constraints.max.x, flex * space_per_flex),
                    };
                    let sized = widget.layout(tree, &flex_constraints);
                    let child = LayoutBox::from_child(sized, Vector2::new(0.0, y_pos));
                    let child_id = tree.insert(child);
                    children.push(child_id);
                    y_pos += flex * space_per_flex;
                },
                None => {
                    let sized = widget.layout(tree, constraints);
                    let child = LayoutBox::from_child(sized, Vector2::new(0.0, y_pos));
                    y_pos += child.rect.max.y - child.rect.min.y;
                    let child_id = tree.insert(child);
                    children.push(child_id);
                },
            }
        }

        SizedLayoutBox {
            size: constraints.max,
            children,
            material: Material::Solid(Color::blue()),
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

// --------------------------------------------------
// Container
// --------------------------------------------------

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
            }
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

// --------------------------------------------------
// Decoration
// --------------------------------------------------

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
