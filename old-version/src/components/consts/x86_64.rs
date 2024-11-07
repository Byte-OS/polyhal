pub const SYSCALL_VECTOR: usize = 0x33445566;
/// The offset of the pic irq.
pub(crate) const PIC_VECTOR_OFFSET: u8 = 0x20;

pub const VIRT_ADDR_START: usize = 0xffff_ff80_0000_0000;
