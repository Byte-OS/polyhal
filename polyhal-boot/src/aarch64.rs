use aarch64_cpu::{asm::barrier, registers::*};
use polyhal::{
    ctor::CtorType,
    pa,
    pagetable::{PTEFlags, PAGE_SIZE, PTE, TLB},
    PageTable,
};
use tock_registers::interfaces::{ReadWriteable, Writeable};

#[link_section = ".data.boot_page_table"]
static mut BOOT_PT: [PTE; PageTable::PTE_NUM_IN_PAGE * 2] =
    [PTE::empty(); PageTable::PTE_NUM_IN_PAGE * 2];

/// Init MMU
///
/// ## Registers Introduction
///
/// - `TCR_EL1` <https://developer.arm.com/documentation/ddi0595/2021-06/AArch64-Registers/TCR-EL1--Translation-Control-Register--EL1->
unsafe extern "C" fn init_mmu() {
    SPSel.write(SPSel::SP::ELx);
    SP_EL0.set(0);
    MAIR_EL1.set(0x44_ff_04);

    // Enable TTBR0 and TTBR1 walks, page size = 4K, vaddr size = 39 bits, paddr size = 40 bits.
    let tcr_flags0 = TCR_EL1::EPD0::EnableTTBR0Walks
        + TCR_EL1::TG0::KiB_4
        + TCR_EL1::SH0::Inner
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T0SZ.val(16);
    let tcr_flags1 = TCR_EL1::EPD1::EnableTTBR1Walks
        + TCR_EL1::TG1::KiB_4
        + TCR_EL1::SH1::Inner
        + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::T1SZ.val(16);
    TCR_EL1.write(TCR_EL1::IPS::Bits_48 + tcr_flags0 + tcr_flags1);
    barrier::isb(barrier::SY);

    // Set both TTBR0 and TTBR1
    let root_paddr = (BOOT_PT.as_ptr() as usize & 0xFFFF_F000) as _;
    TTBR0_EL1.set(root_paddr);
    TTBR1_EL1.set(root_paddr);

    // Flush the entire TLB
    TLB::flush_all();

    // Enable the MMU and turn on I-cache and D-cache
    SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    barrier::isb(barrier::SY);
}

unsafe extern "C" fn init_boot_page_table() {
    BOOT_PT[0] = PTE::new_table(pa!(BOOT_PT.as_ptr() as usize + PAGE_SIZE));
    // Level 1 Entry for Huge Page
    for i in 0..0x200 {
        BOOT_PT[0x200 + i] = PTE::new_page(
            pa!(i * 0x4000_0000),
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
        ldr     x0, =bstack_top
        mov     sp, x0

        bl      {init_boot_page_table}
        bl      {init_mmu}              // setup MMU

        mov     x0, x19                 // call rust_entry(cpu_id, dtb)
        mov     x1, x20
        ldr     x8, ={entry}
        blr     x8
        b      .",
        init_boot_page_table = sym init_boot_page_table,
        init_mmu = sym init_mmu,
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
        bl      {init_mmu}

        mov     x0, x19                 // call rust_entry_secondary(cpu_id)
        ldr     x8, ={entry}
        blr     x8
        b      .",
        init_mmu = sym init_mmu,
        entry = sym rust_secondary_main,
        options(noreturn),
    )
}

pub fn rust_tmp_main(hart_id: usize, dt: usize) {
    super::clear_bss();

    init_cpu();

    polyhal::ctor::ph_init_iter(CtorType::Platform).for_each(|x| (x.func)());

    super::call_real_main(hart_id);
}

/// Rust secondary entry for core except Boot Core.
fn rust_secondary_main(hart_id: usize) {
    // Initialize the cpu configuration.
    init_cpu();

    super::call_real_main(hart_id);
}

/// Initialize the CPU configuration.
fn init_cpu() {
    // Initialize the Timer
    // timer::init();

    // Enable Floating Point Feature.
    CPACR_EL1.write(CPACR_EL1::FPEN::TrapNothing);
    aarch64_cpu::asm::barrier::isb(aarch64_cpu::asm::barrier::SY);
}
