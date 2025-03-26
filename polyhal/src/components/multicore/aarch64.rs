use crate::{arch::psci, consts::VIRT_ADDR_START};

/// Boot a core using hart_id, its stack pointer is sp_top
pub fn boot_core(hart_id: usize, sp_top: usize) {
    psci::cpu_on(
        hart_id,
        crate::components::boot::_secondary_boot as usize & !VIRT_ADDR_START,
        sp_top & !VIRT_ADDR_START,
    );
}
