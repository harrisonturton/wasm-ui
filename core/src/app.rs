use layout::{
    Alignment, Axis, BorderSide, Borders, Color, Container, CrossAxisAlignment, EdgeInsets, Flex,
    Flexible, Layout, MainAxisAlignment, MainAxisSize, Positioned, Stack,
};
use math::Vector2;
use platform::AppDriver;

pub struct App {
    position: Vector2,
}

impl AppDriver for App {
    fn tick(&mut self, _: f32) -> Box<dyn Layout> {
        self.sidebar()
    }
}

impl App {
    pub fn new() -> App {
        let position = Vector2::zero();
        App { position }
    }

    /*#[allow(dead_code)]
    pub fn container(&self) -> Box<dyn Layout> {
        use layout::container2::Container;
        Box::new(Container {
            color: Color::green(),
            alignment: Alignment::center(),
            child: Some(Box::new(Container {
                color: Color::red(),
                ..Default::default()
            })),
            ..Default::default()
        })
    }*/

    #[allow(dead_code)]
    pub fn sidebar(&self) -> Box<dyn Layout> {
        let border_color = Color::rgba(70.0, 70.0, 70.0, 255.0);
        let widgets = Container {
            borders: Borders::bottom(Color::rgba(15.0, 100.0, 225.0, 255.0), 10.0),
            child: Some(Box::new(Flex {
                axis: Axis::Horizontal,
                main_axis_size: MainAxisSize::Max,
                main_axis_alignment: MainAxisAlignment::Start,
                cross_axis_alignment: CrossAxisAlignment::Stretch,
                children: vec![
                    Box::new(Container {
                        width: Some(40.0),
                        height: Some(f32::INFINITY),
                        color: Color::rgba(30.0, 30.0, 30.0, 255.0),
                        padding: EdgeInsets::all(5.0),
                        child: Some(Box::new(Flex {
                            axis: Axis::Vertical,
                            main_axis_size: MainAxisSize::Max,
                            main_axis_alignment: MainAxisAlignment::Start,
                            cross_axis_alignment: CrossAxisAlignment::Center,
                            children: vec![
                                Box::new(Container {
                                    height: Some(30.0),
                                    width: Some(30.0),
                                    color: Color::rgba(45.0, 45.0, 45.0, 255.0),
                                    margin: EdgeInsets::bottom(5.0),
                                    ..Default::default()
                                }),
                                Box::new(Container {
                                    height: Some(30.0),
                                    width: Some(30.0),
                                    color: Color::rgba(45.0, 45.0, 45.0, 255.0),
                                    margin: EdgeInsets::bottom(5.0),
                                    ..Default::default()
                                }),
                                Box::new(Container {
                                    height: Some(30.0),
                                    width: Some(30.0),
                                    color: Color::rgba(45.0, 45.0, 45.0, 255.0),
                                    margin: EdgeInsets::bottom(5.0),
                                    ..Default::default()
                                }),
                            ],
                        })),
                        ..Default::default()
                    }),
                    Box::new(Container {
                        borders: Borders::right(border_color, 1.0),
                        height: Some(f32::INFINITY),
                        width: Some(150.0),
                        color: Color::rgba(35.0, 35.0, 35.0, 255.0),
                        child: Some(Box::new(Flex {
                            axis: Axis::Vertical,
                            main_axis_size: MainAxisSize::Max,
                            main_axis_alignment: MainAxisAlignment::Start,
                            cross_axis_alignment: CrossAxisAlignment::Stretch,
                            children: vec![Box::new(Container {
                                width: Some(f32::INFINITY),
                                height: Some(25.0),
                                margin: EdgeInsets::top(5.0),
                                color: Color::rgba(45.0, 45.0, 45.0, 255.0),
                                ..Default::default()
                            })],
                        })),
                        ..Default::default()
                    }),
                    Box::new(Flexible {
                        flex_factor: 1.0,
                        child: Box::new(Container {
                            color: Color::rgba(22.0, 22.0, 22.0, 255.0),
                            ..Default::default()
                        }),
                    }),
                    Box::new(Container {
                        width: Some(175.0),
                        height: Some(f32::INFINITY),
                        color: Color::rgba(35.0, 35.0, 35.0, 255.0),
                        borders: Borders::left(border_color, 1.0),
                        child: Some(Box::new(Flex {
                            axis: Axis::Vertical,
                            main_axis_size: MainAxisSize::Max,
                            main_axis_alignment: MainAxisAlignment::Start,
                            cross_axis_alignment: CrossAxisAlignment::Stretch,
                            children: vec![Box::new(Container {
                                width: Some(f32::INFINITY),
                                height: Some(100.0),
                                color: Color::rgba(35.0, 35.0, 35.0, 255.0),
                                margin: EdgeInsets::bottom(5.0),
                                ..Default::default()
                            })],
                        })),
                        ..Default::default()
                    }),
                ],
            })),
            ..Container::default()
        };
        Box::new(widgets)
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
                        width: Some(100.0),
                        height: Some(100.0),
                        ..Default::default()
                    }),
                }),
            }),
        })
    }

    #[allow(dead_code)]
    fn render_sidebar(&self, _: f32) -> Box<dyn Layout> {
        Box::new(Positioned {
            position: Vector2::zero(),
            child: Box::new(Container {
                color: Color::rgba(0.0, 0.0, 0.0, 50.0),
                width: Some(150.0),
                height: Some(f32::INFINITY),
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
                        width: Some(100.0),
                        height: Some(100.0),
                        color: Color::green(),
                        ..Default::default()
                    }),
                },
                Positioned {
                    position: (100.0, 0.0).into(),
                    child: Box::new(Container {
                        width: Some(100.0),
                        height: Some(100.0),
                        color: Color::red(),
                        ..Default::default()
                    }),
                },
                Positioned {
                    position: (200.0, 0.0).into(),
                    child: Box::new(Container {
                        width: Some(100.0),
                        height: Some(100.0),
                        color: Color::blue(),
                        ..Default::default()
                    }),
                },
                Positioned {
                    position: (300.0, 0.0).into(),
                    child: Box::new(Container {
                        width: Some(100.0),
                        height: Some(100.0),
                        color: Color::red(),
                        ..Default::default()
                    }),
                },
                Positioned {
                    position: (400.0, 0.0).into(),
                    child: Box::new(Container {
                        width: Some(100.0),
                        height: Some(100.0),
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
                width: Some(100.0),
                height: Some(100.0),
                ..Default::default()
            }),
        })
    }
}
