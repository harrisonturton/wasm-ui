use layout::{Container, Layout, Positioned, Color};
use math::Vector2;
use platform::AppDriver;

pub struct App {
    position: Vector2,
}

impl App {
    pub fn new() -> App {
        let position = Vector2::zero();
        App { position }
    }

    #[allow(dead_code)]
    fn render_sidebar(&self, _: f32) -> Box<dyn Layout> {
        Box::new(Positioned {
            position: Vector2::zero(),
            child: Box::new(Container {
                color: Some(Color::rgba(0.0, 0.0, 0.0, 50.0)),
                size: (150.0, f32::INFINITY).into(),
            })
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
                size: (100.0, 100.0).into(),
            }),
        })
    }
}

impl AppDriver for App {
    fn tick(&mut self, time: f32) -> Box<dyn Layout> {
        self.render_moving_box(time)
    }
}
