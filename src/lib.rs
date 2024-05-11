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
use common::api::*;
use common::multicore::MultiCore;
pub use imp::*;
pub use imp::current_arch::*;
pub use imp::debug::*;
pub use imp::time::*;

pub use common::{
    addr::*, 
    page::PageAlloc, 
    pagetable::{PageTable, PageTableWrapper, MappingFlags, MappingSize}
};
// pub use imp::{
//     get_mem_areas, init,
//     TrapFrameArgs, TrapType, PAGE_SIZE,
//     time::*,
//     debug::DebugConsole,
//     current_arch::{
//         run_user_task, shutdown, kernel_page_table,
//         TrapFrame, VIRT_ADDR_START,
//     },
// };
pub use polyhal_macro::{arch_entry, arch_interrupt};

#[cfg(feature = "kcontext")]
pub use imp::{KContextArgs, current_arch::{KContext, read_current_tp, context_switch_pt, context_switch}};