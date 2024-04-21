use crate::addr::PhysPage;
use crate::{TrapFrame, TrapType, PAGE_ALLOC};

extern "Rust" {
    pub fn _main_for_arch(hartid: usize);
    pub fn _interrupt_for_arch(ctx: &mut TrapFrame, trap_type: TrapType);
}

/// alloc a persistent memory page
#[inline]
pub(crate) fn frame_alloc() -> PhysPage {
    PAGE_ALLOC.alloc()
}

/// release a frame
#[inline]
pub(crate) fn frame_dealloc(ppn: PhysPage) {
    PAGE_ALLOC.dealloc(ppn)
}
