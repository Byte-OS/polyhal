/// Reserved for default usage.
/// This is related to the [polyhal_macro::percpu::PERCPU_RESERVED]
/// Just for x86_64 now.
/// 0: SELF_PTR
/// 1: VALID_PTR
/// 2: USER_RSP
/// 3: KERNEL_RSP
/// 4: USER_CONTEXT
#[repr(C)]
pub struct PerCPUReserved {
    pub self_ptr: usize,
    pub valid_ptr: usize,
    pub user_rsp: usize,
    pub kernel_rsp: usize,
    pub user_context: usize,
}

impl PerCPUReserved {
    pub(crate) fn mut_from_ptr(ptr: *mut Self) -> &'static mut Self {
        unsafe { ptr.as_mut().unwrap() }
    }
}
