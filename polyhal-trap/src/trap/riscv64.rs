use super::{EscapeReason, TrapType};
use crate::trapframe::TrapFrame;
use core::arch::{asm, global_asm};
use polyhal::{consts::VIRT_ADDR_START, timer};
use riscv::{
    interrupt::{Exception, Interrupt},
    register::{
        scause::{self, Trap},
        stval,
        stvec::{self, Stvec},
    },
};

global_asm!(
    r"
    .altmacro
    .macro LOAD reg, offset
        ld  \reg, \offset*8(sp)
    .endm

    .macro SAVE reg, offset
        sd  \reg, \offset*8(sp)
    .endm

    .macro LOAD_N n
        ld  x\n, \n*8(sp)
    .endm

    .macro SAVE_N n
        sd  x\n, \n*8(sp)
    .endm

    .macro SAVE_GENERAL_REGS
        SAVE    x1, 1
        csrr    x1, sscratch
        SAVE    x1, 2
        .set    n, 3
        .rept   29 
            SAVE_N  %n
        .set    n, n + 1
        .endr

        csrr    t0, sstatus
        csrr    t1, sepc
        SAVE    t0, 32
        SAVE    t1, 33
    .endm

    .macro LOAD_GENERAL_REGS
        LOAD    t0, 32
        LOAD    t1, 33
        csrw    sstatus, t0
        csrw    sepc, t1

        LOAD    x1, 1
        .set    n, 3
        .rept   29
            LOAD_N  %n
        .set    n, n + 1
        .endr
        LOAD    x2, 2
    .endm

    .macro LOAD_PERCPU dst, sym
        lui  \dst, %hi(__PERCPU_\sym)
        add  \dst, \dst, gp
        ld   \dst, %lo(__PERCPU_\sym)(\dst)
    .endm

    .macro SAVE_PERCPU sym, temp, src
        lui  \temp, %hi(__PERCPU_\sym)
        add  \temp, \temp, gp
        sd   \src,  %lo(__PERCPU_\sym)(\temp)
    .endm
"
);

#[no_mangle]
#[polyhal_macro::def_percpu]
static KERNEL_RSP: usize = 0;

#[no_mangle]
#[polyhal_macro::def_percpu]
static USER_RSP: usize = 0;

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
    asm!(
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
        options(noreturn)
    )
}

#[naked]
#[no_mangle]
extern "C" fn user_restore(context: *mut TrapFrame) {
    unsafe {
        asm!(
            r"
                .align 4
                .altmacro
            ",
            // 在内核态栈中开一个空间来存储内核态信息
            // 下次发生中断必然会进入中断入口然后恢复这个上下文.
            // 仅保存 Callee-saved regs、gp、tp、ra.
            "   addi    sp, sp, -18*8
                
                sd      sp, 8*1(sp)
                sd      gp, 8*2(sp)
                sd      tp, 8*3(sp)
                sd      s0, 8*4(sp)
                sd      s1, 8*5(sp)
                sd      s2, 8*6(sp)
                sd      s3, 8*7(sp)
                sd      s4, 8*8(sp)
                sd      s5, 8*9(sp)
                sd      s6, 8*10(sp)
                sd      s7, 8*11(sp)
                sd      s8, 8*12(sp)
                sd      s9, 8*13(sp)
                sd      s10, 8*14(sp)
                sd      s11, 8*15(sp)
                sd      a0,  8*16(sp)
                sd      ra,  8*17(sp)
            ",
            // 将栈信息保存到用户栈.
            // a0 是传入的Context, 然后下面会再次恢复 sp 地址.
            "   sd      sp, 8*0(a0)
                csrw    sscratch, a0
                mv      sp, a0
            
                .short  0x2452      # fld  fs0, 272(sp)
                .short  0x24f2      # fld  fs1, 280(sp)

                LOAD_GENERAL_REGS
                sret
            ",
            options(noreturn)
        )
    }
}

#[naked]
#[no_mangle]
#[allow(named_asm_labels)]
pub unsafe extern "C" fn uservec() {
    asm!(
        r"
        .altmacro
    ",
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
        ld      gp, 8*2(sp)
        ld      tp, 8*3(sp)
        ld      s0, 8*4(sp)
        ld      s1, 8*5(sp)
        ld      s2, 8*6(sp)
        ld      s3, 8*7(sp)
        ld      s4, 8*8(sp)
        ld      s5, 8*9(sp)
        ld      s6, 8*10(sp)
        ld      s7, 8*11(sp)
        ld      s8, 8*12(sp)
        ld      s9, 8*13(sp)
        ld      s10, 8*14(sp)
        ld      s11, 8*15(sp)
        ld      ra,  8*17(sp)
        
        ld      sp, 8(sp)
    ",
        // 回收栈
        "addi sp, sp, 18*8
        ret
    ",
        options(noreturn)
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
