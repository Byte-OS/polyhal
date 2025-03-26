
/// PolyHAL defined percpu reserved fields.
/// Just used in the polyHAL and context switch.
#[repr(C)]
pub(crate) struct PerCPUReserved {
    pub user_rsp: usize,
    pub kernel_rsp: usize,
    pub user_context: usize,
}

/// Get the offset of the specified percpu field.
///
/// PerCPU Arrange is that.
///
/// IN x86_64. The Reserved fields was used in manually.
/// IN other architectures, the reserved fields was defined
/// negative offset of the percpu pointer.
pub(crate) macro PerCPUReservedOffset($field: ident) {
    core::mem::offset_of!(PerCPUReserved, $field) as isize
        - core::mem::size_of::<PerCPUReserved>() as isize
}
