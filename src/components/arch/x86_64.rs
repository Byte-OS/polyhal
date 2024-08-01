pub(crate) mod apic;
pub(crate) mod gdt;
pub(crate) mod idt;

use alloc::vec::Vec;
use multiboot::information::MemoryType;

use crate::{components::{boot::use_multiboot, common::{DTB_BIN, MEM_AREA}, consts::VIRT_ADDR_START}, utils::LazyInit};

pub(crate) static MBOOT_PTR: LazyInit<usize> = LazyInit::new();

pub(crate) fn arch_init() {
    DTB_BIN.init_by(Vec::new());
    if let Some(mboot) = use_multiboot(*MBOOT_PTR as _) {
        let mut mem_area = Vec::new();
        if mboot.has_memory_map() {
            mboot
                .memory_regions()
                .unwrap()
                .filter(|x| x.memory_type() == MemoryType::Available)
                .for_each(|x| {
                    let start = x.base_address() as usize | VIRT_ADDR_START;
                    let size = x.length() as usize;
                    // ArchInterface::add_memory_region(start, end);
                    mem_area.push((start, size));
                });
        }
        MEM_AREA.init_by(mem_area);
    }
}

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
    let port = unsafe { (0x400 as *const u16).add(i-1).read_volatile() };
    match port {
        0 => None,
        n => Some(n)
    }
}
