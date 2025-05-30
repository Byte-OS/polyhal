//! Timer module.
//!
//!

use core::time::Duration;

use crate::ctor::CtorType;

super::define_arch_mods!();

/// Get current time
///
/// # Return
///
/// Return [Duration] with current time
#[inline]
pub fn current_time() -> Duration {
    let ticks = get_ticks();
    let freq = get_freq();
    Duration::new(ticks / freq, ((ticks % freq) * 1_000_000_000 / freq) as u32)
}

ph_ctor!(ARCH_INIT_TIMER, CtorType::Platform, init);
