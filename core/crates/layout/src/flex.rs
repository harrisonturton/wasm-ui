use super::{BoxConstraints, Container, Layout, LayoutBox, LayoutTree, Material, SizedLayoutBox};
use math::Vector2;
use std::collections::VecDeque;
use std::fmt::Debug;

// What diboundsion the flex container should face.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Axis {
    // Place items in a row. The main axis is the `x` axis, and the cross axis
    // is the `y` axis.
    Horizontal,
    // Place items in a column. The main axis is the `y` axis, and the cross
    // axis is the `x` axis.
    Vertical,
}

impl Default for Axis {
    fn default() -> Axis {
        Axis::Vertical
    }
}

// How the children of a flex container should be aligned along the main axis.
// For a vertical container, this is their vertical position. For a horizontal
// container, this is their horizontal position.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
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

// How the children of a flex container should be aligned along the cross axis.
// For a vertical container, this is their horizontal position. For a horizontal
// container, this is their vertical position.
#[derive(Copy, Clone, Debug)]
pub enum CrossAxisAlignment {
    // Push children to the start of the container
    Start,
    // Push children to the end of the container
    End,
    // Stretch children to fill the container
    Stretch,
    // Position children in the center
    Center,
}

impl Default for CrossAxisAlignment {
    fn default() -> CrossAxisAlignment {
        CrossAxisAlignment::Start
    }
}

// How large the flex widget should be along the main axis.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum MainAxisSize {
    // Stretch to fill container
    Max,
    // Shrink to fit children
    Min,
}

impl Default for MainAxisSize {
    fn default() -> MainAxisSize {
        MainAxisSize::Min
    }
}

// This trait enables a child to provide it's flex factor to the parent widget
// so the parent can calculate the corbounds `BoxConstraint`. If the flex factor
// is `None`, then it is not a flexible widget.
//
// This is implemented as a trait so that it can be impl'd as a metatrait on
// `Box<dyn Layout>`, which enables `Layout` types to be used as children of
// flex containers that only accept `FlexLayout` children.
pub trait FlexLayout: Debug {
    // The relative area that this flexible widget should grow to, relative to
    // other flexible widgets.
    fn flex_factor(&self) -> Option<f32>;

    fn flex_layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox;
}

// All existing widgets have a flex factor of 0, meaning they are not flexible,
// and have a fixed size.
impl<T> FlexLayout for T
where
    T: Layout,
{
    fn flex_factor(&self) -> Option<f32> {
        None
    }

    fn flex_layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        self.layout(tree, constraints)
    }
}

// `Flexible` is used to provide the flex factor to the flex container in order for
// it to calculate the corbounds `BoxConstraints` for the flexible child.
#[derive(Debug)]
pub struct Flexible {
    pub flex_factor: f32,
    pub child: Box<dyn Layout>,
}

impl FlexLayout for Flexible {
    fn flex_factor(&self) -> Option<f32> {
        Some(self.flex_factor)
    }

    fn flex_layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        self.child.layout(tree, constraints)
    }
}

// A container that sizes and positions its children like CSS flexbox.
#[derive(Debug)]
pub struct Flex {
    pub axis: Axis,
    pub main_axis_size: MainAxisSize,
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
    pub children: Vec<Box<dyn FlexLayout>>,
}

impl Default for Flex {
    fn default() -> Flex {
        Flex {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![],
        }
    }
}

impl Layout for Flex {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let (main_min, main_max) = self.main_axis_constraint(constraints);
        let (_, cross_max) = self.cross_axis_constraint(constraints);

        let mut total_main_size = 0.0;
        let mut total_cross_size = 0.0;

        // Keep track of the children that have already undergone layout.
        // `VecDeque` because we need `pop_front` later.
        let mut layout_cache: VecDeque<(&Box<dyn FlexLayout>, Option<SizedLayoutBox>)> =
            VecDeque::new();

