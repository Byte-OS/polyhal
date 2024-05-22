use loongArch64::register::pgdl;

use crate::{addr::{PhysAddr, PhysPage, VirtAddr, VirtPage}, MappingFlags, PageTable, PTE, TLB};

use super::sigtrx::get_trx_mapping;
use crate::bit;

impl PTE {
    #[inline]
    pub const fn from_addr(ppn: PhysAddr, flags: PTEFlags) -> Self {
        PTE(ppn.0 | flags.bits())
    }

    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub const fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.0)
    }

    #[inline]
    pub fn address(&self) -> PhysAddr {
        PhysAddr((self.0) & 0xffff_ffff_f000)
    }

    #[inline]
    pub fn is_table(&self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub(crate) fn new_table(ppn: PhysPage) -> Self {
        Self(ppn.to_addr())
    }

    #[inline]
    pub(crate) fn new_page(ppn: PhysPage, flags: PTEFlags) -> Self {
        Self(ppn.to_addr() | flags.bits() as usize)
    }
}

impl From<MappingFlags> for PTEFlags {
    fn from(value: MappingFlags) -> Self {
        let mut flags = PTEFlags::V;
        if value.contains(MappingFlags::W) {
            flags |= PTEFlags::W | PTEFlags::D;
        }

        // if !value.contains(MappingFlags::X) {
        //     flags |= PTEFlags::NX;
        // }

        if value.contains(MappingFlags::U) {
            flags |= PTEFlags::PLV_USER;
        }
        flags
    }
}

impl Into<MappingFlags> for PTEFlags {
    fn into(self) -> MappingFlags {
        let mut flags = MappingFlags::empty();
        if self.contains(PTEFlags::W) {
            flags |= MappingFlags::W;
        }

        if self.contains(PTEFlags::D) {
            flags |= MappingFlags::D;
        }

        // if !self.contains(PTEFlags::NX) {
        //     flags |= MappingFlags::X;
        // }

        if self.contains(PTEFlags::PLV_USER) {
            flags |= MappingFlags::U;
        }
        flags
    }
}

bitflags::bitflags! {
    /// Possible flags for a page table entry.
    pub struct PTEFlags: usize {
        /// Page Valid
        const V = bit!(0);
        /// Dirty, The page has been writed.
        const D = bit!(1);

        const PLV_USER = 0b11 << 2;

        const MAT_NOCACHE = 0b01 << 4;

        /// Designates a global mapping OR Whether the page is huge page.
        const GH = bit!(6);

        /// Page is existing.
        const P = bit!(7);
        /// Page is writeable.
        const W = bit!(8);
        /// Is a Global Page if using huge page(GH bit).
        const G = bit!(10);
        /// Page is not readable.
        const NR = bit!(11);
        /// Page is not executable.
        /// FIXME: Is it just for a huge page?
        /// Linux related url: https://github.com/torvalds/linux/blob/master/arch/loongarch/include/asm/pgtable-bits.h
        const NX = bit!(12);
        /// Whether the privilege Level is restricted. When RPLV is 0, the PTE
        /// can be accessed by any program with privilege Level highter than PLV.
        const RPLV = bit!(63);
    }
}

impl PageTable {
    /// The size of the page for this platform.
    pub(crate) const PAGE_SIZE: usize = 0x1000;
    pub(crate) const PAGE_LEVEL: usize = 3;
    pub(crate) const PTE_NUM_IN_PAGE: usize = 0x200;
    pub(crate) const GLOBAL_ROOT_PTE_RANGE: usize = 0x100;
    pub(crate) const VADDR_BITS: usize = 39;
    pub(crate) const USER_VADDR_END: usize = (1 << Self::VADDR_BITS) - 1;
    pub(crate) const KERNEL_VADDR_START: usize = !Self::USER_VADDR_END;

    #[inline]
    pub fn restore(&self) {
        self.release();
        self.0.slice_mut_with_len::<PTE>(0x200)[0x100] = PTE(get_trx_mapping());

        TLB::flush_all();
    }

    #[inline]
    pub fn current() -> Self {
        Self(PhysAddr(pgdl::read().base()))
    }

    #[inline]
    pub fn change(&self) {
        pgdl::set_base(self.0.addr());
        TLB::flush_all();
    }
}

/// TLB operations
impl TLB {
    /// flush the TLB entry by VirtualAddress
    /// just use it directly
    ///
    /// TLB::flush_vaddr(arg0); // arg0 is the virtual address(VirtAddr)
    #[inline]
    pub fn flush_vaddr(vaddr: VirtAddr) {
        unsafe {
            core::arch::asm!("dbar 0; invtlb 0x05, $r0, {reg}", reg = in(reg) vaddr.0);
        }
    }

    /// flush all tlb entry
    ///
    /// how to use ?
    /// just
    /// TLB::flush_all();
    #[inline]
    pub fn flush_all() {
        unsafe {
            core::arch::asm!("dbar 0; invtlb 0x00, $r0, $r0");
        }
    }
}

pub fn kernel_page_table() -> PageTable {
    // FIXME: This should return a valid page table.
    // ref solution: create a blank page table in boot stage.
    PageTable(PhysAddr(0))
}

impl VirtPage {
    /// Get n level page table index of the given virtual address
    #[inline]
    pub fn pn_index(&self, n: usize) -> usize {
        (self.0 >> 9 * n) & 0x1ff
    }
}

impl VirtAddr {
    /// Get n level page table offset of the given virtual address
    #[inline]
    pub fn pn_offest(&self, n: usize) -> usize {
        self.0 % (1 << (12 + 9 * n))
    }
}
