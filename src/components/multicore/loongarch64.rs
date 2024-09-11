use loongArch64::ipi::{csr_mail_send, send_ipi_single};

use crate::components::multicore::MultiCore;

impl MultiCore {
    pub fn boot_all() {
        csr_mail_send(crate::components::boot::_start_secondary as _, 1, 0);
        send_ipi_single(1, 1);
        loop {}
    }
}
