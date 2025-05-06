use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, spanned::Spanned, Error, Ident, ItemFn, ItemStatic, StaticMutability, Token,
};

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

#[proc_macro_attribute]
pub fn percpu(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return compiler_error(Error::new(
            Span::call_site(),
            "expect an empty attribute: `#[def_percpu]`",
        ));
    }

    let found_crate = crate_name("polyhal").expect("polyhal is present in `Cargo.toml`");
    let mut ast = syn::parse_macro_input!(item as ItemStatic);
    ast.mutability = StaticMutability::Mut(Token![mut](ast.span()));

    let raw_name = ast.ident.clone();
    ast.ident = format_ident!("__PERCPU_{}", raw_name);
    let ast_name = &ast.ident;
    let vis = &ast.vis;

    let raw_ty = ast.ty.clone();

    let ty_path = match found_crate {
        FoundCrate::Itself => quote!(crate::utils::percpu::PerCPU),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!( #ident::utils::percpu::PerCPU )
        }
    };

    quote! {
        #[unsafe(link_section = "percpu")]
        #ast

        #vis static #raw_name: #ty_path<#raw_ty> = #ty_path::new(&raw mut #ast_name);
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
