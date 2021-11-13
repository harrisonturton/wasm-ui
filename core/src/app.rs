use layout::{Color, Container, Layout, Positioned, Stack, Flex, Column, Row};
use math::Vector2;
use platform::AppDriver;

pub struct App {
    position: Vector2,
}

impl AppDriver for App {
    fn tick(&mut self, time: f32) -> Box<dyn Layout> {
        self.render_flex_row(time)
    }
}

impl App {
    pub fn new() -> App {
        let position = Vector2::zero();
        App { position }
    }

    #[allow(dead_code)]
    fn render_flex_row(&self, _: f32) -> Box<dyn Layout> {
        Box::new(Row {
            children: vec![
                Flex {
                    flex: Some(1.0),
                    child: Box::new(Container {
                        size: None,
                        color: Some(Color::green()),
                        child: None,
                    }),
                },
                Flex {
                    flex: None,
                    child: Box::new(Container {
                        size: Some(Vector2::new(100.0, 100.0)),
                        color: Some(Color::red()),
                        child: None,
                    }),
                },
                Flex {
                    flex: Some(1.0),
                    child: Box::new(Container {
                        size: None,
                        color: Some(Color::yellow()),
                        child: None,
                    }),
                },
                Flex {
                    flex: None,
                    child: Box::new(Container {
                        size: Some(Vector2::new(100.0, 100.0)),
                        color: Some(Color::red()),
                        child: None,
                    }),
                },
            ],
        })
    }

    #[allow(dead_code)]
    fn render_flex_column(&self, _: f32) -> Box<dyn Layout> {
        Box::new(Column {
            children: vec![
                Flex {
                    flex: Some(1.0),
                    child: Box::new(Container {
                        size: None,
                        color: Some(Color::green()),
                        child: None,
                    }),
                },
                Flex {
                    flex: None,
                    child: Box::new(Container {
                        size: Some(Vector2::new(100.0, 100.0)),
                        color: Some(Color::red()),
                        child: None,
                    }),
                },
                Flex {
                    flex: Some(1.0),
                    child: Box::new(Container {
                        size: None,
                        color: Some(Color::yellow()),
                        child: None,
                    }),
                },
                Flex {
                    flex: None,
                    child: Box::new(Container {
                        size: Some(Vector2::new(100.0, 100.0)),
                        color: Some(Color::red()),
                        child: None,
                    }),
                },
            ],
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