use crate::{boot::secondary_start, common::CPU_ID, consts::VIRT_ADDR_START};

// Boot a core with top pointer of the stack
pub fn boot_core(cpu: usize, sp_top: usize) {
    if cpu == CPU_ID.read_current() {
        return;
    };

    // PERCPU DATA ADDRESS RANGE END
    let aux_core_func = (secondary_start as usize) & (!VIRT_ADDR_START);

    log::info!("secondary addr: {:#x}", secondary_start as usize);
    let ret = sbi_rt::hart_start(cpu, aux_core_func, sp_top);
    match ret.is_ok() {
        true => log::info!("hart {} Startting successfully", cpu),
        false => log::warn!("hart {} Startting failed", cpu),
    }
}
