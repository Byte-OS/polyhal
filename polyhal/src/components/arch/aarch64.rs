use aarch64_cpu::registers::{Readable, MPIDR_EL1};

pub(crate) mod psci;

#[inline]
pub fn hart_id() -> usize {
    MPIDR_EL1.read(MPIDR_EL1::Aff0) as _
}

pub(crate) fn arch_init() {}
