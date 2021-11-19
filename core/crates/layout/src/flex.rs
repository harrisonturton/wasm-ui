use super::{BoxConstraints, Layout, LayoutBox, LayoutTree, Material, SizedLayoutBox};
use math::Vector2;
use std::fmt::Debug;

// --------------------------------------------------
// Axis
// --------------------------------------------------

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Default for Axis {
    fn default() -> Axis {
        Axis::Vertical
    }
}

// --------------------------------------------------
// Axis Alignment
// --------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub enum MainAxisAlignment {
    Start,
    End,
    Center,
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

// --------------------------------------------------
// Axis Size
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

// --------------------------------------------------
// Flex children
// --------------------------------------------------

// Wraps a child of the [FlexGroup] when it should grow. This purely exists to
// add the provide the flex factor value `flex` during layout.
#[derive(Debug)]
pub enum Flex {
    Flexible { flex: f32, child: Box<dyn Layout> },
    Fixed { child: Box<dyn Layout> },
}

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
// FlexGroup
// --------------------------------------------------

#[derive(Debug)]
pub struct FlexGroup {
    pub axis: Axis,
    pub main_axis_size: MainAxisSize,
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
    pub children: Vec<Flex>,
}

impl Default for FlexGroup {
    fn default() -> FlexGroup {
        FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![],
        }
    }
}

impl Layout for FlexGroup {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let mut sum_fixed_size = 0.0; // Along the main axis
        let mut sum_flex_factor = 0.0;

        // Keep track of the children that have already undergone layout
        let mut sbox_cache: Vec<(&Flex, Option<SizedLayoutBox>)> = vec![];

        // Do two passes over the children. The first pass calculates the total
        // size (along the main axis) taken up by the [Flex::Fixed] children.
        // This is needed to determine how much space if left for the
        // [Flex::Flexible] children to use.
        let mut max_cross_size = 0.0;
        for child in &self.children {
            match child {
                Flex::Fixed { .. } => {
                    let constraints = self.fixed_child_constraints(constraints);
                    let sbox = child.layout(tree, &constraints);
                    sum_fixed_size += self.child_main_axis_size(&sbox);
                    let cross_size = self.child_cross_axis_size(&sbox);
                    max_cross_size = f32::max(max_cross_size, cross_size);
                    sbox_cache.push((child, Some(sbox)));
                }
                Flex::Flexible { flex, .. } => {
                    sum_flex_factor += flex;
                    sbox_cache.push((child, None));
                }
            };
        }

        // Now we can determine the relative sizing of the flexible widgets
        let (_, main_max) = self.main_axis_constraint(constraints).into();
        let free_space = main_max - sum_fixed_size;
        let space_per_flex = free_space / sum_flex_factor;

        // The second pass will size all the [Flex::Flexible] children.
        let mut total_size = sum_fixed_size;
        for child in &mut sbox_cache {
            if let (Flex::Flexible { flex, .. }, None) = child {
                let main_axis_size = flex * space_per_flex;
                let constraints = if *flex > 0.0 {
                    self.flex_child_constraints(constraints, main_axis_size)
                } else {
                    self.fixed_child_constraints(constraints)
                };
                let sbox = child.0.layout(tree, &constraints);
                let size = sbox.size;
                let cross_size = self.child_cross_axis_size(&sbox);
                max_cross_size = f32::max(max_cross_size, cross_size);
                child.1 = Some(sbox);
                total_size += match self.axis {
                    Axis::Horizontal => size.x,
                    Axis::Vertical => size.y,
                };
            }
        }

        // Finally, we can determine their positions
        let mut children = vec![];
        let mut current_main_size = 0.0;

        let sboxes = sbox_cache.iter().filter_map(|(_, sbox)| sbox.as_ref());
        for (i, sbox) in sboxes.enumerate() {
            let cross_size = self.child_cross_axis_size(sbox);
            let main_size = self.child_main_axis_size(sbox);
            let cross_pos = self.child_cross_axis_position(constraints, cross_size, max_cross_size);
            let main_pos = self.child_main_axis_position(
                constraints,
                total_size,
                main_size,
                sbox_cache.len(),
                i,
                current_main_size,
            );
            let pos = match self.axis {
                Axis::Vertical => Vector2::new(cross_pos, main_pos),
                Axis::Horizontal => Vector2::new(main_pos, cross_pos),
            };

            let lbox = LayoutBox::from_child(sbox.clone(), pos);
            let id = tree.insert(lbox);
            children.push(id);
            max_cross_size = f32::max(max_cross_size, cross_size);
            current_main_size += match self.main_axis_alignment {
                MainAxisAlignment::End => main_size,
                _ => main_pos - current_main_size + main_size,
            };
        }

