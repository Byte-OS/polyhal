mod lazy_init;
mod mutex_no_irq;

pub use lazy_init::LazyInit;
pub use mutex_no_irq::{MutexNoIrq, MutexNoIrqGuard};
