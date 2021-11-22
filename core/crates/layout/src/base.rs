use math::Vector2;

#[derive(Debug, Default)]
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
    pub fn min(&self) -> Vector2 {
        Vector2::new(self.left, self.top)
    }

    #[must_use]
    pub fn max(&self) -> Vector2 {
        Vector2::new(self.right, self.bottom)
    }
}
