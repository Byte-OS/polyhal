mod apic;
mod barrier;
mod boot;
mod consts;
mod context;
mod gdt;
mod idt;
mod irq;
#[cfg(feature = "kcontext")]
mod kcontext;
mod page_table;
mod sigtrx;
mod time;
mod trap;
mod uart;

use ::multiboot::information::MemoryType;
use alloc::vec::Vec;
pub use boot::boot_page_table;
pub use consts::VIRT_ADDR_START;
pub use context::TrapFrame;
#[cfg(feature = "kcontext")]
pub use kcontext::{context_switch, context_switch_pt, read_current_tp, KContext};
pub use trap::*;
pub use uart::*;

use x86_64::instructions::port::PortWriteOnly;

use crate::{
    currrent_arch::boot::use_multiboot, multicore::MultiCore, utils::LazyInit, DTB_BIN, MEM_AREA,
};

#[polyhal_macro::def_percpu]
static CPU_ID: usize = 1;

pub fn shutdown() -> ! {
    unsafe { PortWriteOnly::new(0x604).write(0x2000u16) };
    loop {}
}

static MBOOT_PTR: LazyInit<usize> = LazyInit::new();

pub fn arch_init() {
    DTB_BIN.init_by(Vec::new());
    if let Some(mboot) = use_multiboot(*MBOOT_PTR as _) {
        let mut mem_area = Vec::new();
        if mboot.has_memory_map() {
            mboot
                .memory_regions()
                .unwrap()
                .filter(|x| x.memory_type() == MemoryType::Available)
                .for_each(|x| {
                    let start = x.base_address() as usize | VIRT_ADDR_START;
                    let size = x.length() as usize;
                    // ArchInterface::add_memory_region(start, end);
                    mem_area.push((start, size));
                });
        }
        MEM_AREA.init_by(mem_area);
    }
}

pub fn hart_id() -> usize {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.initial_local_apic_id() as usize,
        None => 0,
    }
}

#[cfg(feature = "multicore")]
impl MultiCore {
    pub fn boot_all() {}
}

/// Reserved for default usage.
/// This is related to the [polyhal_macro::percpu::PERCPU_RESERVED]
/// Just for x86_64 now.
/// 0: SELF_PTR
/// 1: VALID_PTR
/// 2: USER_RSP
/// 3: KERNEL_RSP
/// 4: USER_CONTEXT
#[repr(C)]
pub(crate) struct PerCPUReserved {
    pub self_ptr: usize,
    pub valid_ptr: usize,
    pub user_rsp: usize,
    pub kernel_rsp: usize,
    pub user_context: usize,
}

impl PerCPUReserved {
    pub fn mut_from_ptr(ptr: *mut Self) -> &'static mut Self {
        unsafe { &mut (*ptr) }
    }
}
