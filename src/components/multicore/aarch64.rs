use crate::components::multicore::MultiCore;

// TODO: Boot a core with top pointer of the stack
pub fn boot_core(_hart_id: usize, _sp_top: usize) {
    log::error!("Boot Core is not implemented yet for aarch64");
}

impl MultiCore {
    /// Boot application cores
    pub fn boot_all() {}
}
