use core::{
    ffi::CStr,
    mem::size_of,
};

use crate::{PhysAddr, PhysPage, VirtAddr};

use crate::{PageTable, VIRT_ADDR_START};

impl PhysAddr {
    #[inline]
    pub fn get_ptr<T>(&self) -> *const T {
        (self.0 | VIRT_ADDR_START) as *const T
    }

    #[inline]
    pub const fn get_mut_ptr<T>(&self) -> *mut T {
        (self.0 | VIRT_ADDR_START) as *mut T
    }

    #[inline]
    pub fn slice_with_len<T>(&self, len: usize) -> &'static [T] {
        unsafe { core::slice::from_raw_parts(self.get_ptr(), len) }
    }

    #[inline]
    pub fn slice_mut_with_len<T>(&self, len: usize) -> &'static mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.get_mut_ptr(), len) }
    }

    #[inline]
    pub fn get_cstr(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.get_ptr::<i8>()) }
    }
}

impl VirtAddr {
    #[inline]
    pub fn floor(&self) -> Self {
        Self(self.0 / PageTable::PAGE_SIZE * PageTable::PAGE_SIZE)
    }

    #[inline]
    pub fn ceil(&self) -> Self {
        Self((self.0 + PageTable::PAGE_SIZE - 1) / PageTable::PAGE_SIZE * PageTable::PAGE_SIZE)
    }
}


impl PhysPage {
    #[inline]
    pub const fn get_buffer(&self) -> &'static mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                (self.0 << 12 | VIRT_ADDR_START) as *mut u8,
                PageTable::PAGE_SIZE,
            )
        }
    }

    #[inline]
    pub fn copy_value_from_another(&self, ppn: PhysPage) {
        self.get_buffer().copy_from_slice(&ppn.get_buffer());
        #[cfg(c906)]
        unsafe {
            asm!(".long 0x0010000b"); // dcache.all
            asm!(".long 0x01b0000b"); // sync.is
        }
        #[cfg(board = "2k1000")]
        unsafe {
            core::arch::asm!("dbar 0;ibar 0;")
        }
    }

    #[inline]
    pub fn drop_clear(&self) {
        // self.get_buffer().fill(0);
        unsafe {
            core::slice::from_raw_parts_mut(
                (self.0 << 12 | VIRT_ADDR_START) as *mut usize,
                PageTable::PAGE_SIZE / size_of::<usize>(),
            )
            .fill(0);
        }
        #[cfg(board = "2k1000")]
        unsafe {
            core::arch::asm!("dbar 0;ibar 0;")
        }
        #[cfg(c906)]
        unsafe {
            asm!(".long 0x0010000b"); // dcache.all
            asm!(".long 0x01b0000b"); // sync.is
        }
    }
}