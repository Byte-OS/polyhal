#![no_std]
#![no_main]

mod allocator;
mod frame;
mod logging;
mod pci;
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicU32, Ordering};

use frame::frame_alloc;
use polyhal::mem::{get_fdt, get_mem_areas};
use polyhal::{
    common::PageAlloc,
    instruction::{ebreak, shutdown},
    trap::TrapType::{self, *},
    trapframe::{TrapFrame, TrapFrameArgs},
    PhysAddr,
};
use polyhal_boot::define_entry;

pub struct PageAllocImpl;

impl PageAlloc for PageAllocImpl {
    fn alloc(&self) -> PhysAddr {
        frame_alloc(1)
    }

    fn dealloc(&self, paddr: PhysAddr) {
        frame::frame_dealloc(paddr)
    }
}

/// kernel interrupt
#[polyhal::arch_interrupt]
fn kernel_interrupt(ctx: &mut TrapFrame, trap_type: TrapType) {
    // println!("trap_type @ {:x?} {:#x?}", trap_type, ctx);
    match trap_type {
        Breakpoint => {
            log::info!("BreakPoint @ {:#x}", ctx[TrapFrameArgs::SEPC]);
        }
        SysCall => {
            // jump to next instruction anyway
            ctx.syscall_ok();
            log::info!("Handle a syscall");
        }
        StorePageFault(paddr) | LoadPageFault(paddr) | InstructionPageFault(paddr) => {
            log::info!("page fault: {:#x}", paddr);
        }
        IllegalInstruction(_) => {
            log::info!("illegal instruction");
        }
        Timer => {
            log::info!("Timer");
        }
        _ => {
            log::warn!("unsuspended trap type: {:?}", trap_type);
        }
    }
}

static CORE_SET: AtomicU32 = AtomicU32::new(0);

/// kernel main function, entry point.
fn main(hartid: usize) {
    if hartid != 0 {
        log::info!("Hello Other Hart: {}", hartid);
        let _ = CORE_SET.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
            Some(x | (1 << hartid))
        });
        loop {}
    }

    println!("[kernel] Hello, world!");
    allocator::init_allocator();
    log::debug!("Test Logger DEBUG!");
    log::info!("Test Logger INFO!");
    log::warn!("Test Logger WARN!");
    log::error!("Test Logger ERROR!");

    // Init page alloc for polyhal
    polyhal::common::init(&PageAllocImpl);

    get_mem_areas().for_each(|(start, size)| {
        println!("init memory region {:#x} - {:#x}", start, start + size);
        frame::add_frame_range(*start, start + size);
    });

    if let Ok(fdt) = get_fdt() {
        fdt.all_nodes().for_each(|x| {
            if let Some(compatibles) = x.compatible() {
                log::debug!("Node Compatiable: {:?}", compatibles.first());
            }
        });
    }

    // // Test BreakPoint Interrupt
    // // ebreak();

    // crate::pci::init();

    log::info!("Run END. Shutdown successfully.");
    shutdown();
}

define_entry!(main);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        log::error!(
            "[kernel] Panicked at {}:{} \n\t{}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        log::error!("[kernel] Panicked: {}", info.message());
    }
    shutdown()
}
