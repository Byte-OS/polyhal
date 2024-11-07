use aarch64_cpu::{asm, asm::barrier, registers::*};

// use page_table_entry::aarch64::{MemAttr, A64PTE};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use crate::{
    common::parse_dtb_info, components::{
        common::DTB_PTR,
        consts::VIRT_ADDR_START,
        debug_console::{display_info, println},
        instruction,
        pagetable::{PTEFlags, TLB},
        percpu::percpu_area_init,
        timer,
    }, multicore::CpuCore, pagetable::PTE, PageTable, PhysPage
};

use super::PageAlignment;

#[link_section = ".data"]
static mut BOOT_PT_L1: PageAlignment = PageAlignment([PTE(0); PageTable::PTE_NUM_IN_PAGE]);

unsafe fn switch_to_el1() {
    SPSel.write(SPSel::SP::ELx);
    SP_EL0.set(0);
    let current_el = CurrentEL.read(CurrentEL::EL);
    if current_el >= 2 {
        if current_el == 3 {
            // Set EL2 to 64bit and enable the HVC instruction.
            SCR_EL3.write(
                SCR_EL3::NS::NonSecure + SCR_EL3::HCE::HvcEnabled + SCR_EL3::RW::NextELIsAarch64,
            );
            // Set the return address and exception level.
            SPSR_EL3.write(
                SPSR_EL3::M::EL1h
                    + SPSR_EL3::D::Masked
                    + SPSR_EL3::A::Masked
                    + SPSR_EL3::I::Masked
                    + SPSR_EL3::F::Masked,
            );
            ELR_EL3.set(LR.get());
        }
        // Disable EL1 timer traps and the timer offset.
        CNTHCTL_EL2.modify(CNTHCTL_EL2::EL1PCEN::SET + CNTHCTL_EL2::EL1PCTEN::SET);
        CNTVOFF_EL2.set(0);
        // Set EL1 to 64bit.
        HCR_EL2.write(HCR_EL2::RW::EL1IsAarch64);
        // Set the return address and exception level.
        SPSR_EL2.write(
            SPSR_EL2::M::EL1h
                + SPSR_EL2::D::Masked
                + SPSR_EL2::A::Masked
                + SPSR_EL2::I::Masked
                + SPSR_EL2::F::Masked,
        );
        core::arch::asm!(
            "
            mov     x8, sp
            msr     sp_el1, x8"
        );
        ELR_EL2.set(LR.get());
        asm::eret();
    }
}

unsafe fn init_mmu() {
    MAIR_EL1.set(0x44_ff_04);

    // Enable TTBR0 and TTBR1 walks, page size = 4K, vaddr size = 39 bits, paddr size = 40 bits.
    let tcr_flags0 = TCR_EL1::EPD0::EnableTTBR0Walks
        + TCR_EL1::TG0::KiB_4
        + TCR_EL1::SH0::Inner
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T0SZ.val(25);
    let tcr_flags1 = TCR_EL1::EPD1::EnableTTBR1Walks
        + TCR_EL1::TG1::KiB_4
        + TCR_EL1::SH1::Inner
        + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T1SZ.val(25);
    TCR_EL1.write(TCR_EL1::IPS::Bits_48 + tcr_flags0 + tcr_flags1);
    barrier::isb(barrier::SY);

    // Set both TTBR0 and TTBR1
    // let root_paddr = PhysAddr::from(BOOT_PT_L0.as_ptr() as usize).addr() as _;
    let root_paddr = (BOOT_PT_L1.0.as_ptr() as usize & 0xFFFF_FFFF_F000) as _;
    TTBR0_EL1.set(root_paddr);
    TTBR1_EL1.set(root_paddr);

    // Flush the entire TLB
    TLB::flush_all();

    // Enable the MMU and turn on I-cache and D-cache
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    barrier::isb(barrier::SY);
}

