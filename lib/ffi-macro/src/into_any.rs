use crate::{
    attribute_list::AttributeList,
    utilities::{generate_type_size_test, parse_crate_path, parse_string_attribute},
};
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use std::error::Error;
use syn::{Ident, ItemStruct};

pub fn generate(
    attributes: &AttributeList,
    type_: &ItemStruct,
) -> Result<TokenStream, Box<dyn Error>> {
    let crate_path = parse_crate_path(attributes)?;
    let function_name = parse_fn(attributes)?;
    let module_name = Ident::new(
        &(type_.ident.to_string().to_case(Case::Snake) + "_module"),
        type_.ident.span(),
    );
    let type_name = &type_.ident;
    let type_size_test = generate_type_size_test(type_name);

    Ok(quote! {
        #type_

        mod #module_name {
            use core::alloc::Layout;
            use super::#type_name;

            #type_size_test

            impl From<#type_name> for #crate_path::Any {
                fn from(x: #type_name) -> Self {
                    #crate_path::import!(#function_name, fn(x: #type_name) -> #crate_path::Any);

                    unsafe { #function_name(x) }.into()
                }
            }
        }
    }
    .into())
}

fn parse_fn(attributes: &AttributeList) -> Result<Ident, Box<dyn Error>> {
    parse_string_attribute(attributes, "name")?.ok_or_else(|| "missing or invalid name".into())
}
