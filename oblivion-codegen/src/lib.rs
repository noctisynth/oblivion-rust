//! # Oblivion 宏
//! Oblivion 使用 Rust 宏实现业务函数的处理及路由系统，它允许你使用同步或异步的函数。

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// ## Oblivion 业务函数宏
///
/// 这是一个示例：
///
/// ```rust
/// use oblivion::oblivion_codegen::async_route;
/// use oblivion::utils::parser::OblivionRequest;
/// use oblivion::models::render::{BaseResponse, Response};
///
/// #[async_route]
/// fn welcome(mut req: OblivionRequest) -> Response {
///     Ok(BaseResponse::TextResponse(
///         format!("欢迎进入信息绝对安全区, 来自[{}]的朋友", req.get_ip()),
///        200,
///     ))
/// }
/// ```
#[proc_macro_attribute]
pub fn async_route(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let func_name = &input.sig.ident;
    let func_args = &input.sig.inputs;
    let func_return = &input.sig.output;
    let func_block = &input.block;

    let expanded = quote! {
        pub fn #func_name(#func_args) #func_return
        {
            Box::pin(async move {
                #func_block
            })
        }
    };

    TokenStream::from(expanded)
}
