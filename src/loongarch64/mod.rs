mod barrier;
mod boot;
mod console;
mod consts;
mod context;
#[cfg(feature = "kcontext")]
mod kcontext;
mod page_table;
mod sigtrx;
mod timer;
mod trap;
mod unaligned;

use crate::{multicore::MultiCore, DTB_BIN, MEM_AREA};
use alloc::vec::Vec;
pub use consts::*;
pub use context::TrapFrame;
#[cfg(feature = "kcontext")]
pub use kcontext::{context_switch, context_switch_pt, read_current_tp, KContext};
pub use page_table::boot_page_table;
pub use trap::{disable_irq, enable_external_irq, enable_irq, run_user_task};

pub fn shutdown() -> ! {
    loop {
        unsafe { loongArch64::asm::idle() };
    }
}

pub(crate) fn arch_init() {
    DTB_BIN.init_by(Vec::new());
    MEM_AREA.init_by({
        let mut mem_area = Vec::new();
        // This is just temporary solution until we find a better way to detect memory areas.
        mem_area.push((VIRT_ADDR_START | 0x9000_0000, 0x2000_0000));
        mem_area
    });
}

pub fn hart_id() -> usize {
    loongArch64::register::cpuid::read().core_id()
}

#[cfg(feature = "multicore")]
impl MultiCore {
    pub fn boot_all() {}
}
