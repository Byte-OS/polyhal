mod shutdown;
use crate::components::instruction::Instruction;

impl Instruction {
    #[inline]
    pub fn hlt() {
        unsafe {
            riscv::register::sstatus::clear_sie();
            riscv::asm::wfi();
            riscv::register::sstatus::set_sie();
        }
    }
}
