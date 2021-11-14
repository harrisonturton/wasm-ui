use super::{BoxConstraints, Layout, LayoutBox, LayoutTree, SizedLayoutBox};
use math::{Vector2, Vector4};
use std::fmt::Debug;

// --------------------------------------------------
// Flex
// --------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub enum MainAxisAlignment {
    Start,
    SpaceEvenly,
    SpaceAround,
    SpaceBetween,
}

impl Default for MainAxisAlignment {
    fn default() -> MainAxisAlignment {
        MainAxisAlignment::Start
    }
}

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
    pub main_axis_alignment: MainAxisAlignment,
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

    fn lbox_position(
        &self,
        constraints: &BoxConstraints,
        // The width of the current element
        width: f32,
        // The amount of space all widgets take up
        total_height: f32,
        // The total number of widgets
        num_widgets: usize,
        // The order of the current widget being positioned
        index: usize,
        // Where we are up to in vertical layout
        current_y_pos: f32,
    ) -> Vector2 {
        let x = match self.cross_axis_alignment {
            CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => 0.0,
            CrossAxisAlignment::End => constraints.max.x - width,
            CrossAxisAlignment::Center => constraints.max.x * 0.5 - width * 0.5,
        };
        let y = match self.main_axis_alignment {
            MainAxisAlignment::Start => current_y_pos,
            MainAxisAlignment::SpaceEvenly => {
                let space = (constraints.max.y - total_height) / (num_widgets as f32 + 1.0);
                current_y_pos + space
            }
            MainAxisAlignment::SpaceAround => {
                let space = (constraints.max.y - total_height) / num_widgets as f32;
                if index == 0 || index == num_widgets {
                    current_y_pos + (space / 2.0)
                } else {
                    current_y_pos + space
                }
            }
            MainAxisAlignment::SpaceBetween => {
                let space = (constraints.max.y - total_height) / (num_widgets as f32 - 1.0);
                if index == 0 {
                    current_y_pos
                } else {
                    current_y_pos + space
                }
            }
        };
        Vector2::new(x, y)
    }
}

impl Layout for Column {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let mut sum_fixed_height = 0.0;
        let mut sum_flex_factor = 0.0;

        // Keep track of the children that have already undergone layout
        let mut sboxes: Vec<(&Flex, Option<SizedLayoutBox>)> = Vec::new();

        // We do two passes over the children. The first pass calculates the
        // total height of the inflexible children, and the sum of all the flex
        // factors of the flexible children, because this is needed to determine
        // how much space the flexible children can take up.
        for child in &self.children {
            match child {
                Flex::Fixed { .. } => {
                    let constraints = self.fixed_constraints(constraints);
                    let sbox = child.layout(tree, &constraints);
                    sum_fixed_height += sbox.size.y;
                    sboxes.push((child, Some(sbox)));
                }
                Flex::Flexible { flex, .. } => {
                    sum_flex_factor += flex;
                    sboxes.push((child, None));
                }
            };
        }

        // Now we can determine the relative sizing of the flexible widgets
        let free_space = constraints.max.y - sum_fixed_height;
        let space_per_flex = free_space / sum_flex_factor;

        // The second pass will size all the flexible children
        let mut total_height = sum_fixed_height;
        for child in &mut sboxes {
            if let (Flex::Flexible { flex, .. }, None) = child {
                let height = flex * space_per_flex;
                let flex_constraints = self.flex_constraints(constraints, height);
                let sbox = child.0.layout(tree, &flex_constraints);
                child.1 = Some(sbox);
                total_height += height;
            }
        }

        // Finally, we can calculate their positions after knowing the sizes of
        // each child
        let mut children = Vec::new();
        let mut total_size = Vector2::zero();
        for (i, (_, maybe_sbox)) in sboxes.iter().enumerate() {
            // All children should be sized
            if let Some(sbox) = maybe_sbox {
                let size = sbox.size;
                let pos = self.lbox_position(
                    constraints,
                    size.x,
                    total_height,
                    sboxes.len(),
                    i,
                    total_size.y,
                );
                let lbox = LayoutBox::from_child(sbox.clone(), pos);
                let id = tree.insert(lbox);
                children.push(id);
                // Keep track of how big we are
                total_size.x = f32::max(total_size.x, size.x);
                total_size.y += (pos.y - total_size.y) + size.y;
            }
        }

        let size_x = match self.cross_axis_alignment {
            CrossAxisAlignment::Start => total_size.x,
            _ => constraints.max.x,
        };
        let size_y = match self.main_axis_alignment {
            MainAxisAlignment::Start => total_size.x,
            MainAxisAlignment::SpaceEvenly
            | MainAxisAlignment::SpaceAround
            | MainAxisAlignment::SpaceBetween => constraints.max.y,
        };
        SizedLayoutBox {
            size: Vector2::new(size_x, size_y),
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
