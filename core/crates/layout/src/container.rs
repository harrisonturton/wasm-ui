use crate::base::EdgeInsets;
use crate::decoration::{Borders, Color, Material};
use crate::tree::{BoxConstraints, Layout, LayoutBox, LayoutTree, SizedLayoutBox};
use math::Vector2;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Container {
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
    pub size: Vector2,
    pub color: Color,
    pub borders: Borders,
    pub child: Option<Box<dyn Layout>>,
}

impl Default for Container {
    fn default() -> Container {
        Container {
            padding: EdgeInsets::zero(),
            margin: EdgeInsets::zero(),
            size: Vector2::new(f32::INFINITY, f32::INFINITY),
            color: Color::transparent(),
            borders: Borders::none(),
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
                let lbox = LayoutBox::from_child(
                    sbox,
                    self.padding.min() + self.margin.min() + self.borders.min(),
                );
                let child_id = tree.insert(lbox);
                let child_size = Vector2::new(
                    child_size.x + self.borders.total_width(),
                    child_size.y + self.borders.total_height(),
                );
                SizedLayoutBox {
                    size: self.size.clamp_between(child_size, constraints.max),
                    children: vec![child_id],
                    material: Some(Material {
                        fill: self.color,
                        borders: self.borders,
                    }),
                    ..SizedLayoutBox::default()
                }
            }
            None => {
                let margin_horizontal = self.margin.left + self.margin.right;
                let margin_vertical = self.margin.top + self.margin.bottom;
                let size_x =
                    (self.size.x + margin_horizontal).clamp(constraints.min.x, constraints.max.x);
                let size_y =
                    (self.size.y + margin_vertical).clamp(constraints.min.y, constraints.max.y);
                let child_size = Vector2::new(
                    size_x + self.borders.total_width(),
                    size_y + self.borders.total_height(),
                );
                SizedLayoutBox {
                    size: child_size,
                    children: vec![],
                    material: Some(Material {
                        fill: self.color,
                        borders: self.borders,
                    }),
                    margin: self.margin,
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
            material: Some(Material::filled(self.color)),
            ..SizedLayoutBox::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_util::assert_slice_eq;

    #[test]
    fn container_with_no_child_has_fixed_size() {
        let container = Container {
            size: (10.0, 10.0).into(),
            color: Color::green(),
            ..Container::default()
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&container, &constraints);
        let expected_layout = vec![LayoutBox {
            bounds: math::Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
            margin: EdgeInsets::zero(),
            children: vec![],
            material: Some(Material::filled(Color::green())),
        }];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn container_with_infinite_size_and_no_child_stays_within_constraints() {
        let container = Container {
            size: (f32::INFINITY, f32::INFINITY).into(),
            color: Color::green(),
            ..Container::default()
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&container, &constraints);
        let expected_layout = vec![LayoutBox {
            bounds: math::Rect::from_pos((0.0, 0.0), (100.0, 100.0)),
            margin: EdgeInsets::zero(),
            children: vec![],
            material: Some(Material::filled(Color::green())),
        }];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn container_with_size_and_child_has_same_size_as_child() {
        let container = Container {
            size: (10.0, 10.0).into(),
            color: Color::green(),
            child: Some(Box::new(Container {
                size: (100.0, 100.0).into(),
                color: Color::red(),
                ..Container::default()
            })),
            ..Container::default()
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&container, &constraints);
        let expected_layout = vec![
            LayoutBox {
                bounds: math::Rect::from_pos((0.0, 0.0), (100.0, 100.0)),
                margin: EdgeInsets::zero(),
                children: vec![],
                material: Some(Material::filled(Color::green())),
            },
            LayoutBox {
                bounds: math::Rect::from_pos((0.0, 0.0), (100.0, 100.0)),
                margin: EdgeInsets::zero(),
                children: vec![0],
                material: Some(Material::filled(Color::green())),
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn container_with_size_and_margin_and_no_child_has_correct_layout() {
        let container = Container {
            margin: EdgeInsets::all(5.0),
            size: (10.0, 10.0).into(),
            color: Color::green(),
            ..Container::default()
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&container, &constraints);
        let expected_layout = vec![LayoutBox {
            bounds: math::Rect::from_pos((0.0, 0.0), (20.0, 20.0)),
            margin: EdgeInsets::all(5.0),
            children: vec![],
            material: Some(Material::filled(Color::green())),
        }];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

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
}
