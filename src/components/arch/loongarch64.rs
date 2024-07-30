use alloc::vec::Vec;

use crate::components::{common::{DTB_BIN, MEM_AREA}, consts::VIRT_ADDR_START};


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
