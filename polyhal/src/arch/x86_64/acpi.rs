use core::ptr::NonNull;

use acpi::{AcpiHandler, AcpiTables};

use crate::{common::CPU_NUM, PhysAddr};

#[derive(Clone)]
struct AcpiImpl;

impl AcpiHandler for AcpiImpl {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        unsafe {
            acpi::PhysicalMapping::new(
                physical_address,
                NonNull::new(pa!(physical_address).get_mut_ptr()).unwrap(),
                size,
                size,
                AcpiImpl,
            )
        }
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {}
}

pub fn init() {
    unsafe {
        match AcpiTables::search_for_rsdp_bios(AcpiImpl) {
            Ok(ref acpi_table) => {
                let madt = acpi_table.find_table::<acpi::madt::Madt>().unwrap();
                let cpu_count = madt
                    .get()
                    .entries()
                    .filter(|x| matches!(x, acpi::madt::MadtEntry::LocalApic(_)))
                    .count();
                CPU_NUM.init_once(cpu_count);
            }
            Err(err) => println!("acpi error: {:#x?}", err),
        }
    }
}

/// Get The Base Address Of The PCI
///
/// Return [Option::None] if the pci is not exists or error
pub fn get_pci_base() -> Option<PhysAddr> {
    unsafe {
        AcpiTables::search_for_rsdp_bios(AcpiImpl)
            .ok()?
            .find_table::<acpi::mcfg::Mcfg>()
            .ok()?
            .entries()
            .iter()
            .next()
            .map(|x| pa!(x.base_address))
    }
}

pub fn get_pm1a_addr() -> Option<usize> {
    unsafe {
        AcpiTables::search_for_rsdp_bios(AcpiImpl)
            .ok()?
            .find_table::<acpi::fadt::Fadt>()
            .ok()?
            .pm1a_control_block()
            .ok()
            .map(|x| x.address as usize)
    }
}
