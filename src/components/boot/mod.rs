//! Boot Components.
//!
//!

// Define multi-architecture modules and pub use them.
super::define_arch_mods!();

/// Boot Stack Size.
/// TODO: reduce the boot stack size. Map stack in boot step.
pub const STACK_SIZE: usize = 0x8_0000;

/// Boot Stack. Boot Stack Size is [STACK_SIZE]
#[link_section = ".bss.stack"]
pub(crate) static mut BOOT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

// Declare the _main_for_arch exists.
extern "Rust" {
    #[cfg(feature = "boot")]
    pub(crate) fn _main_for_arch(hartid: usize);
}
