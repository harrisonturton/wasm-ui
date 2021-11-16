use layout::{
    Center, Color, Container, CrossAxisAlignment, EdgeInsets, Flex, Layout, MainAxisAlignment,
    MainAxisSize, Positioned, Stack,
};
use math::Vector2;
use platform::AppDriver;

pub struct App {
    position: Vector2,
}

impl AppDriver for App {
    fn tick(&mut self, time: f32) -> Box<dyn Layout> {
        self.render_flex_group()
    }
}

impl App {
    pub fn new() -> App {
        let position = Vector2::zero();
        App { position }
    }

    #[allow(dead_code)]
    pub fn render_flex_group(&self) -> Box<dyn Layout> {
        use layout::{Axis, FlexGroup};
        Box::new(FlexGroup {
            axis: Axis::Vertical,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::SpaceBetween,
            cross_axis_alignment: CrossAxisAlignment::Stretch,
            children: vec![
                Flex::Fixed {
                    child: Box::new(Container {
                        margin: EdgeInsets::all(10.0),
                        size: (50.0, 100.0).into(),
                        color: Color::red(),
                        ..Default::default()
                    }),
                },
                Flex::Fixed {
                    child: Box::new(Container {
                        margin: EdgeInsets::all(10.0),
                        size: (200.0, 300.0).into(),
                        color: Color::green(),
                        ..Default::default()
                    }),
                },
            ],
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
