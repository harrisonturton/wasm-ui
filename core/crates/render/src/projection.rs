use math::{Mat4, Vector2D, Vector4D};

pub fn ortho(right: f32, left: f32, top: f32, bottom: f32, near: f32, far: f32) -> Mat4 {
    let mut result = Mat4::new_unit();
    result.0[0] = 2.0 / (right - left);
    result.0[3] = -(right + left) / (right - left);
    result.0[5] = 2.0 / (top - bottom);
    result.0[7] = -(top + bottom) / (top - bottom);
    result.0[10] = -2.0 / (far - near);
    result.0[11] = -(far + near) / (far - near);
    result
}

/*perspective: function(fieldOfViewInRadians, aspect, near, far) {
    var f = Math.tan(Math.PI * 0.5 - 0.5 * fieldOfViewInRadians);
    var rangeInv = 1.0 / (near - far);
 
    return [
      f / aspect, 0, 0, 0,
      0, f, 0, 0,
      0, 0, (near + far) * rangeInv, -1,
      0, 0, near * far * rangeInv * 2, 0
    ];
  },
  */

pub fn perspective2(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fov.to_radians() * 0.5).tan();
    let a = f / aspect;
    let b = (far + near) / (near - far);
    let c = (2.0 * far * near) / (near - far);
    Mat4::new([
        a, 0.0, 0.0, 0.0,
        0.0, f, 0.0, 0.0,
        0.0, 0.0, b, c,
        0.0, 0.0, -1.0, 0.0
    ])
}

pub fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = (std::f32::consts::PI * 0.5 - 0.5 * fov.to_radians()).tan();
    let range_inv = 1.0 / (near - far);
    Mat4::new([
        f / aspect, 0.0, 0.0, 0.0,
        0.0, f, 0.0, 0.0,
        0.0, 0.0, (near + far) * range_inv, -1.0,
        0.0, 0.0, near * far * range_inv * 2.0, 0.0,
    ])
}

pub fn screen(dimensions: Vector2D) -> Vector4D {
    Vector4D::new(dimensions.x, dimensions.y, -1.0, 1.0)
}
