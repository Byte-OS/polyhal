#[macro_use]
mod macros;

use super::{EscapeReason, TrapType};
use crate::trapframe::TrapFrame;
use core::arch::naked_asm;
use polyhal::{consts::VIRT_ADDR_START, timer};
use riscv::{
    interrupt::{Exception, Interrupt},
    register::{
        scause::{self, Trap},
        stval,
        stvec::{self, Stvec},
    },
};

// Initialize the trap handler.
pub(crate) fn init() {
    unsafe {
        let mut stvec = Stvec::from_bits(0);
        stvec.set_address(kernelvec as usize);
        stvec.set_trap_mode(stvec::TrapMode::Direct);
        stvec::write(stvec);
    }

    // Initialize the timer component
    polyhal::timer::init();
}

// 内核中断回调
#[no_mangle]
fn kernel_callback(context: &mut TrapFrame) -> TrapType {
    let scause = scause::read();
    let stval = stval::read();

    let trap_type = match scause.cause().try_into().unwrap() {
        // 中断异常
        Trap::Exception(Exception::Breakpoint) => {
            context.sepc += 2;
            TrapType::Breakpoint
        }
        Trap::Exception(Exception::LoadFault) => {
            if stval > VIRT_ADDR_START {
                panic!("kernel error: {:#x}", stval);
            }
            TrapType::Unknown
        }
        Trap::Exception(Exception::UserEnvCall) => TrapType::SysCall,
        // 时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            timer::set_next_timeout();
            TrapType::Timer
        }
        Trap::Exception(Exception::StorePageFault) => TrapType::StorePageFault(stval),
        Trap::Exception(Exception::StoreFault) => TrapType::StorePageFault(stval),
        Trap::Exception(Exception::InstructionPageFault) => TrapType::InstructionPageFault(stval),
        Trap::Exception(Exception::IllegalInstruction) => TrapType::IllegalInstruction(stval),
        Trap::Exception(Exception::LoadPageFault) => TrapType::LoadPageFault(stval),
        Trap::Interrupt(Interrupt::SupervisorExternal) => TrapType::SupervisorExternal,
        _ => {
            log::error!(
                "内核态中断发生: {:#x} {:?}  stval {:#x}  sepc: {:#x}",
                scause.bits(),
                scause.cause(),
                stval,
                context.sepc
            );
            panic!("未知中断: {:#x?}", context);
        }
    };
    unsafe { super::_interrupt_for_arch(context, trap_type, 0) };
    trap_type
}

#[naked]
pub unsafe extern "C" fn kernelvec() {
    naked_asm!(
        includes_trap_macros!(),
        // 宏定义
        r"
            .align 4
            .altmacro
        
            csrrw   sp, sscratch, sp
            bnez    sp, uservec
            csrr    sp, sscratch

            addi    sp, sp, -{cx_size}
            
            SAVE_GENERAL_REGS
            csrw    sscratch, x0

            mv      a0, sp

            call kernel_callback

            LOAD_GENERAL_REGS
            sret
        ",
        cx_size = const crate::trapframe::TRAPFRAME_SIZE,
    )
}

#[naked]
#[no_mangle]
extern "C" fn user_restore(context: *mut TrapFrame) {
    unsafe {
        naked_asm!(
            includes_trap_macros!(),
            // 在内核态栈中开一个空间来存储内核态信息
            // 下次发生中断必然会进入中断入口然后恢复这个上下文.
            // 仅保存 Callee-saved regs、gp、tp、ra.
            ".align 4
                addi    sp, sp, -18*8

                STR      sp,  1
                STR      gp,  2
                STR      tp,  3
                STR      s0,  4
                STR      s1,  5
                STR      s2,  6
                STR      s3,  7
                STR      s4,  8
                STR      s5,  9
                STR      s6,  10
                STR      s7,  11
                STR      s8,  12
                STR      s9,  13
                STR      s10, 14
                STR      s11, 15
                STR      a0,  16
                STR      ra,  17
            ",
            // 将栈信息保存到用户栈.
            // a0 是传入的Context, 然后下面会再次恢复 sp 地址.
            "   sd       sp, 8*0(a0)
                csrw     sscratch, a0
                mv       sp, a0

                .short   0x2452      # fld  fs0, 272(sp)
                .short   0x24f2      # fld  fs1, 280(sp)

                LOAD_GENERAL_REGS
                sret
            ",
        )
    }
}

#[naked]
#[no_mangle]
#[allow(named_asm_labels)]
pub unsafe extern "C" fn uservec() {
    naked_asm!(
        includes_trap_macros!(),
        // 保存 general registers, 除了 sp
        "
        SAVE_GENERAL_REGS
        csrw    sscratch, x0

        .word   0x10813827          # fsd fs0, 272(sp)
        .word   0x10913c27          # fsd fs1, 280(sp)

        mv      a0, sp
        ld      sp, 0*8(a0)
        sd      x0, 0*8(a0)
    ",
        // 恢复内核上下文信息, 仅恢复 callee-saved 寄存器和 ra、gp、tp
        "  
        LDR      gp,  2
        LDR      tp,  3
        LDR      s0,  4
        LDR      s1,  5
        LDR      s2,  6
        LDR      s3,  7
        LDR      s4,  8
        LDR      s5,  9
        LDR      s6,  10
        LDR      s7,  11
        LDR      s8,  12
        LDR      s9,  13
        LDR      s10, 14
        LDR      s11, 15
        LDR      ra,  17
        
        LDR      sp,  1
    ",
        // 回收栈
        "addi sp, sp, 18*8
        ret
    ",
    );
}

/// Return EscapeReson related to interrupt type.
pub fn run_user_task(context: &mut TrapFrame) -> EscapeReason {
    user_restore(context);
    kernel_callback(context).into()
}

/// Run user task until interrupt is received.
pub fn run_user_task_forever(context: &mut TrapFrame) -> ! {
    loop {
        user_restore(context);
        kernel_callback(context);
    }
}
