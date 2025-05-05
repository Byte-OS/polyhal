use crate::components::kcontext::KContextArgs;
use crate::PageTable;
use core::arch::naked_asm;
use core::ops::{Index, IndexMut};
use x86_64::registers::model_specific::FsBase;

/// Save the task context registers.
macro_rules! save_callee_regs {
    () => {
        "
            mov     [rdi + 0 * 8], rsp
            mov     [rdi + 2 * 8], rbx
            mov     [rdi + 3 * 8], rbp
            mov     [rdi + 4 * 8], r12
            mov     [rdi + 5 * 8], r13
            mov     [rdi + 6 * 8], r14
            mov     [rdi + 7 * 8], r15
            mov     [rdi + 8 * 8], r8     # save old rip to stack    
            
            mov     ecx, 0xC0000100
            rdmsr
            mov     [rdi + 1*8],    eax   # push fabase
            mov     [rdi + 1*8+4],  edx  
        "
    };
}

/// Restore the task context registers.
macro_rules! restore_callee_regs {
    () => {
        "
            mov     ecx, 0xC0000100
            mov     eax, [rsi + 1*8]
            mov     edx, [rsi + 1*8+4]
            wrmsr                         # pop fsbase
            mov     rsp, [rsi + 0 * 8]
            mov     rbx, [rsi + 2 * 8]
            mov     rbp, [rsi + 3 * 8]
            mov     r12, [rsi + 4 * 8]
            mov     r13, [rsi + 5 * 8]
            mov     r14, [rsi + 6 * 8]
            mov     r15, [rsi + 7 * 8]
            mov     r8,  [rsi + 8 * 8]
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
    // Callee saved register
    rbx: usize,
    // Callee saved register
    rbp: usize,
    // Callee saved register
    r12: usize,
    // Callee saved register
    r13: usize,
    // Callee saved register
    r14: usize,
    // Callee saved register
    r15: usize,
    /// Kernel Program Counter, Will return to this address.
    kpc: usize,
}

impl KContext {
    /// Create a new blank Kernel Context.
    pub fn blank() -> Self {
        Self {
            ksp: 0,
            ktp: 0,
            rbx: 0,
            rbp: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
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
///
/// # Safety
///
/// This function is unsafe because it performs a context switch, which can lead to undefined behavior if not used correctly.
#[naked]
pub unsafe extern "C" fn context_switch(from: *mut KContext, to: *const KContext) {
    naked_asm!(
        // Save Kernel Context.
        "
        pop     r8 
        ",
        save_callee_regs!(),
        // Restore Kernel Context.
        restore_callee_regs!(),
        "
        push    r8
        ret
        ",
    )
}

/// Context Switch With Page Table
///
/// Save the context of current task and switch to new task.
///
/// # Safety
///
/// This function is unsafe because it performs a context switch, which can lead to undefined behavior if not used correctly.
/// It also requires a valid page table token.
/// The page table token is used to switch the page table for the new task.
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
        // consume the return address(rip) in the stack
        // for consistency with context_switch.
        // and save page table to r9
        "
            pop     r8
            mov     r9, rdx
        ",
        // Save Kernel Context.
        save_callee_regs!(),
        // Switch to new page table.
        "
            mov     cr3,   r9
        ",
        // Restore Kernel Context.
        restore_callee_regs!(),
        "
            push    r8
            ret
        ",
    )
}

/// Read thread pointer currently.
#[inline]
pub fn read_current_tp() -> usize {
    FsBase::read().as_u64() as _
}
