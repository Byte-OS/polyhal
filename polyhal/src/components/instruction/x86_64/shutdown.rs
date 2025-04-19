use core::hint::spin_loop;

use x86_64::instructions::port::PortWriteOnly;

use crate::acpi::get_pm1a_addr;

#[inline]
pub fn shutdown() -> ! {
    if let Some(port) = get_pm1a_addr() {
        unsafe { PortWriteOnly::new(port as _).write(0x2000u16) };
    }
    loop {
        spin_loop();
    }
}
