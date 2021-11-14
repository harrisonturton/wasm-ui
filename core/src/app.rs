use layout::{Color, Column, Container, CrossAxisAlignment, Flex, Layout, Positioned, Stack};
use math::Vector2;
use platform::AppDriver;

pub struct App {
    position: Vector2,
}

impl AppDriver for App {
    fn tick(&mut self, time: f32) -> Box<dyn Layout> {
        self.render_flex_column(time)
    }
}

impl App {
    pub fn new() -> App {
        let position = Vector2::zero();
        App { position }
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
                        color: Some(Color::green()),
                        size: Some((100.0, 100.0).into()),
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
                    cross_axis_alignment: CrossAxisAlignment::Start,
                    children: vec![
                        Flex::Fixed {
                            child: Box::new(Column {
                                cross_axis_alignment: CrossAxisAlignment::Start,
                                children: vec![Flex::Fixed {
                                    child: Box::new(Container {
                                        size: Some((200.0, 25.0).into()),
                                        color: Some(Color::black().alpha(0.25)),
                                        child: None,
                                    }),
                                }],
                            }),
                        },
                        Flex::Flexible {
                            flex: 1.0,
                            child: Box::new(Container {
                                size: Some((100.0, 100.0).into()),
                                color: Some(Color::green()),
                                ..Default::default()
                            }),
                        },
                        Flex::Fixed {
                            child: Box::new(Container {
                                size: Some(Vector2::new(100.0, 100.0)),
                                color: Some(Color::red()),
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
                color: Some(Color::rgba(0.0, 0.0, 0.0, 50.0)),
                size: Some((150.0, f32::INFINITY).into()),
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
                        size: Some((100.0, 100.0).into()),
                        color: Some(Color::green()),
                        ..Default::default()
                    }),
                },
                Positioned {
                    position: (100.0, 0.0).into(),
                    child: Box::new(Container {
                        size: Some((100.0, 100.0).into()),
                        color: Some(Color::red()),
                        ..Default::default()
                    }),
                },
                Positioned {
                    position: (200.0, 0.0).into(),
                    child: Box::new(Container {
                        size: Some((100.0, 100.0).into()),
                        color: Some(Color::blue()),
                        ..Default::default()
                    }),
                },
                Positioned {
                    position: (300.0, 0.0).into(),
                    child: Box::new(Container {
                        size: Some((100.0, 100.0).into()),
                        color: Some(Color::red()),
                        ..Default::default()
                    }),
                },
                Positioned {
                    position: (400.0, 0.0).into(),
                    child: Box::new(Container {
                        size: Some((100.0, 100.0).into()),
                        color: Some(Color::green()),
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
                color: Some(Color::rgba(0.0, 0.0, 0.0, 50.0)),
                size: Some((100.0, 100.0).into()),
                ..Default::default()
            }),
        })
    }
}