        //let cross_size = max_cross_size;
        let (_, cross_max) = self.cross_axis_constraint(constraints).into();
        let cross_size = match self.cross_axis_alignment {
            CrossAxisAlignment::Stretch | CrossAxisAlignment::Center | CrossAxisAlignment::End => {
                cross_max
            }
            CrossAxisAlignment::Start => max_cross_size,
        };
        let (main_min, main_max) = self.main_axis_constraint(constraints).into();
        let main_size = match self.main_axis_size {
            MainAxisSize::Min => total_size.clamp(main_min, main_max),
            MainAxisSize::Max => main_max,
        };
        let size = match self.axis {
            Axis::Horizontal => Vector2::new(main_size, cross_size),
            Axis::Vertical => Vector2::new(cross_size, main_size),
        };

        SizedLayoutBox {
            size,
            children,
            material: Material::None,
        }
    }
}

impl FlexGroup {
    // Calculate the min and max constraints along the main axis
    fn main_axis_constraint(&self, constraints: &BoxConstraints) -> Vector2 {
        match self.axis {
            Axis::Horizontal => Vector2::new(constraints.min.x, constraints.max.x),
            Axis::Vertical => Vector2::new(constraints.min.y, constraints.max.y),
        }
    }

    // Calculate the min and max constraints along the cross axis
    fn cross_axis_constraint(&self, constraints: &BoxConstraints) -> Vector2 {
        match self.axis {
            Axis::Horizontal => Vector2::new(constraints.min.y, constraints.max.y),
            Axis::Vertical => Vector2::new(constraints.min.x, constraints.max.x),
        }
    }

    fn align_constraints(
        &self,
        main_axis_constraint: Vector2,
        cross_axis_constraint: Vector2,
    ) -> BoxConstraints {
        let (main_min, main_max) = main_axis_constraint.into();
        let (cross_min, cross_max) = cross_axis_constraint.into();
        match self.axis {
            Axis::Horizontal => BoxConstraints {
                min: (main_min, cross_min).into(),
                max: (main_max, cross_max).into(),
            },
            Axis::Vertical => BoxConstraints {
                min: (cross_min, main_min).into(),
                max: (cross_max, main_max).into(),
            },
        }
    }

    // Calculate the [BoxConstraints] for a [Flex::Fixed] child.
    fn fixed_child_constraints(&self, constraints: &BoxConstraints) -> BoxConstraints {
        let (_, main_max) = self.main_axis_constraint(constraints).into();
        let (_, cross_max) = self.cross_axis_constraint(constraints).into();
        let cross_constraint = match self.cross_axis_alignment {
            CrossAxisAlignment::Start | CrossAxisAlignment::End | CrossAxisAlignment::Center => {
                Vector2::new(0.0, cross_max)
            }
            CrossAxisAlignment::Stretch => Vector2::new(cross_max, cross_max),
        };
        let main_constraint = Vector2::new(0.0, main_max);
        self.align_constraints(main_constraint, cross_constraint)
    }

    // Calculate the [BoxConstraints] for a [Flex::Flexible] child.
    fn flex_child_constraints(
        &self,
        constraints: &BoxConstraints,
        flex_size: f32,
    ) -> BoxConstraints {
        let (_, cross_max) = self.cross_axis_constraint(constraints).into();
        let cross_constraint = match self.cross_axis_alignment {
            CrossAxisAlignment::Stretch => Vector2::new(cross_max, cross_max),
            _ => Vector2::new(0.0, cross_max),
        };
        let main_constraint = Vector2::new(flex_size, flex_size);
        self.align_constraints(main_constraint, cross_constraint)
    }

    fn child_main_axis_position(
        &self,
        constraints: &BoxConstraints,
        // The total size of all elements along the main axis
        total_main_axis_size: f32,
        child_main_axis_size: f32,
        // The number of elements in the FlexGroup
        num_widgets: usize,
        // The index of the current element in the FlexGroup
        index: usize,
        // The current cumulative size of all previously laid out elements along
        // the main axis.
        current_main_size: f32,
    ) -> f32 {
        let (_, main_max) = self.main_axis_constraint(constraints).into();
        match self.main_axis_alignment {
            MainAxisAlignment::Start => current_main_size,
            MainAxisAlignment::End => main_max - total_main_axis_size + current_main_size,
            MainAxisAlignment::Center => main_max * 0.5 - child_main_axis_size * 0.5,
            MainAxisAlignment::SpaceEvenly => {
                let space = (main_max - total_main_axis_size) / (num_widgets as f32 + 1.0);
                current_main_size + space
            }
            MainAxisAlignment::SpaceAround => {
                let space = (main_max - total_main_axis_size) / num_widgets as f32;
                if index == 0 || index == num_widgets {
                    current_main_size + (space / 2.0)
                } else {
                    current_main_size + space
                }
            }
            MainAxisAlignment::SpaceBetween => {
                if num_widgets == 1 {
                    return main_max * 0.5 - child_main_axis_size * 0.5;
                }
                if index == 0 {
                    0.0
                } else {
                    let space = (main_max - total_main_axis_size) / (num_widgets as f32 - 1.0);
                    current_main_size + space
                }
            }
        }
    }

