macro_rules! includes_trap_macros {
    () => {
        r#"
        .ifndef REGS_TRAP_MACROS_FLAG
        .equ REGS_TRAP_MACROS_FLAG, 1

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

        .macro PUSH_GENERAL_REGS
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
        .endm

        .endif
        "#
    };
}
