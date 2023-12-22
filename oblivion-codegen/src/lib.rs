use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn async_route(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let func_name = &input.sig.ident;
    let func_args = &input.sig.inputs;
    let func_block = input.block;

    let expanded = quote! {


        pub fn #func_name(#func_args) -> BoxFuture<'static, BaseResponse>
        {
            async move {
                #func_block
            }.boxed()
        }
    };

    TokenStream::from(expanded)
}
