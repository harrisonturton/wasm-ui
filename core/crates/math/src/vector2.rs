use std::default::Default;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};
use bytemuck::{Pod, Zeroable};
use crate::Vector3;

#[macro_export]
macro_rules! vector2 {
    ($x: expr, $y: expr) => {
        Vector2 { x: $x, y: $y }
    };
}

/// A 2-dimensional vector.
#[repr(C)]
#[derive(PartialEq, Copy, Clone, Debug, Pod, Zeroable)]
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

// --------------------------------------------------
// Transform into Vector2
// --------------------------------------------------

impl From<[f32; 2]> for Vector2 {
    fn from(lhs: [f32; 2]) -> Vector2 {
        Vector2::new(lhs[0], lhs[1])
    }
}

impl From<(f32, f32)> for Vector2 {
    fn from(lhs: (f32, f32)) -> Vector2 {
        let (x, y) = lhs;
        Vector2::new(x, y)
    }
}

// --------------------------------------------------
// Transform from Vector2
// --------------------------------------------------

impl From<Vector2> for Vector3 {
    fn from(lhs: Vector2) -> Vector3 {
        Vector3::new(lhs.x, lhs.y, 0.0)
    }
}

impl From<Vector2> for [f32; 2] {
    fn from(lhs: Vector2) -> [f32; 2] {
        [lhs.x, lhs.y]
    }
}

