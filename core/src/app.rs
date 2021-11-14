use layout::{
    Color, Column, Container, CrossAxisAlignment, Flex, Layout, MainAxisAlignment, MainAxisSize,
    Positioned, Row, Stack,
};
use math::Vector2;
use platform::AppDriver;

pub struct App {
    position: Vector2,
}

impl AppDriver for App {
    fn tick(&mut self, time: f32) -> Box<dyn Layout> {
        self.render_sidebar_layout(time)
    }
}

impl App {
    pub fn new() -> App {
        let position = Vector2::zero();
        App { position }
    }

    #[allow(dead_code)]
    pub fn sidebar(&self) -> Box<dyn Layout> {
        Box::new(Column {
            main_axis_size: MainAxisSize::Max,
            cross_axis_alignment: CrossAxisAlignment::Stretch,
            main_axis_alignment: MainAxisAlignment::Start,
            children: vec![Flex::Fixed {
                child: Box::new(Container {
                    size: (100.0, 100.0).into(),
                    color: Color::green(),
                    ..Default::default()
                }),
            }],
        })
    }

    #[allow(dead_code)]
    fn render_sidebar_layout(&self, _time: f32) -> Box<dyn Layout> {
        // Should be a black sidebar that is 100px wide, with one green
        // rectangle inside it that is 25px tall and 100px wide.
        Box::new(Row {
            cross_axis_alignment: CrossAxisAlignment::Stretch,
            main_axis_alignment: MainAxisAlignment::Start,
            children: vec![Flex::Fixed {
                child: Box::new(Container {
                    size: (100.0, f32::INFINITY).into(),
                    color: Color::green(),
                    child: Some(Box::new(Column {
                        main_axis_size: MainAxisSize::Min,
                        cross_axis_alignment: CrossAxisAlignment::Stretch,
                        main_axis_alignment: MainAxisAlignment::Start,
                        children: vec![
                            Flex::Fixed {
                                child: Box::new(Container {
                                    size: (f32::INFINITY, 25.0).into(),
                                    color: Color::red(),
                                    ..Default::default()
                                }),
                            },
                            Flex::Fixed {
                                child: Box::new(Container {
                                    size: (f32::INFINITY, 25.0).into(),
                                    color: Color::yellow(),
                                    ..Default::default()
                                }),
                            },
                        ],
                    })),
                    ..Default::default()
                }),
            }],
        })
    }

    #[allow(dead_code)]
    fn render_space_around_row(&self, _time: f32) -> Box<dyn Layout> {
        Box::new(Row {
            cross_axis_alignment: CrossAxisAlignment::Center,
            main_axis_alignment: MainAxisAlignment::SpaceEvenly,
            children: vec![
                Flex::Fixed {
                    child: Box::new(Container {
                        size: (100.0, 200.0).into(),
                        color: Color::red(),
                        ..Default::default()
                    }),
                },
                Flex::Flexible {
                    flex: 1.0,
                    child: Box::new(Container {
                        size: (100.0, 100.0).into(),
                        color: Color::green(),
                        ..Default::default()
                    }),
                },
                Flex::Fixed {
                    child: Box::new(Container {
                        size: (100.0, 100.0).into(),
                        color: Color::blue(),
                        ..Default::default()
                    }),
                },
            ],
        })
    }

    #[allow(dead_code)]
    fn render_space_around_column(&self, _time: f32) -> Box<dyn Layout> {
        Box::new(Column {
            main_axis_alignment: MainAxisAlignment::SpaceEvenly,
            cross_axis_alignment: CrossAxisAlignment::End,
            children: vec![
                Flex::Fixed {
                    child: Box::new(Container {
                        size: (100.0, 100.0).into(),
                        color: Color::green(),
                        ..Default::default()
                    }),
                },
                Flex::Fixed {
                    child: Box::new(Container {
                        size: (100.0, 100.0).into(),
                        color: Color::red(),
                        ..Default::default()
                    }),
                },
                Flex::Fixed {
                    child: Box::new(Container {
                        size: (100.0, 100.0).into(),
                        color: Color::blue(),
                        ..Default::default()
                    }),
                },
            ],
            ..Default::default()
        })
    }

    // The green box should be positioned at (200, 200). If not, then we are not
    // correctly calculating the cumulative relative offset of a widget from
    // it's ancestors.
    #[allow(dead_code)]
    fn render_nested_positioned(&self, _time: f32) -> Box<dyn Layout> {
        Box::new(Positioned {
            position: (0.0, 0.0).into(),
            child: Box::new(Positioned {
                position: (100.0, 100.0).into(),
                child: Box::new(Positioned {
                    position: (200.0, 200.0).into(),
                    child: Box::new(Container {
                        color: Color::green(),
                        size: (100.0, 100.0).into(),
                        ..Default::default()
                    }),
                }),
            }),
        })
    }

