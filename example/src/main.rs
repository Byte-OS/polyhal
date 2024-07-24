#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod allocator;
mod frame;
mod logging;

use core::panic::PanicInfo;

use frame::frame_alloc;
use polyhal::addr::PhysPage;
use polyhal::{get_mem_areas, PageAlloc, TrapFrame, TrapType};
use polyhal::{shutdown, TrapType::*};

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
#[polyhal::arch_interrupt]
fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType) {
    // println!("trap_type @ {:x?} {:#x?}", trap_type, ctx);
    match trap_type {
        Breakpoint => return,
        UserEnvCall => {
            // jump to next instruction anyway
            ctx.syscall_ok();
            log::info!("Handle a syscall");
        }
        StorePageFault(_paddr) | LoadPageFault(_paddr) | InstructionPageFault(_paddr) => {
            log::info!("page fault");
        }
        IllegalInstruction(_) => {
            log::info!("illegal instruction");
        }
        Time => {
            log::info!("Timer");
        }
        _ => {
            log::warn!("unsuspended trap type: {:?}", trap_type);
        }
    }
}

#[polyhal::arch_entry]
/// kernel main function, entry point.
fn main(hartid: usize) {
    if hartid != 0 {
        return;
    }

    println!("[kernel] Hello, world!");
    allocator::init_allocator();
    logging::init(Some("trace"));
    println!("init logging");

    // Init page alloc for polyhal
    polyhal::init(&PageAllocImpl);

    get_mem_areas().into_iter().for_each(|(start, size)| {
        println!("init memory region {:#x} - {:#x}", start, start + size);
        frame::add_frame_range(start, start + size);
    });
    log::info!("Run END. Shutdown successfully.");
    shutdown();
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
    shutdown()
}