        // Do two passes over the children. The first pass calculates the total
        // main axis size of the fixed-size children. We must do this to
        // calculate how much space is left over for the flexible children to
        // grow into.
        let mut sum_inflexible_size = 0.0;
        let mut sum_flex_factor = 0.0;
        for child in &self.children {
            // Skip layout for flex children, but keep track of them for later
            if let Some(flex_factor) = child.flex_factor() {
                // Treat 0-flex widgets as inflexible
                if flex_factor > 0.0 {
                    sum_flex_factor += flex_factor;
                    layout_cache.push_back((child, None));
                    continue;
                }
            }

            let constraints = self.inflexible_child_constraints(constraints);
            let sbox = child.flex_layout(tree, &constraints);
            let size = sbox.size;
            layout_cache.push_back((child, Some(sbox)));

            let main_size = self.main_axis_size(size);
            let cross_size = self.cross_axis_size(size);
            sum_inflexible_size += main_size;
            total_main_size += main_size;
            total_cross_size = f32::max(cross_size, total_cross_size);
        }

        // Now we can determine the relative sizing of the flex widgets
        let free_space = main_max - sum_inflexible_size;
        let space_per_flex_factor = free_space / sum_flex_factor;

        // The second pass will layout all flexible children.
        for (child, maybe_sbox) in &mut layout_cache {
            let flex_factor = match child.flex_factor() {
                // 0-flex widgets have already been treated as inflexible, and
                // laid out in the first pass, so we can skip them here.
                Some(flex_factor) if flex_factor > 0.0 => flex_factor,
                _ => continue,
            };

            let main_axis_size = flex_factor * space_per_flex_factor;
            let constraints = self.flex_child_constraints(constraints, main_axis_size);
            let sbox = child.flex_layout(tree, &constraints);
            let size = sbox.size;
            *maybe_sbox = Some(sbox);

            let main_size = self.main_axis_size(size);
            total_main_size += main_size;
            let cross_size = self.cross_axis_size(size);
            total_cross_size = f32::max(cross_size, total_cross_size);
        }

        // We now have enough information to position the children
        let num_children = layout_cache.len();
        let mut children = vec![];
        let mut current_total_main_size = 0.0;

        let mut i = 0;
        while let Some((_, Some(sbox))) = layout_cache.pop_front() {
            let cross_size = self.cross_axis_size(sbox.size);
            let main_size = self.main_axis_size(sbox.size);

            let cross_pos =
                self.child_cross_axis_position(constraints, cross_size, total_cross_size);
            let main_pos = self.child_main_axis_position(
                constraints,
                total_main_size,
                current_total_main_size,
                main_size,
                num_children,
                i,
            );
            let pos = self.align_to_axis(main_pos, cross_pos);
            let lbox = LayoutBox::from_child(sbox, pos);
            let id = tree.insert(lbox);
            children.push(id);
            current_total_main_size += match self.main_axis_alignment {
                MainAxisAlignment::End => main_size,
                _ => main_pos + main_size - current_total_main_size,
            };
            i += 1;
        }

        let cross_size = match self.cross_axis_alignment {
            CrossAxisAlignment::Start => total_cross_size,
            _ => cross_max,
        };
        let main_size = match self.main_axis_size {
            MainAxisSize::Min => total_main_size.clamp(main_min, main_max),
            MainAxisSize::Max => main_max,
        };
        let size = self.align_to_axis(main_size, cross_size);
        SizedLayoutBox {
            size,
            children,
            material: None,
            ..SizedLayoutBox::default()
        }
    }
}

