//! Timer module.
//!
//!

use core::time::Duration;

use crate::Time;

super::define_arch_mods!();

/// Get current time
///
/// # Return
///
/// Return [Duration] with current time
#[inline]
pub fn current_time() -> Duration {
    Duration::from_nanos(Time::now().raw() as _)
}
