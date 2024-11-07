use core::slice;

use alloc::vec::Vec;
use fdt::Fdt;

use crate::components::{
    common::{CPU_ID, DTB_BIN, DTB_PTR, MEM_AREA},
    consts::VIRT_ADDR_START,
};

#[inline]
pub fn wfi() {
    unsafe {
        riscv::register::sstatus::clear_sie();
        riscv::asm::wfi();
        riscv::register::sstatus::set_sie();
    }
}

pub fn hart_id() -> usize {
    CPU_ID.read_current()
}

pub fn arch_init() {
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
    let mut mem_area = Vec::new();
    if let Ok(fdt) = Fdt::new(&DTB_BIN) {
        log::info!("There has {} CPU(s)", fdt.cpus().count());
        fdt.memory().regions().for_each(|x| {
            log::info!(
                "memory region {:#X} - {:#X}",
                x.starting_address as usize,
                x.starting_address as usize + x.size.unwrap()
            );
            mem_area.push((
                x.starting_address as usize | VIRT_ADDR_START,
                x.size.unwrap_or(0),
            ));
        });
    } else {
        mem_area.push((0x8000_0000 | VIRT_ADDR_START, 0x1000_0000));
    }
    MEM_AREA.init_by(mem_area);
}
