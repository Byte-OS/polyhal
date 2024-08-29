pub const SYSCALL_VECTOR: usize = 0x33445566;
/// The offset of the pic irq.
pub(crate) const PIC_VECTOR_OFFSET: u8 = 0x20;
impl super::ConfigTrait for super::GenericConfig {
    const VIRT_ADDR: usize = 0xffff_ff80_0000_0000;
}
