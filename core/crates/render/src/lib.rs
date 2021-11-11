#![warn(clippy::all)]
use anyhow::Error;

pub mod browser;

pub trait AppDriver {
    fn tick(&mut self, time: f32) -> Result<(), Error>;
}