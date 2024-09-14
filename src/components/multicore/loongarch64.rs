use loongArch64::ipi::{csr_mail_send, send_ipi_single};

use crate::{boot::BOOT_STACK, components::multicore::MultiCore};

// TODO: Boot a core with top pointer of the stack
pub fn boot_core(hart_id: usize, sp_top: usize) {
    csr_mail_send(crate::components::boot::_start_secondary as _, hart_id, 0);
    csr_mail_send(sp_top as _, hart_id, 1);
    send_ipi_single(1, 1);
}

impl MultiCore {
    pub fn boot_all() {
        // Stack Pointer.
        let stack_ptr = unsafe { BOOT_STACK.as_ptr() as u64 + BOOT_STACK.len() as u64 };
        csr_mail_send(crate::components::boot::_start_secondary as _, 1, 0);
        csr_mail_send(stack_ptr, 1, 1);
        send_ipi_single(1, 1);
    }
}
