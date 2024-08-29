//! Consts components
//!
//!

super::define_arch_mods!();

/// Virtual Address Offset.
pub const VIRT_ADDR_START: usize = GenericConfig::VIRT_ADDR;

/// Generic Configuration Implementation.
struct GenericConfig;

/// Configuration Trait, Bound for configs
pub(self) trait ConfigTrait {
    const VIRT_ADDR: usize;
}
