use crate::utilities::parse_crate_path;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, parse_quote, AttributeArgs, FnArg, ItemFn, Path, ReturnType, Stmt, Type,
};

pub fn generate_binding(attributes: TokenStream, item: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attributes as AttributeArgs);
    let function = parse_macro_input!(item as ItemFn);

    match generate_function(&attributes, &function) {
        Ok(tokens) => tokens,
        Err(message) => quote! { compile_error!(#message) }.into(),
    }
}

fn generate_function(attributes: &AttributeArgs, function: &ItemFn) -> Result<TokenStream, String> {
    let crate_path = parse_crate_path(attributes)?;

    if function
        .sig
        .inputs
        .iter()
        .any(|input| matches!(input, FnArg::Receiver(_)))
    {
        return Err("receiver not supported".into());
    } else if function.sig.abi.is_some() {
        return Err("custom function ABI not supported".into());
    } else if !function.sig.generics.params.is_empty() {
        return Err("generic function not supported".into());
    } else if function.sig.asyncness.is_none() {
        return Ok(generate_sync_function(function, &crate_path));
    }

    let function_name = &function.sig.ident;
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
    let output_type = parse_output_type(function, &crate_path);
    let statements = parse_statements(function, &crate_path);
    let attributes = &function.attrs;

    Ok(quote! {
        #[no_mangle]
        extern "C" fn #function_name(
            stack: &mut #crate_path::cps::AsyncStack<()>,
            continue_: #crate_path::cps::ContinuationFunction<#output_type, ()>,
            #arguments
        ) {
            use core::{future::Future, pin::Pin, task::Poll};

            type OutputFuture = Pin<Box<dyn Future<Output = #output_type>>>;

            #(#attributes)*
            async fn create_future(#arguments) -> #output_type {
                #(#statements);*
            }

            let mut future: OutputFuture = Box::pin(create_future(#(#argument_names),*));

            fn poll(
                stack: &mut #crate_path::cps::AsyncStack<()>,
                continue_: #crate_path::cps::ContinuationFunction<#output_type, ()>,
                mut future: OutputFuture,
            ) {
                match future.as_mut().poll(stack.context().unwrap()) {
                    Poll::Ready(value) => {
                        stack.trampoline(continue_, value).unwrap();
                    }
                    Poll::Pending => {
                        stack.suspend(resume, continue_, future).unwrap();
                    }
                }
            }

            fn resume(
                stack: &mut #crate_path::cps::AsyncStack<()>,
                continue_: #crate_path::cps::ContinuationFunction<#output_type, ()>,
            ) {
                let future = stack.restore().unwrap();
                poll(stack, continue_, future)
            }

            poll(stack, continue_, future)
        }
    }
    .into())
}

fn generate_sync_function(function: &ItemFn, crate_path: &Path) -> TokenStream {
    let function_name = &function.sig.ident;
    let arguments = &function.sig.inputs;
    let output_type = parse_output_type(function, crate_path);
    let statements = parse_statements(function, crate_path);
    let attributes = &function.attrs;

    quote! {
        #(#attributes)*
        #[no_mangle]
        extern "C" fn #function_name(#arguments) -> #output_type {
            #(#statements);*
        }
    }
    .into()
}

fn parse_output_type(function: &ItemFn, crate_path: &Path) -> Box<Type> {
    match &function.sig.output {
        ReturnType::Default => parse_quote!(#crate_path::None),
        ReturnType::Type(_, type_) => type_.clone(),
    }
}

fn parse_statements(function: &ItemFn, crate_path: &Path) -> impl Iterator<Item = Stmt> {
    function.block.stmts.clone().into_iter().chain(
        if matches!(function.sig.output, ReturnType::Default) {
            Some(Stmt::Expr(parse_quote!(#crate_path::None::new())))
        } else {
            None
        },
    )
}
