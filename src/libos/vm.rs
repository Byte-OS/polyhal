use super::mem::{MOCK_PHYS_MEM, PMEM_SIZE};
use crate::{is_aligned, MMUFlags, addr::{PhysAddr, VirtAddr}, PAGE_SIZE};

#[derive(Debug, Copy, Clone)]
pub struct Page {
    pub vaddr: VirtAddr,
}

impl Page {
    pub fn new(vaddr: VirtAddr) -> Self {
        Self { vaddr }
    }
}

/// Dummy page table implemented by `mmap`, `munmap`, and `mprotect`.
pub struct PageTable;

impl PageTable {
    pub fn new() -> Self {
        Self
    }
}

impl PageTable {
    pub fn map(&mut self, page: Page, paddr: PhysAddr, flags: MMUFlags) {
        debug_assert!(is_aligned(paddr.addr()));
        if paddr.addr() < PMEM_SIZE {
            MOCK_PHYS_MEM.mmap(page.vaddr, PAGE_SIZE, paddr, flags);
        } else {
            error!("failed to map(no memory): paddr={:#x}, flags={:?}", paddr.addr(), flags);
        }
    }

    pub fn unmap(&mut self, vaddr: VirtAddr) {
        MOCK_PHYS_MEM.munmap(vaddr as _, PAGE_SIZE);
    }
}
