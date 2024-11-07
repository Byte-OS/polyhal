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
pub mod macros;
pub mod mem;
pub mod multicore;
pub mod pagetable;
pub mod percpu;
pub mod timer;
#[cfg(feature = "trap")]
pub mod trap;
pub mod trapframe;

use polyhal_macro::define_arch_mods;
