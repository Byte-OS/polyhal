use crate::addr::PhysAddr;
use crate::irq::IRQ;
use arm_gic::gic_v2::{GicCpuInterface, GicDistributor};
use arm_gic::{translate_irq, InterruptType};
use irq_safety::MutexIrqSafe;

/// The maximum number of IRQs.
#[allow(dead_code)]
pub const MAX_IRQ_COUNT: usize = 1024;

/// The timer IRQ number.
pub const TIMER_IRQ_NUM: usize = translate_irq(14, InterruptType::PPI).unwrap();

/// The UART IRQ number.
#[allow(dead_code)]
pub const UART_IRQ_NUM: usize = translate_irq(1, InterruptType::SPI).unwrap();

const GICD_BASE: PhysAddr = PhysAddr::new(0x0800_0000);
const GICC_BASE: PhysAddr = PhysAddr::new(0x0801_0000);

static GICD: MutexIrqSafe<GicDistributor> =
    MutexIrqSafe::new(GicDistributor::new(GICD_BASE.get_mut_ptr()));

// per-CPU, no lock
static GICC: GicCpuInterface = GicCpuInterface::new(GICC_BASE.get_mut_ptr());

/// Initializes GICD, GICC on the primary CPU.
pub(crate) fn init() {
    info!("Initialize GICv2...");
    GICD.lock().init();
    GICC.init();
}

#[inline]
pub fn handle_irq<F>(f: F)
where
    F: FnOnce(u32),
{
    GICC.handle_irq(f)
}

/// Implement IRQ operations for the IRQ interface.
impl IRQ {
    /// Enable irq for the given IRQ number.
    #[inline]
    pub fn enable(irq_num: usize) {
        GICD.lock().set_enable(irq_num, true);
    }

    /// Disable irq for the given IRQ number.
    #[inline]
    pub fn disable(irq_num: usize) {
        GICD.lock().set_enable(irq_num, false);
    }
}