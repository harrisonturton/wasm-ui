use super::{BoxConstraints, Layout, LayoutBox, LayoutTree, SizedLayoutBox};
use crate::decoration::{Color, Material};
use math::Vector2;
use std::fmt::Debug;

// --------------------------------------------------
// Center
// --------------------------------------------------

#[derive(Debug)]
pub struct Center {
    pub child: Box<dyn Layout>,
}

impl Layout for Center {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let sbox = self.child.layout(tree, constraints);
        let pos = (constraints.max / 2.0) - (sbox.size / 2.0);
        let lbox = LayoutBox::from_child(sbox, pos);
        let id = tree.insert(lbox);
        SizedLayoutBox {
            size: constraints.max,
            children: vec![id],
            material: None,
            ..SizedLayoutBox::default()
        }
    }
}

// --------------------------------------------------
// Stack
// --------------------------------------------------

#[derive(Debug)]
pub struct Stack {
    pub children: Vec<Positioned>,
}

impl Layout for Stack {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let mut children = Vec::new();
        for child in &self.children {
            let child = child.layout(tree, constraints);
            let lbox = LayoutBox::from_child(child, Vector2::zero());
            let id = tree.insert(lbox);
            children.push(id);
        }
        SizedLayoutBox {
            size: constraints.max,
            children,
            material: None,
            ..Default::default()
        }
    }
}

// --------------------------------------------------
// Positioned
// --------------------------------------------------

#[derive(Debug)]
pub struct Positioned {
    pub position: Vector2,
    pub child: Box<dyn Layout>,
}

impl Layout for Positioned {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox {
        let child_constraints = BoxConstraints {
            min: Vector2::zero(),
            max: constraints.max - self.position,
        };
        let sbox = self.child.layout(tree, &child_constraints);
        let lbox = LayoutBox::from_child(sbox, self.position);
        let child_id = tree.insert(lbox);
        SizedLayoutBox {
            size: constraints.max,
            children: vec![child_id],
            material: Some(Material::filled(Color::black().alpha(0.1))),
            ..Default::default()
        }
    }
}
