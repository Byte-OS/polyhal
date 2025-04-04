use core::hint::spin_loop;

use x86_64::instructions::port::PortWriteOnly;

use crate::acpi::get_pm1a_addr;

#[inline]
pub fn shutdown() -> ! {
    println!("pm1a addr: {:#x?}", get_pm1a_addr());
    if let Some(port) = get_pm1a_addr() {
        println!("pm1a addr: {:#x?}", port);
        unsafe { PortWriteOnly::new(port as _).write(0x2000u16) };
    }
    // unsafe { PortWriteOnly::new(0x604).write(0x2000u16) };
    loop {
        spin_loop();
    }
}
