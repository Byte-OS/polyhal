use bitflags::bitflags;
use crate::addr::{PhysAddr, VirtAddr};
use crate::bit;
use crate::common::addr::PhysPage;

pub const PAGE_SIZE: usize = 0x1000;
pub const PAGE_BITS: usize = 12;

impl PhysPage {
    pub fn addr(&self) -> PhysAddr {
        PhysAddr::new(self.0 << PAGE_BITS)
    }
}

impl VirtAddr {
    pub fn add_offset(&mut self, s: usize) {
        self.0 += s;
    }
}

impl PhysAddr {
    pub fn add_offset(&mut self, s: usize) {
        self.0 += s;
    }
}

pub const fn align_down(addr: usize) -> usize {
    addr & !(PAGE_SIZE - 1)
}

pub const fn align_up(addr: usize) -> usize {
    (addr + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

pub const fn is_aligned(addr: usize) -> bool {
    page_offset(addr) == 0
}

pub const fn page_count(size: usize) -> usize {
    align_up(size) / PAGE_SIZE
}

pub const fn page_offset(addr: usize) -> usize {
    addr & (PAGE_SIZE - 1)
}

bitflags! {
    /// Generic memory flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MMUFlags: u64 {
        #[allow(clippy::identity_op)]
        const READ      = bit!(2);
        const WRITE     = bit!(3);
        const EXECUTE   = bit!(4);
        const USER      = bit!(5);
    }
}