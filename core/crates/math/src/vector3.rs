use bytemuck::{Pod, Zeroable};
use std::default::Default;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};

#[macro_export]
macro_rules! vector3 {
    ($x: expr, $y: expr, $z: expr) => {
        Vector3 {
            x: $x,
            y: $y,
            z: $z,
        }
    };
}

/// A 2-dimensional vector.
#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vector3 {
    /// The x component of the vector.
    pub x: f32,
    /// The y component of the vector.
    pub y: f32,
    /// The z component of the vector.
    pub z: f32,
}

impl Vector3 {
    /// Construct a new vector using the provided components.
    #[must_use]
    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x, y, z }
    }

    /// Construct a new vector where all components are 0.
    #[must_use]
    pub fn zero() -> Vector3 {
        Vector3::new(0.0, 0.0, 0.0)
    }

    /// Check if the vector is zero.
    #[must_use]
    pub fn is_zero(self) -> bool {
        self == Vector3::zero()
    }

    #[must_use]
    pub fn up() -> Vector3 {
        Vector3::new(0.0, 1.0, 0.0)
    }

    #[must_use]
    pub fn is_up(self) -> bool {
        self == Vector3::up()
    }

    #[must_use]
    pub fn down() -> Vector3 {
        Vector3::new(0.0, -1.0, 0.0)
    }

    #[must_use]
    pub fn is_down(self) -> bool {
        self == Vector3::down()
    }

    #[must_use]
    pub fn left() -> Vector3 {
        Vector3::new(-1.0, 0.0, 0.0)
    }

    #[must_use]
    pub fn is_left(self) -> bool {
        self == Vector3::left()
    }

    #[must_use]
    pub fn right() -> Vector3 {
        Vector3::new(1.0, 0.0, 0.0)
    }

    #[must_use]
    pub fn is_right(self) -> bool {
        self == Vector3::right()
    }

    #[must_use]
    pub fn forward() -> Vector3 {
        Vector3::new(0.0, 0.0, 1.0)
    }

    #[must_use]
    pub fn is_forward(self) -> bool {
        self == Vector3::forward()
    }

    #[must_use]
    pub fn backward() -> Vector3 {
        Vector3::new(0.0, 0.0, -1.0)
    }

    #[must_use]
    pub fn is_backward(self) -> bool {
        self == Vector3::backward()
    }

    /// Calculate the sum of the x and y components of the vector.
    #[must_use]
    pub fn sum(self) -> f32 {
        self.x + self.y + self.z
    }

    /// Calculate the product of the x and y components of the vector.
    #[must_use]
    pub fn product(self) -> f32 {
        self.x * self.y * self.z
    }

    /// Get the dot product of two vectors.
    #[must_use]
    pub fn dot(lhs: Vector3, rhs: Vector3) -> f32 {
        (lhs.x * rhs.x) + (lhs.y * rhs.y) + (lhs.z * rhs.z)
    }

    /// Get the magnitude, or length, of the vector.
    #[must_use]
    pub fn magnitude(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    /// Return a vector with a magnitude of 1.
    #[must_use]
    pub fn normalized(self) -> Vector3 {
        let mag = self.magnitude();
        Vector3::new(self.x / mag, self.y / mag, self.z / mag)
    }

    /// Get the smallest component of the vector.
    #[must_use]
    pub fn min(self) -> f32 {
        f32::min(f32::min(self.x, self.y), self.z)
    }

    /// Get the largest component of the vector.
    #[must_use]
    pub fn max(self) -> f32 {
        f32::max(f32::max(self.x, self.y), self.z)
    }
}

// --------------------------------------------------
// Vector operations
// --------------------------------------------------

impl Default for Vector3 {
    /// Get the zero vector.
    #[must_use]
    fn default() -> Vector3 {
        Vector3::zero()
    }
}

impl Neg for Vector3 {
    type Output = Vector3;

    /// Flip the sign on all the components in the vector.
    #[must_use]
    fn neg(self) -> Vector3 {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}

impl Add<Vector3> for Vector3 {
    type Output = Vector3;