impl From<Vector2> for (f32, f32) {
    fn from(lhs: Vector2) -> (f32, f32) {
        (lhs.x, lhs.y)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macro_constructor_is_equal_to_new_constructor() {
        let actual = vector2!(199.0, -512.0);
        let expected = Vector2::new(199.0, -512.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn up_constructor_is_up() {
        let actual = Vector2::up();
        let expected = Vector2::new(0.0, 1.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn down_constructor_is_down() {
        let actual = Vector2::down();
        let expected = Vector2::new(0.0, -1.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn left_constructor_is_left() {
        let actual = Vector2::left();
        let expected = Vector2::new(-1.0, 0.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn right_constructor_is_right() {
        let actual = Vector2::right();
        let expected = Vector2::new(1.0, 0.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn sum_gives_expected_result() {
        let vec = vector2!(-50.0, 100.0);

        let actual = vec.sum();
        let expected = 50.0;
        assert_eq!(expected, actual);
    }

    #[test]
    fn product_gives_expected_result() {
        let vec = vector2!(-50.0, 100.0);

        let actual = vec.product();
        let expected = -5000.0;
        assert_eq!(expected, actual);
    }

    #[test]
    fn dot_product_is_zero_for_equal_vectors() {
        let a = vector2!(0.0, 1.0);
        let b = vector2!(0.0, 1.0);

        let actual = Vector2::dot(a, b);
        let expected = 1.0;
        assert_eq!(expected, actual);
    }

    #[test]
    fn dot_product_gives_expected_result() {
        let a = vector2!(50.0, -100.0);
        let b = vector2!(-20.0, 100.0);

        let actual = Vector2::dot(a, b);
        let expected = -11000.0;
        assert_eq!(expected, actual);
    }

    #[test]
    fn magnitude_for_unit_vectors_are_zero() {
        let up = Vector2::up();
        let up_magnitude = up.magnitude();
        assert_eq!(1.0, up_magnitude);

        let down = Vector2::down();
        let down_magnitude = down.magnitude();
        assert_eq!(1.0, down_magnitude);

        let left = Vector2::left();
        let left_magnitude = left.magnitude();
        assert_eq!(1.0, left_magnitude);

        let right = Vector2::right();
        let right_magnitude = right.magnitude();
        assert_eq!(1.0, right_magnitude);
    }

    #[test]
    fn normalize_gives_correct_result_for_non_unit_vector() {
        let vec = vector2!(0.0, 100.0);
        let actual = vec.normalized();
        let expected = vector2!(0.0, 1.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn normalize_gives_correct_result_for_unit_vector() {
        let vec = Vector2::up();
        let actual = vec.normalized();
        let expected = Vector2::up();
        assert_eq!(expected, actual);
    }

    #[test]
    fn min_gives_expected_result_when_x_is_less_than_y() {
        let vec = vector2!(-100.0, 50.0);
        assert_eq!(-100.0, vec.min());
    }

    #[test]
    fn min_gives_expected_result_when_y_is_less_than_x() {
        let vec = vector2!(50.0, -100.0);
        assert_eq!(-100.0, vec.min());
    }

    #[test]
    fn max_gives_expected_result_when_x_is_greater_than_y() {
        let vec = vector2!(50.0, -100.0);
        assert_eq!(50.0, vec.max());
    }

    #[test]
    fn max_gives_expected_result_when_y_is_greater_than_x() {
        let vec = vector2!(-100.0, 50.0);
        assert_eq!(50.0, vec.max());
    }

    #[test]
    fn default_constructor_is_zero() {
        let actual = Default::default();
        let expected = Vector2::zero();
        assert_eq!(expected, actual);
    }

    #[test]
    fn negate_gives_expected_result() {
        let vec = vector2!(-100.0, 50.0);

        let actual = -vec;
        let expected = vector2!(100.0, -50.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn adding_vector_to_vector_gives_expected_result() {
        let a = vector2!(-100.0, 50.0);
        let b = vector2!(100.0, -50.0);

        let actual = a + b;
        let expected = Vector2::zero();
        assert_eq!(expected, actual);
    }

    #[test]
    fn add_assign_vector_to_vector_gives_expected_result() {
        let mut actual = vector2!(-100.0, 50.0);
        actual += vector2!(100.0, -50.0);

        let expected = Vector2::zero();
        assert_eq!(expected, actual);
    }

    #[test]
    fn subtracting_vector_from_vector_gives_expected_result() {
        let a = vector2!(-100.0, 50.0);
        let b = vector2!(100.0, -50.0);

        let actual = a - b;
        let expected = vector2!(-200.0, 100.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn subtract_assign_vector_to_vector_gives_expected_result() {
        let mut actual = vector2!(-100.0, 50.0);
        actual -= vector2!(100.0, -50.0);

        let expected = vector2!(-200.0, 100.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn multiplying_vector_by_vector_gives_expected_result() {
        let a = vector2!(-100.0, 50.0);
        let b = vector2!(100.0, -50.0);

        let actual = a * b;
        let expected = vector2!(-10000.0, -2500.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn multiply_assign_vector_to_vector_gives_expected_result() {
        let mut actual = vector2!(-100.0, 50.0);
        actual *= vector2!(100.0, -50.0);

        let expected = vector2!(-10000.0, -2500.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn dividing_vector_by_vector_gives_expected_result() {
        let a = vector2!(-100.0, 50.0);
        let b = vector2!(100.0, -50.0);

        let actual = a / b;
        let expected = vector2!(-1.0, -1.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn div_assign_vector_to_vector_gives_expected_result() {
        let mut actual = vector2!(-100.0, 50.0);
        actual /= vector2!(100.0, -50.0);

        let expected = vector2!(-1.0, -1.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn remainder_vector_by_vector_gives_expected_result() {
        let a = vector2!(-100.0, 50.0);
        let b = vector2!(100.0, -50.0);

        let actual = a % b;
        let expected = Vector2::zero();
        assert_eq!(expected, actual);
    }

    #[test]
    fn remainder_assign_vector_to_vector_gives_expected_result() {
        let mut actual = vector2!(-100.0, 50.0);
        actual %= vector2!(100.0, -50.0);

        let expected = Vector2::zero();
        assert_eq!(expected, actual);
    }

    #[test]
    fn add_scalar_to_vector_gives_expected_result() {
        let vec = vector2!(-100.0, 50.0);
        let scalar = 100.0;

        let actual = vec + scalar;
        let expected = vector2!(0.0, 150.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn add_assign_scalar_to_vector_gives_expected_result() {
        let mut actual = vector2!(-100.0, 50.0);
        actual += 100.0;

        let expected = vector2!(0.0, 150.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn subtract_scalar_from_vector_gives_expected_result() {
        let vec = vector2!(-100.0, 50.0);
        let scalar = 100.0;

        let actual = vec - scalar;
        let expected = vector2!(-200.0, -50.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn subtract_assign_scalar_to_vector_gives_expected_result() {
        let mut actual = vector2!(-100.0, 50.0);
        actual -= 100.0;

        let expected = vector2!(-200.0, -50.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn multiply_vector_by_scalar_gives_expected_result() {
        let vec = vector2!(-100.0, 50.0);
        let scalar = 100.0;

        let actual = vec * scalar;
        let expected = vector2!(-10000.0, 5000.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn multiply_assign_scalar_to_vector_gives_expected_result() {
        let mut actual = vector2!(-100.0, 50.0);
        actual *= 100.0;

        let expected = vector2!(-10000.0, 5000.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn divide_vector_by_scalar_gives_expected_result() {
        let vec = vector2!(-100.0, 50.0);
        let scalar = 100.0;

        let actual = vec / scalar;
        let expected = vector2!(-1.0, 0.5);
        assert_eq!(expected, actual);
    }

    #[test]
    fn divide_assign_scalar_to_vector_gives_expected_result() {
        let mut actual = vector2!(-100.0, 50.0);
        actual /= 100.0;

        let expected = vector2!(-1.0, 0.5);
        assert_eq!(expected, actual);
    }

    #[test]
    fn remainder_vector_by_scalar_gives_expected_result() {
        let vec = vector2!(-100.0, 50.0);
        let scalar = 100.0;

        let actual = vec % scalar;
        let expected = vector2!(0.0, 50.0);
        assert_eq!(expected, actual);
    }

    #[test]
    fn remainder_assign_scalar_to_vector_gives_expected_result() {
        let mut actual = vector2!(-100.0, 50.0);
        actual %= 100.0;

        let expected = vector2!(0.0, 50.0);
        assert_eq!(expected, actual);
    }
}
