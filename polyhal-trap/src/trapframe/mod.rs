//! Trapframe module.
//!
//!

use core::mem::size_of;

polyhal_macro::define_arch_mods!();

/// Trap Frame Arg Type
///
/// Using this by Index and IndexMut trait bound on TrapFrame
#[derive(Debug)]
pub enum TrapFrameArgs {
    SEPC,
    RA,
    SP,
    RET,
    ARG0,
    ARG1,
    ARG2,
    TLS,
    SYSCALL,
}

/// The size of the [TrapFrame]
pub const TRAPFRAME_SIZE: usize = size_of::<TrapFrame>();
