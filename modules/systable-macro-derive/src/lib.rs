#![cfg_attr(not(test), no_std)]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn syscall_func(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);
    let expanded = quote! {
        #input
    };
    let stream = TokenStream::from(expanded);
    stream
}
