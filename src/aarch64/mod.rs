mod barrier;
mod boot;
mod consts;
mod context;
mod gic;

#[cfg(feature = "kcontext")]
mod kcontext;
mod page_table;
mod pl011;
mod psci;
mod timer;
mod trap;

use core::slice;

use aarch64_cpu::registers::{Readable, MPIDR_EL1};
use alloc::vec::Vec;
pub use consts::*;
pub use context::TrapFrame;
use fdt::Fdt;

#[cfg(feature = "kcontext")]
pub use kcontext::{context_switch, context_switch_pt, read_current_tp, KContext};

pub use page_table::*;
use polyhal_macro::def_percpu;
pub use psci::system_off as shutdown;
pub use trap::run_user_task;

use crate::{multicore::MultiCore, utils::LazyInit, DTB_BIN, MEM_AREA};

static DTB_PTR: LazyInit<usize> = LazyInit::new();

#[def_percpu]
static CPU_ID: usize = 0;

#[inline]
pub fn hart_id() -> usize {
    MPIDR_EL1.read(MPIDR_EL1::Aff0) as _
}

pub(crate) fn arch_init() {
    let mut buffer = Vec::new();
    if let Ok(fdt) = unsafe { Fdt::from_ptr(*DTB_PTR as *const u8) } {
        unsafe {
            buffer.extend_from_slice(slice::from_raw_parts(
                *DTB_PTR as *const u8,
                fdt.total_size(),
            ));
        }
    }
    DTB_BIN.init_by(buffer);
    if let Ok(fdt) = Fdt::new(&DTB_BIN) {
        info!("There has {} CPU(s)", fdt.cpus().count());
        let mut mem_area = Vec::new();
        fdt.memory().regions().for_each(|x| {
            info!(
                "memory region {:#X} - {:#X}",
                x.starting_address as usize,
                x.starting_address as usize + x.size.unwrap()
            );
            mem_area.push((
                x.starting_address as usize | VIRT_ADDR_START,
                x.size.unwrap_or(0),
            ));
        });
        MEM_AREA.init_by(mem_area);
    }
}

#[cfg(feature = "multicore")]
impl MultiCore {
    /// Boot application cores
    pub fn boot_all() {}
}
