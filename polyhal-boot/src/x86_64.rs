use core::{arch::global_asm, mem, slice};
use multiboot::{
    header::MULTIBOOT_HEADER_MAGIC,
    information::{MemoryManagement, Multiboot, PAddr},
};
use polyhal::{arch::hart_id, consts::VIRT_ADDR_START, utils::bit};
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
const MULTIBOOT_HEADER_FLAGS: usize = bit!(1) | bit!(16) | bit!(2);

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

#[cfg(feature = "graphic")]
const GRAPHIC_MODE: usize = 0;
#[cfg(not(feature = "graphic"))]
const GRAPHIC_MODE: usize = 1;

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

    cr0 = const CR0,
    cr4 = const CR4,
    efer_msr = const (x86::msr::IA32_EFER),
    graphic_mode = const GRAPHIC_MODE,
    efer = const EFER,
);

fn rust_tmp_main(magic: usize, mboot_ptr: usize) {
    super::clear_bss();
    // enable avx extend instruction set and sse if support avx
    // TIPS: QEMU not support avx, so we can't enable avx here
    // IF you want to use avx in the qemu, you can use -cpu IvyBridge-v2 to
    // select a cpu with avx support
    CpuId::new().get_feature_info().map(|features| {
        // Add OSXSave flag to cr4 register if supported
        if features.has_xsave() {
            unsafe {
                Cr4::write(Cr4::read() | Cr4Flags::OSXSAVE);
            }
        }
        if features.has_avx() && features.has_xsave() && Cr4::read().contains(Cr4Flags::OSXSAVE) {
            unsafe {
                XCr0::write(XCr0::read() | XCr0Flags::AVX | XCr0Flags::SSE | XCr0Flags::X87);
            }
        }
    });
    if let Some(mboot) = use_multiboot(mboot_ptr as _) {
        if let Some(mr) = mboot.memory_regions() {
            mr.for_each(|mm| {});
        }
    }

    // Check Multiboot Magic Number.
    assert_eq!(magic, multiboot::information::SIGNATURE_EAX as usize);

    super::call_real_main(hart_id());
}
