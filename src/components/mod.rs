//! General components of the multi-architecture.
//!
//!

pub(crate) mod arch;
pub mod boot;
pub mod common;
pub mod consts;
pub mod debug_console;
pub mod instruction;
pub mod irq;
pub mod kcontext;
pub mod mem;
#[cfg(feature = "multicore")]
pub mod multicore;
pub mod pagetable;
pub mod percpu;
pub mod timer;
pub mod trap;
pub mod trapframe;

/// Define multi-architecture mod code and pub use code.
///
/// Only recommended for use inside [polyhal] crate.
pub(self) macro define_arch_mods() {
    #[cfg(target_arch = "riscv64")]
    mod riscv64;
    #[cfg(target_arch = "riscv64")]
    #[allow(unused_imports)]
    pub use riscv64::*;
    #[cfg(target_arch = "aarch64")]
    mod aarch64;
    #[cfg(target_arch = "aarch64")]
    #[allow(unused_imports)]
    pub use aarch64::*;
    #[cfg(target_arch = "x86_64")]
    mod x86_64;
    #[cfg(target_arch = "x86_64")]
    #[allow(unused_imports)]
    pub use x86_64::*;
    #[cfg(target_arch = "loongarch64")]
    mod loongarch64;
    #[cfg(target_arch = "loongarch64")]
    #[allow(unused_imports)]
    pub use loongarch64::*;
}
