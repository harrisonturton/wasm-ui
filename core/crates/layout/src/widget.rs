use super::{BoxConstraints, Layout, LayoutBox, LayoutTree, SizedLayoutBox};
use math::{Vector2, Vector4};
use std::fmt::Debug;

// --------------------------------------------------
// Flex
// --------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub enum CrossAxisAlignment {
    Start,
    End,
    Stretch,
    Center,
}

impl Default for CrossAxisAlignment {
    fn default() -> CrossAxisAlignment {
        CrossAxisAlignment::Start
    }
}

#[derive(Debug)]
pub enum Flex {
    Flexible { flex: f32, child: Box<dyn Layout> },
    Fixed { child: Box<dyn Layout> },
}

// The [Flex] widget doesn't render any additional primitives. It just wraps an
// existing widget in order to provide the `flex` property to the parent.
impl Layout for Flex {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let child = match self {
            Flex::Flexible { child, .. } => child,
            Flex::Fixed { child } => child,
        };
        child.layout(tree, constraints)
    }
}

// --------------------------------------------------
// Column
// --------------------------------------------------

#[derive(Default, Debug)]
pub struct Column {
    pub cross_axis_alignment: CrossAxisAlignment,
    pub children: Vec<Flex>,
}

impl Column {
    fn fixed_constraints(&self, constraints: &BoxConstraints) -> BoxConstraints {
        match self.cross_axis_alignment {
            CrossAxisAlignment::Start | CrossAxisAlignment::End | CrossAxisAlignment::Center => {
                BoxConstraints {
                    min: (0.0, 0.0).into(),
                    max: (constraints.max.x, f32::INFINITY).into(),
                }
            }
            CrossAxisAlignment::Stretch => BoxConstraints {
                min: (constraints.max.x, 0.0).into(),
                max: (constraints.max.x, f32::INFINITY).into(),
            },
        }
    }

    fn flex_constraints(&self, constraints: &BoxConstraints, flex_height: f32) -> BoxConstraints {
        match self.cross_axis_alignment {
            CrossAxisAlignment::Start | CrossAxisAlignment::End | CrossAxisAlignment::Center => {
                BoxConstraints {
                    min: (0.0, flex_height).into(),
                    max: (constraints.max.x, flex_height).into(),
                }
            }
            CrossAxisAlignment::Stretch => BoxConstraints {
                min: (constraints.max.x, flex_height).into(),
                max: (constraints.max.x, flex_height).into(),
            },
        }
    }

    fn lbox_position(&self, constraints: &BoxConstraints, y_pos: f32, width: f32) -> Vector2 {
        match self.cross_axis_alignment {
            CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => Vector2::new(0.0, y_pos),
            CrossAxisAlignment::End => Vector2::new(constraints.max.x - width, y_pos),
            CrossAxisAlignment::Center => {
                Vector2::new(constraints.max.x * 0.5 - width * 0.5, y_pos)
            }
        }
    }
}

impl Layout for Column {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let mut sum_fixed_height = 0.0;
        let mut sum_flex_factor = 0.0;

        // Keep track of the children that have already undergone layout
        let mut sbox_cache: Vec<(&Flex, Option<SizedLayoutBox>)> = Vec::new();

        // Layout the [Flex::Fixed] elements first, because we need to calculate
        // the total size of inflexible children before we can calculate the
        // relative sized of the flexible children
        for child in &self.children {
            match child {
                Flex::Fixed { .. } => {
                    let constraints = self.fixed_constraints(constraints);
                    let sbox = child.layout(tree, &constraints);
                    sum_fixed_height += sbox.size.y;
                    sbox_cache.push((child, Some(sbox)));
                }
                Flex::Flexible { flex, .. } => {
                    sum_flex_factor += flex;
                    sbox_cache.push((child, None));
                }
            };
        }

        // Now we can determine the relative sizing of the flexible widgets
        let free_space = constraints.max.y - sum_fixed_height;
        let space_per_flex = free_space / sum_flex_factor;

        let mut children = Vec::new();
        let mut max_width = 0.0;
        let mut max_height = 0.0;
        for (child, maybe_sbox) in sbox_cache {
            match maybe_sbox {
                // This is an inflexible widget that has already been laid out
                Some(sbox) => {
                    let size = sbox.size;
                    let pos = self.lbox_position(constraints, max_height, sbox.size.x);
                    let lbox = LayoutBox::from_child(sbox, pos);
                    let id = tree.insert(lbox);
                    children.push(id);
                    max_width = f32::max(max_width, size.x);
                    max_height += size.y;
                }
                // This is a flexible widget that must be laid out
                None => match child {
                    Flex::Fixed { .. } => (),
                    Flex::Flexible { flex, .. } => {
                        let height = flex * space_per_flex;
                        let flex_constraints = self.flex_constraints(constraints, height);
                        let sbox = child.layout(tree, &flex_constraints);
                        let size = sbox.size;
                        let pos = self.lbox_position(constraints, max_height, sbox.size.x);
                        let lbox = LayoutBox::from_child(sbox, pos);
                        let id = tree.insert(lbox);
                        children.push(id);
                        max_height += size.y;
                        max_width = f32::max(max_width, size.x);
                    }
                },
            };
        }

        let size = match self.cross_axis_alignment {
            CrossAxisAlignment::Start => Vector2::new(max_width, max_height),
            _ => Vector2::new(constraints.max.x, max_height),
        };
        SizedLayoutBox {
            size,
            children,
            material: Material::Solid(Color::green().alpha(0.5)),
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
            max: constraints.max - constraints.min - self.position,
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
