use crate::base::EdgeInsets;
use crate::decoration::Material;
use math::{Rect, Vector2};
use std::collections::VecDeque;
use std::fmt::Debug;

/// This is the essential trait of the box model. It is implemented by all
/// components that undergo the box layout process.
///
/// The `layout` method is called repeatedly to generate a `LayoutTree`. Each
/// tree node is responsible for three things:
///
/// 1. Calculating the position of it's children
/// 2. Inserting it's children into the `LayoutTree`
/// 3. Calculating and returning it's own size to it's parent node
///
/// This allows the `LayoutTree` to be generated in one walk down and up the
/// tree. It's how we can perform layout in O(2n) time.
///
/// This process takes heavy inspiration from the [Flutter render
/// pipeline](https://www.youtube.com/watch?v=UUfXWzp0-DU) and the CSS box
/// model.
pub trait Layout: Debug {
    fn layout(&self, tree: &mut LayoutTree, constraints: &BoxConstraints) -> SizedLayoutBox;
}

// The minimum and maximum dimensions that a [SizedLayoutBox] or a [LayoutBox]
// can be.
#[derive(PartialEq, Clone, Debug)]
pub struct BoxConstraints {
    pub min: Vector2,
    pub max: Vector2,
}

impl BoxConstraints {
    #[must_use]
    pub fn from_max<I: Into<Vector2>>(max: I) -> BoxConstraints {
        BoxConstraints {
            min: Vector2::zero(),
            max: max.into(),
        }
    }

    #[must_use]
    pub fn horizontal(&self) -> Vector2 {
        Vector2::new(self.min.x, self.max.x)
    }

    #[must_use]
    pub fn vertical(&self) -> Vector2 {
        Vector2::new(self.min.y, self.max.y)
    }

    #[must_use]
    pub fn has_unbounded_height(&self) -> bool {
        self.max.y != f32::INFINITY
    }

    #[must_use]
    pub fn has_unbounded_width(&self) -> bool {
        self.max.x != f32::INFINITY
    }
}

/// Used to get a `LayoutBox` from a `LayoutTree`.
///
/// This is required because `LayoutTree` is implemented using a memory arena in
/// order to play nice with the borrow-checker. It's easier to pass around a
/// copyable value like `usize` than worry about balancing reference lifetimes
/// and shared ownership, and it's more efficient than copying `LayoutBox`.
pub type LayoutBoxId = usize;

/// An element that has calculated it's own size, but has not been positioned
/// by it's parent yet. This is the intermediate step during layout.
#[derive(PartialEq, Clone, Default, Debug)]
pub struct SizedLayoutBox {
    pub margin: EdgeInsets,
    pub size: Vector2,
    pub children: Vec<LayoutBoxId>,
    pub material: Option<Material>,
}

/// An element that has finished layout. It has been been sized and positioned.
#[derive(PartialEq, Clone, Default, Debug)]
pub struct LayoutBox {
    pub bounds: Rect, // Includes margins
    pub margin: EdgeInsets,
    pub children: Vec<LayoutBoxId>,
    pub material: Option<Material>,
}

impl Eq for LayoutBox {}

impl LayoutBox {
    /// Convenience method to turn a `SizedLayoutBox` into a `LayoutBox`. This
    /// is handy when implementing the [Layout] trait.
    pub fn from_child<I>(child: SizedLayoutBox, pos: I) -> LayoutBox
    where
        I: Into<Vector2>,
    {
        let min = pos.into();
        let max = min + child.size;
        LayoutBox {
            bounds: Rect::new(min, max),
            margin: child.margin,
            children: child.children,
            material: child.material,
        }
    }
}

/// A tree of `LayoutBox` elements. The position of each `LayoutBox` is relative
/// to it's parent.
///
/// This is the data structure that is consumed by the render driver to show on
/// the screen. It is intended to be generic across different deploy targets.
///
/// The tree is implemented as a memory arena to be indexed into using a
/// `LayoutBoxId`. This makes it much easier to use with the borrow checker.
#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Default, Debug)]
pub struct LayoutTree {
    pub root: Option<LayoutBoxId>,
    pub boxes: Vec<LayoutBox>,
}

impl LayoutTree {
    /// Create a new empty `LayoutTree`.
    #[must_use]
    pub fn new() -> LayoutTree {
        LayoutTree {
            root: None,
            boxes: Vec::new(),
        }
    }

    /// Set the root of the tree. This assumes that the `LayoutBoxId` provided
    /// by the caller points to a valid `LayoutBox`.
    pub fn set_root(&mut self, root: Option<LayoutBoxId>) {
        self.root = root;
    }

    /// Insert a `LayoutBox` into the tree and get a `LayoutBoxId` to fetch it
    /// again later.
    pub fn insert(&mut self, lbox: LayoutBox) -> LayoutBoxId {
        self.boxes.push(lbox);
        self.boxes.len() - 1
    }

    /// Get a reference to the `LayoutBox` indexed by a `LayoutBoxId`.
    #[must_use]
    pub fn get(&self, id: LayoutBoxId) -> Option<&LayoutBox> {
        self.boxes.get(id)
    }

    /// Get an iterator over a breadth-first search
    #[must_use]
    pub fn iter(&self) -> LayoutTreeIterator {
        LayoutTreeIterator {
            tree: self,
            parents: match self.root {
                Some(root) => VecDeque::from([root]),
                None => VecDeque::new(),
            },
            offsets: match self.root {
                Some(_) => VecDeque::from([Vector2::zero()]),
                None => VecDeque::new(),
            },
            remaining: match self.root {
                Some(root) => VecDeque::from([root]),
                None => VecDeque::new(),
            },
        }
    }
}

