use crate::{arch::consts::VIRT_ADDR_START, va};

#[inline]
pub fn ebreak() {
    unsafe {
        core::arch::asm!("break 2");
    }
}

#[inline]
pub fn shutdown() -> ! {
    let ged_addr = va!(0x100E001C | VIRT_ADDR_START);
    log::info!("Shutting down...");
    unsafe { ged_addr.get_mut_ptr::<u8>().write_volatile(0x34) };
    unsafe { loongArch64::asm::idle() };
    log::warn!("It should shutdown!");
    unreachable!()
}
