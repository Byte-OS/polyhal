/// bit macro will generate the number through a shift value.
///
/// Here is an example.
/// You can use bit!(0) instead of 1 << 0.
/// bit!(39) instead of 1 << 39.
#[macro_export]
macro_rules! bit {
    ($x:expr) => {
        1 << $x
    };
}