use crate::arch::apic::{io_apic, local_apic};
use crate::components::irq::{IRQVector, IRQ};

/// Implement IRQ operations for the IRQ interface.
impl IRQ {
    /// Enable irq for the given IRQ number.
    #[inline]
    pub fn irq_enable(irq_num: usize) {
        unsafe {
            io_apic().lock().enable_irq(irq_num as _);
        }
    }

    /// Disable irq for the given IRQ number.
    #[inline]
    pub fn irq_disable(irq_num: usize) {
        unsafe {
            io_apic().lock().disable_irq(irq_num as _);
        }
    }

    /// Enable interrupts.
    #[inline]
    pub fn int_enable() {
        x86_64::instructions::interrupts::enable();
    }

    /// Disable interrupts.
    #[inline]
    pub fn int_disable() {
        x86_64::instructions::interrupts::disable();
    }

    /// Check if the interrupts was enabled.
    #[inline]
    pub fn int_enabled() -> bool {
        x86_64::instructions::interrupts::are_enabled()
    }
}

/// Implmente the irq vector methods
impl IRQVector {
    /// Get the irq number in this vector
    #[inline]
    pub fn irq_num(&self) -> usize {
        self.0
    }

    /// Acknowledge the irq
    pub fn ack(&self) {
        unsafe { local_apic().end_of_interrupt() };
    }
}
