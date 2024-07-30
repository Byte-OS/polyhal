super::define_arch_mods!();

/// This is a barrier function.
///
/// This struct has two functions.
/// [`Barrier::complete_sync`]: ensures the correct sequencing of instructions
/// [`Barrier::ordering_sync`]: ensures the visibility and consistency of memory operations
pub struct Barrier;
