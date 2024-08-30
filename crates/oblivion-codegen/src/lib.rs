//! # Oblivion Macros
//!
//! Oblivion use Rust macros to implement the business function processing and routing system,
//! which allows you to use synchronous or asynchronous functions.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// ## Oblivion Macro for Route Handler
#[proc_macro_attribute]
pub fn async_route(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let func_name = &input.sig.ident;
    let func_args = &input.sig.inputs;
    let func_return = &input.sig.output;
    let func_block = &input.block;

    let expanded = quote! {
        pub fn #func_name(#func_args) #func_return {
            Box::pin(async move {
                #func_block
            })
        }
    };

    TokenStream::from(expanded)
}
