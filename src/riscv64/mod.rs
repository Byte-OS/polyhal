mod barrier;
mod boards;
mod consts;
mod context;
mod entry;
mod interrupt;
mod irq;
#[cfg(feature = "kcontext")]
mod kcontext;
mod page_table;
mod sbi;
mod timer;

use core::slice;

use alloc::vec::Vec;
pub use consts::*;
pub use context::{KernelToken, TrapFrame};
pub use entry::{kernel_page_table, switch_to_kernel_page_table};
use fdt::Fdt;
pub use interrupt::{run_user_task, run_user_task_forever};
use sbi::*;

pub use sbi::shutdown;

#[cfg(feature = "kcontext")]
pub use kcontext::{context_switch, context_switch_pt, read_current_tp, KContext};

use riscv::register::{sie, sstatus};

use crate::{
    api::frame_alloc,
    debug::{display_info, println},
    multicore::MultiCore,
    utils::LazyInit,
    CPU_NUM, DTB_BIN, MEM_AREA,
};

#[polyhal_macro::def_percpu]
static CPU_ID: usize = 0;

static DTB_PTR: LazyInit<usize> = LazyInit::new();

pub(crate) fn rust_main(hartid: usize, device_tree: usize) {
    crate::clear_bss();
    // Init allocator
    crate::percpu::set_local_thread_pointer(hartid);
    println!("CPU_ID offset: {:#x}", CPU_ID.offset());
    println!("init success, CPU_ID: {}", CPU_ID.read_current());
    CPU_ID.write_current(hartid);
    // println!("NEWCPU_ID offset: {}", NEW_CPU_ID.offset());
    interrupt::init_interrupt();

    let (_hartid, device_tree) = boards::init_device(hartid, device_tree | VIRT_ADDR_START);

    println!("CPU_ID offset: {:#x}", CPU_ID.offset());

    // 开启 SUM
    unsafe {
        // 开启浮点运算
        sstatus::set_fs(sstatus::FS::Dirty);
        sie::set_sext();
        sie::set_ssoft();
    }

    CPU_NUM.init_by(match unsafe { Fdt::from_ptr(device_tree as *const u8) } {
        Ok(fdt) => fdt.cpus().count(),
        Err(_) => 1,
    });

    DTB_PTR.init_by(device_tree);

    display_info!();
    println!(include_str!("../banner.txt"));
    display_info!("Platform Name", "riscv64");
    if let Ok(fdt) = unsafe { Fdt::from_ptr(device_tree as *const u8) } {
        display_info!("Platform HART Count", "{}", fdt.cpus().count());
        fdt.memory().regions().for_each(|x| {
            display_info!(
                "Platform Memory Region",
                "{:#p} - {:#018x}",
                x.starting_address,
                x.starting_address as usize + x.size.unwrap()
            );
        });
    }
    display_info!("Platform Virt Mem Offset", "{:#x}", VIRT_ADDR_START);
    display_info!();
    display_info!("Boot HART ID", "{}", hartid);
    display_info!();

    unsafe { crate::api::_main_for_arch(hartid) };
    shutdown();
}

pub(crate) extern "C" fn rust_secondary_main(hartid: usize) {
    crate::percpu::set_local_thread_pointer(hartid);
    CPU_ID.write_current(hartid);

    interrupt::init_interrupt();

    let (hartid, _device_tree) = boards::init_device(hartid, 0);

    unsafe {
        // 开启浮点运算
        sstatus::set_fs(sstatus::FS::Dirty);
        sie::set_sext();
        sie::set_ssoft();
    }

    info!("secondary hart {} started", hartid);
    unsafe { crate::api::_main_for_arch(hartid) };
    shutdown();
}

#[inline]
pub fn wfi() {
    unsafe {
        riscv::register::sstatus::clear_sie();
        riscv::asm::wfi();
        riscv::register::sstatus::set_sie();
    }
}

pub fn hart_id() -> usize {
    CPU_ID.read_current()
}

pub fn arch_init() {
    let mut buffer = Vec::new();
    if let Ok(fdt) = unsafe { Fdt::from_ptr(*DTB_PTR as *const u8) } {
        unsafe {
            buffer.extend_from_slice(slice::from_raw_parts(
                *DTB_PTR as *const u8,
                fdt.total_size(),
            ));
        }
    }
    DTB_BIN.init_by(buffer);
    let mut mem_area = Vec::new();
    if let Ok(fdt) = Fdt::new(&DTB_BIN) {
        info!("There has {} CPU(s)", fdt.cpus().count());
        fdt.memory().regions().for_each(|x| {
            info!(
                "memory region {:#X} - {:#X}",
                x.starting_address as usize,
                x.starting_address as usize + x.size.unwrap()
            );
            mem_area.push((
                x.starting_address as usize | VIRT_ADDR_START,
                x.size.unwrap_or(0),
            ));
        });
    } else {
        mem_area.push((0x8000_0000 | VIRT_ADDR_START, 0x1000_0000));
    }
    MEM_AREA.init_by(mem_area);
}

#[cfg(feature = "multicore")]
/// Implement the function for multicore
impl MultiCore {
    /// Boot all application cores.
    pub fn boot_all() {
        use self::entry::secondary_start;
        use crate::{
            addr::VirtPage,
            pagetable::{MappingFlags, MappingSize, PageTable},
        };

        let page_table = PageTable::current();

        (0..*CPU_NUM).into_iter().for_each(|cpu| {
            if cpu == CPU_ID.read_current() {
                return;
            };

            // PERCPU DATA ADDRESS RANGE END
            let cpu_addr_end = MULTI_CORE_AREA + (cpu + 1) * MULTI_CORE_AREA_SIZE;
            let aux_core_func = (secondary_start as usize) & (!VIRT_ADDR_START);

            // Ready to build multi core area.
            // default stack size is 512K
            for i in 0..128 {
                page_table.map_kernel(
                    VirtPage::from_addr(cpu_addr_end - i * PageTable::PAGE_SIZE - 1),
                    frame_alloc(),
                    MappingFlags::RWX | MappingFlags::G,
                    MappingSize::Page4KB,
                )
            }

            info!("secondary addr: {:#x}", secondary_start as usize);
            let ret = sbi_rt::hart_start(cpu, aux_core_func, cpu_addr_end);
            if ret.is_ok() {
                info!("hart {} Startting successfully", cpu);
            } else {
                warn!("hart {} Startting failed", cpu)
            }
        });
    }
}

#[repr(C)]
pub(crate) struct PerCPUReserved {
    pub user_rsp: usize,
    pub kernel_rsp: usize,
    pub user_context: usize,
}

pub macro PerCPUReservedOffset($field: ident) {
    core::mem::offset_of!(PerCPUReserved, $field) as isize
        - core::mem::size_of::<PerCPUReserved>() as isize
}
