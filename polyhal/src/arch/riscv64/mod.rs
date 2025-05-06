pub mod consts;

#[polyhal_macro::percpu]
pub(crate) static CPU_ID: usize = 0;

#[inline]
pub fn wfi() {
    unsafe {
        riscv::register::sstatus::clear_sie();
        riscv::asm::wfi();
        riscv::register::sstatus::set_sie();
    }
}

pub fn hart_id() -> usize {
    *CPU_ID
}
