#![no_std]
#![feature(asm_const)]
#![feature(naked_functions)]

//! Boot Components.
//!
//!

extern crate polyhal;

use core::{hint::spin_loop, mem::size_of};

use polyhal::ctor::{ph_init_iter, CtorType};

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

fn call_real_main(hartid: usize) {
    // Run Kernel's Contructors Before Droping Into Kernel.
    ph_init_iter(CtorType::KernelService).for_each(|x| (x.func)());
    ph_init_iter(CtorType::Normal).for_each(|x| (x.func)());

    // Declare the _main_for_arch exists.
    extern "Rust" {
        pub(crate) fn _main_for_arch(hartid: usize);
    }
    unsafe {
        _main_for_arch(hartid);
    }
    loop {
        spin_loop();
    }
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
        core::arch::global_asm!(concat!(
            "
            .section .bss
            .global bstack_top
            bstack:
            .fill 0x80000
            bstack_top:
        "
        ));
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
