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
}

impl KContext {
    /// Create a new blank Kernel Context.
    pub fn blank() -> Self {
        Self {
            ksp: 0,
            ktp: 0,
            _sregs: [0; 10],
            kpc: 0,
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
