pub mod sigtrx;
mod sv39;

use core::arch::riscv64::sfence_vma;

pub use sv39::*;

use crate::{pagetable::TLB, VirtAddr};

/// TLB operations
impl TLB {
    /// flush the TLB entry by VirtualAddress
    /// just use it directly
    ///
    /// TLB::flush_vaddr(arg0); // arg0 is the virtual address(VirtAddr)
    #[inline]
    pub fn flush_vaddr(vaddr: VirtAddr) {
        unsafe {
            sfence_vma(vaddr.0, 0);
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
            riscv::asm::sfence_vma_all();
        }
    }
}
