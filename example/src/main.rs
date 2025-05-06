#![no_std]
#![no_main]

mod allocator;
mod frame;
mod pci;
use core::panic::PanicInfo;

use frame::frame_alloc;
use polyhal::mem::{get_fdt, get_mem_areas};
use polyhal::percpu::get_local_thread_pointer;
use polyhal::{
    common::PageAlloc,
    instruction::{ebreak, shutdown},
    PhysAddr,
};
use polyhal::{percpu, println};
use polyhal_boot::define_entry;
use polyhal_trap::trap::TrapType::{self, *};
use polyhal_trap::trapframe::{TrapFrame, TrapFrameArgs};

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

/// kernel main function, entry point.
fn main(hartid: usize) {
    check_percpu(hartid);
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
            if let Some(mut compatibles) = x.compatible() {
                log::debug!("Node Compatiable: {:?}", compatibles.next());
            }
        });
    }

    // Test BreakPoint Interrupt
    ebreak();

    crate::pci::init();
    log::info!("Run END. Shutdown successfully.");
    shutdown();
}

#[percpu]
static mut TEST_PERCPU: usize = 0;

fn check_percpu(hartid: usize) {
    log::debug!(
        "hart {} percpu base: {:#x}",
        hartid,
        get_local_thread_pointer()
    );
    assert_eq!(*TEST_PERCPU, 0);
    *TEST_PERCPU.ref_mut() = hartid;
    assert_eq!(*TEST_PERCPU, hartid);
}

fn secondary(hartid: usize) {
    check_percpu(hartid);
    println!("Secondary Hart ID: {}", hartid);
    loop {}
}

define_entry!(main, secondary);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[panic] Panicked at {}:{} \n\t{}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        println!("[panic] Panicked: {}", info.message());
    }
    shutdown()
}
