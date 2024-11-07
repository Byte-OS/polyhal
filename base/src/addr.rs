use core::{
    ffi::CStr,
    fmt::{Debug, Display},
};

use crate::config::KERNEL_OFFSET;

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysAddr(pub(crate) usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtAddr(pub(crate) usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPage(pub(crate) usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtPage(pub(crate) usize);

impl PhysAddr {
    pub const fn to_vaddr(&self) -> VirtAddr {
        VirtAddr(self.0 | KERNEL_OFFSET)
    }
}

impl VirtAddr {
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
        let ptr = self.0 as *mut T;
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
        unsafe { CStr::from_ptr(self.get_ptr::<i8>()) }
    }

    pub const fn floor(&self, align: usize) -> Self {
        Self(self.0 / align * align)
    }

    pub const fn ceil(&self, align: usize) -> Self {
        Self((self.0 + align - 1) / align * align)
    }
}

impl PhysPage {
    pub const fn from_addr(addr: usize) -> Self {
        Self(addr >> 12)
    }

    pub const fn to_paddr(&self) -> PhysAddr {
        PhysAddr(self.0 << 12)
    }

    pub const fn to_vaddr(&self) -> VirtAddr {
        VirtAddr((self.0 << 12) | KERNEL_OFFSET)
    }
}

impl VirtPage {
    #[inline]
    pub const fn from_addr(addr: usize) -> Self {
        Self(addr >> 12)
    }

    #[inline]
    pub const fn to_addr(&self) -> usize {
        self.0 << 12
    }
}

impl From<PhysPage> for PhysAddr {
    fn from(value: PhysPage) -> Self {
        Self(value.0 << 12)
    }
}

impl From<PhysAddr> for PhysPage {
    fn from(value: PhysAddr) -> Self {
        Self(value.0 >> 12)
    }
}

impl From<VirtPage> for VirtAddr {
    fn from(value: VirtPage) -> Self {
        Self(value.to_addr())
    }
}


impl From<VirtAddr> for VirtPage {
    fn from(value: VirtAddr) -> Self {
        Self(value.0 >> 12)
    }
}

macro_rules! impl_addr {
    ($($name:ident),*) => {
        $(
            impl $name {
                pub const fn new(value: usize) -> Self {
                    Self(value)
                }

                pub const fn raw(&self) -> usize {
                    self.0
                }
            }

            impl From<usize> for $name {
                fn from(value: usize) -> Self {
                    Self(value)
                }
            }
            impl Display for $name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_fmt(format_args!("{:#x}", self.0))
                }
            }
            impl Debug for $name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_fmt(format_args!("{:#x}", self.0))
                }
            }
        )*
    };
}

impl_addr!(PhysPage, PhysAddr, VirtPage, VirtAddr);
