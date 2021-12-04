use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, parse_str, AttributeArgs, FnArg, ItemFn, Lit, Meta, NestedMeta,
    Path, ReturnType, Stmt, Type,
};

#[proc_macro_attribute]
pub fn bindgen(attributes: TokenStream, item: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attributes as AttributeArgs);
    let function = parse_macro_input!(item as ItemFn);

    if function
        .sig
        .inputs
        .iter()
        .any(|input| matches!(input, FnArg::Receiver(_)))
    {
        return quote! { compile_error!("receiver not allowed") }.into();
    } else if !function.sig.generics.params.is_empty() {
        return quote! { compile_error!("generic function not allowed") }.into();
    } else if function.sig.asyncness.is_none() {
        return generate_sync_function(&function);
    }

    let crate_path: Path = match parse_str(
        &attributes
            .iter()
            .find_map(|attribute| match attribute {
                NestedMeta::Meta(Meta::NameValue(name_value)) => {
                    if name_value.path.is_ident("serde") {
                        if let Lit::Str(string) = &name_value.lit {
                            Some(string.value())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .unwrap_or_else(|| "ffi".into()),
    ) {
        Ok(path) => path,
        Err(error) => {
            let message = error.to_string();
            return quote! { compile_error!(#message) }.into();
        }
    };

    let output_type = parse_output_type(&function);
    let function_name = function.sig.ident;
    let arguments = &function.sig.inputs;
    let argument_names = function
        .sig
        .inputs
        .iter()
        .filter_map(|input| match input {
            FnArg::Receiver(_) => None,
            FnArg::Typed(arg) => Some(&arg.pat),
        })
        .collect::<Vec<_>>();
    let statements: Vec<Stmt> = function.block.stmts;

    quote! {
        #[no_mangle]
        unsafe extern "C" fn #function_name(
            stack: &mut ffi::cps::AsyncStack,
            continue_: ffi::cps::ContinuationFunction<#output_type>,
            #arguments
        ) -> ffi::cps::Result {
            use std::{future::Future, pin::Pin, task::Poll};

            type OutputFuture = Pin<Box<dyn Future<Output = #output_type>>>;

            async fn create_future(#arguments) -> #output_type {
                #(#statements);*
            }

            let mut future: OutputFuture = Box::pin(create_future(#(#argument_names),*));

            unsafe extern "C" fn poll(
                stack: &mut #crate_path::cps::AsyncStack,
                continue_: #crate_path::cps::ContinuationFunction<#output_type>,
                future: OutputFuture,
            ) -> #crate_path::cps::Result {
                match future.as_mut().poll(stack.context().unwrap()) {
                    Poll::Ready(value) => continue_(stack, value),
                    Poll::Pending => {
                        stack.suspend(resume, continue_, future);
                        #crate_path::cps::Result::new()
                    }
                }
            }

            unsafe extern "C" fn resume(
                stack: &mut #crate_path::cps::AsyncStack,
                continue_: #crate_path::cps::ContinuationFunction<#output_type>,
            ) -> #crate_path::cps::Result {
                poll(stack, continue_, stack.restore().unwrap())
            }

            poll(stack, continue_, future)
        }
    }
    .into()
}

fn generate_sync_function(function: &ItemFn) -> TokenStream {
    let function_name = &function.sig.ident;
    let arguments = &function.sig.inputs;
    let statements: &[Stmt] = &function.block.stmts;
    let output_type = parse_output_type(&function);

    quote! {
        #[no_mangle]
        extern "C" fn #function_name(#arguments) -> #output_type {
            #(#statements);*
        }
    }
    .into()
}

fn parse_output_type(function: &ItemFn) -> Box<Type> {
    match &function.sig.output {
        ReturnType::Default => parse_quote!(ffi::None),
        ReturnType::Type(_, type_) => type_.clone(),
    }
}
