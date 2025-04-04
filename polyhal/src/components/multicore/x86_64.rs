use core::time::Duration;

use crate::{
    apic::{local_apic, raw_apic_id},
    consts::VIRT_ADDR_START,
    pagetable::PAGE_SIZE,
    timer::timer_wait,
};

const AP_BOOT_PAGE: usize = 0x6000;

pub fn boot_core(cpu_id: usize, addr: usize, sp_top: usize) {
    extern "C" {
        fn ap_start();
        fn ap_end();
    }

    unsafe {
        let dst = pa!(AP_BOOT_PAGE).get_mut_ptr::<u8>();
        dst.copy_from_nonoverlapping(ap_start as *const u8, ap_end as usize - ap_start as usize);

        pa!(AP_BOOT_PAGE + 0xff8)
            .get_mut_ptr::<usize>()
            .write_volatile(addr & !VIRT_ADDR_START);
        pa!(AP_BOOT_PAGE + 0xff0)
            .get_mut_ptr::<usize>()
            .write_volatile(sp_top & !VIRT_ADDR_START);
    }

    let apic_id = raw_apic_id(cpu_id as _);
    let lapic = local_apic();
    unsafe {
        lapic.send_init_ipi(apic_id);
        timer_wait(Duration::from_millis(10)); // 10ms
        lapic.send_sipi((AP_BOOT_PAGE / PAGE_SIZE) as _, apic_id);
        timer_wait(Duration::from_micros(200)); // 200us
        lapic.send_sipi((AP_BOOT_PAGE / PAGE_SIZE) as _, apic_id);
    }
}
