use std::fmt::Debug;
use super::LayoutTree;

/// Widgets that implement [Renderable] are able to generate a [ChildRenderBox],
/// which is placed on the [LayoutTree] and then rendered to the screen.
pub trait Renderable: Debug {
    fn render(&self, tree: &mut LayoutTree) -> ChildRenderBox;
}

pub trait Widget: Debug {
    fn render(&self) -> Box<dyn Layout>;
}

#[derive(Clone, Debug)]
pub enum Material {
    None,
    Solid(f32, f32, f32, f32)
}