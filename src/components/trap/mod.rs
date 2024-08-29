//! Define and initialize the trap handler.
//!
//!

use super::irq::IRQVector;
use super::trapframe::TrapFrame;

super::define_arch_mods!();

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
    Exception(usize),
    IRQ,
    Timer,
    SysCall,
}

// TODO: Implement Into EscapeReason
impl Into<EscapeReason> for TrapType {
    fn into(self) -> EscapeReason {
        match self {
            TrapType::SysCall => EscapeReason::SysCall,
            TrapType::Timer   => EscapeReason::Timer,
            TrapType::StorePageFault(e)|
            TrapType::LoadPageFault(e)|
            TrapType::InstructionPageFault(e)|
            TrapType::IllegalInstruction(e)=>EscapeReason::Exception(e),
            _ => EscapeReason::NoReason,
        }
    }
}

extern "Rust" {
    pub(crate) fn _interrupt_for_arch(ctx: &mut TrapFrame, trap_type: TrapType, token: usize);
}
