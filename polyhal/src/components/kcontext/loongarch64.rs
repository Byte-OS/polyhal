use core::mem::offset_of;
use core::{
    arch::naked_asm,
    ops::{Index, IndexMut},
};

use crate::{components::kcontext::KContextArgs, pagetable::PageTable};

/// Save the task context registers.
macro_rules! save_callee_regs {
    () => {
        "
            st.d      $sp, $a0,  0*8
            st.d      $tp, $a0,  1*8
            st.d      $s9, $a0,  2*8
            st.d      $s0, $a0,  3*8
            st.d      $s1, $a0,  4*8
            st.d      $s2, $a0,  5*8
            st.d      $s3, $a0,  6*8
            st.d      $s4, $a0,  7*8
            st.d      $s5, $a0,  8*8
            st.d      $s6, $a0,  9*8
            st.d      $s7, $a0, 10*8
            st.d      $s8, $a0, 11*8
            st.d      $ra, $a0, 12*8
        "
    };
}

/// Restore the task context registers.
macro_rules! restore_callee_regs {
    () => {
        "
            ld.d      $sp, $a1,  0*8
            ld.d      $tp, $a1,  1*8
            ld.d      $s9, $a1,  2*8
            ld.d      $s0, $a1,  3*8
            ld.d      $s1, $a1,  4*8
            ld.d      $s2, $a1,  5*8
            ld.d      $s3, $a1,  6*8
            ld.d      $s4, $a1,  7*8
            ld.d      $s5, $a1,  8*8
            ld.d      $s6, $a1,  9*8
            ld.d      $s7, $a1, 10*8
            ld.d      $s8, $a1, 11*8
            ld.d      $ra, $a1, 12*8
        "
    };
}

#[cfg(feature = "fp_simd")]
macro_rules! save_fp_regs {
    () => {
        "
            // LoongArch64 specific floating point macros
            fst.d $f0,  $a0, 0*8
            fst.d $f1,  $a0, 1*8
            fst.d $f2,  $a0, 2*8
            fst.d $f3,  $a0, 3*8
            fst.d $f4,  $a0, 4*8
            fst.d $f5,  $a0, 5*8
            fst.d $f6,  $a0, 6*8
            fst.d $f7,  $a0, 7*8
            fst.d $f8,  $a0, 8*8
            fst.d $f9,  $a0, 9*8
            fst.d $f10, $a0, 10*8
            fst.d $f11, $a0, 11*8
            fst.d $f12, $a0, 12*8
            fst.d $f13, $a0, 13*8
            fst.d $f14, $a0, 14*8
            fst.d $f15, $a0, 15*8
            fst.d $f16, $a0, 16*8
            fst.d $f17, $a0, 17*8
            fst.d $f18, $a0, 18*8
            fst.d $f19, $a0, 19*8
            fst.d $f20, $a0, 20*8
            fst.d $f21, $a0, 21*8
            fst.d $f22, $a0, 22*8
            fst.d $f23, $a0, 23*8
            fst.d $f24, $a0, 24*8
            fst.d $f25, $a0, 25*8
            fst.d $f26, $a0, 26*8
            fst.d $f27, $a0, 27*8
            fst.d $f28, $a0, 28*8
            fst.d $f29, $a0, 29*8
            fst.d $f30, $a0, 30*8
            fst.d $f31, $a0, 31*8

            addi.d $t8, $a0, 32*8

            // SAVE_FCC
            movcf2gr    $t0, $fcc0
            move        $t1, $t0
            movcf2gr    $t0, $fcc1
            bstrins.d   $t1, $t0, 15, 8
            movcf2gr    $t0, $fcc2
            bstrins.d   $t1, $t0, 23, 16
            movcf2gr    $t0, $fcc3
            bstrins.d   $t1, $t0, 31, 24
            movcf2gr    $t0, $fcc4
            bstrins.d   $t1, $t0, 39, 32
            movcf2gr    $t0, $fcc5
            bstrins.d   $t1, $t0, 47, 40
            movcf2gr    $t0, $fcc6
            bstrins.d   $t1, $t0, 55, 48
            movcf2gr    $t0, $fcc7
            bstrins.d   $t1, $t0, 63, 56
            st.d        $t1, $t8, 0

            addi.d $t8, $a0, 33*8

            // SAVE_FCSR
            movfcsr2gr  $t0, $fcsr0
            st.w        $t0, $t8, 0
        "
    };
}

