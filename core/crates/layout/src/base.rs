use math::Vector2;

// The position of the center of a widget as a fraction of the available area.
// The widget should not overflow at (0.0, 0.0) or at (1.0, 1.0), it should be
// clamped to the edges of the screen.
#[derive(PartialEq, Copy, Clone, Debug, Default)]
pub struct Alignment {
    pub x: f32,
    pub y: f32,
}

impl Alignment {
    #[must_use]
    pub fn new(x: f32, y: f32) -> Alignment {
        Alignment { x, y }
    }

    #[must_use]
    pub fn top_left() -> Alignment {
        Alignment::new(0.0, 0.0)
    }

    #[must_use]
    pub fn top_center() -> Alignment {
        Alignment::new(0.5, 0.0)
    }

    #[must_use]
    pub fn top_right() -> Alignment {
        Alignment::new(0.5, 0.0)
    }

    #[must_use]
    pub fn center_left() -> Alignment {
        Alignment::new(0.0, 0.5)
    }

    #[must_use]
    pub fn center() -> Alignment {
        Alignment::new(0.5, 0.5)
    }

    #[must_use]
    pub fn center_right() -> Alignment {
        Alignment::new(1.0, 0.5)
    }

    #[must_use]
    pub fn bottom_left() -> Alignment {
        Alignment::new(0.0, 1.0)
    }

    #[must_use]
    pub fn bottom_center() -> Alignment {
        Alignment::new(0.5, 1.0)
    }

    #[must_use]
    pub fn bottom_right() -> Alignment {
        Alignment::new(1.0, 1.0)
    }

    #[must_use]
    pub fn to_vector(&self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }
}

#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub struct EdgeInsets {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl EdgeInsets {
    #[must_use]
    pub fn zero() -> EdgeInsets {
        EdgeInsets::all(0.0)
    }

    #[must_use]
    pub fn all(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: inset,
            bottom: inset,
            left: inset,
            right: inset,
        }
    }

    #[must_use]
    pub fn vertical(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: inset,
            bottom: inset,
            left: 0.0,
            right: 0.0,
        }
    }

    #[must_use]
    pub fn horizontal(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: 0.0,
            bottom: 0.0,
            left: inset,
            right: inset,
        }
    }

    #[must_use]
    pub fn top(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: inset,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
        }
    }

    #[must_use]
    pub fn bottom(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: 0.0,
            bottom: inset,
            left: 0.0,
            right: 0.0,
        }
    }

    #[must_use]
    pub fn left(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: 0.0,
            bottom: 0.0,
            left: inset,
            right: 0.0,
        }
    }

    #[must_use]
    pub fn right(inset: f32) -> EdgeInsets {
        EdgeInsets {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: inset,
        }
    }

    #[must_use]
    pub fn total_height(&self) -> f32 {
        self.left + self.right
    }

    #[must_use]
    pub fn total_width(&self) -> f32 {
        self.top + self.bottom
    }

    #[must_use]
    pub fn total(&self) -> Vector2 {
        let x = self.left + self.right;
        let y = self.top + self.bottom;
        Vector2::new(x, y)
    }

    #[must_use]
    pub fn min(&self) -> Vector2 {
        Vector2::new(self.left, self.top)
    }

    #[must_use]
    pub fn max(&self) -> Vector2 {
        Vector2::new(self.right, self.bottom)
    }
}
