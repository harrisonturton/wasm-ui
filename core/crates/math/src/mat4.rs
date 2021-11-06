use crate::util;
use crate::{Vector3D, Vector4D};

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Mat4(pub [f32; 16]);

impl Mat4 {
    pub fn new(elements: [f32; 16]) -> Self {
        Self(elements)
    }

    #[rustfmt::skip]
    pub fn new_zero() -> Self {
        Self([
            0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
        ])
    }

    #[rustfmt::skip]
    pub fn new_unit() -> Self {
        Self([
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[rustfmt::skip]
    pub fn new_scale<I>(amount: I) -> Self
    where
        I: Into<Vector3D>,
    {
        let (x, y, z) = amount.into().into();
        Self([
            x, 0.0, 0.0, 0.0,
            0.0, y, 0.0, 0.0,
            0.0, 0.0, z, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[rustfmt::skip]
    pub fn new_translate<I>(amount: I) -> Self
    where
        I: Into<Vector3D>,
    {
        let (x, y, z) = amount.into().into();
        Self([
            1.0, 0.0, 0.0, x,
            0.0, 1.0, 0.0, y,
            0.0, 0.0, 1.0, z,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn new_rotate<I>(amount: I) -> Self
    where
        I: Into<Vector3D>,
    {
        let (x, y, z) = amount.into().into();
        let x = Mat4::new_rotate_x(x);
        let y = Mat4::new_rotate_y(y);
        let z = Mat4::new_rotate_z(z);
        x * y * z
    }

    #[rustfmt::skip]
    pub fn new_rotate_z(amount: f32) -> Self {
        let rads = amount.to_radians();
        let a = rads.cos();
        let b = rads.sin();
        Self([
            a, -b, 0.0, 0.0,
            b, a, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[rustfmt::skip]
    pub fn new_rotate_x(amount: f32) -> Self {
        let rads = amount.to_radians();
        let a = rads.cos();
        let b = rads.sin();
        Self([
            1.0, 0.0, 0.0, 0.0,
            0.0, a, -b, 0.0,
            0.0, b, a, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    #[rustfmt::skip]
    pub fn new_rotate_y(amount: f32) -> Self {
        let rads = amount.to_radians();
        let a = rads.cos();
        let b = rads.sin();
        Self([
            a, 0.0, b, 0.0,
            0.0, 1.0, 0.0, 0.0,
            -b, 0.0, a, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ])
    }

    pub fn translate<I>(self, amount: I) -> Self
    where
        I: Into<Vector3D>,
    {
        self * Mat4::new_translate(amount)
    }

    pub fn translate_x(self, amount: f32) -> Self {
        self * Mat4::new_translate((amount, 0.0, 0.0))
    }

    pub fn translate_y(self, amount: f32) -> Self {
        self * Mat4::new_translate((0.0, amount, 0.0))
    }

    pub fn translate_z(self, amount: f32) -> Self {
        self * Mat4::new_translate((0.0, 0.0, amount))
    }

    pub fn scale<I>(self, amount: I) -> Self
    where
        I: Into<Vector3D>,
    {
        self * Mat4::new_scale(amount)
    }

    pub fn rotate<I>(self, amount: I) -> Self
    where
        I: Into<Vector3D>,
    {
        self * Mat4::new_rotate(amount)
    }

    pub fn rotate_x(self, amount: f32) -> Self {
        self * Mat4::new_rotate_x(amount)
    }

    pub fn rotate_y(self, amount: f32) -> Self {
        self * Mat4::new_rotate_y(amount)
    }

    pub fn rotate_z(self, amount: f32) -> Self {
        self * Mat4::new_rotate_z(amount)
    }

    pub fn inverse(self) -> Option<Self> {
        let m = self.0;
        let mut inv = self.0;

        inv[0] = m[5] * m[10] * m[15] - m[5] * m[11] * m[14] - m[9] * m[6] * m[15]
            + m[9] * m[7] * m[14]
            + m[13] * m[6] * m[11]
            - m[13] * m[7] * m[10];

        inv[4] = -m[4] * m[10] * m[15] + m[4] * m[11] * m[14] + m[8] * m[6] * m[15]
            - m[8] * m[7] * m[14]
            - m[12] * m[6] * m[11]
            + m[12] * m[7] * m[10];

        inv[8] = m[4] * m[9] * m[15] - m[4] * m[11] * m[13] - m[8] * m[5] * m[15]
            + m[8] * m[7] * m[13]
            + m[12] * m[5] * m[11]
            - m[12] * m[7] * m[9];

        inv[12] = -m[4] * m[9] * m[14] + m[4] * m[10] * m[13] + m[8] * m[5] * m[14]
            - m[8] * m[6] * m[13]
            - m[12] * m[5] * m[10]
            + m[12] * m[6] * m[9];

        inv[1] = -m[1] * m[10] * m[15] + m[1] * m[11] * m[14] + m[9] * m[2] * m[15]
            - m[9] * m[3] * m[14]
            - m[13] * m[2] * m[11]
            + m[13] * m[3] * m[10];

        inv[5] = m[0] * m[10] * m[15] - m[0] * m[11] * m[14] - m[8] * m[2] * m[15]
            + m[8] * m[3] * m[14]
            + m[12] * m[2] * m[11]
            - m[12] * m[3] * m[10];

        inv[9] = -m[0] * m[9] * m[15] + m[0] * m[11] * m[13] + m[8] * m[1] * m[15]
            - m[8] * m[3] * m[13]
            - m[12] * m[1] * m[11]
            + m[12] * m[3] * m[9];

        inv[13] = m[0] * m[9] * m[14] - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
            + m[8] * m[2] * m[13]
            + m[12] * m[1] * m[10]
            - m[12] * m[2] * m[9];

        inv[2] = m[1] * m[6] * m[15] - m[1] * m[7] * m[14] - m[5] * m[2] * m[15]
            + m[5] * m[3] * m[14]
            + m[13] * m[2] * m[7]
            - m[13] * m[3] * m[6];

        inv[6] = -m[0] * m[6] * m[15] + m[0] * m[7] * m[14] + m[4] * m[2] * m[15]
            - m[4] * m[3] * m[14]
            - m[12] * m[2] * m[7]
            + m[12] * m[3] * m[6];

        inv[10] = m[0] * m[5] * m[15] - m[0] * m[7] * m[13] - m[4] * m[1] * m[15]
            + m[4] * m[3] * m[13]
            + m[12] * m[1] * m[7]
            - m[12] * m[3] * m[5];

        inv[14] = -m[0] * m[5] * m[14] + m[0] * m[6] * m[13] + m[4] * m[1] * m[14]
            - m[4] * m[2] * m[13]
            - m[12] * m[1] * m[6]
            + m[12] * m[2] * m[5];

        inv[3] = -m[1] * m[6] * m[11] + m[1] * m[7] * m[10] + m[5] * m[2] * m[11]
            - m[5] * m[3] * m[10]
            - m[9] * m[2] * m[7]
            + m[9] * m[3] * m[6];

        inv[7] = m[0] * m[6] * m[11] - m[0] * m[7] * m[10] - m[4] * m[2] * m[11]
            + m[4] * m[3] * m[10]
            + m[8] * m[2] * m[7]
            - m[8] * m[3] * m[6];

        inv[11] = -m[0] * m[5] * m[11] + m[0] * m[7] * m[9] + m[4] * m[1] * m[11]
            - m[4] * m[3] * m[9]
            - m[8] * m[1] * m[7]
            + m[8] * m[3] * m[5];

        inv[15] = m[0] * m[5] * m[10] - m[0] * m[6] * m[9] - m[4] * m[1] * m[10]
            + m[4] * m[2] * m[9]
            + m[8] * m[1] * m[6]
            - m[8] * m[2] * m[5];

        let mut det = m[0] * inv[0] + m[1] * inv[4] + m[2] * inv[8] + m[3] * inv[12];
        if det == 0.0 {
            return None;
        }
        det = 1.0 / det;

        let mut inv_out = Mat4::new_unit();
        for (i, _) in inv.iter().enumerate() {
            inv_out.0[i] = inv[i] * det;
        }
        Some(inv_out)
    }
}

impl std::cmp::PartialEq for Mat4 {
    fn eq(&self, rhs: &Mat4) -> bool {
        for i in 0..16 {
            if !util::float_eq(self.0[i], rhs.0[i]) {
                return false;
            }
        }
        true
    }
}

impl std::ops::Add<Mat4> for Mat4 {
    type Output = Mat4;

    fn add(self, rhs: Mat4) -> Mat4 {
        let mut result = Mat4::new_zero();
        for i in 0..16 {
            result.0[i] = self.0[i] + rhs.0[i];
        }
        result
    }
}

impl std::ops::Add<f32> for Mat4 {
    type Output = Mat4;

    fn add(self, rhs: f32) -> Mat4 {
        let mut result = Mat4::new_zero();
        for i in 0..16 {
            result.0[i] = self.0[i] + rhs;
        }
        result
    }
}

impl std::ops::Sub<Mat4> for Mat4 {
    type Output = Mat4;

    fn sub(self, rhs: Mat4) -> Mat4 {
        let mut result = Mat4::new_zero();
        for i in 0..16 {
            result.0[i] = self.0[i] - rhs.0[i];
        }
        result
    }
}

impl std::ops::Sub<f32> for Mat4 {
    type Output = Mat4;

    fn sub(self, rhs: f32) -> Mat4 {
        let mut result = Mat4::new_zero();
        for i in 0..16 {
            result.0[i] = self.0[i] - rhs;
        }
        result
    }
}

impl std::ops::Mul<Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        let mut result = Mat4::new_zero();
        for i in 0..4 {
            for j in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += self.0[i + (k * 4)] * rhs.0[k + (j * 4)];
                }
                result.0[i + (j * 4)] = sum;
            }
        }
        result
    }
}

impl std::ops::Mul<Vector4D> for Mat4 {
    type Output = Vector4D;

    fn mul(self, rhs: Vector4D) -> Self::Output {
        let mut result = Vector4D::zero();
        for i in 0..4 {
            let mut sum = 0.0;
            for j in 0..4 {
                sum += self.0[j + (i * 4)] * rhs[j];
            }
            result[i] = sum;
        }
        result
    }
}

impl std::ops::Mul<f32> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: f32) -> Mat4 {
        let mut result = Mat4::new_zero();
        for i in 0..16 {
            result.0[i] = self.0[i] * rhs;
        }
        result
    }
}

impl From<Mat4> for [f32; 16] {
    fn from(lhs: Mat4) -> [f32; 16] {
        lhs.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiply_by_unit() {
        let identity = Mat4::new_unit();
        let mat = Mat4::new([
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        ]);

        let expected = mat;
        let actual = mat * identity;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_multiply_vector_by_unit() {
        let identity = Mat4::new_unit();
        let vec = Vector4D::new(1.0, 2.0, 3.0, 4.0);

        let expected = vec;
        let actual = identity * vec;
        assert_eq!(actual, expected);
    }
}
