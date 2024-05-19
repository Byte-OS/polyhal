use bitflags::bitflags;
use crate::bit;

/// Physical address.
pub type PhysAddr = usize;

/// Virtual address.
pub type VirtAddr = usize;

/// Device address.
pub type DevVAddr = usize;

pub const PAGE_SIZE: usize = 0x1000;


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

/// The error type which is returned from HAL functions.
/// TODO: more error types.
#[derive(Debug)]
pub struct HalError;

/// The result type returned by HAL functions.
pub type HalResult<T = ()> = core::result::Result<T, HalError>;

bitflags! {
    /// Generic memory flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MMUFlags: u64 {
        #[allow(clippy::identity_op)]
        const READ      = bit!(2);
        const WRITE     = bit!(3);
        const EXECUTE   = bit!(4);
        const USER      = bit!(5);
        const HUGE_PAGE = bit!(6);
    }
}