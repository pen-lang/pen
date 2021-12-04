use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, FnArg, ItemFn, ReturnType, Stmt};

#[proc_macro_attribute]
pub fn bindgen(_attributes: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);

    if function
        .sig
        .inputs
        .iter()
        .any(|input| matches!(input, FnArg::Receiver(_)))
    {
        return quote! { compile_error!("receiver not allowed") }.into();
    } else if function.sig.asyncness.is_none() {
        return quote! { compile_error!("non-async function not implemented yet") }.into();
    } else if !function.sig.generics.params.is_empty() {
        return quote! { compile_error!("generic function not allowed") }.into();
    }

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
    let output_type = match function.sig.output {
        ReturnType::Default => parse_quote!(ffi::None),
        ReturnType::Type(_, type_) => type_,
    };

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

            unsafe extern "C" fn resume(
                stack: &mut ffi::cps::AsyncStack,
                continue_: ffi::cps::ContinuationFunction<#output_type>,
            ) -> ffi::cps::Result {
                let mut future: OutputFuture = stack.restore().unwrap();

                match future.as_mut().poll(stack.context().unwrap()) {
                    Poll::Ready(value) => continue_(stack, value),
                    Poll::Pending => {
                        stack.suspend(resume, continue_, future);
                        ffi::cps::Result::new()
                    }
                }
            }

            match future.as_mut().poll(stack.context().unwrap()) {
                Poll::Ready(value) => continue_(stack, value),
                Poll::Pending => {
                    stack.suspend(resume, continue_, future);
                    ffi::cps::Result::new()
                }
            }
        }
    }
    .into()
}
