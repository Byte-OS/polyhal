pub(crate) fn arch_init() {}

#[inline]
pub fn hart_id() -> usize {
    loongArch64::register::cpuid::read().core_id()
}
