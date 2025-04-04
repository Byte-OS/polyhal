use crate::PhysAddr;

pub const VIRT_ADDR_START: usize = 0x9000_0000_0000_0000;
/// QEMU Loongarch64 Virt Machine:
///     https://github.com/qemu/qemu/blob/master/include/hw/loongarch/virt.h
pub const QEMU_DTB_ADDR: PhysAddr = PhysAddr::new(0x100000);
