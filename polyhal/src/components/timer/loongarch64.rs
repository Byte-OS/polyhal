use core::time::Duration;

use loongArch64::register::ecfg::{self, LineBasedInterrupt};
use loongArch64::register::tcfg;
/// Returns the current clock time in hardware ticks.
use loongArch64::time::{get_timer_freq, Time};
use spin::Lazy;

use crate::timer::current_time;

// static mut FREQ: usize = 0;
static FREQ: Lazy<u64> = Lazy::new(|| get_timer_freq() as _);

/// Get ticks from system clock
///
/// # Return
///
/// - [u64] clock ticks
#[inline]
pub fn get_ticks() -> u64 {
    Time::read() as _
}

/// Get frequency of the system clock
///
/// # Return
///
/// - [u64] n ticks per second
#[inline]
pub fn get_freq() -> u64 {
    *FREQ
}

/// Set the next timer
///
/// # parameters
///
/// - next [Duration] next time from system boot#[inline]
pub fn set_next_timer(next: Duration) {
    let curr = current_time();
    if next < curr {
        return;
    }
    let interval = next - curr;
    tcfg::set_init_val(
        (interval.as_secs() * get_freq()
            + interval.subsec_nanos() as u64 * get_freq() / 1_000_000_000) as _,
    );
    tcfg::set_en(true);
}

pub fn init() {
    tcfg::set_periodic(false); // set timer to one-shot mode
    tcfg::set_init_val(0); // set timer initial value
    tcfg::set_en(true); // enable timer

    let inter = LineBasedInterrupt::TIMER
        | LineBasedInterrupt::SWI0
        | LineBasedInterrupt::SWI1
        | LineBasedInterrupt::HWI0;
    ecfg::set_lie(inter);
}
