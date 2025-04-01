#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(cfg_version)]
#![feature(decl_macro)]
#![feature(used_with_arg)]
#![feature(unsafe_attributes)]
#![cfg_attr(not(version("1.79")), feature(stdsimd))]
#![feature(const_mut_refs)]
#![feature(const_slice_from_raw_parts_mut)]
#![feature(const_trait_impl)]
#![cfg_attr(target_arch = "riscv64", feature(riscv_ext_intrinsics))]
#![cfg_attr(target_arch = "aarch64", feature(const_option))]

extern crate alloc;
extern crate log;

#[macro_use]
pub mod ctor;
#[macro_use]
pub mod debug_console;

pub mod arch;
mod components;
pub mod mem;
pub use components::*;
pub mod pagetable;
pub mod time;
pub mod utils;

pub use utils::addr::{PhysAddr, VirtAddr};

#[cfg(feature = "boot")]
pub use polyhal_macro::arch_entry;
#[cfg(feature = "trap")]
pub use polyhal_macro::arch_interrupt;

// Re export the Module like Structure.
pub use pagetable::{MappingFlags, MappingSize, PageTable, PageTableWrapper};
pub use time::Time;
