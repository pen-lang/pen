use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, AttributeArgs, FnArg, ItemFn, Meta, NestedMeta, ReturnType,
    Stmt,
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
    } else if function.sig.asyncness.is_none() {
        return quote! { compile_error!("non-async function not implemented yet") }.into();
    } else if !function.sig.generics.params.is_empty() {
        return quote! { compile_error!("generic function not allowed") }.into();
    }

    let crate_name = attributes
        .iter()
        .find_map(|attribute| match attribute {
            NestedMeta::Meta(Meta::NameValue(name_value)) => {
                if name_value.path.is_ident("serde") {
                    Some(name_value.lit.clone())
                } else {
                    None
                }
            }
            _ => None,
        })
        .unwrap_or(parse_quote!(ffi));

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
                stack: &mut #crate_name::cps::AsyncStack,
                continue_: #crate_name::cps::ContinuationFunction<#output_type>,
            ) -> #crate_name::cps::Result {
                let mut future: OutputFuture = stack.restore().unwrap();

                match future.as_mut().poll(stack.context().unwrap()) {
                    Poll::Ready(value) => continue_(stack, value),
                    Poll::Pending => {
                        stack.suspend(resume, continue_, future);
                        #crate_name::cps::Result::new()
                    }
                }
            }

            match future.as_mut().poll(stack.context().unwrap()) {
                Poll::Ready(value) => continue_(stack, value),
                Poll::Pending => {
                    stack.suspend(resume, continue_, future);
                    #crate_name::cps::Result::new()
                }
            }
        }
    }
    .into()
}
