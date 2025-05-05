macro_rules! includes_trap_macros {
    () => {
        r#"
        .ifndef REGS_TRAP_MACROS_FLAG
        .equ REGS_TRAP_MACROS_FLAG, 1

        // 2, 4, 1
        .macro FIXUP_EX from, to, fix
        .if \fix
            .section .fixup, "ax"
        \to: 
            li.w	$a0, -1
            jr	$ra
            .previous
        .endif
            .section __ex_table, "a"
            .word	\from\()b, \to\()b
            .previous
        .endm

        .equ KSAVE_KSP,  0x30
        .equ KSAVE_CTX,  0x31
        .equ KSAVE_USP,  0x32
        .equ LA_CSR_PGDL,          0x19    /* Page table base address when VA[47] = 0 */
        .equ LA_CSR_PGDH,          0x1a    /* Page table base address when VA[47] = 1 */
        .equ LA_CSR_PGD,           0x1b    /* Page table base */
        .equ LA_CSR_TLBRENTRY,     0x88    /* TLB refill exception entry */
        .equ LA_CSR_TLBRBADV,      0x89    /* TLB refill badvaddr */
        .equ LA_CSR_TLBRERA,       0x8a    /* TLB refill ERA */
        .equ LA_CSR_TLBRSAVE,      0x8b    /* KScratch for TLB refill exception */
        .equ LA_CSR_TLBRELO0,      0x8c    /* TLB refill entrylo0 */
        .equ LA_CSR_TLBRELO1,      0x8d    /* TLB refill entrylo1 */
        .equ LA_CSR_TLBREHI,       0x8e    /* TLB refill entryhi */
        .macro SAVE_REGS
            st.d    $ra, $sp,  1*8
            st.d    $tp, $sp,  2*8
            st.d    $a0, $sp,  4*8
            st.d    $a1, $sp,  5*8
            st.d    $a2, $sp,  6*8
            st.d    $a3, $sp,  7*8
            st.d    $a4, $sp,  8*8
            st.d    $a5, $sp,  9*8
            st.d    $a6, $sp, 10*8
            st.d    $a7, $sp, 11*8
            st.d    $t0, $sp, 12*8
            st.d    $t1, $sp, 13*8
            st.d    $t2, $sp, 14*8
            st.d    $t3, $sp, 15*8
            st.d    $t4, $sp, 16*8
            st.d    $t5, $sp, 17*8
            st.d    $t6, $sp, 18*8
            st.d    $t7, $sp, 19*8
            st.d    $t8, $sp, 20*8
            st.d    $r21,$sp, 21*8
            st.d    $fp, $sp, 22*8
            st.d    $s0, $sp, 23*8
            st.d    $s1, $sp, 24*8
            st.d    $s2, $sp, 25*8
            st.d    $s3, $sp, 26*8
            st.d    $s4, $sp, 27*8
            st.d    $s5, $sp, 28*8
            st.d    $s6, $sp, 29*8
            st.d    $s7, $sp, 30*8
            st.d    $s8, $sp, 31*8
            csrrd   $t0, KSAVE_USP
            st.d    $t0, $sp,  3*8

            csrrd	$t0, 0x1
            st.d	$t0, $sp, 8*32  // prmd

            csrrd   $t0, 0x6        
            st.d    $t0, $sp, 8*33  // era
        .endm

        .macro LOAD_REGS
            ld.d    $t0, $sp, 32*8
            csrwr   $t0, 0x1        // Write PRMD(PLV PIE PWE) to prmd

            ld.d    $t0, $sp, 33*8
            csrwr   $t0, 0x6        // Write Exception Address to ERA

            ld.d    $ra, $sp, 1*8
            ld.d    $tp, $sp, 2*8
            ld.d    $a0, $sp, 4*8
            ld.d    $a1, $sp, 5*8
            ld.d    $a2, $sp, 6*8
            ld.d    $a3, $sp, 7*8
            ld.d    $a4, $sp, 8*8
            ld.d    $a5, $sp, 9*8
            ld.d    $a6, $sp, 10*8
            ld.d    $a7, $sp, 11*8
            ld.d    $t0, $sp, 12*8
            ld.d    $t1, $sp, 13*8
            ld.d    $t2, $sp, 14*8
            ld.d    $t3, $sp, 15*8
            ld.d    $t4, $sp, 16*8
            ld.d    $t5, $sp, 17*8
            ld.d    $t6, $sp, 18*8
            ld.d    $t7, $sp, 19*8
            ld.d    $t8, $sp, 20*8
            ld.d    $r21,$sp, 21*8
            ld.d    $fp, $sp, 22*8
            ld.d    $s0, $sp, 23*8
            ld.d    $s1, $sp, 24*8
            ld.d    $s2, $sp, 25*8
            ld.d    $s3, $sp, 26*8
            ld.d    $s4, $sp, 27*8
            ld.d    $s5, $sp, 28*8
            ld.d    $s6, $sp, 29*8
            ld.d    $s7, $sp, 30*8
            ld.d    $s8, $sp, 31*8
            
            // restore sp
            ld.d    $sp, $sp, 3*8
        .endm

        .endif
        "#
    }
}
