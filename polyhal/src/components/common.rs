use lazyinit::LazyInit;

use crate::PhysAddr;

/// Page Allocation trait for privoids that page allocation
pub trait PageAlloc: Sync {
    /// Allocate a physical page
    fn alloc(&self) -> PhysAddr;
    /// Release a physical page
    fn dealloc(&self, paddr: PhysAddr);
}

static PAGE_ALLOC: LazyInit<&dyn PageAlloc> = LazyInit::new();

/// Init arch with page allocator, like log crate
/// Please initialize the allocator before calling this function.
pub fn init(page_alloc: &'static dyn PageAlloc) {
    PAGE_ALLOC.init_once(page_alloc);
}

/// Store the number of cpu, this will fill up by startup function.
pub(crate) static CPU_NUM: LazyInit<usize> = LazyInit::new();

/// Get the number of cpus
pub fn get_cpu_num() -> usize {
    *CPU_NUM
}

/// alloc a persistent memory page
#[inline]
pub(crate) fn frame_alloc() -> PhysAddr {
    PAGE_ALLOC.alloc()
}

/// release a frame
#[inline]
pub(crate) fn frame_dealloc(paddr: PhysAddr) {
    PAGE_ALLOC.dealloc(paddr)
}
