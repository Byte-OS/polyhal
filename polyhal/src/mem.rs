use core::ptr::NonNull;

use arrayvec::ArrayVec;
use fdt_parser::{Fdt, FdtError};
use lazyinit::LazyInit;

use crate::{
    arch::{consts::VIRT_ADDR_START, MEM_VECTOR_CAPACITY},
    common::CPU_NUM,
    PhysAddr,
};

/// Memory Area
///
/// Memory Area with [MEM_VECTOR_CAPACITY].
static mut MEM_AREA: ArrayVec<(usize, usize), MEM_VECTOR_CAPACITY> = ArrayVec::new_const();

/// Device Tree Infomation
///
/// [DTB_INFO] is a lazy init value
static DTB_INFO: LazyInit<(PhysAddr, usize)> = LazyInit::new();

/// Init Device Tree Binary Pointer
///
/// # Arguments
///
/// - `dtb_ptr` is the pointer to the device tree binary.
///
pub fn init_dtb_once(dtb_ptr: PhysAddr) -> Result<(), FdtError<'static>> {
    // Validate Device Tree
    let ptr = NonNull::new(dtb_ptr.get_mut_ptr());
    let fdt = Fdt::from_ptr(ptr.unwrap())?;
    DTB_INFO.init_once((dtb_ptr, fdt.total_size()));
    fdt.memory()
        .flat_map(|x| x.regions())
        .for_each(|mm| unsafe {
            #[cfg(not(target_arch = "riscv64"))]
            add_memory_region(mm.address as _, mm.address as usize + mm.size);
            #[cfg(target_arch = "riscv64")]
            {
                let mut start = mm.address as _;
                let end = mm.address as usize + mm.size;

                // TODO: using dynamic to skip memory
                start += 0x200_000;

                add_memory_region(start, end);
            }
        });
    Ok(())
}

/// Get Flattened Device Tree
pub fn get_fdt() -> Result<Fdt<'static>, FdtError<'static>> {
    if !DTB_INFO.is_inited() {
        return Err(FdtError::BadPtr);
    }
    unsafe { Fdt::from_ptr(NonNull::new_unchecked(DTB_INFO.0.get_mut_ptr())) }
}

/// Allocate Memory From [MEM_AREA]
///
/// # Safety
///
/// - Ensure call this function in the primary core when booting
/// - Ensure no alignment required
pub unsafe fn alloc(alloc_size: usize) -> *mut u8 {
    unsafe {
        for (start, size) in MEM_AREA.iter_mut() {
            if *size > alloc_size {
                let ptr = *start;
                *start += alloc_size;
                *size -= alloc_size;
                return ptr as _;
            }
        }
        unreachable!()
    }
}

/// Parse Information from the device tree binary or Multiboot
///
/// Display information when booting
/// Initialize the variables and memory from device tree
#[inline]
pub fn parse_system_info() {
    display_info!();
    println!(include_str!("./banner.txt"));
    display_info!("Platform Arch", "{}", env!("HAL_ENV_ARCH"));
    if let Ok(fdt) = get_fdt() {
        display_info!("Boot HART ID", "{}", fdt.boot_cpuid_phys());
        display_info!("Boot HART Count", "{}", fdt.find_nodes("/cpus/cpu").count());
        CPU_NUM.init_once(fdt.find_nodes("/cpus/cpu").count());
        fdt.chosen().inspect(|chosen| {
            display_info!("Boot Args", "{}", chosen.bootargs().unwrap_or(""));
        });
        fdt.memory().flat_map(|x| x.regions()).for_each(|mm| {
            display_info!(
                "Platform Memory Region",
                "{:#p} - {:#018x}",
                mm.address,
                mm.address as usize + mm.size
            );
        });
    }
    get_mem_areas().for_each(|(address, size)| {
        display_info!(
            "Platform Memory Available",
            "{:#018x} - {:#018x}",
            address,
            address + size
        );
    });
}

/// Retrieves an iterator over the registered memory areas.
///
/// # Returns
///
/// An iterator yielding references to tuples `(start, end)`, where:
/// - `start` is the starting address of a memory area.
/// - `end` is the ending address of a memory area.
///
/// # Safety
///
/// - The caller must ensure that `MEM_AREA` is properly initialized before calling this function.
/// - Since this function returns an iterator over a static memory region, concurrent modification  
///   of `MEM_AREA` while iterating may lead to undefined behavior.
pub fn get_mem_areas<'a>() -> impl Iterator<Item = &'a (usize, usize)> {
    unsafe { MEM_AREA.iter() }
}

/// Adds a memory region to the memblock.
///
/// # Parameters
/// - `start` - The starting address of the memory region.
/// - `end` - The ending address of the memory region.
///
/// # Safety
///
/// - This function must be called from a single thread; concurrent access is **not** safe.
/// - The caller must ensure that [MEM_VECTOR_CAPACITY] is sufficient to accommodate the memory region,  
///   otherwise, this function may result in out-of-bounds memory access or undefined behavior.
pub unsafe fn add_memory_region(start: usize, end: usize) {
    if end - start == 0 {
        return;
    }
    extern "C" {
        fn _skernel();
        fn _end();
    }
    let (dtb_s, dtb_e) = DTB_INFO
        .get()
        .map(|x| (x.0.raw(), x.0.raw() + x.1))
        .unwrap_or((0, 0));
    let (self_s, self_e) = (
        _skernel as usize - VIRT_ADDR_START,
        _end as usize - VIRT_ADDR_START,
    );
    unsafe {
        if start <= self_s && self_e <= end {
            if self_s - start > 0 {
                add_memory_region(start, self_s);
            }
            if end - self_e > 0 {
                add_memory_region(self_e, end);
            }
        } else if start <= dtb_s && dtb_e <= end {
            if dtb_s - start > 0 {
                add_memory_region(start, dtb_s);
            }
            if end - dtb_e > 0 {
                add_memory_region(dtb_e, end);
            }
        } else {
            MEM_AREA.push((start, end - start));
        }
    }
}
