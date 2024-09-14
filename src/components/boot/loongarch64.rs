use loongArch64::{
    consts::{LOONGARCH_CSR_MAIL_BUF0, LOONGARCH_CSR_MAIL_BUF1},
    iocsr::iocsr_read_d,
    register::euen,
};

use crate::{
    arch::hart_id,
    clear_bss,
    components::{
        common::CPU_NUM,
        consts::VIRT_ADDR_START,
        debug_console::{display_info, println},
        instruction::Instruction,
        percpu::percpu_area_init,
    },
    PageTable, PhysAddr,
};

/// The earliest entry point for the primary CPU.
///
/// We can't use bl to jump to higher address, so we use jirl to jump to higher address.
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    core::arch::asm!("
        ori         $t0, $zero, 0x1     # CSR_DMW1_PLV0
        lu52i.d     $t0, $t0, -2048     # UC, PLV0, 0x8000 xxxx xxxx xxxx
        csrwr       $t0, 0x180          # LOONGARCH_CSR_DMWIN0
        ori         $t0, $zero, 0x11    # CSR_DMW1_MAT | CSR_DMW1_PLV0
        lu52i.d     $t0, $t0, -1792     # CA, PLV0, 0x9000 xxxx xxxx xxxx
        csrwr       $t0, 0x181          # LOONGARCH_CSR_DMWIN1

        # Goto 1 if hart is not 0
        csrrd       $t1, 0x20       # read cpu from csr
        bnez        $t1, 1f

        # Enable PG 
        li.w		$t0, 0xb0		# PLV=0, IE=0, PG=1
        csrwr		$t0, 0x0        # LOONGARCH_CSR_CRMD
        li.w		$t0, 0x00		# PLV=0, PIE=0, PWE=0
        csrwr		$t0, 0x1        # LOONGARCH_CSR_PRMD
        li.w		$t0, 0x00		# FPE=0, SXE=0, ASXE=0, BTE=0
        csrwr		$t0, 0x2        # LOONGARCH_CSR_EUEN


        la.global   $sp, {boot_stack}
        li.d        $t0, {boot_stack_size}
        add.d       $sp, $sp, $t0       # setup boot stack
        csrrd       $a0, 0x20           # cpuid
        la.global   $t0, {entry}
        jirl        $zero,$t0,0

    1:
        li.w        $s0, {MBUF0}
        iocsrrd.d   $t0, $s0
        la.global   $t1, {sec_entry}
        bne         $t0, $t1, 1b
        jirl        $zero, $t1, 0
        ",
        boot_stack_size = const crate::components::boot::STACK_SIZE,
        boot_stack = sym crate::components::boot::BOOT_STACK,
        MBUF0 = const loongArch64::consts::LOONGARCH_CSR_MAIL_BUF0,
        entry = sym rust_tmp_main,
        sec_entry = sym _start_secondary,
        options(noreturn),
    )
}

/// The earliest entry point for the primary CPU.
///
/// We can't use bl to jump to higher address, so we use jirl to jump to higher address.
/// TODO: Dynamic Stack Pointer.
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
pub(crate) unsafe extern "C" fn _start_secondary() -> ! {
    core::arch::asm!(
        "
        ori          $t0, $zero, 0x1     # CSR_DMW1_PLV0
        lu52i.d      $t0, $t0, -2048     # UC, PLV0, 0x8000 xxxx xxxx xxxx
        csrwr        $t0, 0x180          # LOONGARCH_CSR_DMWIN0
        ori          $t0, $zero, 0x11    # CSR_DMW1_MAT | CSR_DMW1_PLV0
        lu52i.d      $t0, $t0, -1792     # CA, PLV0, 0x9000 xxxx xxxx xxxx
        csrwr        $t0, 0x181          # LOONGARCH_CSR_DMWIN1

        li.w         $t0, {MBUF1}
        iocsrrd.d    $sp, $t0

        csrrd        $a0, 0x20                  # cpuid
        la.global    $t0, {entry}

        jirl $zero,$t0,0
        ",
        options(noreturn),
        MBUF1 = const loongArch64::consts::LOONGARCH_CSR_MAIL_BUF1,
        entry = sym _rust_secondary_main,
    )
}

/// Rust temporary entry point
///
/// This function will be called after assembly boot stage.
pub fn rust_tmp_main(hart_id: usize) {
    clear_bss();
    percpu_area_init(hart_id);
    #[cfg(feature = "logger")]
    crate::components::debug_console::DebugConsole::log_init();

    // Display Information.
    display_info!();
    println!(include_str!("../../banner.txt"));
    display_info!("Platform Name", "loongarch64");
    display_info!("Platform Virt Mem Offset", "{:#x}", VIRT_ADDR_START);
    display_info!();
    display_info!("Boot HART ID", "{}", hart_id);
    display_info!();

    #[cfg(feature = "trap")]
    crate::components::trap::set_trap_vector_base();
    // Initialize CPU Configuration.
    init_cpu();
    crate::components::timer::init_timer();
    #[cfg(feature = "trap")]
    crate::components::trap::tlb_init(crate::components::trap::tlb_fill as _);

    // TODO: Detect CPU Num dynamically.
    CPU_NUM.init_by(2);

    unsafe { crate::components::boot::_main_for_arch(hart_id) };

    Instruction::shutdown();
}

/// Initialize CPU Configuration.
fn init_cpu() {
    // Enable floating point
    euen::set_fpe(true);
}

/// The entry point for the second core.
pub(crate) extern "C" fn _rust_secondary_main() {
    let hart_id = hart_id();
    percpu_area_init(hart_id);

    log::info!("mailbox: {:#x}", iocsr_read_d(LOONGARCH_CSR_MAIL_BUF0));
    log::info!("mailbox: {:#x}", iocsr_read_d(LOONGARCH_CSR_MAIL_BUF1));

    #[cfg(feature = "trap")]
    crate::components::trap::set_trap_vector_base();
    // Initialize CPU Configuration.
    init_cpu();
    crate::components::timer::init_timer();
    #[cfg(feature = "trap")]
    crate::components::trap::tlb_init(crate::components::trap::tlb_fill as _);

    unsafe { crate::components::boot::_main_for_arch(hart_id) };
}

pub fn boot_page_table() -> PageTable {
    // FIXME: This should return a valid page table.
    // ref solution: create a blank page table in boot stage.
    PageTable(PhysAddr(0))
}
