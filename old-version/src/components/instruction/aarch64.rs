include!("aarch64/shutdown.rs");

#[inline]
pub fn ebreak() {
    unsafe {
        core::arch::asm!("brk 0");
    }
}
