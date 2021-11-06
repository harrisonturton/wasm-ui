use crate::{Size, Vector2D};

#[derive(Debug, Clone)]
pub struct Rect {
    pub max: Vector2D,
    pub min: Vector2D,
}

impl Rect {
    pub fn new(min: Vector2D, max: Vector2D) -> Rect {
        Rect { min, max }
    }

    pub fn zero() -> Rect {
        Rect::new(Vector2D::zero(), Vector2D::zero())
    }

    pub fn size(&self) -> Size {
        let width = self.max.x - self.min.x;
        let height = self.max.y - self.min.y;
        Size::new(width, height)
    }

    pub fn translate(self, amount: Vector2D) -> Rect {
        let max = self.max + amount;
        let min = self.min + amount;
        Rect::new(min, max)
    }

    pub fn intersects(self, point: Vector2D) -> bool {
        let in_x = self.min.x < point.x && point.x < self.max.x;
        let in_y = self.min.y < point.y && point.y < self.max.y;
        in_x && in_y
    }

    pub fn with_size(self, width: f32, height: f32) -> Rect {
        let max_x = self.min.x + width;
        let max_y = self.min.y + height;
        Rect::new(self.min, Vector2D::new(max_x, max_y))
    }

    pub fn with_pos(self, x: f32, y: f32) -> Rect {
        let pos = Vector2D::new(x, y);
        let width = self.max.x - self.min.x;
        let height = self.max.y - self.min.y;
        Rect::new(pos, pos + Vector2D::new(width, height))
    }
}

impl std::cmp::PartialEq for Rect {
    fn eq(&self, rhs: &Rect) -> bool {
        let eq_min = self.min == rhs.min;
        let eq_max = self.max == rhs.max;
        eq_min && eq_max
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_with_pos() {
        let expected = Rect::new(Vector2D::new(100.0, 100.0), Vector2D::new(100.0, 100.0));
        let actual = Rect::zero().with_pos(100.0, 100.0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_rect_translate() {
        let rect = Rect::zero().with_size(10.0, 10.0);
        let translate = Vector2D::new(100.0, 100.0);

        let expected = Rect::zero().with_size(10.0, 10.0).with_pos(100.0, 100.0);
        let actual = rect.translate(translate);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_rect_intersect_point() {
        let rect = Rect::zero().with_pos(100.0, 100.0).with_size(50.0, 50.0);
        let point = Vector2D::new(125.0, 125.0);

        let expected = true;
        let actual = rect.intersects(point);
        assert_eq!(actual, expected);
    }
}