pub struct LayoutTreeIterator<'a> {
    tree: &'a LayoutTree,
    parents: VecDeque<LayoutBoxId>,
    offsets: VecDeque<Vector2>,
    remaining: VecDeque<LayoutBoxId>,
}

impl<'a> Iterator for LayoutTreeIterator<'a> {
    type Item = (&'a LayoutBox, &'a LayoutBox, Vector2);

    fn next(&mut self) -> Option<(&'a LayoutBox, &'a LayoutBox, Vector2)> {
        let parent_id = self.parents.pop_front()?;
        let child_id = self.remaining.pop_front()?;
        let parent_offset = self.offsets.pop_front()?;
        let parent = self.tree.get(parent_id)?;
        let child = self.tree.get(child_id)?;
        let offset = child.bounds.min + parent_offset;
        for child in child.children.iter().rev() {
            self.parents.push_front(child_id);
            self.remaining.push_front(*child);
            self.offsets.push_front(offset);
        }
        Some((parent, child, parent_offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoration::{Color, Material};

    #[test]
    fn layout_tree_iter_nested() {
        let mut tree = LayoutTree::new();
        let a = LayoutBox {
            bounds: Rect::new((0.0, 0.0).into(), (0.0, 0.0).into()),
            children: vec![],
            material: None,
            ..Default::default()
        };
        let a_id = tree.insert(a.clone());
        let b = LayoutBox {
            bounds: Rect::new((1.0, 1.0).into(), (1.0, 1.0).into()),
            children: vec![a_id],
            material: None,
            ..Default::default()
        };
        let b_id = tree.insert(b.clone());
        let c = LayoutBox {
            bounds: Rect::new((2.0, 2.0).into(), (2.0, 2.0).into()),
            children: vec![b_id],
            material: None,
            ..Default::default()
        };
        let c_id = tree.insert(c.clone());
        let root = LayoutBox {
            bounds: Rect::new((3.0, 3.0).into(), (3.0, 3.0).into()),
            children: vec![c_id],
            material: None,
            ..Default::default()
        };
        let root_id = tree.insert(root.clone());
        tree.set_root(Some(root_id));

        let mut actual = vec![];
        for item in tree.iter() {
            actual.push(item);
        }

        let expected = vec![
            (&root, &root, Vector2::new(0.0, 0.0)),
            (&root, &c, Vector2::new(3.0, 3.0)),
            (&c, &b, Vector2::new(5.0, 5.0)),
            (&b, &a, Vector2::new(6.0, 6.0)),
        ];
        for (i, _) in actual.iter().enumerate() {
            assert_eq!(expected[i], actual[i]);
        }
    }

    #[test]
    fn layout_tree_iter_works() {
        let mut tree = LayoutTree::new();
        let a_child = LayoutBox {
            bounds: Rect::new((4.0, 4.0).into(), (2.0, 2.0).into()),
            children: vec![],
            material: None,
            ..Default::default()
        };
        let a_child_id = tree.insert(a_child.clone());
        let a = LayoutBox {
            bounds: Rect::new((1.0, 1.0).into(), (2.0, 2.0).into()),
            children: vec![a_child_id],
            material: None,
            ..Default::default()
        };
        let a_id = tree.insert(a.clone());

        let b_child = LayoutBox {
            bounds: Rect::new((5.0, 5.0).into(), (2.0, 2.0).into()),
            children: vec![],
            material: None,
            ..Default::default()
        };
        let b_child_id = tree.insert(b_child.clone());
        let b = LayoutBox {
            bounds: Rect::new((2.0, 2.0).into(), (2.0, 2.0).into()),
            children: vec![b_child_id],
            material: None,
            ..Default::default()
        };
        let b_id = tree.insert(b.clone());
        let c = LayoutBox {
            bounds: Rect::new((3.0, 3.0).into(), (3.0, 3.0).into()),
            children: vec![a_id, b_id],
            material: None,
            ..Default::default()
        };
        let c_id = tree.insert(c.clone());
        tree.set_root(Some(c_id));

        let mut actual = vec![];
        for item in tree.iter() {
            actual.push(item);
        }

        let expected = vec![
            (&c, &c, Vector2::new(0.0, 0.0)),
            (&c, &a, Vector2::new(3.0, 3.0)),
            (&a, &a_child, Vector2::new(4.0, 4.0)),
            (&c, &b, Vector2::new(3.0, 3.0)),
            (&b, &b_child, Vector2::new(5.0, 5.0)),
        ];
        for (i, _) in actual.iter().enumerate() {
            assert_eq!(expected[i], actual[i]);
        }
    }

    #[test]
    fn lbox_partial_eq_with_different_materials_returns_false() {
        let lbox_a = LayoutBox {
            material: Some(Material::filled(Color::red())),
            ..a_layout_box()
        };
        let lbox_b = LayoutBox {
            material: Some(Material::filled(Color::green())),
            ..a_layout_box()
        };
        assert_ne!(lbox_a, lbox_b);
    }

    fn a_layout_box() -> LayoutBox {
        LayoutBox {
            bounds: Rect::from_size((10.0, 10.0)),
            children: vec![],
            material: Some(Material::filled(Color::transparent())),
            ..Default::default()
        }
    }
}
