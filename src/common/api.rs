use crate::PhysPage;
use crate::{TrapFrame, TrapType, PAGE_ALLOC};

extern "Rust" {
    pub(crate) fn _main_for_arch(hartid: usize);
    pub(crate) fn _interrupt_for_arch(ctx: &mut TrapFrame, trap_type: TrapType);
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
