// A variation on assert_eq from the standard library
#[macro_export]
macro_rules! assert_eq_eps {
    ($left:expr, $right:expr, $epsilon:expr $(,)?) => ({
        match (&$left, &$right, &$epsilon) {
            (left_val, right_val, epsilon) => {
                if !((*left_val - *right_val).abs() < *epsilon) {
                    panic!(r#"assertion failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`"#, &*left_val, &*right_val)
                }
            }
        }
    });
    ($left:expr, $right:expr, $epsilon:expr, $($arg:tt)+) => ({
        match (&($left), &($right), &($epsilon)) {
            (left_val, right_val, epsilon) => {
                if !((*left_val - *right_val).abs() < *epsilon) {
                    panic!(r#"assertion failed: `(left == right)`
  left: `{:?}`,
 right: `{:?}`: {}"#, &*left_val, &*right_val,
                           format_args!($($arg)+))
                }
            }
        }
    });
}
