use crate::consts::VIRT_ADDR_START;

// Boot a core with top pointer of the stack
pub fn boot_core(cpuid: usize, addr: usize, sp_top: usize) {
    // PERCPU DATA ADDRESS RANGE END
    let aux_core_func = addr & !VIRT_ADDR_START;

    log::info!("secondary addr: {:#x}", addr);
    let ret = sbi_rt::hart_start(cpuid, aux_core_func, sp_top);
    match ret.is_ok() {
        true => log::info!("hart {} Startting successfully", cpuid),
        false => log::warn!("hart {} Startting failed", cpuid),
    }
}
