use super::{Color, Material};
use math::{Rect, Vector2};
use std::collections::VecDeque;
use std::fmt::Debug;

#[derive(PartialEq, Clone, Debug)]
pub struct RenderBox {
    pub material: Material,
}

/// This is the essential trait of the box model. It is implemented by all
/// components that undergo the box layout process.
///
/// The `layout` method is called repeatedly to generate a [LayoutTree]. Each
/// tree node is responsible for three things:
///
/// 1. Calculating the position of it's children
/// 2. Inserting it's children into the [LayoutTree]
/// 3. Calculating and returning it's own size to it's parent node
///
/// This allows the [LayoutTree] to be generated in one walk down and up the
/// tree. It's how we can perform layout in O(2n) time.
///
/// This process takes heavy inspiration from the [Flutter render
/// pipeline](https://www.youtube.com/watch?v=UUfXWzp0-DU) and the CSS box
/// model.
pub trait Layout: Debug {
    fn layout(&self, tree: &mut LayoutTree) -> SizedLayoutBox;
}

/// Used to get a [LayoutBox] from a [LayoutTree].
///
/// This is required because [LayoutTree] is implemented using a memory arena in
/// order to play nice with the borrow-checker. It's easier to pass around a
/// copyable value like `usize` than worry about balancing reference lifetimes
/// and shared ownership, and it's more efficient than copying [LayoutBox].
pub type LayoutBoxId = usize;

/// An element that has calculated it's own size, but has not been positioned
/// by it's parent yet. This is the intermediate step during layout.
#[derive(PartialEq, Clone, Debug)]
pub struct SizedLayoutBox {
    pub size: Vector2,
    pub content: RenderBox,
    pub children: Vec<LayoutBoxId>,
}

/// An element that has finished layout. It has been been sized and positioned.
#[derive(PartialEq, Clone, Debug)]
pub struct LayoutBox {
    pub rect: Rect,
    pub content: RenderBox,
    pub children: Vec<LayoutBoxId>,
}

impl LayoutBox {
    /// Convenience method to turn a [SizedLayoutBox] into a [LayoutBox]. This
    /// is handy when implementing the [Layout] trait.
    pub fn from_child<I>(child: SizedLayoutBox, pos: I) -> LayoutBox
    where
        I: Into<Vector2>,
    {
        let min = pos.into();
        let max = min + child.size;
        LayoutBox {
            rect: Rect::new(min, max),
            content: child.content,
            children: child.children,
        }
    }
}

/// A tree of [LayoutBox] elements. The position of each [LayoutBox] is relative
/// to it's parent.
///
/// This is the data structure that is consumed by the render driver to show on
/// the screen. It is intended to be generic across different deploy targets.
///
/// The tree is implemented as a memory arena to be indexed into using a
/// [LayoutBoxId]. This makes it much easier to use with the borrow checker.
#[derive(Clone, Default, Debug)]
pub struct LayoutTree {
    pub root: Option<LayoutBoxId>,
    pub boxes: Vec<LayoutBox>,
}

impl LayoutTree {
    /// Create a new empty [LayoutTree].
    pub fn new() -> LayoutTree {
        LayoutTree {
            root: None,
            boxes: Vec::new(),
        }
    }

    /// Set the root of the tree. This assumes that the [LayoutBoxId] provided
    /// by the caller points to a valid [LayoutBox].
    pub fn set_root(&mut self, root: Option<LayoutBoxId>) {
        self.root = root;
    }

    /// Insert a [LayoutBox] into the tree and get a [LayoutBoxId] to fetch it
    /// again later.
    pub fn insert(&mut self, lbox: LayoutBox) -> LayoutBoxId {
        self.boxes.push(lbox);
        self.boxes.len() - 1
    }

    /// Get a reference to the [LayoutBox] indexed by a [LayoutBoxId].
    pub fn get(&self, id: LayoutBoxId) -> Option<&LayoutBox> {
        self.boxes.get(id)
    }

    /// Get an iterator over a breadth-first search
    pub fn iter(&self) -> LayoutTreeIterator {
        LayoutTreeIterator {
            tree: self,
            remaining: match self.root {
                Some(root) => VecDeque::from([root]),
                None => VecDeque::new(),
            },
        }
    }
}

pub struct LayoutTreeIterator<'a> {
    tree: &'a LayoutTree,
    remaining: VecDeque<LayoutBoxId>,
}

impl<'a> Iterator for LayoutTreeIterator<'a> {
    type Item = &'a LayoutBox;

    fn next(&mut self) -> Option<&'a LayoutBox> {
        let next_id = self.remaining.pop_front()?;
        let lbox = self.tree.get(next_id)?;
        for child in &lbox.children {
            self.remaining.push_back(*child);
        }
        Some(lbox)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_tree_iter_works() {
        let mut tree = LayoutTree::new();
        let a_child = LayoutBox {
            rect: Rect::new((4.0, 4.0).into(), (2.0, 2.0).into()),
            content: RenderBox {
                material: Material::None,
            },
            children: vec![],
        };
        let a_child_id = tree.insert(a_child.clone());
        let a = LayoutBox {
            rect: Rect::new((1.0, 1.0).into(), (2.0, 2.0).into()),
            content: RenderBox {
                material: Material::None,
            },
            children: vec![a_child_id],
        };
        let a_id = tree.insert(a.clone());

        let b_child = LayoutBox {
            rect: Rect::new((5.0, 5.0).into(), (2.0, 2.0).into()),
            content: RenderBox {
                material: Material::None,
            },
            children: vec![],
        };
        let b_child_id = tree.insert(b_child.clone());
        let b = LayoutBox {
            rect: Rect::new((2.0, 2.0).into(), (2.0, 2.0).into()),
            content: RenderBox {
                material: Material::None,
            },
            children: vec![b_child_id],
        };
        let b_id = tree.insert(b.clone());
        let c = LayoutBox {
            rect: Rect::new((3.0, 3.0).into(), (3.0, 3.0).into()),
            content: RenderBox {
                material: Material::None,
            },
            children: vec![a_id, b_id],
        };
        let c_id = tree.insert(c.clone());
        tree.set_root(Some(c_id));

        let actual: Vec<_> = tree.iter().cloned().collect();
        let expected = vec![c, a, b, a_child, b_child];
        for (i, _) in actual.iter().enumerate() {
            assert_eq!(expected[i], actual[i])
        }
    }
}
