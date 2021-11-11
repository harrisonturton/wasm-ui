use math::{Vector2, Rect};

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
pub trait Layout<C> {
    fn layout(&self, tree: &mut LayoutTree<C>) -> SizedLayoutBox<C>;
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
#[derive(Clone, Debug)]
pub struct SizedLayoutBox<C> {
    pub size: Vector2,
    pub content: C,
    pub children: Vec<LayoutBoxId>,
}

/// An element that has finished layout. It has been been sized and positioned.
#[derive(Clone, Debug)]
pub struct LayoutBox<C> {
    pub rect: Rect,
    pub content: C,
    pub children: Vec<LayoutBoxId>,
}

impl<C> LayoutBox<C> {
    /// Convenience method to turn a [SizedLayoutBox] into a [LayoutBox]. This
    /// is handy when implementing the [Layout] trait.
    pub fn from_child<I>(child: SizedLayoutBox<C>, pos: I) -> LayoutBox<C>
    where
        I: Into<Vector2>
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
pub struct LayoutTree<C> {
    root: Option<LayoutBoxId>,
    boxes: Vec<LayoutBox<C>>,
}

impl<C> LayoutTree<C> {
    /// Create a new empty [LayoutTree].
    pub fn new() -> LayoutTree<C> {
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
    pub fn insert(&mut self, lbox: LayoutBox<C>) -> LayoutBoxId {
        self.boxes.push(lbox);
        self.boxes.len() -1
    }

    /// Get a reference to the [LayoutBox] indexed by a [LayoutBoxId].
    pub fn get(&mut self, id: LayoutBoxId) -> Option<&LayoutBox<C>> {
        self.boxes.get(id)
    }
}