use crate::utilities::parse_crate_path;
use proc_macro::TokenStream;
use quote::quote;
use std::error::Error;
use syn::{
    parse_quote, Attribute, FnArg, Ident, ItemFn, Pat, Path, PathArguments, PathSegment,
    ReturnType, Type,
};

pub fn generate(
    attributes: &[Attribute],
    function: &ItemFn,
) -> Result<TokenStream, Box<dyn Error>> {
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
        return generate_sync_function(function, &crate_path);
    }

    let function_name = &function.sig.ident;
    let arguments = &function.sig.inputs;
    let moved_arguments = generate_moved_arguments(function)?;
    let wrapper_arguments = if is_default_return_type(function) {
        function.sig.inputs.iter().cloned().collect()
    } else {
        moved_arguments.clone()
    };
    let argument_names = generate_argument_names(function)?;
    let output_type = generate_output_type(function, &crate_path);
    let attributes = &function.attrs;

    let statements = &function.block.stmts;
    let statements = if is_default_return_type(function) {
        quote! {
            #(#statements);*;

            #crate_path::None::default()
        }
    } else {
        let original_output_type = &function.sig.output;

        quote! {
            async fn run(#arguments) #original_output_type {
                #(#statements);*
            }

            run(#(#argument_names),*).await.into()
        }
    };

    Ok(quote! {
        #[no_mangle]
        extern "C" fn #function_name(
            stack: &mut #crate_path::cps::AsyncStack<()>,
            continue_: #crate_path::cps::ContinuationFunction<#output_type, ()>,
            #(#moved_arguments),*
        ) {
            use core::{future::Future, pin::Pin, task::Poll};

            type OutputFuture = Pin<Box<dyn Future<Output = #output_type>>>;

            #(#attributes)*
            async fn create_future(#(#wrapper_arguments),*) -> #output_type {
                #statements
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

fn generate_sync_function(
    function: &ItemFn,
    crate_path: &Path,
) -> Result<TokenStream, Box<dyn Error>> {
    let function_name = &function.sig.ident;
    let arguments = &function.sig.inputs;
    let argument_names = generate_argument_names(function)?;
    let output_type = generate_output_type(function, crate_path);
    let attributes = &function.attrs;

    let statements = &function.block.stmts;

    Ok(if is_default_return_type(function) {
        quote! {
            #(#attributes)*
            #[no_mangle]
            extern "C" fn #function_name(#arguments) -> #output_type {
                #(#statements);*;

                #crate_path::None::default()
            }
        }
    } else {
        let moved_arguments = generate_moved_arguments(function)?;
        let original_output_type = &function.sig.output;

        quote! {
            #(#attributes)*
            #[no_mangle]
            extern "C" fn #function_name(#(#moved_arguments),*) -> #output_type {
                fn run(#arguments) #original_output_type {
                    #(#statements);*
                }

                run(#(#argument_names),*).into()
            }
        }
    }
    .into())
}

fn generate_output_type(function: &ItemFn, crate_path: &Path) -> Box<Type> {
    match &function.sig.output {
        ReturnType::Default => parse_quote!(#crate_path::None),
        ReturnType::Type(_, type_) => match type_.as_ref() {
            Type::Path(path) => {
                if let Some(PathSegment {
                    ident,
                    arguments: PathArguments::AngleBracketed(_),
                }) = path.path.segments.first()
                {
                    let result_ident: Ident = parse_quote!(Result);

                    if ident == &result_ident {
                        parse_quote!(#crate_path::extra::Result)
                    } else {
                        type_.clone()
                    }
                } else {
                    type_.clone()
                }
            }
            _ => type_.clone(),
        },
    }
}

fn generate_argument_names(function: &ItemFn) -> Result<Vec<&Ident>, String> {
    function
        .sig
        .inputs
        .iter()
        .filter_map(|input| match input {
            FnArg::Receiver(_) => None,
            FnArg::Typed(arg) => Some(match arg.pat.as_ref() {
                Pat::Ident(ident) => Ok(&ident.ident),
                _ => Err("unsupported argument format".into()),
            }),
        })
        .collect::<Result<Vec<_>, _>>()
}

fn generate_moved_arguments(function: &ItemFn) -> Result<Vec<FnArg>, Box<dyn Error>> {
    function
        .sig
        .inputs
        .iter()
        .map(|input| match input {
            FnArg::Receiver(_) => Ok(input.clone()),
            FnArg::Typed(arg) => match arg.pat.as_ref() {
                Pat::Ident(ident) => {
                    let identifier = &ident.ident;
                    let type_ = &arg.ty;

                    Ok(parse_quote!(#identifier: #type_))
                }
                _ => Err("unsupported argument format".into()),
            },
        })
        .collect::<Result<Vec<_>, _>>()
}

fn is_default_return_type(function: &ItemFn) -> bool {
    matches!(function.sig.output, ReturnType::Default)
}
