use fdt::Fdt;
use riscv::register::{sie, sstatus};

use crate::components::common::{CPU_ID, CPU_NUM, DTB_PTR};
use crate::components::consts::VIRT_ADDR_START;
use crate::components::debug_console::{display_info, println};
use crate::components::instruction;
use crate::components::pagetable::{PTEFlags, PTE};
use crate::multicore::CpuCore;
use crate::PageTable;

use super::PageAlignment;

/// TODO: Map the whole memory in the available memory region.
pub(crate) static mut PAGE_TABLE: PageAlignment = {
    let mut arr: [PTE; PageTable::PTE_NUM_IN_PAGE] = [PTE(0); PageTable::PTE_NUM_IN_PAGE];
    // Init Page Table
    // 0x00000000_00000000 -> 0x00000000_00000000 (256G)
    // 0xffffffc0_00000000 -> 0x00000000_00000000 (256G)
    // Const Loop, Can't use for i in 0..
    let mut i = 0;
    while i < 0x100 {
        // Base Address
        arr[i] = PTE::from_addr(i * 0x4000_0000, PTEFlags::ADVRWX);
        // Higher Half Kernel
        arr[i + 0x100] = PTE::from_addr(i * 0x4000_0000, PTEFlags::ADGVRWX);
        i += 1;
    }
    PageAlignment(arr)
};

/// Assembly Entry Function
///
/// Initialize Stack, Page Table and call rust entry.
#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() -> ! {
    core::arch::asm!(
        // Chcek boot core
        "
            beqz    a0, 2f
        ",
        // 1. Set Stack Pointer.
        // sp = bootstack + (hartid + 1) * 0x10000
        "2:
            la      sp, {boot_stack}
            li      t0, {stack_size}
            add     sp, sp, t0              // set boot stack

            li      s0, {virt_addr_start}   // add virtual address
            or      sp, sp, s0
        ",
        // 2. Open Paging Mode
        // satp = (8 << 60) | PPN(page_table)
        "
            la      t0, {page_table}
            srli    t0, t0, 12
            li      t1, 8 << 60
            or      t0, t0, t1
            csrw    satp, t0
            sfence.vma
        ",
        // 3. Call rust_main function.
        "
            la      a2, {entry}
            or      a2, a2, s0
            jalr    a2                      // call rust_main
        ",
        stack_size = const crate::components::boot::STACK_SIZE,
        boot_stack = sym crate::components::boot::BOOT_STACK,
        page_table = sym PAGE_TABLE,
        entry = sym rust_main,
        virt_addr_start = const VIRT_ADDR_START,
        options(noreturn),
    )
}

/// Assembly Entry Function
///
/// Initialize Page Information. Call rust_secondary_main entry function.
#[naked]
#[no_mangle]
pub(crate) unsafe extern "C" fn secondary_start() -> ! {
    core::arch::asm!(
        // 1. Set Stack Pointer.
        // sp = a1(given Stack Pointer.)
        "
            mv      s6, a0
            mv      sp, a1

            li      s0, {virt_addr_start}   // add virtual address
            or      sp, sp, s0
        ",
        // 2. Call Paging Mode
        // satp = (8 << 60) | PPN(page_table)
        "
            la      t0, {page_table}
            srli    t0, t0, 12
            li      t1, 8 << 60
            or      t0, t0, t1
            csrw    satp, t0
            sfence.vma
        ", 
        // 3. Call secondary_entry
        "
            la      a2, {entry}
            or      a2, a2, s0
            mv      a0, s6
            jalr    a2                      // call rust_main
        ",
        page_table = sym PAGE_TABLE,
        entry = sym rust_secondary_main,
        virt_addr_start = const VIRT_ADDR_START,
        options(noreturn)
    );
}

pub(crate) fn rust_main(hartid: usize, device_tree: usize) {
    super::clear_bss();
    #[cfg(feature = "logger")]
    crate::components::debug_console::DebugConsole::log_init();
    // Init allocator
    crate::components::percpu::set_local_thread_pointer(hartid);
    CPU_ID.write_current(hartid);
    #[cfg(feature = "trap")]
    crate::components::trap::init();

    CpuCore::init(hartid);

    // Initialize CPU Configuration.
    init_cpu();

    let fdt = unsafe { Fdt::from_ptr(device_tree as *const u8) };
    CPU_NUM.init_by(fdt.map(|fdt| fdt.cpus().count()).unwrap_or(1));

    DTB_PTR.init_by(device_tree);

    display_info!();
    println!(include_str!("../../banner.txt"));
    display_info!("Platform Name", "riscv64");
    if let Ok(fdt) = fdt {
        display_info!("Platform HART Count", "{}", fdt.cpus().count());
        fdt.memory().regions().for_each(|x| {
            display_info!(
                "Platform Memory Region",
                "{:#p} - {:#018x}",
                x.starting_address,
                x.starting_address as usize + x.size.unwrap()
            );
        });
        // TODO: Map available memory regions in the current region.
    }
    display_info!("Platform Virt Mem Offset", "{:#x}", VIRT_ADDR_START);
    display_info!();
    display_info!("Boot HART ID", "{}", hartid);
    display_info!();

    unsafe { crate::components::boot::_main_for_arch(hartid) };
    instruction::shutdown();
}

/// Secondary Main function Entry.
/// 
/// Supports MultiCore, Boot in this function.
pub(crate) extern "C" fn rust_secondary_main(hartid: usize) {
    crate::components::percpu::set_local_thread_pointer(hartid);
    CPU_ID.write_current(hartid);

    // Initialize the trap.
    #[cfg(feature = "trap")]
    crate::components::trap::init();

    // TODO: Get the hart_id and device_tree for the specified device.
    // let (hartid, _device_tree) = boards::init_device(hartid, 0);

    // Initialize CPU Configuration.
    init_cpu();

    log::info!("secondary hart {} started", hartid);
    unsafe { crate::components::boot::_main_for_arch(hartid) };
    instruction::shutdown();
}

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

/// Get Boot Page Table.
pub fn boot_page_table() -> PageTable {
    PageTable(crate::addr::PhysAddr(unsafe {
        PAGE_TABLE.0.as_ptr() as usize & !VIRT_ADDR_START
    }))
}
