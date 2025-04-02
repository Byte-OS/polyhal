use core::hint::spin_loop;

use x86_64::instructions::port::PortWriteOnly;

#[inline]
pub fn shutdown() -> ! {
    unsafe { PortWriteOnly::new(0x604).write(0x2000u16) };
    loop {
        spin_loop();
    }
}
