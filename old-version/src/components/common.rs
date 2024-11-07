use alloc::vec::Vec;
use fdt::Fdt;

use crate::components::arch::arch_init;
use crate::utils::InitNum;
use crate::{utils::LazyInit, PhysPage};

use super::debug_console::display_info;

#[polyhal_macro::def_percpu]
pub(crate) static CPU_ID: usize = 0;

// TODO: Hide DTB_PTR For arch not yet supported.
#[allow(dead_code)]
pub(crate) static DTB_PTR: LazyInit<usize> = LazyInit::new();

pub(crate) static PCI_ADDR: InitNum = InitNum::new(0);

/// Page Allocation trait for privoids that page allocation
pub trait PageAlloc: Sync {
    /// Allocate a physical page
    fn alloc(&self) -> PhysPage;
    /// Release a physical page
    fn dealloc(&self, ppn: PhysPage);
}

static PAGE_ALLOC: LazyInit<&dyn PageAlloc> = LazyInit::new();

/// Init arch with page allocator, like log crate
/// Please initialize the allocator before calling this function.
pub fn init(page_alloc: &'static dyn PageAlloc) {
    PAGE_ALLOC.init_by(page_alloc);

    // Init current architecture
    arch_init();
}

/// Store the number of cpu, this will fill up by startup function.
pub(crate) static CPU_NUM: InitNum = InitNum::new(0);

/// Store the memory area, this will fill up by the arch_init() function in each architecture.
pub(crate) static MEM_AREA: LazyInit<Vec<(usize, usize)>> = LazyInit::new();

/// Store the DTB_area, this will fill up by the arch_init() function in each architecture
pub(crate) static DTB_BIN: LazyInit<Vec<u8>> = LazyInit::new();

/// Get the memory area, this function should be called after initialization
pub fn get_mem_areas() -> Vec<(usize, usize)> {
    MEM_AREA.clone()
}

/// Get the fdt
pub fn get_fdt() -> Option<Fdt<'static>> {
    // Fdt::new(&DTB_BIN).ok()
    unsafe { Fdt::from_ptr(*DTB_PTR.get_unchecked() as *const u8).ok() }
}

/// Get the pci area address
pub fn get_pci_addr() -> Option<usize> {
    PCI_ADDR.get_option()
}

/// Get the number of cpus
pub fn get_cpu_num() -> usize {
    CPU_NUM.get()
}

/// alloc a persistent memory page
#[inline]
pub(crate) fn frame_alloc() -> PhysPage {
    PAGE_ALLOC.alloc()
}

/// release a frame
#[inline]
pub(crate) fn frame_dealloc(ppn: PhysPage) {
    PAGE_ALLOC.dealloc(ppn)
}

/// Parse Information from the device tree binary
///
/// Display information when booting
/// Initialize the variables and memory from device tree
#[inline]
pub(crate) fn parse_dtb_info() {
    let fdt = unsafe { Fdt::from_ptr(*DTB_PTR.get_unchecked() as *mut u8) };

    if let Ok(fdt) = fdt {
        display_info!("Platform Hart Count", "{}", fdt.cpus().count());

        fdt.memory().regions().for_each(|mm| {
            display_info!(
                "Platform Memory Region",
                "{:#p} - {:#018x}",
                mm.starting_address,
                mm.starting_address as usize + mm.size.unwrap_or(0)
            )
        });

        display_info!("Boot Args", "{}", fdt.chosen().bootargs().unwrap_or(""));

        CPU_NUM.init(fdt.cpus().count());

        fdt.all_nodes()
            .find(|x| x.name.starts_with("pci"))
            .inspect(|pci_node| {
                PCI_ADDR.init(
                    pci_node
                        .reg()
                        .map(|mut x| x.next().unwrap())
                        .unwrap()
                        .starting_address as _,
                )
            });
    } else {
        CPU_NUM.init(1);
    }
}
