//! Physical memory operations.

use alloc::vec::Vec;
use core::ops::Range;
use crate::utils::init_once::InitOnce;

use super::mock_mem::MockMemory;
use crate::{PhysAddr, VirtAddr, PAGE_SIZE};

/// Map physical memory from here.
pub(super) const PMEM_MAP_VADDR: VirtAddr = 0x8_0000_0000;
/// Physical memory size = 1GiB
pub(super) const PMEM_SIZE: usize = 0x4000_0000;

pub(super) static MOCK_PHYS_MEM: InitOnce<MockMemory> = InitOnce::new();

pub fn init_mock_mem() {
    MOCK_PHYS_MEM.init_once_by(MockMemory::new(PMEM_SIZE));
}

pub fn free_pmem_regions() -> Vec<Range<PhysAddr>> {
    vec![PAGE_SIZE..PMEM_SIZE]
}

pub fn pmem_read(paddr: PhysAddr, buf: &mut [u8]) {
    trace!("pmem read: paddr={:#x}, len={:#x}", paddr, buf.len());
    assert!(paddr + buf.len() <= PMEM_SIZE);
    let src = MOCK_PHYS_MEM.as_ptr(paddr);
    unsafe { buf.as_mut_ptr().copy_from_nonoverlapping(src, buf.len()) };
}

pub fn pmem_write(paddr: PhysAddr, buf: &[u8]) {
    trace!("pmem write: paddr={:#x}, len={:#x}", paddr, buf.len());
    assert!(paddr + buf.len() <= PMEM_SIZE);
    let dst = MOCK_PHYS_MEM.as_mut_ptr::<u8>(paddr);
    unsafe { dst.copy_from_nonoverlapping(buf.as_ptr(), buf.len()) };
}

pub fn pmem_zero(paddr: PhysAddr, len: usize) {
    trace!("pmem_zero: addr={:#x}, len={:#x}", paddr, len);
    assert!(paddr + len <= PMEM_SIZE);
    unsafe { core::ptr::write_bytes(MOCK_PHYS_MEM.as_mut_ptr::<u8>(paddr), 0, len) };
}

pub fn pmem_copy(dst: PhysAddr, src: PhysAddr, len: usize) {
    trace!("pmem_copy: {:#x} <- {:#x}, len={:#x}", dst, src, len);
    assert!(src + len <= PMEM_SIZE && dst + len <= PMEM_SIZE);
    let dst = MOCK_PHYS_MEM.as_mut_ptr::<u8>(dst);
    let src = MOCK_PHYS_MEM.as_ptr::<u8>(src);
    unsafe { dst.copy_from_nonoverlapping(src, len) };
}