unsafe fn init_boot_page_table() {
    // Level 1 Entry for Huge Page
    for i in 0..0x200 {
        BOOT_PT_L1.0[i] = PTE::new_page(
            PhysPage::from_addr(i * 0x4000_0000),
            PTEFlags::VALID | PTEFlags::AF | PTEFlags::ATTR_INDX | PTEFlags::NG,
        );
    }
}
/// The earliest entry point for the primary CPU.
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    // PC = 0x8_0000
    // X0 = dtb
    core::arch::asm!("
        mrs     x19, mpidr_el1
        and     x19, x19, #0xffffff     // get current CPU id
        mov     x20, x0                 // save DTB pointer
        cbz     x19, 1f
        b       .
    ",
    "1:
        adrp    x8, {boot_stack}        // setup boot stack
        add     x8, x8, {boot_stack_size}
        mov     sp, x8

        bl      {switch_to_el1}         // switch to EL1
        bl      {init_boot_page_table}
        bl      {init_mmu}              // setup MMU

        mov     x8, {phys_virt_offset}  // set SP to the high address
        add     sp, sp, x8

        mov     x0, x19                 // call rust_entry(cpu_id, dtb)
        mov     x1, x20
        ldr     x8, ={entry}
        blr     x8
        b      .",
        switch_to_el1 = sym switch_to_el1,
        init_boot_page_table = sym init_boot_page_table,
        init_mmu = sym init_mmu,
        boot_stack = sym crate::components::boot::BOOT_STACK,
        boot_stack_size = const crate::components::boot::STACK_SIZE,
        phys_virt_offset = const crate::components::consts::VIRT_ADDR_START,
        entry = sym rust_tmp_main,
        options(noreturn),
    )
}

/// The secondary core boot entry point.
#[naked]
#[no_mangle]
pub(crate) unsafe extern "C" fn _secondary_boot() -> ! {
    core::arch::asm!("
        mrs     x19, mpidr_el1
        and     x19, x19, #0xffffff     // get current CPU id

        mov     sp, x0
        bl      {switch_to_el1}
        bl      {init_mmu}

        mov     x8, {phys_virt_offset}  // set SP to the high address
        add     sp, sp, x8

        mov     x0, x19                 // call rust_entry_secondary(cpu_id)
        ldr     x8, ={entry}
        blr     x8
        b      .",
        switch_to_el1 = sym switch_to_el1,
        init_mmu = sym init_mmu,
        phys_virt_offset = const crate::components::consts::VIRT_ADDR_START,
        entry = sym rust_secondary_main,
        options(noreturn),
    )
}

pub fn rust_tmp_main(hart_id: usize, device_tree: usize) {
    super::clear_bss();
    percpu_area_init(hart_id);
    CpuCore::init(hart_id);

    // Init DebugConsole early.
    crate::components::debug_console::init_early();
    #[cfg(feature = "logger")]
    crate::components::debug_console::DebugConsole::log_init();
    #[cfg(feature = "trap")]
    crate::components::trap::init();
    // Init GIC interrupt controller.
    crate::components::irq::init();

    init_cpu();

    DTB_PTR.init_by(device_tree | VIRT_ADDR_START);

    // Display Polyhal and Platform Information
    display_info!();
    println!(include_str!("../../banner.txt"));
    display_info!("Platform Name", "aarch64");
    display_info!("Platform Virt Mem Offset", "{:#x}", VIRT_ADDR_START);
    parse_dtb_info();
    display_info!();
    display_info!("Boot HART ID", "{}", hart_id);
    display_info!();

    // Enter to kernel entry point(`main` function).
    unsafe { crate::components::boot::_main_for_arch(hart_id) };

    instruction::shutdown();
}

/// Rust secondary entry for core except Boot Core.
fn rust_secondary_main(hart_id: usize) {
    // Initialize the cpu configuration.
    init_cpu();

    unsafe { crate::components::boot::_main_for_arch(hart_id) }
}

/// Initialize the CPU configuration.
fn init_cpu() {
    // Initialize the Timer
    timer::init();

    // Enable Floating Point Feature.
    CPACR_EL1.write(CPACR_EL1::FPEN::TrapNothing);
    aarch64_cpu::asm::barrier::isb(aarch64_cpu::asm::barrier::SY);
}

pub fn boot_page_table() -> PageTable {
    PageTable(crate::addr::PhysAddr(unsafe {
        BOOT_PT_L1.0.as_ptr() as usize & !VIRT_ADDR_START
    }))
}
