use crate::{
    boot::secondary_start,
    common::{frame_alloc, CPU_ID, CPU_NUM},
    consts::{MULTI_CORE_AREA, MULTI_CORE_AREA_SIZE, VIRT_ADDR_START},
    multicore::MultiCore,
    MappingFlags, MappingSize, PageTable,
};

// TODO: Boot a core with top pointer of the stack
pub fn boot_core(cpu: usize, sp_top: usize) {
    if cpu == CPU_ID.read_current() {
        return;
    };

    // PERCPU DATA ADDRESS RANGE END
    let aux_core_func = (secondary_start as usize) & (!VIRT_ADDR_START);

    log::info!("secondary addr: {:#x}", secondary_start as usize);
    let ret = sbi_rt::hart_start(cpu, aux_core_func, sp_top);
    if ret.is_ok() {
        log::info!("hart {} Startting successfully", cpu);
    } else {
        log::warn!("hart {} Startting failed", cpu)
    }
}

/// Implement the function for multicore
impl MultiCore {
    /// Boot all application cores.
    pub fn boot_all() {
        use crate::addr::VirtPage;
        use crate::components::boot::secondary_start;

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

            log::info!("secondary addr: {:#x}", secondary_start as usize);
            let ret = sbi_rt::hart_start(cpu, aux_core_func, cpu_addr_end);
            if ret.is_ok() {
                log::info!("hart {} Startting successfully", cpu);
            } else {
                log::warn!("hart {} Startting failed", cpu)
            }
        });
    }
}
