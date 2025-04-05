use aarch64_cpu::registers::{Writeable, TTBR0_EL1};

use super::{MappingFlags, PageTable, PTE, TLB};
use crate::{PhysAddr, VirtAddr};

impl PTE {
    #[inline]
    pub const fn address(&self) -> PhysAddr {
        PhysAddr::new(self.0 & 0xFFFF_FFFF_F000)
    }

    #[inline]
    #[allow(dead_code)]
    pub fn set(&mut self, ppn: usize, flags: PTEFlags) {
        self.0 = (ppn << 10) | flags.bits() as usize;
    }

    #[inline]
    pub const fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits_truncate(self.0)
    }

    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::VALID)
    }

    #[inline]
    pub fn is_table(&self) -> bool {
        self.flags().contains(PTEFlags::NON_BLOCK | PTEFlags::VALID)
    }

    #[inline]
    pub fn new_table(paddr: PhysAddr) -> Self {
        Self(paddr.raw() | PTEFlags::VALID.bits() | PTEFlags::NON_BLOCK.bits())
    }

    #[inline]
    pub fn new_page(paddr: PhysAddr, flags: PTEFlags) -> Self {
        Self(paddr.raw() | flags.bits() as usize)
    }
}

impl From<MappingFlags> for PTEFlags {
    fn from(value: MappingFlags) -> Self {
        let mut flags = PTEFlags::VALID | PTEFlags::NON_BLOCK | PTEFlags::AF;
        if !value.contains(MappingFlags::W) {
            flags |= PTEFlags::AP_RO;
        }

        if !value.contains(MappingFlags::X) {
            flags |= PTEFlags::UXN | PTEFlags::PXN;
        }

        if value.contains(MappingFlags::U) {
            flags |= PTEFlags::AP_EL0;
        }
        if !value.contains(MappingFlags::G) {
            flags |= PTEFlags::NG
        }
        flags
    }
}

impl Into<MappingFlags> for PTEFlags {
    fn into(self) -> MappingFlags {
        if self.is_empty() {
            return MappingFlags::empty();
        };
        let mut flags = MappingFlags::R;

        if !self.contains(PTEFlags::AP_RO) {
            flags |= MappingFlags::W;
        }
        if !self.contains(PTEFlags::UXN) || !self.contains(PTEFlags::PXN) {
            flags |= MappingFlags::X;
        }
        if self.contains(PTEFlags::AP_EL0) {
            flags |= MappingFlags::U;
        }
        if self.contains(PTEFlags::AF) {
            flags |= MappingFlags::A;
        }
        if !self.contains(PTEFlags::NG) {
            flags |= MappingFlags::G;
        }
        flags
    }
}

bitflags::bitflags! {
    /// Possible flags for a page table entry.
    pub struct PTEFlags: usize {
        // Attribute fields in stage 1 VMSAv8-64 Block and Page descriptors:
        /// Whether the descriptor is valid.
        const VALID =       bit!(0);
        /// The descriptor gives the address of the next level of translation table or 4KB page.
        /// (not a 2M, 1G block)
        const NON_BLOCK =   bit!(1);
        /// Memory attributes index field.
        const ATTR_INDX =   0b111 << 2;
        const NORMAL_NONCACHE = 0b010 << 2;
        /// Non-secure bit. For memory accesses from Secure state, specifies whether the output
        /// address is in Secure or Non-secure memory.
        const NS =          bit!(5);
        /// Access permission: accessable at EL0.
        const AP_EL0 =      bit!(6);
        /// Access permission: read-only.
        const AP_RO =       bit!(7);
        /// Shareability: Inner Shareable (otherwise Outer Shareable).
        const INNER =       bit!(8);
        /// Shareability: Inner or Outer Shareable (otherwise Non-shareable).
        const SHAREABLE =   bit!(9);
        /// The Access flag.
        const AF =          bit!(10);
        /// The not global bit.
        const NG =          bit!(11);
        /// Indicates that 16 adjacent translation table entries point to contiguous memory regions.
        const CONTIGUOUS =  bit!(52);
        /// The Privileged execute-never field.
        const PXN =         bit!(53);
        /// The Execute-never or Unprivileged execute-never field.
        const UXN =         bit!(54);

        // Next-level attributes in stage 1 VMSAv8-64 Table descriptors:

        /// PXN limit for subsequent levels of lookup.
        const PXN_TABLE =           bit!(59);
        /// XN limit for subsequent levels of lookup.
        const XN_TABLE =            bit!(60);
        /// Access permissions limit for subsequent levels of lookup: access at EL0 not permitted.
        const AP_NO_EL0_TABLE =     bit!(61);
        /// Access permissions limit for subsequent levels of lookup: write access not permitted.
        const AP_NO_WRITE_TABLE =   bit!(62);
        /// For memory accesses from Secure state, specifies the Security state for subsequent
        /// levels of lookup.
        const NS_TABLE =            bit!(63);
    }
}

impl PageTable {
    /// The size of the page for this platform.
    pub const PAGE_SIZE: usize = 0x1000;
    pub const PAGE_LEVEL: usize = 4;
    pub const PTE_NUM_IN_PAGE: usize = 0x200;
    pub(crate) const GLOBAL_ROOT_PTE_RANGE: usize = 0x200;

    #[inline]
    pub fn current() -> Self {
        Self(PhysAddr::new(TTBR0_EL1.get_baddr() as _))
    }

    #[inline]
    pub fn restore(&self) {
        self.release();
        TLB::flush_all();
    }

    #[inline]
    pub fn change(&self) {
        TTBR0_EL1.set((self.0.raw() & 0xFFFF_FFFF_F000) as _);
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
            core::arch::asm!(
                "
                    tlbi vaale1is, {}
                    dsb sy
                    isb
                ", 
                in(reg) ((vaddr.raw() >> 12) & 0xFFFF_FFFF_FFFF)
            )
        }
    }

    /// flush all tlb entry
    ///
    /// how to use ?
    /// just
    /// TLB::flush_all();
    #[inline]
    pub fn flush_all() {
        unsafe { core::arch::asm!("tlbi vmalle1; dsb sy; isb") }
    }
}
