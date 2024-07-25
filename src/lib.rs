#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(cfg_version)]
#![feature(decl_macro)]
#![feature(cfg_match)]
#![feature(offset_of)]
#![feature(used_with_arg)]
#![cfg_attr(not(version("1.79")), feature(stdsimd))]
#![feature(const_mut_refs)]
#![feature(const_slice_from_raw_parts_mut)]
#![cfg_attr(target_arch = "riscv64", feature(riscv_ext_intrinsics))]
#![cfg_attr(target_arch = "aarch64", feature(const_option))]
// For test
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

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
//! [MultiCore](multicore::MultiCore): MultiCore operations. Now only [multicore::MultiCore::boot_all] is available.
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

#[macro_use]
extern crate log;

pub mod addr;
pub mod api;
#[macro_use]
pub mod consts;
pub mod debug;
pub mod instruction;
pub mod irq;
pub mod mem;
#[cfg(feature = "multicore")]
pub mod multicore;
pub mod pagetable;
pub mod percpu;
pub mod prelude;
pub mod time;
pub mod utils;
use core::mem::size_of;

use alloc::vec::Vec;

use consts::STACK_SIZE;
use fdt::Fdt;
use utils::LazyInit;

#[cfg_attr(target_arch = "riscv64", path = "riscv64/mod.rs")]
#[cfg_attr(target_arch = "aarch64", path = "aarch64/mod.rs")]
#[cfg_attr(target_arch = "x86_64", path = "x86_64/mod.rs")]
#[cfg_attr(target_arch = "loongarch64", path = "loongarch64/mod.rs")]
mod currrent_arch;

/// Trap Frame
pub use currrent_arch::TrapFrame;

pub use currrent_arch::*;

#[cfg(feature = "boot")]
pub use polyhal_macro::arch_entry;
#[cfg(feature = "interrupt")]
pub use polyhal_macro::arch_interrupt;

// Re export the Module like Structure.
pub use addr::{PhysAddr, PhysPage, VirtAddr, VirtPage};
pub use debug::DebugConsole;
pub use irq::{IRQ, IRQVector};
pub use mem::Barrier;
pub use multicore::MultiCore;
pub use pagetable::{MappingFlags, MappingSize, PageTable, PageTableWrapper};
pub use time::Time;

pub const PAGE_SIZE: usize = PageTable::PAGE_SIZE;
pub const USER_VADDR_END: usize = PageTable::USER_VADDR_END;

/// Kernel Context Arg Type.
///
/// Using this by Index and IndexMut trait bound on KContext.
#[derive(Debug)]
#[cfg(feature = "kcontext")]
pub enum KContextArgs {
    /// Kernel Stack Pointer
    KSP,
    /// Kernel Thread Pointer
    KTP,
    /// Kernel Program Counter
    KPC,
}

/// Trap Frame Arg Type
///
/// Using this by Index and IndexMut trait bound on TrapFrame
#[derive(Debug)]
pub enum TrapFrameArgs {
    SEPC,
    RA,
    SP,
    RET,
    ARG0,
    ARG1,
    ARG2,
    TLS,
    SYSCALL,
}

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

#[derive(Debug, Clone, Copy)]
pub enum EscapeReason {
    NoReason,
    IRQ,
    Timer,
    SysCall
}

// TODO: Implement Into EscapeReason
impl Into<EscapeReason> for TrapType {
    fn into(self) -> EscapeReason {
        match self {
            TrapType::SysCall => EscapeReason::SysCall,
            _ => EscapeReason::NoReason
        }
    }
}

#[link_section = ".bss.stack"]
static mut BOOT_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

pub(crate) fn clear_bss() {
    extern "C" {
        fn _sbss();
        fn _ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(
            _sbss as usize as *mut u128,
            (_ebss as usize - _sbss as usize) / size_of::<u128>(),
        )
        .fill(0);
    }
}

/// Page Allocation trait for privoids that page allocation
pub trait PageAlloc: Sync {
    /// Allocate a physical page
    fn alloc(&self) -> PhysPage;
    /// Release a physical page
    fn dealloc(&self, ppn: PhysPage);
}

static PAGE_ALLOC: LazyInit<&dyn PageAlloc> = LazyInit::new();

/// Init arch with page allocator, like log crate
/// Please initialize the allocator before calling this function.
pub fn init(page_alloc: &'static dyn PageAlloc) {
    PAGE_ALLOC.init_by(page_alloc);

    // Init current architecture
    currrent_arch::arch_init();
}

/// Store the number of cpu, this will fill up by startup function.
pub(crate) static CPU_NUM: LazyInit<usize> = LazyInit::new();

/// Store the memory area, this will fill up by the arch_init() function in each architecture.
pub(crate) static MEM_AREA: LazyInit<Vec<(usize, usize)>> = LazyInit::new();

/// Store the DTB_area, this will fill up by the arch_init() function in each architecture
static DTB_BIN: LazyInit<Vec<u8>> = LazyInit::new();

/// Get the memory area, this function should be called after initialization
pub fn get_mem_areas() -> Vec<(usize, usize)> {
    MEM_AREA.clone()
}

/// Get the fdt
pub fn get_fdt() -> Option<Fdt<'static>> {
    Fdt::new(&DTB_BIN).ok()
}

/// Get the number of cpus
pub fn get_cpu_num() -> usize {
    *CPU_NUM
}

#[cfg(test)]
pub mod tests {
    use crate::api::frame_alloc;
    use crate::debug::{print, println};
    use crate::get_mem_areas;
    use crate::shutdown;
    use crate::PageAlloc;
    use crate::PhysPage;
    use crate::TrapFrame;
    use crate::TrapType;
    use buddy_system_allocator::LockedHeap;
    use core::panic::PanicInfo;

    pub fn test_runner(tests: &[&dyn Fn()]) {
        crate::debug::println!("Running {} tests", tests.len());
        for test in tests {
            test();
        }
    }

    #[test_case]
    fn trivial_assertion() {
        print!("trivial assertion... ");
        assert_eq!(1, 1);
        println!("[ok]");
    }

    #[global_allocator]
    static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::empty();

    #[panic_handler]
    fn panic(info: &core::panic::PanicInfo) -> ! {
        crate::debug::println!("{}", info);
        crate::shutdown()
    }

    const KERNEL_HEAP_SIZE: usize = 0x200_0000;

    static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

    pub fn init_allocator() {
        unsafe {
            HEAP_ALLOCATOR
                .lock()
                .init(HEAP_SPACE.as_ptr() as usize, KERNEL_HEAP_SIZE);
        }
    }

    pub struct PageAllocImpl;

    impl PageAlloc for PageAllocImpl {
        fn alloc(&self) -> PhysPage {
            // frame_alloc()
            todo!("alloc page")
        }

        fn dealloc(&self, ppn: PhysPage) {
            // frame::frame_dealloc(ppn)
            todo!("dealloc page");
        }
    }

    /// kernel interrupt
    #[crate::arch_interrupt]
    fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType) {
        // println!("trap_type @ {:x?} {:#x?}", trap_type, ctx);
    }

    #[crate::arch_entry]
    /// kernel main function, entry point.
    fn main(hartid: usize) {
        if hartid != 0 {
            return;
        }

        println!("[kernel] Hello, world!");
        init_allocator();

        // Init page alloc for polyhal
        crate::init(&PageAllocImpl);

        crate::test_main();

        // get_mem_areas().into_iter().for_each(|(start, size)| {
        //     println!("init memory region {:#x} - {:#x}", start, start + size);
        //     frame::add_frame_range(start, start + size);
        // });
        panic!("end of rust_main!");
    }
}
