include!("x86_64/shutdown.rs");

/// Riscv64 ebreak instruction.
pub fn ebreak() {
    unsafe {
        core::arch::asm!("int 3");
    }
}
