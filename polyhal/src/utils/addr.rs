use core::{
    ffi::{c_char, CStr},
    fmt::{Debug, Display},
    ops::Add,
};

use crate::{arch::consts::VIRT_ADDR_START, PageTable};

#[macro_export]
macro_rules! pa {
    ($e:expr) => {
        $crate::PhysAddr::new(($e) as usize)
    };
}

#[macro_export]
macro_rules! va {
    ($e:expr) => {
        $crate::VirtAddr::new(($e) as usize)
    };
}

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(usize);

impl PhysAddr {
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    pub const fn raw(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn get_ptr<T>(&self) -> *const T {
        (self.0 | VIRT_ADDR_START) as *const T
    }

    #[inline]
    pub const fn get_mut_ptr<T>(&self) -> *mut T {
        (self.0 | VIRT_ADDR_START) as *mut T
    }

    #[inline]
    pub fn write_volatile<T>(&self, v: T) {
        unsafe { self.get_mut_ptr::<T>().write_volatile(v) }
    }

    #[inline]
    pub fn slice_with_len<T>(&self, len: usize) -> &'static [T] {
        unsafe { core::slice::from_raw_parts(self.get_ptr(), len) }
    }

    #[inline]
    pub fn slice_mut_with_len<T: Sized>(&self, len: usize) -> &'static mut [T] {
        unsafe { core::slice::from_raw_parts_mut(self.get_mut_ptr(), len) }
    }

    #[inline]
    pub fn clear_len(&self, len: usize) {
        self.slice_mut_with_len::<u8>(len).fill(0);
    }

    #[inline]
    pub fn get_cstr(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.get_ptr::<c_char>()) }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(usize);

impl From<usize> for VirtAddr {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<VirtAddr> for usize {
    fn from(value: VirtAddr) -> Self {
        value.0
    }
}

impl VirtAddr {
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }
    pub const fn raw(&self) -> usize {
        self.0
    }

    #[inline]
    pub fn get_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    #[inline]
    pub fn get_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }

    #[inline]
    pub fn get_ref<T>(&self) -> &'static T {
        unsafe { &*(self.0 as *const T) }
    }

    #[inline]
    pub fn get_mut_ref<T>(&self) -> &'static mut T {
        unsafe { &mut *(self.0 as *mut T) }
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
    pub fn slice_until<T>(&self, is_valid: fn(T) -> bool) -> &'static mut [T] {
        let ptr = self.raw() as *mut T;
        unsafe {
            let mut len = 0;
            if !ptr.is_null() {
                loop {
                    if !is_valid(ptr.add(len).read()) {
                        break;
                    }
                    len += 1;
                }
            }
            core::slice::from_raw_parts_mut(ptr, len)
        }
    }

    #[inline]
    pub fn get_cstr(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.get_ptr::<c_char>()) }
    }

    #[inline]
    pub fn floor(&self) -> Self {
        Self(self.0 / PageTable::PAGE_SIZE * PageTable::PAGE_SIZE)
    }

    #[inline]
    pub fn ceil(&self) -> Self {
        Self(self.0.div_ceil(PageTable::PAGE_SIZE) * PageTable::PAGE_SIZE)
    }
}

impl Add<usize> for PhysAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        PhysAddr::new(self.0 + rhs)
    }
}

impl Add<usize> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::new(self.0 + rhs)
    }
}

impl Display for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Display for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Debug for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}

impl Debug for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{:#x}", self.0))
    }
}
