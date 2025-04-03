//! IRQ(Interrupt requests queue) module.
//!
//! How to use this interface.
//! ```rust
//! // Init irq
//! IRQ::init();
//! [IRQ::irq_enable]
//! // Enable irq 3
//! IRQ::irq_enable(3);
//! [IRQ::irq_disable]
//! // Disable irq 3
//! IRQ::irq_disable(3);
//! [IRQ::irq_enabled]
//! // Check if irq is enabled
//! // Return true if the irq is enabled.
//! IRQ::irq_enabled(3);
//! [IRQ::int_enable]
//! // Enable interrupt
//! IRQ::int_enable();
//! [IRQ::int_disable]
//! // Disable interrupt
//! IRQ::int_disable();
//! [IRQ::int_enabled]
//! // Check if interrupt is enabled
//! let enabled = IRQ::int_enabled();
//! ```

super::define_arch_mods!();

pub struct IRQ;

impl IRQ {}

#[derive(Debug, Clone, Copy)]
pub struct IRQVector(usize);

impl IRQVector {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }
}
