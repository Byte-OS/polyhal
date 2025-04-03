use crate::apic::{local_apic, raw_apic_id};

// TODO: Boot a core with top pointer of the stack
pub fn boot_core(cpu_id: usize, _addr: usize, _sp_top: usize) {
    let apic_id = raw_apic_id(cpu_id as _);
    let lapic = local_apic();
    unsafe {
        lapic.send_init_ipi(apic_id);
    }

    log::error!("Boot Core is not implemented yet for aarch64");
}
