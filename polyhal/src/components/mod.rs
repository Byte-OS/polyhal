//! General components of the multi-architecture.
//!
//!

pub mod common;
pub mod instruction;
pub mod irq;
pub mod kcontext;
pub mod mem;
pub mod multicore;
pub mod percpu;
pub mod timer;

use polyhal_macro::define_arch_mods;
