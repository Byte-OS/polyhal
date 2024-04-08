#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod allocator;
mod frame;
mod logging;

use core::panic::PanicInfo;

use arch::addr::PhysPage;
use arch::{api::ArchInterface, TrapFrame, TrapType};
use arch::{shutdown, TrapType::*};
use crate_interface::impl_interface;
use fdt::node::FdtNode;

pub struct ArchInterfaceImpl;

#[impl_interface]
impl ArchInterface for ArchInterfaceImpl {
    /// Init allocator
    fn init_allocator() {
        allocator::init_allocator();
    }
    /// kernel interrupt
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
    /// init log
    fn init_logging() {
        logging::init(Some("trace"));
        println!("init logging");
    }
    /// add a memory region
    fn add_memory_region(start: usize, end: usize) {
        println!("init memory region {:#x} - {:#x}", start, end);
        frame::add_frame_range(start, end);
    }
    /// kernel main function, entry point.
    fn main(hartid: usize) {
        if hartid != 0 {
            return;
        }
        println!("[kernel] Hello, world!");
        arch::init_interrupt();
        panic!("end of rust_main!");
    }
    /// Alloc a persistent memory page.
    fn frame_alloc_persist() -> PhysPage {
        frame::frame_alloc()
    }
    /// Unalloc a persistent memory page
    fn frame_unalloc(ppn: PhysPage) {
        frame::frame_dealloc(ppn)
    }
    /// Preprare drivers.
    fn prepare_drivers() {
        println!("prepare drivers");
    }
    /// Try to add device through FdtNode
    fn try_to_add_device(_fdt_node: &FdtNode) {}
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
