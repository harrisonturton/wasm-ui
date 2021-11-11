use std::default::Default;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};
use bytemuck::{Pod, Zeroable};

#[macro_export]
macro_rules! vector4 {
    ($x: expr, $y: expr, $z: expr, $w: expr) => {
        Vector4 { x: $x, y: $y, z: $z, w: $w }
    };
}

/// A 2-dimensional vector.
#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vector4 {
    /// The x component of the vector.
    pub x: f32,
    /// The y component of the vector.
    pub y: f32,
    /// The z component of the vector.
    pub z: f32,
    /// The w component of the vector.
    pub w: f32,
}

impl Vector4 {
    /// Construct a new vector using the provided components.
    #[inline]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vector4 {
        Vector4 { x, y, z, w }
    }

    /// Construct a new vector where all components are 0.
    #[inline]
    pub fn zero() -> Vector4 {
        Vector4::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Check if the vector is zero.
    #[inline]
    pub fn is_zero(self) -> bool {
        self == Vector4::zero()
    }

    #[inline]
    pub fn up() -> Vector4 {
        Vector4::new(0.0, 1.0, 0.0, 0.0)
    }

    #[inline]
    pub fn is_up(self) -> bool {
        self == Vector4::up()
    }

    #[inline]
    pub fn down() -> Vector4 {
        Vector4::new(0.0, -1.0, 0.0, 0.0)
    }

    #[inline]
    pub fn is_down(self) -> bool {
        self == Vector4::down()
    }

    #[inline]
    pub fn left() -> Vector4 {
        Vector4::new(-1.0, 0.0, 0.0, 0.0)
    }

    #[inline]
    pub fn is_left(self) -> bool {
        self == Vector4::left()
    }

    #[inline]
    pub fn right() -> Vector4 {
        Vector4::new(1.0, 0.0, 0.0, 0.0)
    }

    #[inline]
    pub fn is_right(self) -> bool {
        self == Vector4::right()
    }

    #[inline]
    pub fn forward() -> Vector4 {
        Vector4::new(0.0, 0.0, 1.0, 0.0)
    }

    #[inline]
    pub fn is_forward(self) -> bool {
        self == Vector4::forward()
    }

    #[inline]
    pub fn backward() -> Vector4 {
        Vector4::new(0.0, 0.0, -1.0, 0.0)
    }

    #[inline]
    pub fn is_backward(self) -> bool {
        self == Vector4::backward()
    }

    /// Calculate the sum of the x and y components of the vector.
    #[inline]
    pub fn sum(self) -> f32 {
        self.x + self.y + self.z + self.w
    }

    /// Calculate the product of the x and y components of the vector.
    #[inline]
    pub fn product(self) -> f32 {
        self.x * self.y * self.z * self.w
    }

    /// Get the dot product of two vectors.
    #[inline]
    pub fn dot(lhs: Vector4, rhs: Vector4) -> f32 {
        (lhs.x * rhs.x) + (lhs.y * rhs.y) + (lhs.z * rhs.z) + (lhs.w * rhs.w)
    }

    /// Get the magnitude, or length, of the vector.
    #[inline]
    pub fn magnitude(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2)).sqrt()
    }

    /// Return a vector with a magnitude of 1.
    #[inline]
    pub fn normalized(self) -> Vector4 {
        let mag = self.magnitude();
        Vector4::new(self.x / mag, self.y / mag, self.z / mag, self.w / mag)
    }

    /// Get the smallest component of the vector.
    #[inline]
    pub fn min(self) -> f32 {
        f32::min(f32::min(f32::min(self.x, self.y), self.z), self.w)
    }

    /// Get the largest component of the vector.
    #[inline]
    pub fn max(self) -> f32 {
        f32::max(f32::max(f32::max(self.x, self.y), self.z), self.w)
    }
}

// --------------------------------------------------
// Vector operations
// --------------------------------------------------

impl Default for Vector4 {
    /// Get the zero vector.
    #[inline]
    fn default() -> Vector4 {
        Vector4::zero()
    }
}

impl Neg for Vector4 {
    type Output = Vector4;

    /// Flip the sign on all the components in the vector.
    #[inline]
    fn neg(self) -> Vector4 {
        Vector4::new(-self.x, -self.y, -self.z, -self.w)
    }
}

impl Add<Vector4> for Vector4 {
    type Output = Vector4;

    /// Add two vectors together. The result will have each component be the sum
    /// of the original components.
    #[inline]
    fn add(self, rhs: Vector4) -> Vector4 {
        Vector4::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z, self.w + rhs.w)
    }
}

impl AddAssign<Vector4> for Vector4 {
    /// Add the components of another vector.
    fn add_assign(&mut self, rhs: Vector4) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.w += self.w;
    }
}

impl Sub<Vector4> for Vector4 {
    type Output = Vector4;

    /// Subtract two vectors together. The result will have each component be the difference
    /// of the original components.
    #[inline]
    fn sub(self, rhs: Vector4) -> Vector4 {
        Vector4::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z, self.w - rhs.w)
    }
}

impl SubAssign<Vector4> for Vector4 {
    /// Subtract the components of another vector.
    fn sub_assign(&mut self, rhs: Vector4) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.w -= rhs.w;
    }
}

