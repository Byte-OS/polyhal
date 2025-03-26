// https://github.com/torvalds/linux/blob/3d25a941ea5013b552b96330c83052ccace73a48/arch/loongarch/include/asm/barrier.h#L20
// DBAR instruction
// Bit4: ordering or completion (0: completion, 1: ordering)
// Bit3: barrier for previous read (0: true, 1: false)
// Bit2: barrier for previous write (0: true, 1: false)
// Bit1: barrier for succeeding read (0: true, 1: false)
// Bit0: barrier for succeeding write (0: true, 1: false)
//
// Hint 0x700: barrier for "read after read" from the same address

use crate::components::mem::Barrier;

const CRWRW: usize = 0b00000;
// const CR_R_: usize	= 0b00101;
// const C_W_W: usize	= 0b01010;
const ORWRW: usize = 0b10000;
// const OR_R_: usize	= 0b10101;
// const O_W_W: usize	= 0b11010;
// const ORW_W: usize	= 0b10010;
// const OR_RW: usize	= 0b10100;

impl Barrier {
    #[inline]
    pub fn complete_sync() {
        unsafe { core::arch::asm!("dbar {}", in(reg) CRWRW) };
    }

    #[inline]
    pub fn ordering_sync() {
        unsafe { core::arch::asm!("dbar {}", in(reg) ORWRW) };
    }
}
