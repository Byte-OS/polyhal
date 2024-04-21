use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

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
