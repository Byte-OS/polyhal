use core::sync::atomic::{AtomicBool, Ordering};

/// TODO: Boot a core with top pointer of the stack
///     Fina a way to pass the stack pointer to core.
pub fn boot_core(hart_id: usize, _sp_top: usize) {
    static BOOT_LOCK: AtomicBool = AtomicBool::new(false);

    // Waiting until the previous boot is completed.
    while BOOT_LOCK.load(Ordering::SeqCst) {};

    // Set the boot lock to true and start the boot process.
    BOOT_LOCK.store(true, Ordering::SeqCst);
    log::error!("Boot Core is not implemented yet for x86_64");
    
    let apic_id = crate::arch::apic::raw_apic_id(hart_id as _);
    let lapic = crate::arch::apic::local_apic();

    // This is the 
    const START_PAGE_IDX: u8 = 6;

    unsafe {
        lapic.send_init_ipi(apic_id);
    
        lapic.send_sipi(START_PAGE_IDX, apic_id);

        lapic.send_sipi(START_PAGE_IDX, apic_id);
    }
}
