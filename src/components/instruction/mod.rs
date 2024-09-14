//! Instruction Module.
//!
//! This module contains the instruction of the different architectures.
//!

use crate::pub_use_arch;

super::define_arch_mods!();

pub_use_arch!(shutdown, ebreak);
