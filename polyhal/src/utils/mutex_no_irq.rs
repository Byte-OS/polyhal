use core::ops::{Deref, DerefMut};
use spin::{Mutex, MutexGuard};

use crate::components::irq::IRQ;

pub struct MutexNoIrq<T: ?Sized> {
    lock: Mutex<T>,
}

/// Irq Status Struct.
/// This structure contains the status of the current IRQ
/// And it will restore irq status after dropping.
struct IrqStatus {
    irq_enabled: bool,
}

/// Restore the IRQ status when dropping
impl Drop for IrqStatus {
    fn drop(&mut self) {
        if self.irq_enabled {
            IRQ::int_enable();
        }
    }
}

/// Implement Sync for MutexNoIrq
unsafe impl<T: ?Sized + Send> Sync for MutexNoIrq<T> {}
/// Implement Send for MutexNoIrq
unsafe impl<T: ?Sized + Send> Send for MutexNoIrq<T> {}

impl<T> MutexNoIrq<T> {
    pub const fn new(data: T) -> MutexNoIrq<T> {
        MutexNoIrq {
            lock: Mutex::new(data),
        }
    }

    #[inline(always)]
    pub fn into_inner(self) -> T {
        self.lock.into_inner()
    }
}

impl<T: ?Sized> MutexNoIrq<T> {
    #[inline]
    pub fn lock(&self) -> MutexNoIrqGuard<T> {
        loop {
            if let Some(guard) = self.try_lock() {
                return guard;
            }
        }
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.lock.is_locked()
    }

    /// Force unlock the mutex.
    ///
    /// # Safety
    ///
    /// Ensures that the mutex is not locked by any thread.
    pub unsafe fn force_unlock(&self) {
        self.lock.force_unlock()
    }

    #[inline]
    pub fn try_lock(&self) -> Option<MutexNoIrqGuard<T>> {
        if self.lock.is_locked() {
            return None;
        }
        let _irq_status = IrqStatus {
            irq_enabled: IRQ::int_enabled(),
        };
        IRQ::int_disable();
        self.lock
            .try_lock()
            .map(|guard| MutexNoIrqGuard { guard, _irq_status })
    }
}

/// The Mutex Guard.
pub struct MutexNoIrqGuard<'a, T: ?Sized + 'a> {
    guard: MutexGuard<'a, T>,
    _irq_status: IrqStatus,
}

impl<'a, T: ?Sized> Deref for MutexNoIrqGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &(self.guard)
    }
}

impl<'a, T: ?Sized> DerefMut for MutexNoIrqGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut (self.guard)
    }
}
