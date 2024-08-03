#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod allocator;
mod frame;
mod logging;
use core::panic::PanicInfo;

use frame::frame_alloc;
use polyhal::addr::PhysPage;
use polyhal::common::{get_mem_areas, PageAlloc};
use polyhal::debug_console::DebugConsole;
use polyhal::instruction::Instruction;
// use polyhal::trap::TrapType::{self, *};
// use polyhal::trapframe::TrapFrame;

pub struct PageAllocImpl;

impl PageAlloc for PageAllocImpl {
    fn alloc(&self) -> PhysPage {
        frame_alloc()
    }

    fn dealloc(&self, ppn: PhysPage) {
        frame::frame_dealloc(ppn)
    }
}

/// kernel interrupt
// #[polyhal::arch_interrupt]
// fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType) {
//     // println!("trap_type @ {:x?} {:#x?}", trap_type, ctx);
//     match trap_type {
//         Breakpoint => return,
//         SysCall => {
//             // jump to next instruction anyway
//             ctx.syscall_ok();
//             log::info!("Handle a syscall");
//         }
//         StorePageFault(paddr) | LoadPageFault(paddr) | InstructionPageFault(paddr) => {
//             log::info!("page fault: {:#x}", paddr);
//         }
//         IllegalInstruction(_) => {
//             log::info!("illegal instruction");
//         }
//         Timer => {
//             log::info!("Timer");
//         }
//         _ => {
//             log::warn!("unsuspended trap type: {:?}", trap_type);
//         }
//     }
// }

#[polyhal::arch_entry]
/// kernel main function, entry point.
fn main(hartid: usize) {
    if hartid != 0 {
        return;
    }

    println!("[kernel] Hello, world!");
    allocator::init_allocator();
    log::debug!("Test Logger DEBUG!");
    log::info!("Test Logger INFO!");
    log::warn!("Test Logger WARN!");
    log::error!("Test Logger ERROR!");

    // Init page alloc for polyhal
    polyhal::common::init(&PageAllocImpl);

    get_mem_areas().into_iter().for_each(|(start, size)| {
        println!("init memory region {:#x} - {:#x}", start, start + size);
        // frame::add_frame_range(start, start + size);
    });

    loop {
        if let Some(c) = DebugConsole::getchar() {
            DebugConsole::putchar(c);
        }
    }

    log::info!("Run END. Shutdown successfully.");
    Instruction::shutdown();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        log::error!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        log::error!("[kernel] Panicked: {}", info.message().unwrap());
    }
    Instruction::shutdown()
}
