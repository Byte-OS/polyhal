use loongArch64::register::ecfg::{self, LineBasedInterrupt};
use loongArch64::register::tcfg;
/// Returns the current clock time in hardware ticks.
use loongArch64::time::{get_timer_freq, Time};
use spin::Lazy;

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

pub fn init() {
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
