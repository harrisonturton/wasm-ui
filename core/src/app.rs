use layout::{
    Axis, Borders, Color, Container, CrossAxisAlignment, EdgeInsets, Flex, Flexible, Layout,
    MainAxisAlignment, MainAxisSize, Positioned, Stack,
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

    #[allow(dead_code)]
    pub fn simple(&self) -> Box<dyn Layout> {
        use layout::container2::Container;
        Box::new(Container {
            color: Color::red(),
            width: Some(100.0),
            height: Some(100.0),
            ..Default::default()
        })
    }

    #[allow(dead_code)]
    pub fn container(&self) -> Box<dyn Layout> {
        use layout::container2::{Alignment, Container};
        Box::new(Container {
            color: Color::green(),
            alignment: Alignment::center(),
            child: Some(Box::new(layout::container2::Container {
                width: Some(100.0),
                height: Some(100.0),
                color: Color::red(),
                ..Default::default()
            })),
            ..Default::default()
        })
    }

    #[allow(dead_code)]
    pub fn sidebar(&self) -> Box<dyn Layout> {
        let widgets = Flex {
            axis: Axis::Horizontal,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Stretch,
            children: vec![
                Box::new(Container {
                    borders: Borders::right(Color::rgba(70.0, 70.0, 70.0, 255.0), 1.0),
                    size: (200.0, f32::INFINITY).into(),
                    color: Color::rgba(35.0, 35.0, 35.0, 255.0),
                    child: Some(Box::new(Flex {
                        axis: Axis::Vertical,
                        main_axis_size: MainAxisSize::Max,
                        main_axis_alignment: MainAxisAlignment::Start,
                        cross_axis_alignment: CrossAxisAlignment::Stretch,
                        children: vec![
                            Box::new(Container {
                                size: (f32::INFINITY, 25.0).into(),
                                margin: EdgeInsets::bottom(5.0),
                                color: Color::rgba(40.0, 40.0, 40.0, 255.0),
                                ..Default::default()
                            }),
                            Box::new(Container {
                                size: (f32::INFINITY, 25.0).into(),
                                margin: EdgeInsets::bottom(5.0),
                                color: Color::rgba(45.0, 45.0, 45.0, 255.0),
                                ..Default::default()
                            }),
                        ],
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
                    size: (175.0, f32::INFINITY).into(),
                    color: Color::rgba(35.0, 35.0, 35.0, 255.0),
                    borders: Borders::left(Color::rgba(70.0, 70.0, 70.0, 255.0), 1.0),
                    child: Some(Box::new(Flex {
                        axis: Axis::Vertical,
                        main_axis_size: MainAxisSize::Max,
                        main_axis_alignment: MainAxisAlignment::Start,
                        cross_axis_alignment: CrossAxisAlignment::Stretch,
                        children: vec![
                            Box::new(Container {
                                size: (f32::INFINITY, 100.0).into(),
                                color: Color::rgba(35.0, 35.0, 35.0, 255.0),
                                margin: EdgeInsets::bottom(5.0),
                                ..Default::default()
                            }),
                            Box::new(Flexible {
                                flex_factor: 1.0,
                                child: Box::new(Container {
                                    size: (f32::INFINITY, 25.0).into(),
                                    color: Color::rgba(45.0, 45.0, 45.0, 255.0),
                                    ..Default::default()
                                }),
                            }),
                        ],
                    })),
                    ..Default::default()
                }),
            ],
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
                        size: (100.0, 100.0).into(),
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
