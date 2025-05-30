use core::time::Duration;

use aarch64_cpu::registers::{CNTFRQ_EL0, CNTPCT_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0};
use tock_registers::interfaces::{Readable, Writeable};

use crate::components::irq::{IRQ, TIMER_IRQ_NUM};

use super::current_time;

/// Get ticks from system clock
///
/// # Return
///
/// - [u64] clock ticks
#[inline]
pub fn get_ticks() -> u64 {
    CNTPCT_EL0.get()
}

/// Get frequency of the system clock
///
/// # Return
///
/// - [u64] n ticks per second
#[inline]
pub fn get_freq() -> u64 {
    CNTFRQ_EL0.get()
}

/// Set the next timer
///
/// # parameters
///
/// - next [Duration] next time from system boot
#[inline]
pub fn set_next_timer(next: Duration) {
    let curr = current_time();
    if next < curr {
        return;
    }
    let interval = next - curr;
    CNTP_TVAL_EL0.set(
        interval.as_secs() * get_freq()
            + interval.subsec_nanos() as u64 * get_freq() / 1_000_000_000,
    );
}

pub fn init() {
    let freq = CNTFRQ_EL0.get();
    log::debug!("freq: {}", freq);
    CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
    CNTP_TVAL_EL0.set(0);
    // Enable the timer irq.
    IRQ::irq_enable(TIMER_IRQ_NUM);
    set_next_timer(Duration::ZERO);
}
