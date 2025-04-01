use crate::arch::psci;

/// Close the computer. Call PSCI.
#[inline]
pub fn shutdown() -> ! {
    psci::system_off()
}
