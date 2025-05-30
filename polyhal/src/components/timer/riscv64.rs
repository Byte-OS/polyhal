use core::time::Duration;

// TODO: Get CLOCK_FREQUENCY CLOCK_FREQ
use riscv::register::{sie, time};

const CLOCK_FREQ: u64 = 12500000;

/// Get ticks from system clock
///
/// # Return
///
/// - [u64] clock ticks
#[inline]
pub fn get_ticks() -> u64 {
    time::read64()
}

/// Get frequency of the system clock
///
/// # Return
///
/// - [u64] n ticks per second
#[inline]
pub fn get_freq() -> u64 {
    CLOCK_FREQ
}

/// Set the next timer
///
/// # parameters
///
/// - next [Duration] next time from system boot#[inline]
pub fn set_next_timer(next: Duration) {
    sbi_rt::set_timer(
        next.as_secs() * CLOCK_FREQ + next.subsec_nanos() as u64 * CLOCK_FREQ / 1_000_000_000,
    );
}

// Initialize the Timer
pub fn init() {
    unsafe {
        sie::set_stimer();
    }
    set_next_timer(Duration::ZERO);
    log::info!("initialize timer interrupt");
}
