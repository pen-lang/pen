use crate::utilities::{parse_crate_path, parse_string_attribute};
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use std::error::Error;
use syn::{parse_macro_input, AttributeArgs, Ident, ItemStruct};

pub fn generate(attributes: TokenStream, item: TokenStream) -> TokenStream {
    let attributes = parse_macro_input!(attributes as AttributeArgs);
    let type_ = parse_macro_input!(item as ItemStruct);

    match generate_type(&attributes, &type_) {
        Ok(tokens) => tokens,
        Err(message) => {
            let message = message.to_string();

            quote! { compile_error!(#message) }.into()
        }
    }
}

fn generate_type(
    attributes: &AttributeArgs,
    type_: &ItemStruct,
) -> Result<TokenStream, Box<dyn Error>> {
    let crate_path = parse_crate_path(attributes)?;
    let function_name = parse_fn(attributes)?;
    let module_name = Ident::new(
        &(type_.ident.to_string().to_case(Case::Snake) + "_module"),
        type_.ident.span(),
    );
    let type_name = &type_.ident;

    Ok(quote! {
        #type_

        mod #module_name {
            use core::alloc::Layout;
            use super::#type_name;

            #[test]
            fn type_size() {
                assert!(
                    Layout::new::<#type_name>().size() <= Layout::new::<*const u8>().size(),
                    "type size too large",
                );
            }

            impl From<#type_name> for #crate_path::Any {
                fn from(x: #type_name) -> Self {
                    #crate_path::import!(#function_name, fn(x: #type_name) -> #crate_path::BoxAny);

                    unsafe { #function_name(x) }.into()
                }
            }
        }
    }
    .into())
}

fn parse_fn(attributes: &AttributeArgs) -> Result<Ident, Box<dyn Error>> {
    parse_string_attribute(attributes, "fn")?.ok_or_else(|| "missing or invalid fn".into())
}
