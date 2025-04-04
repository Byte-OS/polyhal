use core::{arch::global_asm, ptr::addr_of_mut, slice};
use multiboot::{
    header::MULTIBOOT_HEADER_MAGIC,
    information::{MemoryManagement, MemoryType, Multiboot, PAddr},
};
use polyhal::{
    bits,
    common::get_cpu_num,
    consts::VIRT_ADDR_START,
    ctor::{ph_init_iter, CtorType},
    display_info, hart_id,
    mem::{add_memory_region, parse_system_info},
    percpu::set_local_thread_pointer,
};
use raw_cpuid::CpuId;
use x86_64::registers::{
    control::{Cr0Flags, Cr4, Cr4Flags},
    model_specific::EferFlags,
    xcontrol::{XCr0, XCr0Flags},
};

/// Flags set in the 'flags' member of the multiboot header.
///
/// (bits 1, 16: memory information, address fields in header)
/// bits 2 graphic information
const MULTIBOOT_HEADER_FLAGS: usize = bits!(1, 16, 2);

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
        Some(slice::from_raw_parts(pa!(addr).get_mut_ptr(), size))
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
    // unsafe { Multiboot::from_ptr(mboot_ptr, &mut MEM) }
    unsafe { Multiboot::from_ptr(mboot_ptr, addr_of_mut!(MEM).as_mut().unwrap()) }
}

#[cfg(feature = "graphic")]
const GRAPHIC: usize = 0;
#[cfg(not(feature = "graphic"))]
const GRAPHIC: usize = 1;

global_asm!(
    include_str!("x86_64/multiboot.S"),
    mb_hdr_magic = const MULTIBOOT_HEADER_MAGIC,
    mb_hdr_flags = const MULTIBOOT_HEADER_FLAGS,
    entry = sym rust_tmp_main,
    entry_secondary = sym _rust_secondary_main,

    kernel_offset = const VIRT_ADDR_START,
    graphic = const GRAPHIC,

    cr0 = const CR0,
    cr4 = const CR4,
    efer_msr = const (x86::msr::IA32_EFER),
    efer = const EFER,
);

core::arch::global_asm!(
    include_str!("x86_64/ap_start.S"),
    start_page_paddr = const 0x6000,
);

fn rust_tmp_main(magic: usize, mboot_ptr: usize) {
    super::clear_bss();

    ph_init_iter(CtorType::Primary).for_each(|x| (x.func)());
    // Check Multiboot Magic Number.
    assert_eq!(magic, multiboot::information::SIGNATURE_EAX as usize);

    let mboot = use_multiboot(mboot_ptr as _);
    mboot.as_ref().inspect(|mboot| {
        if let Some(mr) = mboot.memory_regions() {
            mr.for_each(|mm| unsafe {
                let mm_end = mm.base_address() + mm.length();
                if mm.memory_type() != MemoryType::Available || mm_end < 0x100000 {
                    return;
                }
                add_memory_region(mm.base_address() as _, mm_end as _);
            });
        }

        #[cfg(feature = "graphic")]
        if let Some(fb) = mboot.framebuffer_table() {
            polyhal::debug_console::init_fb(
                fb.addr as _,
                fb.width as _,
                fb.height as _,
                fb.pitch as _,
            );
        }
    });

    set_local_thread_pointer(hart_id());
    polyhal::acpi::init();

    parse_system_info();
    mboot.as_ref().inspect(|mboot| {
        if let Some(mr) = mboot.memory_regions() {
            mr.for_each(|mm| {
                let mm_end = mm.base_address() + mm.length();
                display_info!(
                    "Platform Memory Region",
                    "{:#018x} - {:#018x}  {:?}",
                    mm.base_address(),
                    mm_end,
                    mm.memory_type()
                );
            });
        }
        display_info!(
            "Platform Boot Args",
            "{}",
            mboot.command_line().unwrap_or("")
        );
        display_info!("Platform HART Count", "{}", get_cpu_num())
    });
    ph_init_iter(CtorType::Cpu).for_each(|x| (x.func)());
    init_cpu();

    // Init contructor functions
    ph_init_iter(CtorType::Platform).for_each(|x| (x.func)());
    ph_init_iter(CtorType::HALDriver).for_each(|x| (x.func)());

    super::call_real_main(hart_id());
}

fn _rust_secondary_main() {
    set_local_thread_pointer(hart_id());

    ph_init_iter(CtorType::Cpu).for_each(|x| (x.func)());

    init_cpu();

    super::call_real_main(hart_id());
}

fn init_cpu() {
    // enable avx extend instruction set and sse if support avx
    // TIPS: QEMU not support avx, so we can't enable avx here
    // IF you want to use avx in the qemu, you can use -cpu IvyBridge-v2 to
    // select a cpu with avx support
    CpuId::new().get_feature_info().inspect(|features| unsafe {
        // Add OSXSave flag to cr4 register if supported
        if features.has_xsave() {
            Cr4::update(|x| x.insert(Cr4Flags::OSXSAVE));
        }
        // XSAVE And Processor Extended States Enable
        // This bit should open if the processor was supported.
        if features.has_avx() && features.has_xsave() && features.has_sse() {
            XCr0::write(XCr0::read() | XCr0Flags::AVX | XCr0Flags::SSE | XCr0Flags::X87);
        }
    });
}
