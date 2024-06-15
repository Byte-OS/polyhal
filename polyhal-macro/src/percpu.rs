use std::mem::size_of;

use quote::quote;
use syn::{Ident, Type};

/// Reserved for default usage, Just for x86_64 now.
pub const PERCPU_RESERVED: usize = 4 * size_of::<usize>();

pub fn gen_current_ptr(_symbol: &Ident, ty: &Type) -> proc_macro2::TokenStream {
    // TODO: Move this consts into polyhal crate.
    quote! {
        let base: usize;
        // The first index is SELF_PTR in the x86_64 format
        #[cfg(target_arch = "x86_64")]
        ::core::arch::asm!(
            "mov {0}, gs:0",
            out(reg) base,
        );
        #[cfg(target_arch = "aarch64")]
        ::core::arch::asm!("mrs {}, TPIDR_EL1", out(reg) base);
        #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
        ::core::arch::asm!("mv {}, gp", out(reg) base);
        #[cfg(target_arch = "loongarch64")]
        ::core::arch::asm!("move {}, $r21", out(reg) base);
        (base + self.offset()) as *const #ty
    }
}
