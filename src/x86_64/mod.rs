mod apic;
mod barrier;
mod consts;
mod context;
mod gdt;
mod idt;
mod interrupt;
mod irq;
#[cfg(feature = "kcontext")]
mod kcontext;
mod multiboot;
mod page_table;
mod sigtrx;
mod time;
mod uart;

use core::cmp;

use ::multiboot::information::MemoryType;
use alloc::vec::Vec;
pub use consts::VIRT_ADDR_START;
pub use context::TrapFrame;
pub use interrupt::*;
#[cfg(feature = "kcontext")]
pub use kcontext::{context_switch, context_switch_pt, read_current_tp, KContext};
pub use multiboot::kernel_page_table;
use raw_cpuid::CpuId;
pub use uart::*;

use x86_64::{
    instructions::port::PortWriteOnly,
    registers::{
        control::{Cr4, Cr4Flags},
        xcontrol::{XCr0, XCr0Flags},
    },
};

use crate::{
    currrent_arch::multiboot::use_multiboot,
    debug::{display_info, println},
    multicore::MultiCore,
    once::LazyInit,
    percpu::set_local_thread_pointer,
    CPU_NUM, DTB_BIN, MEM_AREA,
};

#[polyhal_macro::def_percpu]
static CPU_ID: usize = 1;

pub fn shutdown() -> ! {
    unsafe { PortWriteOnly::new(0x604).write(0x2000u16) };
    loop {}
}

static MBOOT_PTR: LazyInit<usize> = LazyInit::new();

fn rust_tmp_main(magic: usize, mboot_ptr: usize) {
    crate::clear_bss();
    uart::init_early();
    idt::init();
    apic::init();
    sigtrx::init();
    // Init allocator
    set_local_thread_pointer(hart_id());
    gdt::init();
    interrupt::init_syscall();
    time::init_early();

    // enable avx extend instruction set and sse if support avx
    // TIPS: QEMU not support avx, so we can't enable avx here
    // IF you want to use avx in the qemu, you can use -cpu IvyBridge-v2 to
    // select a cpu with avx support
    CpuId::new().get_feature_info().map(|features| {
        info!("is there a avx feature: {}", features.has_avx());
        info!("is there a xsave feature: {}", features.has_xsave());
        // Add OSXSave flag to cr4 register if supported
        if features.has_xsave() {
            unsafe {
                Cr4::write(Cr4::read() | Cr4Flags::OSXSAVE);
            }
        }
        info!("cr4 has OSXSAVE feature: {:?}", Cr4::read());
        if features.has_avx() && features.has_xsave() && Cr4::read().contains(Cr4Flags::OSXSAVE) {
            unsafe {
                XCr0::write(XCr0::read() | XCr0Flags::AVX | XCr0Flags::SSE | XCr0Flags::X87);
            }
        }
    });

    // TODO: This is will be fixed with ACPI support
    CPU_NUM.init_by(1);

    info!("magic: {magic:#x}, mboot_ptr: {mboot_ptr:#x}");

    MBOOT_PTR.init_by(mboot_ptr);

    // Print PolyHAL information.
    display_info!();
    println!(include_str!("../banner.txt"));
    display_info!("Platform Arch", "x86_64");
    if let Some(features) = CpuId::new().get_feature_info() {
        display_info!(
            "Platform Hart Count",
            "{}",
            cmp::max(1, features.max_logical_processor_ids())
        );
        display_info!("Platform FPU Support", "{}", features.has_fpu());
    }
    display_info!("Platform Virt Mem Offset", "{VIRT_ADDR_START:#x}");
    // TODO: Use the dynamic uart information.
    display_info!("Platform UART Name", "Uart16550");
    display_info!("Platform UART Port", "0x3f8");
    display_info!("Platform UART IRQ", "0x4");
    if let Some(mboot) = use_multiboot(mboot_ptr as _) {
        mboot
            .command_line()
            .inspect(|cl| display_info!("Platform Command Line", "{}", cl));
        if let Some(mr) = mboot.memory_regions() {
            mr.for_each(|mm| {
                display_info!(
                    "Platform Memory Region",
                    "{:#018x} - {:#018x} {:?}",
                    mm.base_address(),
                    mm.base_address() + mm.length(),
                    mm.memory_type()
                )
            });
        }
        display_info!();
        display_info!(
            "Boot Image Highest Addr",
            "{:#018x}",
            mboot.find_highest_address()
        );
    }
    display_info!("Boot HART ID", "{:#x}", CPU_ID.read_current());
    display_info!();

    unsafe { crate::api::_main_for_arch(0) };

    shutdown()
}

pub fn arch_init() {
    DTB_BIN.init_by(Vec::new());
    if let Some(mboot) = use_multiboot(*MBOOT_PTR as _) {
        let mut mem_area = Vec::new();
        if mboot.has_memory_map() {
            mboot
                .memory_regions()
                .unwrap()
                .filter(|x| x.memory_type() == MemoryType::Available)
                .for_each(|x| {
                    let start = x.base_address() as usize | VIRT_ADDR_START;
                    let size = x.length() as usize;
                    // ArchInterface::add_memory_region(start, end);
                    mem_area.push((start, size));
                });
        }
        MEM_AREA.init_by(mem_area);
    }
}

pub fn hart_id() -> usize {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.initial_local_apic_id() as usize,
        None => 0,
    }
}

#[cfg(feature = "multicore")]
impl MultiCore {
    pub fn boot_all() {}
}
