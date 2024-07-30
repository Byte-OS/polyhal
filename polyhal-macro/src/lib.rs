mod percpu;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Error, ItemFn, ItemStatic};

#[proc_macro_attribute]
pub fn arch_entry(_input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let annotated_item = parse_macro_input!(annotated_item as ItemFn);
    TokenStream::from(quote! {
        #[export_name = "_main_for_arch"]
        #annotated_item
    })
}

#[proc_macro_attribute]
pub fn arch_interrupt(_input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    let annotated_item = parse_macro_input!(annotated_item as ItemFn);
    TokenStream::from(quote! {
        #[export_name = "_interrupt_for_arch"]
        #annotated_item
    })
}

/// Defines a per-CPU data structure.
///
/// It should be used on a `static` variable.
///
/// See the [crate-level documentation](../percpu/index.html) for more details.
#[proc_macro_attribute]
pub fn def_percpu(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return compiler_error(Error::new(
            Span::call_site(),
            "expect an empty attribute: `#[def_percpu]`",
        ));
    }

    let ast = syn::parse_macro_input!(item as ItemStatic);

    let attrs = &ast.attrs;
    let vis = &ast.vis;
    let name = &ast.ident;
    let ty = &ast.ty;
    let init_expr = &ast.expr;

    let inner_symbol_name = &format_ident!("__PERCPU_{}", name);
    let struct_name = &format_ident!("{}_WRAPPER", name);

    let ty_str = quote!(#ty).to_string();
    let is_primitive_int = ["bool", "u8", "u16", "u32", "u64", "usize"].contains(&ty_str.as_str());

    // Do not generate `fn read_current()`, `fn write_current()`, etc for non primitive types.
    let read_write_methods = if is_primitive_int {
        quote! {
            /// Returns the value of the per-CPU data on the current CPU.
            ///
            /// # Safety
            ///
            /// Caller must ensure that preemption is disabled on the current CPU.
            #[inline]
            pub unsafe fn read_current_raw(&self) -> #ty {
                unsafe { *self.current_ptr() }
            }

            /// Set the value of the per-CPU data on the current CPU.
            ///
            /// # Safety
            ///
            /// Caller must ensure that preemption is disabled on the current CPU.
            #[inline]
            pub unsafe fn write_current_raw(&self, val: #ty) {
                unsafe { *self.current_ref_mut_raw() = val };
            }

            /// Returns the value of the per-CPU data on the current CPU. Preemption will
            /// be disabled during the call.
            pub fn read_current(&self) -> #ty {
                unsafe { self.read_current_raw() }
            }

            /// Set the value of the per-CPU data on the current CPU. Preemption will
            /// be disabled during the call.
            pub fn write_current(&self, val: #ty) {
                unsafe { self.write_current_raw(val) }
            }
        }
    } else {
        quote! {}
    };

    let current_ptr = percpu::gen_current_ptr(inner_symbol_name, ty);
    quote! {
        #[cfg_attr(not(target_os = "macos"), link_section = "percpu")] // unimplemented on macos
        #[used(linker)]
        #(#attrs)*
        static mut #inner_symbol_name: #ty = #init_expr;

        #[doc = concat!("Wrapper struct for the per-CPU data [`", stringify!(#name), "`]")]
        #[allow(non_camel_case_types)]
        #vis struct #struct_name {}

        #(#attrs)*
        #vis static #name: #struct_name = #struct_name {};

        impl #struct_name {
            /// Returns the offset relative to the per-CPU data area base on the current CPU.
            #[inline]
            pub fn offset(&self) -> usize {
                extern "Rust" {
                    fn __start_percpu();
                }
                unsafe {
                    &#inner_symbol_name as *const _ as usize - __start_percpu as usize
                }
            }

            /// Returns the raw pointer of this per-CPU data on the current CPU.
            ///
            /// # Safety
            ///
            /// Caller must ensure that preemption is disabled on the current CPU.
            #[inline]
            pub unsafe fn current_ptr(&self) -> *const #ty {
                #current_ptr
            }

            /// Returns the reference of the per-CPU data on the current CPU.
            ///
            /// # Safety
            ///
            /// Caller must ensure that preemption is disabled on the current CPU.
            #[inline]
            pub unsafe fn current_ref_raw(&self) -> &#ty {
                &*self.current_ptr()
            }

            /// Returns the mutable reference of the per-CPU data on the current CPU.
            ///
            /// # Safety
            ///
            /// Caller must ensure that preemption is disabled on the current CPU.
            #[inline]
            #[allow(clippy::mut_from_ref)]
            pub unsafe fn current_ref_mut_raw(&self) -> &mut #ty {
                &mut *(self.current_ptr() as *mut #ty)
            }

            /// Manipulate the per-CPU data on the current CPU in the given closure.
            /// Preemption will be disabled during the call.
            pub fn with_current<F, T>(&self, f: F) -> T
            where
                F: FnOnce(&mut #ty) -> T,
            {
                f(unsafe { self.current_ref_mut_raw() })
            }

            #read_write_methods
        }
    }
    .into()
}

#[proc_macro]
pub fn define_arch_mods(_input: TokenStream) -> TokenStream {
    quote! {
        #[cfg(target_arch = "riscv64")]
        mod riscv64;
        #[cfg(target_arch = "riscv64")]
        #[allow(unused_imports)]
        pub use riscv64::*;
        #[cfg(target_arch = "aarch64")]
        mod aarch64;
        #[cfg(target_arch = "aarch64")]
        #[allow(unused_imports)]
        pub use aarch64::*;
        #[cfg(target_arch = "x86_64")]
        mod x86_64;
        #[cfg(target_arch = "x86_64")]
        #[allow(unused_imports)]
        pub use x86_64::*;
        #[cfg(target_arch = "loongarch64")]
        mod loongarch64;
        #[cfg(target_arch = "loongarch64")]
        #[allow(unused_imports)]
        pub use loongarch64::*;
    }
    .into()
}

fn compiler_error(err: Error) -> TokenStream {
    err.to_compile_error().into()
}
