use quote::quote;
use syn::{Ident, Type};

pub fn gen_offset(symbol: &Ident) -> proc_macro2::TokenStream {
    quote! {
        let value: usize;
        unsafe {
            cfg_match! {
                cfg(target_arch = "x86_64") => {
                    ::core::arch::asm!(
                        "movabs {0}, offset {VAR}",
                        out(reg) value,
                        VAR = sym #symbol,
                    );
                }
                cfg(target_arch = "aarch64") => {
                    ::core::arch::asm!(
                        "movz {0}, #:abs_g0_nc:{VAR}",
                        out(reg) value,
                        VAR = sym #symbol,
                    );
                }
                cfg(any(target_arch = "riscv32", target_arch = "riscv64")) => {
                    ::core::arch::asm!(
                        "lui {0}, %hi({VAR})",
                        "addi {0}, {0}, %lo({VAR})",
                        out(reg) value,
                        VAR = sym #symbol,
                    );
                }
                cfg(target_arch = "loongarch64") => {
                    ::core::arch::asm!(
                        "la.abs {0}, {VAR}",
                        out(reg) value,
                        VAR = sym #symbol,
                    );
                }
            }
        }
        value
    }
}

pub fn gen_current_ptr(symbol: &Ident, ty: &Type) -> proc_macro2::TokenStream {
    quote! {
        let base: usize;
        #[cfg(target_arch = "x86_64")]
        {
            // `__PERCPU_SELF_PTR` stores GS_BASE, which is defined in crate `percpu`.
            ::core::arch::asm!(
                "mov {0}, gs:[offset __PERCPU_SELF_PTR]",
                "add {0}, offset {VAR}",
                out(reg) base,
                VAR = sym #symbol,
            );
            base as *const #ty
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            #[cfg(target_arch = "aarch64")]
            ::core::arch::asm!("mrs {}, TPIDR_EL1", out(reg) base);
            #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
            ::core::arch::asm!("mv {}, gp", out(reg) base);
            #[cfg(target_arch = "loongarch64")]
            ::core::arch::asm!("move {}, $r21", out(reg) base);
            (base + self.offset()) as *const #ty
        }
    }
}
