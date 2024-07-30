use crate::components::irq::IRQ;


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
        log::warn!("int_enable not implemented in loongarch64 platform yet");
    }

    /// Disable interrupt
    #[inline]
    pub fn int_disable() {
        log::warn!("int_disable not implemented in loongarch64 platform yet");
    }

    /// Check if the interrupt was enabled.
    #[inline]
    pub fn int_enabled() -> bool {
        log::warn!("int_enabled not implemented in loongarch64 platform yet");
        false
    }
}
