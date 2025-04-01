use super::{EscapeReason, TrapType};
use crate::trapframe::{FxsaveArea, TrapFrame, TRAPFRAME_SIZE};
use bitflags::bitflags;
use core::arch::{asm, global_asm};
use core::mem::{offset_of, size_of};
use polyhal::arch::apic::{local_apic, vectors::*};
use polyhal::arch::gdt::{set_tss_kernel_sp, GdtStruct};
use polyhal::consts::{PIC_VECTOR_OFFSET, SYSCALL_VECTOR};
use polyhal::irq;
use polyhal::percpu::PerCPUReserved;
use x86::{controlregs::cr2, irq::*};
use x86_64::registers::model_specific::{Efer, EferFlags, KernelGsBase, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;

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

    .macro SAVE_TP_N n
        sd  x\n, \n*8(tp)
    .endm
"
);

global_asm!(include_str!("x86_64/trap.S"));

bitflags! {
    // https://wiki.osdev.org/Exceptions#Page_Fault
    #[derive(Debug)]
    struct PageFaultFlags: u32 {
        const P = 1;
        const W = 1 << 1;
        const U = 1 << 2;
        const R = 1 << 3;
        const I = 1 << 4;
        const PK = 1 << 5;
        const SS = 1 << 6;
        const SGX = 1 << 15;
    }
}