impl Flex {
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    fn child_main_axis_position(
        &self,
        constraints: &BoxConstraints,
        total_main_axis_size: f32,
        current_total_main_axis_size: f32,
        child_main_axis_size: f32,
        num_children: usize,
        index: u32,
    ) -> f32 {
        let (_, main_max) = self.main_axis_constraint(constraints);
        match self.main_axis_alignment {
            MainAxisAlignment::Start => current_total_main_axis_size,
            MainAxisAlignment::End => {
                main_max - total_main_axis_size + current_total_main_axis_size
            }
            MainAxisAlignment::Center => (main_max * 0.5) - (child_main_axis_size * 0.5),
            MainAxisAlignment::SpaceEvenly => {
                let spacing = (main_max - total_main_axis_size) / (num_children as f32 + 1.0);
                current_total_main_axis_size + spacing
            }
            MainAxisAlignment::SpaceAround => {
                let space = (main_max - total_main_axis_size) / num_children as f32;
                if index == 0 || index == num_children as u32 {
                    current_total_main_axis_size + (space / 2.0)
                } else {
                    current_total_main_axis_size + space
                }
            }
            MainAxisAlignment::SpaceBetween => {
                if num_children == 1 {
                    (main_max * 0.5) - (child_main_axis_size * 0.5)
                } else if index == 0 {
                    0.0
                } else {
                    let spacing = (main_max - total_main_axis_size) / (num_children as f32 - 1.0);
                    current_total_main_axis_size + spacing
                }
            }
        }
    }

    fn child_cross_axis_position(
        &self,
        constraints: &BoxConstraints,
        child_cross_axis_size: f32,
        total_cross_axis_size: f32,
    ) -> f32 {
        let (_, cross_max) = self.cross_axis_constraint(constraints);
        match self.cross_axis_alignment {
            CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => 0.0,
            CrossAxisAlignment::End => cross_max - child_cross_axis_size,
            CrossAxisAlignment::Center => (cross_max * 0.5) - (child_cross_axis_size * 0.5),
        }
    }

    // Calculate the `BoxConstraints` for a fixed-size child.
    fn inflexible_child_constraints(&self, parent_constraints: &BoxConstraints) -> BoxConstraints {
        let (_, main_max) = self.main_axis_constraint(parent_constraints);
        let (_, cross_max) = self.cross_axis_constraint(parent_constraints);
        let main_axis_constraint = Vector2::new(0.0, main_max);
        let cross_axis_constraint = match self.cross_axis_alignment {
            CrossAxisAlignment::Stretch => Vector2::new(cross_max, cross_max),
            _ => Vector2::new(0.0, cross_max),
        };
        self.align_constraints(main_axis_constraint, cross_axis_constraint)
    }

    // Calculate the `BoxConstraints` for a flexible child.
    fn flex_child_constraints(
        &self,
        constraints: &BoxConstraints,
        main_axis_size: f32,
    ) -> BoxConstraints {
        let (_, cross_max) = self.cross_axis_constraint(constraints);
        let main_axis_constraint = Vector2::new(main_axis_size, main_axis_size);
        let cross_axis_constraint = match self.cross_axis_alignment {
            CrossAxisAlignment::Stretch => Vector2::new(cross_max, cross_max),
            _ => Vector2::new(0.0, cross_max),
        };
        self.align_constraints(main_axis_constraint, cross_axis_constraint)
    }

    // Get the minimum and maximum size of a constraint along the main axis.
    fn main_axis_constraint(&self, constraints: &BoxConstraints) -> (f32, f32) {
        let BoxConstraints { min, max } = constraints;
        match self.axis {
            Axis::Horizontal => (min.x, max.x),
            Axis::Vertical => (min.y, max.y),
        }
    }

    // Get the minimum and maximum size of a constraint along the cross axis.
    fn cross_axis_constraint(&self, constraints: &BoxConstraints) -> (f32, f32) {
        let BoxConstraints { min, max } = constraints;
        match self.axis {
            Axis::Horizontal => (min.y, max.y),
            Axis::Vertical => (min.x, max.x),
        }
    }

