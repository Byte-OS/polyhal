use crate::addr::PhysPage;
use crate::{TrapFrame, TrapType, PAGE_ALLOC};

extern "Rust" {
    #[cfg(feature = "boot")]
    pub(crate) fn _main_for_arch(hartid: usize);
    #[cfg(feature = "interrupt")]
    pub(crate) fn _interrupt_for_arch(ctx: &mut TrapFrame, trap_type: TrapType, token: usize);
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
