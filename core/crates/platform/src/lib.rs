#![warn(clippy::all)]
use layout::Layout;

pub mod browser;

pub trait AppDriver {
    fn tick(&mut self, time: f32) -> Box<dyn Layout>;
}
