use crate::irq::IRQ;

/// Implement IRQ operations for the IRQ interface.
impl IRQ {
    /// Enable irq for the given IRQ number.
    #[inline]
    pub fn enable(_irq_num: usize) {
        log::warn!("irq not implemented in riscv platform yet");
    }

    /// Disable irq for the given IRQ number.
    #[inline]
    pub fn disable(_irq_num: usize) {
        log::warn!("irq not implemented in riscv platform yet");
    }
}
