use crate::percpu::{__start_percpu, get_percpu_ptr};
use core::ops::Deref;

pub struct PerCPU<T>(*mut T);

impl<T> PerCPU<T> {
    pub const fn new(raw_ptr: *mut T) -> Self {
        Self(raw_ptr)
    }
}

unsafe impl<T> Sync for PerCPU<T> {}

impl<T> PerCPU<T> {
    #[inline(always)]
    pub fn get_mut_ptr(&self) -> *mut T {
        let percpu_base = get_percpu_ptr();
        (self.0 as usize + percpu_base - __start_percpu as usize) as _
    }
    #[inline(always)]
    pub fn ref_mut(&self) -> &mut T {
        unsafe { self.get_mut_ptr().as_mut().unwrap() }
    }

    #[inline(always)]
    pub fn write(&self, value: T) {
        unsafe {
            self.get_mut_ptr().write(value);
        }
    }
}

impl<T> Deref for PerCPU<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.get_mut_ptr().as_ref().unwrap() }
    }
}
