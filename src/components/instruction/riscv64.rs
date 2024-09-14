include!("riscv64/shutdown.rs");

/// Riscv64 ebreak instruction.
#[inline]
pub fn ebreak() {
    unsafe {
        riscv::asm::ebreak();
    }
}

#[inline]
pub fn hlt() {
    unsafe {
        riscv::register::sstatus::clear_sie();
        riscv::asm::wfi();
        riscv::register::sstatus::set_sie();
    }
}
