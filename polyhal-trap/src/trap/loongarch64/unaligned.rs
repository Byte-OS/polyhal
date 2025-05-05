#![allow(dead_code)]
use crate::trapframe::TrapFrame;
use core::arch::naked_asm;
use loongArch64::register::badv;

pub const LDH_OP: u32 = 0xa1;
pub const LDHU_OP: u32 = 0xa9;
pub const LDW_OP: u32 = 0xa2;
pub const LDWU_OP: u32 = 0xaa;
pub const LDD_OP: u32 = 0xa3;
pub const STH_OP: u32 = 0xa5;
pub const STW_OP: u32 = 0xa6;
pub const STD_OP: u32 = 0xa7;

pub const LDPTRW_OP: u32 = 0x24;
pub const LDPTRD_OP: u32 = 0x26;
pub const STPTRW_OP: u32 = 0x25;
pub const STPTRD_OP: u32 = 0x27;

pub const LDXH_OP: u32 = 0x7048;
pub const LDXHU_OP: u32 = 0x7008;
pub const LDXW_OP: u32 = 0x7010;
pub const LDXWU_OP: u32 = 0x7050;
pub const LDXD_OP: u32 = 0x7018;
pub const STXH_OP: u32 = 0x7028;
pub const STXW_OP: u32 = 0x7030;
pub const STXD_OP: u32 = 0x7038;

pub const FLDS_OP: u32 = 0xac;
pub const FLDD_OP: u32 = 0xae;
pub const FSTS_OP: u32 = 0xad;
pub const FSTD_OP: u32 = 0xaf;

pub const FSTXS_OP: u32 = 0x7070;
pub const FSTXD_OP: u32 = 0x7078;
pub const FLDXS_OP: u32 = 0x7060;
pub const FLDXD_OP: u32 = 0x7068;

#[allow(binary_asm_labels)]
#[naked]
unsafe extern "C" fn unaligned_read(addr: u64, value: &mut u64, n: u64, symbol: u32) -> i32 {
    naked_asm!(
        includes_trap_macros!(),
        "
            beqz	$a2, 5f

            li.w	$t1, 8
            li.w	$t2, 0

            addi.d	$t0, $a2, -1
            mul.d	$t1, $t0, $t1
            add.d 	$a0, $a0, $t0

            beq	    $a3, $zero, 2f
        1:	ld.b	$t3, $a0, 0
            b	3f

        2:	ld.bu	$t3, $a0, 0
        3:	sll.d	$t3, $t3, $t1
            or	    $t2, $t2, $t3
            addi.d	$t1, $t1, -8
            addi.d	$a0, $a0, -1
            addi.d	$a2, $a2, -1
            bgt	    $a2, $zero, 2b
        4:	st.d	$t2, $a1, 0

            move	$a0, $a2
            jr	    $ra

        5:	li.w    $a0, -1
            jr	    $ra

            FIXUP_EX 1, 6, 1
            FIXUP_EX 2, 6, 0
            FIXUP_EX 4, 6, 0
        ",
    )
}

#[allow(binary_asm_labels)]
#[naked]
unsafe extern "C" fn unaligned_write(_addr: u64, _value: u64, _n: u64) -> i32 {
    naked_asm!(
        includes_trap_macros!(),
        "
        beqz	$a2, 3f

        li.w	$t0, 0
    1:	srl.d	$t1, $a1, $t0
    2:	st.b	$t1, $a0, 0
        addi.d	$t0, $t0, 8
        addi.d	$a2, $a2, -1
        addi.d	$a0, $a0, 1
        bgt	    $a2, $zero, 1b
    
        move	$a0, $a2
        jr	    $ra
    
    3:	li.w    $a0, -1
        jr	    $ra
    
        FIXUP_EX 2, 4, 1
        ",
    )
}

#[inline]
pub unsafe fn write_bytes(addr: u64, value: u64, n: usize) {
    let ptr = addr as *mut u8;
    let bytes = value.to_ne_bytes();
    for i in 0..n {
        ptr.add(i).write_volatile(bytes[i]);
    }
}

