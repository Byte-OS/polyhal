// TODO: Get CLOCK_FREQUENCY CLOCK_FREQ
use crate::time::Time;
use riscv::register::{sie, time};

const CLOCK_FREQ: usize = 12500000;

impl Time {
    #[inline]
    pub fn get_freq() -> usize {
        CLOCK_FREQ
    }

    #[inline]
    pub fn now() -> Self {
        Self(time::read())
    }
}

// Setting the time interval for then next time
#[inline]
pub fn set_next_timeout() {
    // Setting the timer through calling SBI.
    sbi_rt::set_timer((time::read() + CLOCK_FREQ / 100) as _);
}

// Initialize the Timer 
pub fn init() {
    unsafe {
        sie::set_stimer();
    }
    set_next_timeout();
    log::info!("initialize timer interrupt");
}