    /// Add two vectors together. The result will have each component be the sum
    /// of the original two components.
    #[must_use]
    fn add(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign<Vector3> for Vector3 {
    /// Add the components of another vector.
    fn add_assign(&mut self, rhs: Vector3) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub<Vector3> for Vector3 {
    type Output = Vector3;

    /// Subtract two vectors together. The result will have each component be the difference
    /// of the original two components.
    #[must_use]
    fn sub(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign<Vector3> for Vector3 {
    /// Subtract the components of another vector.
    fn sub_assign(&mut self, rhs: Vector3) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Mul<Vector3> for Vector3 {
    type Output = Vector3;

    /// Multiply two vectors together. The result will have each component be the multiplication
    /// of the original two components.
    #[must_use]
    fn mul(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl MulAssign<Vector3> for Vector3 {
    /// Multiply by the components of another vector.
    fn mul_assign(&mut self, rhs: Vector3) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl Div<Vector3> for Vector3 {
    type Output = Vector3;

    /// Divide two vectors together. The result will have each component be the division
    /// of the original two components.
    #[must_use]
    fn div(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl DivAssign<Vector3> for Vector3 {
    /// Divide by the components of another vector.
    fn div_assign(&mut self, rhs: Vector3) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl Rem<Vector3> for Vector3 {
    type Output = Vector3;

    /// Get the elementwise remainder of two vectors. The result will have each component be
    /// the remainder of the original two components.
    #[must_use]
    fn rem(self, rhs: Vector3) -> Vector3 {
        Vector3::new(self.x % rhs.x, self.y % rhs.y, self.z % rhs.z)
    }
}

impl RemAssign for Vector3 {
    /// Get the elementwise remainder using the components of another vector.
    fn rem_assign(&mut self, rhs: Vector3) {
        self.x %= rhs.x;
        self.y %= rhs.y;
        self.z %= rhs.z;
    }
}

// --------------------------------------------------
// Scalar operations
// --------------------------------------------------

impl Add<f32> for Vector3 {
    type Output = Vector3;

    /// Add a scalar to each component of the vector.
    #[must_use]
    fn add(self, rhs: f32) -> Vector3 {
        Vector3::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl AddAssign<f32> for Vector3 {
    /// Add a scalar to each component of the vector.
    fn add_assign(&mut self, rhs: f32) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}

impl Sub<f32> for Vector3 {
    type Output = Vector3;

    /// Subtract a scalar from each component of the vector.
    #[must_use]
    fn sub(self, rhs: f32) -> Vector3 {
        Vector3::new(self.x - rhs, self.y - rhs, self.z - rhs)
    }
}

impl SubAssign<f32> for Vector3 {
    /// Subtract a scalar from each component of the vector.
    fn sub_assign(&mut self, rhs: f32) {
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;
    }
}

impl Mul<f32> for Vector3 {
    type Output = Vector3;

    /// Multiply each component of the vector by a scalar.
    #[must_use]
    fn mul(self, rhs: f32) -> Vector3 {
        Vector3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl MulAssign<f32> for Vector3 {
    /// Multiply each component of the vector by a scalar.
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Div<f32> for Vector3 {
    type Output = Vector3;

    /// Divide each component of the vector by a scalar.
    #[must_use]
    fn div(self, rhs: f32) -> Vector3 {
        Vector3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl DivAssign<f32> for Vector3 {
    /// Divide each component of the vector by a scalar.
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Rem<f32> for Vector3 {
    type Output = Vector3;

    /// Get the remainder of each component after dividing by a scalar.
    #[must_use]
    fn rem(self, rhs: f32) -> Vector3 {
        Vector3::new(self.x % rhs, self.y % rhs, self.z % rhs)
    }
}

impl RemAssign<f32> for Vector3 {
    /// Get the remainder after a component-wise division of a scalar.
    fn rem_assign(&mut self, rhs: f32) {
        self.x %= rhs;
        self.y %= rhs;
        self.z %= rhs;
    }
}

// --------------------------------------------------
// Transform into Vector3
// --------------------------------------------------

impl From<[f32; 3]> for Vector3 {
    fn from(lhs: [f32; 3]) -> Vector3 {
        Vector3::new(lhs[0], lhs[1], lhs[2])
    }
}

impl From<(f32, f32, f32)> for Vector3 {
    fn from(lhs: (f32, f32, f32)) -> Vector3 {
        let (x, y, z) = lhs;
        Vector3::new(x, y, z)
    }
}

// --------------------------------------------------
// Transform from Vector3
// --------------------------------------------------

impl From<Vector3> for [f32; 3] {
    fn from(lhs: Vector3) -> [f32; 3] {
        [lhs.x, lhs.y, lhs.z]
    }
}

impl From<Vector3> for (f32, f32, f32) {
    fn from(lhs: Vector3) -> (f32, f32, f32) {
        (lhs.x, lhs.y, lhs.z)
    }
}
