#![allow(unused_imports)]

use aarch64_cpu::registers::{CNTFRQ_EL0, CNTPCT_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0};
use tock_registers::interfaces::{Readable, Writeable};

use crate::time::Time;

impl Time {
    #[inline]
    pub fn get_freq() -> usize {
        CNTFRQ_EL0.get() as _
    }

    /// Returns the current clock time in hardware ticks.
    #[inline]
    pub fn now() -> Self {
        Self(CNTPCT_EL0.get() as _)
    }
}

pub fn set_next_timer() {
    CNTP_TVAL_EL0.set(CNTFRQ_EL0.get() / 1000);
}

pub fn init() {
    let freq = CNTFRQ_EL0.get();
    debug!("freq: {}", freq);
    CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
    CNTP_TVAL_EL0.set(0);
    // Enable the timer irq.
    // TODO: Enable IRQ Timer.
    // IRQ::irq_enable(super::gic::TIMER_IRQ_NUM);
    set_next_timer();
}