    fn child_cross_axis_position(
        &self,
        constraints: &BoxConstraints,
        // The size of the current element along the cross axis
        cross_axis_size: f32,
        // The size of all elements along the cross axis
        max_cross_axis_size: f32,
    ) -> f32 {
        let (_, cross_max) = self.cross_axis_constraint(constraints).into();
        match self.cross_axis_alignment {
            CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => 0.0,
            CrossAxisAlignment::End => cross_max - cross_axis_size,
            CrossAxisAlignment::Center => (max_cross_axis_size * 0.5) - (cross_axis_size * 0.5),
        }
    }

    // Get the size of a child element along the main axis of the [FlexGroup]
    fn child_main_axis_size(&self, sbox: &SizedLayoutBox) -> f32 {
        match self.axis {
            Axis::Horizontal => sbox.size.x,
            Axis::Vertical => sbox.size.y,
        }
    }

    // Get the size of a child element along the cross axis of the [FlexGroup]
    fn child_cross_axis_size(&self, sbox: &SizedLayoutBox) -> f32 {
        match self.axis {
            Axis::Horizontal => sbox.size.y,
            Axis::Vertical => sbox.size.x,
        }
    }
}

/// Flex layout is complex. To help make sure our tests cover all possible
/// cases, this test module uses the following format:
///
/// 1. Feature section header
/// 2. "flex_group_vertical_{parameter name}_with_fixed_child
/// 3. "flex_group_horizontal_{parameter name}_with_fixed_child
/// 4. "flex_group_vertical_{parameter name}_with_three_fixed_children
/// 5. "flex_group_horizontal_{parameter name}_with_three_fixed_children
/// 6. "flex_group_vertical_{parameter name}_with_flex_child
/// 7. "flex_group_horizontal_{parameter name}_with_flex_child
/// 8. "flex_group_vertical_{parameter name}_with_three_flex_children
/// 9. "flex_group_horizontal_{parameter name}_with_three_flex_children
///
/// In other words, alternative between the following units under test:
///
/// 1. Main axis direction (vertical or horizontal)
/// 2. Child count (one or multiple, make sure to check order)
/// 3. Fixed or flexible children
///
/// This is an obscene number of tests. Implementing a fully-featured flexbox
/// layout made we want to knock myself out, especially since I'm normally
/// programming at 1am and I'm good at thinking at that time. Instead, these
/// tests make it almost impossible for me to write an (undetected) bug.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{widget, Color};
    use math::{Rect, Vector2};
    use test_util::assert_slice_eq;

    // --------------------------------------------------
    // Flex child
    // --------------------------------------------------

    #[test]
    fn flex_with_zero_flex_is_treated_like_fixed_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Min,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                Flex::Flexible {
                    flex: 0.0,
                    child: Box::new(widget::Rect {
                        size: (10.0, 10.0).into(),
                        color: Color::green(),
                    }),
                },
                Flex::Flexible {
                    flex: 0.0,
                    child: Box::new(widget::Rect {
                        size: (10.0, 10.0).into(),
                        color: Color::blue(),
                    }),
                },
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..flex_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 10.0), (10.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 20.0)),
                children: vec![0, 1],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    // --------------------------------------------------
    // Main axis size
    // --------------------------------------------------

    #[test]
    fn flex_group_vertical_main_axis_size_min_with_fixed_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Min,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            fixed_child_lbox(Color::green()),
            LayoutBox {
                rect: Rect::from_size((10.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_size_min_with_fixed_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Min,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            fixed_child_lbox(Color::green()),
            LayoutBox {
                rect: Rect::from_size((10.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_size_min_with_three_fixed_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Min,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 10.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 20.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 30.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_size_min_with_three_fixed_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Min,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((10.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((20.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((30.0, 10.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_size_min_with_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Min,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_size_min_with_flex_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Min,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_size_min_with_three_flex_child_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Min,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_size_min_with_three_flex_child_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Min,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((size, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((size * 2.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_size_max_with_fixed_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_size((10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_size_max_with_fixed_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_size((10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_size_max_with_three_fixed_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 10.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 20.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_size_max_with_three_fixed_children() {
        let column = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((10.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((20.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_size_max_with_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_size_max_with_flex_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_size_max_with_three_flex_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_size_max_with_three_flex_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((size, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((size * 2.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    // --------------------------------------------------
    // Main axis alignment
    // --------------------------------------------------

    #[test]
    fn flex_group_vertical_main_axis_alignment_start_with_fixed_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_size((10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_start_with_fixed_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_size((10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_start_with_three_fixed_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 10.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 20.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_start_with_three_fixed_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((10.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((20.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_start_with_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_start_with_flex_child() {
        let column = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_start_with_three_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_start_with_three_flex_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((size, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((size * 2.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_end_with_fixed_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::End,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 90.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_end_with_fixed_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::End,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((90.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_end_with_three_fixed_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::End,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                Flex::Fixed {
                    child: Box::new(widget::Rect {
                        size: (10.0, 10.0).into(),
                        color: Color::red(),
                    }),
                },
                Flex::Fixed {
                    child: Box::new(widget::Rect {
                        size: (10.0, 10.0).into(),
                        color: Color::green(),
                    }),
                },
                Flex::Fixed {
                    child: Box::new(widget::Rect {
                        size: (10.0, 10.0).into(),
                        color: Color::blue(),
                    }),
                },
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 70.0), (10.0, 10.0)),
                material: Material::Solid(Color::red()),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 80.0), (10.0, 10.0)),
                material: Material::Solid(Color::green()),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 90.0), (10.0, 10.0)),
                material: Material::Solid(Color::blue()),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_end_with_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::End,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_end_with_flex_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::End,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_center_with_fixed_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Center,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 45.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_center_with_fixed_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Center,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((45.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_center_with_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Center,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_center_with_flex_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Center,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_space_between_with_fixed_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceBetween,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 45.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_space_between_with_fixed_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceBetween,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((45.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_space_between_with_three_fixed_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceBetween,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 45.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 90.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_space_between_with_three_fixed_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceBetween,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((45.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((90.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_space_between_with_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceBetween,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_space_between_with_flex_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceBetween,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_space_between_with_three_flex_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceBetween,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_space_between_with_three_flex_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceBetween,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((size, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((size * 2.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_space_around_with_fixed_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceAround,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 45.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_space_around_with_fixed_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceAround,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((45.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_space_around_with_three_fixed_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceAround,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let spacing = (100.0 - 30.0) / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, spacing * 0.5), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 10.0 + spacing * 1.5), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 20.0 + spacing * 2.5), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_space_around_with_three_fixed_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceAround,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let spacing = (100.0 - 30.0) / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((spacing * 0.5, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((10.0 + spacing * 1.5, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((20.0 + spacing * 2.5, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_space_around_with_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceAround,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_space_around_with_flex_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceAround,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_space_around_with_three_flex_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceAround,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_space_around_with_three_flex_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceAround,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((size, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((size * 2.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_space_evenly_with_fixed_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceEvenly,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 45.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_space_evenly_with_fixed_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceEvenly,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((45.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_space_evenly_with_three_fixed_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceEvenly,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let spacing = (100.0 - 30.0) / 4.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, spacing), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 10.0 + spacing * 2.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 20.0 + spacing * 3.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_space_evenly_with_three_fixed_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceEvenly,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let spacing = (100.0 - 30.0) / 4.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((spacing, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((10.0 + spacing * 2.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((20.0 + spacing * 3.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_main_axis_alignment_space_evenly_with_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceEvenly,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_main_axis_alignment_space_evenly_with_flex_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceEvenly,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    // --------------------------------------------------
    // Cross axis alignment
    // --------------------------------------------------

    #[test]
    fn flex_group_vertical_cross_axis_alignment_start_with_fixed_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_cross_axis_alignment_start_with_fixed_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_cross_axis_alignment_start_with_three_fixed_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 10.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 20.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_cross_axis_alignment_start_with_three_fixed_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::green()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((10.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((20.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_cross_axis_alignment_start_with_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_cross_axis_alignment_start_with_flex_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_cross_axis_alignment_start_with_three_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_cross_axis_alignment_start_with_three_flex_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((size, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_pos((size * 2.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_cross_axis_alignment_end_with_fixed_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::End,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((90.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_cross_axis_alignment_end_with_fixed_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::End,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 90.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_cross_axis_alignment_end_with_three_fixed_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::End,
            children: vec![
                create_fixed_child(Color::red()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((90.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::red())
            },
            LayoutBox {
                rect: Rect::from_pos((90.0, 10.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((90.0, 20.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_cross_axis_alignment_end_with_three_fixed_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::End,
            children: vec![
                create_fixed_child(Color::red()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 90.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::red())
            },
            LayoutBox {
                rect: Rect::from_pos((10.0, 90.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((20.0, 90.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_cross_axis_alignment_end_with_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::End,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((90.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_cross_axis_alignment_end_with_flex_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::End,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 90.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_cross_axis_alignment_end_with_three_flex_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::End,
            children: vec![
                create_flex_child(Color::red()),
                create_flex_child(Color::green()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((90.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::red())
            },
            LayoutBox {
                rect: Rect::from_pos((90.0, size), (10.0, size)),
                ..flex_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((90.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_cross_axis_alignment_end_with_three_flex_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::End,
            children: vec![
                create_flex_child(Color::red()),
                create_flex_child(Color::green()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 90.0), (size, 10.0)),
                ..flex_child_lbox(Color::red())
            },
            LayoutBox {
                rect: Rect::from_pos((size, 90.0), (size, 10.0)),
                ..flex_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((size * 2.0, 90.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_cross_axis_alignment_stretch_with_fixed_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Stretch,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_cross_axis_alignment_stretch_with_fixed_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Stretch,
            children: vec![create_fixed_child(Color::green())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_cross_axis_alignment_stretch_with_three_fixed_children() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Stretch,
            children: vec![
                create_fixed_child(Color::red()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..fixed_child_lbox(Color::red())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 10.0), (100.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, 20.0), (100.0, 10.0)),
                ..fixed_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_cross_axis_alignment_stretch_with_three_fixed_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Stretch,
            children: vec![
                create_fixed_child(Color::red()),
                create_fixed_child(Color::green()),
                create_fixed_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..fixed_child_lbox(Color::red())
            },
            LayoutBox {
                rect: Rect::from_pos((10.0, 0.0), (10.0, 100.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((20.0, 0.0), (10.0, 100.0)),
                ..fixed_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_cross_axis_alignment_stretch_with_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Stretch,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (100.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_cross_axis_alignment_stretch_with_flex_child() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Stretch,
            children: vec![create_flex_child(Color::blue())],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (100.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_vertical_cross_axis_alignment_stretch_with_three_flex_child() {
        let column = FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Stretch,
            children: vec![
                create_flex_child(Color::red()),
                create_flex_child(Color::green()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (100.0, size)),
                ..flex_child_lbox(Color::red())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size), (100.0, size)),
                ..flex_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((0.0, size * 2.0), (100.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex_group_horizontal_cross_axis_alignment_stretch_with_three_flex_children() {
        let row = FlexGroup {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Stretch,
            children: vec![
                create_flex_child(Color::red()),
                create_flex_child(Color::green()),
                create_flex_child(Color::blue()),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&row, &constraints);
        let size = 100.0 / 3.0;
        let expected_layout = vec![
            LayoutBox {
                rect: Rect::from_pos((0.0, 0.0), (size, 100.0)),
                ..flex_child_lbox(Color::red())
            },
            LayoutBox {
                rect: Rect::from_pos((size, 0.0), (size, 100.0)),
                ..flex_child_lbox(Color::green())
            },
            LayoutBox {
                rect: Rect::from_pos((size * 2.0, 0.0), (size, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                rect: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: Material::None,
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

    fn create_fixed_child(color: Color) -> Flex {
        Flex::Fixed {
            child: Box::new(widget::Rect {
                size: (10.0, 10.0).into(),
                color,
            }),
        }
    }

    fn fixed_child_lbox(color: Color) -> LayoutBox {
        LayoutBox {
            rect: Rect::from_size((10.0, 10.0)),
            children: vec![],
            material: Material::Solid(color),
        }
    }

    fn create_flex_child(color: Color) -> Flex {
        Flex::Flexible {
            flex: 1.0,
            child: Box::new(widget::Rect {
                size: (10.0, 10.0).into(),
                color,
            }),
        }
    }

    fn flex_child_lbox(color: Color) -> LayoutBox {
        LayoutBox {
            rect: Rect::from_size((10.0, 10.0)),
            children: vec![],
            material: Material::Solid(color),
        }
    }
}
