use platform::AppDriver;
use math::Vector2;
use layout::{Layout, Positioned, Container};

pub struct App {
    position: Vector2,
}

impl App {
    pub fn new() -> App {
        let position = Vector2::zero();
        App { position }
    }
}

impl AppDriver for App {
    fn tick(&mut self, time: f32) -> Box<dyn Layout> {
        let speed = 0.005;
        let radius = 100.0;
        let offset = Vector2::new(100.0, 100.0);
        self.position.x = 100.0 + radius * (time * speed).sin();
        self.position.y = 100.0 + radius * (time * speed).cos();
        self.position += offset;

        Box::new(Positioned {
            position: self.position,
            child: Box::new(Container {
                size: (100.0, 100.0).into(),
            })
        })
    }
}