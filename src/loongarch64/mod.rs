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

use crate::{clear_bss, multicore::MultiCore, CPU_NUM, DTB_BIN, MEM_AREA};
use alloc::vec::Vec;
pub use consts::*;
pub use context::TrapFrame;
#[cfg(feature = "kcontext")]
pub use kcontext::{context_switch, context_switch_pt, read_current_tp, KContext};
use loongArch64::register::euen;
pub use page_table::kernel_page_table;
pub use trap::{disable_irq, enable_external_irq, enable_irq, run_user_task};

pub fn rust_tmp_main(hart_id: usize) {
    clear_bss();
    console::init();
    trap::set_trap_vector_base();
    sigtrx::init();
    // Enable floating point
    euen::set_fpe(true);
    timer::init_timer();
    trap::tlb_init(trap::tlb_fill as _);

    CPU_NUM.init_by(2);

    unsafe { crate::api::_main_for_arch(hart_id) };

    shutdown();
}

pub fn shutdown() -> ! {
    error!("shutdown!");
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

pub(crate) extern "C" fn _rust_secondary_main(_hartid: usize) {}

#[cfg(feature = "multicore")]
impl MultiCore {
    pub fn boot_all() {}
}