impl Mul<Vector4> for Vector4 {
    type Output = Vector4;

    /// Multiply two vectors together. The result will have each component be the multiplication
    /// of the original two components.
    #[inline]
    fn mul(self, rhs: Vector4) -> Vector4 {
        Vector4::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z, self.w * rhs.w)
    }
}

impl MulAssign<Vector4> for Vector4 {
    /// Multiply by the components of another vector.
    fn mul_assign(&mut self, rhs: Vector4) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
        self.w *= rhs.w;
    }
}

impl Div<Vector4> for Vector4 {
    type Output = Vector4;

    /// Divide two vectors together. The result will have each component be the division
    /// of the original components.
    #[inline]
    fn div(self, rhs: Vector4) -> Vector4 {
        Vector4::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z, self.w / rhs.w)
    }
}

impl DivAssign<Vector4> for Vector4 {
    /// Divide by the components of another vector.
    fn div_assign(&mut self, rhs: Vector4) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
        self.w /= rhs.w;
    }
}

impl Rem<Vector4> for Vector4 {
    type Output = Vector4;

    /// Get the elementwise remainder of two vectors. The result will have each component be
    /// the remainder of the original two components.
    #[inline]
    fn rem(self, rhs: Vector4) -> Vector4 {
        Vector4::new(self.x % rhs.x, self.y % rhs.y, self.z % rhs.z, self.w % rhs.w)
    }
}

impl RemAssign for Vector4 {
    /// Get the elementwise remainder using the components of another vector.
    fn rem_assign(&mut self, rhs: Vector4) {
        self.x %= rhs.x;
        self.y %= rhs.y;
        self.z %= rhs.z;
        self.w %= rhs.w;
    }
}

// --------------------------------------------------
// Scalar operations
// --------------------------------------------------

impl Add<f32> for Vector4 {
    type Output = Vector4;

    /// Add a scalar to each component of the vector.
    #[inline]
    fn add(self, rhs: f32) -> Vector4 {
        Vector4::new(self.x + rhs, self.y + rhs, self.z + rhs, self.w + rhs)
    }
}

impl AddAssign<f32> for Vector4 {
    /// Add a scalar to each component of the vector.
    #[inline]
    fn add_assign(&mut self, rhs: f32) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
        self.w += rhs;
    }
}

impl Sub<f32> for Vector4 {
    type Output = Vector4;

    /// Subtract a scalar from each component of the vector.
    #[inline]
    fn sub(self, rhs: f32) -> Vector4 {
        Vector4::new(self.x - rhs, self.y - rhs, self.z - rhs, self.w - rhs)
    }
}

impl SubAssign<f32> for Vector4 {
    /// Subtract a scalar from each component of the vector.
    #[inline]
    fn sub_assign(&mut self, rhs: f32) {
        self.x -= rhs;
        self.y -= rhs;
        self.z -= rhs;
        self.w -= rhs;
    }
}

impl Mul<f32> for Vector4 {
    type Output = Vector4;

    /// Multiply each component of the vector by a scalar.
    #[inline]
    fn mul(self, rhs: f32) -> Vector4 {
        Vector4::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

impl MulAssign<f32> for Vector4 {
    /// Multiply each component of the vector by a scalar.
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
        self.w *= rhs;
    }
}

impl Div<f32> for Vector4 {
    type Output = Vector4;

    /// Divide each component of the vector by a scalar.
    #[inline]
    fn div(self, rhs: f32) -> Vector4 {
        Vector4::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
    }
}

impl DivAssign<f32> for Vector4 {
    /// Divide each component of the vector by a scalar.
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
        self.w /= rhs;
    }
}

impl Rem<f32> for Vector4 {
    type Output = Vector4;

    /// Get the remainder of each component after dividing by a scalar.
    #[inline]
    fn rem(self, rhs: f32) -> Vector4 {
        Vector4::new(self.x % rhs, self.y % rhs, self.z % rhs, self.w % rhs)
    }
}

impl RemAssign<f32> for Vector4 {
    /// Get the remainder after a component-wise division of a scalar.
    #[inline]
    fn rem_assign(&mut self, rhs: f32) {
        self.x %= rhs;
        self.y %= rhs;
        self.z %= rhs;
        self.w %= rhs;
    }
}

// --------------------------------------------------
// Transform into Vector4
// --------------------------------------------------

impl From<[f32; 4]> for Vector4 {
    fn from(lhs: [f32; 4]) -> Vector4 {
        Vector4::new(lhs[0], lhs[1], lhs[2], lhs[3])
    }
}

impl From<(f32, f32, f32, f32)> for Vector4 {
    fn from(lhs: (f32, f32, f32, f32)) -> Vector4 {
        let (x, y, z, w) = lhs;
        Vector4::new(x, y, z, w)
    }
}

// --------------------------------------------------
// Transform from Vector4
// --------------------------------------------------

impl From<Vector4> for [f32; 4] {
    fn from(lhs: Vector4) -> [f32; 4] {
        [lhs.x, lhs.y, lhs.z, lhs.w]
    }
}

impl From<Vector4> for (f32, f32, f32, f32) {
    fn from(lhs: Vector4) -> (f32, f32, f32, f32) {
        (lhs.x, lhs.y, lhs.z, lhs.w)
    }
}
