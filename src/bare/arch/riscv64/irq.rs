use riscv::register::sstatus::{self, clear_sie, set_sie};
use crate::irq::IRQ;

/// Implement IRQ operations for the IRQ interface.
impl IRQ {
    /// Enable irq for the given IRQ number.
    #[inline]
    pub fn irq_enable(_irq_num: usize) {
        log::warn!("irq not implemented in riscv platform yet");
    }

    /// Disable irq for the given IRQ number.
    #[inline]
    pub fn irq_disable(_irq_num: usize) {
        log::warn!("irq not implemented in riscv platform yet");
    }

    /// Enable interrupts.
    #[inline]
    pub fn int_enable() {
        unsafe { set_sie() }
    }

    /// Disable interrupts.
    #[inline]
    pub fn int_disable() {
        unsafe { clear_sie() }
    }

    /// Check if the interrupts was enabled.
    #[inline]
    pub fn int_enabled() -> bool {
        sstatus::read().sie()
    }
}