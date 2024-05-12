use bitflags::bitflags;
use numeric_enum_macro::numeric_enum;

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
    pub struct MMUFlags: usize {
        #[allow(clippy::identity_op)]
        const CACHE_1   = 1 << 0;
        const CACHE_2   = 1 << 1;
        const READ      = 1 << 2;
        const WRITE     = 1 << 3;
        const EXECUTE   = 1 << 4;
        const USER      = 1 << 5;
        const HUGE_PAGE = 1 << 6;
        const DEVICE    = 1 << 7;
        const RXW = Self::READ.bits | Self::WRITE.bits | Self::EXECUTE.bits;
    }
}