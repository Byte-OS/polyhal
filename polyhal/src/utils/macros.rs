/// bit macro will generate the number through a shift value.
///
/// Here is an example.
/// You can use bit!(0) instead of 1 << 0.
/// bit!(39) instead of 1 << 39.

#[macro_export]
macro_rules! bit {
    ($x: expr) => {
        (1 << $x)
    };
}

#[macro_export]
macro_rules! bits {
    ($($x: expr),*) => {
        $(1 << $x)|*
    };
}

#[macro_export]
macro_rules! pub_use_arch {
    ($($name:ident),*) => {
        $(
            #[cfg(target_arch = "loongarch64")]
            pub use self::loongarch64::$name;
            #[cfg(target_arch = "x86_64")]
            pub use self::x86_64::$name;
            #[cfg(target_arch = "riscv64")]
            pub use self::riscv64::$name;
            #[cfg(target_arch = "aarch64")]
            pub use self::aarch64::$name;
        )*
    };
}
