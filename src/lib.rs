#![cfg_attr(not(feature = "libos"), no_std)]
#![feature(decl_macro)]
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
#[macro_use]
mod utils;
#[macro_use]
mod common;
pub use common::{page::PageAlloc, *};

cfg_if! {
    if #[cfg(feature = "libos")] {
        #[path = "libos/mod.rs"]
        mod imp;

        pub use polyhal_macro::{arch_entry, arch_interrupt};
        pub use imp::context::*;
        pub use imp::addr::*;
        pub use imp::init;
        pub use imp::mem::get_mem_areas;
        pub use imp::vm::{Page, PageTable};
        pub use imp::addr::MMUFlags;
        pub use imp::mem::{pmem_read, pmem_copy, pmem_write, pmem_zero};
        pub use imp::shutdown;
    } else {
        #[path = "bare/mod.rs"]
        mod imp;
        use imp::api::*;
        use imp::pagetable::*;
        use imp::consts::*;
        use imp::multicore::MultiCore;
        use imp::current_arch::*;
        use irq::IRQVector;

        pub use imp::{
            *,
            get_mem_areas, init,
            TrapFrameArgs, TrapType, PAGE_SIZE,
            time::*,
            current_arch::{
                run_user_task, shutdown, kernel_page_table,
                TrapFrame, VIRT_ADDR_START,
            },
        };
        pub use polyhal_macro::{arch_entry, arch_interrupt};

        #[cfg(feature = "kcontext")]
        pub use imp::{KContextArgs, current_arch::{KContext, read_current_tp, context_switch_pt, context_switch}};
    }
}
