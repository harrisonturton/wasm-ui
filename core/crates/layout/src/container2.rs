use crate::base::EdgeInsets;
use crate::decoration::{Color, Material};
use crate::tree::{BoxConstraints, Layout, LayoutBox, LayoutTree, SizedLayoutBox};
use math::Vector2;
use std::fmt::Debug;

// The position of the center of a widget as a fraction of the available area.
// The widget should not overflow at (0.0, 0.0) or at (1.0, 1.0), it should be
// clamped to the edges of the screen.
#[derive(PartialEq, Copy, Clone, Debug, Default)]
pub struct Alignment {
    x: f32,
    y: f32,
}

impl Alignment {
    #[must_use]
    pub fn new(x: f32, y: f32) -> Alignment {
        Alignment { x, y }
    }

    #[must_use]
    pub fn top_left() -> Alignment {
        Alignment::new(0.0, 0.0)
    }

    #[must_use]
    pub fn top_center() -> Alignment {
        Alignment::new(0.5, 0.0)
    }

    #[must_use]
    pub fn top_right() -> Alignment {
        Alignment::new(0.5, 0.0)
    }

    #[must_use]
    pub fn center_left() -> Alignment {
        Alignment::new(0.0, 0.5)
    }

    #[must_use]
    pub fn center() -> Alignment {
        Alignment::new(0.5, 0.5)
    }

    #[must_use]
    pub fn center_right() -> Alignment {
        Alignment::new(1.0, 0.5)
    }

    #[must_use]
    pub fn bottom_left() -> Alignment {
        Alignment::new(0.0, 1.0)
    }

    #[must_use]
    pub fn bottom_center() -> Alignment {
        Alignment::new(0.5, 1.0)
    }

    #[must_use]
    pub fn bottom_right() -> Alignment {
        Alignment::new(1.0, 1.0)
    }

    #[must_use]
    pub fn to_vector(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }
}

#[derive(Debug, Default)]
pub struct Container {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub alignment: Alignment,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
    pub color: Color,
    pub child: Option<Box<dyn Layout>>,
}

impl Layout for Container {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        match &self.child {
            Some(child) => self.layout_with_child(tree, constraints, child.as_ref()),
            None => self.layout_without_child(constraints),
        }
    }
}

impl Container {
    fn layout_with_child(
        &self,
        tree: &mut LayoutTree,
        constraints: &BoxConstraints,
        child: &dyn Layout,
    ) -> SizedLayoutBox {
        let h_axis_constraints = constraints.horizontal();
        let v_axis_constraints = constraints.vertical();
        let width = Container::calculate_size(self.width, h_axis_constraints);
        let height = Container::calculate_size(self.height, v_axis_constraints);

        let child_h_constraints = match width {
            Some(width) => Vector2::new(0.0, width),
            None => h_axis_constraints,
        };
        let child_v_constraints = match height {
            Some(height) => Vector2::new(0.0, height),
            None => v_axis_constraints,
        };
        let child_constraints = BoxConstraints {
            min: Vector2::new(child_h_constraints.x, child_v_constraints.x),
            max: Vector2::new(child_h_constraints.y, child_v_constraints.y),
        };
        let sbox = child.layout(tree, &child_constraints);
        let child_size = sbox.size;

        let mut pos_x = 0.0;
        let mut pos_y = 0.0;
        let size_x = match width {
            Some(width) => {
                pos_x = width * self.alignment.x;
                width
            }
            None => child_size.x,
        };
        let size_y = match height {
            Some(height) => {
                pos_y = height * self.alignment.y;
                height
            }
            None => child_size.y,
        };
        let size = Vector2::new(size_x, size_y) + self.margin.total();

        let pos_x = (pos_x - child_size.x * 0.5).clamp(0.0, constraints.max.x - child_size.x);
        let pos_y = (pos_y - child_size.y * 0.5).clamp(0.0, constraints.max.y - child_size.y);
        let pos = Vector2::new(pos_x, pos_y);
        let lbox = LayoutBox::from_child(sbox, pos + self.margin.min());
        let id = tree.insert(lbox);

        SizedLayoutBox {
            size,
            children: vec![id],
            material: Some(Material::filled(self.color)),
            margin: self.margin,
        }
    }

    fn layout_without_child(&self, constraints: &BoxConstraints) -> SizedLayoutBox {
        let h_axis_constraints = constraints.horizontal();
        let v_axis_constraints = constraints.vertical();
        let width = Container::calculate_size(self.width, h_axis_constraints).unwrap_or(0.0);
        let height = Container::calculate_size(self.height, v_axis_constraints).unwrap_or(0.0);
        let size = Vector2::new(width, height) + self.margin.total();
        SizedLayoutBox {
            size,
            children: vec![],
            material: Some(Material::filled(self.color)),
            margin: self.margin,
        }
    }

    // If return none, then shrink to fit child
    fn calculate_size(desired_size: Option<f32>, axis_constraints: Vector2) -> Option<f32> {
        let (min_size, max_size) = axis_constraints.into();
        match desired_size {
            Some(size) => Some(size.clamp(min_size, max_size)),
            None if max_size == f32::INFINITY => None,
            None => Some(max_size),
        }
    }
}

#[cfg(tests)]
mod tests {
    use super::*;
}