#[cfg(feature = "fp_simd")]
macro_rules! restore_fp_regs {
    () => {
        "
            // LoongArch64 specific floating point macros
            fld.d $f0,  $a0, 0*8
            fld.d $f1,  $a0, 1*8
            fld.d $f2,  $a0, 2*8
            fld.d $f3,  $a0, 3*8
            fld.d $f4,  $a0, 4*8
            fld.d $f5,  $a0, 5*8
            fld.d $f6,  $a0, 6*8
            fld.d $f7,  $a0, 7*8
            fld.d $f8,  $a0, 8*8
            fld.d $f9,  $a0, 9*8
            fld.d $f10, $a0, 10*8
            fld.d $f11, $a0, 11*8
            fld.d $f12, $a0, 12*8
            fld.d $f13, $a0, 13*8
            fld.d $f14, $a0, 14*8
            fld.d $f15, $a0, 15*8
            fld.d $f16, $a0, 16*8
            fld.d $f17, $a0, 17*8
            fld.d $f18, $a0, 18*8
            fld.d $f19, $a0, 19*8
            fld.d $f20, $a0, 20*8
            fld.d $f21, $a0, 21*8
            fld.d $f22, $a0, 22*8
            fld.d $f23, $a0, 23*8
            fld.d $f24, $a0, 24*8
            fld.d $f25, $a0, 25*8
            fld.d $f26, $a0, 26*8
            fld.d $f27, $a0, 27*8
            fld.d $f28, $a0, 28*8
            fld.d $f29, $a0, 29*8
            fld.d $f30, $a0, 30*8
            fld.d $f31, $a0, 31*8

            addi.d $t8, $a0, 32*8

            // RESTORE_FCC
            ld.d        $t0, $t8, 0
            bstrpick.d  $t1, $t0, 7, 0
            movgr2cf    $fcc0, $t1
            bstrpick.d  $t1, $t0, 15, 8
            movgr2cf    $fcc1, $t1
            bstrpick.d  $t1, $t0, 23, 16
            movgr2cf    $fcc2, $t1
            bstrpick.d  $t1, $t0, 31, 24
            movgr2cf    $fcc3, $t1
            bstrpick.d  $t1, $t0, 39, 32
            movgr2cf    $fcc4, $t1
            bstrpick.d  $t1, $t0, 47, 40
            movgr2cf    $fcc5, $t1
            bstrpick.d  $t1, $t0, 55, 48
            movgr2cf    $fcc6, $t1
            bstrpick.d  $t1, $t0, 63, 56
            movgr2cf    $fcc7, $t1

            addi.d $t8, $a0, 33*8

            // RESTORE_FCSR
            ld.w        $t0, $t8, 0
            movgr2fcsr  $fcsr0, $t0
        "
    };
}

/// Floating-point registers of LoongArch64.
#[cfg(feature = "fp_simd")]
#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct FpStatus {
    /// Floating-point registers (f0-f31)
    pub fp: [u64; 32],
    /// Floating-point Condition Code register
    pub fcc: [u8; 8],
    /// Floating-point Control and Status register
    pub fcsr: usize,
}

/// Kernel Context
///
/// Kernel Context is used to switch context between kernel task.
#[derive(Debug)]
#[repr(C)]
pub struct KContext {
    /// Kernel Stack Pointer
    ksp: usize,
    /// Kernel Thread Pointer
    ktp: usize,
    /// Kernel Static registers, r22 - r31 (r22 is s9, s0 - s8)
    _sregs: [usize; 10],
    /// Kernel Program Counter, Will return to this address.
    kpc: usize,
    #[cfg(feature = "fp_simd")]
    /// Floating Point Status
    fp_status: FpStatus,
}

