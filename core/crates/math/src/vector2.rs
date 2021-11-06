use std::default::Default;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

/// A 2-dimensional vector.
#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Vector2 {
    /// The x component of the vector.
    pub x: f32,
    /// The y component of the vector.
    pub y: f32,
}

impl Vector2 {
    /// Construct a new vector using the provided components.
    #[inline]
    pub fn new(x: f32, y: f32) -> Vector2 {
        Vector2 { x, y }
    }

    /// Construct a new vector where all components are 0.
    #[inline]
    pub fn zero() -> Vector2 {
        Vector2::new(0.0, 0.0)
    }

    /// Check if the vector is zero.
    #[inline]
    pub fn is_zero(self) -> bool {
        self == Vector2::zero()
    }

    #[inline]
    pub fn up() -> Vector2 {
        Vector2::new(0.0, 1.0)
    }

    #[inline]
    pub fn is_up(self) -> bool {
        self == Vector2::up()
    }

    #[inline]
    pub fn down() -> Vector2 {
        Vector2::new(0.0, -1.0)
    }

    #[inline]
    pub fn is_down(self) -> bool {
        self == Vector2::down()
    }

    #[inline]
    pub fn left() -> Vector2 {
        Vector2::new(-1.0, 0.0)
    }

    #[inline]
    pub fn is_left(self) -> bool {
        self == Vector2::left()
    }

    #[inline]
    pub fn right() -> Vector2 {
        Vector2::new(1.0, 0.0)
    }

    #[inline]
    pub fn is_right(self) -> bool {
        self == Vector2::right()
    }

    /// Calculate the sum of the x and y components of the vector.
    #[inline]
    pub fn sum(self) -> f32 {
        self.x + self.y
    }

    /// Calculate the product of the x and y components of the vector.
    #[inline]
    pub fn product(self) -> f32 {
        self.x * self.y
    }

    /// Get the dot product of two vectors.
    #[inline]
    pub fn dot(lhs: Vector2, rhs: Vector2) -> f32 {
        (lhs.x * rhs.x) + (lhs.y * rhs.y)
    }

    /// Get the magnitude, or length, of the vector.
    #[inline]
    pub fn magnitude(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    /// Return a vector with a magnitude of 1.
    #[inline]
    pub fn normalized(self) -> Vector2 {
        let mag = self.magnitude();
        Vector2::new(self.x / mag, self.y / mag)
    }

    /// Get the smallest component of the vector.
    #[inline]
    pub fn min(self) -> f32 {
        if self.x < self.y {
            self.x
        } else {
            self.y
        }
    }

    /// Get the largest component of the vector.
    #[inline]
    pub fn max(self) -> f32 {
        if self.x > self.y {
            self.x
        } else {
            self.y
        }
    }
}

// --------------------------------------------------
// Vector operations
// --------------------------------------------------

impl Default for Vector2 {
    /// Get the zero vector.
    #[inline]
    fn default() -> Vector2 {
        Vector2::zero()
    }
}

impl Neg for Vector2 {
    type Output = Vector2;

    /// Flip the sign on all the components in the vector.
    #[inline]
    fn neg(self) -> Vector2 {
        Vector2::new(-self.x, -self.y)
    }
}

impl Add<Vector2> for Vector2 {
    type Output = Vector2;

    /// Add two vectors together. The result will have each component be the sum
    /// of the original two components.
    #[inline]
    fn add(self, rhs: Vector2) -> Vector2 {
        Vector2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign<Vector2> for Vector2 {
    /// Add the components of another vector.
    fn add_assign(&mut self, rhs: Vector2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub<Vector2> for Vector2 {
    type Output = Vector2;

    /// Subtract two vectors together. The result will have each component be the difference
    /// of the original two components.
    #[inline]
    fn sub(self, rhs: Vector2) -> Vector2 {
        Vector2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign<Vector2> for Vector2 {
    /// Subtract the components of another vector.
    fn sub_assign(&mut self, rhs: Vector2) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<Vector2> for Vector2 {
    type Output = Vector2;

    /// Multiply two vectors together. The result will have each component be the multiplication
    /// of the original two components.
    #[inline]
    fn mul(self, rhs: Vector2) -> Vector2 {
        Vector2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl MulAssign<Vector2> for Vector2 {
    /// Multiply by the components of another vector.
    fn mul_assign(&mut self, rhs: Vector2) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl Div<Vector2> for Vector2 {
    type Output = Vector2;

    /// Divide two vectors together. The result will have each component be the division
    /// of the original two components.
    #[inline]
    fn div(self, rhs: Vector2) -> Vector2 {
        Vector2::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl DivAssign<Vector2> for Vector2 {
    /// Divide by the components of another vector.
    fn div_assign(&mut self, rhs: Vector2) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl Rem<Vector2> for Vector2 {
    type Output = Vector2;

    /// Get the elementwise remainder of two vectors. The result will have each component be
    /// the remainder of the original two components.
    #[inline]
    fn rem(self, rhs: Vector2) -> Vector2 {
        Vector2::new(self.x % rhs.x, self.y % rhs.y)
    }
}

impl RemAssign for Vector2 {
    /// Get the elementwise remainder using the components of another vector.
    fn rem_assign(&mut self, rhs: Vector2) {
        self.x %= rhs.x;
        self.y %= rhs.y;
    }
}

// --------------------------------------------------
// Scalar operations
// --------------------------------------------------

impl Add<f32> for Vector2 {
    type Output = Vector2;

    /// Add a scalar to each component of the vector.
    #[inline]
    fn add(self, rhs: f32) -> Vector2 {
        Vector2::new(self.x + rhs, self.y + rhs)
    }
}

impl AddAssign<f32> for Vector2 {
    /// Add a scalar to each component of the vector.
    #[inline]
    fn add_assign(&mut self, rhs: f32) {
        self.x += rhs;
        self.y += rhs;
    }
}

impl Sub<f32> for Vector2 {
    type Output = Vector2;

    /// Subtract a scalar from each component of the vector.
    #[inline]
    fn sub(self, rhs: f32) -> Vector2 {
        Vector2::new(self.x - rhs, self.y - rhs)
    }
}

impl SubAssign<f32> for Vector2 {
    /// Subtract a scalar from each component of the vector.
    #[inline]
    fn sub_assign(&mut self, rhs: f32) {
        self.x -= rhs;
        self.y -= rhs;
    }
}

impl Mul<f32> for Vector2 {
    type Output = Vector2;

    /// Multiply each component of the vector by a scalar.
    #[inline]
    fn mul(self, rhs: f32) -> Vector2 {
        Vector2::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign<f32> for Vector2 {
    /// Multiply each component of the vector by a scalar.
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<f32> for Vector2 {
    type Output = Vector2;

    /// Divide each component of the vector by a scalar.
    #[inline]
    fn div(self, rhs: f32) -> Vector2 {
        Vector2::new(self.x / rhs, self.y / rhs)
    }
}

impl DivAssign<f32> for Vector2 {
    /// Divide each component of the vector by a scalar.
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Rem<f32> for Vector2 {
    type Output = Vector2;

    /// Get the remainder of each component after dividing by a scalar.
    #[inline]
    fn rem(self, rhs: f32) -> Vector2 {
        Vector2::new(self.x % rhs, self.y % rhs)
    }
}

impl RemAssign<f32> for Vector2 {
    /// Get the remainder after a component-wise division of a scalar.
    #[inline]
    fn rem_assign(&mut self, rhs: f32) {
        self.x %= rhs;
        self.y %= rhs;
    }
}
