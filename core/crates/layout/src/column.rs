use super::{
    BoxConstraints, Color, CrossAxisAlignment, Flex, Layout, LayoutBox, LayoutTree,
    MainAxisAlignment, MainAxisSize, Material, SizedLayoutBox,
};
use math::Vector2;

// --------------------------------------------------
// Column
// --------------------------------------------------

#[derive(Default, Debug)]
pub struct Column {
    pub main_axis_size: MainAxisSize,
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
                    max: (constraints.max.x, constraints.max.y).into(),
                }
            }
            CrossAxisAlignment::Stretch => BoxConstraints {
                min: (constraints.max.x, 0.0).into(),
                max: (constraints.max.x, constraints.max.y).into(),
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
            CrossAxisAlignment::Start => total_size.x.clamp(constraints.min.x, constraints.max.x),
            _ => constraints.max.x,
        };
        let size = match self.main_axis_size {
            MainAxisSize::Min => Vector2::new(size_x, total_height.clamp(constraints.min.y, constraints.max.y)),
            MainAxisSize::Max => Vector2::new(size_x, constraints.max.y),
        };

        SizedLayoutBox {
            size,
            children,
            material: Material::Solid(Color::blue()),
        }
    }
}
