//! Kernel Context module.
//!
//!

use crate::pub_use_arch;

super::define_arch_mods!();

/// Kernel Context Arg Type.
///
/// Using this by Index and IndexMut trait bound on KContext.
#[derive(Debug)]
pub enum KContextArgs {
    /// Kernel Stack Pointer
    KSP,
    /// Kernel Thread Pointer
    KTP,
    /// Kernel Program Counter
    KPC,
}

pub_use_arch!(context_switch, context_switch_pt);
