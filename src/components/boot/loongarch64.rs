use loongArch64::register::euen;

use crate::{
    clear_bss, components::{common::CPU_NUM, consts::VIRT_ADDR_START, debug_console::{display_info, println}, instruction::Instruction, percpu::percpu_area_init}
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

        // csrrd       $t1, 0x20       # read cpu from csr
        // bnez        $t1, _start_secondary

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
        ",
        boot_stack_size = const crate::components::boot::STACK_SIZE,
        boot_stack = sym crate::components::boot::BOOT_STACK,
        entry = sym rust_tmp_main,
        options(noreturn),
    )
}

/// The earliest entry point for the primary CPU.
///
/// We can't use bl to jump to higher address, so we use jirl to jump to higher address.
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start_secondary() -> ! {
    core::arch::asm!(
        "
        idle    1
        b       _start_secondary
        ",
        options(noreturn),
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

    display_info!();
    println!(include_str!("../../banner.txt"));
    display_info!("Platform Name", "loongarch64");
    display_info!("Platform Virt Mem Offset", "{:#x}", VIRT_ADDR_START);
    display_info!();
    display_info!("Boot HART ID", "{}", hart_id);
    display_info!();

    #[cfg(target_arch = "trap")]
    crate::components::trap::set_trap_vector_base();
    // Enable floating point
    euen::set_fpe(true);
    crate::components::timer::init_timer();
    #[cfg(target_arch = "trap")]
    crate::components::trap::tlb_init(crate::components::trap::tlb_fill as _);

    CPU_NUM.init_by(2);

    unsafe { crate::components::boot::_main_for_arch(hart_id) };

    Instruction::shutdown();
}

/// The entry point for the second core.
pub(crate) extern "C" fn _rust_secondary_main(_hartid: usize) {}
