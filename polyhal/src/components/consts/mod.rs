//! Consts components
//!
//!

use crate::pub_use_arch;

super::define_arch_mods!();

pub_use_arch!(VIRT_ADDR_START);

pub const MEM_VECTOR_CAPACITY: usize = 0x20;
