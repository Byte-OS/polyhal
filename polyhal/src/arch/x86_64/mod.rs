pub mod acpi;
pub mod apic;
pub mod consts;
pub mod gdt;
pub mod idt;

pub fn hart_id() -> usize {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.initial_local_apic_id() as usize,
        None => 0,
    }
}

/// Get the port of COMx, x in range [1,4]
/// BIOS Data Area: https://wiki.osdev.org/Memory_Map_(x86)#BIOS_Data_Area_(BDA)
pub(crate) fn get_com_port(i: usize) -> Option<u16> {
    if i > 0x4 || i == 0 {
        return None;
    }
    let port = unsafe { (0x400 as *const u16).add(i - 1).read_volatile() };
    match port {
        0 => None,
        n => Some(n),
    }
}
