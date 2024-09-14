#[inline]
pub fn ebreak() {
    unsafe {
        core::arch::asm!("break 2");
    }
}

#[inline]
pub fn shutdown() -> ! {
    log::warn!("Shutting down on loongarch64 platform was not implemented!");
    loop {
        unsafe { loongArch64::asm::idle() };
    }
}
