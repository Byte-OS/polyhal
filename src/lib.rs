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
use imp::*;
use imp::current_arch::*;

pub use common::addr::*;
pub use common::page::PageAlloc;
pub use imp::{get_cpu_num, get_mem_areas, get_fdt, init};
pub use imp::time::*;
pub use imp::debug::DebugConsole;
pub use imp::current_arch::{run_user_task, run_user_task_forever, disable_irq, enable_irq, enable_external_irq, switch_to_kernel_page_table, kernel_page_table};
pub use imp::current_arch::TrapFrame;
pub use imp::current_arch::VIRT_ADDR_START;
pub use imp::{TrapFrameArgs, TrapType, PAGE_ALLOC, PAGE_SIZE};
#[cfg(feature = "kcontext")]
pub use imp::{KContextArgs, current_arch::{KContext, read_current_tp, context_switch_pt, context_switch}};
pub use polyhal_macro::{arch_entry, arch_interrupt};
pub use imp::current_arch::shutdown;
pub use common::pagetable::{PageTable, PageTableWrapper, MappingFlags, MappingSize};