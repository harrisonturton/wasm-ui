use super::{BoxConstraints, Layout, LayoutBox, LayoutTree, SizedLayoutBox};
use math::{Vector2, Vector4};
use std::fmt::Debug;

// --------------------------------------------------
// Flex Container Utilities
// --------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub enum MainAxisSize {
    Max,
    Min,
}

impl Default for MainAxisSize {
    fn default() -> MainAxisSize {
        MainAxisSize::Min
    }
}

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
// Row
// --------------------------------------------------

#[derive(Default, Debug)]
pub struct Row {
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
    pub children: Vec<Flex>,
}

impl Row {
    fn fixed_constraints(&self, constraints: &BoxConstraints) -> BoxConstraints {
        match self.cross_axis_alignment {
            CrossAxisAlignment::Start | CrossAxisAlignment::End | CrossAxisAlignment::Center => {
                BoxConstraints {
                    min: (0.0, 0.0).into(),
                    max: (constraints.max.x, constraints.max.y).into(),
                }
            }
            CrossAxisAlignment::Stretch => BoxConstraints {
                min: (0.0, constraints.max.y).into(),
                max: (constraints.max.x, constraints.max.y).into(),
            },
        }
    }

    fn flex_constraints(&self, constraints: &BoxConstraints, flex_width: f32) -> BoxConstraints {
        match self.cross_axis_alignment {
            CrossAxisAlignment::Start | CrossAxisAlignment::End | CrossAxisAlignment::Center => {
                BoxConstraints {
                    min: (flex_width, 0.0).into(),
                    max: (flex_width, constraints.max.y).into(),
                }
            }
            CrossAxisAlignment::Stretch => BoxConstraints {
                min: (flex_width, constraints.max.y).into(),
                max: (flex_width, constraints.max.y).into(),
            },
        }
    }

    fn lbox_position(
        &self,
        constraints: &BoxConstraints,
        // The height of the current element
        height: f32,
        // The amount of space all widgets take up
        total_width: f32,
        // The total number of widgets
        num_widgets: usize,
        // The order of the current widget being positioned
        index: usize,
        // Where we are up to in vertical layout
        current_x_pos: f32,
    ) -> Vector2 {
        let y = match self.cross_axis_alignment {
            CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => 0.0,
            CrossAxisAlignment::End => constraints.max.y - height,
            CrossAxisAlignment::Center => constraints.max.y * 0.5 - height * 0.5,
        };
        let x = match self.main_axis_alignment {
            MainAxisAlignment::Start => current_x_pos,
            MainAxisAlignment::SpaceEvenly => {
                let space = (constraints.max.x - total_width) / (num_widgets as f32 + 1.0);
                current_x_pos + space
            }
            MainAxisAlignment::SpaceAround => {
                let space = (constraints.max.x - total_width) / num_widgets as f32;
                if index == 0 || index == num_widgets {
                    current_x_pos + (space / 2.0)
                } else {
                    current_x_pos + space
                }
            }
            MainAxisAlignment::SpaceBetween => {
                let space = (constraints.max.x - total_width) / (num_widgets as f32 - 1.0);
                if index == 0 {
                    current_x_pos
                } else {
                    current_x_pos + space
                }
            }
        };
        Vector2::new(x, y)
    }
}

impl Layout for Row {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let mut sum_fixed_width = 0.0;
        let mut sum_flex_factor = 0.0;

        // Keep track of the children that have already undergone layout
        let mut sboxes: Vec<(&Flex, Option<SizedLayoutBox>)> = Vec::new();

        // We do two passes over the children. The first pass calculates the
        // total width of the inflexible children, and the sum of all the flex
        // factors of the flexible children, because this is needed to determine
        // how much space the flexible children can take up.
        for child in &self.children {
            match child {
                Flex::Fixed { .. } => {
                    let constraints = self.fixed_constraints(constraints);
                    let sbox = child.layout(tree, &constraints);
                    sum_fixed_width += sbox.size.x;
                    sboxes.push((child, Some(sbox)));
                }
                Flex::Flexible { flex, .. } => {
                    sum_flex_factor += flex;
                    sboxes.push((child, None));
                }
            };
        }

        // Now we can determine the relative sizing of the flexible widgets
        let free_space = constraints.max.x - sum_fixed_width;
        let space_per_flex = free_space / sum_flex_factor;

        // The second pass will size all the flexible children
        let mut total_width = sum_fixed_width;
        for child in &mut sboxes {
            if let (Flex::Flexible { flex, .. }, None) = child {
                let width = flex * space_per_flex;
                let flex_constraints = self.flex_constraints(constraints, width);
                let sbox = child.0.layout(tree, &flex_constraints);
                child.1 = Some(sbox);
                total_width += width;
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
                    size.y,
                    total_width,
                    sboxes.len(),
                    i,
                    total_size.x,
                );
                let lbox = LayoutBox::from_child(sbox.clone(), pos);
                let id = tree.insert(lbox);
                children.push(id);
                // Keep track of how big we are
                total_size.y = f32::max(total_size.y, size.y);
                total_size.x += (pos.x - total_size.x) + size.x;
            }
        }

        let size_y = match self.cross_axis_alignment {
            CrossAxisAlignment::Start => total_size.y,
            _ => constraints.max.y,
        };
        let size_x = match self.main_axis_alignment {
            MainAxisAlignment::Start => total_size.x,
            MainAxisAlignment::SpaceEvenly
            | MainAxisAlignment::SpaceAround
            | MainAxisAlignment::SpaceBetween => constraints.max.x,
        };
        SizedLayoutBox {
            size: Vector2::new(size_x, size_y),
            children,
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
