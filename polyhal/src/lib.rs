#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(cfg_version)]
#![feature(decl_macro)]
#![feature(cfg_match)]
#![feature(used_with_arg)]
#![cfg_attr(not(version("1.79")), feature(stdsimd))]
#![feature(const_mut_refs)]
#![feature(const_slice_from_raw_parts_mut)]
#![feature(const_trait_impl)]
#![cfg_attr(target_arch = "riscv64", feature(riscv_ext_intrinsics))]
#![cfg_attr(target_arch = "aarch64", feature(const_option))]

//! This is a crate to help you supporting multiple platforms.
//!
//! If you want to use this crate, you should implement the following trait in your code.
//!
//! ```rust
//! /// impl
//! pub struct PageAllocImpl;
//!
//! impl PageAlloc for PageAllocImpl {
//!     fn alloc(&self) -> PhysPage {
//!         frame_alloc()
//!     }
//!
//!     fn dealloc(&self, ppn: PhysPage) {
//!         frame::frame_dealloc(ppn)
//!     }
//! }
//!
//! /// kernel interrupt
//! #[polyhal::arch_interrupt]
//! fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType) {
//!     // println!("trap_type @ {:x?} {:#x?}", trap_type, ctx);
//!     match trap_type {
//!         Breakpoint => return,
//!         UserEnvCall => {
//!             // jump to next instruction anyway
//!             ctx.syscall_ok();
//!             log::info!("Handle a syscall");
//!         }
//!         StorePageFault(_paddr) | LoadPageFault(_paddr) | InstructionPageFault(_paddr) => {
//!             log::info!("page fault");
//!         }
//!         IllegalInstruction(_) => {
//!             log::info!("illegal instruction");
//!         }
//!         Time => {
//!             log::info!("Timer");
//!         }
//!         _ => {
//!             log::warn!("unsuspended trap type: {:?}", trap_type);
//!         }
//!     }
//! }
//!
//! #[polyhal::arch_entry]
//! /// kernel main function, entry point.
//! fn main(hartid: usize) {
//!     if hartid != 0 {
//!         return;
//!     }
//!
//!     println!("[kernel] Hello, world!");
//!     allocator::init_allocator();
//!     logging::init(Some("trace"));
//!     println!("init logging");
//!
//!     // Init page alloc for polyhal
//!     polyhal::init(&PageAllocImpl);
//!
//!     get_mem_areas().into_iter().for_each(|(start, size)| {
//!         println!("init memory region {:#x} - {:#x}", start, start + size);
//!         frame::add_frame_range(start, start + size);
//!     });
//!     panic!("end of rust_main!");
//! }
//!
//! ```
//!
//! The main(hardid: usize) is the entry point.
//!
//! You can find details in the example.
//!
//! In this crate you can find some interfaces to use.
//! These interfaces are classified into some structures.
//!
//! [PhysPage]: PhysicalPage And its associated functions.
//!
//! [PhysAddr](addr::PhysAddr): PhysicalAddr And its associated functions.
//!
//! [VirtPage](addr::VirtPage): VirtualPage And its associated functions.
//!
//! [VirtAddr](addr::VirtAddr): VirtualAddr And its associated functions.
//!
//! [IRQ](irq::IRQ): Interrupt ReQuest management, includes enable and disable.
//!
//! [Barrier](mem::Barrier): Memory barrier operations.
//!
//! [PageTable]: PageTable and its associated functions.
//!
//! [MappingFlags](pagetable::MappingFlags): MappingFlags, This is an abstraction of pagetable flags.
//!
//! [TLB](pagetable::TLB): TLB operations.
//!
//! [PageTableWrapper](pagetable::PageTableWrapper): PageTableWrapper. It will dealloc all pagetable leaf when it was dropping.
//!
//! [Time](time::Time): Time and its associated functions.
//!
//! [Instruction](instruction::Instruction): Some platform instruction.
//!
//! There also provides a debugging console(recommanded only for debugging).
//!
//! [DebugConsole](debug::DebugConsole): A console for debugging.
//!
//! This crate provides a [TrapFrame], you can operate it through index with [TrapFrameArgs].
//!
//! If you are using kernel task. You should to enable feature `kcontext`.
//! Then you can use kernel task context structure [KContext], and manipulate it with [KContextArgs].
//!
//! You can switch kcontext through [context_switch_pt] or [context_switch]
//!
//! There are also some consts.
//!
//! [VIRT_ADDR_START]: This is a higher half kernel offset address.
//! [USER_VADDR_END]: End of the user address range.
//! [PAGE_SIZE]: The size of the page.
//!
//! You can get some device information using the functions below.
//! [get_mem_areas]: Get the avaliable memorys.
//! [get_fdt]: Get the Fdt structure(fdt is a rust dtb operation crate).
//! [get_cpu_num]: Get the number of cpus.
//!
//! TIPS: You should have finished [init] before using [get_mem_areas] and [get_fdt].

extern crate alloc;

extern crate log;

pub mod arch;
mod components;
pub use components::*;
pub mod time;
pub mod utils;

pub use utils::addr::{PhysAddr, VirtAddr};

#[cfg(feature = "boot")]
pub use polyhal_macro::arch_entry;
#[cfg(feature = "trap")]
pub use polyhal_macro::arch_interrupt;

// Re export the Module like Structure.
pub use components::pagetable::{MappingFlags, MappingSize, PageTable, PageTableWrapper};
pub use time::Time;
