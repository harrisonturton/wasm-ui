use crate::util;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vector3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3D {
    pub fn new(x: f32, y: f32, z: f32) -> Vector3D {
        Vector3D { x, y, z }
    }

    pub fn zero() -> Vector3D {
        Vector3D::new(0.0, 0.0, 0.0)
    }

    pub fn magnitude(self) -> f32 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    pub fn dot(self, rhs: Vector3D) -> f32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z + rhs.z)
    }
}

impl std::cmp::PartialEq for Vector3D {
    fn eq(&self, rhs: &Vector3D) -> bool {
        let eq_x = util::float_eq(self.x, rhs.x);
        let eq_y = util::float_eq(self.y, rhs.y);
        let eq_z = util::float_eq(self.z, rhs.z);
        eq_x && eq_y && eq_z
    }
}

impl std::ops::Add<Vector3D> for Vector3D {
    type Output = Vector3D;

    fn add(self, rhs: Vector3D) -> Vector3D {
        Vector3D::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub<Vector3D> for Vector3D {
    type Output = Vector3D;

    fn sub(self, rhs: Vector3D) -> Vector3D {
        Vector3D::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Mul<f32> for Vector3D {
    type Output = Vector3D;

    fn mul(self, rhs: f32) -> Vector3D {
        Vector3D::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl From<[f32; 3]> for Vector3D {
    fn from(lhs: [f32; 3]) -> Vector3D {
        Vector3D::new(lhs[0], lhs[1], lhs[2])
    }
}

impl From<(f32, f32, f32)> for Vector3D {
    fn from(lhs: (f32, f32, f32)) -> Vector3D {
        Vector3D::new(lhs.0, lhs.1, lhs.2)
    }
}

impl From<Vector3D> for (f32, f32, f32) {
    fn from(lhs: Vector3D) -> (f32, f32, f32) {
        (lhs.x, lhs.y, lhs.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector3d_eq_succeeds() {
        let a = Vector3D::new(0.0, 0.0, 0.0);
        let b = Vector3D::new(0.0, 0.0, 0.0);

        let expected = true;
        let actual = a == b;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_vector3d_eq_fails() {
        let a = Vector3D::new(0.0, 0.0, 0.0);
        let b = Vector3D::new(10.0, 10.0, 10.0);

        let expected = false;
        let actual = a == b;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_vector3d_addition_zero() {
        let a = Vector3D::new(0.0, 0.0, 0.0);
        let b = Vector3D::new(10.0, 10.0, 10.0);

        let expected = Vector3D::new(10.0, 10.0, 10.0);
        let actual = a + b;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_vector3d_scalar_mul() {
        let a = Vector3D::new(1.0, 1.0, 1.0);
        let b = 10.0;

        let expected = Vector3D::new(10.0, 10.0, 10.0);
        let actual = a * b;
        assert_eq!(expected, actual);
    }
}