    // Create a `BoxConstraints` that is aligned along the same axis as the
    // `Flex` parent.
    fn align_constraints(
        &self,
        main_constraint: Vector2,
        cross_constraint: Vector2,
    ) -> BoxConstraints {
        let (main_min, main_max) = main_constraint.into();
        let (cross_min, cross_max) = cross_constraint.into();
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

    // Get a 2D coordinate from positions relative to the main and cross axes.
    fn align_to_axis(&self, main_pos: f32, cross_pos: f32) -> Vector2 {
        match self.axis {
            Axis::Horizontal => Vector2::new(main_pos, cross_pos),
            Axis::Vertical => Vector2::new(cross_pos, main_pos),
        }
    }

    // Get the size along the main axis.
    fn main_axis_size(&self, size: Vector2) -> f32 {
        match self.axis {
            Axis::Horizontal => size.x,
            Axis::Vertical => size.y,
        }
    }

    // Get the size along the cross axis.
    fn cross_axis_size(&self, size: Vector2) -> f32 {
        match self.axis {
            Axis::Horizontal => size.y,
            Axis::Vertical => size.x,
        }
    }
}

/// Flex layout is complex. To help make sure our tests cover all possible
/// cases, this test module uses the following format:
///
/// 1. Feature section header
/// 2. `flex2_vertical_{parameter name}_with_fixed_child`
/// 3. `flex2_horizontal_{parameter name}_with_fixed_child`
/// 4. `flex2_vertical_{parameter name}_with_three_fixed_children`
/// 5. `flex2_horizontal_{parameter name}_with_three_fixed_children`
/// 6. `flex2_vertical_{parameter name}_with_flex_child`
/// 7. `flex2_horizontal_{parameter name}_with_flex_child`
/// 8. `flex2_vertical_{parameter name}_with_three_flex_children`
/// 9. `flex2_horizontal_{parameter name}_with_three_flex_children`
///
/// In other words, alternative between the following units under test:
///
/// 1. Main axis diboundsion (vertical or horizontal)
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
    use crate::container;
    use crate::decoration::Color;
    use math::{Rect, Vector2};
    use test_util::assert_slice_eq;

    // --------------------------------------------------
    // Flexible
    // --------------------------------------------------

