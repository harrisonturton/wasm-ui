use crate::tree::{LayoutTree, Layout, SizedLayoutBox, LayoutBox, BoxConstraints};
use crate::base::EdgeInsets;
use crate::decoration::{Color, Material};
use math::Vector2;
use std::fmt::Debug;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_with_size_and_no_child_has_fixed_size() {
        let container = Container {
            size: (10.0, 10.0).into(),
            color: Color::green(),
            ..Default::default()
        };
    }
}