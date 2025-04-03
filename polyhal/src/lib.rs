#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(cfg_version)]
#![feature(decl_macro)]
#![feature(used_with_arg)]
#![feature(unsafe_attributes)]
#![cfg_attr(not(version("1.79")), feature(stdsimd))]
#![cfg_attr(target_arch = "riscv64", feature(riscv_ext_intrinsics))]
#![cfg_attr(target_arch = "aarch64", feature(const_option))]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]

// extern crate alloc;
extern crate log;

#[macro_use]
pub mod ctor;
#[macro_use]
pub mod debug_console;
#[macro_use]
pub mod utils;

mod arch;
pub use arch::*;
mod components;
pub mod mem;
pub use components::*;
pub mod pagetable;
pub mod time;

pub use utils::addr::{PhysAddr, VirtAddr};

#[cfg(feature = "boot")]
pub use polyhal_macro::arch_entry;
#[cfg(feature = "trap")]
pub use polyhal_macro::arch_interrupt;

// Re export the Module like Structure.
pub use pagetable::{MappingFlags, MappingSize, PageTable, PageTableWrapper};
pub use time::Time;
