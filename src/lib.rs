#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(stdsimd)]
#![feature(const_mut_refs)]
#![feature(const_slice_from_raw_parts_mut)]
#![cfg_attr(target_arch = "riscv64", feature(riscv_ext_intrinsics))]
#![cfg_attr(target_arch = "aarch64", feature(const_option))]

extern crate alloc;

#[macro_use]
extern crate log;
#[macro_use]
extern crate cfg_if;

mod utils;
#[macro_use]
mod common;

cfg_if! {
    if #[cfg(feature = "libos")] {
        #[path = "libos/mod.rs"]
        mod imp;
    } else {
        #[path = "bare/mod.rs"]
        mod imp;
    }
}

use common::pagetable::*;
use common::consts::*;
use common::addr::*;
use common::api::*;
use common::multicore::MultiCore;
use imp::*;
use imp::current_arch::*;

pub use imp::{get_cpu_num, get_mem_areas, get_fdt, init};