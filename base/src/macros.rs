#[cfg(target_arch = "loongarch64")]
#[macro_export]
macro_rules! pub_use_arch {
    ($($name:ident),*) => {
        $(
            pub use self::loongarch64::$name;
        )*
    };
}

#[cfg(target_arch = "x86_64")]
#[macro_export]
macro_rules! pub_use_arch {
    ($($name:ident),*) => {
        $(
            pub use self::x86_64::$name;
        )*
    };
}

#[cfg(target_arch = "riscv64")]
#[macro_export]
macro_rules! pub_use_arch {
    ($($name:ident),*) => {
        $(
            pub use self::riscv64::$name;
        )*
    };
}

#[cfg(target_arch = "aarch64")]
#[macro_export]
macro_rules! pub_use_arch {
    ($($name:ident),*) => {
        $(
            pub use self::aarch64::$name;
        )*
    };
}
