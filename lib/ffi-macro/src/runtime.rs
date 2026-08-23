use crate::{attribute_list::AttributeList, utilities::parse_crate_path};
use proc_macro::TokenStream;
use quote::quote;
use std::error::Error;
use syn::ItemFn;

pub fn generate(
    attributes: &AttributeList,
    function: &ItemFn,
) -> Result<TokenStream, Box<dyn Error>> {
    let crate_path = parse_crate_path(attributes)?;

    if function.sig.asyncness.is_none() {
        return Err("synchronous function not supported".into());
    }

    let attributes = &function.attrs;
    let visibility = &function.vis;
    let signature = &function.sig;
    let block = &function.block;

    Ok(quote! {
        #(#attributes)*
        #visibility #signature {
            #crate_path::runtime::run(async #block).await
        }
    }
    .into())
}
