#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::similar_names)]

use layout::{
    Axis, BoxConstraints, Color, Container, CrossAxisAlignment, EdgeInsets, Flex, Flexible, Layout,
    LayoutBox, LayoutTree, MainAxisAlignment, MainAxisSize, Material,
};
use math::{Rect, Vector2};
use test_util::assert_slice_eq;

#[test]
pub fn canary_layouts_flex_sidebar() {
    let widgets = Flex {
        axis: Axis::Horizontal,
        main_axis_size: MainAxisSize::Max,
        main_axis_alignment: MainAxisAlignment::Start,
        cross_axis_alignment: CrossAxisAlignment::Stretch,
        children: vec![
            Box::new(Container {
                width: Some(f32::INFINITY),
                height: Some(50.0),
                color: Color::green(),
                padding: EdgeInsets::all(10.0),
                child: Some(Box::new(Flex {
                    axis: Axis::Vertical,
                    main_axis_size: MainAxisSize::Max,
                    main_axis_alignment: MainAxisAlignment::Start,
                    cross_axis_alignment: CrossAxisAlignment::Stretch,
                    children: vec![
                        Box::new(Container {
                            width: Some(f32::INFINITY),
                            height: Some(25.0),
                            margin: EdgeInsets::bottom(15.0),
                            color: Color::red(),
                            ..Container::default()
                        }),
                        Box::new(Container {
                            width: Some(f32::INFINITY),
                            height: Some(25.0),
                            margin: EdgeInsets::bottom(15.0),
                            color: Color::red(),
                            ..Container::default()
                        }),
                    ],
                })),
                ..Container::default()
            }),
            Box::new(Flexible {
                flex_factor: 1.0,
                child: Box::new(Container {
                    color: Color::blue(),
                    ..Container::default()
                }),
            }),
        ],
    };

    let constraints = BoxConstraints::from_max(Vector2::new(100.0, 100.0));
    let actual_layout = layout_with_constraints(&widgets, &constraints);
    let expected_layout = vec![
        LayoutBox {
            bounds: Rect::from_pos((0.0, 0.0), (30.0, 40.0)),
            margin: EdgeInsets::bottom(15.0),
            children: vec![],
            material: Some(Material::filled(Color::red())),
        },
        LayoutBox {
            bounds: Rect::from_pos((0.0, 40.0), (30.0, 40.0)),
            margin: EdgeInsets::bottom(15.0),
            children: vec![],
            material: Some(Material::filled(Color::red())),
        },
        LayoutBox {
            bounds: Rect::from_pos((10.0, 10.0), (30.0, 80.0)),
            margin: EdgeInsets::zero(),
            children: vec![0, 1],
            material: None,
        },
        LayoutBox {
            bounds: Rect::from_pos((0.0, 0.0), (50.0, 100.0)),
            margin: EdgeInsets::zero(),
            children: vec![2],
            material: Some(Material::filled(Color::green())),
        },
        LayoutBox {
            bounds: Rect::from_pos((50.0, 0.0), (50.0, 100.0)),
            margin: EdgeInsets::zero(),
            children: vec![],
            material: Some(Material::filled(Color::blue())),
        },
        LayoutBox {
            bounds: Rect::from_pos((0.0, 0.0), (100.0, 100.0)),
            margin: EdgeInsets::zero(),
            children: vec![3, 4],
            material: None,
        },
    ];
    assert_slice_eq(&expected_layout, &actual_layout);
}

fn layout_with_constraints(widget: &dyn Layout, constraints: &BoxConstraints) -> Vec<LayoutBox> {
    let mut tree = LayoutTree::new();
    let sbox = widget.layout(&mut tree, constraints);
    let lbox = LayoutBox::from_child(sbox, (0.0, 0.0));
    tree.insert(lbox);
    tree.boxes
}