impl KContext {
    /// Create a new blank Kernel Context.
    pub fn blank() -> Self {
        Self {
            ksp: 0,
            ktp: 0,
            _sregs: [0; 10],
            kpc: 0,
            #[cfg(feature = "fp_simd")]
            fp_status: FpStatus::default(),
        }
    }
}

/// Indexing operations for KContext
///
/// Using it just like the Vector.
///
/// #[derive(Debug)]
/// pub enum KContextArgs {
///     /// Kernel Stack Pointer
///     KSP,
///     /// Kernel Thread Pointer
///     KTP,
///     /// Kernel Program Counter
///     KPC
/// }
///
/// etc. Get reg of the kernel stack:
///
/// let ksp = KContext[KContextArgs::KSP]
/// let kpc = KContext[KContextArgs::KPC]
/// let ktp = KContext[KContextArgs::KTP]
///
impl Index<KContextArgs> for KContext {
    type Output = usize;

    fn index(&self, index: KContextArgs) -> &Self::Output {
        match index {
            KContextArgs::KSP => &self.ksp,
            KContextArgs::KTP => &self.ktp,
            KContextArgs::KPC => &self.kpc,
        }
    }
}

/// Indexing Mutable operations for KContext
///
/// Using it just like the Vector.
///
/// etc. Change the value of the kernel Context using IndexMut
///
/// KContext[KContextArgs::KSP] = ksp;
/// KContext[KContextArgs::KPC] = kpc;
/// KContext[KContextArgs::KTP] = ktp;
///
impl IndexMut<KContextArgs> for KContext {
    fn index_mut(&mut self, index: KContextArgs) -> &mut Self::Output {
        match index {
            KContextArgs::KSP => &mut self.ksp,
            KContextArgs::KTP => &mut self.ktp,
            KContextArgs::KPC => &mut self.kpc,
        }
    }
}

/// Context Switch
///
/// Save the context of current task and switch to new task.
#[naked]
pub unsafe extern "C" fn context_switch(from: *mut KContext, to: *const KContext) {
    naked_asm!(
        // Save Kernel Context.
        save_callee_regs!(),
        // Restore Kernel Context.
        restore_callee_regs!(),
        // Return to the caller.
        "ret",
    )
}

/// Context Switch With Page Table
///
/// Save the context of current task and switch to new task.
#[inline]
pub unsafe extern "C" fn context_switch_pt(
    from: *mut KContext,
    to: *const KContext,
    pt_token: PageTable,
) {
    context_switch_pt_impl(from, to, pt_token.root().raw());
}

/// Context Switch With Page Table Implement
///
/// The detail implementation of [context_switch_pt].
#[naked]
unsafe extern "C" fn context_switch_pt_impl(
    from: *mut KContext,
    to: *const KContext,
    pt_token: usize,
) {
    naked_asm!(
        // Save Kernel Context.
        save_callee_regs!(),
        // Switch to new page table.
        // Write PageTable to pgdl(CSR 0x19)
        "
            csrwr     $a2, 0x19
            dbar      0
            invtlb    0x00, $r0, $r0
        ",
        // Restore Kernel Context.
        restore_callee_regs!(),
        // Return to the caller.
        "ret",
    )
}

#[cfg(feature = "fp_simd")]
#[naked]
pub unsafe extern "C" fn save_fp_regs(from: *mut KContext) {
    naked_asm!(
        // Save floating point registers.
        "addi.d    $a0, $a0, {fp_offset}",
        save_fp_regs!(),
        "ret",
        fp_offset = const offset_of!(KContext, fp_status)
    )
}

#[cfg(feature = "fp_simd")]
#[naked]
pub unsafe extern "C" fn restore_fp_regs(to: *const KContext) {
    naked_asm!(
        "addi.d    $a0, $a0, {fp_offset}",
        restore_fp_regs!(),
        "ret",
        fp_offset = const offset_of!(KContext, fp_status)
    )
}

#[naked]
pub extern "C" fn read_current_tp() -> usize {
    unsafe {
        naked_asm!(
            "
                move    $a0, $tp
                ret
            ",
        )
    }
}
