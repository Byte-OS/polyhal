use alloc::vec;
use alloc::vec::Vec;

use crate::components::{common::{DTB_BIN, MEM_AREA}, consts::VIRT_ADDR_START};


pub(crate) fn arch_init() {
    DTB_BIN.init_by(Vec::new());
    MEM_AREA.init_by(vec![
        (VIRT_ADDR_START | 0x9000_0000, 0x2000_0000)
    ]);
}

#[inline]
pub fn hart_id() -> usize {
    loongArch64::register::cpuid::read().core_id()
}
