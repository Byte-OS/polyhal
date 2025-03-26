#![no_std]
#![feature(asm_const)]
#![feature(naked_functions)]

//! Boot Components.
//!
//!

use core::mem::size_of;

// Define multi-architecture modules and pub use them.
cfg_if::cfg_if! {
    if #[cfg(target_arch = "loongarch64")] {
        mod loongarch64;
    } else if #[cfg(target_arch = "aarch64")] {
        mod aarch64;
    } else if #[cfg(target_arch = "riscv64")] {
        mod riscv64;
    } else if #[cfg(target_arch = "x86_64")] {
        mod x86_64;
    } else {
        compile_error!("unsupported architecture!");
    }
}

/// Boot Stack Size.
/// TODO: reduce the boot stack size. Map stack in boot step.
pub const STACK_SIZE: usize = 0x8_0000;

/// Boot Stack. Boot Stack Size is [STACK_SIZE]
#[link_section = ".bss.stack"]
pub(crate) static mut BOOT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

#[cfg(any(target_arch = "riscv64", target_arch = "aarch64"))]
#[repr(align(4096))]
pub(crate) struct PageAlignment([polyhal::pagetable::PTE; polyhal::PageTable::PTE_NUM_IN_PAGE]);

/// Clear the bss section
pub(crate) fn clear_bss() {
    extern "C" {
        fn _sbss();
        fn _ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(
            _sbss as usize as *mut u128,
            (_ebss as usize - _sbss as usize) / size_of::<u128>(),
        )
        .fill(0);
    }
}

// Declare the _main_for_arch exists.
extern "Rust" {
    pub(crate) fn _main_for_arch(hartid: usize);
}

/// Define the entry point.
///
/// TODO: Support secondary Entry Point for the application core.
///     - Implement MultiCore
///     - Jump to _secondary_for_arch function in application core.
/// Application Core always have the differnt logic than Boot Core.
#[macro_export]
macro_rules! define_entry {
    ($main_fn:ident, $sec_entry:ident) => {
        #[export_name = "_main_for_arch"]
        fn _polyhal_defined_main(hart_id: usize) {
            $main_fn(hart_id);
        }
        #[export_name = "_secondary_for_arch"]
        fn _polyhal_defined_secondary(hart_id: usize) {
            $sec_entry(hart_id);
        }
    };
    ($main_fn:ident) => {
        define_entry!($main_fn, $main_fn);
    };
}
