use crate::util;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vector4D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vector4D {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vector4D {
        Vector4D { x, y, z, w }
    }

    pub fn zero() -> Vector4D {
        Vector4D::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn magnitude(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2) + self.w.powi(2)).sqrt()
    }

    pub fn dot(self, rhs: Vector4D) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z + rhs.z) + (self.w + rhs.w)
    }
}

impl std::ops::Index<usize> for Vector4D {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("index out of bounds on Vector4D"),
        }
    }
}

impl std::ops::IndexMut<usize> for Vector4D {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("index out of bounds on Vector4D"),
        }
    }
}

impl IntoIterator for Vector4D {
    type Item = f32;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self.x, self.y, self.z, self.w].into_iter()
    }
}

impl std::cmp::PartialEq for Vector4D {
    fn eq(&self, rhs: &Vector4D) -> bool {
        let eq_x = util::float_eq(self.x, rhs.x);
        let eq_y = util::float_eq(self.y, rhs.y);
        let eq_z = util::float_eq(self.z, rhs.z);
        let eq_w = util::float_eq(self.w, rhs.w);
        eq_x && eq_y && eq_z && eq_w
    }
}

impl std::ops::Add<Vector4D> for Vector4D {
    type Output = Vector4D;

    fn add(self, rhs: Vector4D) -> Vector4D {
        Vector4D::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

impl std::ops::Sub<Vector4D> for Vector4D {
    type Output = Vector4D;

    fn sub(self, rhs: Vector4D) -> Vector4D {
        Vector4D::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
            self.w - rhs.w,
        )
    }
}

impl std::ops::Mul<f32> for Vector4D {
    type Output = Vector4D;

    fn mul(self, rhs: f32) -> Vector4D {
        Vector4D::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

impl From<[f32; 4]> for Vector4D {
    fn from(lhs: [f32; 4]) -> Vector4D {
        Vector4D::new(lhs[0], lhs[1], lhs[2], lhs[3])
    }
}

impl From<(f32, f32, f32, f32)> for Vector4D {
    fn from(lhs: (f32, f32, f32, f32)) -> Vector4D {
        Vector4D::new(lhs.0, lhs.1, lhs.2, lhs.3)
    }
}

impl From<Vector4D> for [f32; 4] {
    fn from(lhs: Vector4D) -> [f32; 4] {
        [lhs.x, lhs.y, lhs.z, lhs.w]
    }
}

impl From<Vector4D> for (f32, f32, f32, f32) {
    fn from(lhs: Vector4D) -> (f32, f32, f32, f32) {
        (lhs.x, lhs.y, lhs.z, lhs.w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector4d_eq_succeeds() {
        let a = Vector4D::new(0.0, 0.0, 0.0, 0.0);
        let b = Vector4D::new(0.0, 0.0, 0.0, 0.0);

        let expected = true;
        let actual = a == b;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_vector4d_eq_fails() {
        let a = Vector4D::new(0.0, 0.0, 0.0, 0.0);
        let b = Vector4D::new(10.0, 10.0, 10.0, 10.0);

        let expected = false;
        let actual = a == b;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_vector4d_addition_zero() {
        let a = Vector4D::new(0.0, 0.0, 0.0, 7.0);
        let b = Vector4D::new(10.0, 10.0, 10.0, 3.0);

        let expected = Vector4D::new(10.0, 10.0, 10.0, 10.0);
        let actual = a + b;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_vector4d_scalar_mul() {
        let a = Vector4D::new(1.0, 1.0, 1.0, 1.0);
        let b = 10.0;

        let expected = Vector4D::new(10.0, 10.0, 10.0, 10.0);
        let actual = a * b;
        assert_eq!(expected, actual);
    }
}