    #[allow(dead_code)]
    fn render_flex_column(&self, time: f32) -> Box<dyn Layout> {
        let x_pos = -200.0 * (0.5 + 0.5 * (time * 0.005).sin());
        Box::new(Stack {
            children: vec![Positioned {
                position: (x_pos, 0.0).into(),
                child: Box::new(Column {
                    main_axis_size: MainAxisSize::Max,
                    main_axis_alignment: MainAxisAlignment::Start,
                    cross_axis_alignment: CrossAxisAlignment::Start,
                    children: vec![
                        Flex::Fixed {
                            child: Box::new(Column {
                                main_axis_size: MainAxisSize::Max,
                                main_axis_alignment: MainAxisAlignment::Start,
                                cross_axis_alignment: CrossAxisAlignment::Start,
                                children: vec![Flex::Fixed {
                                    child: Box::new(Container {
                                        size: (200.0, 25.0).into(),
                                        color: Color::black().alpha(0.25),
                                        child: None,
                                    }),
                                }],
                            }),
                        },
                        Flex::Flexible {
                            flex: 1.0,
                            child: Box::new(Container {
                                size: (100.0, 100.0).into(),
                                color: Color::green(),
                                ..Default::default()
                            }),
                        },
                        Flex::Fixed {
                            child: Box::new(Container {
                                size: Vector2::new(100.0, 100.0),
                                color: Color::red(),
                                child: None,
                            }),
                        },
                    ],
                }),
            }],
        })
    }

    #[allow(dead_code)]
    fn render_sidebar(&self, _: f32) -> Box<dyn Layout> {
        Box::new(Positioned {
            position: Vector2::zero(),
            child: Box::new(Container {
                color: Color::rgba(0.0, 0.0, 0.0, 50.0),
                size: (150.0, f32::INFINITY).into(),
                ..Default::default()
            }),
        })
    }

    // 5 boxes should be placed directly next to eachother in a row. This tests
    // for bugs in how the layout algorithm positions widgets relative to their
    // parent, specifically when there are multiple independently-positioned
    // widgets that are laid out in a row.
    #[allow(dead_code)]
    fn render_boxes(&self, _: f32) -> Box<dyn Layout> {
        Box::new(Stack {
            children: vec![
                Positioned {
                    position: (0.0, 0.0).into(),
                    child: Box::new(Container {
                        size: (100.0, 100.0).into(),
                        color: Color::green(),
                        ..Default::default()
                    }),
                },
                Positioned {
                    position: (100.0, 0.0).into(),
                    child: Box::new(Container {
                        size: (100.0, 100.0).into(),
                        color: Color::red(),
                        ..Default::default()
                    }),
                },
                Positioned {
                    position: (200.0, 0.0).into(),
                    child: Box::new(Container {
                        size: (100.0, 100.0).into(),
                        color: Color::blue(),
                        ..Default::default()
                    }),
                },
                Positioned {
                    position: (300.0, 0.0).into(),
                    child: Box::new(Container {
                        size: (100.0, 100.0).into(),
                        color: Color::red(),
                        ..Default::default()
                    }),
                },
                Positioned {
                    position: (400.0, 0.0).into(),
                    child: Box::new(Container {
                        size: (100.0, 100.0).into(),
                        color: Color::green(),
                        ..Default::default()
                    }),
                },
            ],
        })
    }

    #[allow(dead_code)]
    fn render_moving_box(&mut self, time: f32) -> Box<dyn Layout> {
        let speed = 0.005;
        let radius = 100.0;
        let offset = Vector2::new(100.0, 100.0);
        self.position.x = 100.0 + radius * (time * speed).sin();
        self.position.y = 100.0 + radius * (time * speed).cos();
        self.position += offset;

        Box::new(Positioned {
            position: self.position,
            child: Box::new(Container {
                color: Color::rgba(0.0, 0.0, 0.0, 50.0),
                size: (100.0, 100.0).into(),
                ..Default::default()
            }),
        })
    }
}

/*LayoutTree {
    root: Some(3),
    boxes: [
        LayoutBox {
            rect: Rect { min: Vector2 { x: 0.0, y: 0.0 }, max: Vector2 { x: inf, y: 100.0 } },
            children: [],
            material: Solid(Color { r: 0.0, g: 255.0, b: 0.0, a: 255.0 }) },
        // Column
        LayoutBox {
            // Why is max (100.0, 100.0)?
            rect: Rect { min: Vector2 { x: 0.0, y: 0.0 }, max: Vector2 { x: 100.0, y: 100.0 } },
            children: [0],
            material: None },
        // Fixed Container Black, (100.0, INFINITY)
        LayoutBox {
            rect: Rect { min: Vector2 { x: 0.0, y: 0.0 }, max: Vector2 { x: 100.0, y: 100.0 } },
            children: [1],
            material: Solid(Color { r: 0.0, g: 0.0, b: 0.0, a: 255.0 }) },
        // Row
        LayoutBox {
            rect: Rect { min: Vector2 { x: 0.0, y: 0.0 }, max: Vector2 { x: 100.0, y: 798.0 } },
            children: [2],
            material: None }
    ]
}*/
