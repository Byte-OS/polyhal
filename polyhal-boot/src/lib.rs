#![no_std]
#![feature(naked_functions)]

//! Boot Components.
//!
//!

#[allow(unused_imports)]
#[macro_use]
extern crate polyhal;

mod arch;

/// Define the entry point.
///
/// TODO: Support secondary Entry Point for the application core.
///     - Implement MultiCore
///     - Jump to _secondary_for_arch function in application core.
/// Application Core always have the differnt logic than Boot Core.
#[macro_export]
macro_rules! define_entry {
    ($main_fn:ident, $sec_entry:ident) => {
        core::arch::global_asm!(
            "
                .section .bss.bstack
                .global bstack
                .global bstack_top
                bstack:
                .fill 0x80000
                .size bstack, . - bstack
                bstack_top:
            "
        );
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
