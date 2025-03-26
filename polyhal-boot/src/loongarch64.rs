use loongArch64::register::euen;
use polyhal::arch::hart_id;

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

        # Enable PG 
        li.w        $t0, 0xb0       # PLV=0, IE=0, PG=1
        csrwr       $t0, 0x0        # LOONGARCH_CSR_CRMD
        li.w        $t0, 0x00       # PLV=0, PIE=0, PWE=0
        csrwr       $t0, 0x1        # LOONGARCH_CSR_PRMD
        li.w        $t0, 0x00       # FPE=0, SXE=0, ASXE=0, BTE=0
        csrwr       $t0, 0x2        # LOONGARCH_CSR_EUEN

        la.global   $sp, bstack_top
        csrrd       $a0, 0x20           # cpuid
        la.global   $t0, {entry}
        jirl        $zero,$t0,0
        ",
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
    super::clear_bss();
    // Initialize CPU Configuration.
    init_cpu();

    super::call_real_main(hart_id);
}

/// Initialize CPU Configuration.
fn init_cpu() {
    // Enable floating point
    euen::set_fpe(true);

    // Initialzie Timer
    // timer::init_timer();
}

/// The entry point for the second core.
pub(crate) extern "C" fn _rust_secondary_main() {
    // Initialize CPU Configuration.
    init_cpu();

    super::call_real_main(hart_id());
}
