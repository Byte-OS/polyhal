use core::ptr::NonNull;

use acpi::{AcpiError, AcpiHandler, AcpiTables};

use crate::{common::{CPU_NUM, PCI_ADDR}, consts::VIRT_ADDR_START};

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
                NonNull::new((physical_address | VIRT_ADDR_START) as *mut T).unwrap(),
                size,
                size,
                AcpiImpl,
            )
        }
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {}
}

/// Detects the address of acpi through acpi_signature.
///
/// Detects in bios area.
pub(crate) fn detect_acpi() -> Result<(), AcpiError> {
    unsafe {
        match AcpiTables::search_for_rsdp_bios(AcpiImpl) {
            Ok(ref acpi_table) => {
                let madt = acpi_table.find_table::<acpi::madt::Madt>()?;
                let cpu_count = madt
                    .entries()
                    .filter(|x| matches!(x, acpi::madt::MadtEntry::LocalApic(_)))
                    .count();
                CPU_NUM.init(cpu_count);
            }
            Err(err) => log::warn!("Not Found Available ACPI: {:#x?}", err),
        }
    }
    Err(AcpiError::NoValidRsdp)
}

/// Parse informations from acpi table.
pub(crate) fn parse_acpi_info() -> Result<(), AcpiError> {
    unsafe {
        match AcpiTables::search_for_rsdp_bios(AcpiImpl) {
            Ok(ref acpi_table) => {                
                acpi::PciConfigRegions::new(acpi_table).expect("can't find pci config");
                let pci_addr = acpi::PciConfigRegions::new(acpi_table)?
                    .physical_address(0, 0, 0, 0)
                    .ok_or(AcpiError::NoValidRsdp)?;
                PCI_ADDR.init(pci_addr as _);
            }
            Err(err) => log::warn!("Not Found Available ACPI: {:#x?}", err),
        }
    }
    Err(AcpiError::NoValidRsdp)
}
