use core::mem::size_of;

pub const VIRT_ADDR_START: usize = 0xffff_ff80_0000_0000;

pub const SYSCALL_VECTOR: usize = 0x33445566;
/// The offset of the pic irq.
pub(super) const PIC_VECTOR_OFFSET: u8 = 0x20;

/// Reserved percpu index
pub(super) const PERCPU_USER_RSP_OFFSET:        usize = 1 * size_of::<usize>();
pub(super) const PERCPU_KERNEL_RSP_OFFSET:      usize = 2 * size_of::<usize>();
pub(super) const PERCPU_USER_CONTEXT_OFFSET:    usize = 3 * size_of::<usize>();
