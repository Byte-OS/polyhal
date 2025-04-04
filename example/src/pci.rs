use log::{info, trace};
use polyhal::{consts::VIRT_ADDR_START, mem::get_fdt};
use virtio_drivers::transport::pci::{
    bus::{Cam, Command, DeviceFunction, HeaderType, PciRoot},
    virtio_device_type,
};

/// Initialize PCI Configuration.
pub fn init() {
    if let Ok(fdt) = get_fdt() {
        if let Some(pci_node) = fdt.all_nodes().find(|x| x.name.starts_with("pci")) {
            let pci_addr = pci_node.reg().map(|mut x| x.next().unwrap()).unwrap();
            log::info!("PCI Address: {:#x}", pci_addr.address);
            enumerate_pci((pci_addr.address as usize | VIRT_ADDR_START) as *mut u8);
            return;
        }
    }
    #[cfg(target_arch = "x86_64")]
    {
        if let Some(addr) = polyhal::acpi::get_pci_base() {
            enumerate_pci(addr.get_mut_ptr());
        }
    }
}

/// Enumerate the PCI devices
fn enumerate_pci(mmconfig_base: *mut u8) {
    info!("mmconfig_base = {:#x}", mmconfig_base as usize);

    let mut pci_root = unsafe { PciRoot::new(mmconfig_base, Cam::Ecam) };
    for (device_function, info) in pci_root.enumerate_bus(0) {
        // Skip if it is not a PCI Type0 device (Standard PCI device).
        if info.header_type != HeaderType::Standard {
            continue;
        }
        let (status, command) = pci_root.get_status_command(device_function);
        info!(
            "Found {} at {}, status {:?} command {:?}",
            info, device_function, status, command
        );

        if info.vendor_id == 0x8086 && info.device_id == 0x100e {
            // Detected E1000 Net Card
            pci_root.set_command(
                device_function,
                Command::IO_SPACE | Command::MEMORY_SPACE | Command::BUS_MASTER,
            );
        }
        for i in 0..6 {
            dump_bar_contents(&mut pci_root, device_function, i);
        }
        if let Some(virtio_type) = virtio_device_type(&info) {
            info!("  VirtIO {:?}", virtio_type);

            // Enable the device to use its BARs.
            pci_root.set_command(
                device_function,
                Command::IO_SPACE | Command::MEMORY_SPACE | Command::BUS_MASTER,
            );
        }
    }
}

/// Dump bar Contents.
fn dump_bar_contents(root: &mut PciRoot, device_function: DeviceFunction, bar_index: u8) {
    let bar_info = root.bar_info(device_function, bar_index).unwrap();
    if let Some((_addr, size)) = bar_info.memory_address_size() {
        if size == 0 {
            return;
        }
        trace!("Dumping bar {}: {:#x?}", bar_index, bar_info);
    }
}
