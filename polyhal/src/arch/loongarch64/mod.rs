pub mod consts;

#[inline]
pub fn hart_id() -> usize {
    loongArch64::register::cpuid::read().core_id()
}
