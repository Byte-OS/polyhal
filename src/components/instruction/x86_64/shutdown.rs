use x86_64::instructions::port::PortWriteOnly;

use crate::components::instruction::Instruction;

impl Instruction {
    #[inline]
    pub fn shutdown() -> ! {
        unsafe { PortWriteOnly::new(0x604).write(0x2000u16) };
        loop {}
    }
}