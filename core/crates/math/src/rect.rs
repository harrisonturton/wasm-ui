use crate::Vector2;

/// A 2-dimensional rectangle.
#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug, Default)]
pub struct Rect {
    pub min: Vector2,
    pub max: Vector2,
}

impl Rect {
    /// Construct a new vector that starts at the point described by `min` and
    /// ends at the point described by `max`.
    #[inline]
    pub fn new(min: Vector2, max: Vector2) -> Self {
        Self { min, max }
    }

    pub fn zero() -> Self {
        Self {
            min: Vector2::zero(),
            max: Vector2::zero(),
        }
    }

    pub fn from_pos<I: Into<Vector2>>(pos: I, size: I) -> Self {
        let pos = pos.into();
        Self {
            min: pos,
            max: pos + size.into(),
        }
    }

    pub fn from_size<I: Into<Vector2>>(max: I) -> Self {
        Self {
            min: Vector2::zero(),
            max: max.into(),
        }
    }

    /// Get the width and height of the rectangle.
    #[inline]
    pub fn size(self) -> Vector2 {
        let width = self.max.x - self.min.x;
        let height = self.max.y - self.min.y;
        Vector2::new(width, height)
    }

    /// Move the rectangle by the provided amount.
    #[inline]
    pub fn translate(self, amount: Vector2) -> Rect {
        Rect::new(self.min + amount, self.max + amount)
    }

    /// Check if the rectangle intersects a point.
    #[inline]
    pub fn intersects(self, point: Vector2) -> bool {
        let intersects_x = self.min.x < point.x && point.x < self.max.x;
        let intersects_y = self.min.y < point.y && point.y < self.max.y;
        intersects_x && intersects_y
    }
}
