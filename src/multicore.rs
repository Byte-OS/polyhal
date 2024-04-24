/// This is a leader for the multicore operation
///
/// You can use this function to use the multicore operation
///
/// Boot other calls after the multicore
/// If you use this function call, you should call it after arch::init(..);
/// This function will allocate the stack and map it for itself.
/// Multicore::boot_all();
///
/// Here will have more functionality about multicore in the future.
///
pub struct MultiCore;
