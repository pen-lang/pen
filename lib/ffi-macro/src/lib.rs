mod any;
mod bindgen;
mod into_any;
mod utilities;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn, ItemStruct};

#[proc_macro_attribute]
pub fn bindgen(attributes: TokenStream, item: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attributes as AttributeArgs);
    let function = parse_macro_input!(item as ItemFn);

    convert_result(bindgen::generate(&attributes, &function))
}

#[proc_macro_attribute]
pub fn any(attributes: TokenStream, item: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attributes as AttributeArgs);
    let type_ = parse_macro_input!(item as ItemStruct);

    convert_result(any::generate(&attributes, &type_))
}

#[proc_macro_attribute]
pub fn into_any(attributes: TokenStream, item: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attributes as AttributeArgs);
    let type_ = parse_macro_input!(item as ItemStruct);

    convert_result(into_any::generate(&attributes, &type_))
}

fn convert_result(result: Result<TokenStream, impl ToString>) -> TokenStream {
    result.unwrap_or_else(|error| {
        let message = error.to_string();

        quote! { compile_error!(#message) }.into()
    })
}
