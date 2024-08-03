use core::arch::global_asm;
use core::{mem, slice};
use multiboot::information::{MemoryManagement, Multiboot, PAddr};
use raw_cpuid::CpuId;
use x86_64::registers::control::{Cr0Flags, Cr4, Cr4Flags};
use x86_64::registers::model_specific::EferFlags;
use x86_64::registers::xcontrol::{XCr0, XCr0Flags};

use crate::components::arch::{get_com_port, hart_id, MBOOT_PTR};
use crate::components::common::{CPU_ID, CPU_NUM};
use crate::components::consts::VIRT_ADDR_START;
use crate::components::debug_console::{display_info, println};
use crate::components::instruction::Instruction;
use crate::components::pagetable::PageTable;
use crate::components::percpu::set_local_thread_pointer;
use crate::utils::bit;

/// Flags set in the 'flags' member of the multiboot header.
///
/// (bits 1, 16: memory information, address fields in header)
/// bits 2 graphic information
const MULTIBOOT_HEADER_FLAGS: usize = bit!(1) | bit!(16) | bit!(2);

/// The magic field should contain this.
const MULTIBOOT_HEADER_MAGIC: usize = 0x1BADB002;

/// CR0 Registers introduction: https://wiki.osdev.org/CPU_Registers_x86-64#CR0
const CR0: u64 = Cr0Flags::PROTECTED_MODE_ENABLE.bits()
    | Cr0Flags::MONITOR_COPROCESSOR.bits()
    | Cr0Flags::NUMERIC_ERROR.bits()
    | Cr0Flags::WRITE_PROTECT.bits()
    | Cr0Flags::PAGING.bits();

/// CR4 registers introduction: https://wiki.osdev.org/CPU_Registers_x86-64#CR4
/// Physical Address Extension
const CR4: u64 = Cr4Flags::PHYSICAL_ADDRESS_EXTENSION.bits()
    // Page Global Enable
    | Cr4Flags::PAGE_GLOBAL.bits()
    // OS support for fxsave and fxrstor instructions
    | Cr4Flags::OSFXSR.bits()
    // Add Support for 2M Huge Page Support.
    | Cr4Flags::PAGE_SIZE_EXTENSION.bits()
    // XSAVE And Processor Extended States Enable
    // This bit should open if the processor was supported.
    // | Cr4Flags::OSXSAVE.bits()
    // OS Support for unmasked simd floating point exceptions
    | Cr4Flags::OSXMMEXCPT_ENABLE.bits();

/// EFER registers introduction: https://wiki.osdev.org/CPU_Registers_x86-64#IA32_EFER
const EFER: u64 = EferFlags::LONG_MODE_ENABLE.bits();
// TODO: enable if it supports NO_EXECUTE_ENABLE
// | EferFlags::NO_EXECUTE_ENABLE.bits()

static mut MEM: Mem = Mem;

struct Mem;

impl MemoryManagement for Mem {
    unsafe fn paddr_to_slice(&self, addr: PAddr, size: usize) -> Option<&'static [u8]> {
        let ptr = mem::transmute(addr | VIRT_ADDR_START as u64);
        Some(slice::from_raw_parts(ptr, size))
    }

    // If you only want to read fields, you can simply return `None`.
    unsafe fn allocate(&mut self, _length: usize) -> Option<(PAddr, &mut [u8])> {
        None
    }

    unsafe fn deallocate(&mut self, addr: PAddr) {
        if addr != 0 {
            unimplemented!()
        }
    }
}

/// mboot_ptr is the initial pointer to the multiboot structure
/// provided in %ebx on start-up.
pub fn use_multiboot(mboot_ptr: PAddr) -> Option<Multiboot<'static, 'static>> {
    unsafe { Multiboot::from_ptr(mboot_ptr, &mut MEM) }
}

global_asm!(
    include_str!("x86_64/multiboot.S"),
    mb_hdr_magic = const MULTIBOOT_HEADER_MAGIC,
    mb_hdr_flags = const MULTIBOOT_HEADER_FLAGS,
    entry = sym rust_tmp_main,

    offset = const VIRT_ADDR_START,
    boot_stack_size = const crate::components::boot::STACK_SIZE,
    boot_stack = sym crate::components::boot::BOOT_STACK,

    cr0 = const CR0,
    cr4 = const CR4,
    efer_msr = const x86::msr::IA32_EFER,
    efer = const EFER,
);

pub fn boot_page_table() -> PageTable {
    extern "C" {
        fn _boot_page_table();
    }
    PageTable(crate::addr::PhysAddr(
        _boot_page_table as usize - VIRT_ADDR_START,
    ))
}

fn rust_tmp_main(magic: usize, mboot_ptr: usize) {
    crate::clear_bss();
    #[cfg(feature = "graphic")]
    if let Some(mboot) = use_multiboot(mboot_ptr as _)  {
        if let Some(ft) = mboot.framebuffer_table() {
            crate::components::debug_console::init_early(ft.addr as _, ft.width as _, ft.height as _, ft.pitch as _);
        }
    }
    #[cfg(not(feature = "graphic"))]
    crate::components::debug_console::init_early();
    #[cfg(feature = "logger")]
    crate::components::debug_console::DebugConsole::log_init();
    crate::components::arch::idt::init();
    crate::components::arch::apic::init();
    // Init allocator
    set_local_thread_pointer(hart_id());
    crate::components::arch::gdt::init();
    crate::components::trap::init_syscall();
    crate::components::timer::init_early();

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

    // Set the multiboot pointer.
    MBOOT_PTR.init_by(mboot_ptr);

    // Print PolyHAL information.
    display_info!();
    println!(include_str!("../../banner.txt"));
    display_info!("Platform Arch", "x86_64");
    if let Some(features) = CpuId::new().get_feature_info() {
        display_info!(
            "Platform Hart Count",
            "{}",
            core::cmp::max(1, features.max_logical_processor_ids())
        );
        display_info!("Platform FPU Support", "{}", features.has_fpu());
    }
    display_info!("Platform Virt Mem Offset", "{VIRT_ADDR_START:#x}");
    // TODO: Use the dynamic uart information.
    #[cfg(not(feature = "vga_text"))]
    {
        display_info!("Platform UART Name", "Uart16550");
        display_info!("Platform UART IRQ", "0x4");
    }
    #[cfg(feature = "vga_text")]
    {
        display_info!("Platform Console", "VGA Text Mode");
    }
    // TODO: Display Uart Ports and IRQs
    (1..5)
        .map(get_com_port)
        .filter(Option::is_some)
        .map(Option::unwrap)
        .for_each(|port| {
            display_info!("Platform UART Port", "{port:#X}");
        });
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
        if let Some(ft) = mboot.framebuffer_table() {
            display_info!("Platform VBE Addr", "{:#x}", ft.addr);
            display_info!("Platform VBE Width", "{}", ft.width);
            display_info!("Platform VBE Pitch", "{}", ft.pitch);
            display_info!("Platform VBE Height", "{}", ft.height);
            display_info!("Platform VBE BPB", "{}", ft.bpp);
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

    unsafe { crate::components::boot::_main_for_arch(0) };

    Instruction::shutdown()
}
