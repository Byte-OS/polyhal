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

#[cfg(feature = "fp_simd")]
#[cfg(target_arch = "loongarch64")]
pub_use_arch!(save_fp_regs, restore_fp_regs);