// 内核中断回调
#[no_mangle]
fn kernel_callback(context: &mut TrapFrame) {
    let trap_type = match context.vector as u8 {
        PAGE_FAULT_VECTOR => {
            let pflags = PageFaultFlags::from_bits_truncate(context.rflags as _);
            if pflags.contains(PageFaultFlags::I) {
                TrapType::InstructionPageFault(unsafe { cr2() })
            } else if pflags.contains(PageFaultFlags::W) {
                TrapType::StorePageFault(unsafe { cr2() })
            } else {
                TrapType::LoadPageFault(unsafe { cr2() })
            }
        }
        BREAKPOINT_VECTOR => TrapType::Breakpoint,
        GENERAL_PROTECTION_FAULT_VECTOR => {
            panic!(
                "#GP @ {:#x}, fault_vaddr={:#x} error_code={:#x}:\n{:#x?}",
                context.rip,
                unsafe { cr2() },
                context.error_code,
                context
            );
        }
        APIC_TIMER_VECTOR => {
            unsafe { local_apic().end_of_interrupt() };
            TrapType::Timer
        }
        // PIC IRQS
        0x20..=0x2f => TrapType::Irq(irq::IRQVector::new(
            context.vector as usize - PIC_VECTOR_OFFSET as usize,
        )),
        _ => {
            panic!(
                "Unhandled exception {} (error_code = {:#x}) @ {:#x}:\n{:#x?}",
                context.vector, context.error_code, context.rip, context
            );
        }
    };
    unsafe { super::_interrupt_for_arch(context, trap_type, 0) };
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn kernelvec() {
    asm!(
        r"
            sub     rsp, 16                     # push fs_base, gs_base

            push    r15
            push    r14
            push    r13
            push    r12
            push    r11
            push    r10
            push    r9
            push    r8
            push    rdi
            push    rsi
            push    rbp
            push    rbx
            push    rdx
            push    rcx
            push    rax

            mov     rdi, rsp
            call    {trap_handler}

            pop     rax
            pop     rcx
            pop     rdx
            pop     rbx
            pop     rbp
            pop     rsi
            pop     rdi
            pop     r8
            pop     r9
            pop     r10
            pop     r11
            pop     r12
            pop     r13
            pop     r14
            pop     r15

            add     rsp, 32                     # pop fs_base, gs_base, vector, error_code
            iretq
        ",
        trap_handler = sym kernel_callback,
        options(noreturn)
    )
}

#[naked]
#[no_mangle]
pub unsafe extern "C" fn uservec() {
    asm!(
        r"
            sub     rsp, 16

            push    r15
            push    r14
            push    r13
            push    r12
            push    r11
            push    r10
            push    r9
            push    r8
            push    rdi
            push    rsi
            push    rbp
            push    rbx
            push    rdx
            push    rcx
            push    rax

            swapgs

            mov     rdi, rsp
            mov    rsp, gs:{PERCPU_KERNEL_RSP_OFFSET}  // kernel rsp

            pop r15
            pop r14
            pop r13
            pop r12
            pop rbx
            pop rbp
            pop rax

            mov ecx, 0xC0000100
            mov rdx, rax
            shr rdx, 32
            wrmsr                   # pop fsbase

            ret
        ",
        // PERCPU_KERNEL_RSP_OFFSET = const PERCPU_KERNEL_RSP_OFFSET,
        PERCPU_KERNEL_RSP_OFFSET = const offset_of!(PerCPUReserved, kernel_rsp),
        options(noreturn)
    );
}

#[naked]
#[no_mangle]
pub extern "C" fn user_restore(context: *mut TrapFrame) {
    unsafe {
        asm!(
            // Save callee saved registers and cs and others.
            r"
                mov ecx, 0xC0000100
                rdmsr
                shl rdx, 32
                or  rax, rdx
                push rax                # push fsbase

                push rbp
                push rbx
                push r12
                push r13
                push r14
                push r15

                mov gs:{PERCPU_KERNEL_RSP_OFFSET}, rsp
            ",
            // Write fs_base and gs_base
            "
                mov ecx, 0xC0000100
                mov edx, [rdi + 15*8+4]
                mov eax, [rdi + 15*8]
                wrmsr                   # pop fsbase
                mov ecx, 0xC0000102
                mov edx, [rdi + 16*8+4]
                mov eax, [rdi + 16*8]
                wrmsr                   # pop gsbase to kernel_gsbase
            ",
            // push fs_base
            "
                mov     rsp, rdi
                pop     rax
                pop     rcx
                pop     rdx
                pop     rbx
                pop     rbp
                pop     rsi
                pop     rdi
                pop     r8
                pop     r9
                pop     r10
                pop     r11
                pop     r12
                pop     r13
                pop     r14
                pop     r15

                add     rsp, 32         # pop fs_base,gs_base,vector,error_code
                cmp DWORD PTR [rsp - 8*2], {syscall_vector}
                je      {sysretq}
                
                swapgs
                iretq
            ",
            syscall_vector = const SYSCALL_VECTOR,
            sysretq = sym sysretq,
            // PERCPU_KERNEL_RSP_OFFSET = const PERCPU_KERNEL_RSP_OFFSET,
            PERCPU_KERNEL_RSP_OFFSET = const offset_of!(PerCPUReserved, kernel_rsp),
            options(noreturn)
        )
    }
}

#[naked]
unsafe extern "C" fn sysretq() {
    asm!(
        "
            pop rcx
            add rsp, 8
            pop r11
            pop rsp
            swapgs

            sysretq
        ",
        options(noreturn)
    )
}

pub fn init_syscall() {
    LStar::write(VirtAddr::new(syscall_entry as usize as _));
    Star::write(
        GdtStruct::UCODE64_SELECTOR,
        GdtStruct::UDATA_SELECTOR,
        GdtStruct::KCODE64_SELECTOR,
        GdtStruct::KDATA_SELECTOR,
    )
    .unwrap();
    SFMask::write(
        RFlags::TRAP_FLAG
            | RFlags::INTERRUPT_FLAG
            | RFlags::DIRECTION_FLAG
            | RFlags::IOPL_LOW
            | RFlags::IOPL_HIGH
            | RFlags::NESTED_TASK
            | RFlags::ALIGNMENT_CHECK,
    ); // TF | IF | DF | IOPL | AC | NT (0x47700)
    unsafe {
        Efer::update(|efer| *efer |= EferFlags::SYSTEM_CALL_EXTENSIONS);
    }
    KernelGsBase::write(VirtAddr::new(0));
}

#[naked]
unsafe extern "C" fn syscall_entry() {
    asm!(
        r"
            swapgs
            mov     gs:{PERCPU_USER_RSP_OFFSET}, rsp
            mov     rsp, gs:{PERCPU_USER_CONTEXT_OFFSET}
        
            sub     rsp, 8                          // skip user_ss
            push    gs:{PERCPU_USER_RSP_OFFSET}     // user_rsp
            push    r11                             // rflags
            mov     [rsp - 2 * 8], rcx              // rip
            mov     r11, {syscall_vector}
            mov     [rsp - 4 * 8], r11              // vector
            sub     rsp, 6 * 8                      // skip until general registers

            push    r15
            push    r14
            push    r13
            push    r12
            push    r11
            push    r10
            push    r9
            push    r8
            push    rdi
            push    rsi
            push    rbp
            push    rbx
            push    rdx
            push    rcx
            push    rax

            mov ecx, 0xC0000100
            rdmsr
            mov [rsp + 15*8+4], edx
            mov [rsp + 15*8], eax   # push fabase

            mov ecx, 0xC0000102
            rdmsr
            mov [rsp + 16*8+4], edx
            mov [rsp + 16*8], eax   # push gs_base
        
            mov    rsp, gs:{PERCPU_KERNEL_RSP_OFFSET}  // kernel rsp
            pop r15
            pop r14
            pop r13
            pop r12
            pop rbx
            pop rbp
            pop rax

            mov ecx, 0xC0000100
            mov rdx, rax
            shr rdx, 32
            wrmsr                   # pop fsbase
            ret
        ",
        syscall_vector = const SYSCALL_VECTOR,
        // PERCPU_USER_CONTEXT_OFFSET = const PERCPU_USER_CONTEXT_OFFSET,
        // PERCPU_USER_RSP_OFFSET = const PERCPU_USER_RSP_OFFSET,
        // PERCPU_KERNEL_RSP_OFFSET = const PERCPU_KERNEL_RSP_OFFSET,
        PERCPU_USER_CONTEXT_OFFSET = const offset_of!(PerCPUReserved, user_context),
        PERCPU_USER_RSP_OFFSET = const offset_of!(PerCPUReserved, user_rsp),
        PERCPU_KERNEL_RSP_OFFSET = const offset_of!(PerCPUReserved, kernel_rsp),
        options(noreturn)
    )
}

/// Return Some(()) if it was interrupt by syscall, otherwise None.
pub fn run_user_task(context: &mut TrapFrame) -> EscapeReason {
    // TODO: set tss kernel sp just once, before task run.
    let cx_general_top =
        context as *mut TrapFrame as usize + TRAPFRAME_SIZE - size_of::<FxsaveArea>();
    set_tss_kernel_sp(cx_general_top);
    // USER_CONTEXT.write_current(cx_general_top);
    unsafe {
        core::arch::asm!(
            "mov gs:{USER_CONTEXT}, {0}",
            in(reg) cx_general_top,
            // USER_CONTEXT = const PERCPU_USER_CONTEXT_OFFSET
            USER_CONTEXT = const offset_of!(PerCPUReserved, user_context)
        );
    }
    context.fx_area.restore();
    user_restore(context);
    context.fx_area.save();

    match context.vector {
        SYSCALL_VECTOR => {
            unsafe { super::_interrupt_for_arch(context, TrapType::SysCall, 0) };
            EscapeReason::SysCall
        }
        _ => {
            kernel_callback(context);
            EscapeReason::NoReason
        }
    }
}
