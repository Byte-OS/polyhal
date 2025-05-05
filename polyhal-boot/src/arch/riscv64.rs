use core::{arch::naked_asm, ptr::addr_of_mut};
use polyhal::{
    consts::VIRT_ADDR_START,
    ctor::{ph_init_iter, CtorType},
    mem::{init_dtb_once, parse_system_info},
    pagetable::{PTEFlags, PTE, TLB},
    percpu::set_local_thread_pointer,
    PageTable, PhysAddr,
};
use riscv::register::{satp, sie, sstatus};

#[link_section = ".data.boot_page_table"]
static mut BOOT_PT: [PTE; PageTable::PTE_NUM_IN_PAGE] = [PTE::empty(); PageTable::PTE_NUM_IN_PAGE];

unsafe extern "C" fn init_boot_page_table() {
    let boot_pt = addr_of_mut!(BOOT_PT).as_mut().unwrap();
    let flags = PTEFlags::A | PTEFlags::D | PTEFlags::R | PTEFlags::V | PTEFlags::W | PTEFlags::X;

    for i in 0..0x100 {
        let target_addr = i * 0x4000_0000;
        // 0x00000000_00000000 -> 0x00000000_00000000 (256G, 1G PerPage)
        boot_pt[i] = PTE::from_addr(target_addr, flags);
        // 0xffffffc0_00000000 -> 0x00000000_00000000 (256G, 1G PerPage)
        boot_pt[i + 0x100] = PTE::from_addr(target_addr, flags | PTEFlags::G);
    }
}

unsafe extern "C" fn init_mmu() {
    let ptr = (&raw mut BOOT_PT) as usize;
    satp::set(satp::Mode::Sv39, 0, ptr >> 12);
    TLB::flush_all();
}

/// Assembly Entry Function
///
/// Initialize Stack, Page Table and call rust entry.
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        // 1. Set Stack Pointer.
        // sp = bootstack + (hartid + 1) * 0x10000
        "   mv      s0, a0
            mv      s1, a1
            la      sp, bstack_top
            li      t0, {virt_addr_start}
            not     t0, t0
            and     sp, sp, t0

            call    {init_boot_page_table}
            call    {init_mmu}

            li      t0, {virt_addr_start}   // add virtual address
            or      sp, sp, t0

            la      a2, {entry}
            or      a2, a2, t0
            mv      a0, s0
            mv      a1, s1
            jalr    a2                      // call rust_main
        ",
        init_boot_page_table = sym init_boot_page_table,
        init_mmu = sym init_mmu,
        entry = sym rust_main,
        virt_addr_start = const VIRT_ADDR_START,
    )
}

/// Assembly Entry Function
///
/// Initialize Page Information. Call rust_secondary_main entry function.
#[naked]
#[no_mangle]
unsafe extern "C" fn _secondary_start() -> ! {
    naked_asm!(
        // 1. Set Stack Pointer.
        // sp = a1(given Stack Pointer.)
        "
            mv      s0, a0
            mv      sp, a1

            call    {init_mmu}

            li      t0, {virt_addr_start}   // add virtual address
            or      sp, sp, t0

            la      a2, {entry}
            or      a2, a2, t0
            mv      a0, s0
            jalr    a2                      // call rust_main
        ",
        init_mmu = sym init_mmu,
        entry = sym rust_secondary_main,
        virt_addr_start = const VIRT_ADDR_START,
    );
}

unsafe extern "C" fn rust_main(hartid: usize, dt: PhysAddr) {
    super::clear_bss();
    let _ = init_dtb_once(dt);
    // Initialize CPU Configuration.
    set_local_thread_pointer(hartid);
    init_cpu();
    ph_init_iter(CtorType::Cpu).for_each(|x| (x.func)());

    parse_system_info();

    // Init contructor functions
    ph_init_iter(CtorType::Platform).for_each(|x| (x.func)());
    ph_init_iter(CtorType::HALDriver).for_each(|x| (x.func)());

    super::call_real_main(hartid);
}

/// Secondary Main function Entry.
///
/// Supports MultiCore, Boot in this function.
pub(crate) extern "C" fn rust_secondary_main(hartid: usize) {
    // Initialize CPU Configuration.
    set_local_thread_pointer(hartid);

    init_cpu();
    ph_init_iter(CtorType::Cpu).for_each(|x| (x.func)());

    super::call_real_main(hartid);
}

/// Init CPU Configuration
///
/// - `status` <https://riscv.github.io/riscv-isa-manual/snapshot/privileged/#sstatus>
/// - `sum`    <https://riscv.github.io/riscv-isa-manual/snapshot/privileged/#sum>
/// - `sie`    <https://riscv.github.io/riscv-isa-manual/snapshot/privileged/#_supervisor_interrupt_sip_and_sie_registers>
#[inline]
fn init_cpu() {
    unsafe {
        // Enable SUM for access user memory directly.
        // TODO: Call set_sum() for riscv version up than 1.0, Close when below 1.0
        sstatus::set_sum();
        // Open float point support.
        sstatus::set_fs(sstatus::FS::Dirty);
        sie::set_sext();
        sie::set_ssoft();
    }
}
