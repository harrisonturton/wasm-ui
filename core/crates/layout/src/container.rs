use crate::base::{Alignment, EdgeInsets};
use crate::decoration::{Borders, Color, Material};
use crate::tree::{BoxConstraints, Layout, LayoutBox, LayoutTree, SizedLayoutBox};
use math::Vector2;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Spacer {}

impl Layout for Spacer {
    fn layout(&self, _: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        SizedLayoutBox {
            size: constraints.max,
            children: vec![],
            material: None,
            margin: EdgeInsets::zero(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Container {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub alignment: Alignment,
    pub padding: EdgeInsets,
    pub borders: Borders,
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

        let h_padding = self.padding.left + self.padding.right + self.borders.total_width();
        let v_padding = self.padding.top + self.padding.bottom + self.borders.total_height();
        let child_h_constraints = match width {
            Some(width) => Vector2::new(0.0, width - h_padding),
            None => h_axis_constraints - Vector2::new(h_padding, 0.0),
        };
        let child_v_constraints = match height {
            Some(height) => Vector2::new(0.0, height - v_padding),
            None => v_axis_constraints - Vector2::new(0.0, v_padding),
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
        let lbox = LayoutBox::from_child(
            sbox,
            pos + self.margin.min() + self.padding.min() + self.borders.min(),
        );
        let id = tree.insert(lbox);

        SizedLayoutBox {
            size,
            children: vec![id],
            material: Some(Material {
                fill: self.color,
                borders: self.borders,
            }),
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
            material: Some(Material {
                fill: self.color,
                borders: self.borders,
            }),
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

#[cfg(test)]
mod tests {
    use super::*;
    use math::Rect;
    use test_util::assert_slice_eq;

    #[test]
    pub fn container_with_no_child_fills_constraints() {
        let container = Container {
            color: Color::green(),
            ..Container::default()
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&container, &constraints);
        let expected_layout = vec![LayoutBox {
            bounds: Rect::from_pos((0.0, 0.0), (100.0, 100.0)),
            ..fixed_child_lbox(Color::green())
        }];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    pub fn container_with_no_child_and_has_height_fills_container_width() {
        let container = Container {
            height: Some(50.0),
            color: Color::green(),
            ..Container::default()
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&container, &constraints);
        let expected_layout = vec![LayoutBox {
            bounds: Rect::from_pos((0.0, 0.0), (100.0, 50.0)),
            ..fixed_child_lbox(Color::green())
        }];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    pub fn container_with_no_child_and_has_width_fills_container_height() {
        let container = Container {
            width: Some(50.0),
            color: Color::green(),
            ..Container::default()
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&container, &constraints);
        let expected_layout = vec![LayoutBox {
            bounds: Rect::from_pos((0.0, 0.0), (50.0, 100.0)),
            ..fixed_child_lbox(Color::green())
        }];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    pub fn container_with_child_has_same_size_as_child_in_unbounded_parent() {
        let container = Container {
            color: Color::green(),
            child: Some(Box::new(Container {
                width: Some(50.0),
                height: Some(50.0),
                color: Color::red(),
                ..Container::default()
            })),
            ..Container::default()
        };

        let constraints = BoxConstraints::from_max(Vector2::new(f32::INFINITY, f32::INFINITY));
        let actual_layout = layout_with_constraints(&container, &constraints);
        let expected_layout = vec![
            LayoutBox {
                bounds: Rect::from_pos((0.0, 0.0), (50.0, 50.0)),
                ..fixed_child_lbox(Color::red())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 0.0), (50.0, 50.0)),
                children: vec![0],
                ..fixed_child_lbox(Color::green())
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    // --------------------------------------------------
    // Helpers
    // --------------------------------------------------

    fn layout_with_constraints(
        widget: &dyn Layout,
        constraints: &BoxConstraints,
    ) -> Vec<LayoutBox> {
        let mut tree = LayoutTree::new();
        let sbox = widget.layout(&mut tree, constraints);
        let lbox = LayoutBox::from_child(sbox, Vector2::zero());
        tree.insert(lbox);
        tree.boxes
    }

    fn fixed_child_lbox(color: Color) -> LayoutBox {
        LayoutBox {
            bounds: Rect::from_size((10.0, 10.0)),
            children: vec![],
            material: Some(Material::filled(color)),
            ..LayoutBox::default()
        }
    }
}
