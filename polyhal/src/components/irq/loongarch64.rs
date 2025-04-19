use loongArch64::register::crmd;

use crate::components::irq::IRQ;

/// Timer IRQ of loongarch64
pub const TIMER_IRQ: usize = 11;

/// Implement IRQ operations for the IRQ interface.
impl IRQ {
    /// Enable irq for the given IRQ number.
    #[inline]
    pub fn irq_enable(_irq_num: usize) {
        log::warn!("irq not implemented in loongarch64 platform yet");
    }

    /// Disable irq for the given IRQ number.
    #[inline]
    pub fn irq_disable(_irq_num: usize) {
        log::warn!("irq not implemented in loongarch64 platform yet");
    }

    /// Enable interrupt
    #[inline]
    pub fn int_enable() {
        crmd::set_ie(true);
    }

    /// Disable interrupt
    #[inline]
    pub fn int_disable() {
        crmd::set_ie(false);
    }

    /// Check if the interrupt was enabled.
    #[inline]
    pub fn int_enabled() -> bool {
        crmd::read().ie()
    }
}
