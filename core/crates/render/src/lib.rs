#![warn(clippy::all)]
use anyhow::Error;
use layout::LayoutTree;

pub mod browser;

pub trait AppDriver {
    fn tick(&mut self, time: f32) -> Result<LayoutTree, Error>;
}