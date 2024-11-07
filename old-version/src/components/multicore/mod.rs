//! Multi-core Module.
//!
//! This is a leader for the multicore operation
//!
//! You can use this function to use the multicore operation
//!
//! Boot other calls after the multicore
//! If you use this function call, you should call it after arch::init(..);
//! This function will allocate the stack and map it for itself.
//!
//! ```rust
//! boot_core(hart_id, sp_top);
//! ```
//!
//! Here will have more functionality about multicore in the future.
//!

use crate::{pub_use_arch, utils::MutexNoIrq};

static CORE_SET: MutexNoIrq<u64> = MutexNoIrq::new(0);

pub struct CpuCore;

/// Initialize the core with boot_hart_id
impl CpuCore {
    pub(crate) fn init(boot_hart_id: usize) {
        let mut set = CORE_SET.lock();
        *set |= 1 << boot_hart_id;
    }
}

super::define_arch_mods!();
pub_use_arch!(boot_core);