    #[test]
    fn flex_with_zero_flex_is_treated_like_fixed_child() {
        let column = Flex {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Min,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            children: vec![
                Box::new(Flexible {
                    flex_factor: 0.0,
                    child: Box::new(Container {
                        width: Some(10.0),
                        height: Some(10.0),
                        color: Color::green(),
                        ..Container::default()
                    }),
                }),
                Box::new(Flexible {
                    flex_factor: 0.0,
                    child: Box::new(Container {
                        width: Some(10.0),
                        height: Some(10.0),
                        color: Color::blue(),
                        ..Container::default()
                    }),
                }),
            ],
        };

        let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
        let actual_layout = layout_with_constraints(&column, &constraints);
        let expected_layout = vec![
            LayoutBox {
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..flex_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 10.0), (10.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 20.0)),
                children: vec![0, 1],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    // --------------------------------------------------
    // Main axis size
    // --------------------------------------------------

    #[test]
    fn flex2_vertical_main_axis_size_min_with_fixed_child() {
        let column = Flex {
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
                bounds: Rect::from_size((10.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_size_min_with_fixed_child() {
        let row = Flex {
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
                bounds: Rect::from_size((10.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_size_min_with_three_fixed_children() {
        let column = Flex {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Min,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::red())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 10.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 20.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 30.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_size_min_with_three_fixed_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((10.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((20.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((30.0, 10.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_size_min_with_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_size((10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_size_min_with_flex_child() {
        let row = Flex {
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
                bounds: Rect::from_size((100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_size_min_with_three_flex_child_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_size_min_with_three_flex_child_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((size, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((size * 2.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_size_max_with_fixed_child() {
        let column = Flex {
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
                bounds: Rect::from_size((10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_size_max_with_fixed_child() {
        let row = Flex {
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
                bounds: Rect::from_size((10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_size_max_with_three_fixed_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 10.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 20.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_size_max_with_three_fixed_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((10.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((20.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_size_max_with_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_size((10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_size_max_with_flex_child() {
        let row = Flex {
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
                bounds: Rect::from_size((100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_size_max_with_three_flex_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_size_max_with_three_flex_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((size, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((size * 2.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    // --------------------------------------------------
    // Main axis alignment
    // --------------------------------------------------

    #[test]
    fn flex2_vertical_main_axis_alignment_start_with_fixed_child() {
        let column = Flex {
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
                bounds: Rect::from_size((10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_start_with_fixed_child() {
        let row = Flex {
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
                bounds: Rect::from_size((10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_start_with_three_fixed_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 10.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 20.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_start_with_three_fixed_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((10.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((20.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_start_with_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_size((10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_start_with_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_size((100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_start_with_three_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_start_with_three_flex_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((size, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((size * 2.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_end_with_fixed_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 90.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_end_with_fixed_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((90.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_end_with_three_fixed_children() {
        let column = Flex {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::End,
            cross_axis_alignment: CrossAxisAlignment::Start,
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
                bounds: Rect::from_pos((0.0, 70.0), (10.0, 10.0)),
                material: Some(Material::filled(Color::red())),
                ..fixed_child_lbox(Color::red())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 80.0), (10.0, 10.0)),
                material: Some(Material::filled(Color::green())),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 90.0), (10.0, 10.0)),
                material: Some(Material::filled(Color::blue())),
                ..fixed_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_end_with_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_end_with_flex_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_center_with_fixed_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 45.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_center_with_fixed_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((45.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_center_with_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_center_with_flex_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_space_between_with_fixed_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 45.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_space_between_with_fixed_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((45.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_space_between_with_three_fixed_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 45.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 90.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_space_between_with_three_fixed_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((45.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((90.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_space_between_with_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_space_between_with_flex_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_space_between_with_three_flex_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_space_between_with_three_flex_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((size, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((size * 2.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_space_around_with_fixed_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 45.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_space_around_with_fixed_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((45.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_space_around_with_three_fixed_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, spacing * 0.5), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 10.0 + spacing * 1.5), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 20.0 + spacing * 2.5), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_space_around_with_three_fixed_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((spacing * 0.5, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((10.0 + spacing * 1.5, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((20.0 + spacing * 2.5, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_space_around_with_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_space_around_with_flex_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_space_around_with_three_flex_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_space_around_with_three_flex_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((size, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((size * 2.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_space_evenly_with_fixed_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 45.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_space_evenly_with_fixed_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((45.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_space_evenly_with_three_fixed_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, spacing), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 10.0 + spacing * 2.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 20.0 + spacing * 3.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_space_evenly_with_three_fixed_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((spacing, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((10.0 + spacing * 2.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((20.0 + spacing * 3.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_main_axis_alignment_space_evenly_with_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_main_axis_alignment_space_evenly_with_flex_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    // --------------------------------------------------
    // Cross axis alignment
    // --------------------------------------------------

    #[test]
    fn flex2_vertical_cross_axis_alignment_start_with_fixed_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_cross_axis_alignment_start_with_fixed_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_cross_axis_alignment_start_with_three_fixed_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 10.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 20.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_cross_axis_alignment_start_with_three_fixed_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((10.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((20.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_cross_axis_alignment_start_with_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_cross_axis_alignment_start_with_flex_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_cross_axis_alignment_start_with_three_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((10.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_cross_axis_alignment_start_with_three_flex_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((size, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_pos((size * 2.0, 0.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 10.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_cross_axis_alignment_end_with_fixed_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((90.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_cross_axis_alignment_end_with_fixed_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 90.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_cross_axis_alignment_end_with_three_fixed_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((90.0, 0.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::red())
            },
            LayoutBox {
                bounds: Rect::from_pos((90.0, 10.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((90.0, 20.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_cross_axis_alignment_end_with_three_fixed_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 90.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::red())
            },
            LayoutBox {
                bounds: Rect::from_pos((10.0, 90.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((20.0, 90.0), (10.0, 10.0)),
                ..fixed_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_cross_axis_alignment_end_with_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((90.0, 0.0), (10.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_cross_axis_alignment_end_with_flex_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 90.0), (100.0, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_cross_axis_alignment_end_with_three_flex_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((90.0, 0.0), (10.0, size)),
                ..flex_child_lbox(Color::red())
            },
            LayoutBox {
                bounds: Rect::from_pos((90.0, size), (10.0, size)),
                ..flex_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((90.0, size * 2.0), (10.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_cross_axis_alignment_end_with_three_flex_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 90.0), (size, 10.0)),
                ..flex_child_lbox(Color::red())
            },
            LayoutBox {
                bounds: Rect::from_pos((size, 90.0), (size, 10.0)),
                ..flex_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((size * 2.0, 90.0), (size, 10.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_cross_axis_alignment_stretch_with_fixed_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_cross_axis_alignment_stretch_with_fixed_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_cross_axis_alignment_stretch_with_three_fixed_children() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (100.0, 10.0)),
                ..fixed_child_lbox(Color::red())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 10.0), (100.0, 10.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, 20.0), (100.0, 10.0)),
                ..fixed_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_cross_axis_alignment_stretch_with_three_fixed_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (10.0, 100.0)),
                ..fixed_child_lbox(Color::red())
            },
            LayoutBox {
                bounds: Rect::from_pos((10.0, 0.0), (10.0, 100.0)),
                ..fixed_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((20.0, 0.0), (10.0, 100.0)),
                ..fixed_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_cross_axis_alignment_stretch_with_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (100.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_cross_axis_alignment_stretch_with_flex_child() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (100.0, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_vertical_cross_axis_alignment_stretch_with_three_flex_child() {
        let column = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (100.0, size)),
                ..flex_child_lbox(Color::red())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size), (100.0, size)),
                ..flex_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((0.0, size * 2.0), (100.0, size)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
            },
        ];
        assert_slice_eq(&expected_layout, &actual_layout);
    }

    #[test]
    fn flex2_horizontal_cross_axis_alignment_stretch_with_three_flex_children() {
        let row = Flex {
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
                bounds: Rect::from_pos((0.0, 0.0), (size, 100.0)),
                ..flex_child_lbox(Color::red())
            },
            LayoutBox {
                bounds: Rect::from_pos((size, 0.0), (size, 100.0)),
                ..flex_child_lbox(Color::green())
            },
            LayoutBox {
                bounds: Rect::from_pos((size * 2.0, 0.0), (size, 100.0)),
                ..flex_child_lbox(Color::blue())
            },
            LayoutBox {
                bounds: Rect::from_size((100.0, 100.0)),
                children: vec![0, 1, 2],
                material: None,
                ..LayoutBox::default()
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

    fn create_fixed_child(color: Color) -> Box<Container> {
        Box::new(Container {
            width: Some(10.0),
            height: Some(10.0),
            color,
            ..Container::default()
        })
    }

    fn fixed_child_lbox(color: Color) -> LayoutBox {
        LayoutBox {
            bounds: Rect::from_size((10.0, 10.0)),
            children: vec![],
            material: Some(Material::filled(color)),
            ..LayoutBox::default()
        }
    }

    fn create_flex_child(color: Color) -> Box<Flexible> {
        Box::new(Flexible {
            flex_factor: 1.0,
            child: Box::new(Container {
                width: Some(10.0),
                height: Some(10.0),
                color,
                ..Container::default()
            }),
        })
    }

    fn flex_child_lbox(color: Color) -> LayoutBox {
        LayoutBox {
            bounds: Rect::from_size((10.0, 10.0)),
            children: vec![],
            material: Some(Material::filled(color)),
            ..LayoutBox::default()
        }
    }
}
