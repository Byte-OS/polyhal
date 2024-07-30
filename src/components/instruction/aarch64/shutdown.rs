use crate::components::{arch::psci, instruction::Instruction};

impl Instruction {
    /// Close the computer. Call PSCI.
    #[inline]
    pub fn shutdown() -> ! {
        psci::system_off()
    }
}