#[allow(unused_assignments)]
pub unsafe fn emulate_load_store_insn(pt_regs: &mut TrapFrame) {
    let la_inst: u32;
    let addr: u64;
    let rd: usize;

    let mut value: u64 = 0;
    let mut res: i32 = 0;

    // debug!("Unaligned Access PC @ {:#x} ", pt_regs.era);

    unsafe {
        core::arch::asm!(
            "ld.w {val}, {addr}, 0 ",
             addr = in(reg) pt_regs.era as u64,
             val = out(reg) la_inst,
        )
    }
    addr = badv::read().vaddr() as u64;
    // debug!("badv is {:#x}", addr);
    rd = (la_inst & 0x1f) as usize;
    // debug!("rd: {}  inst: {:#x}", rd, la_inst);

    if (la_inst >> 22) == LDD_OP || (la_inst >> 24) == LDPTRD_OP || (la_inst >> 15) == LDXD_OP {
        res = unaligned_read(addr, &mut value, 8, 1);
        if res < 0 {
            panic!("Address Error @ {:#x}", addr)
        }
        pt_regs.regs[rd] = value as usize;
    } else if (la_inst >> 22) == LDW_OP
        || (la_inst >> 24) == LDPTRW_OP
        || (la_inst >> 15) == LDXW_OP
    {
        res = unaligned_read(addr, &mut value, 4, 1);
        if res < 0 {
            panic!("Address Error @ {:#x}", addr)
        }
        pt_regs.regs[rd] = value as usize;
    } else if (la_inst >> 22) == LDWU_OP || (la_inst >> 15) == LDXWU_OP {
        res = unaligned_read(addr, &mut value, 4, 0);
        if res < 0 {
            panic!("Address Error @ {:#x}", addr)
        }
        pt_regs.regs[rd] = value as usize;
    } else if (la_inst >> 22) == LDH_OP || (la_inst >> 15) == LDXH_OP {
        res = unaligned_read(addr, &mut value, 2, 1);
        if res < 0 {
            panic!("Address Error @ {:#x}", addr)
        }
        pt_regs.regs[rd] = value as usize;
    } else if (la_inst >> 22) == LDHU_OP || (la_inst >> 15) == LDXHU_OP {
        res = unaligned_read(addr, &mut value, 2, 0);
        if res < 0 {
            panic!("Address Error @ {:#x}", addr)
        }
        pt_regs.regs[rd] = value as usize;
    } else if (la_inst >> 22) == STD_OP
        || (la_inst >> 24) == STPTRD_OP
        || (la_inst >> 15) == STXD_OP
    {
        value = pt_regs.regs[rd] as u64;
        res = unaligned_write(addr, value, 8);
        // write_bytes(addr, value, 8);
    } else if (la_inst >> 22) == STW_OP
        || (la_inst >> 24) == STPTRW_OP
        || (la_inst >> 15) == STXW_OP
    {
        value = pt_regs.regs[rd] as u64;
        res = unaligned_write(addr, value, 4);
        // write_bytes(addr, value, 4);
    } else if (la_inst >> 22) == STH_OP || (la_inst >> 15) == STXH_OP {
        value = pt_regs.regs[rd] as u64;
        res = unaligned_write(addr, value, 2);
        // write_bytes(addr, value, 2);
    } else {
        panic!("unhandled unaligned address, inst:{:#x}", la_inst);
    }
    // else if (la_inst >> 22 ) == FLDD_OP
    //       ||  (la_inst >> 15 ) == FLDXD_OP {
    //     res = unaligned_read(addr, &mut value, 8, 1);
    //     if res < 0 { panic!("Address Error @ {:#x}", addr) }
    //     write_fpr(rd, value);
    // } else if (la_inst >> 22 ) == FLDS_OP
    //       ||  (la_inst >> 15 ) == FLDXS_OP {
    //     res = unaligned_read(addr, &mut value, 4, 1);
    //     if res < 0 { panic!("Address Error @ {:#x}", addr) }
    //     write_fpr(rd, value);
    // } else if (la_inst >> 22 ) == FSTD_OP
    //       ||  (la_inst >> 15 ) == FSTXD_OP {
    //    value = read_fpr(rd);
    //     res = unaligned_write(addr, value, 8);
    // } else if (la_inst >> 22 ) == FSTS_OP
    //       ||  (la_inst >> 15 ) == FSTXS_OP {
    //     value = read_fpr(rd);
    //     res = unaligned_write(addr, value, 4);
    // }

    if res < 0 {
        panic!("Address Error @ {:#x}", addr)
    }

    pt_regs.era += 4;
}
