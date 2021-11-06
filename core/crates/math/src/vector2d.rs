use crate::util;

#[repr(C)]
#[derive(Copy, Clone, Default, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl Vector2D {
    pub fn new(x: f32, y: f32) -> Vector2D {
        Vector2D { x, y }
    }

    pub fn zero() -> Vector2D {
        Vector2D::new(0.0, 0.0)
    }

    pub fn magnitude(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn dot(self, rhs: Vector2D) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y)
    }
}

impl std::cmp::PartialEq for Vector2D {
    fn eq(&self, rhs: &Vector2D) -> bool {
        let eq_x = util::float_eq(self.x, rhs.x);
        let eq_y = util::float_eq(self.y, rhs.y);
        eq_x && eq_y
    }
}

impl std::ops::Add<Vector2D> for Vector2D {
    type Output = Vector2D;

    fn add(self, rhs: Vector2D) -> Vector2D {
        Vector2D::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::Sub<Vector2D> for Vector2D {
    type Output = Vector2D;

    fn sub(self, rhs: Vector2D) -> Vector2D {
        Vector2D::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl std::ops::Mul<f32> for Vector2D {
    type Output = Vector2D;

    fn mul(self, rhs: f32) -> Vector2D {
        Vector2D::new(self.x * rhs, self.y * rhs)
    }
}

impl From<[f32; 2]> for Vector2D {
    fn from(lhs: [f32; 2]) -> Vector2D {
        Vector2D::new(lhs[0], lhs[1])
    }
}

impl From<(f32, f32)> for Vector2D {
    fn from(lhs: (f32, f32)) -> Vector2D {
        Vector2D::new(lhs.0, lhs.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector2d_eq_succeeds() {
        let a = Vector2D::new(0.0, 0.0);
        let b = Vector2D::new(0.0, 0.0);

        let expected = true;
        let actual = a == b;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_vector2d_eq_fails() {
        let a = Vector2D::new(0.0, 0.0);
        let b = Vector2D::new(10.0, 10.0);

        let expected = false;
        let actual = a == b;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_vector2d_addition_zero() {
        let a = Vector2D::new(0.0, 0.0);
        let b = Vector2D::new(10.0, 10.0);

        let expected = Vector2D::new(10.0, 10.0);
        let actual = a + b;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_vector2d_scalar_mul() {
        let a = Vector2D::new(1.0, 1.0);
        let b = 10.0;

        let expected = Vector2D::new(10.0, 10.0);
        let actual = a * b;
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_vector2d_2tuple_into() {
        let tuple = (100.0, 50.0);

        let expected = Vector2D::new(100.0, 50.0);
        let actual: Vector2D = tuple.into();
        assert_eq!(expected, actual);
    }
}
