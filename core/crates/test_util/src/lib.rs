use std::fmt::Debug;

/// # Panics
///
/// Will panic when the slices are different.
pub fn assert_slice_eq<T: PartialEq + Debug>(expected: &[T], actual: &[T]) {
    assert!(
        expected.len() == actual.len(),
        "\nVectors do not match. Expected length {} but got {}",
        expected.len(),
        actual.len(),
    );
    for (i, expected) in expected.iter().enumerate() {
        let actual = &actual[i];
        assert!(
            *expected == *actual,
            "\nVectors do not match. Unexpected item at index {}.\nExpected: {:?}\nActual:   {:?}",
            i,
            expected,
            actual,
        );
    }
}
