use crate::addr::PhysPage;
use crate::{TrapFrame, TrapType, PAGE_ALLOC};

/// ArchInterface
///
/// This trait indicates the interface was should be implemented
/// from the kernel layer.
///
/// You need to implement the interface manually.
///
/// eg: in kernel/src/main.rs
///
/// ```rust
/// #[crate_interface::impl_interface]
/// impl ArchInterface for ArchInterfaceImpl {
///     /// kernel interrupt
///     fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType) {}
///     /// kernel main function, entry point.
///     fn main(hartid: usize) {}
/// }
/// ```

#[crate_interface::def_interface]
pub trait ArchInterface {
    /// kernel interrupt
    fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType);
    /// kernel main function, entry point.
    fn main(hartid: usize);
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
