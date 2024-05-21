use super::mem::{MOCK_PHYS_MEM, PMEM_MAP_VADDR, PMEM_SIZE};
use crate::{is_aligned, MMUFlags, PhysAddr, VirtAddr, PAGE_SIZE};

/// Errors may occur during address translation.
#[derive(Debug)]
pub enum PagingError {
    NoMemory,
    NotMapped,
    AlreadyMapped,
}

/// Address translation result.
pub type PagingResult<T = ()> = Result<T, PagingError>;

/// The [`PagingError::NotMapped`] can be ignored.
pub trait IgnoreNotMappedErr {
    /// If self is `Err(PagingError::NotMapped`, ignores the error and returns
    /// `Ok(())`, otherwise remain unchanged.
    fn ignore(self) -> PagingResult;
}

impl<T> IgnoreNotMappedErr for PagingResult<T> {
    fn ignore(self) -> PagingResult {
        match self {
            Ok(_) | Err(PagingError::NotMapped) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

/// A 4K, 2M or 1G size page.
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

    pub fn from_current() -> Self {
        Self
    }

    pub fn clone_kernel(&self) -> Self {
        Self::new()
    }
}

impl Default for PageTable {
    fn default() -> Self {
        Self::new()
    }
}

pub trait GenericPageTable: Sync + Send {
    /// Get the physical address of root page table.
    fn table_phys(&self) -> PhysAddr;

    /// Map the `page` to the frame of `paddr` with `flags`.
    fn map(&mut self, page: Page, paddr: PhysAddr, flags: MMUFlags) -> PagingResult;

    /// Unmap the page of `vaddr`.
    fn unmap(&mut self, vaddr: VirtAddr) -> PagingResult<PhysAddr>;

    /// Query the physical address which the page of `vaddr` maps to.
    fn query(&self, vaddr: VirtAddr) -> PagingResult<(PhysAddr, MMUFlags)>;

    fn map_cont(
        &mut self,
        start_vaddr: VirtAddr,
        size: usize,
        start_paddr: PhysAddr,
        flags: MMUFlags,
    ) -> PagingResult {
        assert!(is_aligned(start_vaddr.addr()));
        assert!(is_aligned(start_vaddr.addr()));
        assert!(is_aligned(size));
        let mut vaddr = start_vaddr;
        let mut paddr = start_paddr;
        let end_vaddr = vaddr.addr() + size;
        while vaddr.addr() < end_vaddr {
            let page = Page::new(vaddr);
            self.map(page, paddr, flags)?;
            vaddr.add_offset(PAGE_SIZE);
            paddr.add_offset(PAGE_SIZE);
        }
        Ok(())
    }

    fn unmap_cont(&mut self, start_vaddr: VirtAddr, size: usize) -> PagingResult {
        assert!(is_aligned(start_vaddr.addr()));
        assert!(is_aligned(size));
        let mut vaddr = start_vaddr;
        let end_vaddr = vaddr.addr() + size;
        while vaddr.addr() < end_vaddr {
            let page_size = match self.unmap(vaddr) {
                Ok(s) => { 
                    info!("unmap_cont: {:?}", s);
                    PAGE_SIZE
                }
                Err(PagingError::NotMapped) => PAGE_SIZE,
                Err(e) => return Err(e),
            };
            vaddr.add_offset(page_size as usize);
            assert!(vaddr.addr() <= end_vaddr);
        }
        Ok(())
    }
}

impl GenericPageTable for PageTable {
    fn table_phys(&self) -> PhysAddr {
        PhysAddr::new(0)
    }

    fn map(&mut self, page: Page, paddr: PhysAddr, flags: MMUFlags) -> PagingResult {
        debug_assert!(is_aligned(paddr.addr()));
        if paddr.addr() < PMEM_SIZE {
            MOCK_PHYS_MEM.mmap(page.vaddr, PAGE_SIZE, paddr, flags);
            Ok(())
        } else {
            Err(PagingError::NoMemory)
        }
    }

    fn unmap(&mut self, vaddr: VirtAddr) -> PagingResult<PhysAddr> {
        self.unmap_cont(vaddr, PAGE_SIZE)?;
        Ok(PhysAddr::new(0))
    }

    fn query(&self, vaddr: VirtAddr) -> PagingResult<(PhysAddr, MMUFlags)> {
        debug_assert!(is_aligned(vaddr.addr()));
        if (PMEM_MAP_VADDR.addr()..PMEM_MAP_VADDR.addr() + PMEM_SIZE).contains(&vaddr.addr()) {
            Ok((
                PhysAddr::new(vaddr.addr() - PMEM_MAP_VADDR.addr()),
                MMUFlags::READ | MMUFlags::WRITE,
            ))
        } else {
            Err(PagingError::NotMapped)
        }
    }

    fn unmap_cont(&mut self, vaddr: VirtAddr, size: usize) -> PagingResult {
        if size == 0 {
            return Ok(());
        }
        debug_assert!(is_aligned(vaddr.addr()));
        MOCK_PHYS_MEM.munmap(vaddr as _, size);
        Ok(())
    }
}
