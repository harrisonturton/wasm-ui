use super::{BoxConstraints, Color, Layout, LayoutBox, LayoutTree, Material, SizedLayoutBox};
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
                let constraints = self.flex_child_constraints(constraints, main_axis_size);
                let sbox = child.0.layout(tree, &constraints);
                let cross_size = self.child_cross_axis_size(&sbox);
                max_cross_size = f32::max(max_cross_size, cross_size);
                child.1 = Some(sbox);
                total_size += main_axis_size;
            }
        }

        // Finally, we can determine their positions
        let mut children = vec![];
        let mut current_main_size = 0.0;

        let sboxes = sbox_cache.iter().filter_map(|(_, sbox)| sbox.as_ref());
        for (i, sbox) in sboxes.enumerate() {
            let cross_size = self.child_cross_axis_size(sbox);
            let cross_pos = self.child_cross_axis_position(constraints, cross_size, max_cross_size);
            let main_pos = self.child_main_axis_position(
                constraints,
                total_size,
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
            let main_size = self.child_main_axis_size(sbox);
            current_main_size += (main_pos - current_main_size) + main_size;
        }

        let cross_size = max_cross_size;
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
            material: Material::Solid(Color::blue()),
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

    // Calculate the [BoxConstraints] for a [Flex::Fixed] child.
    fn fixed_child_constraints(&self, constraints: &BoxConstraints) -> BoxConstraints {
        let (_, main_max) = self.main_axis_constraint(constraints).into();
        let (_, cross_max) = self.cross_axis_constraint(constraints).into();
        match self.cross_axis_alignment {
            CrossAxisAlignment::Start | CrossAxisAlignment::End | CrossAxisAlignment::Center => {
                BoxConstraints {
                    min: (0.0, 0.0).into(),
                    max: (cross_max, main_max).into(),
                }
            }
            CrossAxisAlignment::Stretch => BoxConstraints {
                min: (cross_max, 0.0).into(),
                max: (cross_max, main_max).into(),
            },
        }
    }

    // Calculate the [BoxConstraints] for a [Flex::Flexible] child.
    fn flex_child_constraints(
        &self,
        constraints: &BoxConstraints,
        flex_size: f32,
    ) -> BoxConstraints {
        let (_, cross_max) = self.cross_axis_constraint(constraints).into();
        match self.cross_axis_alignment {
            CrossAxisAlignment::Start | CrossAxisAlignment::End | CrossAxisAlignment::Center => {
                BoxConstraints {
                    min: (0.0, flex_size).into(),
                    max: (cross_max, flex_size).into(),
                }
            }
            CrossAxisAlignment::Stretch => BoxConstraints {
                min: (cross_max, flex_size).into(),
                max: (cross_max, flex_size).into(),
            },
        }
    }

    fn child_main_axis_position(
        &self,
        constraints: &BoxConstraints,
        // The total size of all elements along the main axis
        total_main_axis_size: f32,
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
                if index == 0 {
                    current_main_size
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
            CrossAxisAlignment::End => max_cross_axis_size - cross_axis_size,
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
