// Used for float comparison.
const EPSILON: f32 = 0.00001;

pub fn float_eq(lhs: f32, rhs: f32) -> bool {
    (lhs - rhs).abs() < EPSILON
}
