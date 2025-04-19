//! Define and initialize the trap handler.
//!
//!

use super::trapframe::TrapFrame;
use polyhal::{ctor::CtorType, irq::IRQVector, ph_ctor};

polyhal_macro::define_arch_mods!();

#[derive(Debug, Clone, Copy)]
pub enum TrapType {
    Breakpoint,
    SysCall,
    Timer,
    Unknown,
    SupervisorExternal,
    StorePageFault(usize),
    LoadPageFault(usize),
    InstructionPageFault(usize),
    IllegalInstruction(usize),
    Irq(IRQVector),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeReason {
    NoReason,
    IRQ,
    Timer,
    SysCall,
}

// TODO: Add more trap types as needed
impl From<TrapType> for EscapeReason {
    fn from(value: TrapType) -> Self {
        match value {
            TrapType::SysCall => EscapeReason::SysCall,
            TrapType::Timer => EscapeReason::Timer,
            TrapType::Irq(_) => EscapeReason::IRQ,
            _ => EscapeReason::NoReason,
        }
    }
}

extern "Rust" {
    pub(crate) fn _interrupt_for_arch(ctx: &mut TrapFrame, trap_type: TrapType, token: usize);
}

ph_ctor!(TRAP_INIT, CtorType::Cpu, init);
