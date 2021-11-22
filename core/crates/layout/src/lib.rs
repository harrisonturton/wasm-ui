#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::similar_names)]

mod base;
pub use base::*;

mod container;
pub use container::*;

pub mod container2;

mod decoration;
pub use decoration::*;

mod tree;
pub use tree::*;

mod widget;
pub use widget::*;

mod flex;
pub use flex::*;
