use aarch64_cpu::registers::{Readable, MPIDR_EL1};

pub(crate) mod psci;

#[inline]
pub fn hart_id() -> usize {
    MPIDR_EL1.read(MPIDR_EL1::Aff0) as _
}
