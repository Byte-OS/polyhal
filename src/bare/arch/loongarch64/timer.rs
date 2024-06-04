use loongArch64::register::ecfg::{self, LineBasedInterrupt};
use loongArch64::register::tcfg;
/// Returns the current clock time in hardware ticks.
use loongArch64::time::{get_timer_freq, Time};
use spin::Lazy;
use crate::irq::IRQ;

// static mut FREQ: usize = 0;
static FREQ: Lazy<usize> = Lazy::new(|| get_timer_freq());

impl crate::time::Time {
    #[inline]
    pub fn get_freq() -> usize {
        *FREQ
    }

    /// Returns the current clock time in hardware ticks.
    #[inline]
    pub fn now() -> Self {
        Self(Time::read())
    }
}

pub fn init_timer() {
    let ticks = ((*FREQ / 1000) + 3) & !3;
    tcfg::set_periodic(true); // set timer to one-shot mode
    tcfg::set_init_val(ticks); // set timer initial value
    tcfg::set_en(true); // enable timer

    let inter = LineBasedInterrupt::TIMER
        | LineBasedInterrupt::SWI0
        | LineBasedInterrupt::SWI1
        | LineBasedInterrupt::HWI0;
    ecfg::set_lie(inter);
}

/// Implement IRQ operations for the IRQ interface.
impl IRQ {
    /// Enable irq for the given IRQ number.
    #[inline]
    pub fn enable(_irq_num: usize) {
        log::warn!("irq not implemented in loongarch64 platform yet");
    }

    /// Disable irq for the given IRQ number.
    #[inline]
    pub fn disable(_irq_num: usize) {
        log::warn!("irq not implemented in loongarch64 platform yet");
    }
}