//! # Oblivion Macros
//!
//! Oblivion use Rust macros to implement the business function processing and routing system,
//! which allows you to use synchronous or asynchronous functions.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

enum ReturnType {
    ServerResponse,
    String,
    Json,
    Result(Box<Self>),
}

fn extract_result_return_type(segment: &syn::PathSegment) -> Option<ReturnType> {
    assert!(segment.ident == "Result");
    match &segment.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            let ok_type = args.args[0].clone();

            if let syn::GenericArgument::Type(syn::Type::Path(type_path)) = ok_type {
                assert!(type_path.path.segments.len() == 1);
                match type_path.path.segments[0].ident.to_string().as_str() {
                    "ServerResponse" => {
                        Some(ReturnType::Result(Box::new(ReturnType::ServerResponse)))
                    }
                    "String" => Some(ReturnType::Result(Box::new(ReturnType::String))),
                    "Value" => Some(ReturnType::Result(Box::new(ReturnType::Json))),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

/// ## Oblivion Macro for Route Handler
#[proc_macro_attribute]
pub fn async_route(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let func_name = &input.sig.ident;
    let func_args = &input.sig.inputs;
    let func_return = match &input.sig.output {
        syn::ReturnType::Default => {
            return TokenStream::from(
                quote! { compile_error!("Handler function must have a return type"); },
            )
        }
        syn::ReturnType::Type(_, ty) => ty,
    };

    let return_type = match &input.sig.output {
        syn::ReturnType::Default => {
            return TokenStream::from(
                quote! { compile_error!("Handler function must have a return type"); },
            )
        }
        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            syn::Type::Path(type_path) => {
                if type_path.path.segments.len() == 1 {
                    match type_path.path.segments[0].ident.to_string().as_str() {
                        "ServerResponse" => ReturnType::ServerResponse,
                        "String" => ReturnType::String,
                        "Value" => ReturnType::Json,
                        "Result" => {
                            let extracted_type =
                                extract_result_return_type(&type_path.path.segments[0]);
                            if extracted_type.is_none() {
                                return TokenStream::from(
                                    quote! { compile_error!("Unsupported [Result] return type")},
                                );
                            }
                            extracted_type.unwrap()
                        }
                        _ => {
                            return TokenStream::from(
                                quote! { compile_error!("Unsupported path like return type"); },
                            )
                        }
                    }
                } else {
                    return TokenStream::from(
                        quote! { compile_error!("Unsupported complex path like return type"); },
                    );
                }
            }
            _ => return TokenStream::from(quote! { compile_error!("Unsupported return type"); }),
        },
    };
    let input_block = &input.block;
    let func_block = match return_type {
        ReturnType::ServerResponse => quote! {
            Box::pin(async move {
                #input_block
            })
        },
        ReturnType::String | ReturnType::Json => quote! {
            Box::pin(async move {
                let result = async move {
                    #input_block
                }.await;
                Ok(result.into())
            })
        },
        ReturnType::Result(return_type) => match *return_type {
            ReturnType::String | ReturnType::Json => quote! {
                Box::pin(async move {
                    let result: #func_return = async move {
                        #input_block
                    }.await;
                    Ok(result?.into())
                })
            },
            ReturnType::ServerResponse => {
                return TokenStream::from(
                    quote! { compile_error!("Unsupported [ServerResponse] in [Result] return type")},
                )
            }
            ReturnType::Result(_) => {
                return TokenStream::from(
                    quote! { compile_error!("Unsupported complex [Result] return type")},
                )
            }
        },
    };

    let expanded = quote! {
        use oblivion::prelude::*;

        pub fn #func_name(#func_args) -> ServerResponse {
            #func_block
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn internal_handler(_input: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let func_name = &input.sig.ident;
    let func_args = &input.sig.inputs;
    let func_block = &input.block;
    let func_return = &input.sig.output;

    let expanded = quote! {
        pub fn #func_name(#func_args) #func_return {
            Box::pin(async move { #func_block })
        }
    };

    TokenStream::from(expanded)
}
